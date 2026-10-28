//! Parsed Trust Task *Type URI*.
//!
//! Models every form SPEC.md §6.1 defines — the public registry's canonical
//! form, and the two shapes a private specification (SPEC.md §6.5) may use
//! under any other authority:
//!
//! ```text
//! https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>          public registry
//! <prefix>/spec/<slug>/<MAJOR.MINOR>                        path form (https, DID URL, …)
//! urn:<NID>:<prefix>:spec:<slug-segments>:<MAJOR.MINOR>     URN form
//! ```
//!
//! each optionally followed by `#request` or `#response` (§4.4.1). A Type URI
//! whose scheme is `urn` must use the URN form, and every other scheme the
//! path form; `http` is refused in every form. The slug, version, and variant
//! are exposed as typed fields, and [`fmt::Display`] reproduces the parsed
//! string exactly — a Type URI is a name, compared by its whole string
//! (§6.1), so the authority is kept verbatim and is part of equality.

use std::fmt;
use std::str::FromStr;

use serde_with::{DeserializeFromStr, SerializeDisplay};
use thiserror::Error;

/// Parsed Trust Task *Type URI*.
///
/// Round-trips through [`FromStr`] and [`fmt::Display`] and serializes as a
/// single JSON string via [`SerializeDisplay`] / [`DeserializeFromStr`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, SerializeDisplay, DeserializeFromStr)]
pub struct TypeUri {
    /// Everything before `/spec/` (path form) or `:spec:` (URN form).
    authority: String,
    /// The URN form: `<authority>:spec:<seg>:…:<M.N>`.
    urn: bool,
    slug: String,
    major: u32,
    minor: u32,
    variant: Option<Variant>,
}

/// The `#request` / `#response` fragment of a *Type URI*, per SPEC.md §4.4.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Variant {
    /// `#request` — or no fragment, which is semantically equivalent.
    Request,
    /// `#response` — the success-response variant of the same specification.
    Response,
}

impl Variant {
    fn as_fragment(self) -> &'static str {
        match self {
            Variant::Request => "request",
            Variant::Response => "response",
        }
    }
}

/// Reasons a string fails to parse as a [`TypeUri`].
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseTypeUriError {
    /// The URI has no scheme, or its scheme is `http`, which SPEC §6.1
    /// forbids in every form.
    #[error("type URI must have a scheme other than http (SPEC §6.1): {0}")]
    Scheme(String),

    /// Neither the path form `<prefix>/spec/<slug>/<major.minor>` nor (for a
    /// `urn:`) the URN form `urn:<nid>:<prefix>:spec:<seg>:…:<major.minor>`.
    #[error(
        "type URI must be <prefix>/spec/<slug>/<major.minor>, or \
         urn:<nid>:<prefix>:spec:<slug-segments>:<major.minor>: {0}"
    )]
    Shape(String),

    /// Slug fails the §6.1 grammar.
    #[error("slug {0:?} does not satisfy SPEC §6.1 grammar")]
    Slug(String),

    /// Slug is in the `trust-task` reserved namespace and is not one of the
    /// framework-defined allowed slugs.
    #[error("slug {0:?} is reserved per SPEC §6.1")]
    ReservedSlug(String),

    /// Version segment is not `<MAJOR>.<MINOR>`.
    #[error("version {0:?} must be MAJOR.MINOR per SPEC §5.1")]
    Version(String),

    /// Fragment is set but is neither `request` nor `response`.
    #[error("only #request and #response fragments are permitted (got #{0})")]
    Fragment(String),
}

impl TypeUri {
    /// Build a [`TypeUri`] from its components on the canonical
    /// `https://trusttasks.org/` authority.
    pub fn canonical(
        slug: impl Into<String>,
        major: u32,
        minor: u32,
    ) -> Result<Self, ParseTypeUriError> {
        Self::new("https://trusttasks.org", slug, major, minor, None)
    }

