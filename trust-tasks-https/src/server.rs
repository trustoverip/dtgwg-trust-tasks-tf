//! Axum-based server: receives Trust Task documents over HTTP, dispatches
//! them to per-spec handlers, and returns a typed response document or a
//! `trust-task-error` document with the appropriate HTTP status code.
//!
//! Single endpoint: `POST /trust-tasks` accepts any document the server has
//! a handler registered for. Bearer auth (`Authorization: Bearer <token>`)
//! is consulted before framework validation; the resulting VID becomes the
//! transport-authenticated sender for §4.8.1 precedence.
//!
//! ## One pipeline
//!
//! The server owns only what is HTTP: the body limit, the content type, the
//! bearer token and the mapping of an outcome onto a status line. Every
//! framework check — routing, the per-spec policy, freshness, identity, the
//! attribution gate, the proof, the §7.2 item 11 record — is
//! [`trust_tasks_rs::consume_dispatched_answer`], the pipeline every other
//! Rust binding runs, configured through [`ConsumeChecks`]. The same document
//! gets the same answer whichever Rust entry point receives it.
//!
//! ## Request pipeline order
//!
//! The order below is load-bearing, not incidental. Every step that can
//! cost the server work an unauthenticated sender chose — above all DID
//! resolution inside a configured [`ProofVerifier`], which for `did:web`
//! is an outbound HTTPS request to a host named in the request body —
//! runs **after** the cheap, local rejections:
//!
//! 1. The body is buffered under the limit [`HttpsServer::into_router`]
//!    applies (413 past it).
//! 2. `Content-Type` must be `application/json` (binding spec §2), else 415.
//! 3. Body parses as a Trust Task document.
//! 4. Bearer token → transport-authenticated peer ([`AsyncAuth`]).
//! 5. `validate_basic` (expiry, recipient), the **freshness bound** over
//!    `issuedAt` / `expiresAt` (§7.2 item 13) — see
//!    [`HttpsServerBuilder::freshness`] — and `resolve_parties` (§4.8.1). With
//!    no authenticated peer, `resolve_parties` runs first, so a misaddressed
//!    document is suppressed (§8.1).
//! 6. **Attribution gate** — see [`HttpsServerBuilder::require_attribution`].
//! 7. **Route lookup** (§7.2 item 3) — an unregistered slug is
//!    `unsupportedType`, a registered slug at a version that neither has a
//!    handler nor reaches one along its declared `extends` chain is
//!    `unsupportedVersion`, and a version that does reach one is routed to it
//!    with its payload pruned (SPEC §5.2). Steps 5 and 6 depend on nothing a
//!    route holds and run first, so a sender the attribution gate refuses
//!    gets the same answer whether or not its `type` is registered: the route
//!    table is not readable without attribution, just as discovery keeps it
//!    unreadable without authentication. The lookup still precedes everything
//!    that touches the network.
//! 8. The **per-type size bound**: a document larger than the `maxDocumentBytes`
//!    its specification declares — [`MAX_BODY_BYTES`] for one that declares
//!    none — is `malformedRequest`.
//! 9. The payload deserializes into the registered type, and, with
//!    [`HttpsServerBuilder::with_payload_validator`], passes its schema
//!    (§7.2 item 2).
//! 10. `proof.verificationMethod` DID-method pre-screen — see
//!     [`HttpsServerBuilder::allowed_did_methods`]. With a verifier
//!     configured and no explicit choice, only `did:key` passes.
//! 11. Proof verification, over the body as received (the only step that may
//!     egress).
//! 12. Per-spec policy (§7.2 items 5b, 7A, 8; §7.3 item 17).
//! 13. **Duplicate-execution claim** (§7.2 item 11) — see
//!     [`HttpsServerBuilder::replay_protection`].
//! 14. The registered handler.
//!
//! ## Where the replay claim sits, and why
//!
//! Step 13 is the `validated → accepted` transition: the last instant at
//! which the server can still refuse a document for a reason that has nothing
//! to do with having seen it before, and the first at which it has committed
//! to execute. Claiming any earlier would write a record for documents the
//! server then refuses — burning the `id` of every malformed, stale,
//! unattributable or badly-signed arrival, so that a corrected resend came
//! back `idConflict` forever and a stranger could pre-burn an `id` it had
//! merely observed. Claiming any later is not a claim at all: the effect
//! would already have happened.
//!
//! What the handler answers settles the claim as `consume_inbound` settles
//! it: a success is recorded and a duplicate answered with it; a
//! **retryable** refusal releases the claim, so the bit-for-bit resend SPEC
//! §8.4 invites is evaluated afresh; a **non-retryable** refusal is the
//! determination for that document, recorded and returned to a resend with
//! its own status. A handler that panics releases the claim. A request
//! abandoned **while its handler runs** (the request timeout, a client
//! disconnect) leaves the claim in flight, because the handler may already
//! have committed its effect: a resend is answered `202 Accepted` until the
//! record expires with the acceptance window, and is never executed again.
//!
//! ## Handlers and blocking
//!
//! [`HttpsServerBuilder::on_async`] / [`HttpsServerBuilder::on_ack_async`]
//! register handlers that `await` — a database, an ACL store, a signing
//! service. [`HttpsServerBuilder::on`] / [`HttpsServerBuilder::on_ack`] take
//! a synchronous closure, which may block: on a multi-threaded Tokio runtime
//! it runs under [`tokio::task::block_in_place`], so the worker's other tasks
//! move elsewhere while it does; on a current-thread runtime it runs inline.
//!
//! Not `spawn_blocking`, deliberately. A request abandoned by the
//! [`TimeoutLayer`] (or a client disconnect) would leave a closure on the
//! blocking pool executing with nobody left to record its result, so every
//! resend would be answered as in flight until the record expired, and the
//! timeout would bound nothing anyway. A synchronous handler therefore runs to
//! completion inside its request, its result is recorded, and
//! [`HttpsServerBuilder::request_timeout`] does **not** bound it: the timeout
//! fires only at an `await`. Write a handler that can take long as an async
//! one.

use std::collections::BTreeSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use axum::body::Bytes;
use axum::error_handling::HandleErrorLayer;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use serde::Serialize;
use serde_json::Value;
use tower::limit::ConcurrencyLimitLayer;
use tower::timeout::TimeoutLayer;
use tower::ServiceBuilder;
use trust_tasks_rs::{
    consume_dispatched_answer, discovery::DiscoveryRegistry, erase_verifier,
    specs::trust_task_discovery::v0_4 as discovery, AnswerOutcome, AsyncDispatcher, ConsumeChecks,
    DynProofVerifier, ErrorPayload, ErrorResponse, FreshnessPolicy, InMemoryReplayGuard, Payload,
    PayloadPolicy, PayloadValidator, ProofPolicy, ProofVerifier, RejectReason, ReplayGuard,
    ReplayPolicy, RequestPayload, ResolvedParties, TransportHandler, TrustTask,
};
use uuid::Uuid;

use crate::auth::{AsyncAuth, BearerAuth};

