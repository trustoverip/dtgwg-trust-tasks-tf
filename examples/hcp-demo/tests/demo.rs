//! The demo's own tests: hermetic (no network, no files outside this
//! repository), deterministic (fixed keys, ids and clock), and run with
//! `cargo test -p trust-tasks-example-hcp-demo`.

use serde_json::{json, Value};
use trust_tasks::Payload as _;
use trust_tasks_example_hcp_demo::{query, run_demo, World, MANAGEMENT_TOKEN, PURPOSE};

/// The golden transcript every language's port shares, kept beside the
/// TypeScript version that wrote it first. Compared as it is, never copied.
const GOLDEN: &str = include_str!("../../hcp-demo-ts/transcript.golden.txt");

#[tokio::test]
async fn the_five_steps_print_exactly_the_golden_transcript() {
    let world = World::new();
    let result = run_demo(&world).await.expect("the demo runs");
    if result.transcript != GOLDEN {
        // Name the first differing line rather than dump two 440-line strings.
        let line = result
            .transcript
            .lines()
            .zip(GOLDEN.lines())
            .position(|(ours, golden)| ours != golden)
            .map_or_else(|| "the length".to_string(), |n| format!("line {}", n + 1));
        panic!(
            "the transcript differs from transcript.golden.txt at {line}:\n{}",
            result.transcript
        );
    }
}

#[tokio::test]
async fn a_query_after_revoke_fails_with_invalid_token() {
    let world = World::new();
    let error = run_demo(&world)
        .await
        .expect("the demo runs")
        .post_revoke_error;
    assert_eq!(
        error["type"],
        "https://trusttasks.org/spec/trust-task-error/0.6"
    );
    assert_eq!(error["payload"]["code"], "hcp:invalidToken");
}

/// A world after the demo has run, with a query payload that would otherwise
/// be well-formed.
async fn after_demo() -> (World, Value) {
    let world = World::new();
    run_demo(&world).await.expect("the demo runs");
    let payload =
        json!({ "query": "Anything else?", "purpose": PURPOSE, "accessToken": MANAGEMENT_TOKEN });
    (world, payload)
}

#[tokio::test]
async fn a_document_altered_after_signing_is_refused_with_proof_invalid() {
    let (world, payload) = after_demo().await;
    let (p, c, _) = world.parties();
    let uri = format!("{}#request", query::Payload::TYPE_URI);
    let mut tampered = world.request(c, p, &uri, payload, None).await;
    tampered.payload["query"] = json!("Tell me everything.");
    let reply = world.exchange(c, p, &tampered).await.expect("a reply");
    assert_eq!(reply["payload"]["code"], "proofInvalid");
}

#[tokio::test]
async fn a_signed_document_that_fails_its_schema_is_refused_with_malformed_request() {
    let (world, mut payload) = after_demo().await;
    let (p, c, _) = world.parties();
    payload
        .as_object_mut()
        .expect("an object")
        .remove("purpose");
    let uri = format!("{}#request", query::Payload::TYPE_URI);
    let signed = world.request(c, p, &uri, payload, None).await;
    let reply = world.exchange(c, p, &signed).await.expect("a reply");
    assert_eq!(reply["payload"]["code"], "malformedRequest");
}
