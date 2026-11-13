//! ACL policy guards.
//!
//! Pure functions a maintainer calls from inside an `acl/*` handler to
//! enforce policy checks the spec names but does not implement. Each
//! returns an `Option<&'static str>` where `Some(error_code)` is the
//! extended error code the specification declares for the refusal — the
//! `code` of the error payload to answer with (see
//! [`reject_last_authority`] for building one).

/// `acl/revoke:lastAuthorityProtected` — the extended error code every
/// version of `acl/revoke` declares for a revocation that would leave no
/// holder of a protected role, and what [`reject_last_authority`] returns.
///
/// With the `acl` feature, the generated `error_codes::LAST_AUTHORITY_PROTECTED`
/// of `crate::specs::acl::revoke::v0_3` is the same code, and also carries
/// its declared `retryable` and `detailsSchema`; prefer it where the
/// generated module is in scope.
pub const LAST_AUTHORITY_PROTECTED: &str = "acl/revoke:lastAuthorityProtected";

/// Detect whether revoking `subject` from the entries iterator would
/// empty the maintainer's privileged-role set, per the
/// `acl/revoke:lastAuthorityProtected` error code declared by
/// [`acl/revoke`](https://trusttasks.org/spec/acl/revoke/0.3).
///
/// `entries` yields the maintainer's current ACL entries (or any
/// ecosystem-flavoured subtype that exposes role + subject identity via
/// the two closures). `is_protected` returns `true` for entries whose
/// role is part of the "must always have at least one of" set
/// (typically `admin`, `owner`, …). `is_target_subject` returns `true`
/// for the entry being revoked.
///
/// Returns `Some(`[`LAST_AUTHORITY_PROTECTED`]`)` when the revocation would
/// leave zero protected entries; otherwise `None`.
///
/// The returned code is the error payload's `code`. Parse it into a
/// [`TrustTaskCode`](crate::TrustTaskCode) and build the payload with
/// [`ErrorPayload::new`](crate::ErrorPayload::new) — or, with the `acl`
/// feature, build it from the generated declaration with
/// `ErrorPayload::from(acl::revoke::v0_3::error_codes::LAST_AUTHORITY_PROTECTED)`,
/// which also sets the declared `retryable`. The code names `acl/revoke`, so
/// it is the one to answer an `acl/revoke` request with; a handler for
/// another task mints its own (SPEC §8.5 rule 2).
///
/// The function is generic in the entry type so consumers that carry
/// ecosystem extension fields (webvh's `AclEntry.ext.vnd.affinidi.webvh.*`,
/// etc.) pass their own type without conversion.
///
/// ```rust
/// use trust_tasks_rs::guards::acl::{reject_last_authority, LAST_AUTHORITY_PROTECTED};
/// use trust_tasks_rs::{ErrorPayload, TrustTaskCode};
///
/// struct LocalEntry { subject: &'static str, role: &'static str }
///
/// let entries = [
///     LocalEntry { subject: "did:web:alice.example", role: "admin" },
///     LocalEntry { subject: "did:web:bob.example",   role: "member" },
/// ];
///
/// // Revoking the only admin trips the guard.
/// let code = reject_last_authority(
///     entries.iter(),
///     |e: &&LocalEntry| e.role == "admin",
///     |e: &&LocalEntry| e.subject == "did:web:alice.example",
/// );
/// assert_eq!(code, Some(LAST_AUTHORITY_PROTECTED));
///
/// // The error payload to answer with.
/// let code: TrustTaskCode = code.unwrap().parse().unwrap();
/// let payload = ErrorPayload::new(code).with_retryable(false);
/// assert_eq!(payload.code.to_string(), "acl/revoke:lastAuthorityProtected");
///
/// // Revoking a non-admin does not trip the guard.
/// assert_eq!(
///     reject_last_authority(
///         entries.iter(),
///         |e: &&LocalEntry| e.role == "admin",
///         |e: &&LocalEntry| e.subject == "did:web:bob.example",
///     ),
///     None,
/// );
/// ```
pub fn reject_last_authority<E, F, G>(
    entries: impl IntoIterator<Item = E>,
    is_protected: F,
    is_target_subject: G,
) -> Option<&'static str>
where
    F: Fn(&E) -> bool,
    G: Fn(&E) -> bool,
{
    let mut target_is_protected = false;
    let mut other_protected = 0usize;
    for entry in entries {
        let protected = is_protected(&entry);
        let target = is_target_subject(&entry);
        if protected && target {
            target_is_protected = true;
        } else if protected {
            other_protected += 1;
        }
    }
    if target_is_protected && other_protected == 0 {
        Some(LAST_AUTHORITY_PROTECTED)
    } else {
        None
    }
}