    /// Build a [`TypeUri`] from its components.
    ///
    /// `authority` is the scheme + host + optional port + optional `/spec`-stripped
    /// prefix — i.e. everything before `/spec/<slug>/<version>` in the final URI.
    pub fn new(
        authority: impl Into<String>,
        slug: impl Into<String>,
        major: u32,
        minor: u32,
        variant: Option<Variant>,
    ) -> Result<Self, ParseTypeUriError> {
        let slug = slug.into();
        validate_slug(&slug)?;
        Ok(Self {
            authority: authority.into().trim_end_matches('/').to_string(),
            urn: false,
            slug,
            major,
            minor,
            variant,
        })
    }

    /// Build a URN-form [`TypeUri`] (SPEC §6.1):
    /// `<prefix>:spec:<slug segments>:<major>.<minor>`, where `prefix` is
    /// `urn:<NID>:<prefix>`, e.g. `urn:example:tasks`.
    pub fn new_urn(
        prefix: impl Into<String>,
        slug: impl Into<String>,
        major: u32,
        minor: u32,
        variant: Option<Variant>,
    ) -> Result<Self, ParseTypeUriError> {
        let slug = slug.into();
        validate_slug(&slug)?;
        Ok(Self {
            authority: prefix.into().trim_end_matches(':').to_string(),
            urn: true,
            slug,
            major,
            minor,
            variant,
        })
    }

    /// The slug, e.g. `"acl/grant"` or `"trust-task-control"`.
    pub fn slug(&self) -> &str {
        &self.slug
    }

    /// The major version.
    pub fn major(&self) -> u32 {
        self.major
    }

    /// The minor version.
    pub fn minor(&self) -> u32 {
        self.minor
    }

    /// The variant fragment, if any. A missing fragment is semantically
    /// equivalent to [`Variant::Request`] per SPEC §4.4.1.
    pub fn variant(&self) -> Option<Variant> {
        self.variant
    }

    /// Returns `true` if this URI identifies the success-response variant.
    pub fn is_response(&self) -> bool {
        self.variant == Some(Variant::Response)
    }

    /// Returns a copy of this URI with the fragment stripped — the form used
    /// to dereference the schema.
    pub fn bare(&self) -> Self {
        self.with_variant(None)
    }

    /// Returns a copy of this URI in its **routing canonical form**, per
    /// SPEC.md §4.4.1 item 1 ("no fragment" and `#request` are semantically
    /// equivalent). Used by [`crate::Dispatcher`] so that producers emitting
    /// either form route to the same handler.
    ///
    /// * `Variant::Request` and `None` collapse to `None`.
    /// * `Variant::Response` is preserved.
    pub fn for_routing(&self) -> Self {
        match self.variant {
            Some(Variant::Request) | None => self.with_variant(None),
            Some(Variant::Response) => self.clone(),
        }
    }

    /// Returns a copy of this URI with the fragment set to `#response`,
    /// per SPEC.md §4.4.1. Used when minting a success-response document
    /// against a known request URI.
    pub fn with_response(&self) -> Self {
        self.with_variant(Some(Variant::Response))
    }

    /// Returns a copy of this URI with the fragment set to `#request`. The
    /// bare form (no fragment) is semantically equivalent — prefer
    /// [`bare`](Self::bare) unless you need the fragment to be explicit.
    pub fn with_request(&self) -> Self {
        self.with_variant(Some(Variant::Request))
    }

    /// Returns a copy of this URI with `variant` set to `v` (or stripped
    /// when `v` is `None`).
    pub fn with_variant(&self, v: Option<Variant>) -> Self {
        Self {
            variant: v,
            ..self.clone()
        }
    }

    /// The specification this URI names a version of, apart from the version
    /// and variant: its authority, form and slug. Two Type URIs with the same
    /// slug under different authorities are different specifications (SPEC
    /// §6.1 — a slug is unique only within its authority), so this, not the
    /// slug alone, decides whether an unrouted document is an unsupported
    /// *version* of something a consumer implements.
    pub(crate) fn specification_key(&self) -> String {
        let mut bare = self.with_variant(None);
        bare.major = 0;
        bare.minor = 0;
        bare.to_string()
    }

