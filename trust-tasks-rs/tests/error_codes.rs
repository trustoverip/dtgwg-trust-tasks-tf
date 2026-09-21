//! The declared extended error codes (SPEC §7.3 item 9, §8.5) as a consumer
//! sees them: per module as `ERROR_CODES` / `error_codes::*`, and by Type URI
//! through `schema_index::error_codes_for`.
//!
//! Like `spec_const_emission.rs` these are spot checks, not coverage. The
//! exhaustive comparison of every module's `ERROR_CODES` against its spec's
//! front matter is in `scripts/check-bindings-conformance.mjs`, which derives
//! the expected list independently of the generator.
//!
//! The index half needs `schema_index`, which is behind the `validate` feature.

use trust_tasks_rs::specs::keys::revoke::v0_1 as keys_revoke;
use trust_tasks_rs::specs::trust_task_discovery::v0_1 as discovery;
use trust_tasks_rs::specs::vta::management::reload_services::v1_0 as reload_services;
use trust_tasks_rs::specs::vtc::join_requests::withdraw::v0_1 as withdraw;
use trust_tasks_rs::{DeclaredErrorCode, ErrorPayload, Payload, TrustTaskCode};

/// The wire string is usable where a consumer keeps its own `&str` constant, so
/// a hand-written literal can be replaced rather than merely checked.
const WITHDRAW_NOT_FOUND: &str = withdraw::error_codes::NOT_FOUND.code;

#[test]
fn declaring_nothing_is_an_empty_slice() {
    assert_eq!(discovery::ERROR_CODES, &[] as &[DeclaredErrorCode]);
}

#[test]
fn a_module_carries_its_declarations_in_order() {
    assert_eq!(WITHDRAW_NOT_FOUND, "vtc/join-requests/withdraw:notFound");
    assert_eq!(
        withdraw::ERROR_CODES
            .iter()
            .map(|c| c.code)
            .collect::<Vec<_>>(),
        [
            "vtc/join-requests/withdraw:notFound",
            "vtc/join-requests/withdraw:alreadyDecided",
        ]
    );
    assert!(withdraw::ERROR_CODES.iter().all(|c| !c.retryable));
    const { assert!(reload_services::error_codes::RELOAD_FAILED.retryable) };
}

#[test]
fn a_declared_code_converts_to_the_wire_code_it_names() {
    let declared = withdraw::error_codes::ALREADY_DECIDED;
    let code = TrustTaskCode::from(declared);
    assert_eq!(code.to_string(), declared.code);
    assert_eq!(declared.code.parse::<TrustTaskCode>().unwrap(), code);
    assert_eq!(code, withdraw::Payload::extended_code("alreadyDecided"));
    assert!(declared.matches(&code));
    assert!(!withdraw::error_codes::NOT_FOUND.matches(&code));
}

/// SPEC §8.5 rule 2: a family namespace keeps its own prefix, and the declared
/// constant agrees with what `family_code` builds for it.
#[test]
fn a_family_code_keeps_its_family_namespace() {
    let declared = keys_revoke::error_codes::NOT_FOUND;
    assert_eq!(declared.namespace(), "keys");
    assert_eq!(declared.local(), "notFound");
    assert!(declared.matches(&keys_revoke::Payload::family_code("keys", "notFound")));
}

/// An extended code the consumer does not recognise defaults to
/// non-retryable (§8.5); a declared one carries what the spec declares.
#[test]
fn an_error_payload_takes_the_declared_retryable() {
    let payload: ErrorPayload = reload_services::error_codes::RELOAD_FAILED.into();
    assert!(payload.retryable);
    assert_eq!(
        payload.code.to_string(),
        "vta/management/reload-services:reloadFailed"
    );
}

#[cfg(feature = "validate")]
mod index {
    use super::*;
    use trust_tasks_rs::schema_index::error_codes_for;

    const WITHDRAW: &str = "https://trusttasks.org/spec/vtc/join-requests/withdraw/0.1";

    #[test]
    fn the_index_serves_the_same_slice_as_the_module() {
        assert_eq!(error_codes_for(WITHDRAW), Some(withdraw::ERROR_CODES));
        assert_eq!(
            error_codes_for(<keys_revoke::Payload as Payload>::TYPE_URI),
            Some(keys_revoke::ERROR_CODES)
        );
    }

    /// `Some(&[])` and `None` are different answers, and a census that confused
    /// them would pass every task it had never heard of.
    #[test]
    fn declaring_nothing_is_not_the_same_as_being_unknown() {
        assert_eq!(
            error_codes_for(<discovery::Payload as Payload>::TYPE_URI),
            Some(&[] as &[DeclaredErrorCode])
        );
        assert_eq!(
            error_codes_for("https://trusttasks.org/spec/not/a/task/9.9"),
            None
        );
    }

    /// Codes belong to the specification, not a variant.
    #[test]
    fn the_index_is_keyed_on_the_request_uri_only() {
        assert_eq!(error_codes_for(&format!("{WITHDRAW}#response")), None);
    }
}
