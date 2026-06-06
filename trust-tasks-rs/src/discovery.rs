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

use std::collections::{BTreeMap, BTreeSet};
use std::iter::FromIterator;

use crate::payload::Payload;
use crate::specs::trust_task_discovery::v0_1 as wire;
use crate::type_uri::TypeUri;

/// Optional framework version this registry advertises in its discovery
/// responses. Per SPEC §4.5.1 + §5.2 + the `trust-task-discovery/0.1` spec,
/// the response payload's `frameworkVersion` is OPTIONAL in 0.1 and
/// RECOMMENDED in future revisions. The default is `"0.2"` because that's
/// the framework version this crate targets; callers MAY override or
/// clear it.
const DEFAULT_FRAMEWORK_VERSION: &str = "0.2";

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
    /// Per-URI `ext` namespaces this responder requires on inbound
    /// documents. Populated via [`Self::with_required_ext`] /
    /// [`Self::require_ext`]. URIs in this map MUST also appear in
    /// `type_uris`. When `respond_to` emits the response, URIs with an
    /// entry here use the expanded form so producers see the requirement
    /// before the wire trip.
    required_ext: BTreeMap<String, BTreeSet<String>>,
    /// MAJOR.MINOR framework version advertised in the response payload's
    /// `frameworkVersion` field. `None` suppresses the field (caller opted
    /// out); `Some` is emitted verbatim. Defaults to the crate's target.
    framework_version: Option<String>,
}

impl DiscoveryRegistry {
    /// New empty registry that advertises `frameworkVersion = "0.1"` in
    /// responses. Use [`Self::framework_version`] / [`Self::no_framework_version`]
    /// to change or suppress the advertised value.
    pub fn new() -> Self {
        Self {
            framework_version: Some(DEFAULT_FRAMEWORK_VERSION.to_string()),
            ..Self::default()
        }
    }

    /// Override the `frameworkVersion` advertised in the response payload.
    /// Most consumers should leave this at the default — the registry
    /// emits the framework version this crate targets.
    pub fn framework_version(mut self, version: impl Into<String>) -> Self {
        self.framework_version = Some(version.into());
        self
    }

    /// Suppress the `frameworkVersion` field in the response payload.
    /// The field is OPTIONAL in 0.1; callers who want to remain silent
    /// about their framework version (e.g. for privacy reasons per
    /// SPEC §11.5) can opt out with this.
    pub fn no_framework_version(mut self) -> Self {
        self.framework_version = None;
        self
    }

    /// Advertise that inbound documents of `uri` MUST carry the given
    /// reverse-DNS `ext` namespaces (SPEC §4.5.1, §7.2). The registry
    /// surfaces these in `respond_to()`'s expanded `supportedTypes` entry
    /// for `uri` so a producer sees the requirement before the wire trip.
    ///
    /// `uri` is also registered if not already present; calling this
    /// method is equivalent to calling [`Self::with_type_uri`] for the
    /// same URI plus recording the namespace requirements.
    ///
    /// Repeated calls for the same URI union their namespace sets.
    pub fn with_required_ext<I, S>(mut self, uri: TypeUri, namespaces: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let bare = uri.bare().to_string();
        self.type_uris.insert(bare.clone());
        let entry = self.required_ext.entry(bare).or_default();
        entry.extend(namespaces.into_iter().map(Into::into));
        self
    }

