//! Mapping from framework error codes to HTTP status codes.
//!
//! The mapping is informative — SPEC.md leaves transport-level signalling
//! to the binding. The authority for this table is the HTTPS binding
//! specification's §4 status mapping (`bindings/https/0.2/spec.md`); a
//! client that cannot or does not want to parse the body can still
//! classify the response broadly.
//!
//! | Standard code        | HTTP status |
//! |----------------------|-------------|
//! | `malformedRequest`   | 400 Bad Request |
//! | `permissionDenied`   | 403 Forbidden |
//! | `unsupportedType`    | 422 Unprocessable Entity |
//! | `unsupportedVersion` | 422 Unprocessable Entity |
//! | `expired`            | 422 Unprocessable Entity |
//! | `proofRequired`      | 422 Unprocessable Entity |
//! | `proofInvalid`       | 422 Unprocessable Entity |
//! | `identityMismatch`   | 422 Unprocessable Entity |
//! | `wrongRecipient`     | 422 Unprocessable Entity |
//! | `cancelled`          | 422 Unprocessable Entity |
//! | `taskFailed`         | 422 Unprocessable Entity |
//! | `idConflict`         | 409 Conflict |
//! | `unavailable`        | 503 Service Unavailable |
//! | `internalError`      | 500 Internal Server Error |
//!
//! Two outcomes have no framework error code and so no row here: a duplicate
//! of an execution still in progress (`202 Accepted`) and a duplicate for
//! which no response was retained (`204 No Content`). They are successes, not
//! rejections — see the binding spec's §5.1 — and the server emits them
//! directly rather than through this table.
//!
//! The flatness of the 422 bucket is the point, not an oversight. An
//! earlier version of this table split `proofRequired` / `proofInvalid`
//! into 401 and `permissionDenied` / `wrongRecipient` / `identityMismatch`
//! into 403 — which handed an unauthenticated prober a status-code oracle
//! distinguishing "your proof is bad" from "you are not who you said" from
//! "this is not addressed to me", without ever reading a body. Collapsing
//! them to one status removes that signal; the framework error code in the
//! body remains available to a legitimate producer, which is the party
//! entitled to it. `permissionDenied` keeps 403 because the binding's table
//! says so: it is the one outcome that reports on the *authenticated*
//! caller's authorization, and so tells an unauthenticated prober nothing.
//!
//! Extension codes (`<slug>:<local>`) default to **422 Unprocessable
//! Entity** since they are spec-defined application-layer failures.

use trust_tasks_rs::{StandardCode, TrustTaskCode};

/// Map a [`TrustTaskCode`] to a numeric HTTP status code.
///
/// Returns `u16` rather than `http::StatusCode` so the function is
/// dependency-light and usable from both the server (axum) and client
/// (reqwest) sides without dragging in extra crates here.
pub fn status_for_code(code: &TrustTaskCode) -> u16 {
    match code {
        TrustTaskCode::Standard(c) => standard_status(*c),
        TrustTaskCode::Extended { .. } => 422,
    }
}

fn standard_status(code: StandardCode) -> u16 {
    match code {
        StandardCode::MalformedRequest => 400,
        StandardCode::PermissionDenied => 403,
        StandardCode::IdConflict => 409,
        // The flat 422 bucket of the binding spec's §4 table. Everything
        // here is "understood, well-formed, and refused"; keeping them
        // indistinguishable at the status line is deliberate — see the
        // module docs for the oracle the finer split created.
        StandardCode::UnsupportedType
        | StandardCode::UnsupportedVersion
        | StandardCode::Expired
        | StandardCode::ProofRequired
        | StandardCode::ProofInvalid
        | StandardCode::WrongRecipient
        | StandardCode::IdentityMismatch
        | StandardCode::Cancelled
        | StandardCode::TaskFailed => 422,
        StandardCode::Unavailable => 503,
        StandardCode::InternalError => 500,
        // `StandardCode` is `#[non_exhaustive]` (trust-tasks-rs 0.7.0): a
        // framework revision can add a code without breaking this crate. An
        // unmapped code is a server-side failure to keep up, not a client
        // error, so it maps to 500 rather than being silently bucketed as 400.
        _ => 500,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// REGRESSION: the table must match `bindings/https/0.2/spec.md` §4.
    /// Before this was fixed, `proofRequired`/`proofInvalid` mapped to 401,
    /// `permissionDenied`/`wrongRecipient`/`identityMismatch` to 403, and
    /// `unsupportedType`/`unsupportedVersion`/`expired` to 400 — three rows
    /// of drift from the binding, and a status-code identity oracle.
    #[test]
    fn standard_codes_map_to_binding_spec_statuses() {
        assert_eq!(status_for_code(&StandardCode::MalformedRequest.into()), 400);
        assert_eq!(status_for_code(&StandardCode::PermissionDenied.into()), 403);
        assert_eq!(status_for_code(&StandardCode::IdConflict.into()), 409);
        assert_eq!(status_for_code(&StandardCode::UnsupportedType.into()), 422);
        assert_eq!(
            status_for_code(&StandardCode::UnsupportedVersion.into()),
            422
        );
        assert_eq!(status_for_code(&StandardCode::Expired.into()), 422);
        assert_eq!(status_for_code(&StandardCode::ProofRequired.into()), 422);
        assert_eq!(status_for_code(&StandardCode::ProofInvalid.into()), 422);
        assert_eq!(status_for_code(&StandardCode::WrongRecipient.into()), 422);
        assert_eq!(status_for_code(&StandardCode::IdentityMismatch.into()), 422);
        assert_eq!(status_for_code(&StandardCode::Cancelled.into()), 422);
        assert_eq!(status_for_code(&StandardCode::TaskFailed.into()), 422);
        assert_eq!(status_for_code(&StandardCode::Unavailable.into()), 503);
        assert_eq!(status_for_code(&StandardCode::InternalError.into()), 500);
    }

    /// REGRESSION: the codes that report on identity or attribution must be
    /// indistinguishable at the status line. Asserted as a set property so a
    /// future edit that re-splits one of them out fails here, not only in
    /// the per-code table above.
    #[test]
    fn identity_related_codes_share_one_status() {
        let identity_codes = [
            StandardCode::ProofRequired,
            StandardCode::ProofInvalid,
            StandardCode::IdentityMismatch,
            StandardCode::WrongRecipient,
        ];
        for code in identity_codes {
            assert_eq!(
                status_for_code(&code.into()),
                422,
                "{code:?} must not be distinguishable by status code"
            );
        }
    }

    #[test]
    fn extension_codes_default_to_422() {
        let code = TrustTaskCode::Extended {
            slug: "acl/grant".into(),
            local: "lastAuthorityProtected".into(),
        };
        assert_eq!(status_for_code(&code), 422);
    }
}
