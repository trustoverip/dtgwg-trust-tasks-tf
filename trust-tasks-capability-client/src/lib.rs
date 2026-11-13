//! Client-side wire helpers for the **capability** Trust Task families —
//! `governance/capability/*` (enable / disable / list a community capability)
//! and `git-trust/*` (grant / revoke commit-signing trust).
//!
//! This crate owns the *documents*, not a transport: it builds request
//! documents, parses inbound envelope replies and classifies them. Signing is
//! left to the caller (see below). Each consumer keeps
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
//! - **Replies**: [`classify_git_trust_reply_to`] (for grant/revoke writers) and
//!   [`parse_capability_reply_to`] (for governance management UIs), which take
//!   the request you sent and accept only a reply that answers it
//!   ([`answers_request`]). The older forms that take only a thread id
//!   ([`classify_git_trust_reply`], [`parse_capability_reply`]) remain; they
//!   cannot check who answered or for which request type.
//!
//! ## Retries versus fresh attempts
//!
//! This crate is a **producer**, and the producer half of SPEC §7.2 item 11 is
//! §8.4: *a retry is a bit-for-bit identical resend*. As of `trust-tasks-rs`
//! 0.12.0 there is a consumer that enforces it — the record is keyed on the
//! document `id` and compared against the whole document — and the DIDComm and
//! TSP bindings now keep that record by default. So the two ways of sending a
//! request again have become genuinely different operations:
//!
//! | Intent | What to send | What the consumer does |
//! |---|---|---|
//! | The first send may not have arrived | `previous` itself, unchanged | Absorbs it; returns whatever the first execution determined |
//! | Something about the request changed | [`new_attempt(&previous)`](new_attempt) | Treats it as the new document it is |
//! | Anything else under a reused `id` | — | Rejects it with `idConflict` |
//!
//! "Something about the request changed" is wider than it sounds: a re-stamped
//! `issuedAt` or a re-signed `proof` over identical content is already a
//! different document. That is deliberate — §8.4 says a producer that
//! "retries" by re-signing "has not retried", and the whole point of item 11's
//! comparison is that an `id` alone cannot tell the retry it must absorb from
//! the conflict it must reject.
//!
//! [`build_document`] and every builder over it mint a fresh `id` per call, so
//! a caller that rebuilds is already minting a new attempt. [`new_attempt`]
//! covers the case where the document has already been built (and possibly
//! signed) and is about to be sent again.
//!
//! Signing is deliberately **not** here — it is a thin Data-Integrity call
//! each consumer makes with its own signer (a service reuses its credential
//! signer; a client signs with the persona key), so this crate stays free of
//! any crypto dependency. Sign the built document over its canonical form
//! (the document minus its `proof` member, `eddsa-jcs-2022`) and set the
//! `proof` member.
//!
//! # Versioning
//!
//! This crate exposes `trust-tasks-rs` types in its own public API, so a
//! breaking change there breaks this crate's callers even when nothing here
//! changes. `cargo-semver-checks` cannot catch that: it compares each crate's
//! rustdoc against that crate's own published baseline, and does not track
//! type identity across dependency versions. The crates that share
//! `trust-tasks-rs` in their public API are therefore released as one
//! compatibility unit with a single shared version — see `version_group` in
//! `release-plz.toml`.

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

/// The extended error code `git-trust/grant` declares for "an active grant
/// already exists for this subject and resource" (SPEC §8.5; the code is
/// declared in the registry entry's `errorCodes` front matter).
///
/// This is the **control surface** for idempotent success on a grant. SPEC
/// §8.2 types `message` as non-normative free text "intended for logs and
/// operator UI"; a client that decides an outcome from it is deciding on a
/// string the emitting service is free to reword, translate, or drop.
pub const GIT_TRUST_ALREADY_GRANTED_CODE: &str = "git-trust/grant:already_granted";

/// The extended error code `git-trust/revoke` declares for "no active grant
/// exists for this subject and resource".
pub const GIT_TRUST_NOT_GRANTED_CODE: &str = "git-trust/revoke:not_granted";

/// The lowerCamelCase spellings of the two codes above.
///
/// The registry entries declare the snake_case forms, which is what a
/// conforming emitter sends today. SPEC §4.10 rule 4 **SHOULD**s lowerCamelCase
/// for specification-defined values, so the registry may normalise; accepting
/// both spellings means that normalisation is not a flag day for this client.
/// Both are namespaced extended codes either way — neither is free text.
pub const GIT_TRUST_ALREADY_GRANTED_CODE_CAMEL: &str = "git-trust/grant:alreadyGranted";
/// See [`GIT_TRUST_ALREADY_GRANTED_CODE_CAMEL`].
pub const GIT_TRUST_NOT_GRANTED_CODE_CAMEL: &str = "git-trust/revoke:notGranted";

/// Errors from document construction.
#[derive(Debug, thiserror::Error)]
pub enum CapabilityClientError {
    #[error("capability document error: {0}")]
    Document(String),
}

/// The registry's authority: where the framework's own specifications, and the
/// capability families this crate builds, are published.
const REGISTRY_SPEC_PREFIX: &str = "https://trusttasks.org/spec/";

// --- builders ----------------------------------------------------------------

/// A fresh document `id`. One per *attempt* — never reused across attempts;
/// see [`new_attempt`] and the [module docs](self#retries-versus-fresh-attempts).
fn fresh_id() -> String {
    format!("urn:uuid:{}", Uuid::new_v4())
}

