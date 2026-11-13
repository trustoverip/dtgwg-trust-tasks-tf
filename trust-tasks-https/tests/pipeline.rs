//! The HTTPS server on the shared §7.2 pipeline (#1089, #1090, #1117, #1121).
//!
//! These tests use hand-written payload types under `example.test`, so they
//! can declare what no published specification does — an `extends` chain, a
//! `maxDocumentBytes` — without leaning on a prebuilt module:
//!
//! * async handlers and async authentication;
//! * a synchronous handler on a multi-threaded runtime;
//! * routing: `unsupportedType` against `unsupportedVersion`, and a later
//!   version declaring `extends` routed to the implemented one (SPEC §5.2);
//! * the per-type size bound and the body limit derived from it (SPEC §12.4);
//! * the payload-schema policy (SPEC §7.2 item 2).

use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use trust_tasks_https::{
    AsyncAuth, AuthFuture, BearerAuth, HttpsServer, HttpsServerBuilder, MAX_BODY_BYTES,
};
use trust_tasks_rs::{Payload, PayloadValidator, RejectReason, RequestPayload};

const SERVER_VID: &str = "did:web:server.example";
const ALICE: &str = "did:web:alice.example";
const ECHO: &str = "https://example.test/spec/demo/echo";

// ─── Hand-written specifications ──────────────────────────────────────────

/// `demo/echo/0.1`: one `message`, refusing members it does not know, as the
/// generated types do. Declares no size bound.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Echo {
    message: String,
}

impl Payload for Echo {
    const TYPE_URI: &'static str = "https://example.test/spec/demo/echo/0.1";
    const PAYLOAD_SCHEMA: Option<&'static str> = Some(
        r#"{"type":"object","additionalProperties":false,"required":["message"],"properties":{"message":{"type":"string"}}}"#,
    );
}

#[derive(Debug, Serialize, Deserialize)]
struct EchoResponse {
    message: String,
}

impl Payload for EchoResponse {
    const TYPE_URI: &'static str = "https://example.test/spec/demo/echo/0.1#response";
}

impl RequestPayload for Echo {
    type Response = EchoResponse;
}

/// `demo/notify/0.1`: fire-and-forget (no response).
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Notify {
    message: String,
}

impl Payload for Notify {
    const TYPE_URI: &'static str = "https://example.test/spec/demo/notify/0.1";
}

/// `demo/upload/0.1`: declares `maxDocumentBytes` of 1 MiB.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Upload {
    message: String,
}

const UPLOAD_BOUND: usize = 1024 * 1024;

impl Payload for Upload {
    const TYPE_URI: &'static str = "https://example.test/spec/demo/upload/0.1";
    const MAX_DOCUMENT_BYTES: Option<usize> = Some(UPLOAD_BOUND);
}

#[derive(Debug, Serialize, Deserialize)]
struct UploadResponse {}

impl Payload for UploadResponse {
    const TYPE_URI: &'static str = "https://example.test/spec/demo/upload/0.1#response";
}

impl RequestPayload for Upload {
    type Response = UploadResponse;
}

/// `demo/ping/0.1`: declares `maxDocumentBytes` of 1 KiB — below the
/// transport default, so the declaration is what refuses.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Ping {
    message: String,
}

impl Payload for Ping {
    const TYPE_URI: &'static str = "https://example.test/spec/demo/ping/0.1";
    const MAX_DOCUMENT_BYTES: Option<usize> = Some(1024);
}

#[derive(Debug, Serialize, Deserialize)]
struct PingResponse {}

impl Payload for PingResponse {
    const TYPE_URI: &'static str = "https://example.test/spec/demo/ping/0.1#response";
}

impl RequestPayload for Ping {
    type Response = PingResponse;
}

// ─── Fixtures ─────────────────────────────────────────────────────────────

fn alice() -> BearerAuth {
    BearerAuth::from_pairs([("alice", ALICE)])
}

fn base() -> HttpsServerBuilder {
    HttpsServer::builder()
        .local_vid(SERVER_VID)
        .with_auth(alice())
}

async fn spawn(server: HttpsServer) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = server.into_router();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

fn document(type_uri: &str, id: &str, payload: Value) -> Value {
    json!({
        "id": id,
        "type": type_uri,
        "issuer": ALICE,
        "recipient": SERVER_VID,
        "issuedAt": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "payload": payload,
    })
}

fn echo(id: &str, message: &str) -> Value {
    document(Echo::TYPE_URI, id, json!({ "message": message }))
}

/// POST `body` with `token` as the bearer; the status and the parsed body
/// (`Value::Null` when there is none).
async fn post(addr: SocketAddr, token: Option<&str>, body: &Value) -> (u16, Value) {
    let mut request = reqwest::Client::new()
        .post(format!("http://{addr}/trust-tasks"))
        .header("content-type", "application/json")
        .body(body.to_string());
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    let response = request.send().await.unwrap();
    let status = response.status().as_u16();
    let bytes = response.bytes().await.unwrap();
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or(Value::Null)
    };
    (status, body)
}

