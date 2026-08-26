//! Axum-based server: receives Trust Task documents over HTTP, dispatches
//! them to per-spec handlers, and returns a typed response document or a
//! `trust-task-error` document with the appropriate HTTP status code.
//!
//! Single endpoint: `POST /trust-tasks` accepts any document the server has
//! a handler registered for. Bearer auth (`Authorization: Bearer <token>`)
//! is consulted before framework validation; the resulting VID becomes the
//! transport-authenticated sender for §4.8.1 precedence.
//!
//! ## Request pipeline order
//!
//! The order below is load-bearing, not incidental. Every step that can
//! cost the server work an unauthenticated sender chose — above all DID
//! resolution inside a configured [`ProofVerifier`], which for `did:web`
//! is an outbound HTTPS request to a host named in the request body —
//! runs **after** the cheap, local rejections:
//!
//! 1. `Content-Type` must be `application/json` (binding spec §2), else 415.
//! 2. Body parses as a Trust Task document.
//! 3. Bearer token → transport-authenticated peer (local lookup).
//! 4. **Route lookup** — an unregistered `type` is `unsupportedType` here,
//!    before anything else touches the network.
//! 5. `resolve_parties` (§4.8.1), `validate_basic` (expiry, recipient), and
//!    the **freshness bound** over `issuedAt` / `expiresAt` (§7.2 item 13) —
//!    see [`HttpsServerBuilder::freshness`].
//! 6. **Attribution gate** — see [`HttpsServerBuilder::require_attribution`].
//! 7. `proof.verificationMethod` DID-method pre-screen — see
//!    [`HttpsServerBuilder::allowed_did_methods`].
//! 8. Proof verification (the only step that may egress).
//! 9. **Duplicate-execution claim** (§7.2 item 11) — see
//!    [`HttpsServerBuilder::replay_protection`].
//! 10. Per-spec policy + the registered handler.
//!
//! ## Where the replay claim sits, and why
//!
//! Step 9 is the `validated → accepted` transition: the last instant at
//! which the server can still refuse a document for a reason that has nothing
//! to do with having seen it before, and the first at which it has committed
//! to execute. Claiming any earlier would write a record for documents the
//! server then refuses — burning the `id` of every malformed, stale,
//! unattributable or badly-signed arrival, so that a corrected resend came
//! back `idConflict` forever and a stranger could pre-burn an `id` it had
//! merely observed. Claiming any later is not a claim at all: the effect
//! would already have happened.
//!
//! The one check that runs *after* the claim is the per-spec policy inside
//! the dispatch closure (`enforce_spec_policy`), which needs the typed
//! payload the closure alone can produce. A refusal there releases the claim
//! (see [`ReplayGuard::release`]), so nothing is burned by it either.

use std::collections::{BTreeSet, HashMap};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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
    discovery::DiscoveryRegistry, document_digest, erase_verifier,
    specs::trust_task_discovery::v0_1 as discovery, DocumentDigest, DynProofVerifier, ErrorPayload,
    ErrorResponse, FreshnessPolicy, InMemoryReplayGuard, Payload, ProofVerifier, RejectReason,
    ReplayGuard, ReplayVerdict, RequestPayload, ResolvedParties, StaleReason, StandardCode,
    TransportHandler, TrustTask, PROOF_NOT_ACCEPTED_BY_POLICY,
};
use uuid::Uuid;

use crate::auth::{Auth, BearerAuth};
use crate::handler::HttpsHandler;
use crate::status::status_for_code;

/// Maximum accepted request-body size (SPEC §10.2), applied by
/// [`HttpsServer::into_router`]. Trust Task payloads are small; 256 KiB is
/// generous headroom while bounding pre-auth memory use. Callers needing a
/// different bound can rebuild the router with their own
/// [`DefaultBodyLimit`] layer — which is why the value is public: a caller
/// replacing the layer wants to know what it is replacing.
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

