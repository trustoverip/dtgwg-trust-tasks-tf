//! [`GuardedConsumer`] — the inbound half every message-transport binding
//! shares: the SPEC §7.2 pipeline with the duplicate-execution record of item
//! 11 and the freshness bound it needs already wired, plus the answer to an
//! authenticated body that does not parse.

use std::future::Future;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

use crate::async_dispatch::AsyncDispatcher;
use crate::consume::{
    consume_dispatched, consume_inbound, ConsumeChecks, ConsumeOutcome, PayloadPolicy,
    PayloadValidator, ProofPolicy,
};
use crate::document::{ErrorResponse, TrustTask};
use crate::error::RejectReason;
use crate::freshness::FreshnessPolicy;
use crate::payload::Payload;
use crate::proof::ProofVerifier;
use crate::replay::{InMemoryReplayGuard, ReplayGuard, ReplayPolicy};
use crate::transport::{ResolvedParties, TransportHandler};
use crate::type_uri::TypeUri;

/// Where a [`GuardedConsumer`] keeps the SPEC §7.2 item 11 record.
enum Record {
    /// The batteries-included default: an in-process, bounded record.
    InProcess(InMemoryReplayGuard),
    /// A caller-supplied guard — the shape a replicated deployment needs.
    Shared(Arc<dyn ReplayGuard>),
    /// No record at all. Conformant only where the task is not
    /// *consequential*; see [`GuardedConsumer::without_replay_record`].
    Disabled,
}

/// A body a transport authenticated that cannot be answered with a
/// `trust-task-error` document.
///
/// Returned by [`GuardedConsumer::consume_body`] and
/// [`GuardedConsumer::consume_body_dispatched`] in place of a
/// [`ConsumeOutcome`]. A binding maps it to its own transport error: there is
/// no document to emit.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum UnanswerableBody {
    /// The body did not parse as a document, and it carries no string `id`
    /// and Trust Task `type` an error response could name in `inResponseTo`
    /// (SPEC §8.2).
    #[error("body is not a Trust Task document: {0}")]
    Unparsable(#[source] serde_json::Error),

    /// The body was refused before the pipeline, and the transport
    /// authenticated no sender to address the refusal to. Answering the
    /// in-band `issuer` instead would send an error to a party nothing has
    /// authenticated.
    #[error("refused ({0}) with no transport-authenticated sender to answer")]
    NoAuthenticatedSender(RejectReason),
}

/// The inbound half of a message-transport binding, with SPEC §7.2's two
/// stateful checks — item 11's duplicate-execution record and the freshness
/// bound that lets the record be dropped — already wired onto it.
///
/// Construct one and keep it: it holds the record.
///
/// # Why this type exists
///
/// The DIDComm v2.1, DIDComm v1 and TSP binding crates each carried a copy of
/// the same wrapper — where the record lives, the freshness policy, and the
/// calls into [`consume_inbound`] and [`consume_dispatched`] — differing only
/// in the names (#1096). They now hold a `GuardedConsumer` and keep only what
/// is theirs: opening the envelope, and the envelope-specific checks
/// (DIDComm's thread headers).
///
/// # Keyed on the document `id`, never on the transport identifier
///
/// SPEC §7.2 (*Keying and comparison for item 11*): the key is the document
/// `id` alone, and "transport request identifiers, transport message
/// identifiers, and execution handles **MUST NOT** substitute". A mediator's
/// or an intermediary's re-delivery is a *fresh* transport message carrying
/// the *same* document. [`consume_inbound`] takes the `id` from the document
/// it was handed, so no transport identifier is ever in a position to
/// substitute.
///
/// # Defaults
///
/// [`GuardedConsumer::new`] keeps the record **on**, backed by an in-process
/// [`InMemoryReplayGuard`], and applies [`FreshnessPolicy::consequential`]
/// (`issuedAt` required, five-minute acceptance window). The opt-out,
/// [`GuardedConsumer::without_replay_record`], names what it gives up.
///
/// # One consumer per process, not one per message
///
/// The guard **is** the record. A consumer constructed per inbound message has
/// an empty record every time and absorbs nothing — build one and keep it for
/// the lifetime of the process, or hand every replica a shared store with
/// [`GuardedConsumer::with_replay_guard`].
///
/// # An authenticated body that does not parse is answered
///
/// [`consume_body`](GuardedConsumer::consume_body) and
/// [`consume_body_dispatched`](GuardedConsumer::consume_body_dispatched) take
/// the body a binding has opened — authenticated by the transport, not yet
/// parsed as a document. A body that does not deserialise, or that an
/// envelope-specific check refuses, is answered `malformedRequest` (every
/// binding's §4) as [`ConsumeOutcome::Rejected`], addressed to the
/// **transport-authenticated sender** and never to an in-band `issuer` nothing
/// has yet checked. It is refused before the record is written, so it never
/// burns its `id`. Only a body with nothing to answer — no string `id` or no
/// Trust Task `type` to name in `inResponseTo` — or a transport that
/// authenticated no sender comes back as an [`UnanswerableBody`] error.
pub struct GuardedConsumer {
    record: Record,
    freshness: FreshnessPolicy,
}