// --- trust-task-discovery 0.1 / 0.2 / 0.3 (superseded) ----------------------
// 0.1 and 0.2 are superseded by 0.3 and served until they are withdrawn; the
// withdrawal batch deletes `server/superseded_discovery.rs` and every block
// marked like this one.
mod superseded_discovery;
// --- end trust-task-discovery 0.1 / 0.2 / 0.3 (superseded) ------------------
use crate::handler::HttpsHandler;
use crate::status::status_for_code;

/// The transport bound on a Trust Task document (SPEC §12.2, §12.4): the
/// smallest request-body limit [`HttpsServer::into_router`] applies, and the
/// size bound held against a document whose specification declares no
/// `maxDocumentBytes`. Trust Task payloads are small; 256 KiB is generous
/// headroom while bounding pre-auth memory use.
///
/// The router's limit is the larger of this and the largest
/// `maxDocumentBytes` among the registered request types
/// ([`Payload::MAX_DOCUMENT_BYTES`]) — so a server that registers a task
/// whose specification promises to accept a larger document can, and one
/// that does not never buffers more than this — unless
/// [`HttpsServerBuilder::max_body_bytes`] sets it outright.
pub const MAX_BODY_BYTES: usize = 256 * 1024;

/// Default per-request wall-clock budget applied by [`HttpsServer::into_router`].
/// A Trust Task exchange is one small JSON round trip; a request still in
/// flight after this is wedged, and holding the connection open is exactly
/// what a slowloris wants. Override with
/// [`HttpsServerBuilder::request_timeout`].
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Default ceiling on concurrently-executing requests, applied by
/// [`HttpsServer::into_router`]. Bounds the memory a flood of slow senders
/// can pin. Override with [`HttpsServerBuilder::max_concurrent_requests`].
pub const DEFAULT_MAX_CONCURRENT_REQUESTS: usize = 512;

/// Context handed to every spec handler — the transport-authenticated
/// peer (when present), convenience accessors for the inbound
/// document's metadata, and the SPEC §4.8.1-resolved party identities.
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// VID of the transport-authenticated sender, if any. Equivalent to
    /// `resolved.issuer` when the in-band `issuer` is absent and the
    /// transport authenticated a peer; preserved separately so handlers
    /// can audit-log the distinction between in-band and derived.
    pub authenticated_sender: Option<String>,
    /// VID of the local party serving this request.
    pub local: Option<String>,
    /// SPEC §4.8.1-resolved party identities (in-band wins over
    /// transport-derived; transport fills in absent in-band values).
    /// Handlers can use this directly instead of re-running
    /// [`TransportHandler::resolve_parties`].
    pub resolved: ResolvedParties,
}

/// What a registered handler answers once the pipeline has accepted the
/// document: the JSON body of a `#response` document, `None` where the
/// specification defines no success response and the handler acknowledged
/// (SPEC §4.4.1 — 204, and recorded as *no response*, so a duplicate is
/// absorbed in silence), or the §8.1-routed error document of a refusal.
type Answer = Result<Option<Value>, ErrorResponse>;

/// The routing table: every handler, behind the dispatcher the pipeline
/// routes, prunes and polices with.
type Routes = AsyncDispatcher<RequestContext, Answer>;

/// A handler's own outcome, before the server wraps it: the response body (or
/// `None` for an acknowledgement), or a refusal.
type HandlerFuture = Pin<Box<dyn Future<Output = Result<Option<Value>, RejectReason>> + Send>>;

/// Internal state shared by every axum request handler.
struct ServerState {
    local_vid: Option<String>,
    auth: Box<dyn AsyncAuth>,
    routes: Routes,
    verifier: Option<Arc<dyn DynProofVerifier>>,
    payload_validator: Option<Box<dyn PayloadValidator + Send + Sync>>,
    require_attribution: bool,
    allowed_did_methods: Option<BTreeSet<String>>,
    /// `None` only when the deployment called
    /// `replay_protection(false)` — see that method's warning.
    replay_guard: Option<Arc<dyn ReplayGuard>>,
    freshness: FreshnessPolicy,
    /// The bound for a document whose specification declares none.
    default_document_bound: usize,
}

/// Which DID methods `proof.verificationMethod` may name, as configured.
enum DidMethodPolicy {
    /// No explicit choice: `did:key` only when a verifier is configured
    /// (see [`DEFAULT_ALLOWED_DID_METHODS`]); nothing to screen without one.
    Default,
    /// [`HttpsServerBuilder::allowed_did_methods`].
    Only(BTreeSet<String>),
    /// [`HttpsServerBuilder::allow_any_did_method`].
    Any,
}

/// The DID methods a server with a verifier accepts in
/// `proof.verificationMethod` when neither
/// [`HttpsServerBuilder::allowed_did_methods`] nor
/// [`HttpsServerBuilder::allow_any_did_method`] was called: `did:key`, whose
/// key is in the identifier, so verifying it never leaves the process.
pub const DEFAULT_ALLOWED_DID_METHODS: &[&str] = &["key"];

/// Builder for [`HttpsServer`].
pub struct HttpsServerBuilder {
    local_vid: Option<String>,
    auth: Option<Box<dyn AsyncAuth>>,
    routes: Routes,
    /// The largest `maxDocumentBytes` among the registered request types.
    largest_declared_bound: usize,
    max_body_bytes: Option<usize>,
    verifier: Option<Arc<dyn DynProofVerifier>>,
    payload_validator: Option<Box<dyn PayloadValidator + Send + Sync>>,
    require_attribution: bool,
    did_methods: DidMethodPolicy,
    replay_protection: bool,
    replay_guard: Option<Arc<dyn ReplayGuard>>,
    freshness: FreshnessPolicy,
    /// Shared with the discovery handler closure so `.public_discovery()`
    /// can be called either side of `.enable_discovery()`.
    public_discovery: Arc<AtomicBool>,
    /// The freshness policy the built server applies, filled in by
    /// [`Self::build`] and read by the discovery handler
    /// [`Self::enable_discovery`] installs, so the acceptance window it
    /// advertises is the one enforced however the builder calls are ordered.
    applied_freshness: Arc<OnceLock<FreshnessPolicy>>,
    request_timeout: Option<Duration>,
    max_concurrent_requests: Option<usize>,
}

impl HttpsServerBuilder {
    /// Set the local party's VID. Becomes `local` on the per-request
    /// [`HttpsHandler`] and is what `recipient`-bearing documents are
    /// cross-checked against under SPEC.md §7.2 item 5.
    pub fn local_vid(mut self, vid: impl Into<String>) -> Self {
        self.local_vid = Some(vid.into());
        self
    }

    /// Plug in an authenticator. Defaults to a [`BearerAuth`] with no tokens
    /// — every request is treated as unauthenticated and the framework falls
    /// back entirely to in-band identity.
    ///
    /// Takes any [`AsyncAuth`], which every [`Auth`](crate::Auth) is: a
    /// synchronous lookup (a token map, a JWT signature check) is called
    /// directly, and one that must `await` — a session store, an
    /// introspection endpoint — implements [`AsyncAuth`] itself.
    pub fn with_auth(mut self, auth: impl AsyncAuth) -> Self {
        self.auth = Some(Box::new(auth));
        self
    }