/// Build a capability Trust Task addressed `issuer` → `recipient`.
///
/// Mints a fresh `id` and stamps `issuedAt` **on every call**, so each built
/// document is a new attempt in the sense of SPEC §8.4. To re-send one you
/// have already built, see [`new_attempt`] — and read
/// [Retries versus fresh attempts](self#retries-versus-fresh-attempts) first,
/// because the choice between resending the identical document and minting a
/// new one is now enforced by the consumer.
///
/// # Panics
///
/// When `type_uri` is not a Trust Task Type URI (SPEC §6.1). Every builder in
/// this crate passes one of its own constants, which always parse; a caller
/// passing its own Type URI should use [`try_build_document`].
pub fn build_document(
    issuer_did: &str,
    recipient_did: &str,
    type_uri: &str,
    payload: Value,
) -> TrustTask<Value> {
    match try_build_document(issuer_did, recipient_did, type_uri, payload) {
        Ok(doc) => doc,
        Err(e) => {
            panic!("build_document: {e}; use try_build_document for a caller-supplied Type URI")
        }
    }
}

/// [`build_document`] for a Type URI that may not parse: an error rather than a
/// panic when `type_uri` is not a Trust Task Type URI (SPEC §6.1).
pub fn try_build_document(
    issuer_did: &str,
    recipient_did: &str,
    type_uri: &str,
    payload: Value,
) -> Result<TrustTask<Value>, CapabilityClientError> {
    let type_uri = type_uri.parse().map_err(|e| {
        CapabilityClientError::Document(format!("`{type_uri}` is not a Trust Task Type URI: {e}"))
    })?;
    let mut doc = TrustTask::new(fresh_id(), type_uri, payload);
    doc.issuer = Some(issuer_did.to_string());
    doc.recipient = Some(recipient_did.to_string());
    doc.issued_at = Some(trust_tasks_rs::issued_at_now());
    Ok(doc)
}

