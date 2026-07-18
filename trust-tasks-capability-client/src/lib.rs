//! Client-side wire helpers for the **capability** Trust Task families —
//! `governance/capability/*` (enable / disable / list a community capability)
//! and `git-trust/*` (grant / revoke commit-signing trust).
//!
//! This crate owns the *documents*, not a transport: it builds request
//! documents, parses inbound envelope replies, classifies them, and (behind
//! the `signing` feature) attaches a Data-Integrity proof. Each consumer keeps
//! its own send/receive plumbing but shares this wire layer, so a capability
//! producer (a community service) and a management UI cannot drift on the
//! contract.
//!
//! ## Layers
//!
//! - **Envelope**: capability documents travel as the `trust-tasks-didcomm`
//!   binding envelope ([`TRUST_TASK_ENVELOPE_TYPE`]); [`parse_envelope_document`]
//!   turns an inbound body into `(threadId, document)`.
//! - **Builders**: [`build_document`] plus the family-specific
//!   [`build_list_document`], [`build_toggle_document`],
//!   [`build_git_trust_grant`], [`build_git_trust_revoke`].
//! - **Replies**: [`classify_git_trust_reply`] (for grant/revoke writers) and
//!   [`parse_capability_reply`] (for governance management UIs).
//!
//! Signing is deliberately **not** here — it is a thin Data-Integrity call
//! each consumer makes with its own signer (a service reuses its credential
//! signer; a client signs with the persona key), so this crate stays free of
//! any crypto dependency. Sign the built document over its canonical form
//! (the document minus its `proof` member, `eddsa-jcs-2022`) and set the
//! `proof` member.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use trust_tasks_rs::TrustTask;
use uuid::Uuid;

/// The `trust-tasks-didcomm` binding envelope type (what a registry's DIDComm
/// Trust Task handler listens for).
pub const TRUST_TASK_ENVELOPE_TYPE: &str = "https://trusttasks.org/binding/didcomm/0.1/envelope";

/// `governance/capability/*` type URIs.
pub const CAPABILITY_LIST_TYPE: &str = "https://trusttasks.org/spec/governance/capability/list/0.1";
pub const CAPABILITY_ENABLE_TYPE: &str =
    "https://trusttasks.org/spec/governance/capability/enable/0.1";
pub const CAPABILITY_DISABLE_TYPE: &str =
    "https://trusttasks.org/spec/governance/capability/disable/0.1";

/// `git-trust/*` type URIs.
pub const GIT_TRUST_GRANT_TYPE: &str = "https://trusttasks.org/spec/git-trust/grant/0.1";
pub const GIT_TRUST_REVOKE_TYPE: &str = "https://trusttasks.org/spec/git-trust/revoke/0.1";

/// Errors from document construction.
#[derive(Debug, thiserror::Error)]
pub enum CapabilityClientError {
    #[error("capability document error: {0}")]
    Document(String),
}

// --- builders ----------------------------------------------------------------

/// Build a capability Trust Task addressed `issuer` → `recipient`.
pub fn build_document(
    issuer_did: &str,
    recipient_did: &str,
    type_uri: &str,
    payload: Value,
) -> TrustTask<Value> {
    let type_uri = type_uri
        .parse()
        .unwrap_or_else(|_| unreachable!("static capability type URIs are valid"));
    let mut doc = TrustTask::new(format!("urn:uuid:{}", Uuid::new_v4()), type_uri, payload);
    doc.issuer = Some(issuer_did.to_string());
    doc.recipient = Some(recipient_did.to_string());
    doc.issued_at = Some(chrono::Utc::now());
    doc
}

/// Build a `governance/capability/list` request (status `all`).
pub fn build_list_document(issuer_did: &str, vtc_did: &str) -> TrustTask<Value> {
    build_document(
        issuer_did,
        vtc_did,
        CAPABILITY_LIST_TYPE,
        serde_json::json!({ "status": "all" }),
    )
}

/// Build a `governance/capability/enable` or `/disable` request. On enable,
/// `config.authority` defaults to the community's own DID — the community is
/// the authority its capability records are issued under.
pub fn build_toggle_document(
    issuer_did: &str,
    vtc_did: &str,
    slug: &str,
    version: &str,
    enable: bool,
) -> TrustTask<Value> {
    if enable {
        build_document(
            issuer_did,
            vtc_did,
            CAPABILITY_ENABLE_TYPE,
            serde_json::json!({
                "capability": slug,
                "version": version,
                "config": { "authority": vtc_did },
            }),
        )
    } else {
        build_document(
            issuer_did,
            vtc_did,
            CAPABILITY_DISABLE_TYPE,
            serde_json::json!({ "capability": slug }),
        )
    }
}

/// Build a `git-trust/grant`: grant `subject` commit-signing trust for
/// `resource` (an org or `org/repo` slug).
pub fn build_git_trust_grant(
    authority_did: &str,
    registry_did: &str,
    subject_did: &str,
    resource: &str,
) -> TrustTask<Value> {
    build_document(
        authority_did,
        registry_did,
        GIT_TRUST_GRANT_TYPE,
        serde_json::json!({ "subject": subject_did, "resource": resource }),
    )
}