    /// A copy of this URI at `major.minor` — same authority, slug and variant.
    /// Used to walk a declared-extension chain (SPEC §5.2), whose links name
    /// only a version.
    pub(crate) fn with_version(&self, major: u32, minor: u32) -> Self {
        Self {
            major,
            minor,
            ..self.clone()
        }
    }
}

impl fmt::Display for TypeUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.urn {
            write!(
                f,
                "{}:spec:{}:{}.{}",
                self.authority,
                self.slug.replace('/', ":"),
                self.major,
                self.minor
            )?;
        } else {
            write!(
                f,
                "{}/spec/{}/{}.{}",
                self.authority, self.slug, self.major, self.minor
            )?;
        }
        if let Some(v) = self.variant {
            write!(f, "#{}", v.as_fragment())?;
        }
        Ok(())
    }
}

impl FromStr for TypeUri {
    type Err = ParseTypeUriError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (without_fragment, fragment) = match s.split_once('#') {
            Some((u, f)) => (u, Some(f)),
            None => (s, None),
        };

        let variant = match fragment {
            None => None,
            Some("request") => Some(Variant::Request),
            Some("response") => Some(Variant::Response),
            Some(other) => return Err(ParseTypeUriError::Fragment(other.to_string())),
        };

        // SPEC §6.1: any absolute URI, but never `http` — an `http` Type URI
        // would normalise a transport-downgrade path for consumers that
        // dereference it under §6.2 content negotiation, so the parser
        // refuses it in every form.
        let (scheme, _) = without_fragment
            .split_once(':')
            .ok_or_else(|| ParseTypeUriError::Scheme(s.to_string()))?;
        let scheme_ok = scheme
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic())
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
        if !scheme_ok || scheme.eq_ignore_ascii_case("http") {
            return Err(ParseTypeUriError::Scheme(s.to_string()));
        }
        let shape = || ParseTypeUriError::Shape(s.to_string());

        let (authority, slug, version, urn) = if scheme.eq_ignore_ascii_case("urn") {
            // `urn:<NID>:<prefix>:spec:<seg>:…:<MAJOR.MINOR>`. The slug is the
            // segments between the first `:spec:` and the version.
            let (prefix, rest) = without_fragment.split_once(":spec:").ok_or_else(shape)?;
            let mut parts = prefix.splitn(3, ':');
            let (_, nid, ns) = (parts.next(), parts.next(), parts.next());
            if nid.is_none_or(str::is_empty) || ns.is_none_or(str::is_empty) {
                return Err(shape());
            }
            if rest.contains('/') {
                return Err(shape());
            }
            let (slug, version) = rest.rsplit_once(':').ok_or_else(shape)?;
            (prefix, slug.replace(':', "/"), version, true)
        } else {
            // `<prefix>/spec/<slug>/<MAJOR.MINOR>` — an https authority (with an
            // optional path prefix), a DID URL, or any other scheme's prefix.
            let spec_idx = without_fragment.find("/spec/").ok_or_else(shape)?;
            let authority = &without_fragment[..spec_idx];
            let after_scheme = &authority[scheme.len() + 1..];
            if after_scheme.trim_start_matches('/').is_empty()
                || (scheme.eq_ignore_ascii_case("https") && !after_scheme.starts_with("//"))
            {
                return Err(shape());
            }
            let rest = &without_fragment[spec_idx + "/spec/".len()..];
            // The version is the final '/' segment; everything before it is the slug.
            let (slug, version) = rest.rsplit_once('/').ok_or_else(shape)?;
            (authority, slug.to_string(), version, false)
        };

        validate_slug(&slug)?;
        let (major, minor) = parse_version(version)?;

        Ok(Self {
            authority: authority.to_string(),
            urn,
            slug,
            major,
            minor,
            variant,
        })
    }
}

