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
//! 5. `resolve_parties` (§4.8.1) and `validate_basic` (expiry, recipient).
//! 6. **Attribution gate** — see [`HttpsServerBuilder::require_attribution`].
//! 7. `proof.verificationMethod` DID-method pre-screen — see
//!    [`HttpsServerBuilder::allowed_did_methods`].
//! 8. Proof verification (the only step that may egress).
//! 9. Per-spec policy + the registered handler.

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
    discovery::DiscoveryRegistry, erase_verifier, specs::trust_task_discovery::v0_1 as discovery,
    DynProofVerifier, ErrorPayload, ErrorResponse, Payload, ProofVerifier, RejectReason,
    ResolvedParties, StandardCode, TransportHandler, TrustTask, PROOF_NOT_ACCEPTED_BY_POLICY,
};
use uuid::Uuid;

use crate::auth::{Auth, BearerAuth};
use crate::handler::HttpsHandler;
use crate::status::status_for_code;

/// Maximum accepted request-body size (SPEC §10.2). Trust Task payloads are
/// small; 256 KiB is generous headroom while bounding pre-auth memory use.
/// Callers needing a different bound can rebuild the router with their own
/// [`DefaultBodyLimit`] layer.
const MAX_BODY_BYTES: usize = 256 * 1024;

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
/// context. Output: either the JSON body of a success response (a
/// fully-formed `#response` document), or a [`RejectReason`] the server
/// will convert into a `trust-task-error` response via §8.1 routing.
type DispatchFn =
    Box<dyn Fn(TrustTask<Value>, &RequestContext) -> Result<Value, RejectReason> + Send + Sync>;

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
}

/// Builder for [`HttpsServer`].
pub struct HttpsServerBuilder {
    local_vid: Option<String>,
    auth: Option<Box<dyn Auth>>,
    routes: HashMap<String, Route>,
    verifier: Option<Arc<dyn DynProofVerifier>>,
    require_attribution: bool,
    allowed_did_methods: Option<BTreeSet<String>>,
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
    pub fn on<P, Resp, F>(mut self, handler: F) -> Self
    where
        P: Payload + 'static,
        Resp: Payload + Serialize + 'static,
        F: Fn(&TrustTask<P>, &RequestContext) -> Result<Resp, RejectReason> + Send + Sync + 'static,
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
            Ok(serde_json::to_value(&response_doc).expect("response serialises (typed structs)"))
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
        self.on::<discovery::Payload, discovery::Response, _>(move |req, ctx| {
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
    ///     .on::<grant::Payload, grant::Response, _>(handle_grant)
    ///     .on::<revoke::Payload, revoke::Response, _>(handle_revoke)
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
            public_discovery: Arc::new(AtomicBool::new(false)),
            request_timeout: None,
            max_concurrent_requests: None,
        }
    }

    /// Build the axum [`Router`] without starting a listener — useful for
    /// integration tests that want to spawn the app inline.
    ///
    /// The router applies an explicit `DefaultBodyLimit` of
    /// `MAX_BODY_BYTES` (256 KiB) as an audited DoS control (SPEC §10.2): the body is
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
        Err(e) => {
            return reject_response(
                None,
                None,
                RejectReason::MalformedRequest {
                    reason: format!("body did not parse as a Trust Task document: {e}"),
                },
            );
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

    // ─── 9. Dispatch to the registered handler (which runs the per-spec
    // §7.2 policy checks first).
    let ctx = RequestContext {
        authenticated_sender: handler.peer().map(str::to_string),
        local: handler.local().map(str::to_string),
        resolved,
    };
    let dispatch_result = (route.dispatch)(doc.clone(), &ctx);

    match dispatch_result {
        Ok(success_body) => success_response(success_body),
        Err(reason) => reject_response(Some(&handler), Some(&doc), reason),
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
    let payload: P =
        serde_json::from_value(payload).map_err(|e| RejectReason::MalformedRequest {
            reason: format!("payload does not match {}: {e}", P::TYPE_URI),
        })?;
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
            let mut doc = TrustTask::new(
                new_id,
                trust_tasks_rs::TypeUri::canonical("trust-task-error", 0, 2)
                    .expect("framework type URI"),
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
        trust_tasks_rs::TypeUri::canonical("trust-task-error", 0, 2).expect("framework type URI"),
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