    /// Plug in an in-band [`ProofVerifier`]. When configured, the
    /// server verifies the `proof` member of every proof-bearing
    /// document — over the request body as received — and rejects with
    /// `proofInvalid` on failure; when absent, the server rejects
    /// proof-bearing documents with `malformedRequest` (the
    /// [`PROOF_NOT_ACCEPTED_BY_POLICY`](trust_tasks_rs::PROOF_NOT_ACCEPTED_BY_POLICY)
    /// rule — [`ProofPolicy::RejectIfPresentReceived`]).
    ///
    /// The verifier is the **last** check before the per-spec policy and
    /// the duplicate-execution claim: it is the only one that may egress,
    /// so route lookup, transport auth and the attribution gate all run
    /// ahead of it. See [`Self::allowed_did_methods`] for the pre-screen
    /// that bounds where that egress can go.
    ///
    /// Accepts any concrete [`ProofVerifier`] (e.g. from
    /// `trust-tasks-proof`'s `affinidi` backend); the server stores it
    /// behind [`trust_tasks_rs::DynProofVerifier`] for object-safe
    /// dispatch.
    pub fn with_verifier<V>(mut self, verifier: V) -> Self
    where
        V: ProofVerifier + Send + Sync + 'static,
    {
        self.verifier = Some(erase_verifier(verifier));
        self
    }

    /// Check every payload against its specification's schema (SPEC §7.2
    /// item 2) with `validator` — the residue deserialization cannot express
    /// (`minProperties`, `minItems` on an optional array, conditional
    /// subschemas); see [`PayloadPolicy`]. Without it the payload is accepted
    /// on the strength of deserializing into the registered type.
    ///
    /// The `validate` feature of `trust-tasks-rs` provides
    /// `validate::SchemaValidator`.
    pub fn with_payload_validator<W>(mut self, validator: W) -> Self
    where
        W: PayloadValidator + Send + Sync + 'static,
    {
        self.payload_validator = Some(Box::new(validator));
        self
    }

    /// Set the request-body limit [`HttpsServer::into_router`] applies,
    /// outright, replacing the one derived from the registered types (see
    /// [`MAX_BODY_BYTES`]). It is also then the bound held against a document
    /// whose specification declares no `maxDocumentBytes`.
    ///
    /// The body is buffered before authentication, so this is the memory an
    /// unauthenticated sender can make each request cost. A limit below a
    /// registered type's declared bound refuses documents that specification
    /// says a consumer **MUST NOT** refuse on size alone.
    pub fn max_body_bytes(mut self, limit: usize) -> Self {
        self.max_body_bytes = Some(limit);
        self
    }

    /// Register a handler for the request payload type `P`. The handler
    /// receives the typed request and a [`RequestContext`]; it returns
    /// either the response payload (which the server wraps in a
    /// `#response`-variant document via [`TrustTask::respond_with`]) or a
    /// [`RejectReason`] (which the server wraps in a `trust-task-error`
    /// document via [`TransportHandler::reject`], applying SPEC §8.1
    /// routing).
    ///
    /// The handler is synchronous and may block — see the module docs,
    /// *Handlers and blocking*: it runs to completion inside its request,
    /// under `block_in_place` on a multi-threaded runtime, and the request
    /// timeout does not bound it. A handler that does I/O is better written
    /// with [`Self::on_async`].
    ///
    /// The handler also receives documents of a later version of `P`'s
    /// specification that declares an `extends` chain reaching `P`'s, when
    /// that version has no handler of its own (SPEC §5.2), with the payload
    /// pruned to what `P` knows and `type` as received — so the response is
    /// the received version's `#response`.
    pub fn on<P, F>(self, handler: F) -> Self
    where
        P: RequestPayload + Send + 'static,
        P::Response: Serialize + 'static,
        F: Fn(&TrustTask<P>, &RequestContext) -> Result<P::Response, RejectReason>
            + Send
            + Sync
            + 'static,
    {
        self.on_sync(handler, true)
    }

    /// [`Self::on`] for a handler that runs no longer than its arguments take
    /// to read — the discovery handlers — called inline, without
    /// `block_in_place`.
    pub(crate) fn on_sync<P, F>(self, handler: F, may_block: bool) -> Self
    where
        P: RequestPayload + Send + 'static,
        P::Response: Serialize + 'static,
        F: Fn(&TrustTask<P>, &RequestContext) -> Result<P::Response, RejectReason>
            + Send
            + Sync
            + 'static,
    {
        let handler = Arc::new(handler);
        self.register::<P>(move |req, ctx| {
            let handler = Arc::clone(&handler);
            Box::pin(async move {
                let response = run_sync(may_block, || handler(&req, &ctx))?;
                respond(&req, response).map(Some)
            })
        })
    }