impl std::fmt::Debug for GuardedConsumer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let record = match &self.record {
            Record::InProcess(guard) => format!("in-process ({} records)", guard.len()),
            Record::Shared(_) => "caller-supplied".to_string(),
            Record::Disabled => "disabled".to_string(),
        };
        f.debug_struct("GuardedConsumer")
            .field("replay_record", &record)
            .field("freshness", &self.freshness)
            .finish()
    }
}

impl Default for GuardedConsumer {
    fn default() -> Self {
        Self::new()
    }
}

impl GuardedConsumer {
    /// A consumer with the duplicate-execution record **on** (in-process) and
    /// [`FreshnessPolicy::consequential`] applied.
    ///
    /// These are the defaults because they are the ones a deployment cannot
    /// discover it is missing: nothing about a consumer without a record looks
    /// wrong until a mediator or an intermediary re-delivers an `acl/grant`
    /// and it is applied twice.
    pub fn new() -> Self {
        Self {
            record: Record::InProcess(InMemoryReplayGuard::default()),
            freshness: FreshnessPolicy::consequential(),
        }
    }

    /// A consumer whose record is `guard` — the shape a replicated deployment
    /// needs.
    ///
    /// [`InMemoryReplayGuard`] is correct only for a single process: two
    /// replicas each hold their own map, so a document accepted by one is
    /// `Fresh` at the other and the consequential effect happens twice. A
    /// replicated deployment **MUST** pass a guard backed by a store all its
    /// replicas share.
    pub fn with_replay_guard(guard: Arc<dyn ReplayGuard>) -> Self {
        Self {
            record: Record::Shared(guard),
            freshness: FreshnessPolicy::consequential(),
        }
    }

    /// **SECURITY: keep no duplicate-execution record.**
    ///
    /// This re-opens exactly what SPEC §7.2 item 11 closes. With no record the
    /// same document delivered twice executes twice — and no message transport
    /// stops that happening by accident: a mediator or an intermediary that
    /// re-delivers a queued message needs no attacker, and a replayed message
    /// is indistinguishable from that.
    ///
    /// Item 11 permits this only where the task is not *consequential* (SPEC
    /// §2), or where the *Trust Task specification* "explicitly declares
    /// repeated execution safe and intended". Both are properties of the
    /// operation, not of the consumer's convenience.
    ///
    /// The freshness bound drops to [`FreshnessPolicy::default`] with it:
    /// [`FreshnessPolicy::consequential`]'s acceptance window exists to bound
    /// a record that is no longer being kept. The two internal-consistency
    /// checks — a future-dated document, an empty validity interval — still
    /// apply; no conforming producer emits either.
    #[must_use]
    pub fn without_replay_record() -> Self {
        Self {
            record: Record::Disabled,
            freshness: FreshnessPolicy::default(),
        }
    }

    /// Builder: replace the freshness policy.
    ///
    /// SPEC §7.2 (*Bounding the record*) makes the acceptance window and the
    /// record's retention **one bound** — [`FreshnessPolicy::record_expiry`]
    /// is what this consumer hands the guard as `retain_until`. Widening the
    /// window therefore widens retention with it; there is no second TTL to
    /// configure, and a deployment whose transport may hold a message for
    /// longer than five minutes should widen this rather than reach for one.
    #[must_use]
    pub fn with_freshness(mut self, freshness: FreshnessPolicy) -> Self {
        self.freshness = freshness;
        self
    }