fn parse_version(s: &str) -> Result<(u32, u32), ParseTypeUriError> {
    let (maj, min) = s
        .split_once('.')
        .ok_or_else(|| ParseTypeUriError::Version(s.to_string()))?;
    if has_leading_zero(maj) || has_leading_zero(min) {
        return Err(ParseTypeUriError::Version(s.to_string()));
    }
    let major = maj
        .parse::<u32>()
        .map_err(|_| ParseTypeUriError::Version(s.to_string()))?;
    let minor = min
        .parse::<u32>()
        .map_err(|_| ParseTypeUriError::Version(s.to_string()))?;
    Ok((major, minor))
}

fn has_leading_zero(s: &str) -> bool {
    s.len() > 1 && s.starts_with('0')
}

fn validate_slug(slug: &str) -> Result<(), ParseTypeUriError> {
    if slug.is_empty() {
        return Err(ParseTypeUriError::Slug(slug.to_string()));
    }
    for segment in slug.split('/') {
        validate_segment(segment).ok_or_else(|| ParseTypeUriError::Slug(slug.to_string()))?;
    }
    if is_reserved_namespace(slug) && !is_allowed_framework_slug(slug) {
        return Err(ParseTypeUriError::ReservedSlug(slug.to_string()));
    }
    Ok(())
}

fn validate_segment(seg: &str) -> Option<()> {
    let mut chars = seg.chars();
    let first = chars.next()?;
    if !first.is_ascii_lowercase() {
        return None;
    }
    let mut prev_hyphen = false;
    for c in chars {
        match c {
            'a'..='z' | '0'..='9' => prev_hyphen = false,
            '-' => {
                if prev_hyphen {
                    return None;
                }
                prev_hyphen = true;
            }
            _ => return None,
        }
    }
    // Cannot end with a hyphen.
    if prev_hyphen {
        return None;
    }
    Some(())
}

fn is_reserved_namespace(slug: &str) -> bool {
    // SPEC §6.1 reserves `^trust-(task|ceremony)($|-|/)`. Both halves: the
    // `trust-ceremony` half is unused by published specs at framework 0.5,
    // and exists precisely so the ceremony layer has a namespace no other
    // party can claim first — a check that omits it hands that namespace
    // to whoever asks.
    let first = slug.split('/').next().unwrap_or("");
    for stem in ["trust-task", "trust-ceremony"] {
        if first == stem || first.starts_with(&format!("{stem}-")) {
            return true;
        }
    }
    false
}

/// The framework slugs permitted inside the reservation of
/// [`is_reserved_namespace`].
///
/// This list is the second of two hand-maintained copies — the other is
/// `#/properties/slug/anyOf[1]/enum` in `specs/spec.meta.schema.json`, which
/// is what the registry build enforces. `reserved_allowlist_matches_the_meta_schema`
/// below reads that file and fails if the two disagree, so the copies cannot
/// drift silently the way they did before: this list was missing
/// `trust-task-control` and `trust-ceremony-receipt`, so the library rejected
/// the Type URI of a published specification.
fn is_allowed_framework_slug(slug: &str) -> bool {
    matches!(
        slug,
        "trust-task"
            | "trust-task-error"
            | "trust-task-ok"
            | "trust-task-next-step"
            | "trust-task-discovery"
            | "trust-task-control"
            | "trust-ceremony-receipt"
    )
}

#[cfg(test)]
mod reserved_slug_conformance {
    use super::*;

    /// SPEC §6.1 reserves `^trust-(task|ceremony)($|-|/)`. Both halves.
    #[test]
    fn both_halves_of_the_reservation_are_enforced() {
        for slug in [
            "trust-task",
            "trust-task-error",
            "trust-task-anything",
            "trust-ceremony",
            "trust-ceremony-receipt",
            "trust-ceremony-anything",
            "trust-task/nested",
        ] {
            assert!(is_reserved_namespace(slug), "{slug} is reserved by §6.1");
        }
        for slug in ["acl/grant", "trust-tasks", "trusty-task", "vault/get"] {
            assert!(!is_reserved_namespace(slug), "{slug} is not reserved");
        }
    }