    /// Register an **async** handler for the request payload type `P` —
    /// [`Self::on`] for a handler that `await`s.
    ///
    /// The handler takes the typed request and the [`RequestContext`] by
    /// value and returns a future of the response payload or a
    /// [`RejectReason`]. The future runs on the request's task, so
    /// [`Self::request_timeout`] bounds it.
    ///
    /// A request abandoned mid-handler (the timeout, a client disconnect)
    /// drops the future at whichever `await` it had reached. The effect may
    /// already have committed, so the duplicate-execution claim is **left in
    /// flight** rather than released: a bit-for-bit resend is answered
    /// `202 Accepted` and does not run the handler again, until the record
    /// expires with the acceptance window. The producer learns nothing more
    /// about that execution, and a new attempt is a new `id` (SPEC §8.4). An
    /// effect half-done at the cancellation point stays half-done: make it
    /// atomic, or idempotent, across each `await`.
    ///
    /// ```rust,ignore
    /// HttpsServer::builder()
    ///     .on_async::<grant::Payload, _, _>(move |req, ctx| {
    ///         let store = store.clone();
    ///         async move {
    ///             store.grant(&req.payload.entry, ctx.resolved.issuer.as_deref()).await?;
    ///             Ok(grant::Response { entry: req.payload.entry, ext: None })
    ///         }
    ///     })
    /// ```
    pub fn on_async<P, F, Fut>(self, handler: F) -> Self
    where
        P: RequestPayload + Send + 'static,
        P::Response: Serialize + 'static,
        F: Fn(TrustTask<P>, RequestContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<P::Response, RejectReason>> + Send + 'static,
    {
        self.register::<P>(move |req, ctx| {
            let head = head_of(&req);
            let answer = handler(req, ctx);
            Box::pin(async move {
                let response = answer.await?;
                respond(&head, response).map(Some)
            })
        })
    }

    /// Register a handler for a **fire-and-forget** specification — one that
    /// defines no success response.
    ///
    /// SPEC §4.4.1: such a specification's consumer signals success by the
    /// absence of a `trust-task-error`, and **MUST NOT** emit a
    /// `#response`-variant document. There is therefore no response payload
    /// for the handler to return, and [`Self::on`] cannot be used — those
    /// specifications get no [`RequestPayload`] impl precisely so that the
    /// mistake is a compile error rather than a runtime surprise.
    ///
    /// The handler returns `Ok(())` to acknowledge or a [`RejectReason`] to
    /// refuse; an acknowledgement is answered `204 No Content`, matching the
    /// status the binding already uses for a duplicate of a completed
    /// fire-and-forget execution ([§5.1](https://trusttasks.org/binding/https/0.2)).
    /// Synchronous, with [`Self::on`]'s blocking rules; see
    /// [`Self::on_ack_async`].
    pub fn on_ack<P, F>(self, handler: F) -> Self
    where
        P: Payload + Send + 'static,
        F: Fn(&TrustTask<P>, &RequestContext) -> Result<(), RejectReason> + Send + Sync + 'static,
    {
        let handler = Arc::new(handler);
        self.register::<P>(move |req, ctx| {
            let handler = Arc::clone(&handler);
            Box::pin(async move {
                run_sync(true, || handler(&req, &ctx))?;
                Ok(None)
            })
        })
    }

    /// [`Self::on_ack`] for a handler that `await`s, with
    /// [`Self::on_async`]'s cancellation rules.
    pub fn on_ack_async<P, F, Fut>(self, handler: F) -> Self
    where
        P: Payload + Send + 'static,
        F: Fn(TrustTask<P>, RequestContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), RejectReason>> + Send + 'static,
    {
        self.register::<P>(move |req, ctx| {
            let answer = handler(req, ctx);
            Box::pin(async move {
                answer.await?;
                Ok(None)
            })
        })
    }

    /// Put a handler in the routing table. The pipeline has run every check
    /// by the time it is called; what is left is turning a refusal into the
    /// §8.1-routed error document the record and the wire carry.
    fn register<P>(
        mut self,
        handler: impl Fn(TrustTask<P>, RequestContext) -> HandlerFuture + Send + Sync + 'static,
    ) -> Self
    where
        P: Payload + Send + 'static,
    {
        if let Some(bound) = P::MAX_DOCUMENT_BYTES {
            self.largest_declared_bound = self.largest_declared_bound.max(bound);
        }
        let routes = std::mem::take(&mut self.routes);
        self.routes = routes.on_async::<P, _, _>(move |req: TrustTask<P>, ctx: RequestContext| {
            let head = head_of(&req);
            let transport = HttpsHandler::new(ctx.local.clone(), ctx.authenticated_sender.clone());
            let answer = handler(req, ctx);
            async move {
                answer
                    .await
                    .map_err(|reason| routed_error(&transport, &head, reason))
            }
        });
        self
    }

    /// Reject any document that arrives with **neither** a transport-
    /// authenticated peer **nor** a `proof`. Defaults to `true`.
    ///
    /// # What this closes
    ///
    /// Without it, an unauthenticated `POST` carrying
    /// `"issuer": "did:web:victim.example"` and no `proof` reaches the
    /// registered handler with [`RequestContext::resolved`]`.issuer` set
    /// to the attacker's string: with no transport peer the framework
    /// falls back entirely to the document's in-band `issuer`, and the
    /// per-spec `IS_PROOF_REQUIRED` check only fires for specs whose front
    /// matter declares `proof` REQUIRED — which most do not, mutating ones
    /// included. The result is an unauthenticated, unattributable request
    /// that a handler cannot tell apart from a genuine one.
    ///
    /// The binding spec states plainly that "this binding does not permit
    /// `proof` to be omitted" (`bindings/https/0.2` §5). This flag is that
    /// rule's runtime representation. It is deliberately blanket — applied
    /// to every registered spec, with no attempt to infer which ones are
    /// consequential, because a per-spec derivation is one mis-classified
    /// spec away from the same hole.
    ///
    /// # Turning it off
    ///
    /// `require_attribution(false)` restores the pre-0.11 behaviour, in
    /// which **any** party that can reach the socket can assert **any**
    /// `issuer` and have handlers act on it. Every authorization decision a
    /// handler makes from `ctx.resolved.issuer` becomes attacker-controlled,
    /// and nothing downstream can recover the distinction. Use it only for
    /// local development and for tests that deliberately exercise the
    /// unauthenticated path — never for a deployment reachable by anything
    /// you do not already trust.
    pub fn require_attribution(mut self, require: bool) -> Self {
        self.require_attribution = require;
        self
    }

    /// Keep the SPEC §7.2 item 11 duplicate-execution record. Defaults to
    /// `true`.
    ///
    /// # What it does
    ///
    /// Item 11 is normative and unconditional for a *consequential Trust
    /// Task*: a document already accepted under an `id` **MUST NOT** cause the
    /// effect a second time, and a *different* document under the same `id`
    /// **MUST** be rejected with `idConflict`. This binding provides no
    /// transport freshness of its own — `bindings/https/0.2` §5 — so the
    /// record kept here is the only thing between a captured request body and
    /// a repeated effect, and between an ordinary proxy or client retry and a
    /// second ACL grant.
    ///
    /// With no [`Self::with_replay_guard`] the server builds an
    /// [`InMemoryReplayGuard`] with its default capacity. That is correct for
    /// a single-process consumer and **not** correct behind a load balancer:
    /// two replicas each hold their own map, so a document accepted by one is
    /// `Fresh` at the other and the effect happens twice. A replicated
    /// deployment **MUST** supply a shared-store guard.
    ///
    /// # Turning it off
    ///
    /// `replay_protection(false)` restores the pre-0.13 behaviour, in which
    /// this server keeps **no** record of what it has executed. Every
    /// consequential handler registered on it becomes replayable by anyone who
    /// can re-send a request body — including any TLS-terminating intermediary
    /// this binding cannot characterise (§5, *Re-origination*) — and a
    /// duplicate delivery from an ordinary retry is executed a second time
    /// with no signal that it was a duplicate. Nothing downstream can recover
    /// the distinction, because nothing downstream is told. Use it only where
    /// every registered specification "explicitly declares repeated execution
    /// safe and intended" (the narrow disapplication item 11 allows), or for
    /// local development.
    pub fn replay_protection(mut self, keep_record: bool) -> Self {
        self.replay_protection = keep_record;
        self
    }

    /// Supply the [`ReplayGuard`] backing [`Self::replay_protection`],
    /// replacing the default [`InMemoryReplayGuard`].
    ///
    /// Use this for any deployment that is replicated, or that must survive a
    /// process restart with its record intact: back the trait with Redis,
    /// Postgres, DynamoDB, or whatever store every replica shares. The
    /// implementation **MUST** make claim-and-record atomic across concurrent
    /// calls — see [`ReplayGuard::claim`].
    ///
    /// Calling this does not by itself enable the record; it is enabled by
    /// default and disabled only by `replay_protection(false)`, which
    /// suppresses this guard too.
    pub fn with_replay_guard<G>(mut self, guard: G) -> Self
    where
        G: ReplayGuard + 'static,
    {
        self.replay_guard = Some(Arc::new(guard));
        self
    }

    /// The freshness bound applied to `issuedAt` / `expiresAt` (SPEC §7.2).
    /// Defaults to [`FreshnessPolicy::consequential`].
    ///
    /// The default is the posture §7.2 (*Bounding the record*) describes:
    /// `issuedAt` REQUIRED and a [`DEFAULT_MAX_AGE`](trust_tasks_rs::DEFAULT_MAX_AGE)
    /// acceptance window. It is not a separate feature from
    /// [`Self::replay_protection`] but the other half of it —
    /// [`FreshnessPolicy::record_expiry`] is what tells the guard how long to
    /// retain each record, and SPEC makes the acceptance window and that
    /// retention *the same bound*. There is deliberately no second TTL to
    /// configure.
    ///
    /// Widen `max_age` for a deployment whose intermediaries may hold a
    /// request longer than five minutes — and understand that doing so widens
    /// the replay record in lockstep, because it is one bound.
    ///
    /// A policy with no `max_age` cannot place a document that carries no
    /// `expiresAt` in any window; while the record is being kept, such a
    /// document is refused with `expired` rather than executed on, per §7.2's
    /// "**MUST NOT** execute a *consequential Trust Task* on it".
    pub fn freshness(mut self, policy: FreshnessPolicy) -> Self {
        self.freshness = policy;
        self
    }

    /// Restrict which DID methods may appear in `proof.verificationMethod`.
    ///
    /// The check runs **before** the configured [`ProofVerifier`] is called.
    /// Verifying a proof means resolving its verification method, and for
    /// `did:web` that resolution is an outbound HTTPS request to a host the
    /// *sender* chose — so an unscreened verifier turns this endpoint into
    /// an SSRF and traffic-amplification primitive for anyone who can reach
    /// it. Pinning the accepted methods (`["key"]`, `["web", "webvh"]`, …)
    /// bounds where that egress can go.
    ///
    /// **Default:** with a verifier configured and neither this nor
    /// [`Self::allow_any_did_method`] called, only
    /// [`DEFAULT_ALLOWED_DID_METHODS`] (`did:key`) passes — a method that
    /// resolves over the network is opt-in, never implied by plugging in a
    /// resolving verifier. A server with no verifier screens nothing by
    /// default: it refuses proof-bearing documents outright and never
    /// egresses. A `verificationMethod` that is not a DID URL at all is
    /// rejected whenever a restriction applies.
    ///
    /// This is a coarse pre-screen at the binding boundary. Depth — resolved
    /// host allow-listing, redirect policy, per-resolution timeouts and
    /// caching — belongs in the resolver behind the verifier, not here.
    pub fn allowed_did_methods<I, S>(mut self, methods: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.did_methods = DidMethodPolicy::Only(methods.into_iter().map(Into::into).collect());
        self
    }

    /// Accept a `proof.verificationMethod` of any DID method, lifting the
    /// `did:key`-only default described on [`Self::allowed_did_methods`].
    ///
    /// Only for a verifier that never resolves over the network, or whose
    /// resolver bounds where it goes (a host allow-list, private addresses
    /// refused, a deadline per resolution) — otherwise any caller can make
    /// this server fetch from a host it names.
    pub fn allow_any_did_method(mut self) -> Self {
        self.did_methods = DidMethodPolicy::Any;
        self
    }

    /// Serve `trust-task-discovery` to unauthenticated callers.
    ///
    /// By default the discovery handler installed by [`Self::with_discovery`]
    /// / [`Self::enable_discovery`] requires a transport-authenticated
    /// sender and answers everyone else with `permissionDenied`: a discovery
    /// response enumerates the full route table, and SPEC §12 says a
    /// responder **SHOULD** authenticate the discoverer before answering.
    ///
    /// Call this when the supported-task set is genuinely public. Order does
    /// not matter — it applies to a discovery handler registered either side
    /// of this call.
    pub fn public_discovery(self) -> Self {
        self.public_discovery.store(true, Ordering::Relaxed);
        self
    }

    /// Per-request wall-clock budget applied by [`HttpsServer::into_router`].
    /// Defaults to [`DEFAULT_REQUEST_TIMEOUT`].
    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = Some(timeout);
        self
    }

