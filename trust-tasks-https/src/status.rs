//! Mapping from framework error codes to HTTP status codes.
//!
//! The mapping is informative — SPEC.md leaves transport-level signalling
//! to the binding. This crate uses the table below so a client that
//! cannot or does not want to parse the body can still classify the
//! response broadly.
//!
//! | Standard code         | HTTP status |
//! |-----------------------|-------------|
//! | `malformed_request`   | 400 Bad Request |
//! | `unsupported_type`    | 400 Bad Request |
//! | `unsupported_version` | 400 Bad Request |
//! | `expired`             | 400 Bad Request |
//! | `proof_required`      | 401 Unauthorized |
//! | `proof_invalid`       | 401 Unauthorized |
//! | `permission_denied`   | 403 Forbidden |
//! | `wrong_recipient`     | 403 Forbidden |
//! | `identity_mismatch`   | 403 Forbidden |
//! | `task_failed`         | 422 Unprocessable Entity |
//! | `unavailable`         | 503 Service Unavailable |
//! | `internal_error`      | 500 Internal Server Error |
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
        StandardCode::TaskFailed => 422,
        StandardCode::Unavailable => 503,
        StandardCode::InternalError => 500,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_codes_map_to_documented_statuses() {
        assert_eq!(status_for_code(&StandardCode::MalformedRequest.into()), 400);
        assert_eq!(status_for_code(&StandardCode::Expired.into()), 400);
        assert_eq!(status_for_code(&StandardCode::ProofRequired.into()), 401);
        assert_eq!(status_for_code(&StandardCode::IdentityMismatch.into()), 403);
        assert_eq!(status_for_code(&StandardCode::TaskFailed.into()), 422);
        assert_eq!(status_for_code(&StandardCode::Unavailable.into()), 503);
        assert_eq!(status_for_code(&StandardCode::InternalError.into()), 500);
    }

    #[test]
    fn extension_codes_default_to_422() {
        let code = TrustTaskCode::Extended {
            slug: "acl/grant".into(),
            local: "permission_denied".into(),
        };
        assert_eq!(status_for_code(&code), 422);
    }
}
