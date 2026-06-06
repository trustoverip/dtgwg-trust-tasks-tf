//! Property tests for the two hand-rolled parsers — `TypeUri::from_str` and
//! `TrustTaskCode::from_str`. They are string-slicing state machines, so the
//! cheap high-value guarantees are:
//!   * parse ∘ display is the identity on valid inputs (round-trip), and
//!   * `from_str` never panics on arbitrary input (no slicing on a non-char
//!     boundary, no index-out-of-bounds, etc.).

use proptest::prelude::*;
use trust_tasks_rs::{StandardCode, TrustTaskCode, TypeUri};

/// A valid slug: 1–3 segments of `[a-z][a-z0-9]{0,5}` joined by `/`. The short,
/// hyphen-free segments can never spell the reserved `trust-task*` namespace.
fn slug_strategy() -> impl Strategy<Value = String> {
    proptest::collection::vec("[a-z][a-z0-9]{0,5}", 1..4).prop_map(|segs| segs.join("/"))
}

proptest! {
    /// Any canonical Type URI (with or without a request/response fragment)
    /// survives a display→parse→display round-trip unchanged.
    #[test]
    fn type_uri_round_trips(slug in slug_strategy(), major in 0u32..1000, minor in 0u32..1000, frag in 0u8..3) {
        let base = TypeUri::canonical(&slug, major, minor).expect("canonical Type URI");
        let uri = match frag {
            0 => base,
            1 => base.with_request(),
            _ => base.with_response(),
        };
        let s = uri.to_string();
        let parsed: TypeUri = s.parse().expect("round-trip parse");
        prop_assert_eq!(&parsed, &uri);
        prop_assert_eq!(parsed.to_string(), s);
    }

    /// `TypeUri::from_str` must never panic, whatever the input.
    #[test]
    fn type_uri_from_str_never_panics(s in any::<String>()) {
        let _ = s.parse::<TypeUri>();
    }

    /// A `<slug>:<local>` extended code round-trips. `local` exercises both
    /// lowerCamelCase (0.2) and snake_case (0.1) shapes.
    #[test]
    fn extended_code_round_trips(slug in slug_strategy(), local in "[a-z][a-zA-Z0-9_]{0,8}") {
        let raw = format!("{slug}:{local}");
        let code: TrustTaskCode = raw.parse().expect("parse extended code");
        prop_assert!(matches!(code, TrustTaskCode::Extended { .. }), "expected Extended variant");
        prop_assert_eq!(code.to_string(), raw);
    }

    /// `TrustTaskCode::from_str` must never panic, whatever the input.
    #[test]
    fn trust_task_code_from_str_never_panics(s in any::<String>()) {
        let _ = s.parse::<TrustTaskCode>();
    }
}

/// Regression: the framework 0.2 lowerCamelCase extended-code locals (and the
/// frozen 0.1 snake_case locals) both parse — `validate_local` accepts upper
/// case and underscores.
#[test]
fn camel_and_snake_extended_locals_parse() {
    for raw in [
        "acl/change-role:lastAuthorityProtected", // 0.2 camelCase
        "kyc-handoff:documentRevoked",            // 0.2 camelCase
        "acl/grant:permission_denied",            // 0.1 snake_case (frozen)
    ] {
        let code: TrustTaskCode = raw.parse().unwrap_or_else(|e| panic!("parse {raw}: {e}"));
        assert!(matches!(code, TrustTaskCode::Extended { .. }));
        assert_eq!(code.to_string(), raw);
    }
}

/// Every standard code parses from its emitted (camelCase) wire form.
#[test]
fn all_standard_codes_round_trip() {
    for c in [
        StandardCode::MalformedRequest,
        StandardCode::UnsupportedType,
        StandardCode::UnsupportedVersion,
        StandardCode::Expired,
        StandardCode::ProofRequired,
        StandardCode::ProofInvalid,
        StandardCode::PermissionDenied,
        StandardCode::WrongRecipient,
        StandardCode::IdentityMismatch,
        StandardCode::TaskFailed,
        StandardCode::Unavailable,
        StandardCode::InternalError,
    ] {
        let parsed: TrustTaskCode = c.as_str().parse().unwrap();
        assert_eq!(parsed, TrustTaskCode::Standard(c));
    }
}