    /// The freshness policy in force.
    pub fn freshness(&self) -> &FreshnessPolicy {
        &self.freshness
    }

    /// The guard backing the record, or `None` where the record is disabled.
    pub fn replay_guard(&self) -> Option<&dyn ReplayGuard> {
        match &self.record {
            Record::InProcess(guard) => Some(guard),
            Record::Shared(guard) => Some(guard.as_ref()),
            Record::Disabled => None,
        }
    }

    /// The [`ConsumeChecks`] this consumer applies — the freshness policy and
    /// the replay record as one argument, because SPEC §7.2 ties them
    /// together.
    ///
    /// Exposed for a caller that drives [`consume_inbound`] itself and wants
    /// the same two checks without re-deriving them.
    pub fn checks(&self) -> ConsumeChecks<'_> {
        ConsumeChecks {
            freshness: self.freshness,
            replay: match self.replay_guard() {
                Some(guard) => ReplayPolicy::Guard(guard),
                None => ReplayPolicy::NotConsequential,
            },
            ..ConsumeChecks::not_consequential()
        }
    }

    /// The guarded pipeline over a body a binding has opened: parse it, apply
    /// `envelope_check`, then run [`consume_inbound`] with this consumer's
    /// checks and the proof policy bound to `body` as received.
    ///
    /// `transport` must carry the identities the envelope **authenticated**.
    /// `envelope_check` is the binding's own cross-check between envelope and
    /// document (DIDComm's thread headers, say); `|_| Ok(())` where there is
    /// none.
    ///
    /// The document is read as `P` through
    /// [`extension::read_as`](crate::extension::read_as): `P`'s own version,
    /// or a later one whose declared extension chain reaches it (SPEC §5.2),
    /// pruned. A body that does not parse, that `envelope_check` refuses, or
    /// whose `type` `P` cannot read, is answered — `malformedRequest`,
    /// `unsupportedType` or `unsupportedVersion` — as
    /// [`ConsumeOutcome::Rejected`] addressed to the transport-authenticated
    /// sender, before the record is written. See [`UnanswerableBody`] for the
    /// two cases that cannot be answered.
    ///
    /// Beyond that the outcomes are [`consume_inbound`]'s, unchanged —
    /// including [`ConsumeOutcome::Duplicate`], which is **not** an error and
    /// must never be answered with `taskFailed`.
    #[allow(clippy::too_many_arguments)]
    pub async fn consume_body<P, R, T, V, W, F, Fut>(
        &self,
        transport: &T,
        body: Value,
        envelope_check: impl FnOnce(&TrustTask<Value>) -> Result<(), RejectReason>,
        my_vid: &str,
        proof_policy: ProofPolicy<'_, V>,
        payload_policy: PayloadPolicy<'_, W>,
        now: DateTime<Utc>,
        error_id_factory: impl FnOnce() -> String,
        handler: F,
    ) -> Result<ConsumeOutcome<R>, UnanswerableBody>
    where
        P: Payload + Serialize + Send + Sync,
        R: Serialize,
        T: TransportHandler + Sync + ?Sized,
        V: ProofVerifier + ?Sized,
        W: PayloadValidator + ?Sized,
        F: FnOnce(TrustTask<P>, ResolvedParties) -> Fut,
        Fut: Future<Output = Result<TrustTask<R>, ErrorResponse>>,
    {
        let doc = match open_body(transport, &body, envelope_check, my_vid).and_then(|doc| {
            crate::extension::read_as::<P>(doc)
                .map_err(|reason| refuse(transport, &body, my_vid, reason))
        }) {
            Ok(doc) => doc,
            Err(refusal) => return refusal.into_outcome(error_id_factory),
        };
        Ok(self
            .consume(
                transport,
                doc,
                my_vid,
                proof_policy.with_received(&body),
                payload_policy,
                now,
                error_id_factory,
                handler,
            )
            .await)
    }

    /// [`consume_body`](Self::consume_body) for a consumer that takes a
    /// document of **any** type and routes it on its `type` with
    /// [`consume_dispatched`] — answering a type it has no handler for with
    /// `unsupportedType`, and a known one at an unhandled version with
    /// `unsupportedVersion`.
    ///
    /// The body's parse and `envelope_check` are answered as in
    /// [`consume_body`](Self::consume_body); the rest is
    /// [`consume_dispatched`]'s, with this consumer's checks and the proof
    /// verified over the body as received. `context` builds the dispatcher's
    /// per-document `Ctx` from the resolved parties.
    #[allow(clippy::too_many_arguments)]
    pub async fn consume_body_dispatched<Ctx, R, T, V, W>(
        &self,
        transport: &T,
        body: Value,
        envelope_check: impl FnOnce(&TrustTask<Value>) -> Result<(), RejectReason>,
        my_vid: &str,
        proof_policy: ProofPolicy<'_, V>,
        payload_policy: PayloadPolicy<'_, W>,
        dispatcher: &AsyncDispatcher<Ctx, Result<TrustTask<R>, ErrorResponse>>,
        now: DateTime<Utc>,
        error_id_factory: impl FnOnce() -> String,
        context: impl FnOnce(ResolvedParties) -> Ctx,
    ) -> Result<ConsumeOutcome<R>, UnanswerableBody>
    where
        R: Serialize,
        T: TransportHandler + Sync + ?Sized,
        V: ProofVerifier + ?Sized,
        W: PayloadValidator + ?Sized,
    {
        let doc = match open_body(transport, &body, envelope_check, my_vid) {
            Ok(doc) => doc,
            Err(refusal) => return refusal.into_outcome(error_id_factory),
        };
        Ok(self
            .consume_dispatched(
                transport,
                doc,
                my_vid,
                proof_policy.with_received(&body),
                payload_policy,
                dispatcher,
                now,
                error_id_factory,
                context,
            )
            .await)
    }

    /// The guarded pipeline over a document the caller parsed itself —
    /// [`consume_inbound`] with this consumer's [`checks`](Self::checks).
    ///
    /// Use it where the envelope was opened by a stack the binding does not
    /// drive — a mediator SDK's own delivery loop, most often — so the record
    /// covers that path too. Pass [`ProofPolicy::VerifyReceived`] with the
    /// document as it arrived where you still have it.
    ///
    /// The verdicts of SPEC §7.2's *Disposition of a duplicate* are applied by
    /// [`consume_inbound`]: `Fresh` dispatches and then records the response;
    /// a duplicate returns [`ConsumeOutcome::Duplicate`] carrying the prior
    /// response — or `in_flight` where the first execution has not finished —
    /// **without** dispatching; a differing document under a reused `id` is
    /// rejected `idConflict`; and a guard that cannot answer fails closed as
    /// `unavailable` with `retryable = true`, never by executing.
    #[allow(clippy::too_many_arguments)]
    pub async fn consume<P, R, T, V, W, F, Fut>(
        &self,
        transport: &T,
        doc: TrustTask<P>,
        my_vid: &str,
        proof_policy: ProofPolicy<'_, V>,
        payload_policy: PayloadPolicy<'_, W>,
        now: DateTime<Utc>,
        error_id_factory: impl FnOnce() -> String,
        handler: F,
    ) -> ConsumeOutcome<R>
    where
        P: Payload + Serialize + Send + Sync,
        R: Serialize,
        T: TransportHandler + Sync + ?Sized,
        V: ProofVerifier + ?Sized,
        W: PayloadValidator + ?Sized,
        F: FnOnce(TrustTask<P>, ResolvedParties) -> Fut,
        Fut: Future<Output = Result<TrustTask<R>, ErrorResponse>>,
    {
        consume_inbound(
            transport,
            proof_policy,
            payload_policy,
            self.checks(),
            doc,
            my_vid,
            now,
            error_id_factory,
            handler,
        )
        .await
    }

    /// The guarded, dispatched pipeline over a document the caller parsed
    /// itself — [`consume_dispatched`] with this consumer's
    /// [`checks`](Self::checks).
    ///
    /// Pass [`ProofPolicy::VerifyReceived`] with the document as it arrived:
    /// a forward-routed payload is pruned before the handler sees it, and a
    /// proof covers what was signed.
    #[allow(clippy::too_many_arguments)]
    pub async fn consume_dispatched<Ctx, R, T, V, W>(
        &self,
        transport: &T,
        doc: TrustTask<Value>,
        my_vid: &str,
        proof_policy: ProofPolicy<'_, V>,
        payload_policy: PayloadPolicy<'_, W>,
        dispatcher: &AsyncDispatcher<Ctx, Result<TrustTask<R>, ErrorResponse>>,
        now: DateTime<Utc>,
        error_id_factory: impl FnOnce() -> String,
        context: impl FnOnce(ResolvedParties) -> Ctx,
    ) -> ConsumeOutcome<R>
    where
        R: Serialize,
        T: TransportHandler + Sync + ?Sized,
        V: ProofVerifier + ?Sized,
        W: PayloadValidator + ?Sized,
    {
        consume_dispatched(
            transport,
            proof_policy,
            payload_policy,
            self.checks(),
            dispatcher,
            doc,
            my_vid,
            now,
            error_id_factory,
            context,
        )
        .await
    }
}