    /// Ceiling on concurrently-executing requests, applied by
    /// [`HttpsServer::into_router`]. Defaults to
    /// [`DEFAULT_MAX_CONCURRENT_REQUESTS`].
    pub fn max_concurrent_requests(mut self, limit: usize) -> Self {
        self.max_concurrent_requests = Some(limit);
        self
    }

    /// Register `trust-task-discovery` handlers that respond with the contents
    /// of `registry`, each version in the version it was asked: 0.3, the
    /// current version, and the superseded 0.2 and 0.1 until they are
    /// withdrawn. 0.2 and 0.3 write `frameworkVersion` three-part (SPEC
    /// §5.1.1), 0.1 `MAJOR.MINOR`; only 0.3 can carry the registry's
    /// acceptance window ([`DiscoveryRegistry::acceptance_window_from`]).
    /// Combine with [`Self::on`] in any order; the registry is consulted
    /// afresh on every inbound query.
    ///
    /// Unless [`Self::public_discovery`] is set, the handler answers a
    /// caller with no transport-authenticated sender with `permissionDenied`
    /// rather than enumerating the route table (SPEC §12, discovery
    /// privacy).
    ///
    /// Use this when the server's discoverable set differs from its
    /// actually-handled set — for example, when the server delegates
    /// some types downstream but wants to advertise them as supported.
    /// For the common "advertise exactly what I handle" case, see
    /// [`Self::enable_discovery`].
    pub fn with_discovery(self, registry: DiscoveryRegistry) -> Self {
        self.install_discovery(registry, false)
    }

    /// Snapshot every Type URI currently registered via [`Self::on`] and
    /// install `trust-task-discovery` handlers that advertise them (and
    /// discovery itself), as [`Self::with_discovery`] does. Call this
    /// **after** every other `.on(...)`; URIs registered afterward will not
    /// be included.
    ///
    /// A 0.4 response also advertises the acceptance window this server
    /// enforces — its [`Self::freshness`] policy, whenever that is set — so a
    /// producer holding a document can tell whether it will still be
    /// accepted. A policy with no `max_age` has no window to state, and none
    /// is advertised.
    ///
    /// ```rust,ignore
    /// let server = HttpsServer::builder()
    ///     .local_vid("did:web:server.example")
    ///     .with_auth(BearerAuth::from_pairs([("alice", "did:web:alice.example")]))
    ///     .on::<grant::Payload, _>(handle_grant)
    ///     .on::<revoke::Payload, _>(handle_revoke)
    ///     .enable_discovery() // ← advertises grant + revoke
    ///     .build();
    /// ```
    pub fn enable_discovery(self) -> Self {
        let mut registry: DiscoveryRegistry = self.routes.registered_uris().into_iter().collect();
        // Always advertise discovery itself — otherwise a discoverer who
        // somehow guessed they could ask wouldn't see their own protocol
        // listed back.
        registry.register_payload::<discovery::Payload>();
        // --- trust-task-discovery 0.1 / 0.2 / 0.3 (superseded) ----------------------
        superseded_discovery::register(&mut registry);
        // --- end trust-task-discovery 0.1 / 0.2 / 0.3 (superseded) ------------------
        self.install_discovery(registry, true)
    }

