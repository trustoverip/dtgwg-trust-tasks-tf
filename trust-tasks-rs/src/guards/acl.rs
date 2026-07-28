//! ACL policy guards.
//!
//! Pure functions a maintainer calls from inside an `acl/*` handler to
//! enforce policy checks the spec names but does not implement. Each
//! returns an `Option<&'static str>` where `Some(error_code)` is the
//! extended error code the maintainer should respond with — slot it
//! straight into [`RejectReason::TaskFailed`](crate::RejectReason::TaskFailed)
//! or directly into the error payload's `code`.

/// Detect whether revoking `subject` from the entries iterator would
/// empty the maintainer's privileged-role set, per the
/// `acl/revoke:last_authority_protected` error code declared by
/// [`acl/revoke/0.1`](https://trusttasks.org/spec/acl/revoke/0.1).
///
/// `entries` yields the maintainer's current ACL entries (or any
/// ecosystem-flavoured subtype that exposes role + subject identity via
/// the two closures). `is_protected` returns `true` for entries whose
/// role is part of the "must always have at least one of" set
/// (typically `admin`, `owner`, …). `is_target_subject` returns `true`
/// for the entry being revoked.
///
/// Returns `Some("acl/revoke:last_authority_protected")` when the
/// revocation would leave zero protected entries; otherwise `None`.
///
/// The function is generic in the entry type so consumers that carry
/// ecosystem extension fields (webvh's `AclEntry.ext.vnd.affinidi.webvh.*`,
/// etc.) pass their own type without conversion.
///
/// ```rust
/// use trust_tasks_rs::guards::acl::reject_last_authority;
///
/// struct LocalEntry { subject: &'static str, role: &'static str }
///
/// let entries = [
///     LocalEntry { subject: "did:web:alice.example", role: "admin" },
///     LocalEntry { subject: "did:web:bob.example",   role: "member" },
/// ];
///
/// // Revoking the only admin trips the guard.
/// assert_eq!(
///     reject_last_authority(
///         entries.iter(),
///         |e: &&LocalEntry| e.role == "admin",
///         |e: &&LocalEntry| e.subject == "did:web:alice.example",
///     ),
///     Some("acl/revoke:last_authority_protected"),
/// );
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
        Some("acl/revoke:last_authority_protected")
    } else {
        None
    }
}

#[cfg(test)]
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
            Some("acl/revoke:last_authority_protected")
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

    /// Worked example mirroring webvh's likely usage: AclEntry carries an
    /// ecosystem `ext` field, and the protected predicate checks the
    /// generated AclEntry's `role` directly.
    #[test]
    fn works_with_codegen_acl_entry_shape() {
        use crate::specs::acl::grant::v0_1 as grant;

        let admin = grant::AclEntry {
            subject: "did:web:alice.example".into(),
            role: "admin".into(),
            scopes: vec![],
            label: None,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
            expires_at: None,
            approve: None,
            step_up: None,
            ext: None,
        };
        let member = grant::AclEntry {
            subject: "did:web:bob.example".into(),
            role: "member".into(),
            scopes: vec![],
            label: None,
            created_at: None,
            created_by: None,
            updated_at: None,
            updated_by: None,
            expires_at: None,
            approve: None,
            step_up: None,
            ext: None,
        };
        let entries = [admin, member];

        assert_eq!(
            reject_last_authority(
                entries.iter(),
                |e: &&grant::AclEntry| e.role == "admin",
                |e: &&grant::AclEntry| e.subject == "did:web:alice.example",
            ),
            Some("acl/revoke:last_authority_protected")
        );
    }
}