fn code(body: &Value) -> &str {
    body["payload"]["code"].as_str().unwrap_or("")
}

// ─── #1090: async handlers and async authentication ───────────────────────

/// An async handler awaits — the database round trip a synchronous `Fn`
/// could only block a worker for — and receives the request context.
#[tokio::test]
async fn an_async_handler_awaits_and_receives_the_context() {
    let seen = Arc::new(Mutex::new(None));
    let record = Arc::clone(&seen);
    let server = base()
        .on_async::<Echo, _, _>(move |req, ctx| {
            let record = Arc::clone(&record);
            async move {
                tokio::time::sleep(Duration::from_millis(5)).await;
                *record.lock().unwrap() = ctx.authenticated_sender.clone();
                Ok(EchoResponse {
                    message: req.payload.message,
                })
            }
        })
        .build();
    let addr = spawn(server).await;

    let (status, body) = post(addr, Some("alice"), &echo("urn:uuid:async-1", "hi")).await;

    assert_eq!(status, 200, "{body}");
    assert_eq!(body["type"], json!(format!("{}#response", Echo::TYPE_URI)));
    assert_eq!(body["payload"]["message"], json!("hi"));
    assert_eq!(body["threadId"], json!("urn:uuid:async-1"));
    assert_eq!(seen.lock().unwrap().as_deref(), Some(ALICE));
}

/// An async handler's refusal is routed per §8.1 like a synchronous one's.
#[tokio::test]
async fn an_async_handler_refusal_is_an_error_document() {
    let server = base()
        .on_async::<Echo, _, _>(|_req, _ctx| async {
            Err(RejectReason::PermissionDenied {
                reason: "not you".into(),
            })
        })
        .build();
    let addr = spawn(server).await;

    let (status, body) = post(addr, Some("alice"), &echo("urn:uuid:async-2", "hi")).await;

    assert_eq!(status, 403);
    assert_eq!(code(&body), "permissionDenied");
    assert_eq!(body["recipient"], json!(ALICE));
    assert_eq!(
        body["payload"]["inResponseTo"]["id"],
        json!("urn:uuid:async-2")
    );
}

/// A fire-and-forget async handler acknowledges with 204, and a duplicate of
/// the acknowledged document is absorbed in silence — the record holds the
/// completion, not a `null` response.
#[tokio::test]
async fn an_async_acknowledgement_is_204_and_its_duplicate_is_silence() {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let server = base()
        .on_ack_async::<Notify, _, _>(move |_req, _ctx| {
            let counter = Arc::clone(&counter);
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        })
        .build();
    let addr = spawn(server).await;
    let doc = document(
        Notify::TYPE_URI,
        "urn:uuid:notify-1",
        json!({ "message": "hi" }),
    );

    let (first, body) = post(addr, Some("alice"), &doc).await;
    let (second, second_body) = post(addr, Some("alice"), &doc).await;

    assert_eq!(first, 204);
    assert_eq!(body, Value::Null);
    assert_eq!(second, 204, "a duplicate of an acknowledgement is silence");
    assert_eq!(second_body, Value::Null);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

/// An authenticator that must await — a session store — implements
/// `AsyncAuth` and authenticates the sender, which is what lets a proofless
/// document past the attribution gate.
struct SessionAuth;

impl AsyncAuth for SessionAuth {
    fn resolve_async<'a>(&'a self, token: &'a str) -> AuthFuture<'a> {
        Box::pin(async move {
            tokio::time::sleep(Duration::from_millis(5)).await;
            (token == "session-alice").then(|| ALICE.to_string())
        })
    }
}

#[tokio::test]
async fn an_async_authenticator_authenticates_the_sender() {
    let server = HttpsServer::builder()
        .local_vid(SERVER_VID)
        .with_auth(SessionAuth)
        .on_async::<Echo, _, _>(|req, ctx| async move {
            Ok(EchoResponse {
                message: format!(
                    "{} from {}",
                    req.payload.message,
                    ctx.authenticated_sender.unwrap_or_default()
                ),
            })
        })
        .build();
    let addr = spawn(server).await;

    let (status, body) = post(
        addr,
        Some("session-alice"),
        &echo("urn:uuid:session-1", "hi"),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        body["payload"]["message"],
        json!(format!("hi from {ALICE}"))
    );

    // An unknown token is no sender: a proofless document is unattributable.
    let (status, body) = post(addr, Some("forged"), &echo("urn:uuid:session-2", "hi")).await;
    assert_eq!(status, 422);
    assert_eq!(code(&body), "proofRequired");
}

/// A synchronous handler that blocks runs on a multi-threaded runtime (under
/// `block_in_place`) and the server keeps answering other requests meanwhile.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_blocking_sync_handler_runs_on_a_multi_thread_runtime() {
    let server = base()
        .on::<Echo, _>(|req, _ctx| {
            if req.payload.message == "slow" {
                std::thread::sleep(Duration::from_millis(300));
            }
            Ok(EchoResponse {
                message: req.payload.message.clone(),
            })
        })
        .build();
    let addr = spawn(server).await;

    let slow =
        tokio::spawn(
            async move { post(addr, Some("alice"), &echo("urn:uuid:slow", "slow")).await },
        );
    tokio::time::sleep(Duration::from_millis(50)).await;
    let started = std::time::Instant::now();
    let (status, _) = post(addr, Some("alice"), &echo("urn:uuid:fast", "fast")).await;
    assert_eq!(status, 200);
    assert!(
        started.elapsed() < Duration::from_millis(250),
        "a blocking handler must not stall the other requests"
    );
    assert_eq!(slow.await.unwrap().0, 200);
}