    /// Install the discovery handlers. With `advertise_freshness`, a 0.4
    /// (or 0.3) response states the window of the freshness policy the built server
    /// applies.
    fn install_discovery(self, registry: DiscoveryRegistry, advertise_freshness: bool) -> Self {
        let public = Arc::clone(&self.public_discovery);
        let applied_freshness = Arc::clone(&self.applied_freshness);
        let this = self;
        // --- trust-task-discovery 0.1 / 0.2 / 0.3 (superseded) ----------------------
        let this = this.with_superseded_discovery(registry.clone(), advertise_freshness);
        // --- end trust-task-discovery 0.1 / 0.2 / 0.3 (superseded) ------------------
        this.on_sync::<discovery::Payload, _>(
            move |req, ctx| {
                refuse_unauthenticated_discovery(&public, ctx.authenticated_sender.is_none())?;
                Ok(match applied_freshness.get() {
                    Some(policy) if advertise_freshness => registry
                        .clone()
                        .acceptance_window_from(policy)
                        .respond_to_v0_4(&req.payload),
                    _ => registry.respond_to_v0_4(&req.payload),
                })
            },
            false,
        )
    }

    /// Build the server. Run it with [`HttpsServer::serve`].
    ///
    /// Note the default authenticator is a [`BearerAuth`] with an empty token
    /// table, so every request is unauthenticated unless
    /// [`Self::with_auth`] was called. That is survivable only because
    /// [`Self::require_attribution`] defaults to `true`; read its
    /// documentation before turning it off.
    pub fn build(self) -> HttpsServer {
        let auth = self.auth.unwrap_or_else(|| Box::new(BearerAuth::new()));
        // A builder is built once, so this is the first and only `set`; the
        // discovery handler reads the policy the server actually applies.
        let _ = self.applied_freshness.set(self.freshness);
        let allowed_did_methods = match self.did_methods {
            DidMethodPolicy::Only(methods) => Some(methods),
            DidMethodPolicy::Any => None,
            DidMethodPolicy::Default => self.verifier.is_some().then(|| {
                DEFAULT_ALLOWED_DID_METHODS
                    .iter()
                    .map(|m| (*m).to_string())
                    .collect()
            }),
        };
        // SPEC §12.4: the body is buffered before anything else, so the limit
        // is what an unauthenticated sender can make a request cost. It grows
        // past the transport default only as far as a registered task's own
        // declaration requires.
        let body_limit = self
            .max_body_bytes
            .unwrap_or_else(|| MAX_BODY_BYTES.max(self.largest_declared_bound));
        HttpsServer {
            state: Arc::new(ServerState {
                local_vid: self.local_vid,
                auth,
                routes: self.routes,
                verifier: self.verifier,
                payload_validator: self.payload_validator,
                require_attribution: self.require_attribution,
                allowed_did_methods,
                // Secure by default: the record is kept unless the deployment
                // said otherwise, and the default guard is the in-process one.
                replay_guard: self.replay_protection.then(|| {
                    self.replay_guard
                        .unwrap_or_else(|| Arc::new(InMemoryReplayGuard::default()))
                }),
                freshness: self.freshness,
                default_document_bound: self.max_body_bytes.unwrap_or(MAX_BODY_BYTES),
            }),
            body_limit,
            request_timeout: self.request_timeout.unwrap_or(DEFAULT_REQUEST_TIMEOUT),
            max_concurrent_requests: self
                .max_concurrent_requests
                .unwrap_or(DEFAULT_MAX_CONCURRENT_REQUESTS),
        }
    }
}

/// An HTTPS Trust Tasks server.
///
/// Build with [`HttpsServer::builder`], register handlers via
/// [`HttpsServerBuilder::on`] / [`HttpsServerBuilder::on_async`], then
/// [`Self::serve`].
pub struct HttpsServer {
    state: Arc<ServerState>,
    body_limit: usize,
    request_timeout: Duration,
    max_concurrent_requests: usize,
}

impl HttpsServer {
    /// Start a new builder.
    pub fn builder() -> HttpsServerBuilder {
        HttpsServerBuilder {
            local_vid: None,
            auth: None,
            routes: AsyncDispatcher::new(),
            largest_declared_bound: 0,
            max_body_bytes: None,
            verifier: None,
            payload_validator: None,
            // Secure by default: see `HttpsServerBuilder::require_attribution`.
            require_attribution: true,
            did_methods: DidMethodPolicy::Default,
            // Secure by default: see `HttpsServerBuilder::replay_protection`.
            replay_protection: true,
            replay_guard: None,
            freshness: FreshnessPolicy::consequential(),
            public_discovery: Arc::new(AtomicBool::new(false)),
            applied_freshness: Arc::new(OnceLock::new()),
            request_timeout: None,
            max_concurrent_requests: None,
        }
    }

    /// The request-body limit [`Self::into_router`] applies: [`MAX_BODY_BYTES`],
    /// or the largest `maxDocumentBytes` a registered request type declares if
    /// that is larger, or what [`HttpsServerBuilder::max_body_bytes`] set.
    pub fn body_limit(&self) -> usize {
        self.body_limit
    }

    /// Build the axum [`Router`] without starting a listener — useful for
    /// integration tests that want to spawn the app inline.
    ///
    /// The router applies an explicit [`DefaultBodyLimit`] of
    /// [`Self::body_limit`] as an audited DoS control (SPEC §12.2): the body is
    /// buffered and parsed *before* authentication, so an unbounded body would
    /// otherwise be a pre-auth memory-exhaustion vector. JSON nesting depth is
    /// separately bounded by `serde_json`'s default 128-level recursion limit,
    /// so a pathologically nested body within the size budget fails to parse
    /// (→ `malformedRequest`) rather than overflowing the stack. Once the type
    /// is known, the document is held to its own specification's bound.
    ///
    /// It also applies two liveness controls, both configurable on the
    /// builder ([`HttpsServerBuilder::request_timeout`],
    /// [`HttpsServerBuilder::max_concurrent_requests`]):
    ///
    /// * a [`TimeoutLayer`] — a request that outlives its budget is
    ///   abandoned with `408 Request Timeout` rather than holding a
    ///   connection and a task open indefinitely (slowloris). It fires at an
    ///   `await`, so it bounds async handlers and not synchronous ones — see
    ///   the module docs, *Handlers and blocking*;
    /// * a [`ConcurrencyLimitLayer`] — requests past the ceiling wait for a
    ///   permit instead of each pinning a parse buffer, bounding the memory
    ///   a flood can commit.
    ///
    /// Both sit **outside** the body limit, so they also bound the time and
    /// concurrency spent reading a body that is still under the size cap.
    pub fn into_router(self) -> Router {
        let timeout = self.request_timeout;
        let concurrency = self.max_concurrent_requests;
        Router::new()
            .route("/trust-tasks", post(dispatch_handler))
            .layer(DefaultBodyLimit::max(self.body_limit))
            .layer(
                ServiceBuilder::new()
                    // Outermost: the inner layers are fallible, axum's
                    // `Router` is not. Anything that is not a timeout is a
                    // middleware fault, not a client error, so it is a 500.
                    .layer(HandleErrorLayer::new(|err: tower::BoxError| async move {
                        if err.is::<tower::timeout::error::Elapsed>() {
                            StatusCode::REQUEST_TIMEOUT
                        } else {
                            StatusCode::INTERNAL_SERVER_ERROR
                        }
                    }))
                    .layer(ConcurrencyLimitLayer::new(concurrency))
                    .layer(TimeoutLayer::new(timeout)),
            )
            .with_state(self.state)
    }