/// Build a `git-trust/revoke`.
pub fn build_git_trust_revoke(
    authority_did: &str,
    registry_did: &str,
    subject_did: &str,
    resource: &str,
    reason: Option<&str>,
) -> TrustTask<Value> {
    let mut payload = serde_json::json!({ "subject": subject_did, "resource": resource });
    if let Some(reason) = reason {
        payload["reason"] = serde_json::json!(reason);
    }
    build_document(authority_did, registry_did, GIT_TRUST_REVOKE_TYPE, payload)
}

// --- envelope parsing --------------------------------------------------------

/// Parse a DIDComm envelope body into `(threadId, document)`. `None` when the
/// body is not a threaded Trust Task document.
pub fn parse_envelope_document(body: &Value) -> Option<(String, TrustTask<Value>)> {
    let doc: TrustTask<Value> = serde_json::from_value(body.clone()).ok()?;
    let thid = doc.thread_id.clone()?;
    Some((thid, doc))
}

// --- git-trust write replies (grant/revoke producers) ------------------------

/// The classification of a `git-trust` write reply.
///
/// `IdempotentSuccess` is load-bearing for redelivery-safe writers:
/// `already_granted`/`not_granted` rejections mean the desired end state
/// already holds, so the write is done, not failed.
#[derive(Debug, Clone, PartialEq)]
pub enum WriteOutcome {
    /// The `#response` document acknowledged the write.
    Success,
    /// Rejected because the end state already holds.
    IdempotentSuccess,
    /// Any other rejection: the machine-readable code and human detail.
    Rejected {
        code: String,
        message: Option<String>,
    },
}

/// Classify the reply to a `git-trust/grant` or `git-trust/revoke` write.
/// `None` when `doc` is not a reply to this family.
pub fn classify_git_trust_reply(doc: &TrustTask<Value>) -> Option<WriteOutcome> {
    let slug = doc.type_uri.slug();
    if slug == "trust-task-error" {
        let (code, message) = error_code_and_message(doc);
        let reason = message.as_deref().unwrap_or("");
        if code == "taskFailed"
            && (reason.contains("already_granted:") || reason.contains("not_granted:"))
        {
            return Some(WriteOutcome::IdempotentSuccess);
        }
        return Some(WriteOutcome::Rejected { code, message });
    }
    if doc.type_uri.is_response() && matches!(slug, "git-trust/grant" | "git-trust/revoke") {
        return Some(WriteOutcome::Success);
    }
    None
}

// --- governance/capability replies (management UIs) --------------------------

/// One capability entry as rendered by a management UI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilitySummary {
    pub slug: String,
    pub title: Option<String>,
    pub version: String,
    pub enabled: bool,
    pub enabled_at: Option<String>,
    pub delegate: Option<String>,
    /// The full manifest, for a detail view.
    pub manifest: Value,
}

/// The classification of a `governance/capability/*` reply.
#[derive(Debug, Clone, PartialEq)]
pub enum CapabilityReply {
    /// A `list` response: the community's capabilities.
    Listing(Vec<CapabilitySummary>),
    /// An `enable`/`disable` acknowledgement.
    Toggled { capability: String, enabled: bool },
    /// A `trust-task-error` document.
    Rejected {
        code: String,
        message: Option<String>,
    },
}

/// Parse an inbound envelope body directly into `(threadId, reply)` — the
/// entry point for a UI's inbound dispatch, which holds only a `Value`.
pub fn parse_envelope_reply(body: &Value) -> Option<(String, CapabilityReply)> {
    let (thid, doc) = parse_envelope_document(body)?;
    let reply = parse_capability_reply(&doc)?;
    Some((thid, reply))
}

/// Classify a `governance/capability/*` reply document. `None` when it is not
/// part of this family.
pub fn parse_capability_reply(doc: &TrustTask<Value>) -> Option<CapabilityReply> {
    let slug = doc.type_uri.slug();
    if slug == "trust-task-error" {
        let (code, message) = error_code_and_message(doc);
        return Some(CapabilityReply::Rejected { code, message });
    }
    if !doc.type_uri.is_response() {
        return None;
    }
    match slug {
        "governance/capability/list" => {
            let entries = doc
                .payload
                .get("capabilities")
                .and_then(Value::as_array)
                .map(|entries| entries.iter().filter_map(summary_of).collect())
                .unwrap_or_default();
            Some(CapabilityReply::Listing(entries))
        }
        "governance/capability/enable" | "governance/capability/disable" => {
            Some(CapabilityReply::Toggled {
                capability: doc
                    .payload
                    .get("capability")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                enabled: doc
                    .payload
                    .get("enabled")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            })
        }
        _ => None,
    }
}

