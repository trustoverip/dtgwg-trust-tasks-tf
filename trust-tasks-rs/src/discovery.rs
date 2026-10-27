//! Helpers for the `trust-task-discovery` exchange.
//!
//! This module is the framework crate's companion to the registry specs at
//! `specs/trust-task-discovery/`. **0.3** is the current version and the one
//! [`DiscoveryRegistry::respond_to_v0_3`] answers: its response may carry the
//! acceptance window the responder applies to `issuedAt` (SPEC §7.2 item 13),
//! at the top level and per entry. A responder answers each version in the
//! version it was asked, so the superseded 0.1 and 0.2 are still answered too
//! — `DiscoveryRegistry::respond_to` and `DiscoveryRegistry::respond_to_v0_2`
//! — until they are withdrawn; that support lives on its own in
//! `discovery/superseded.rs`. The module supplies:
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
//!     specs::{acl::grant, acl::revoke, trust_task_discovery::v0_3 as discovery},
//!     FreshnessPolicy,
//! };
//!
//! let policy = FreshnessPolicy::consequential();
//! let registry = DiscoveryRegistry::new()
//!     .with::<grant::v0_2::Payload>()
//!     .with::<revoke::v0_2::Payload>()
//!     // Advertise the window this responder enforces (300 s, 60 s skew).
//!     .acceptance_window_from(&policy);
//!
//! // Query received off the wire. Generated payload types are
//! // `#[non_exhaustive]`, so construction goes through the builder.
//! let query: discovery::Payload = discovery::Payload::builder()
//!     .patterns(vec!["acl/*".parse::<discovery::PayloadPatternsItem>()?])
//!     .try_into()?;
//!
//! let response = registry.respond_to_v0_3(&query);
//! // response.supported_types now lists the bare Type URIs of every
//! // registered Payload whose slug matches one of the query patterns.
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::iter::FromIterator;
use std::num::NonZeroU64;

use crate::freshness::FreshnessPolicy;
use crate::payload::Payload;
use crate::specs::trust_task_discovery::v0_3 as wire_v0_3;
use crate::type_uri::TypeUri;

// --- trust-task-discovery 0.1 / 0.2 (superseded) ----------------------------
// The superseded versions 0.1 and 0.2, answered until they are withdrawn. The
// withdrawal batch deletes this block and `discovery/superseded.rs`, nothing
// else in this file.
mod superseded;
// --- end trust-task-discovery 0.1 / 0.2 (superseded) ------------------------

/// Framework release this registry advertises in its discovery responses by
/// default: the newest one whose envelope this crate's generated types
/// include. Callers MAY override or clear it. It is held three-part (SPEC
/// §5.1.1) and written in whichever form the answered version admits — the
/// full value in a 0.2 or 0.3 response, its `MAJOR.MINOR` in a 0.1 response.
const DEFAULT_FRAMEWORK_VERSION: &str = "0.6.0";

/// `MAJOR.MINOR` or `MAJOR.MINOR.PATCH`, split into its numeric parts. A
/// caller-supplied value in neither form yields `None`.
fn framework_version_parts(v: &str) -> Option<(u64, u64, u64)> {
    let mut parts = v.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = match parts.next() {
        Some(p) => p.parse().ok()?,
        None => 0,
    };
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

/// Match a single glob `pattern` against a `slug`, per the
/// `trust-task-discovery` pattern grammar (unchanged since 0.1):
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
/// per the [`trust-task-discovery`](../specs/trust-task-discovery)
/// MUST in its §Conformance.
pub fn query_matches<S: AsRef<str>>(patterns: &[S], slug: &str) -> bool {
    if patterns.is_empty() {
        return true;
    }
    patterns.iter().any(|p| match_slug(p.as_ref(), slug))
}

/// The acceptance window a responder applies to `issuedAt`, in whole
/// seconds — `trust-task-discovery/0.3`'s `acceptanceWindow`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AcceptanceWindow {
    max_age_seconds: NonZeroU64,
    clock_skew_seconds: u64,
}