/// A body refused before the pipeline.
enum Refusal {
    /// Answerable: `reason`, about the document `view` names, to `sender`.
    Answer {
        view: Box<TrustTask<()>>,
        sender: String,
        reason: RejectReason,
    },
    /// Not answerable.
    Unanswerable(UnanswerableBody),
}

impl Refusal {
    fn into_outcome<R>(
        self,
        error_id_factory: impl FnOnce() -> String,
    ) -> Result<ConsumeOutcome<R>, UnanswerableBody> {
        match self {
            Refusal::Answer {
                view,
                sender,
                reason,
            } => Ok(ConsumeOutcome::Rejected(view.reject_with_recipient(
                error_id_factory(),
                reason,
                Some(sender),
            ))),
            Refusal::Unanswerable(unanswerable) => Err(unanswerable),
        }
    }
}

/// Parse `body` as an untyped document and apply the binding's envelope
/// check, turning either failure into a [`Refusal`].
fn open_body<T>(
    transport: &T,
    body: &Value,
    envelope_check: impl FnOnce(&TrustTask<Value>) -> Result<(), RejectReason>,
    my_vid: &str,
) -> Result<TrustTask<Value>, Refusal>
where
    T: TransportHandler + ?Sized,
{
    let doc = match serde_json::from_value::<TrustTask<Value>>(body.clone()) {
        Ok(doc) => doc,
        Err(err) => {
            let reason = RejectReason::malformed_from_serde(&err);
            return Err(match refuse(transport, body, my_vid, reason) {
                // Nothing to answer: report the parse failure it is.
                Refusal::Unanswerable(UnanswerableBody::Unparsable(_)) => {
                    Refusal::Unanswerable(UnanswerableBody::Unparsable(err))
                }
                other => other,
            });
        }
    };
    envelope_check(&doc).map_err(|reason| refuse(transport, body, my_vid, reason))?;
    Ok(doc)
}

