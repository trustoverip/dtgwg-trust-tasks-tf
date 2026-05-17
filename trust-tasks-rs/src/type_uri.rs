//! Parsed Trust Task *Type URI*.
//!
//! Models the canonical URI form defined in SPEC.md §6.1:
//!
//! ```text
//! https://trusttasks.org/spec/<slug>/<MAJOR.MINOR>[#request|#response]
//! ```
//!
//! Private-registry authorities (SPEC.md §6.5) are accepted as long as the
//! path shape is preserved. The slug, version, and optional request/response
//! variant fragment (§4.4.1) are exposed as typed fields.

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
    authority: String,
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
    /// URI scheme is not `https` or `http`.
    #[error("type URI must use https (or http) scheme: {0}")]
    Scheme(String),

    /// The path is missing the `/spec/<slug>/<version>` shape.
    #[error("type URI path must be /spec/<slug>/<major.minor>: {0}")]
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
            slug,
            major,
            minor,
            variant,
        })
    }

    /// The slug, e.g. `"acl/grant"` or `"kyc-handoff"`.
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
}

impl fmt::Display for TypeUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/spec/{}/{}.{}",
            self.authority, self.slug, self.major, self.minor
        )?;
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

        let scheme_split = without_fragment
            .find("://")
            .ok_or_else(|| ParseTypeUriError::Scheme(s.to_string()))?;
        let scheme = &without_fragment[..scheme_split];
        // SPEC §6.1: the scheme MUST be `https`. `http://` would normalise
        // a transport-downgrade path for consumers that dereference the
        // Type URI under §6.2 content negotiation, so we refuse it at the
        // parser.
        if scheme != "https" {
            return Err(ParseTypeUriError::Scheme(s.to_string()));
        }

        // Split into "<scheme>://<authority-and-path-prefix>/spec/<rest>"
        let spec_idx = without_fragment
            .find("/spec/")
            .ok_or_else(|| ParseTypeUriError::Shape(s.to_string()))?;
        let authority = &without_fragment[..spec_idx];
        let rest = &without_fragment[spec_idx + "/spec/".len()..];

        // The version is the final '/' segment; everything before it is the slug.
        let last_slash = rest
            .rfind('/')
            .ok_or_else(|| ParseTypeUriError::Shape(s.to_string()))?;
        let slug = &rest[..last_slash];
        let version = &rest[last_slash + 1..];

        validate_slug(slug)?;
        let (major, minor) = parse_version(version)?;

        Ok(Self {
            authority: authority.to_string(),
            slug: slug.to_string(),
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
    let first = slug.split('/').next().unwrap_or("");
    first == "trust-task" || first.starts_with("trust-task-")
}

fn is_allowed_framework_slug(slug: &str) -> bool {
    matches!(
        slug,
        "trust-task"
            | "trust-task-error"
            | "trust-task-ok"
            | "trust-task-next-step"
            | "trust-task-discovery"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_form() {
        let uri: TypeUri = "https://trusttasks.org/spec/kyc-handoff/1.0"
            .parse()
            .unwrap();
        assert_eq!(uri.slug(), "kyc-handoff");
        assert_eq!(uri.major(), 1);
        assert_eq!(uri.minor(), 0);
        assert_eq!(uri.variant(), None);
        assert_eq!(
            uri.to_string(),
            "https://trusttasks.org/spec/kyc-handoff/1.0"
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
        // SPEC §6.1 — only https is conformant.
        let err = "http://trusttasks.org/spec/acl/grant/0.1"
            .parse::<TypeUri>()
            .unwrap_err();
        assert!(matches!(err, ParseTypeUriError::Scheme(_)));
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
        let uri: TypeUri = "https://trusttasks.org/spec/kyc-handoff/1.0#response"
            .parse()
            .unwrap();
        let json = serde_json::to_string(&uri).unwrap();
        assert_eq!(
            json,
            "\"https://trusttasks.org/spec/kyc-handoff/1.0#response\""
        );
        let back: TypeUri = serde_json::from_str(&json).unwrap();
        assert_eq!(back, uri);
    }
}