#[cfg(test)]
// The specimens are prebuilt modules, deprecated in favour of generating the
// tasks an application uses (trust-tasks-build).
#[allow(deprecated)]
mod tests {
    use super::*;

    struct E {
        subject: &'static str,
        role: &'static str,
    }

    fn is_admin(e: &&E) -> bool {
        e.role == "admin"
    }

    fn target_alice(e: &&E) -> bool {
        e.subject == "did:web:alice.example"
    }

    #[test]
    fn single_admin_revocation_trips_guard() {
        let entries = [
            E {
                subject: "did:web:alice.example",
                role: "admin",
            },
            E {
                subject: "did:web:bob.example",
                role: "member",
            },
        ];
        assert_eq!(
            reject_last_authority(entries.iter(), is_admin, target_alice),
            Some(LAST_AUTHORITY_PROTECTED)
        );
    }

    #[test]
    fn two_admins_revocation_does_not_trip_guard() {
        let entries = [
            E {
                subject: "did:web:alice.example",
                role: "admin",
            },
            E {
                subject: "did:web:bob.example",
                role: "admin",
            },
        ];
        assert_eq!(
            reject_last_authority(entries.iter(), is_admin, target_alice),
            None
        );
    }

    #[test]
    fn revoking_non_admin_does_not_trip_guard_even_if_only_one_admin() {
        let entries = [
            E {
                subject: "did:web:alice.example",
                role: "admin",
            },
            E {
                subject: "did:web:bob.example",
                role: "member",
            },
        ];
        // Target is bob — a non-admin. Even though revoking him would
        // leave alice as the sole admin (which is fine), this guard is
        // about emptying the protected set, not about reducing it.
        assert_eq!(
            reject_last_authority(entries.iter(), is_admin, |e: &&E| e.subject
                == "did:web:bob.example"),
            None
        );
    }

    #[test]
    fn revoking_target_not_in_acl_does_not_trip_guard() {
        let entries = [E {
            subject: "did:web:alice.example",
            role: "admin",
        }];
        assert_eq!(
            reject_last_authority(entries.iter(), is_admin, |e: &&E| e.subject
                == "did:web:nobody.example"),
            None
        );
    }

    #[test]
    fn empty_acl_does_not_trip_guard() {
        let entries: [E; 0] = [];
        assert_eq!(
            reject_last_authority(entries.iter(), is_admin, target_alice),
            None
        );
    }

    /// The guard's code is the one every version of `acl/revoke` declares —
    /// the generated `DeclaredErrorCode`, not a hand-spelled variant of it.
    #[test]
    #[cfg(feature = "acl")]
    fn the_code_is_the_declared_one() {
        use crate::specs::acl::revoke::{v0_1, v0_2, v0_3};
        for declared in [
            v0_1::error_codes::LAST_AUTHORITY_PROTECTED,
            v0_2::error_codes::LAST_AUTHORITY_PROTECTED,
            v0_3::error_codes::LAST_AUTHORITY_PROTECTED,
        ] {
            assert_eq!(declared.code, LAST_AUTHORITY_PROTECTED);
            let parsed: crate::TrustTaskCode = LAST_AUTHORITY_PROTECTED.parse().unwrap();
            assert!(declared.matches(&parsed));
        }
    }

    /// Worked example mirroring webvh's likely usage: AclEntry carries an
    /// ecosystem `ext` field, and the protected predicate checks the
    /// generated AclEntry's `role` directly.
    #[test]
    #[cfg(feature = "acl")]
    fn works_with_codegen_acl_entry_shape() {
        use crate::specs::acl::grant::v0_2 as grant;

        let entry = |subject: &str, role: &str| -> grant::AclEntry {
            serde_json::from_value(serde_json::json!({
                "subject": subject,
                "role": role,
                "act": { "scope": "all" },
                "keys": { "scope": "all" },
                "capabilities": { "scope": "ceiling" }
            }))
            .expect("a valid acl/grant/0.2 entry")
        };

        let admin = entry("did:web:alice.example", "admin");
        let member = entry("did:web:bob.example", "member");
        let entries = [admin, member];

        assert_eq!(
            reject_last_authority(
                entries.iter(),
                |e: &&grant::AclEntry| e.role.as_str() == "admin",
                |e: &&grant::AclEntry| e.subject.as_str() == "did:web:alice.example",
            ),
            Some(LAST_AUTHORITY_PROTECTED)
        );
    }
}