/// A **new attempt** at the request `previous` carried: the same addressing,
/// type and payload under a *fresh* `id`, a fresh `issuedAt`, and no `proof`.
///
/// This is the counterpart of a SPEC §8.4 retry, and the two are not
/// interchangeable:
///
/// * A **retry** is a bit-for-bit identical resend of `previous`. Send
///   `previous` itself — unchanged, same `id`, same `issuedAt`, same `proof`.
///   The consumer's §7.2 item 11 record absorbs it and returns whatever the
///   first execution determined; that absorption is the whole reason retrying
///   is safe.
/// * A **new attempt** is a different document. Anything that changes the
///   bytes makes it one: an edited payload, a re-stamped `issuedAt`, even a
///   re-signed `proof` over identical content. It **MUST** carry a fresh `id`,
///   which is what this function is for.
///
/// Reusing an `id` with altered content used to pass unnoticed. As of
/// `trust-tasks-rs` 0.12.0 the consumer keeps a record keyed on the document
/// `id` and compares the whole document against it, so that combination is
/// rejected with `idConflict` — and, as the DIDComm and TSP bindings now
/// default that record on, it will be rejected by every consumer this client
/// talks to.
///
/// `proof` is cleared because it committed to the previous `id` and
/// `issuedAt`; carrying it over would ship a signature over a document that no
/// longer exists. Sign the returned document before sending it.
///
/// **Hold the new correlation thread.** Where `previous` opened its own
/// exchange (no `threadId`), SPEC §4.9's fallback names that exchange by the
/// document `id` — so a new attempt opens a *new* exchange, and the value to
/// wait on is [`correlation_thread`] of the returned document, not of
/// `previous`. Where `previous` carried an explicit `threadId` it is preserved
/// and the attempt stays in the same exchange.
#[must_use]
pub fn new_attempt(previous: &TrustTask<Value>) -> TrustTask<Value> {
    let mut next = previous.clone();
    next.id = fresh_id();
    next.issued_at = Some(trust_tasks_rs::issued_at_now());
    next.proof = None;
    next
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
///
/// The returned `threadId` is a **dispatch key, not a check**: it tells a
/// caller holding a map of outstanding requests which one this document
/// belongs to. It does not establish that the document is a reply to anything
/// the caller sent. Correlate before acting — either by finding the thread in
/// your own outstanding map, or with [`parse_envelope_document_for`].
pub fn parse_envelope_document(body: &Value) -> Option<(String, TrustTask<Value>)> {
    let doc: TrustTask<Value> = serde_json::from_value(body.clone()).ok()?;
    let thid = doc.thread_id.clone()?;
    Some((thid, doc))
}

/// Parse a DIDComm envelope body into the document it carries, **only** if
/// that document is threaded to `expected_thread_id`.
///
/// `None` covers both "not a Trust Task document" and "a Trust Task document
/// belonging to some other exchange"; in either case it is not an answer to
/// the request you are waiting on, and the correct action is to keep waiting.
pub fn parse_envelope_document_for(
    body: &Value,
    expected_thread_id: &str,
) -> Option<TrustTask<Value>> {
    let (_, doc) = parse_envelope_document(body)?;
    replies_to(&doc, expected_thread_id).then_some(doc)
}

/// The thread an exchange started by `doc` is correlated by: its own
/// `threadId`, or its `id` where it opens the exchange (SPEC §4.9's fallback,
/// which is the value `respond_with` and `reject_with` will thread the reply
/// to).
///
/// Hold this from the moment you send a request; it is what every
/// reply-classifying function here wants as `expected_thread_id`.
pub fn correlation_thread<P>(doc: &TrustTask<P>) -> &str {
    doc.thread_id.as_deref().unwrap_or(&doc.id)
}

/// Whether `reply` is threaded to `expected_thread_id` — SPEC §4.9
/// correlation, and the precondition for acting on any reply.
///
/// A reply with no `threadId` at all matches nothing: §8.1 requires an error
/// response to carry one, and a `#response` gets one from `respond_with`, so
/// its absence means the document is not correlated to any exchange.
pub fn replies_to<P>(reply: &TrustTask<P>, expected_thread_id: &str) -> bool {
    reply.thread_id.as_deref() == Some(expected_thread_id)
}

/// Whether `reply` answers `request` — the four equalities the HTTPS client
/// applies to a response (SPEC §4.9, §8.1), for a client that holds the
/// request it sent:
///
/// | Member | Must be |
/// |---|---|
/// | `threadId` | [`correlation_thread`] of `request` |
/// | `issuer` | `request`'s `recipient`, where the request named one |
/// | `recipient` | `request`'s `issuer`, where the request named one |
/// | `type` | `request`'s `type` with `#response`, or the framework's `trust-task-error` under the registry's authority |
///
/// An error reply whose payload names `inResponseTo.id` must name `request`'s
/// `id` (SPEC §8.1 lets it be absent).
pub fn answers_request(reply: &TrustTask<Value>, request: &TrustTask<Value>) -> bool {
    if !replies_to(reply, correlation_thread(request)) {
        return false;
    }
    if let Some(expected) = request.recipient.as_deref() {
        if reply.issuer.as_deref() != Some(expected) {
            return false;
        }
    }
    if let Some(expected) = request.issuer.as_deref() {
        if reply.recipient.as_deref() != Some(expected) {
            return false;
        }
    }
    if is_framework_error(reply) {
        return match reply.payload.get("inResponseTo").and_then(|r| r.get("id")) {
            None => true,
            Some(id) => id.as_str() == Some(request.id.as_str()),
        };
    }
    reply.type_uri == request.type_uri.with_response()
}

/// Whether `doc`'s type is under the registry's authority.
fn is_registry_type<P>(doc: &TrustTask<P>) -> bool {
    doc.type_uri.to_string().starts_with(REGISTRY_SPEC_PREFIX)
}

/// Whether `doc` is a framework error document: `trust-task-error` under the
/// registry's authority. The same slug under any other authority is some
/// other publisher's specification, not a refusal.
fn is_framework_error<P>(doc: &TrustTask<P>) -> bool {
    doc.type_uri.slug() == "trust-task-error"
        && !doc.type_uri.is_response()
        && is_registry_type(doc)
}

// --- git-trust write replies (grant/revoke producers) ------------------------

/// The classification of a `git-trust` write reply.
///
/// `IdempotentSuccess` is load-bearing for redelivery-safe writers: an
/// `already_granted` / `not_granted` rejection means the desired end state
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

/// How much a caller is willing to infer from a non-conforming peer.
///
/// The default infers nothing: an outcome is decided from the error `code`
/// alone.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReplyPolicy {
    /// **DEPRECATED — opt-in compatibility only, removed in the next MAJOR.**
    ///
    /// Also treat a `taskFailed` whose free-text `message` contains
    /// `already_granted:` or `not_granted:` as [`WriteOutcome::IdempotentSuccess`].
    ///
    /// This is how the classification worked before 0.10.0, and it was wrong:
    /// SPEC §8.2 defines `message` as non-normative free text "intended for
    /// logs and operator UI". Deciding an outcome from it means the client's
    /// behaviour hinges on wording the emitting service may reword, translate
    /// or drop at any time — and, in the other direction, that a *genuine*
    /// `taskFailed` whose operator message happens to quote the phrase is
    /// silently reported to the caller as success.
    ///
    /// Enable this only while a specific peer still emits the free-text form,
    /// and only after confirming there is no code-bearing alternative. The
    /// correct fix is on the emitting side: send
    /// [`GIT_TRUST_ALREADY_GRANTED_CODE`] / [`GIT_TRUST_NOT_GRANTED_CODE`],
    /// which SPEC §8.5 provides for exactly this and the registry entries
    /// already declare.
    pub accept_legacy_free_text_idempotence: bool,
}

impl ReplyPolicy {
    /// The strict policy: the error `code` decides, free text decides nothing.
    pub fn strict() -> Self {
        Self::default()
    }

    /// DEPRECATED: additionally accept the pre-0.10 free-text form. See
    /// [`Self::accept_legacy_free_text_idempotence`].
    pub fn with_legacy_free_text() -> Self {
        Self {
            accept_legacy_free_text_idempotence: true,
        }
    }
}

/// Classify the reply to a `git-trust/grant` or `git-trust/revoke` write.
///
/// `expected_thread_id` is the thread of the request this is supposed to be
/// answering — [`correlation_thread`] of the document you sent. A reply
/// threaded to anything else is not an answer to it, and yields `None` rather
/// than an outcome: acting on an uncorrelated reply means letting whichever
/// document arrives next decide the fate of a write it has nothing to do with.
///
/// `None` therefore means "not an answer to this request" — either a document
/// of another family, or a reply belonging to another exchange. Use
/// [`replies_to`] if you need to tell those apart.
///
/// Idempotent success is keyed on the **extended error code** of SPEC §8.5
/// ([`GIT_TRUST_ALREADY_GRANTED_CODE`], [`GIT_TRUST_NOT_GRANTED_CODE`]), never
/// on the free-text `message`. See
/// [`classify_git_trust_reply_with_policy`] for the deprecated compatibility
/// path.
pub fn classify_git_trust_reply(
    doc: &TrustTask<Value>,
    expected_thread_id: &str,
) -> Option<WriteOutcome> {
    classify_git_trust_reply_with_policy(doc, expected_thread_id, ReplyPolicy::strict())
}

