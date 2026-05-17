//! Axum-based server: receives Trust Task documents over HTTP, dispatches
//! them to per-spec handlers, and returns a typed response document or a
//! `trust-task-error/0.1` document with the appropriate HTTP status code.
//!
//! Single endpoint: `POST /trust-tasks` accepts any document the server has
//! a handler registered for. Bearer auth (`Authorization: Bearer <token>`)
//! is consulted before framework validation; the resulting VID becomes the
//! transport-authenticated sender for §4.8.1 precedence.

use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Router;
use serde::Serialize;
use serde_json::Value;
use trust_tasks_rs::{
    discovery::DiscoveryRegistry, specs::trust_task_discovery::v0_1 as discovery, ErrorPayload,
    ErrorResponse, Payload, RejectReason, StandardCode, TransportHandler, TrustTask,
};
use uuid::Uuid;

use crate::auth::{Auth, BearerAuth};
use crate::handler::HttpsHandler;
use crate::status::status_for_code;

/// Context handed to every spec handler — the transport-authenticated
/// peer (when present) and convenience accessors for the inbound
/// document's metadata.
#[derive(Debug, Clone)]
pub struct RequestContext {
    /// VID of the transport-authenticated sender, if any.
    pub authenticated_sender: Option<String>,
    /// VID of the local party serving this request.
    pub local: Option<String>,
}

/// A spec-specific handler stored type-erased in the dispatch table.
///
/// Inputs: the parsed (but untyped) inbound document and the request
/// context. Output: either the JSON body of a success response (a
/// fully-formed `#response` document), or a [`RejectReason`] the server
/// will convert into a `trust-task-error/0.1` response via §8.1 routing.
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
}

/// Builder for [`HttpsServer`].
pub struct HttpsServerBuilder {
    local_vid: Option<String>,
    auth: Option<Box<dyn Auth>>,
    routes: HashMap<String, Route>,
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

    /// Register a handler for the request payload type `P`. The handler
    /// receives the typed request and a [`RequestContext`]; it returns
    /// either the response payload (which the server wraps in a
    /// `#response`-variant document via [`TrustTask::respond_with`]) or a
    /// [`RejectReason`] (which the server wraps in a `trust-task-error/0.1`
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

            // SPEC §7.2 item 8 — audience binding. This is the first point
            // where we have a typed payload (and therefore P::IS_BEARER).
            typed.enforce_audience_binding()?;

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

    /// Register a `trust-task-discovery/0.1` handler that responds with
    /// the contents of `registry`. Combine with [`Self::on`] in any order;
    /// the registry is consulted afresh on every inbound query.
    ///
    /// Use this when the server's discoverable set differs from its
    /// actually-handled set — for example, when the server delegates
    /// some types downstream but wants to advertise them as supported.
    /// For the common "advertise exactly what I handle" case, see
    /// [`Self::enable_discovery`].
    pub fn with_discovery(self, registry: DiscoveryRegistry) -> Self {
        self.on::<discovery::Payload, discovery::Response, _>(move |req, _ctx| {
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
    pub fn build(self) -> HttpsServer {
        let auth = self.auth.unwrap_or_else(|| Box::new(BearerAuth::new()));
        HttpsServer {
            state: Arc::new(ServerState {
                local_vid: self.local_vid,
                auth,
                routes: self.routes,
            }),
        }
    }
}

/// An HTTPS Trust Tasks server.
///
/// Build with [`HttpsServer::builder`], register handlers via
/// [`HttpsServerBuilder::on`], then [`Self::serve`].
pub struct HttpsServer {
    state: Arc<ServerState>,
}

impl HttpsServer {
    /// Start a new builder.
    pub fn builder() -> HttpsServerBuilder {
        HttpsServerBuilder {
            local_vid: None,
            auth: None,
            routes: HashMap::new(),
        }
    }

    /// Build the axum [`Router`] without starting a listener — useful for
    /// integration tests that want to spawn the app inline.
    pub fn into_router(self) -> Router {
        Router::new()
            .route("/trust-tasks", post(dispatch_handler))
            .with_state(self.state)
    }

    /// Bind to `addr` and serve incoming requests. Returns when the
    /// listener stops.
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

    // ─── 2. Authenticate the bearer token (if any) into a peer VID.
    let peer_vid = extract_bearer(&headers).and_then(|tok| state.auth.resolve(tok));

    // ─── 3. Build the per-request HttpsHandler and resolve parties.
    let handler = HttpsHandler::new(state.local_vid.clone(), peer_vid);
    let _resolved = match handler.resolve_parties(&doc) {
        Ok(r) => r,
        Err(consistency) => {
            let reason: RejectReason = consistency.into();
            return reject_response(Some(&handler), Some(&doc), reason);
        }
    };

    // ─── 4. Framework-level checks: expiry + recipient identity.
    let now = chrono::Utc::now();
    let my_vid = state.local_vid.as_deref().unwrap_or("");
    if let Err(reason) = doc.validate_basic(now, my_vid) {
        return reject_response(Some(&handler), Some(&doc), reason);
    }

    // ─── 5. Routing: look up the handler registered for this Type URI.
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

    // ─── 6. Dispatch to the registered handler.
    let ctx = RequestContext {
        authenticated_sender: handler.peer().map(str::to_string),
        local: handler.local().map(str::to_string),
    };
    let dispatch_result = (route.dispatch)(doc.clone(), &ctx);

    match dispatch_result {
        Ok(success_body) => success_response(success_body),
        Err(reason) => reject_response(Some(&handler), Some(&doc), reason),
    }
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
    let status = status_for_code(&reason.code().into());
    let status = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    let error_doc = build_error_response(handler, request, reason);
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
                None => suppressed_error_response(&new_id, reason),
            }
        }
        (_, Some(req)) => req.reject_with(new_id, reason),
        _ => {
            // No request context — synthesise a free-standing error doc.
            // This branch is hit when the body failed to deserialise into
            // a TrustTask at all.
            let mut doc = TrustTask::new(
                new_id,
                trust_tasks_rs::TypeUri::canonical("trust-task-error", 0, 1)
                    .expect("framework type URI"),
                ErrorPayload::from(reason),
            );
            doc.issued_at = Some(chrono::Utc::now());
            doc
        }
    }
}

/// When `TransportHandler::reject` returns `None` (no transport-
/// authenticated sender under identity_mismatch), we still need to
/// produce *something* for the HTTP body — the alternative is dropping
/// the TCP connection, which is worse for diagnostics. We synthesise a
/// recipient-less error document; the HTTP transport's authenticated
/// sender will see only the headers (status code + content type) and
/// can choose to ignore the body.
fn suppressed_error_response(new_id: &str, reason: RejectReason) -> ErrorResponse {
    let mut doc = TrustTask::new(
        new_id.to_string(),
        trust_tasks_rs::TypeUri::canonical("trust-task-error", 0, 1).expect("framework type URI"),
        ErrorPayload::from(reason).with_message(
            "identity_mismatch with no transport-authenticated sender — \
             response not addressed (SPEC §8.1)",
        ),
    );
    doc.issued_at = Some(chrono::Utc::now());
    doc
}

/// Make sure StandardCode → TrustTaskCode conversion is available where
/// we map status codes.
fn _verify_standard_code_into() {
    let _: trust_tasks_rs::TrustTaskCode = StandardCode::Expired.into();
}