    /// Every framework slug the registry publishes must parse. This is the
    /// regression: `trust-task-control/0.1` is a published specification whose
    /// Type URI this crate rejected, because the allowlist here had drifted
    /// from the one the registry build enforces.
    #[test]
    fn published_framework_type_uris_parse() {
        for slug in [
            "trust-task-error",
            "trust-task-ok",
            "trust-task-next-step",
            "trust-task-discovery",
            "trust-task-control",
            "trust-ceremony-receipt",
        ] {
            let uri = format!("https://trusttasks.org/spec/{slug}/0.1");
            assert!(
                uri.parse::<TypeUri>().is_ok(),
                "{slug} is published by the framework and must parse"
            );
        }
    }

    /// A reserved slug that is *not* a published framework specification must
    /// be refused, or the reservation protects nothing.
    #[test]
    fn an_unclaimed_reserved_slug_is_refused() {
        for slug in ["trust-task-evil", "trust-ceremony-evil"] {
            let uri = format!("https://trusttasks.org/spec/{slug}/0.1");
            assert!(
                uri.parse::<TypeUri>().is_err(),
                "{slug} sits inside the §6.1 reservation and is not framework-published"
            );
        }
    }

    /// The allowlist above is one of two hand-maintained copies; the registry
    /// build enforces the other. Read that one and fail if they disagree —
    /// CLAUDE.md records several incidents of exactly this drift, and this
    /// pair had already drifted by two entries.
    #[test]
    fn reserved_allowlist_matches_the_meta_schema() {
        let meta =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../specs/spec.meta.schema.json");
        let Ok(raw) = std::fs::read_to_string(&meta) else {
            // Published crate: the registry is not vendored alongside it.
            return;
        };
        let json: serde_json::Value = serde_json::from_str(&raw).expect("meta-schema parses");
        let listed = json["properties"]["slug"]["anyOf"][1]["enum"]
            .as_array()
            .expect("reserved-slug allowlist is an array");
        for v in listed {
            let slug = v.as_str().expect("allowlist entries are strings");
            assert!(
                is_allowed_framework_slug(slug),
                "{slug} is allowlisted by specs/spec.meta.schema.json but not by \
                 is_allowed_framework_slug — the two copies have drifted"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_form() {
        // A single-segment slug; the hierarchical case is covered below.
        let uri: TypeUri = "https://trusttasks.org/spec/trust-task-error/0.5"
            .parse()
            .unwrap();
        assert_eq!(uri.slug(), "trust-task-error");
        assert_eq!(uri.major(), 0);
        assert_eq!(uri.minor(), 5);
        assert_eq!(uri.variant(), None);
        assert_eq!(
            uri.to_string(),
            "https://trusttasks.org/spec/trust-task-error/0.5"
        );
    }

    #[test]
    fn parses_hierarchical_slug() {
        let uri: TypeUri = "https://trusttasks.org/spec/acl/grant/0.1".parse().unwrap();
        assert_eq!(uri.slug(), "acl/grant");
        assert_eq!(uri.major(), 0);
        assert_eq!(uri.minor(), 1);
    }

    #[test]
    fn parses_request_and_response_fragments() {
        let req: TypeUri = "https://trusttasks.org/spec/acl/grant/0.1#request"
            .parse()
            .unwrap();
        assert_eq!(req.variant(), Some(Variant::Request));

        let resp: TypeUri = "https://trusttasks.org/spec/acl/grant/0.1#response"
            .parse()
            .unwrap();
        assert_eq!(resp.variant(), Some(Variant::Response));
        assert!(resp.is_response());
        assert_eq!(
            resp.bare().to_string(),
            "https://trusttasks.org/spec/acl/grant/0.1"
        );
    }

    #[test]
    fn rejects_http_scheme() {
        // SPEC §6.1 — http is non-conformant in every form.
        for uri in [
            "http://trusttasks.org/spec/acl/grant/0.1",
            "HTTP://example.com/spec/acl/grant/0.1",
            "http://example.com/x/spec/acl/grant/0.1#response",
        ] {
            let err = uri.parse::<TypeUri>().unwrap_err();
            assert!(matches!(err, ParseTypeUriError::Scheme(_)), "{uri}");
        }
    }

    /// Every SPEC §6.1 example parses, names its slug and version, and prints
    /// back as the same string — with each variant fragment.
    #[test]
    fn every_spec_6_1_form_round_trips() {
        for (uri, slug) in [
            ("https://trusttasks.org/spec/acl/grant/0.1", "acl/grant"),
            (
                "https://example.com/trust-tasks/spec/acl/grant/0.1",
                "acl/grant",
            ),
            (
                "did:example:123456789abcdefghi/spec/acl/grant/0.1",
                "acl/grant",
            ),
            ("did:web:tasks.example.com:x/spec/pay/0.1", "pay"),
            ("urn:example:tasks:spec:acl:grant:0.1", "acl/grant"),
            ("urn:example:tasks:sub:spec:pay:3.10", "pay"),
            ("tag:example.com,2026:tasks/spec/acl/grant/0.1", "acl/grant"),
        ] {
            for fragment in ["", "#request", "#response"] {
                let s = format!("{uri}{fragment}");
                let parsed: TypeUri = s.parse().unwrap_or_else(|e| panic!("{s}: {e}"));
                assert_eq!(parsed.slug(), slug, "{s}");
                assert_eq!(parsed.to_string(), s);
                assert_eq!(parsed.bare().to_string(), uri);
                let json = serde_json::to_string(&parsed).unwrap();
                assert_eq!(serde_json::from_str::<TypeUri>(&json).unwrap(), parsed);
            }
        }
        let urn: TypeUri = "urn:example:tasks:spec:acl:grant:0.1".parse().unwrap();
        assert_eq!((urn.major(), urn.minor()), (0, 1));
        assert_eq!(
            urn.with_response().to_string(),
            "urn:example:tasks:spec:acl:grant:0.1#response"
        );
        assert_eq!(
            urn.with_version(0, 2).to_string(),
            "urn:example:tasks:spec:acl:grant:0.2"
        );
        assert_eq!(
            TypeUri::new_urn("urn:example:tasks", "acl/grant", 0, 1, None)
                .unwrap()
                .to_string(),
            "urn:example:tasks:spec:acl:grant:0.1"
        );
    }

    /// A Type URI is a name compared by its whole string: the same slug and
    /// version under two authorities, or in two forms, are different.
    #[test]
    fn the_authority_is_part_of_identity() {
        let a: TypeUri = "https://trusttasks.org/spec/vault/get/0.3".parse().unwrap();
        let b: TypeUri = "https://tasks.example.com/spec/vault/get/0.3"
            .parse()
            .unwrap();
        let c: TypeUri = "urn:example:tasks:spec:vault:get:0.3".parse().unwrap();
        let d: TypeUri = "did:web:tasks.example.com/spec/vault/get/0.3"
            .parse()
            .unwrap();
        for (x, y) in [(&a, &b), (&a, &c), (&b, &c), (&b, &d), (&c, &d)] {
            assert_eq!(x.slug(), y.slug());
            assert_ne!(x, y);
            assert_ne!(x.for_routing(), y.for_routing());
        }
    }

    #[test]
    fn malformed_forms_are_refused() {
        for (uri, scheme_error) in [
            ("trusttasks.org/spec/acl/grant/0.1", true),
            ("1https://example.com/spec/a/0.1", true),
            ("https:/spec/a/0.1", false),
            ("https:///spec/a/0.1", false),
            ("https:example.com/spec/a/0.1", false),
            ("did:/spec/a/0.1", false),
            ("urn:example:spec:a:0.1", false),
            ("urn::tasks:spec:a:0.1", false),
            ("urn:example:tasks:spec:acl/grant:0.1", false),
            ("urn:example:tasks/spec/acl/grant/0.1", false),
            ("did:example:123", false),
        ] {
            let err = uri.parse::<TypeUri>().unwrap_err();
            assert_eq!(
                matches!(err, ParseTypeUriError::Scheme(_)),
                scheme_error,
                "{uri}: {err:?}"
            );
        }
        // The slug and version grammars hold in the URN form too.
        assert!(matches!(
            "urn:example:tasks:spec:Acl:0.1".parse::<TypeUri>(),
            Err(ParseTypeUriError::Slug(_))
        ));
        assert!(matches!(
            "urn:example:tasks:spec:acl:01.0".parse::<TypeUri>(),
            Err(ParseTypeUriError::Version(_))
        ));
        assert!(matches!(
            "urn:example:tasks:spec:trust-task-custom:0.1".parse::<TypeUri>(),
            Err(ParseTypeUriError::ReservedSlug(_))
        ));
    }

    #[test]
    fn rejects_unknown_fragment() {
        let err = "https://trusttasks.org/spec/acl/grant/0.1#cancel"
            .parse::<TypeUri>()
            .unwrap_err();
        assert!(matches!(err, ParseTypeUriError::Fragment(s) if s == "cancel"));
    }

    #[test]
    fn rejects_reserved_slug() {
        let err = "https://example.com/spec/trust-task-custom/0.1"
            .parse::<TypeUri>()
            .unwrap_err();
        assert!(matches!(err, ParseTypeUriError::ReservedSlug(_)));
    }

    #[test]
    fn allows_framework_slug() {
        let uri: TypeUri = "https://trusttasks.org/spec/trust-task-error/0.1"
            .parse()
            .unwrap();
        assert_eq!(uri.slug(), "trust-task-error");
    }

    #[test]
    fn accepts_private_authority() {
        let uri: TypeUri = "https://example.com/trust-tasks/spec/my-task/0.1"
            .parse()
            .unwrap();
        assert_eq!(uri.slug(), "my-task");
        // Round-trip preserves the prefix path.
        assert_eq!(
            uri.to_string(),
            "https://example.com/trust-tasks/spec/my-task/0.1"
        );
    }

    #[test]
    fn rejects_bad_slug() {
        for bad in [
            "Acl",      // uppercase
            "acl--bad", // consecutive hyphens
            "-leading",
            "trailing-",
            "1numeric-start",
            "with_underscore",
        ] {
            let s = format!("https://trusttasks.org/spec/{bad}/1.0");
            let err = s.parse::<TypeUri>().unwrap_err();
            assert!(matches!(err, ParseTypeUriError::Slug(_)), "{bad}: {err:?}");
        }
    }

    #[test]
    fn rejects_bad_version() {
        for bad in ["01.0", "1.01", "1", "1.0.0", "a.b"] {
            let s = format!("https://trusttasks.org/spec/x/{bad}");
            let err = s.parse::<TypeUri>().unwrap_err();
            assert!(
                matches!(err, ParseTypeUriError::Version(_)),
                "{bad}: {err:?}"
            );
        }
    }

    #[test]
    fn round_trips_via_serde() {
        let uri: TypeUri = "https://trusttasks.org/spec/acl/change-role/0.1#response"
            .parse()
            .unwrap();
        let json = serde_json::to_string(&uri).unwrap();
        assert_eq!(
            json,
            "\"https://trusttasks.org/spec/acl/change-role/0.1#response\""
        );
        let back: TypeUri = serde_json::from_str(&json).unwrap();
        assert_eq!(back, uri);
    }
}