/// [`classify_git_trust_reply`] with an explicit [`ReplyPolicy`].
///
/// Pass [`ReplyPolicy::strict`] unless you are talking to a peer that predates
/// the extended error codes; see
/// [`ReplyPolicy::accept_legacy_free_text_idempotence`].
pub fn classify_git_trust_reply_with_policy(
    doc: &TrustTask<Value>,
    expected_thread_id: &str,
    policy: ReplyPolicy,
) -> Option<WriteOutcome> {
    // SPEC §4.9: correlation comes first. Nothing below is safe to act on for
    // a document that is not answering this request.
    if !replies_to(doc, expected_thread_id) {
        return None;
    }

    let slug = doc.type_uri.slug();
    if is_framework_error(doc) {
        let (code, message) = error_code_and_message(doc);
        if is_idempotent_code(&code) {
            return Some(WriteOutcome::IdempotentSuccess);
        }
        // DEPRECATED: pre-0.10 peers signalled idempotence in the free-text
        // `message` under a bare `taskFailed`. SPEC §8.2 makes `message`
        // non-normative, so this is a string match on a field nobody promised
        // to keep stable — opt-in, and gone in the next MAJOR. Remove this
        // block, `ReplyPolicy`, and the `*_with_policy` entry point together.
        if policy.accept_legacy_free_text_idempotence && code == "taskFailed" {
            let reason = message.as_deref().unwrap_or("");
            if reason.contains("already_granted:") || reason.contains("not_granted:") {
                return Some(WriteOutcome::IdempotentSuccess);
            }
        }
        return Some(WriteOutcome::Rejected { code, message });
    }
    if doc.type_uri.is_response()
        && is_registry_type(doc)
        && matches!(slug, "git-trust/grant" | "git-trust/revoke")
    {
        return Some(WriteOutcome::Success);
    }
    None
}

/// Classify the reply to the `git-trust/grant` or `git-trust/revoke` document
/// `request` — the form to use when you hold the request you sent.
///
/// `None` unless `reply` [answers](answers_request) `request`: threaded to it,
/// from the party it was addressed to, for the party that sent it, and typed as
/// its own `#response` or as a framework error. So a revoke's response does not
/// complete a grant, and a refusal typed under another authority completes
/// nothing. Otherwise as [`classify_git_trust_reply_with_policy`].
pub fn classify_git_trust_reply_to(
    reply: &TrustTask<Value>,
    request: &TrustTask<Value>,
    policy: ReplyPolicy,
) -> Option<WriteOutcome> {
    if !matches!(
        request.type_uri.slug(),
        "git-trust/grant" | "git-trust/revoke"
    ) || !answers_request(reply, request)
    {
        return None;
    }
    classify_git_trust_reply_with_policy(reply, correlation_thread(request), policy)
}