impl AcceptanceWindow {
    /// The window `policy` enforces, rounded **down** to whole seconds so the
    /// advertised window is never wider than the applied one (0.3 responder
    /// rule 1). `None` when the policy bounds no age, or bounds it below one
    /// second, which the wire form cannot state.
    fn of_policy(policy: &FreshnessPolicy) -> Option<Self> {
        let max_age_seconds = u64::try_from(policy.max_age?.num_seconds()).ok()?;
        Some(Self {
            max_age_seconds: NonZeroU64::new(max_age_seconds)?,
            clock_skew_seconds: u64::try_from(policy.skew.num_seconds()).unwrap_or(0),
        })
    }

    fn to_wire(self) -> wire_v0_3::AcceptanceWindow {
        wire_v0_3::AcceptanceWindow {
            max_age_seconds: self.max_age_seconds,
            clock_skew_seconds: self.clock_skew_seconds,
        }
    }
}

/// Builds responses to `trust-task-discovery` queries from a list of
/// registered Type URIs.
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
    /// `type_uris`. When a response is built, URIs with an entry here use
    /// the expanded form so producers see the requirement before the wire
    /// trip.
    required_ext: BTreeMap<String, BTreeSet<String>>,
    /// Framework release advertised in the response payload's
    /// `frameworkVersion` field. `None` suppresses the field (caller opted
    /// out). Defaults to the crate's target. Accepted two- or three-part; a
    /// two-part `M.N` is the release `M.N.0` (SPEC §5.1.1).
    framework_version: Option<String>,
    /// The response-level `acceptanceWindow` of a 0.3 response. `None` (the
    /// default) states nothing.
    acceptance_window: Option<AcceptanceWindow>,
    /// Per-URI `acceptanceWindow`s of a 0.3 response, each taking precedence
    /// over the response-level one for its entry. URIs in this map MUST also
    /// appear in `type_uris`.
    entry_acceptance_windows: BTreeMap<String, AcceptanceWindow>,
}

impl DiscoveryRegistry {
    /// New empty registry that advertises the crate's target framework
    /// release (`0.6.0`) in responses. Use [`Self::framework_version`] / [`Self::no_framework_version`]
    /// to change or suppress the advertised value.
    pub fn new() -> Self {
        Self {
            framework_version: Some(DEFAULT_FRAMEWORK_VERSION.to_string()),
            ..Self::default()
        }
    }

    /// Override the `frameworkVersion` advertised in the response payload.
    /// Most consumers should leave this at the default — the registry
    /// emits the framework version this crate targets. Pass the release
    /// three-part (`"0.6.0"`); a two-part value is read as its `.0`.
    pub fn framework_version(mut self, version: impl Into<String>) -> Self {
        self.framework_version = Some(version.into());
        self
    }

    /// Suppress the `frameworkVersion` field in the response payload.
    /// The field is OPTIONAL in every version; callers who want to remain
    /// silent about their framework version (e.g. for privacy reasons per
    /// SPEC §10.6) can opt out with this.
    pub fn no_framework_version(mut self) -> Self {
        self.framework_version = None;
        self
    }

    /// Advertise, in a `trust-task-discovery/0.3` response, the acceptance
    /// window this responder applies to the `issuedAt` of every listed Type
    /// URI that carries no window of its own: it refuses a document when
    /// `now > issuedAt + max_age_seconds + clock_skew_seconds`, and when
    /// `issuedAt > now + clock_skew_seconds`.
    ///
    /// The window MUST be no wider than the one actually enforced (0.3
    /// responder rule 1) — a wider claim makes producers deliver documents
    /// that are then refused. [`Self::acceptance_window_from`] derives it
    /// from the [`FreshnessPolicy`] the consumer applies, which keeps the two
    /// from drifting. Not stated by default; 0.1 and 0.2 responses never
    /// carry it.
    pub fn acceptance_window(
        mut self,
        max_age_seconds: NonZeroU64,
        clock_skew_seconds: u64,
    ) -> Self {
        self.acceptance_window = Some(AcceptanceWindow {
            max_age_seconds,
            clock_skew_seconds,
        });
        self
    }

