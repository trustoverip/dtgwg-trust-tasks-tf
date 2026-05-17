//! Helpers for the `trust-task-discovery/0.1` exchange.
//!
//! This module is the framework crate's companion to the registry spec
//! at `specs/trust-task-discovery/0.1`. It supplies:
//!
//! * [`match_slug`] / [`query_matches`] — the slug-glob matcher in
//!   primitive form, useful when integrating discovery into an existing
//!   server dispatcher.
//! * [`DiscoveryRegistry`] — a small builder that registers the
//!   [`Payload`] types a server supports and answers inbound discovery
//!   queries with the matching subset.
//!
//! ```rust,ignore
//! use trust_tasks_rs::{
//!     discovery::DiscoveryRegistry,
//!     specs::{acl::grant, acl::revoke, trust_task_discovery::v0_1 as discovery},
//! };
//!
//! let registry = DiscoveryRegistry::new()
//!     .with::<grant::v0_1::Payload>()
//!     .with::<revoke::v0_1::Payload>();
//!
//! // Query received off the wire:
//! let query = discovery::Payload {
//!     patterns: vec!["acl/*".parse().unwrap()],
//! };
//!
//! let response = registry.respond_to(&query);
//! // response.supported_types now lists the bare Type URIs of every
//! // registered Payload whose slug matches one of the query patterns.
//! ```

use std::collections::BTreeSet;
use std::iter::FromIterator;

use crate::payload::Payload;
use crate::specs::trust_task_discovery::v0_1 as wire;
use crate::type_uri::TypeUri;

/// Match a single glob `pattern` against a `slug`, per the
/// `trust-task-discovery/0.1` pattern grammar:
///
/// * `"*"` matches every slug.
/// * `"<prefix>/*"` matches every slug starting with `<prefix>/`.
/// * any other pattern is an exact slug match (wildcards in the
///   interior are treated literally and therefore never match).
///
/// Returns `true` on a match.
pub fn match_slug(pattern: &str, slug: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix("/*") {
        // Reject pathological pattern like just "/" or "/*".
        if prefix.is_empty() {
            return false;
        }
        let mut full_prefix = String::with_capacity(prefix.len() + 1);
        full_prefix.push_str(prefix);
        full_prefix.push('/');
        return slug.starts_with(&full_prefix);
    }
    pattern == slug
}

/// `true` if any pattern in `patterns` matches `slug`. An empty
/// `patterns` slice is treated as `["*"]` — every slug matches —
/// per the [`trust-task-discovery/0.1`](../specs/trust-task-discovery)
/// MUST in its §Conformance.
pub fn query_matches<S: AsRef<str>>(patterns: &[S], slug: &str) -> bool {
    if patterns.is_empty() {
        return true;
    }
    patterns.iter().any(|p| match_slug(p.as_ref(), slug))
}

/// Builds responses to `trust-task-discovery/0.1` queries from a list
/// of registered Type URIs.
///
/// Use `with::<P: Payload>` to register a generated payload type — the
/// registry pulls the Type URI off the trait's `TYPE_URI` constant. For
/// hand-built types or for one-off URIs, use [`with_type_uri`].
///
/// The registry de-duplicates entries and sorts output lexicographically
/// for stable wire bytes.
///
/// [`with_type_uri`]: Self::with_type_uri
#[derive(Debug, Clone, Default)]
pub struct DiscoveryRegistry {
    type_uris: BTreeSet<String>,
}

impl DiscoveryRegistry {
    /// New empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a [`Payload`] type by reading its `TYPE_URI` constant.
    /// The bare URI (no `#request` / `#response` fragment) is stored;
    /// per SPEC §11.3 the response always lists bare URIs.
    pub fn with<P: Payload>(self) -> Self {
        let uri = P::type_uri();
        self.with_type_uri(uri)
    }

    /// Register a `TypeUri` directly. Useful for callers that hold a
    /// `TypeUri` without a corresponding `Payload`-implementing type
    /// (e.g. a server that delegates a slug to a downstream consumer
    /// without compiling against the typed payload itself).
    pub fn with_type_uri(mut self, uri: TypeUri) -> Self {
        self.type_uris.insert(uri.bare().to_string());
        self
    }

    /// Mutating equivalent of [`with_type_uri`](Self::with_type_uri).
    pub fn register(&mut self, uri: TypeUri) {
        self.type_uris.insert(uri.bare().to_string());
    }

    /// Mutating registration by `Payload`-implementing type.
    pub fn register_payload<P: Payload>(&mut self) {
        self.register(P::type_uri());
    }

    /// Insert a Type URI from its string form. Caller's responsibility to
    /// pass a well-formed bare Type URI — the registry stores it as-is
    /// without re-parsing. Useful for integrations that already hold
    /// canonical URI strings (e.g. server routing tables that key on
    /// `TypeUri::for_routing().to_string()`).
    pub fn register_str(&mut self, uri: impl Into<String>) {
        self.type_uris.insert(uri.into());
    }

    /// Builder-flavored [`register_str`](Self::register_str).
    pub fn with_str(mut self, uri: impl Into<String>) -> Self {
        self.register_str(uri);
        self
    }

    /// Bare Type URIs the registry currently holds, lexicographically
    /// sorted.
    pub fn supported_types(&self) -> Vec<&str> {
        self.type_uris.iter().map(String::as_str).collect()
    }