/// Whether `code` is one of the extended codes that mean "the end state you
/// asked for already holds" (SPEC §8.5).
fn is_idempotent_code(code: &str) -> bool {
    matches!(
        code,
        GIT_TRUST_ALREADY_GRANTED_CODE
            | GIT_TRUST_NOT_GRANTED_CODE
            | GIT_TRUST_ALREADY_GRANTED_CODE_CAMEL
            | GIT_TRUST_NOT_GRANTED_CODE_CAMEL
    )
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

/// Parse an inbound envelope body directly into a reply to the request
/// threaded `expected_thread_id` — the entry point for a UI's inbound
/// dispatch, which holds only a `Value`.
///
/// `None` when the body is not a `governance/capability/*` reply, or is one
/// belonging to a different exchange. As on the write side, a reply the caller
/// did not ask for must not be allowed to resolve a request the caller did.
pub fn parse_envelope_reply(body: &Value, expected_thread_id: &str) -> Option<CapabilityReply> {
    let doc = parse_envelope_document_for(body, expected_thread_id)?;
    parse_capability_reply(&doc, expected_thread_id)
}

/// Classify the reply to the `governance/capability/*` document `request` —
/// the form to use when you hold the request you sent.
///
/// `None` unless `reply` [answers](answers_request) `request`; otherwise as
/// [`parse_capability_reply`].
pub fn parse_capability_reply_to(
    reply: &TrustTask<Value>,
    request: &TrustTask<Value>,
) -> Option<CapabilityReply> {
    if !request
        .type_uri
        .slug()
        .starts_with("governance/capability/")
        || !answers_request(reply, request)
    {
        return None;
    }
    parse_capability_reply(reply, correlation_thread(request))
}

/// Classify a `governance/capability/*` reply document. `None` when it is not
/// part of this family, is not threaded to `expected_thread_id`, or does not
/// carry the members its kind requires (a listing's `capabilities` array, each
/// entry's `manifest.capability` and `enabled`; a toggle's `capability` and
/// `enabled`) — a reply that cannot say what it reports is not read as
/// reporting "disabled".
pub fn parse_capability_reply(
    doc: &TrustTask<Value>,
    expected_thread_id: &str,
) -> Option<CapabilityReply> {
    // SPEC §4.9 correlation, as on the write side: a listing or a toggle
    // acknowledgement from another exchange answers nothing here.
    if !replies_to(doc, expected_thread_id) {
        return None;
    }
    let slug = doc.type_uri.slug();
    if is_framework_error(doc) {
        let (code, message) = error_code_and_message(doc);
        return Some(CapabilityReply::Rejected { code, message });
    }
    if !doc.type_uri.is_response() || !is_registry_type(doc) {
        return None;
    }
    match slug {
        "governance/capability/list" => {
            let entries = doc.payload.get("capabilities")?.as_array()?;
            let entries = entries.iter().map(summary_of).collect::<Option<Vec<_>>>()?;
            Some(CapabilityReply::Listing(entries))
        }
        "governance/capability/enable" | "governance/capability/disable" => {
            Some(CapabilityReply::Toggled {
                capability: doc.payload.get("capability")?.as_str()?.to_string(),
                enabled: doc.payload.get("enabled")?.as_bool()?,
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
        enabled: entry.get("enabled")?.as_bool()?,
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

    /// An error response carrying an arbitrary `code`, built the way a
    /// conforming emitter would (SPEC §8.2), so the tests below exercise the
    /// wire form rather than a Rust enum.
    fn error_reply(
        request: &TrustTask<Value>,
        code: &str,
        message: Option<&str>,
    ) -> TrustTask<Value> {
        let mut payload = serde_json::json!({ "code": code, "retryable": false });
        if let Some(message) = message {
            payload["message"] = serde_json::json!(message);
        }
        let mut doc = TrustTask::new(
            "urn:uuid:err".to_string(),
            "https://trusttasks.org/spec/trust-task-error/0.5"
                .parse()
                .unwrap(),
            payload,
        );
        doc.thread_id = Some(correlation_thread(request).to_string());
        doc
    }

    #[test]
    fn git_trust_reply_classification() {
        let grant = build_git_trust_grant("did:a", "did:r", "did:s", "org");
        let thread = correlation_thread(&grant).to_string();

        let ok = grant.respond_with(
            "urn:uuid:r".to_string(),
            serde_json::json!({ "granted": true }),
        );
        assert_eq!(
            classify_git_trust_reply(&ok, &thread),
            Some(WriteOutcome::Success)
        );

        let denied = reserialize(&grant.reject_with(
            "urn:uuid:e2".to_string(),
            RejectReason::PermissionDenied {
                reason: "no".to_string(),
            },
        ));
        assert!(matches!(
            classify_git_trust_reply(&denied, &thread),
            Some(WriteOutcome::Rejected { .. })
        ));
    }

    /// SPEC §8.2 defines `message` as non-normative free text "intended for
    /// logs and operator UI". Before 0.10.0 this client decided idempotent
    /// success from a substring of it, which meant (a) the client's behaviour
    /// was pinned to wording no emitter promised to keep, and (b) a genuine
    /// `taskFailed` whose operator message merely *quoted* the phrase was
    /// reported to the caller as success — a failed write recorded as done.
    ///
    /// The control surface is the namespaced extended code of §8.5, which the
    /// registry entries for `git-trust/grant` and `git-trust/revoke` already
    /// declare.
    #[test]
    fn idempotent_success_is_keyed_on_the_extended_code_not_the_message() {
        let grant = build_git_trust_grant("did:a", "did:r", "did:s", "org");
        let thread = correlation_thread(&grant).to_string();

        // The declared code decides — with no `message` at all.
        let by_code = error_reply(&grant, GIT_TRUST_ALREADY_GRANTED_CODE, None);
        assert_eq!(
            classify_git_trust_reply(&by_code, &thread),
            Some(WriteOutcome::IdempotentSuccess)
        );
        let revoke = build_git_trust_revoke("did:a", "did:r", "did:s", "org", None);
        let revoke_thread = correlation_thread(&revoke).to_string();
        assert_eq!(
            classify_git_trust_reply(
                &error_reply(&revoke, GIT_TRUST_NOT_GRANTED_CODE, None),
                &revoke_thread
            ),
            Some(WriteOutcome::IdempotentSuccess)
        );
        // §4.10's lowerCamelCase spelling, should the registry normalise.
        assert_eq!(
            classify_git_trust_reply(
                &error_reply(&grant, GIT_TRUST_ALREADY_GRANTED_CODE_CAMEL, None),
                &thread
            ),
            Some(WriteOutcome::IdempotentSuccess)
        );

        // The free text does NOT decide. This is the regression: a real
        // failure whose operator message names the condition is a failure.
        let free_text = error_reply(
            &grant,
            "taskFailed",
            Some("registry write aborted; not already_granted: the tuple was never written"),
        );
        assert_eq!(
            classify_git_trust_reply(&free_text, &thread),
            Some(WriteOutcome::Rejected {
                code: "taskFailed".to_string(),
                message: Some(
                    "registry write aborted; not already_granted: the tuple was never written"
                        .to_string()
                ),
            }),
            "a taskFailed whose free text quotes the phrase is still a failure"
        );

        // The deprecated compatibility path is opt-in, and only reachable by
        // asking for it by name.
        assert_eq!(
            classify_git_trust_reply_with_policy(
                &free_text,
                &thread,
                ReplyPolicy::with_legacy_free_text()
            ),
            Some(WriteOutcome::IdempotentSuccess)
        );
        assert_eq!(
            classify_git_trust_reply_with_policy(&free_text, &thread, ReplyPolicy::strict()),
            classify_git_trust_reply(&free_text, &thread),
            "strict is the default"
        );
        assert!(!ReplyPolicy::default().accept_legacy_free_text_idempotence);
    }

    /// A reply must be matched to the request before it is acted on (SPEC
    /// §4.9). Before 0.10.0 the thread was extracted and discarded, so a reply
    /// belonging to a different exchange — including an attacker-chosen one on
    /// a shared inbound path — could resolve a write the caller was still
    /// waiting on.
    #[test]
    fn a_reply_on_another_thread_resolves_nothing() {
        let mine = build_git_trust_grant("did:a", "did:r", "did:s", "org");
        let theirs = build_git_trust_grant("did:a", "did:r", "did:other", "other-org");
        let my_thread = correlation_thread(&mine).to_string();
        assert_ne!(my_thread, correlation_thread(&theirs));

        // A perfectly valid success for somebody else's grant.
        let their_ok = theirs.respond_with(
            "urn:uuid:r".to_string(),
            serde_json::json!({ "granted": true }),
        );
        assert_eq!(
            classify_git_trust_reply(&their_ok, &my_thread),
            None,
            "a reply to another exchange must not resolve this request"
        );
        // And the same document does resolve the request it actually answers.
        assert_eq!(
            classify_git_trust_reply(&their_ok, correlation_thread(&theirs)),
            Some(WriteOutcome::Success)
        );

        // Same for the idempotent-success shortcut, which is the one an
        // attacker would want to forge: it must not be reachable off-thread.
        let their_already = error_reply(&theirs, GIT_TRUST_ALREADY_GRANTED_CODE, None);
        assert_eq!(classify_git_trust_reply(&their_already, &my_thread), None);

        // A reply carrying no thread at all correlates to nothing.
        let mut unthreaded = their_ok.clone();
        unthreaded.thread_id = None;
        assert!(!replies_to(&unthreaded, &my_thread));
        assert_eq!(classify_git_trust_reply(&unthreaded, &my_thread), None);

        // The governance family enforces the same rule.
        let list = build_list_document("did:me", "did:vtc");
        let other_list = build_list_document("did:me", "did:vtc");
        let other_reply = other_list.respond_with(
            "urn:uuid:r".to_string(),
            serde_json::json!({ "capabilities": [] }),
        );
        assert_eq!(
            parse_capability_reply(&other_reply, correlation_thread(&list)),
            None
        );
        assert_eq!(
            parse_envelope_reply(
                &serde_json::to_value(&other_reply).unwrap(),
                correlation_thread(&list)
            ),
            None
        );
    }

    #[test]
    fn governance_reply_classification() {
        let list = build_list_document("did:me", "did:vtc");
        let list_thread = correlation_thread(&list).to_string();
        let reply = list.respond_with(
            "urn:uuid:r".to_string(),
            serde_json::json!({ "capabilities": [{
                "manifest": { "capability": "git-trust", "version": "0.1", "title": "Git Commit Trust" },
                "enabled": true, "enabledAt": "2026-07-18T00:00:00Z"
            }]}),
        );
        let Some(CapabilityReply::Listing(items)) = parse_capability_reply(&reply, &list_thread)
        else {
            panic!("expected listing");
        };
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].slug, "git-trust");
        assert!(items[0].enabled);

        let toggle = build_toggle_document("did:me", "did:vtc", "git-trust", "0.1", true);
        let toggle_thread = correlation_thread(&toggle).to_string();
        let ack = toggle.respond_with(
            "urn:uuid:t".to_string(),
            serde_json::json!({ "capability": "git-trust", "enabled": true }),
        );
        assert_eq!(
            parse_capability_reply(&ack, &toggle_thread),
            Some(CapabilityReply::Toggled {
                capability: "git-trust".to_string(),
                enabled: true
            })
        );
        // And through the envelope entry point a UI actually uses.
        assert_eq!(
            parse_envelope_reply(&serde_json::to_value(&ack).unwrap(), &toggle_thread),
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

        // The correlating form takes the same body only for the right thread.
        assert!(parse_envelope_document_for(&body, &grant.id).is_some());
        assert!(parse_envelope_document_for(&body, "urn:uuid:someone-else").is_none());
    }

    // --- binding a reply to the request it answers ---------------------------

    /// `reply` re-addressed as a different party would send it.
    fn from_party(mut reply: TrustTask<Value>, issuer: &str) -> TrustTask<Value> {
        reply.issuer = Some(issuer.to_string());
        reply
    }

    #[test]
    fn a_reply_answers_only_the_request_it_was_made_for() {
        let grant = build_git_trust_grant("did:a", "did:r", "did:s", "org");
        let strict = ReplyPolicy::strict();

        let ok = grant.respond_with("urn:uuid:r".to_string(), serde_json::json!({}));
        assert!(answers_request(&ok, &grant));
        assert_eq!(
            classify_git_trust_reply_to(&ok, &grant, strict),
            Some(WriteOutcome::Success)
        );

        // From someone other than the registry it was sent to.
        let elsewhere = from_party(ok.clone(), "did:someone-else");
        assert!(!answers_request(&elsewhere, &grant));
        assert_eq!(
            classify_git_trust_reply_to(&elsewhere, &grant, strict),
            None
        );

        // For someone other than the sender.
        let mut misaddressed = ok.clone();
        misaddressed.recipient = Some("did:someone-else".to_string());
        assert_eq!(
            classify_git_trust_reply_to(&misaddressed, &grant, strict),
            None
        );

        // A revoke's response on the grant's thread does not complete the
        // grant, though the thread-only form cannot tell.
        let mut revoke_ack = ok.clone();
        revoke_ack.type_uri = format!("{GIT_TRUST_REVOKE_TYPE}#response").parse().unwrap();
        assert_eq!(
            classify_git_trust_reply_to(&revoke_ack, &grant, strict),
            None
        );

        // An idempotent refusal from the registry, naming this request, is
        // still an idempotent success.
        let mut already = error_reply(&grant, GIT_TRUST_ALREADY_GRANTED_CODE, None);
        already.issuer = grant.recipient.clone();
        already.recipient = grant.issuer.clone();
        assert_eq!(
            classify_git_trust_reply_to(&already, &grant, strict),
            Some(WriteOutcome::IdempotentSuccess)
        );
        // ... but not when it names another request.
        let mut other = already.clone();
        other.payload["inResponseTo"] = serde_json::json!({ "id": "urn:uuid:another" });
        assert_eq!(classify_git_trust_reply_to(&other, &grant, strict), None);
        other.payload["inResponseTo"] = serde_json::json!({ "id": grant.id });
        assert_eq!(
            classify_git_trust_reply_to(&other, &grant, strict),
            Some(WriteOutcome::IdempotentSuccess)
        );

        // The governance family is bound the same way.
        let toggle = build_toggle_document("did:me", "did:vtc", "git-trust", "0.1", true);
        let ack = toggle.respond_with(
            "urn:uuid:t".to_string(),
            serde_json::json!({ "capability": "git-trust", "enabled": true }),
        );
        assert!(parse_capability_reply_to(&ack, &toggle).is_some());
        assert_eq!(
            parse_capability_reply_to(&from_party(ack, "did:someone-else"), &toggle),
            None
        );
        // A git-trust request is not classified as a governance one, nor the
        // reverse.
        assert_eq!(parse_capability_reply_to(&already, &grant), None);
        let list = build_list_document("did:me", "did:vtc");
        let listing = list.respond_with(
            "urn:uuid:l".to_string(),
            serde_json::json!({ "capabilities": [] }),
        );
        assert_eq!(classify_git_trust_reply_to(&listing, &list, strict), None);
    }

    /// `trust-task-error` under another authority is another publisher's
    /// specification, not a framework refusal, and a `#response` under another
    /// authority is not this family's.
    #[test]
    fn only_the_registry_authority_is_read_as_this_family_or_an_error() {
        let grant = build_git_trust_grant("did:a", "did:r", "did:s", "org");
        let thread = correlation_thread(&grant).to_string();

        let mut foreign_error = error_reply(&grant, GIT_TRUST_ALREADY_GRANTED_CODE, None);
        foreign_error.type_uri = "https://elsewhere.example/spec/trust-task-error/9.9"
            .parse()
            .unwrap();
        assert_eq!(classify_git_trust_reply(&foreign_error, &thread), None);

        let mut foreign_ack = grant.respond_with("urn:uuid:r".to_string(), serde_json::json!({}));
        foreign_ack.type_uri = "https://elsewhere.example/spec/git-trust/grant/0.1#response"
            .parse()
            .unwrap();
        assert_eq!(classify_git_trust_reply(&foreign_ack, &thread), None);

        let list = build_list_document("did:me", "did:vtc");
        let mut foreign_rejection = error_reply(&list, "permissionDenied", None);
        foreign_rejection.type_uri = "https://elsewhere.example/spec/trust-task-error/0.5"
            .parse()
            .unwrap();
        assert_eq!(
            parse_capability_reply(&foreign_rejection, correlation_thread(&list)),
            None
        );
    }

    /// A reply that does not carry what its kind reports is not read as
    /// reporting a default ("disabled", "no capabilities").
    #[test]
    fn a_malformed_governance_reply_reports_nothing() {
        let toggle = build_toggle_document("did:me", "did:vtc", "git-trust", "0.1", true);
        let thread = correlation_thread(&toggle).to_string();
        for payload in [
            serde_json::json!({ "capability": "git-trust" }),
            serde_json::json!({ "capability": "git-trust", "enabled": "yes" }),
            serde_json::json!({ "enabled": true }),
            serde_json::json!({ "capability": 7, "enabled": false }),
        ] {
            let reply = toggle.respond_with("urn:uuid:t".to_string(), payload.clone());
            assert_eq!(parse_capability_reply(&reply, &thread), None, "{payload}");
        }

        let list = build_list_document("did:me", "did:vtc");
        let thread = correlation_thread(&list).to_string();
        for payload in [
            serde_json::json!({}),
            serde_json::json!({ "capabilities": {} }),
            serde_json::json!({ "capabilities": [{ "manifest": { "capability": "x" } }] }),
            serde_json::json!({ "capabilities": [{ "enabled": true }] }),
        ] {
            let reply = list.respond_with("urn:uuid:l".to_string(), payload.clone());
            assert_eq!(parse_capability_reply(&reply, &thread), None, "{payload}");
        }
        let empty = list.respond_with(
            "urn:uuid:l".to_string(),
            serde_json::json!({ "capabilities": [] }),
        );
        assert_eq!(
            parse_capability_reply(&empty, &thread),
            Some(CapabilityReply::Listing(vec![]))
        );
    }

    #[test]
    fn a_caller_supplied_type_uri_is_an_error_not_a_panic() {
        let err = try_build_document("did:a", "did:b", "not a type uri", serde_json::json!({}))
            .unwrap_err();
        assert!(
            err.to_string().contains("not a Trust Task Type URI"),
            "{err}"
        );
        let doc = try_build_document(
            "did:a",
            "did:b",
            GIT_TRUST_GRANT_TYPE,
            serde_json::json!({}),
        )
        .unwrap();
        assert_eq!(doc.type_uri.slug(), "git-trust/grant");
    }

    // --- SPEC §8.4: retries versus fresh attempts ---------------------------

    /// Every builder mints its own `id`, so rebuilding a request is already a
    /// new attempt rather than a reuse.
    #[test]
    fn builders_mint_a_fresh_id_per_attempt() {
        let first = build_git_trust_grant("did:a", "did:r", "did:s", "openvtc");
        let second = build_git_trust_grant("did:a", "did:r", "did:s", "openvtc");
        assert_ne!(first.id, second.id);
        assert!(first.id.starts_with("urn:uuid:"));
    }

    /// A new attempt is a *different document*: fresh `id`, fresh `issuedAt`,
    /// and no carried-over `proof` — the old one committed to the old `id`.
    #[test]
    fn a_new_attempt_mints_a_fresh_id_and_drops_the_stale_proof() {
        let mut first = build_git_trust_grant("did:a", "did:r", "did:s", "openvtc");
        first.proof = Some(trust_tasks_rs::Proof {
            proof_type: "DataIntegrityProof".into(),
            cryptosuite: "eddsa-jcs-2022".into(),
            created: chrono::Utc::now(),
            proof_purpose: "assertionMethod".into(),
            verification_method: "did:a#key-1".into(),
            proof_value: "zStale".into(),
            extra: Default::default(),
        });

        let next = new_attempt(&first);
        assert_ne!(next.id, first.id, "a new attempt MUST NOT reuse the `id`");
        assert!(next.proof.is_none(), "the old proof signed the old `id`");
        assert_eq!(next.payload, first.payload);
        assert_eq!(next.issuer, first.issuer);
        assert_eq!(next.recipient, first.recipient);
        assert_eq!(next.type_uri.to_string(), first.type_uri.to_string());
    }

    /// SPEC §4.9's fallback: a request that opens its own exchange is named by
    /// its `id`, so a new attempt opens a new exchange and the caller must
    /// hold the *new* correlation thread. An explicit `threadId` is preserved.
    #[test]
    fn a_new_attempt_re_threads_only_where_the_id_was_the_thread() {
        let opening = build_git_trust_grant("did:a", "did:r", "did:s", "openvtc");
        let next = new_attempt(&opening);
        assert_eq!(correlation_thread(&next), next.id);
        assert_ne!(correlation_thread(&next), correlation_thread(&opening));

        let mut in_exchange = build_git_trust_grant("did:a", "did:r", "did:s", "openvtc");
        in_exchange.thread_id = Some("exchange-0001".into());
        let next = new_attempt(&in_exchange);
        assert_eq!(correlation_thread(&next), "exchange-0001");
    }

    /// The end-to-end producer property, checked against the consumer rule
    /// that now enforces it (`trust-tasks-rs` 0.12's §7.2 item 11 record).
    ///
    /// * resending the identical document is absorbed — that is the §8.4 retry;
    /// * altering it while reusing the `id` is `Conflict` → `idConflict`;
    /// * `new_attempt` is the way through, because its `id` is fresh.
    #[tokio::test]
    async fn a_reused_id_with_altered_content_conflicts_and_new_attempt_does_not() {
        use trust_tasks_rs::{document_digest, InMemoryReplayGuard, ReplayGuard, ReplayVerdict};

        let guard = InMemoryReplayGuard::new(16);
        let now = chrono::Utc::now();
        let retain = Some(now + chrono::TimeDelta::minutes(5));

        let sent = build_git_trust_grant("did:a", "did:r", "did:s", "openvtc");
        let digest = document_digest(&sent).unwrap();
        assert_eq!(
            guard.claim(&sent.id, &digest, retain, now).await.unwrap(),
            ReplayVerdict::Fresh
        );

        // §8.4 retry: the identical document, resent. Absorbed.
        let retried = sent.clone();
        let retried_digest = document_digest(&retried).unwrap();
        assert_eq!(retried_digest, digest, "a retry is bit-for-bit identical");
        assert!(matches!(
            guard
                .claim(&retried.id, &retried_digest, retain, now)
                .await
                .unwrap(),
            ReplayVerdict::Duplicate { .. }
        ));

        // The mistake this release closes: edit the payload, keep the `id`.
        let mut altered = sent.clone();
        altered.payload["resource"] = serde_json::json!("some-other-org");
        let altered_digest = document_digest(&altered).unwrap();
        assert_eq!(
            guard
                .claim(&altered.id, &altered_digest, retain, now)
                .await
                .unwrap(),
            ReplayVerdict::Conflict,
            "a reused `id` with altered content is `idConflict`, not a retry"
        );

        // `new_attempt` is how a producer sends a changed request instead.
        let attempt = new_attempt(&altered);
        let attempt_digest = document_digest(&attempt).unwrap();
        assert_eq!(
            guard
                .claim(&attempt.id, &attempt_digest, retain, now)
                .await
                .unwrap(),
            ReplayVerdict::Fresh
        );
    }
}