    /// [`Self::acceptance_window`] read off the [`FreshnessPolicy`] this
    /// responder enforces: `max_age` and `skew`, each rounded down to whole
    /// seconds so the claim is never wider than the policy.
    ///
    /// A policy that bounds no age (`max_age: None`), or bounds it below one
    /// second, has no window the wire form can state, and leaves the registry
    /// unchanged — absence tells a discoverer it has learnt nothing, which is
    /// the truth.
    pub fn acceptance_window_from(mut self, policy: &FreshnessPolicy) -> Self {
        if let Some(window) = AcceptanceWindow::of_policy(policy) {
            self.acceptance_window = Some(window);
        }
        self
    }

    /// Stop advertising a response-level acceptance window.
    pub fn no_acceptance_window(mut self) -> Self {
        self.acceptance_window = None;
        self
    }

    /// Advertise, on `uri`'s entry of a `trust-task-discovery/0.3` response,
    /// an acceptance window that applies to documents of that Type URI in
    /// place of the response-level one — for a task the responder holds to a
    /// tighter window than the rest. The same no-wider-than-enforced rule as
    /// [`Self::acceptance_window`] applies.
    ///
    /// `uri` is also registered if not already present, as with
    /// [`Self::with_required_ext`]. A later call for the same URI replaces
    /// the earlier window.
    pub fn with_acceptance_window_for(
        mut self,
        uri: TypeUri,
        max_age_seconds: NonZeroU64,
        clock_skew_seconds: u64,
    ) -> Self {
        let bare = uri.bare().to_string();
        self.type_uris.insert(bare.clone());
        self.entry_acceptance_windows.insert(
            bare,
            AcceptanceWindow {
                max_age_seconds,
                clock_skew_seconds,
            },
        );
        self
    }

    /// Advertise that inbound documents of `uri` MUST carry the given
    /// reverse-DNS `ext` namespaces (SPEC §4.5.1, §7.2). The registry
    /// surfaces these in the expanded `supportedTypes` entry for `uri` of
    /// every response it builds, so a producer sees the requirement before
    /// the wire trip.
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
    /// per SPEC §10.3 the response always lists bare URIs.
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

    /// Build a response to a `trust-task-discovery/0.3` `query`, listing
    /// every registered Type URI whose slug matches at least one of the
    /// query's patterns. Absent or empty patterns produce the full list.
    ///
    /// A URI with no annotation is emitted in shorthand form
    /// ([`wire_v0_3::ResponseSupportedTypesItem::Uri`]). One with a
    /// `requiredExt` policy ([`Self::with_required_ext`]) or a window of its
    /// own ([`Self::with_acceptance_window_for`]) is emitted in expanded form
    /// ([`wire_v0_3::ResponseSupportedTypesItem::Object`]) carrying them.
    /// `frameworkVersion` is the full three-part release (SPEC §5.1.1), and
    /// the response-level `acceptanceWindow` is present when one was
    /// configured ([`Self::acceptance_window`],
    /// [`Self::acceptance_window_from`]).
    pub fn respond_to_v0_3(&self, query: &wire_v0_3::Payload) -> wire_v0_3::Response {
        let patterns: Vec<&str> = query.patterns.iter().map(|p| p.as_str()).collect();
        let supported_types: Vec<wire_v0_3::ResponseSupportedTypesItem> = self
            .matching(&patterns)
            .map(|uri| {
                // Namespaces that don't match the reverse-DNS pattern are
                // skipped rather than panicking on caller-supplied input; if
                // none survives, the member is left out (the schema requires
                // at least one item).
                let required_ext = self
                    .required_ext_for(uri)
                    .map(|namespaces| {
                        namespaces
                            .filter_map(|ns| ns.parse().ok())
                            .collect::<Vec<_>>()
                    })
                    .filter(|namespaces| !namespaces.is_empty());
                let acceptance_window = self
                    .entry_acceptance_windows
                    .get(uri)
                    .map(|window| window.to_wire());
                if required_ext.is_none() && acceptance_window.is_none() {
                    wire_v0_3::ResponseSupportedTypesItem::Uri(uri.to_string())
                } else {
                    wire_v0_3::ResponseSupportedTypesItem::Object {
                        type_: uri.to_string(),
                        required_ext,
                        acceptance_window,
                    }
                }
            })
            .collect();

        wire_v0_3::Response {
            supported_types,
            framework_version: self
                .three_part_framework_version()
                .and_then(|v| v.parse().ok()),
            acceptance_window: self.acceptance_window.map(AcceptanceWindow::to_wire),
        }
    }