/// The refusal of `body` with `reason`, addressed to the transport-
/// authenticated sender.
///
/// The error response names the document by the `id`, `type` and thread
/// members of `body` as received — read one by one, so a body that failed to
/// parse as a whole is still answered — and is addressed **from** the
/// transport-authenticated recipient (or `my_vid`) **to** the
/// transport-authenticated sender. The in-band `issuer` and `recipient` are
/// deliberately not used: nothing has cross-checked them against the envelope
/// yet (SPEC §4.8.1 runs inside the pipeline), and the envelope the error
/// travels in is addressed to the authenticated sender regardless.
fn refuse<T>(transport: &T, body: &Value, my_vid: &str, reason: RejectReason) -> Refusal
where
    T: TransportHandler + ?Sized,
{
    let str_member = |key: &str| body.get(key).and_then(Value::as_str).map(str::to_string);
    let (Some(id), Some(type_uri)) = (
        str_member("id"),
        str_member("type").and_then(|t| t.parse::<TypeUri>().ok()),
    ) else {
        return Refusal::Unanswerable(UnanswerableBody::Unparsable(serde::de::Error::custom(
            "body carries no string `id` and Trust Task `type` to answer",
        )));
    };
    let parties = transport.derive_parties();
    let Some(sender) = parties.issuer else {
        return Refusal::Unanswerable(UnanswerableBody::NoAuthenticatedSender(reason));
    };
    let mut view = TrustTask::new(id, type_uri, ());
    view.thread_id = str_member("threadId");
    view.parent_thread_id = str_member("parentThreadId");
    view.ceremony = body
        .get("ceremony")
        .and_then(|c| serde_json::from_value(c.clone()).ok());
    view.recipient = Some(parties.recipient.unwrap_or_else(|| my_vid.to_string()));
    Refusal::Answer {
        view: Box::new(view),
        sender,
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::TransportContext;
    use crate::{StandardCode, TrustTaskCode};

    struct Peer(Option<&'static str>);

    impl TransportHandler for Peer {
        fn binding_uri(&self) -> &str {
            "test:peer"
        }
        fn derive_parties(&self) -> TransportContext {
            TransportContext {
                issuer: self.0.map(str::to_string),
                recipient: Some("did:example:me".into()),
            }
        }
    }

    fn malformed(outcome: ConsumeOutcome<Value>) -> ErrorResponse {
        match outcome {
            ConsumeOutcome::Rejected(error) => {
                assert_eq!(
                    error.payload.code,
                    TrustTaskCode::Standard(StandardCode::MalformedRequest)
                );
                error
            }
            other => panic!("expected Rejected, got {other:?}"),
        }
    }

    fn open(
        transport: &Peer,
        body: Value,
        check: impl FnOnce(&TrustTask<Value>) -> Result<(), RejectReason>,
    ) -> Result<ConsumeOutcome<Value>, UnanswerableBody> {
        match open_body(transport, &body, check, "did:example:me") {
            Ok(_) => panic!("expected a refusal"),
            Err(refusal) => refusal.into_outcome(|| "urn:uuid:err".into()),
        }
    }

    fn unparsable_member() -> Value {
        serde_json::json!({
            "id": "urn:uuid:1",
            "type": "https://trusttasks.org/spec/acl/grant/0.1",
            "issuer": "did:example:someone-else",
            "recipient": "did:example:also-someone-else",
            "threadId": "thread-1",
            "issuedAt": 42,
            "payload": {},
        })
    }

    #[test]
    fn an_unparsable_body_is_answered_to_the_authenticated_sender() {
        let outcome = open(
            &Peer(Some("did:example:sender")),
            unparsable_member(),
            |_| Ok(()),
        );
        let error = malformed(outcome.unwrap());
        // The authenticated parties, never the unchecked in-band ones.
        assert_eq!(error.recipient.as_deref(), Some("did:example:sender"));
        assert_eq!(error.issuer.as_deref(), Some("did:example:me"));
        assert_eq!(error.thread_id.as_deref(), Some("thread-1"));
        let in_response_to = error.payload.in_response_to.unwrap();
        assert_eq!(in_response_to.id.as_deref(), Some("urn:uuid:1"));
        // §10.4: no deserializer detail on the wire.
        assert!(!error.payload.message.unwrap_or_default().contains("line"));
    }

    #[test]
    fn a_body_with_nothing_to_answer_is_an_error() {
        let body = serde_json::json!({ "id": 7, "payload": {} });
        assert!(matches!(
            open(&Peer(Some("did:example:sender")), body, |_| Ok(())),
            Err(UnanswerableBody::Unparsable(_))
        ));
    }

    #[test]
    fn no_authenticated_sender_means_no_answer() {
        assert!(matches!(
            open(&Peer(None), unparsable_member(), |_| Ok(())),
            Err(UnanswerableBody::NoAuthenticatedSender(_))
        ));
    }

    #[test]
    fn an_envelope_refusal_is_answered_to_the_authenticated_sender() {
        let body = serde_json::json!({
            "id": "urn:uuid:1",
            "type": "https://trusttasks.org/spec/acl/grant/0.1",
            "issuer": "did:example:someone-else",
            "payload": {},
        });
        let outcome = open(&Peer(Some("did:example:sender")), body, |_| {
            Err(RejectReason::MalformedRequest {
                reason: "thread".into(),
            })
        });
        let error = malformed(outcome.unwrap());
        assert_eq!(error.recipient.as_deref(), Some("did:example:sender"));
    }
}