    /// Build a response to `query`, listing every registered Type URI
    /// whose slug matches at least one of the query's patterns. Absent
    /// or empty patterns produce the full list.
    pub fn respond_to(&self, query: &wire::Payload) -> wire::Response {
        // Generated `Payload` wraps each pattern in a `PayloadPatternsItem`
        // newtype; deref to &str for matching.
        let patterns: Vec<&str> = query.patterns.iter().map(|p| p.as_str()).collect();
        let supported_types: Vec<String> = self
            .type_uris
            .iter()
            .filter(|uri| match parse_slug(uri) {
                Some(slug) => query_matches(&patterns, slug),
                None => false,
            })
            .cloned()
            .collect();
        wire::Response { supported_types }
    }
}

impl<S: Into<String>> FromIterator<S> for DiscoveryRegistry {
    fn from_iter<I: IntoIterator<Item = S>>(iter: I) -> Self {
        let mut registry = Self::new();
        for uri in iter {
            registry.register_str(uri);
        }
        registry
    }
}

/// Extract the slug from a registered Type URI. Returns `None` for
/// inputs that don't parse as a Type URI (which can't happen for
/// URIs we registered through `with_type_uri` since they came from a
/// `TypeUri`).
fn parse_slug(uri: &str) -> Option<&str> {
    // The Type URI shape is `https://.../spec/<slug>/<MAJOR.MINOR>`.
    // We don't re-parse via `TypeUri::from_str` because we already know
    // the URIs are well-formed; cheap string slicing is fine.
    let after_spec = uri.split_once("/spec/")?.1;
    let last_slash = after_spec.rfind('/')?;
    Some(&after_spec[..last_slash])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn star_matches_anything() {
        assert!(match_slug("*", "acl/grant"));
        assert!(match_slug("*", "kyc-handoff"));
        assert!(match_slug("*", "trust-task-discovery"));
    }

    #[test]
    fn prefix_wildcard_matches_descendants() {
        assert!(match_slug("acl/*", "acl/grant"));
        assert!(match_slug("acl/*", "acl/revoke"));
        assert!(match_slug("acl/*", "acl/grant/sub"));
    }

    #[test]
    fn prefix_wildcard_does_not_match_bare_prefix() {
        // SPEC §11.2 — `acl/*` requires the trailing slash; `acl` alone
        // does not satisfy the pattern.
        assert!(!match_slug("acl/*", "acl"));
        assert!(!match_slug("acl/*", "aclx"));
        assert!(!match_slug("acl/*", "kyc-handoff"));
    }

    #[test]
    fn exact_pattern_matches_exact_slug_only() {
        assert!(match_slug("kyc-handoff", "kyc-handoff"));
        assert!(!match_slug("kyc-handoff", "kyc-handoff/v2"));
        assert!(!match_slug("kyc-handoff", "kyc"));
    }

    #[test]
    fn interior_wildcards_are_treated_literally() {
        // No slug can contain `*`, so a pattern with an interior `*`
        // never matches anything.
        assert!(!match_slug("acl/*/grant", "acl/grant"));
        assert!(!match_slug("a*b", "ab"));
    }

    #[test]
    fn empty_patterns_match_everything() {
        let patterns: &[&str] = &[];
        assert!(query_matches(patterns, "acl/grant"));
        assert!(query_matches(patterns, "anything"));
    }

    #[test]
    fn or_semantics_across_patterns() {
        let patterns = ["acl/*", "kyc-handoff"];
        assert!(query_matches(&patterns, "acl/grant"));
        assert!(query_matches(&patterns, "kyc-handoff"));
        assert!(!query_matches(&patterns, "consent/give"));
    }

    #[test]
    fn registry_dedupes_and_sorts() {
        let registry = DiscoveryRegistry::new()
            .with_type_uri(TypeUri::canonical("acl/revoke", 0, 1).unwrap())
            .with_type_uri(TypeUri::canonical("acl/grant", 0, 1).unwrap())
            .with_type_uri(TypeUri::canonical("acl/grant", 0, 1).unwrap());
        let types = registry.supported_types();
        assert_eq!(
            types,
            vec![
                "https://trusttasks.org/spec/acl/grant/0.1",
                "https://trusttasks.org/spec/acl/revoke/0.1",
            ]
        );
    }

    #[test]
    fn registry_responds_to_query_with_filtered_subset() {
        use crate::specs::acl::{change_role, grant, list, revoke, show};
        let registry = DiscoveryRegistry::new()
            .with::<grant::v0_1::Payload>()
            .with::<revoke::v0_1::Payload>()
            .with::<show::v0_1::Payload>()
            .with::<list::v0_1::Payload>()
            .with::<change_role::v0_1::Payload>();

        // ACL family — all 5 listed.
        let acl_only = wire::Payload {
            patterns: vec!["acl/*".parse().unwrap()],
        };
        let response = registry.respond_to(&acl_only);
        assert_eq!(response.supported_types.len(), 5);

        // Exact match — only one.
        let only_grant = wire::Payload {
            patterns: vec!["acl/grant".parse().unwrap()],
        };
        let response = registry.respond_to(&only_grant);
        assert_eq!(
            response.supported_types,
            vec!["https://trusttasks.org/spec/acl/grant/0.1"]
        );

        // Empty patterns → everything.
        let everything = wire::Payload { patterns: vec![] };
        let response = registry.respond_to(&everything);
        assert_eq!(response.supported_types.len(), 5);

        // No match.
        let nothing = wire::Payload {
            patterns: vec!["does-not-exist/*".parse().unwrap()],
        };
        let response = registry.respond_to(&nothing);
        assert!(response.supported_types.is_empty());
    }
}