fn error_code_and_message(doc: &TrustTask<Value>) -> (String, Option<String>) {
    let code = doc
        .payload
        .get("code")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let message = doc
        .payload
        .get("message")
        .and_then(Value::as_str)
        .map(str::to_string);
    (code, message)
}

fn summary_of(entry: &Value) -> Option<CapabilitySummary> {
    let manifest = entry.get("manifest")?.clone();
    Some(CapabilitySummary {
        slug: manifest.get("capability")?.as_str()?.to_string(),
        title: manifest
            .get("title")
            .and_then(Value::as_str)
            .map(str::to_string),
        version: manifest
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or("?")
            .to_string(),
        enabled: entry
            .get("enabled")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        enabled_at: entry
            .get("enabledAt")
            .and_then(Value::as_str)
            .map(str::to_string),
        delegate: entry
            .get("delegate")
            .and_then(Value::as_str)
            .map(str::to_string),
        manifest,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use trust_tasks_rs::RejectReason;

    #[test]
    fn builders_are_addressed_and_typed() {
        let list = build_list_document("did:example:me", "did:example:vtc");
        assert_eq!(list.type_uri.slug(), "governance/capability/list");
        assert_eq!(list.issuer.as_deref(), Some("did:example:me"));
        assert_eq!(list.payload["status"], "all");

        let enable = build_toggle_document(
            "did:example:me",
            "did:example:vtc",
            "git-trust",
            "0.1",
            true,
        );
        assert_eq!(enable.payload["config"]["authority"], "did:example:vtc");
        let disable = build_toggle_document(
            "did:example:me",
            "did:example:vtc",
            "git-trust",
            "0.1",
            false,
        );
        assert_eq!(disable.type_uri.slug(), "governance/capability/disable");

        let grant = build_git_trust_grant("did:a", "did:r", "did:s", "openvtc");
        assert_eq!(grant.type_uri.slug(), "git-trust/grant");
        assert_eq!(grant.payload["subject"], "did:s");
        let revoke = build_git_trust_revoke("did:a", "did:r", "did:s", "openvtc", Some("ended"));
        assert_eq!(revoke.payload["reason"], "ended");
    }

    fn reserialize(doc: &trust_tasks_rs::ErrorResponse) -> TrustTask<Value> {
        serde_json::from_value(serde_json::to_value(doc).unwrap()).unwrap()
    }

    #[test]
    fn git_trust_reply_classification() {
        let grant = build_git_trust_grant("did:a", "did:r", "did:s", "org");
        let ok = grant.respond_with(
            "urn:uuid:r".to_string(),
            serde_json::json!({ "granted": true }),
        );
        assert_eq!(classify_git_trust_reply(&ok), Some(WriteOutcome::Success));

        let already = reserialize(&grant.reject_with(
            "urn:uuid:e".to_string(),
            RejectReason::TaskFailed {
                reason: "already_granted: exists".to_string(),
                details: None,
            },
        ));
        assert_eq!(
            classify_git_trust_reply(&already),
            Some(WriteOutcome::IdempotentSuccess)
        );

        let denied = reserialize(&grant.reject_with(
            "urn:uuid:e2".to_string(),
            RejectReason::PermissionDenied {
                reason: "no".to_string(),
            },
        ));
        assert!(matches!(
            classify_git_trust_reply(&denied),
            Some(WriteOutcome::Rejected { .. })
        ));
    }

    #[test]
    fn governance_reply_classification() {
        let list = build_list_document("did:me", "did:vtc");
        let reply = list.respond_with(
            "urn:uuid:r".to_string(),
            serde_json::json!({ "capabilities": [{
                "manifest": { "capability": "git-trust", "version": "0.1", "title": "Git Commit Trust" },
                "enabled": true, "enabledAt": "2026-07-18T00:00:00Z"
            }]}),
        );
        let Some(CapabilityReply::Listing(items)) = parse_capability_reply(&reply) else {
            panic!("expected listing");
        };
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].slug, "git-trust");
        assert!(items[0].enabled);

        let toggle = build_toggle_document("did:me", "did:vtc", "git-trust", "0.1", true);
        let ack = toggle.respond_with(
            "urn:uuid:t".to_string(),
            serde_json::json!({ "capability": "git-trust", "enabled": true }),
        );
        assert_eq!(
            parse_capability_reply(&ack),
            Some(CapabilityReply::Toggled {
                capability: "git-trust".to_string(),
                enabled: true
            })
        );
    }

    #[test]
    fn envelope_parse_requires_thread_id() {
        let grant = build_git_trust_grant("did:a", "did:r", "did:s", "org");
        let reply = grant.respond_with("urn:uuid:r".to_string(), serde_json::json!({}));
        let body = serde_json::to_value(&reply).unwrap();
        let (thid, _) = parse_envelope_document(&body).unwrap();
        assert_eq!(thid, grant.id);
        assert!(parse_envelope_document(&serde_json::to_value(&grant).unwrap()).is_none());
    }
}
