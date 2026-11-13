//! A timing harness for the server's inbound path (#1092): from the request
//! bytes to the response, through the whole §7.2 pipeline
//! (`consume_dispatched_answer`) with the default replay record and freshness
//! bound, a bearer-authenticated sender and a trivial handler.
//!
//! No network: each request is driven through the router as a `tower`
//! service, so the figure is the framework's own cost per document.
//!
//! ```sh
//! cargo run --release -p trust-tasks-https --example inbound_bench
//! ```
//!
//! Each scenario runs a warm-up, then several timed rounds of fresh documents
//! (every `id` distinct, so each is a first delivery that is claimed and
//! recorded), and prints the median round's microseconds per request. An
//! argument runs only the scenarios whose name contains it.

use std::time::Instant;

use axum::body::Body;
use axum::http::Request;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tower::Service;
use trust_tasks_https::{BearerAuth, HttpsServer};
use trust_tasks_rs::{InMemoryReplayGuard, Payload, RequestPayload};

const SERVER_VID: &str = "did:web:server.example";
const ALICE: &str = "did:web:alice.example";

/// `bench/record/0.1`: a list of entries, so a document's size is a choice.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    entries: Vec<Entry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    subject: String,
    role: String,
    note: String,
}

impl Payload for Record {
    const TYPE_URI: &'static str = "https://example.test/spec/bench/record/0.1";
    const PAYLOAD_SCHEMA: Option<&'static str> = Some(
        r#"{"type":"object","additionalProperties":false,"required":["entries"],"properties":{"entries":{"type":"array","items":{"type":"object","additionalProperties":false,"properties":{"subject":{"type":"string"},"role":{"type":"string"},"note":{"type":"string"}}}}}}"#,
    );
}

#[derive(Debug, Serialize, Deserialize)]
struct RecordResponse {
    count: usize,
}

impl Payload for RecordResponse {
    const TYPE_URI: &'static str = "https://example.test/spec/bench/record/0.1#response";
}

impl RequestPayload for Record {
    type Response = RecordResponse;
}

/// `bench/record/0.2` declares that it extends `0.1`.
static EXTENDS: &[(&str, &str)] = &[("https://example.test/spec/bench/record/0.2", "0.1")];

fn body(type_uri: &str, id: usize, entries: usize, extra: bool) -> Vec<u8> {
    let entries: Vec<Value> = (0..entries)
        .map(|i| {
            json!({
                "subject": format!("did:web:subject-{i}.example"),
                "role": "maintainer",
                "note": "granted for the quarterly review of the trust registry",
            })
        })
        .collect();
    let mut payload = json!({ "entries": entries });
    if extra {
        payload["addedIn0_2"] = json!({ "flag": true, "list": [1, 2, 3] });
    }
    serde_json::to_vec(&json!({
        "id": format!("urn:uuid:bench-{id:08}"),
        "threadId": format!("urn:uuid:bench-thread-{id:08}"),
        "type": type_uri,
        "issuer": ALICE,
        "recipient": SERVER_VID,
        "issuedAt": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "payload": payload,
    }))
    .unwrap()
}

fn request(bytes: Vec<u8>) -> Request<Body> {
    Request::post("/trust-tasks")
        .header("content-type", "application/json")
        .header("authorization", "Bearer alice")
        .body(Body::from(bytes))
        .unwrap()
}

async fn scenario(name: &str, type_uri: &str, entries: usize, extra: bool, per_round: usize) {
    if let Some(filter) = std::env::args().nth(1) {
        if !name.contains(&filter) {
            return;
        }
    }
    let server = HttpsServer::builder()
        .local_vid(SERVER_VID)
        .with_auth(BearerAuth::from_pairs([("alice", ALICE)]))
        // The default guard's per-issuer share would refuse one sender's
        // flood of distinct documents, which is what this sends.
        .with_replay_guard(InMemoryReplayGuard::new(1_000_000).with_per_issuer_limit(1_000_000))
        .on_async::<Record, _, _>(|req, _ctx| async move {
            Ok(RecordResponse {
                count: req.payload.entries.len(),
            })
        })
        .build();
    let mut router = server.into_router();

    let size = body(type_uri, 0, entries, extra).len();
    let rounds = 7;
    let mut next = 0usize;
    let mut per_request = Vec::with_capacity(rounds);
    for round in 0..=rounds {
        let bodies: Vec<Vec<u8>> = (0..per_round)
            .map(|_| {
                next += 1;
                body(type_uri, next, entries, extra)
            })
            .collect();
        let start = Instant::now();
        for bytes in bodies {
            std::future::poll_fn(|cx| {
                <axum::Router as Service<Request<Body>>>::poll_ready(&mut router, cx)
            })
            .await
            .unwrap();
            let response = router.call(request(bytes)).await.unwrap();
            assert_eq!(response.status(), 200, "{name}");
        }
        // Round 0 is the warm-up.
        if round > 0 {
            per_request.push(start.elapsed().as_secs_f64() * 1e6 / per_round as f64);
        }
    }
    per_request.sort_by(f64::total_cmp);
    println!(
        "{name:<34} {size:>8} B  {:>9.2} µs/request (median of {rounds} rounds of {per_round})",
        per_request[rounds / 2],
    );
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    trust_tasks_rs::extension::declare_extends(EXTENDS);
    let v01 = Record::TYPE_URI;
    let v02 = "https://example.test/spec/bench/record/0.2";
    scenario("typical (3 entries)", v01, 3, false, 20_000).await;
    scenario("medium (100 entries)", v01, 100, false, 2_000).await;
    scenario("large (1500 entries)", v01, 1500, false, 200).await;
    scenario("forward route, typical (0.2→0.1)", v02, 3, true, 20_000).await;
    scenario("forward route, medium (0.2→0.1)", v02, 100, true, 2_000).await;
}