    /// Bind to `addr` and serve incoming requests. Returns when the
    /// listener stops.
    ///
    /// The router returned by [`Self::into_router`] carries the body-size,
    /// timeout and concurrency controls; this method adds nothing beyond
    /// binding the listener, so a caller that builds its own listener gets
    /// the same protections.
    pub async fn serve(self, addr: impl tokio::net::ToSocketAddrs) -> std::io::Result<()> {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, self.into_router()).await
    }
}

async fn dispatch_handler(
    State(state): State<Arc<ServerState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // ─── Content-Type. The binding spec (§2) makes
    // `Content-Type: application/json` a MUST on the request. Enforcing it
    // is not pedantry: `text/plain` is one of the three media types a
    // cross-origin `fetch`/form POST can send *without* a preflight, so
    // accepting it lets any web page in the victim's browser drive this
    // endpoint. Requiring JSON forces the preflight, and the browser then
    // refuses on our behalf. 415 is a transport-level answer, not a
    // framework error document — nothing here has been parsed yet.
    if !is_json_content_type(&headers) {
        return StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response();
    }

    // ─── Parse the body into a TrustTask<Value>.
    let doc: TrustTask<Value> = match serde_json::from_slice(&body) {
        Ok(d) => d,
        // `RejectReason::malformed_from_serde` rather than `format!("{e}")`:
        // `serde_json`'s `Display` renders the member path and the byte offset
        // it failed at ("missing field `subject` at line 1 column 214"), which
        // on the wire describes this consumer's internal type layout and its
        // framing of the body to anyone willing to POST malformed JSON — the
        // §10.4 leak. The category is what a producer can act on; the detail
        // belongs in the operator's log.
        Err(e) => {
            return error_response(free_standing_error(RejectReason::malformed_from_serde(&e)))
        }
    };
    // The same bytes as a plain `Value`: a proof is verified over the document
    // as received, not over `doc` re-serialised, which re-renders the
    // framework timestamps, and the item-11 digest is taken over it for the
    // same reason. Parsed separately rather than `doc` being built from it,
    // because the typed parse is the one that refuses a duplicated top-level
    // member, which a `Value` silently resolves to the last.
    let received: Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => {
            return error_response(free_standing_error(RejectReason::malformed_from_serde(&e)))
        }
    };

    // ─── Authenticate the bearer token (if any) into a peer VID — before any
    // framework check, so the pipeline knows whether a sender is
    // authenticated (§8.1 suppression, the attribution gate).
    let peer_vid = match extract_bearer(&headers) {
        Some(token) => state.auth.resolve_async(token).await,
        None => None,
    };
    let transport = HttpsHandler::new(state.local_vid.clone(), peer_vid);

    // ─── The §7.2 pipeline: `consume_dispatched_answer`, configured with
    // what this binding adds.
    let replay = match &state.replay_guard {
        Some(guard) => ReplayPolicy::Guard(guard.as_ref()),
        None => ReplayPolicy::NotConsequential,
    };
    let mut checks = ConsumeChecks {
        freshness: state.freshness,
        replay,
        ..ConsumeChecks::not_consequential()
    }
    .require_attribution(state.require_attribution)
    .with_received_bytes(body.len(), Some(state.default_document_bound));
    checks.proof_did_methods = state.allowed_did_methods.clone();

    let proof_policy: ProofPolicy<'_, dyn DynProofVerifier> = match &state.verifier {
        Some(verifier) => ProofPolicy::VerifyReceived(verifier.as_ref(), &received),
        // Silently dropping a producer's proof would mislead them about the
        // exchange's integrity guarantees, and naming the server's
        // configuration on the wire would let a probe enumerate verifier
        // coverage across a fleet — so a proof is refused with the
        // framework-shared wording.
        None => ProofPolicy::RejectIfPresentReceived(&received),
    };
    let payload_policy: PayloadPolicy<'_, dyn PayloadValidator + Send + Sync> =
        match &state.payload_validator {
            Some(validator) => PayloadPolicy::Validate(validator.as_ref()),
            None => PayloadPolicy::AcceptUnvalidated,
        };

    let authenticated_sender = transport.peer().map(str::to_string);
    let local = transport.local().map(str::to_string);
    let outcome = consume_dispatched_answer(
        &transport,
        proof_policy,
        payload_policy,
        checks,
        &state.routes,
        doc,
        state.local_vid.as_deref().unwrap_or(""),
        chrono::Utc::now(),
        new_id,
        |resolved| RequestContext {
            authenticated_sender,
            local,
            resolved,
        },
    )
    .await;

    match outcome {
        AnswerOutcome::Handled(Some(body)) => success_response(body),
        // SPEC §4.4.1 fire-and-forget: nothing to return, and the absence of
        // a `trust-task-error` is itself the success signal. 204 is the same
        // answer the binding gives a duplicate of a completed fire-and-forget
        // execution.
        AnswerOutcome::Handled(None) => StatusCode::NO_CONTENT.into_response(),
        AnswerOutcome::Rejected(error) => error_response(error),
        // §8.1: no response SHOULD be emitted. HTTP has no way not to answer,
        // so the answer is the one a body that did not parse gets — see
        // `suppressed_error_response`.
        AnswerOutcome::Suppressed => error_response(suppressed_error_response()),

        // Already executed, and the result was retained. §7.2 (*Disposition
        // of a duplicate*): "the consumer SHOULD return the previously
        // determined result" — the same body, with the status it had: a
        // response document is 200, a recorded non-retryable refusal its own
        // error status. No second dispatch, which is what makes a §8.4
        // bit-for-bit retry safe for the producer to send.
        AnswerOutcome::Duplicate {
            prior_response: Some(prior),
            ..
        } => replayed_response(prior),

        // Already accepted; the original execution has not finished. SPEC:
        // "the consumer SHOULD return or expose the existing execution state
        // rather than begin another." HTTP's word for *accepted, outcome not
        // yet available* is `202 Accepted`, and it is the honest one here: the
        // document was accepted (it is being executed right now, by the first
        // delivery), there is no result to return yet, and nothing failed. 200
        // would be a lie about having a result, 409 would claim a conflict
        // that does not exist, and any error code would report a failure that
        // did not happen.
        AnswerOutcome::Duplicate {
            in_flight: true, ..
        } => StatusCode::ACCEPTED.into_response(),

        // Already executed, and no result was retained — the fire-and-forget
        // shape (§4.4.1), or a guard that keeps only the claim. §7.2: "where
        // the specification defines no success response, silence is the
        // correct disposition", and "in no case is a duplicate reported as
        // `taskFailed`; the task did not fail, it already happened." `204 No
        // Content` is silence with a status line, which is the most HTTP will
        // allow.
        AnswerOutcome::Duplicate { .. } => StatusCode::NO_CONTENT.into_response(),
    }
}