/// A spec-specific handler stored type-erased in the dispatch table.
///
/// Inputs: the parsed (but untyped) inbound document and the request
/// context. Output: the JSON body of a success response (a fully-formed
/// `#response` document); `None` where the specification defines no success
/// response and the handler acknowledged instead; or a [`RejectReason`] the
/// server converts into a `trust-task-error` response via §8.1 routing.
///
/// `Option` rather than an empty `Value`: SPEC §4.4.1 distinguishes "a
/// `#response` document whose payload happens to be empty" from "this
/// specification defines no response at all", and the two earn different
/// statuses (200 with a body, 204 without). Collapsing them would make a
/// fire-and-forget acknowledgement indistinguishable on the wire from a
/// `#response` carrying `{}`.
type DispatchFn = Box<
    dyn Fn(TrustTask<Value>, &RequestContext) -> Result<Option<Value>, RejectReason> + Send + Sync,
>;

struct Route {
    dispatch: DispatchFn,
}

/// Internal state shared by every axum request handler.
struct ServerState {
    local_vid: Option<String>,
    auth: Box<dyn Auth>,
    routes: HashMap<String, Route>,
    verifier: Option<Arc<dyn DynProofVerifier>>,
    require_attribution: bool,
    allowed_did_methods: Option<BTreeSet<String>>,
    /// `None` only when the deployment called
    /// `replay_protection(false)` — see that method's warning.
    replay_guard: Option<Arc<dyn ReplayGuard>>,
    freshness: FreshnessPolicy,
}

/// Builder for [`HttpsServer`].
pub struct HttpsServerBuilder {
    local_vid: Option<String>,
    auth: Option<Box<dyn Auth>>,
    routes: HashMap<String, Route>,
    verifier: Option<Arc<dyn DynProofVerifier>>,
    require_attribution: bool,
    allowed_did_methods: Option<BTreeSet<String>>,
    replay_protection: bool,
    replay_guard: Option<Arc<dyn ReplayGuard>>,
    freshness: FreshnessPolicy,
    /// Shared with the discovery handler closure so `.public_discovery()`
    /// can be called either side of `.enable_discovery()`.
    public_discovery: Arc<AtomicBool>,
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

    /// Plug in an [`Auth`] implementation. Defaults to a [`BearerAuth`]
    /// with no tokens — every request is treated as unauthenticated and
    /// the framework falls back entirely to in-band identity.
    pub fn with_auth(mut self, auth: impl Auth) -> Self {
        self.auth = Some(Box::new(auth));
        self
    }

    /// Plug in an in-band [`ProofVerifier`]. When configured, the
    /// server verifies the `proof` member of every proof-bearing
    /// document and rejects with `proof_invalid` on failure; when
    /// absent, the server rejects proof-bearing documents with
    /// `malformed_request` (the
    /// [`PROOF_NOT_ACCEPTED_BY_POLICY`](trust_tasks_rs::PROOF_NOT_ACCEPTED_BY_POLICY)
    /// rule — matches `consume_inbound` under
    /// [`ProofPolicy::RejectIfPresent`](trust_tasks_rs::ProofPolicy::RejectIfPresent)).
    ///
    /// The verifier is invoked between identity resolution (§7.2 item
    /// 6) and the per-spec dispatch closure (`IS_PROOF_REQUIRED` check
    /// and audience binding), so a failed signature short-circuits
    /// before the user handler runs. It is deliberately the **last**
    /// step before dispatch: it is the only one that may egress, so
    /// route lookup, transport auth and the attribution gate all run
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

