//! Mapping from framework error codes to HTTP status codes.
//!
//! The mapping is informative — SPEC.md leaves transport-level signalling
//! to the binding. This crate uses the table below so a client that
//! cannot or does not want to parse the body can still classify the
//! response broadly.
//!
//! | Standard code        | HTTP status |
//! |----------------------|-------------|
//! | `malformedRequest`   | 400 Bad Request |
//! | `unsupportedType`    | 400 Bad Request |
//! | `unsupportedVersion` | 400 Bad Request |
//! | `expired`            | 400 Bad Request |
//! | `proofRequired`      | 401 Unauthorized |
//! | `proofInvalid`       | 401 Unauthorized |
//! | `permissionDenied`   | 403 Forbidden |
//! | `wrongRecipient`     | 403 Forbidden |
//! | `identityMismatch`   | 403 Forbidden |
//! | `taskFailed`         | 422 Unprocessable Entity |
//! | `unavailable`        | 503 Service Unavailable |
//! | `internalError`      | 500 Internal Server Error |
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
        StandardCode::MalformedRequest
        | StandardCode::UnsupportedType
        | StandardCode::UnsupportedVersion
        | StandardCode::Expired => 400,
        StandardCode::ProofRequired | StandardCode::ProofInvalid => 401,
        StandardCode::PermissionDenied
        | StandardCode::WrongRecipient
        | StandardCode::IdentityMismatch => 403,
        StandardCode::IdConflict => 409,
        StandardCode::TaskFailed => 422,
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

    #[test]
    fn standard_codes_map_to_documented_statuses() {
        // All twelve standard codes — keep in lockstep with the module table.
        assert_eq!(status_for_code(&StandardCode::MalformedRequest.into()), 400);
        assert_eq!(status_for_code(&StandardCode::UnsupportedType.into()), 400);
        assert_eq!(
            status_for_code(&StandardCode::UnsupportedVersion.into()),
            400
        );
        assert_eq!(status_for_code(&StandardCode::Expired.into()), 400);
        assert_eq!(status_for_code(&StandardCode::ProofRequired.into()), 401);
        assert_eq!(status_for_code(&StandardCode::ProofInvalid.into()), 401);
        assert_eq!(status_for_code(&StandardCode::PermissionDenied.into()), 403);
        assert_eq!(status_for_code(&StandardCode::WrongRecipient.into()), 403);
        assert_eq!(status_for_code(&StandardCode::IdentityMismatch.into()), 403);
        assert_eq!(status_for_code(&StandardCode::TaskFailed.into()), 422);
        assert_eq!(status_for_code(&StandardCode::Unavailable.into()), 503);
        assert_eq!(status_for_code(&StandardCode::InternalError.into()), 500);
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