// ─── #1117: routing along `extends`, and `unsupportedVersion` ─────────────

/// `demo/echo/0.2` declares that it extends `0.1`.
static ECHO_EXTENDS: &[(&str, &str)] = &[("https://example.test/spec/demo/echo/0.2", "0.1")];

#[tokio::test]
async fn a_later_version_declaring_extends_reaches_the_implemented_handler() {
    trust_tasks_rs::extension::declare_extends(ECHO_EXTENDS);
    let seen = Arc::new(Mutex::new(None));
    let record = Arc::clone(&seen);
    let server = base()
        .on_async::<Echo, _, _>(move |req, _ctx| {
            let record = Arc::clone(&record);
            async move {
                *record.lock().unwrap() = Some(req.type_uri.to_string());
                Ok(EchoResponse {
                    message: req.payload.message,
                })
            }
        })
        .build();
    let addr = spawn(server).await;

    // A member 0.2 added, which 0.1's type refuses unless it is pruned.
    let doc = document(
        &format!("{ECHO}/0.2"),
        "urn:uuid:extends-1",
        json!({ "message": "hi", "addedIn0_2": true }),
    );
    let (status, body) = post(addr, Some("alice"), &doc).await;

    assert_eq!(status, 200, "{body}");
    assert_eq!(
        body["type"],
        json!(format!("{ECHO}/0.2#response")),
        "answered in the version received"
    );
    assert_eq!(body["payload"]["message"], json!("hi"));
    assert_eq!(
        seen.lock().unwrap().as_deref(),
        Some(format!("{ECHO}/0.2").as_str())
    );
}

#[tokio::test]
async fn an_unregistered_version_is_unsupported_version_and_an_unknown_slug_unsupported_type() {
    let server = base()
        .on_async::<Echo, _, _>(|req, _ctx| async move {
            Ok(EchoResponse {
                message: req.payload.message,
            })
        })
        .build();
    let addr = spawn(server).await;

    let version = document(
        &format!("{ECHO}/0.7"),
        "urn:uuid:version-1",
        json!({ "message": "hi" }),
    );
    let (status, body) = post(addr, Some("alice"), &version).await;
    assert_eq!(status, 422);
    assert_eq!(code(&body), "unsupportedVersion");

    let other = document(
        "https://example.test/spec/demo/never/0.1",
        "urn:uuid:version-2",
        json!({ "message": "hi" }),
    );
    let (status, body) = post(addr, Some("alice"), &other).await;
    assert_eq!(status, 422);
    assert_eq!(code(&body), "unsupportedType");
}

// ─── #1121: the per-type size bound and the body limit ────────────────────

/// A document of `bytes` total size, of `type_uri`, padded in `message`.
fn sized(type_uri: &str, id: &str, bytes: usize) -> Value {
    let mut doc = document(type_uri, id, json!({ "message": "" }));
    let overhead = doc.to_string().len();
    doc["payload"]["message"] = json!("x".repeat(bytes.saturating_sub(overhead)));
    assert_eq!(doc.to_string().len(), bytes.max(overhead));
    doc
}

fn size_server() -> HttpsServerBuilder {
    base()
        .on::<Echo, _>(|req, _| {
            Ok(EchoResponse {
                message: req.payload.message.len().to_string(),
            })
        })
        .on::<Ping, _>(|_, _| Ok(PingResponse {}))
}

#[tokio::test]
async fn the_body_limit_is_the_transport_default_until_a_task_declares_more() {
    assert_eq!(size_server().build().body_limit(), MAX_BODY_BYTES);
    let server = size_server()
        .on::<Upload, _>(|_, _| Ok(UploadResponse {}))
        .build();
    assert_eq!(server.body_limit(), UPLOAD_BOUND);
    assert_eq!(
        size_server().max_body_bytes(64 * 1024).build().body_limit(),
        64 * 1024,
        "an explicit limit replaces the derived one"
    );
}