    /// Register a handler for the request payload type `P`. The handler
    /// receives the typed request and a [`RequestContext`]; it returns
    /// either the response payload (which the server wraps in a
    /// `#response`-variant document via [`TrustTask::respond_with`]) or a
    /// [`RejectReason`] (which the server wraps in a `trust-task-error`
    /// document via [`TransportHandler::reject`], applying SPEC §8.1
    /// routing).
    pub fn on<P, F>(mut self, handler: F) -> Self
    where
        P: RequestPayload + 'static,
        P::Response: Serialize + 'static,
        F: Fn(&TrustTask<P>, &RequestContext) -> Result<P::Response, RejectReason>
            + Send
            + Sync
            + 'static,
    {
        let dispatch: DispatchFn = Box::new(move |doc: TrustTask<Value>, ctx: &RequestContext| {
            // Downcast payload to P.
            let typed = downcast::<P>(doc)?;

            // SPEC §7.2 items 5b + 7A + 8 — the flag-driven per-spec checks
            // (recipient-REQUIRED, proof-REQUIRED, audience binding). This is
            // the first point where the typed payload (and its codegen-emitted
            // flags) is available. It calls the SAME method as the library
            // `consume_inbound` path (`TrustTask::enforce_spec_policy`) so the
            // two pipelines cannot diverge on the check set. The non-typed
            // checks (expiry, cross-check, proof verification) ran upstream in
            // `dispatch_handler`.
            typed.enforce_spec_policy()?;

            // Invoke user handler.
            let response_payload = handler(&typed, ctx)?;
            let new_id = format!("urn:uuid:{}", Uuid::new_v4());
            let response_doc = typed.respond_with(new_id, response_payload);
            Ok(Some(
                serde_json::to_value(&response_doc).expect("response serialises (typed structs)"),
            ))
        });

        let key = P::type_uri().for_routing().to_string();
        self.routes.insert(key, Route { dispatch });
        self
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
    pub fn on_ack<P, F>(mut self, handler: F) -> Self
    where
        P: Payload + 'static,
        F: Fn(&TrustTask<P>, &RequestContext) -> Result<(), RejectReason> + Send + Sync + 'static,
    {
        let dispatch: DispatchFn = Box::new(move |doc: TrustTask<Value>, ctx: &RequestContext| {
            let typed = downcast::<P>(doc)?;
            // Same §7.2 items 5b/7A/8 gate as `on`; a fire-and-forget task is
            // no less consequential for having nothing to say back.
            typed.enforce_spec_policy()?;
            handler(&typed, ctx)?;
            Ok(None)
        });

        let key = P::type_uri().for_routing().to_string();
        self.routes.insert(key, Route { dispatch });
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
    /// Defaults to no restriction, which preserves existing behaviour for
    /// servers with no verifier configured (those reject proof-bearing
    /// documents outright and never egress). A `verificationMethod` that is
    /// not a DID URL at all is rejected whenever a restriction is set.
    ///
    /// This is a coarse pre-screen at the binding boundary. Depth — resolved
    /// host allow-listing, redirect policy, per-resolution timeouts and
    /// caching — belongs in the resolver behind the verifier, not here.
    pub fn allowed_did_methods<I, S>(mut self, methods: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.allowed_did_methods = Some(methods.into_iter().map(Into::into).collect());
        self
    }

    /// Serve `trust-task-discovery` to unauthenticated callers.
    ///
    /// By default the discovery handler installed by [`Self::with_discovery`]
    /// / [`Self::enable_discovery`] requires a transport-authenticated
    /// sender and answers everyone else with `permissionDenied`: a discovery
    /// response enumerates the full route table, and SPEC §10 says a
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

    /// Register a `trust-task-discovery/0.1` handler that responds with
    /// the contents of `registry`. Combine with [`Self::on`] in any order;
    /// the registry is consulted afresh on every inbound query.
    ///
    /// Unless [`Self::public_discovery`] is set, the handler answers a
    /// caller with no transport-authenticated sender with `permissionDenied`
    /// rather than enumerating the route table (SPEC §10, discovery
    /// privacy).
    ///
    /// Use this when the server's discoverable set differs from its
    /// actually-handled set — for example, when the server delegates
    /// some types downstream but wants to advertise them as supported.
    /// For the common "advertise exactly what I handle" case, see
    /// [`Self::enable_discovery`].
    pub fn with_discovery(self, registry: DiscoveryRegistry) -> Self {
        let public = Arc::clone(&self.public_discovery);
        self.on::<discovery::Payload, _>(move |req, ctx| {
            if !public.load(Ordering::Relaxed) && ctx.authenticated_sender.is_none() {
                // Deliberately the same generic wording any other permission
                // failure uses — a discovery-specific message would itself
                // confirm that discovery is installed here.
                return Err(RejectReason::PermissionDenied {
                    reason: "discovery requires an authenticated sender".into(),
                });
            }
            Ok(registry.respond_to(&req.payload))
        })
    }

    /// Snapshot every Type URI currently registered via [`Self::on`] and
    /// install a `trust-task-discovery/0.1` handler that advertises them.
    /// Call this **after** every other `.on(...)`; URIs registered
    /// afterward will not be included.
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
        let mut registry: DiscoveryRegistry = self.routes.keys().cloned().collect();
        // Always advertise discovery itself — otherwise a discoverer who
        // somehow guessed they could ask wouldn't see their own protocol
        // listed back.
        registry.register_payload::<discovery::Payload>();
        self.with_discovery(registry)
    }

    /// Build the server. Run it with [`HttpsServer::serve`].
    ///
    /// Note the default [`Auth`] is a [`BearerAuth`] with an empty token
    /// table, so every request is unauthenticated unless
    /// [`Self::with_auth`] was called. That is survivable only because
    /// [`Self::require_attribution`] defaults to `true`; read its
    /// documentation before turning it off.
    pub fn build(self) -> HttpsServer {
        let auth = self.auth.unwrap_or_else(|| Box::new(BearerAuth::new()));
        HttpsServer {
            state: Arc::new(ServerState {
                local_vid: self.local_vid,
                auth,
                routes: self.routes,
                verifier: self.verifier,
                require_attribution: self.require_attribution,
                allowed_did_methods: self.allowed_did_methods,
                // Secure by default: the record is kept unless the deployment
                // said otherwise, and the default guard is the in-process one.
                replay_guard: self.replay_protection.then(|| {
                    self.replay_guard
                        .unwrap_or_else(|| Arc::new(InMemoryReplayGuard::default()))
                }),
                freshness: self.freshness,
            }),
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
/// [`HttpsServerBuilder::on`], then [`Self::serve`].
pub struct HttpsServer {
    state: Arc<ServerState>,
    request_timeout: Duration,
    max_concurrent_requests: usize,
}

impl HttpsServer {
    /// Start a new builder.
    pub fn builder() -> HttpsServerBuilder {
        HttpsServerBuilder {
            local_vid: None,
            auth: None,
            routes: HashMap::new(),
            verifier: None,
            // Secure by default: see `HttpsServerBuilder::require_attribution`.
            require_attribution: true,
            allowed_did_methods: None,
            // Secure by default: see `HttpsServerBuilder::replay_protection`.
            replay_protection: true,
            replay_guard: None,
            freshness: FreshnessPolicy::consequential(),
            public_discovery: Arc::new(AtomicBool::new(false)),
            request_timeout: None,
            max_concurrent_requests: None,
        }
    }

    /// Build the axum [`Router`] without starting a listener — useful for
    /// integration tests that want to spawn the app inline.
    ///
    /// The router applies an explicit [`DefaultBodyLimit`] of
    /// [`MAX_BODY_BYTES`] (256 KiB) as an audited DoS control (SPEC §10.2): the body is
    /// buffered and parsed *before* authentication, so an unbounded body would
    /// otherwise be a pre-auth memory-exhaustion vector. JSON nesting depth is
    /// separately bounded by `serde_json`'s default 128-level recursion limit,
    /// so a pathologically nested body within the size budget fails to parse
    /// (→ `malformedRequest`) rather than overflowing the stack.
    ///
    /// It also applies two liveness controls, both configurable on the
    /// builder ([`HttpsServerBuilder::request_timeout`],
    /// [`HttpsServerBuilder::max_concurrent_requests`]):
    ///
    /// * a [`TimeoutLayer`] — a request that outlives its budget is
    ///   abandoned with `408 Request Timeout` rather than holding a
    ///   connection and a task open indefinitely (slowloris);
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
            .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
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
    // ─── 0. Content-Type. The binding spec (§2) makes
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

    // ─── 1. Parse the body into a TrustTask<Value>.
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
            return reject_response(None, None, RejectReason::malformed_from_serde(&e));
        }
    };

    // ─── 2. Authenticate the bearer token (if any) into a peer VID. Purely
    // local — an in-process map, a JWT signature check, a database lookup.
    // No network egress the sender can steer.
    let peer_vid = extract_bearer(&headers).and_then(|tok| state.auth.resolve(tok));

    // ─── 3. Build the per-request HttpsHandler.
    let handler = HttpsHandler::new(state.local_vid.clone(), peer_vid);

    // ─── 4. Routing FIRST — deliberately ahead of every step that can cost
    // this server work an unauthenticated sender chose. Proof verification
    // (step 8) resolves the `verificationMethod` DID, which for `did:web` is
    // an outbound HTTPS request to a host named in the request body. With
    // routing last, a stranger could aim this endpoint at an arbitrary host
    // by POSTing a document whose `type` we do not even implement, and get
    // an amplifier for free. There is no rate limit here; the ordering is
    // the control.
    let routing_key = doc.type_uri.for_routing().to_string();
    let Some(route) = state.routes.get(&routing_key) else {
        return reject_response(
            Some(&handler),
            Some(&doc),
            RejectReason::UnsupportedType {
                type_uri: routing_key,
            },
        );
    };

    // ─── 5. Identity resolution (§4.8.1), then the framework-level checks:
    // expiry + recipient identity.
    let resolved = match handler.resolve_parties(&doc) {
        Ok(r) => r,
        Err(consistency) => {
            let reason: RejectReason = consistency.into();
            return reject_response(Some(&handler), Some(&doc), reason);
        }
    };
    let now = chrono::Utc::now();
    let my_vid = state.local_vid.as_deref().unwrap_or("");
    if let Err(reason) = doc.validate_basic(now, my_vid) {
        return reject_response(Some(&handler), Some(&doc), reason);
    }

    // ─── 5b. Freshness (§7.2 item 13). `validate_basic` honours `expiresAt`,
    // which is optional and which a producer sets for its own reasons; on its
    // own it leaves a document stamped years ago — or years hence — acceptable
    // forever. This is also the bound the replay record at step 9 is retained
    // for: §7.2 (*Bounding the record*) makes the acceptance window and the
    // record's retention one bound, not two.
    if let Err(reason) = doc.validate_freshness(now, &state.freshness) {
        return reject_response(Some(&handler), Some(&doc), reason);
    }

    // ─── 6. Attribution gate. A document with neither a transport-
    // authenticated peer nor a `proof` is attributable to nobody: the
    // §4.8.1 fallback hands the handler whatever `issuer` string the sender
    // typed. The binding spec does not permit `proof` to be omitted
    // (`bindings/https/0.2` §5); this is that rule at runtime. See
    // `HttpsServerBuilder::require_attribution`.
    if state.require_attribution && handler.peer().is_none() && doc.proof.is_none() {
        return reject_response(Some(&handler), Some(&doc), RejectReason::ProofRequired);
    }

    // ─── 7. DID-method pre-screen on `proof.verificationMethod`, ahead of
    // the verifier so an unacceptable method never reaches a resolver.
    if let (Some(allowed), Some(proof)) = (&state.allowed_did_methods, &doc.proof) {
        if !did_method_allowed(&proof.verification_method, allowed) {
            return reject_response(
                Some(&handler),
                Some(&doc),
                RejectReason::ProofInvalid {
                    // Does not name the accepted methods: that set is
                    // deployment configuration, and echoing it would let a
                    // probe fingerprint the fleet.
                    reason: "verification method is not acceptable under this consumer's \
                             proof policy"
                        .to_string(),
                },
            );
        }
    }

    // ─── 8. Proof handling (SPEC §7.2 item 7 + §4.7.1). The dispatch
    // pipeline applies one of two policies based on builder
    // configuration:
    //
    //   * Verifier configured (`HttpsServerBuilder::with_verifier`):
    //     mirror `ProofPolicy::Verify` — proof-bearing documents are
    //     verified, failure rejects `proof_invalid`. Proofless
    //     documents proceed; the per-spec `IS_PROOF_REQUIRED` check in
    //     the dispatch closure catches REQUIRED specs.
    //
    //   * No verifier (default): mirror `ProofPolicy::RejectIfPresent`
    //     — proof-bearing documents are rejected `malformed_request`
    //     with the framework-shared `PROOF_NOT_ACCEPTED_BY_POLICY`
    //     wire message. Silently dropping a producer's proof would
    //     mislead them about the exchange's integrity guarantees, and
    //     naming the server's configuration on the wire would let a
    //     probe enumerate verifier coverage across a fleet.
    if doc.proof.is_some() {
        match &state.verifier {
            Some(v) => {
                if let Err(err) = v.verify_json(&doc).await {
                    return reject_response(
                        Some(&handler),
                        Some(&doc),
                        RejectReason::ProofInvalid {
                            reason: err.to_string(),
                        },
                    );
                }
            }
            None => {
                return reject_response(
                    Some(&handler),
                    Some(&doc),
                    RejectReason::MalformedRequest {
                        reason: PROOF_NOT_ACCEPTED_BY_POLICY.to_string(),
                    },
                );
            }
        }
    }

    // ─── 9. SPEC §7.2 item 11 — claim this document `id` for execution.
    //
    // This is the `validated → accepted` transition: every check that could
    // still refuse the document for a reason unrelated to having seen it
    // before has run, and the next thing that happens is the effect. See the
    // module docs for why claiming earlier is wrong.
    let mut claim: Option<DocumentDigest> = None;
    if let Some(guard) = &state.replay_guard {
        // Keyed on `id`, compared on a digest of the whole canonical document
        // (`proof` included) — §7.2's *Keying and comparison*. An `id` alone
        // cannot tell the §8.4 retry it must absorb from the conflict it must
        // reject.
        let digest = match document_digest(&doc) {
            Ok(digest) => digest,
            Err(e) => {
                return reject_response(
                    Some(&handler),
                    Some(&doc),
                    RejectReason::malformed_from_serde(&e),
                );
            }
        };

        // §7.2 (*Bounding the record*): "A *consumer* that can establish
        // neither an `expiresAt` nor an age for a document has no window in
        // which to place it, and MUST NOT execute a *consequential Trust Task*
        // on it." A guard asked to retain a record forever is not a guard.
        let Some(retain_until) = state.freshness.record_expiry(&doc, now) else {
            return reject_response(
                Some(&handler),
                Some(&doc),
                RejectReason::Stale {
                    detail: StaleReason::Unboundable,
                },
            );
        };

        match guard.claim(&doc.id, &digest, Some(retain_until), now).await {
            // First sight. Execute, then record the result at step 10 so the
            // next arrival is answered with it rather than re-executed.
            Ok(ReplayVerdict::Fresh) => claim = Some(digest),

            // Already executed, and the result was retained. §7.2
            // (*Disposition of a duplicate*): "the consumer SHOULD return the
            // previously determined result". Same 200, same body, no second
            // dispatch — which is what makes a §8.4 bit-for-bit retry safe for
            // the producer to send.
            Ok(ReplayVerdict::Duplicate {
                prior_response: Some(prior),
                ..
            }) => return success_response(prior),

            // Already accepted; the original execution has not finished. SPEC:
            // "the consumer SHOULD return or expose the existing execution
            // state rather than begin another." HTTP's word for *accepted,
            // outcome not yet available* is `202 Accepted`, and it is the
            // honest one here: the document was accepted (it is being executed
            // right now, by the first delivery), there is no result to return
            // yet, and nothing failed. 200 would be a lie about having a
            // result, 409 would claim a conflict that does not exist, and any
            // error code would report a failure that did not happen.
            Ok(ReplayVerdict::Duplicate {
                in_flight: true, ..
            }) => return StatusCode::ACCEPTED.into_response(),

            // Already executed, and no result was retained — the
            // fire-and-forget shape (§4.4.1), or a guard that keeps only the
            // claim. §7.2: "where the specification defines no success
            // response, silence is the correct disposition", and "in no case
            // is a duplicate reported as `taskFailed`; the task did not fail,
            // it already happened." `204 No Content` is silence with a status
            // line, which is the most HTTP will allow.
            Ok(ReplayVerdict::Duplicate { .. }) => return StatusCode::NO_CONTENT.into_response(),

            // A different document under a reused `id`. §7.2 item 11 requires
            // `idConflict` and forbids treating it as a retry of the original.
            Ok(ReplayVerdict::Conflict) => {
                return reject_response(Some(&handler), Some(&doc), RejectReason::IdConflict);
            }

            // `ReplayVerdict` is `#[non_exhaustive]`. A verdict this binding
            // does not understand is not a licence to execute; fail closed on
            // the same `unavailable` path as a guard error.
            Ok(_) => {
                return reject_response(
                    Some(&handler),
                    Some(&doc),
                    RejectReason::Unavailable { retry_after: None },
                );
            }

            // Fail closed. A consumer that cannot consult its record has not
            // satisfied item 11, and executing anyway is precisely the double
            // execution the rule forbids. `unavailable` is `retryable`, which
            // is the honest answer: the producer's bit-for-bit resend will be
            // absorbed correctly once the store is back. The store's identity
            // and failure mode stay in the log — `From<ReplayGuardError>`
            // discards them rather than putting them on the wire (§10.4).
            Err(e) => return reject_response(Some(&handler), Some(&doc), e.into()),
        }
    }

    // ─── 10. Dispatch to the registered handler (which runs the per-spec
    // §7.2 policy checks first).
    let ctx = RequestContext {
        authenticated_sender: handler.peer().map(str::to_string),
        local: handler.local().map(str::to_string),
        resolved,
    };
    let dispatched = doc.clone();
    // A panicking handler must not leave the claim standing: the `id` would be
    // burned until `retain_until` and every honest resend absorbed in silence,
    // which is worse than the risk of re-running an effect whose completion
    // nobody can establish anyway. The panic itself is re-raised unchanged, so
    // this changes nothing a caller observes except the state of the record.
    let dispatch_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        (route.dispatch)(dispatched, &ctx)
    }));
    let dispatch_result = match dispatch_result {
        Ok(result) => result,
        Err(panic) => {
            release_claim(&state, &doc.id, claim.as_ref()).await;
            std::panic::resume_unwind(panic);
        }
    };

    match dispatch_result {
        Ok(success_body) => {
            if claim.is_some() {
                if let Some(guard) = &state.replay_guard {
                    // Best-effort: the effect has already happened, so failing
                    // to cache the response cannot un-happen it. The claim —
                    // which is what item 11 needs — is already written, so a
                    // duplicate is still absorbed; all that is lost is the
                    // ability to hand back the same body.
                    // `None` records that the execution completed with no
                    // response to hand back — a later duplicate is absorbed
                    // and answered 204, not re-executed.
                    let _ = guard.record_response(&doc.id, success_body.as_ref()).await;
                }
            }
            match success_body {
                Some(body) => success_response(body),
                // SPEC §4.4.1 fire-and-forget: nothing to return, and the
                // absence of a `trust-task-error` is itself the success
                // signal. 204 is the same answer the binding gives a
                // duplicate of a completed fire-and-forget execution.
                None => StatusCode::NO_CONTENT.into_response(),
            }
        }
        Err(reason) => {
            // Nothing was executed, so nothing may be remembered as executed.
            // Holding the claim would answer a corrected — or merely
            // better-timed — resend under this `id` with silence or
            // `idConflict` for the whole retention window, which is a denial
            // of service manufactured out of a refusal.
            release_claim(&state, &doc.id, claim.as_ref()).await;
            reject_response(Some(&handler), Some(&doc), reason)
        }
    }
}