    /// The configured framework release written three-part, a two-part value
    /// as its `.0`. A `framework_version` override that is not a version is
    /// omitted rather than panicking on this caller-supplied value — building
    /// a discovery response (which is advisory) must not abort on a malformed
    /// setter argument.
    fn three_part_framework_version(&self) -> Option<String> {
        self.framework_version
            .as_deref()
            .and_then(framework_version_parts)
            .map(|(major, minor, patch)| format!("{major}.{minor}.{patch}"))
    }

    /// Registered Type URIs whose slug matches at least one of `patterns`.
    fn matching<'a>(&'a self, patterns: &'a [&'a str]) -> impl Iterator<Item = &'a String> + 'a {
        self.type_uris
            .iter()
            .filter(move |uri| match parse_slug(uri) {
                Some(slug) => query_matches(patterns, &slug),
                None => false,
            })
    }

    /// The `requiredExt` namespaces declared for `uri`, if any.
    fn required_ext_for(&self, uri: &str) -> Option<impl Iterator<Item = &String>> {
        self.required_ext
            .get(uri)
            .filter(|namespaces| !namespaces.is_empty())
            .map(|namespaces| namespaces.iter())
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

/// Extract the slug from a registered Type URI, in any SPEC §6.1 form — the
/// URN form spells it with `:` between segments, so it is read through
/// [`crate::TypeUri`] rather than sliced. A string `register_str` accepted
/// that does not parse (the framework's own three-part
/// `…/spec/trust-task/<M.N.P>`) falls back to the path-form slicing this
/// always did. `None` when neither finds a slug.
fn parse_slug(uri: &str) -> Option<String> {
    if let Ok(t) = uri.parse::<crate::TypeUri>() {
        return Some(t.slug().to_string());
    }
    let after_spec = uri.split_once("/spec/")?.1;
    let last_slash = after_spec.rfind('/')?;
    Some(after_spec[..last_slash].to_string())
}

#[cfg(test)]
// The specimens are prebuilt modules, deprecated in favour of generating the
// tasks an application uses (trust-tasks-build).
#[allow(deprecated)]
mod tests {
    use super::*;

    fn nz(n: u64) -> NonZeroU64 {
        NonZeroU64::new(n).unwrap()
    }

    fn everything() -> wire_v0_3::Payload {
        wire_v0_3::Payload { patterns: vec![] }
    }

    fn uri_of(entry: &wire_v0_3::ResponseSupportedTypesItem) -> &str {
        match entry {
            wire_v0_3::ResponseSupportedTypesItem::Uri(s) => s.as_str(),
            wire_v0_3::ResponseSupportedTypesItem::Object { type_, .. } => type_.as_str(),
        }
    }

    fn uris_in(response: &wire_v0_3::Response) -> Vec<&str> {
        response.supported_types.iter().map(uri_of).collect()
    }

    #[test]
    fn star_matches_anything() {
        assert!(match_slug("*", "acl/grant"));
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
        // SPEC §10.2 — `acl/*` requires the trailing slash; `acl` alone
        // does not satisfy the pattern.
        assert!(!match_slug("acl/*", "acl"));
        assert!(!match_slug("acl/*", "aclx"));
        assert!(!match_slug("acl/*", "trust-task-discovery"));
    }

    #[test]
    fn exact_pattern_matches_exact_slug_only() {
        assert!(match_slug("trust-task-discovery", "trust-task-discovery"));
        assert!(!match_slug(
            "trust-task-discovery",
            "trust-task-discovery/v2"
        ));
        assert!(!match_slug("trust-task-discovery", "trust-task"));
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
        let patterns = ["acl/*", "trust-task-discovery"];
        assert!(query_matches(&patterns, "acl/grant"));
        assert!(query_matches(&patterns, "trust-task-discovery"));
        assert!(!query_matches(&patterns, "consent/give"));
    }

    #[test]
    fn registry_dedupes_and_sorts() {
        let registry = DiscoveryRegistry::new()
            .with_type_uri(TypeUri::canonical("acl/revoke", 0, 2).unwrap())
            .with_type_uri(TypeUri::canonical("acl/grant", 0, 2).unwrap())
            .with_type_uri(TypeUri::canonical("acl/grant", 0, 2).unwrap());
        let types = registry.supported_types();
        assert_eq!(
            types,
            vec![
                "https://trusttasks.org/spec/acl/grant/0.2",
                "https://trusttasks.org/spec/acl/revoke/0.2",
            ]
        );
    }

    #[cfg(feature = "acl")] // its specimens are acl payloads
    #[test]
    fn registry_responds_to_query_with_filtered_subset() {
        use crate::specs::acl::{change_role, grant, list, revoke, show};
        let registry = DiscoveryRegistry::new()
            .with::<grant::v0_2::Payload>()
            .with::<revoke::v0_2::Payload>()
            .with::<show::v0_2::Payload>()
            .with::<list::v0_3::Payload>()
            .with::<change_role::v0_2::Payload>()
            .with::<wire_v0_3::Payload>();

        // ACL family — all 5 listed, discovery itself not.
        let acl_only = wire_v0_3::Payload {
            patterns: vec!["acl/*".parse().unwrap()],
        };
        let response = registry.respond_to_v0_3(&acl_only);
        assert_eq!(response.supported_types.len(), 5);

        // Exact match — only one.
        let only_grant = wire_v0_3::Payload {
            patterns: vec!["acl/grant".parse().unwrap()],
        };
        let response = registry.respond_to_v0_3(&only_grant);
        assert_eq!(
            uris_in(&response),
            vec!["https://trusttasks.org/spec/acl/grant/0.2"]
        );

        // Empty patterns → everything.
        let response = registry.respond_to_v0_3(&everything());
        assert_eq!(response.supported_types.len(), 6);

        // No match.
        let nothing = wire_v0_3::Payload {
            patterns: vec!["does-not-exist/*".parse().unwrap()],
        };
        let response = registry.respond_to_v0_3(&nothing);
        assert!(response.supported_types.is_empty());

        // frameworkVersion defaults to the framework release this crate
        // targets, three-part; no acceptance window unless configured.
        let response = registry.respond_to_v0_3(&everything());
        assert_eq!(
            response.framework_version.as_ref().map(|v| v.to_string()),
            Some("0.6.0".to_string())
        );
        assert!(response.acceptance_window.is_none());
    }

    #[test]
    fn no_framework_version_suppresses_field() {
        let registry = DiscoveryRegistry::new().no_framework_version();
        let response = registry.respond_to_v0_3(&everything());
        assert!(response.framework_version.is_none());
    }

    #[test]
    fn a_patch_release_is_written_verbatim() {
        let registry = DiscoveryRegistry::new().framework_version("0.6.1");
        let response = registry.respond_to_v0_3(&everything());
        assert_eq!(
            response.framework_version.map(|v| v.to_string()),
            Some("0.6.1".into())
        );
    }

    #[test]
    fn a_two_part_override_is_written_as_its_dot_zero() {
        // SPEC §5.1.1: a two-part M.N is the release M.N.0.
        let registry = DiscoveryRegistry::new().framework_version("0.5");
        let response = registry.respond_to_v0_3(&everything());
        assert_eq!(
            response.framework_version.map(|v| v.to_string()),
            Some("0.5.0".into())
        );
    }

    #[test]
    fn malformed_framework_version_override_is_omitted_not_panicked() {
        // A caller-supplied override that is not a version must not panic
        // when the response is built (M6); the field is simply omitted.
        let registry = DiscoveryRegistry::new().framework_version("not-a-version");
        assert!(registry
            .respond_to_v0_3(&everything())
            .framework_version
            .is_none());
    }

    #[test]
    fn with_required_ext_advertises_namespace_policy_in_expanded_form() {
        let grant_uri = TypeUri::canonical("acl/grant", 0, 2).unwrap();
        let registry = DiscoveryRegistry::new()
            .with_type_uri(TypeUri::canonical("acl/revoke", 0, 2).unwrap())
            .with_required_ext(grant_uri, ["vnd.affinidi.webvh"]);

        let response = registry.respond_to_v0_3(&everything());

        match &response.supported_types[0] {
            wire_v0_3::ResponseSupportedTypesItem::Object {
                type_,
                required_ext,
                acceptance_window,
            } => {
                assert_eq!(type_, "https://trusttasks.org/spec/acl/grant/0.2");
                let namespaces: Vec<String> = required_ext
                    .as_ref()
                    .expect("requiredExt populated")
                    .iter()
                    .map(|n| n.to_string())
                    .collect();
                assert_eq!(namespaces, vec!["vnd.affinidi.webvh".to_string()]);
                assert!(acceptance_window.is_none());
            }
            other => panic!("expected expanded Object form, got {other:?}"),
        }
        // acl/revoke had no annotation → shorthand string form.
        assert!(matches!(
            &response.supported_types[1],
            wire_v0_3::ResponseSupportedTypesItem::Uri(uri)
                if uri == "https://trusttasks.org/spec/acl/revoke/0.2"
        ));
    }

    #[test]
    fn a_required_ext_with_no_valid_namespace_is_left_out() {
        // `requiredExt` has `minItems: 1`; an all-malformed set must not
        // become an empty array.
        let registry = DiscoveryRegistry::new().with_required_ext(
            TypeUri::canonical("acl/grant", 0, 2).unwrap(),
            ["Not A Namespace"],
        );
        let response = registry.respond_to_v0_3(&everything());
        assert!(matches!(
            &response.supported_types[0],
            wire_v0_3::ResponseSupportedTypesItem::Uri(_)
        ));
    }

    #[test]
    fn the_response_level_acceptance_window_is_carried() {
        let registry = DiscoveryRegistry::new()
            .with_type_uri(TypeUri::canonical("acl/list", 0, 3).unwrap())
            .acceptance_window(nz(600), 60);
        let response = registry.respond_to_v0_3(&everything());
        let window = response.acceptance_window.expect("window stated");
        assert_eq!(window.max_age_seconds.get(), 600);
        assert_eq!(window.clock_skew_seconds, 60);
        // An entry with no window of its own stays shorthand.
        assert!(matches!(
            &response.supported_types[0],
            wire_v0_3::ResponseSupportedTypesItem::Uri(_)
        ));

        let cleared = registry.no_acceptance_window();
        assert!(cleared
            .respond_to_v0_3(&everything())
            .acceptance_window
            .is_none());
    }

    #[test]
    fn a_per_entry_window_is_carried_in_expanded_form_beside_the_response_level_one() {
        // The 0.3 spec's own example: 600 s / 60 s for everything, 120 s /
        // 30 s for acl/grant.
        let registry = DiscoveryRegistry::new()
            .with_type_uri(TypeUri::canonical("acl/list", 0, 3).unwrap())
            .acceptance_window(nz(600), 60)
            .with_acceptance_window_for(
                TypeUri::canonical("acl/grant", 0, 2).unwrap(),
                nz(120),
                30,
            );
        let response = registry.respond_to_v0_3(&everything());

        let window = response.acceptance_window.as_ref().unwrap();
        assert_eq!(
            (window.max_age_seconds.get(), window.clock_skew_seconds),
            (600, 60)
        );
        match &response.supported_types[0] {
            wire_v0_3::ResponseSupportedTypesItem::Object {
                type_,
                acceptance_window: Some(window),
                required_ext: None,
            } => {
                assert_eq!(type_, "https://trusttasks.org/spec/acl/grant/0.2");
                assert_eq!(
                    (window.max_age_seconds.get(), window.clock_skew_seconds),
                    (120, 30)
                );
            }
            other => panic!("expected an expanded entry with a window, got {other:?}"),
        }
        assert_eq!(
            uris_in(&response),
            vec![
                "https://trusttasks.org/spec/acl/grant/0.2",
                "https://trusttasks.org/spec/acl/list/0.3",
            ]
        );
    }

    #[test]
    fn the_window_is_read_off_the_freshness_policy_never_wider() {
        // `consequential()` is five minutes with sixty seconds of skew.
        let registry =
            DiscoveryRegistry::new().acceptance_window_from(&FreshnessPolicy::consequential());
        let window = registry
            .respond_to_v0_3(&everything())
            .acceptance_window
            .expect("a bounded policy has a window");
        assert_eq!(
            (window.max_age_seconds.get(), window.clock_skew_seconds),
            (300, 60)
        );

        // Fractions of a second round down, never up.
        let policy = FreshnessPolicy::consequential()
            .with_max_age(chrono::TimeDelta::milliseconds(90_900))
            .with_skew(chrono::TimeDelta::milliseconds(1_999));
        let window = DiscoveryRegistry::new()
            .acceptance_window_from(&policy)
            .respond_to_v0_3(&everything())
            .acceptance_window
            .unwrap();
        assert_eq!(
            (window.max_age_seconds.get(), window.clock_skew_seconds),
            (90, 1)
        );
    }

    #[test]
    fn a_policy_with_no_age_bound_states_no_window() {
        // `FreshnessPolicy::default()` bounds no age: there is no window to
        // state, and absence is what tells a discoverer so.
        let registry = DiscoveryRegistry::new().acceptance_window_from(&FreshnessPolicy::default());
        assert!(registry
            .respond_to_v0_3(&everything())
            .acceptance_window
            .is_none());

        // Under a second cannot be stated either (`maxAgeSeconds >= 1`).
        let policy = FreshnessPolicy::default().with_max_age(chrono::TimeDelta::milliseconds(500));
        assert!(DiscoveryRegistry::new()
            .acceptance_window_from(&policy)
            .respond_to_v0_3(&everything())
            .acceptance_window
            .is_none());
    }

    /// Every response shape the registry can build validates against the
    /// `$anchor: "response"` schema of `trust-task-discovery/0.3`.
    #[cfg(feature = "validate")]
    #[test]
    fn every_0_3_response_shape_validates_against_the_schema() {
        use crate::validate::ValidatedPayload;
        let registries = [
            DiscoveryRegistry::new(),
            DiscoveryRegistry::new().no_framework_version(),
            DiscoveryRegistry::new()
                .with_type_uri(TypeUri::canonical("acl/list", 0, 3).unwrap())
                .with_required_ext(
                    TypeUri::canonical("acl/grant", 0, 2).unwrap(),
                    ["vnd.affinidi.webvh"],
                )
                .acceptance_window_from(&FreshnessPolicy::consequential())
                .with_acceptance_window_for(
                    TypeUri::canonical("acl/show", 0, 2).unwrap(),
                    nz(120),
                    0,
                ),
        ];
        for registry in registries {
            let response = registry.respond_to_v0_3(&everything());
            let value = serde_json::to_value(&response).unwrap();
            wire_v0_3::Response::validate_value(&value)
                .unwrap_or_else(|e| panic!("{value} does not validate: {:?}", e.messages()));
        }
    }
}