/// Run a synchronous handler. One that may block runs under
/// `block_in_place` on a multi-threaded runtime, so the worker's other tasks
/// are moved off it, and inline otherwise — never on the blocking pool, where
/// it would outlive a cancelled request with nobody to record its result
/// (module docs, *Handlers and blocking*).
fn run_sync<T>(may_block: bool, handler: impl FnOnce() -> T) -> T {
    if may_block {
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            if runtime.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread {
                return tokio::task::block_in_place(handler);
            }
        }
    }
    handler()
}

/// The `#response` document answering `request` with `payload`, as JSON.
fn respond<P, R: Serialize>(request: &TrustTask<P>, payload: R) -> Result<Value, RejectReason> {
    serde_json::to_value(request.respond_with(new_id(), payload)).map_err(|_| {
        RejectReason::InternalError {
            reason: "the response could not be serialised".to_string(),
        }
    })
}

/// The envelope of `doc` without its payload: what an error or response
/// document is built from once the payload has been handed to a handler.
fn head_of<P>(doc: &TrustTask<P>) -> TrustTask<()> {
    TrustTask {
        id: doc.id.clone(),
        thread_id: doc.thread_id.clone(),
        parent_thread_id: doc.parent_thread_id.clone(),
        ceremony: doc.ceremony.clone(),
        type_uri: doc.type_uri.clone(),
        issuer: doc.issuer.clone(),
        recipient: doc.recipient.clone(),
        issued_at: doc.issued_at,
        expires_at: doc.expires_at,
        payload: (),
        context: doc.context.clone(),
        proof: doc.proof.clone(),
        extra: doc.extra.clone(),
    }
}

/// A handler's refusal as the error document §8.1 routes: addressed by
/// [`TransportHandler::reject`], or — where that suppresses the answer — the
/// generic one HTTP sends instead.
fn routed_error(
    transport: &HttpsHandler,
    request: &TrustTask<()>,
    reason: RejectReason,
) -> ErrorResponse {
    transport
        .reject(request, new_id(), reason)
        .unwrap_or_else(suppressed_error_response)
}

fn new_id() -> String {
    format!("urn:uuid:{}", Uuid::new_v4())
}

/// `true` when the request declares `application/json` (parameters such as
/// `; charset=utf-8` are permitted and ignored). A missing header is *not*
/// acceptable: the binding spec makes `Content-Type` a MUST on the request,
/// and treating "absent" as "probably JSON" would re-open the simple-request
/// path this check exists to close.
fn is_json_content_type(headers: &HeaderMap) -> bool {
    let Some(value) = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
    else {
        return false;
    };
    let media_type = value.split(';').next().unwrap_or("").trim();
    media_type.eq_ignore_ascii_case("application/json")
}

fn extract_bearer(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let token = value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))?;
    Some(token.trim())
}

fn success_response(body: Value) -> Response {
    let bytes = serde_json::to_vec(&body).expect("serialise success body");
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json")],
        bytes,
    )
        .into_response()
}

/// A duplicate's recorded answer, with the status it was first given: a
/// recorded `trust-task-error` (a non-retryable refusal) keeps its error
/// status, anything else is the 200 of a response document.
fn replayed_response(prior: Value) -> Response {
    let is_error = prior
        .get("type")
        .and_then(Value::as_str)
        .and_then(|t| t.parse::<trust_tasks_rs::TypeUri>().ok())
        .is_some_and(|t| {
            t.for_routing() == trust_tasks_rs::trust_task_error_type_uri().for_routing()
        });
    if is_error {
        if let Ok(error) = serde_json::from_value::<ErrorResponse>(prior.clone()) {
            return error_response(error);
        }
    }
    success_response(prior)
}

fn error_response(error_doc: ErrorResponse) -> Response {
    // Status follows the error document actually emitted, not the inbound
    // reason. The suppressed identity-mismatch path rewrites the body to a
    // generic `malformedRequest` (SPEC §12.4); the status MUST match so it is
    // indistinguishable from a plain parse failure (no 403-vs-400 oracle for an
    // unauthenticated prober).
    let status = status_for_code(&error_doc.payload.code);
    let status = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let body = serde_json::to_vec(&error_doc).expect("serialise error response");
    (status, [(header::CONTENT_TYPE, "application/json")], body).into_response()
}

/// An error document for a body that did not parse into a Trust Task at all,
/// so there is nothing to address it to.
///
/// `trust_task_error_type_uri()` rather than a version spelled out here: this
/// crate once emitted `trust-task-error/0.2` on its fallback paths while
/// `trust-tasks-rs` and the TypeScript runtime both emitted `0.5`, so the
/// version a producer saw depended on which branch it hit. One definition,
/// one answer.
fn free_standing_error(reason: RejectReason) -> ErrorResponse {
    let mut doc = TrustTask::new(
        new_id(),
        trust_tasks_rs::trust_task_error_type_uri(),
        ErrorPayload::from(reason),
    );
    doc.issued_at = Some(trust_tasks_rs::issued_at_now());
    doc
}

/// When the pipeline suppresses an answer (§8.1: an `identityMismatch` with
/// no transport-authenticated sender), the framework rule is that the
/// consumer SHOULD NOT emit a response. HTTP gives us no way to "not emit" —
/// the TCP connection already exists and the peer is waiting for a status
/// line. We therefore emit the *same* generic `malformedRequest`/400 that a
/// body parse failure produces.
///
/// Crucially we MUST NOT echo the `identityMismatch` code or status here
/// (SPEC §12.4): an unauthenticated prober who POSTs a spoofed in-band
/// identity would otherwise learn, from the code + 4xx, that this consumer
/// performs the cross-check and that the identity was contested — an
/// identity-probing oracle. Collapsing to the indistinguishable generic
/// rejection removes that signal; the diagnostic loss for honest peers is
/// the deliberate, safer trade.
fn suppressed_error_response() -> ErrorResponse {
    free_standing_error(RejectReason::MalformedRequest {
        reason: String::new(),
    })
}

/// Unless discovery is public, refuse a caller with no transport-authenticated
/// sender rather than enumerating the route table (SPEC §12, discovery privacy).
///
/// Deliberately the same generic wording any other permission failure uses — a
/// discovery-specific message would itself confirm that discovery is installed
/// here.
fn refuse_unauthenticated_discovery(
    public: &AtomicBool,
    unauthenticated: bool,
) -> Result<(), RejectReason> {
    if !public.load(Ordering::Relaxed) && unauthenticated {
        return Err(RejectReason::PermissionDenied {
            reason: "discovery requires an authenticated sender".into(),
        });
    }
    Ok(())
}