    /// Mutating equivalent of [`Self::with_required_ext`].
    pub fn require_ext<I, S>(&mut self, uri: TypeUri, namespaces: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let bare = uri.bare().to_string();
        self.type_uris.insert(bare.clone());
        let entry = self.required_ext.entry(bare).or_default();
        entry.extend(namespaces.into_iter().map(Into::into));
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
    ///
    /// URIs with no `requiredExt` policy are emitted in shorthand form
    /// ([`wire::ResponseSupportedTypesItem::Uri`]); URIs with a policy
    /// declared via [`Self::with_required_ext`] / [`Self::require_ext`]
    /// are emitted in expanded form
    /// ([`wire::ResponseSupportedTypesItem::Object`]) carrying the
    /// `requiredExt` array.
    pub fn respond_to(&self, query: &wire::Payload) -> wire::Response {
        // Generated `Payload` wraps each pattern in a `PayloadPatternsItem`
        // newtype; deref to &str for matching.
        let patterns: Vec<&str> = query.patterns.iter().map(|p| p.as_str()).collect();
        let supported_types: Vec<wire::ResponseSupportedTypesItem> = self
            .type_uris
            .iter()
            .filter(|uri| match parse_slug(uri) {
                Some(slug) => query_matches(&patterns, slug),
                None => false,
            })
            .map(|uri| self.entry_for(uri))
            .collect();

        // A `framework_version` override that doesn't match the MAJOR.MINOR
        // pattern is omitted rather than panicking on this caller-supplied
        // value — building a discovery response (which is advisory) must not
        // abort on a malformed setter argument.
        let framework_version = self
            .framework_version
            .as_deref()
            .and_then(|v| v.parse::<wire::ResponseFrameworkVersion>().ok());

        wire::Response {
            supported_types,
            framework_version,
        }
    }

    fn entry_for(&self, uri: &str) -> wire::ResponseSupportedTypesItem {
        match self.required_ext.get(uri) {
            Some(namespaces) if !namespaces.is_empty() => {
                // Namespaces that don't match the reverse-DNS pattern are
                // skipped rather than panicking on caller-supplied input.
                let required_ext: Vec<wire::ResponseSupportedTypesItemObjectRequiredExtItem> =
                    namespaces.iter().filter_map(|ns| ns.parse().ok()).collect();
                wire::ResponseSupportedTypesItem::Object {
                    type_: uri.to_string(),
                    required_ext: Some(required_ext),
                }
            }
            _ => wire::ResponseSupportedTypesItem::Uri(uri.to_string()),
        }
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
            uris_in(&response),
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

        // frameworkVersion defaults to the framework version this crate targets.
        let response = registry.respond_to(&wire::Payload { patterns: vec![] });
        assert_eq!(
            response.framework_version.as_ref().map(|v| v.to_string()),
            Some("0.2".to_string())
        );
    }

    #[test]
    fn no_framework_version_suppresses_field() {
        let registry = DiscoveryRegistry::new().no_framework_version();
        let response = registry.respond_to(&wire::Payload { patterns: vec![] });
        assert!(response.framework_version.is_none());
    }

    #[test]
    fn override_framework_version_is_emitted_verbatim() {
        let registry = DiscoveryRegistry::new().framework_version("0.2");
        let response = registry.respond_to(&wire::Payload { patterns: vec![] });
        assert_eq!(
            response.framework_version.as_ref().map(|v| v.to_string()),
            Some("0.2".to_string())
        );
    }

    #[test]
    fn malformed_framework_version_override_is_omitted_not_panicked() {
        // A caller-supplied override that doesn't match MAJOR.MINOR must not
        // panic when the response is built (M6); the field is simply omitted.
        let registry = DiscoveryRegistry::new().framework_version("not-a-version");
        let response = registry.respond_to(&wire::Payload { patterns: vec![] });
        assert!(response.framework_version.is_none());
    }

    #[test]
    fn with_required_ext_advertises_namespace_policy_in_expanded_form() {
        let grant_uri = TypeUri::canonical("acl/grant", 0, 1).unwrap();
        let registry = DiscoveryRegistry::new()
            .with::<crate::specs::acl::revoke::v0_1::Payload>()
            .with_required_ext(grant_uri, ["vnd.affinidi.webvh"]);

        let response = registry.respond_to(&wire::Payload { patterns: vec![] });

        // acl/grant carries the requiredExt annotation in expanded form.
        let grant_entry = response
            .supported_types
            .iter()
            .find(|e| uri_of(e) == "https://trusttasks.org/spec/acl/grant/0.1")
            .expect("acl/grant entry present");
        match grant_entry {
            wire::ResponseSupportedTypesItem::Object { required_ext, .. } => {
                let namespaces: Vec<String> = required_ext
                    .as_ref()
                    .expect("requiredExt populated")
                    .iter()
                    .map(|n| n.to_string())
                    .collect();
                assert_eq!(namespaces, vec!["vnd.affinidi.webvh".to_string()]);
            }
            other => panic!("expected expanded Object form, got {other:?}"),
        }

        // acl/revoke had no policy declared → shorthand string form.
        let revoke_entry = response
            .supported_types
            .iter()
            .find(|e| uri_of(e) == "https://trusttasks.org/spec/acl/revoke/0.1")
            .expect("acl/revoke entry present");
        assert!(matches!(
            revoke_entry,
            wire::ResponseSupportedTypesItem::Uri(_)
        ));
    }

    fn uris_in(response: &wire::Response) -> Vec<&str> {
        response.supported_types.iter().map(uri_of).collect()
    }

    fn uri_of(entry: &wire::ResponseSupportedTypesItem) -> &str {
        match entry {
            wire::ResponseSupportedTypesItem::Uri(s) => s.as_str(),
            wire::ResponseSupportedTypesItem::Object { type_, .. } => type_.as_str(),
        }
    }
}