#[tokio::test]
async fn a_document_within_its_declared_bound_is_accepted_above_the_transport_default() {
    let addr = spawn(
        size_server()
            .on::<Upload, _>(|_, _| Ok(UploadResponse {}))
            .build(),
    )
    .await;
    let doc = sized(Upload::TYPE_URI, "urn:uuid:upload-1", 600 * 1024);
    let (status, body) = post(addr, Some("alice"), &doc).await;
    assert_eq!(status, 200, "{body}");
}

#[tokio::test]
async fn a_type_declaring_no_bound_keeps_the_transport_default_under_a_larger_limit() {
    let addr = spawn(
        size_server()
            .on::<Upload, _>(|_, _| Ok(UploadResponse {}))
            .build(),
    )
    .await;
    // Under the router's 1 MiB limit, over the 256 KiB that applies to a type
    // that declares nothing.
    let doc = sized(Echo::TYPE_URI, "urn:uuid:echo-big", 300 * 1024);
    let (status, body) = post(addr, Some("alice"), &doc).await;
    assert_eq!(status, 400);
    assert_eq!(code(&body), "malformedRequest");

    let doc = sized(Echo::TYPE_URI, "urn:uuid:echo-ok", 200 * 1024);
    assert_eq!(post(addr, Some("alice"), &doc).await.0, 200);
}

#[tokio::test]
async fn a_document_over_its_declared_bound_is_refused_before_the_handler() {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let addr = spawn(
        base()
            .on::<Ping, _>(move |_, _| {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok(PingResponse {})
            })
            .build(),
    )
    .await;

    let (status, body) = post(
        addr,
        Some("alice"),
        &sized(Ping::TYPE_URI, "urn:uuid:ping-big", 2048),
    )
    .await;
    assert_eq!(status, 400);
    assert_eq!(code(&body), "malformedRequest");
    assert_eq!(calls.load(Ordering::SeqCst), 0);

    let (status, _) = post(
        addr,
        Some("alice"),
        &sized(Ping::TYPE_URI, "urn:uuid:ping-ok", 900),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn a_body_over_the_router_limit_is_413() {
    let addr = spawn(size_server().build()).await;
    let doc = sized(Echo::TYPE_URI, "urn:uuid:too-big", MAX_BODY_BYTES + 1024);
    assert_eq!(post(addr, Some("alice"), &doc).await.0, 413);
}

// ─── #1089: the payload-schema policy is available on the server ──────────

struct RefuseEverything;

impl PayloadValidator for RefuseEverything {
    fn validate(&self, _schema: &str, _payload: &Value) -> Result<(), String> {
        Err("refused by the test validator".into())
    }
}

#[tokio::test]
async fn a_payload_validator_refuses_before_the_handler() {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let addr = spawn(
        base()
            .with_payload_validator(RefuseEverything)
            .on::<Echo, _>(move |req, _| {
                counter.fetch_add(1, Ordering::SeqCst);
                Ok(EchoResponse {
                    message: req.payload.message.clone(),
                })
            })
            .build(),
    )
    .await;

    let (status, body) = post(addr, Some("alice"), &echo("urn:uuid:validated", "hi")).await;
    assert_eq!(status, 400);
    assert_eq!(code(&body), "malformedRequest");
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

// ─── Cancellation after the effect committed ──────────────────────────────

/// An async handler that commits its effect and is then cut off at a later
/// `await` by the request timeout must not run again on a resend: the claim
/// is left in flight (the effect may have happened), so the resend is
/// answered `202 Accepted`, not executed.
#[tokio::test]
async fn an_async_handler_cancelled_after_committing_is_not_executed_again() {
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let server = base()
        .request_timeout(Duration::from_millis(100))
        .on_async::<Echo, _, _>(move |req, _ctx| {
            let counter = Arc::clone(&counter);
            async move {
                // The effect commits...
                counter.fetch_add(1, Ordering::SeqCst);
                // ...then the handler awaits past the request's budget.
                tokio::time::sleep(Duration::from_secs(5)).await;
                Ok(EchoResponse {
                    message: req.payload.message,
                })
            }
        })
        .build();
    let addr = spawn(server).await;
    let doc = echo("urn:uuid:cancelled-1", "hi");

    let (status, _) = post(addr, Some("alice"), &doc).await;
    assert_eq!(status, 408, "the request outlived its budget");
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    let (status, body) = post(addr, Some("alice"), &doc).await;
    assert_eq!(status, 202, "in flight, not re-executed: {body}");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "the committed effect ran a second time"
    );
}