/// Drop a claim whose execution produced no recorded response. Best-effort: a
/// guard that cannot release leaves the record standing until `retain_until`,
/// which is safe (item 11 still holds) but unkind to a legitimate retry.
async fn release_claim(state: &ServerState, id: &str, digest: Option<&DocumentDigest>) {
    if let (Some(guard), Some(digest)) = (&state.replay_guard, digest) {
        let _ = guard.release(id, digest).await;
    }
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

/// `true` when `verification_method` is a DID URL whose method is in
/// `allowed`. Anything that is not a DID URL (`did:<method>:<rest>`) fails —
/// an `https://…` verification method would hand the resolver an arbitrary
/// URL directly.
fn did_method_allowed(verification_method: &str, allowed: &BTreeSet<String>) -> bool {
    let Some(rest) = verification_method.strip_prefix("did:") else {
        return false;
    };
    let Some((method, remainder)) = rest.split_once(':') else {
        return false;
    };
    if method.is_empty() || remainder.is_empty() {
        return false;
    }
    allowed.contains(method)
}

fn extract_bearer(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let token = value
        .strip_prefix("Bearer ")
        .or_else(|| value.strip_prefix("bearer "))?;
    Some(token.trim())
}

fn downcast<P: Payload>(doc: TrustTask<Value>) -> Result<TrustTask<P>, RejectReason> {
    let TrustTask {
        id,
        thread_id,
        parent_thread_id,
        ceremony,
        type_uri,
        issuer,
        recipient,
        issued_at,
        expires_at,
        payload,
        context,
        proof,
        extra,
    } = doc;
    // Same §10.4 rule as the body parse above: the deserializer's rendering
    // names the missing or unexpected member and its offset. The Type URI the
    // sender itself chose is safe to omit, and the category is what it needs.
    let payload: P =
        serde_json::from_value(payload).map_err(|e| RejectReason::malformed_from_serde(&e))?;
    Ok(TrustTask {
        id,
        thread_id,
        parent_thread_id,
        ceremony,
        type_uri,
        issuer,
        recipient,
        issued_at,
        expires_at,
        payload,
        context,
        proof,
        extra,
    })
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

fn reject_response(
    handler: Option<&HttpsHandler>,
    request: Option<&TrustTask<Value>>,
    reason: RejectReason,
) -> Response {
    // Status follows the error document actually emitted, not the inbound
    // reason. The suppressed identity-mismatch path rewrites the body to a
    // generic `malformedRequest` (SPEC §10.4); the status MUST match so it is
    // indistinguishable from a plain parse failure (no 403-vs-400 oracle for an
    // unauthenticated prober).
    let error_doc = build_error_response(handler, request, reason);
    let status = status_for_code(&error_doc.payload.code);
    let status = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let body = serde_json::to_vec(&error_doc).expect("serialise error response");
    (status, [(header::CONTENT_TYPE, "application/json")], body).into_response()
}

fn build_error_response(
    handler: Option<&HttpsHandler>,
    request: Option<&TrustTask<Value>>,
    reason: RejectReason,
) -> ErrorResponse {
    let new_id = format!("urn:uuid:{}", Uuid::new_v4());
    match (handler, request) {
        (Some(h), Some(req)) => {
            // Apply §8.1 routing — identity_mismatch goes to the
            // transport-authenticated peer (or is suppressed; we still
            // synthesise the document for the HTTP body, but route the
            // outbound recipient correctly).
            match h.reject(req, new_id.clone(), reason.clone()) {
                Some(resp) => resp,
                None => suppressed_error_response(&new_id),
            }
        }
        (_, Some(req)) => req.reject_with(new_id, reason),
        _ => {
            // No request context — synthesise a free-standing error doc.
            // This branch is hit when the body failed to deserialise into
            // a TrustTask at all.
            // `trust_task_error_type_uri()` rather than a version spelled out
            // here: this crate emitted `trust-task-error/0.2` on these two
            // fallback paths while `trust-tasks-rs` and the TypeScript runtime
            // both emitted `0.5`, so the version a producer saw depended on
            // which branch of this function it hit. One definition, one answer.
            let mut doc = TrustTask::new(
                new_id,
                trust_tasks_rs::trust_task_error_type_uri(),
                ErrorPayload::from(reason),
            );
            doc.issued_at = Some(chrono::Utc::now());
            doc
        }
    }
}

/// When `TransportHandler::reject` returns `None` (no transport-
/// authenticated sender under identity_mismatch), the framework rule
/// (SPEC §8.1) is that the consumer SHOULD NOT emit a response. HTTP
/// gives us no way to "not emit" — the TCP connection already exists and
/// the peer is waiting for a status line. We therefore emit the *same*
/// generic `malformedRequest`/400 that a body parse failure produces.
///
/// Crucially we MUST NOT echo the `identityMismatch` code or status here
/// (SPEC §10.4): an unauthenticated prober who POSTs a spoofed in-band
/// identity would otherwise learn, from the code + 4xx, that this consumer
/// performs the cross-check and that the identity was contested — an
/// identity-probing oracle. Collapsing to the indistinguishable generic
/// rejection removes that signal; the diagnostic loss for honest peers is
/// the deliberate, safer trade.
fn suppressed_error_response(new_id: &str) -> ErrorResponse {
    let mut doc = TrustTask::new(
        new_id.to_string(),
        trust_tasks_rs::trust_task_error_type_uri(),
        ErrorPayload::from(RejectReason::MalformedRequest {
            reason: String::new(),
        }),
    );
    doc.issued_at = Some(chrono::Utc::now());
    doc
}

/// Make sure StandardCode → TrustTaskCode conversion is available where
/// we map status codes.
fn _verify_standard_code_into() {
    let _: trust_tasks_rs::TrustTaskCode = StandardCode::Expired.into();
}
