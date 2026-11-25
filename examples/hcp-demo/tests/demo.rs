//! The demo's own tests: hermetic (no network, no files outside this
//! repository), deterministic (fixed keys, ids and clock), and run with
//! `cargo test -p trust-tasks-example-hcp-demo`. SCENARIO.md "Tests every
//! port has" lists them; every port has the same ones.

use serde_json::{json, Value};
use trust_tasks::Payload as _;
use trust_tasks_example_hcp_demo::{
    ask, check_reply, grant_request, query, query_payload, request_uri, revoked, run_demo,
    step_discover, step_grant_request, step_notice, step_revoke, DemoResult, Granted, World,
    AFTER_APPROVAL_WINDOW, ERROR_TYPE_URI, MANAGEMENT_TOKEN,
};

/// The golden transcript every language's port shares, kept beside the
/// TypeScript version that wrote it first. Compared as it is, never copied.
const GOLDEN: &str = include_str!("../../hcp-demo-ts/transcript.golden.txt");
/// The grant's token, as the provider minted it.
const TOKEN: &str = "hcp_at_demo_0001";

/// A world after the whole demo has run.
async fn after_demo() -> (World, DemoResult) {
    let world = World::new();
    let result = run_demo(&world).await.expect("the demo runs");
    (world, result)
}

/// A world after steps 1 and 2: the grant is issued and active.
async fn after_grant() -> (World, Granted) {
    let world = World::new();
    step_discover(&world).await.expect("step 1");
    let granted = step_grant_request(&world).await.expect("step 2");
    (world, granted)
}

fn grant_uri() -> String {
    request_uri(grant_request::Payload::TYPE_URI)
}

fn query_uri() -> String {
    request_uri(query::Payload::TYPE_URI)
}

#[tokio::test]
async fn t01_the_six_steps_print_exactly_the_golden_transcript() {
    let (_, result) = after_demo().await;
    if result.transcript != GOLDEN {
        // Name the first differing line rather than dump two 450-line strings.
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
async fn t02_step_5_a_query_after_revoke_fails_with_invalid_token() {
    let (_, result) = after_demo().await;
    let error = result.post_revoke_error;
    assert_eq!(error["type"], ERROR_TYPE_URI);
    assert_eq!(error["payload"]["code"], "hcp:invalidToken");
}

#[tokio::test]
async fn t03_a_document_altered_after_signing_is_refused_with_proof_invalid() {
    let (world, _) = after_demo().await;
    let (p, c, _) = world.parties();
    let mut tampered = world
        .request(c, p, &query_uri(), query_payload(MANAGEMENT_TOKEN), None)
        .await;
    tampered.payload["query"] = json!("Tell me everything.");
    let reply = world.exchange(c, p, &tampered).await.expect("a reply");
    assert_eq!(reply["payload"]["code"], "proofInvalid");
}

#[tokio::test]
async fn t04_a_signed_document_that_fails_its_schema_is_refused_with_malformed_request() {
    let (world, _) = after_demo().await;
    let (p, c, _) = world.parties();
    let mut payload = query_payload(MANAGEMENT_TOKEN);
    payload
        .as_object_mut()
        .expect("an object")
        .remove("purpose");
    let signed = world.request(c, p, &query_uri(), payload, None).await;
    let reply = world.exchange(c, p, &signed).await.expect("a reply");
    assert_eq!(reply["payload"]["code"], "malformedRequest");
}

#[tokio::test]
async fn t05_after_the_notice_the_client_holds_no_token_and_a_later_attempt_is_not_acted_on() {
    let (world, result) = after_demo().await;
    let (p, c, _) = world.parties();
    assert!(!world.wallet().contains_key(&result.grant_id));
    let notice_id = result.notice.payload["noticeId"]
        .as_str()
        .expect("a noticeId")
        .to_string();
    let expected = [(notice_id, result.grant_id.clone())].into_iter().collect();
    assert_eq!(world.processed_notices(), expected);
    // A later delivery attempt: a new document, the same threadId and payload (noticeId included).
    let again = world
        .request(
            p,
            c,
            &request_uri(revoked::Payload::TYPE_URI),
            result.notice.payload.clone(),
            result.notice.thread_id.as_deref(),
        )
        .await;
    let ack = world.exchange(p, c, &again).await.expect("a reply");
    assert_eq!(ack["type"], revoked::Response::TYPE_URI);
    assert_eq!(ack["payload"], json!({}));
    assert_eq!(world.processed_notices(), expected);
}

#[tokio::test]
async fn t06_the_notice_alone_makes_the_client_drop_the_grant() {
    let (world, Granted { grant_id, .. }) = after_grant().await;
    step_revoke(&world, &grant_id).await.expect("step 4");
    assert!(world.wallet().contains_key(&grant_id));
    step_notice(&world, &grant_id).await.expect("step 6");
    assert!(!world.wallet().contains_key(&grant_id));
}

#[tokio::test]
async fn t07_the_grants_token_from_another_vid_is_refused_with_permission_denied() {
    let (world, _) = after_grant().await;
    let (p, _, m) = world.parties();
    let sent = world
        .request(m, p, &query_uri(), query_payload(TOKEN), None)
        .await;
    let reply = world.exchange(m, p, &sent).await.expect("a reply");
    assert_eq!(reply["type"], ERROR_TYPE_URI);
    assert_eq!(reply["payload"]["code"], "permissionDenied");
}

#[tokio::test]
async fn t08_a_grant_request_from_an_unregistered_issuer_is_refused_with_permission_denied() {
    let (world, _) = after_demo().await;
    let (p, _, m) = world.parties();
    let sent = world.request(m, p, &grant_uri(), ask(), None).await;
    let reply = world.exchange(m, p, &sent).await.expect("a reply");
    assert_eq!(reply["payload"]["code"], "permissionDenied");
    assert_eq!(world.hold_count(), 1);
}

#[tokio::test]
async fn t09_a_resubmission_that_differs_from_the_held_request_is_refused_with_malformed_request() {
    let (world, Granted { thread_id, .. }) = after_grant().await;
    let (p, c, _) = world.parties();
    let mut differs = ask();
    differs["expiresIn"] = json!(3600);
    let sent = world
        .request(c, p, &grant_uri(), differs, Some(&thread_id))
        .await;
    let reply = world.exchange(c, p, &sent).await.expect("a reply");
    assert_eq!(reply["payload"]["code"], "malformedRequest");
}

#[tokio::test]
async fn t10_within_the_hour_a_resubmission_gets_the_same_grant_never_a_second() {
    let (
        world,
        Granted {
            thread_id,
            grant_id,
        },
    ) = after_grant().await;
    let (p, c, _) = world.parties();
    // The same payload with its members in another order: the provider compares JCS forms.
    let a = ask();
    let reordered: Value = serde_json::from_str(&format!(
        r#"{{"subjectHint": {}, "expiresIn": {}, "access": {}}}"#,
        a["subjectHint"], a["expiresIn"], a["access"]
    ))
    .expect("JSON");
    let sent = world
        .request(c, p, &grant_uri(), reordered, Some(&thread_id))
        .await;
    let reply = world.exchange(c, p, &sent).await.expect("a reply");
    assert_eq!(reply["type"], grant_request::Response::TYPE_URI);
    assert_eq!(reply["payload"]["grant"]["grantId"], json!(grant_id));
    assert_eq!(world.grant_count(), 1);
}

#[tokio::test]
async fn t11_a_resubmission_after_the_hour_is_refused_with_permission_denied() {
    let (world, Granted { thread_id, .. }) = after_grant().await;
    let (p, c, _) = world.parties();
    world.clock().advance(AFTER_APPROVAL_WINDOW + 1);
    let sent = world
        .request(c, p, &grant_uri(), ask(), Some(&thread_id))
        .await;
    let reply = world.exchange(c, p, &sent).await.expect("a reply");
    assert_eq!(reply["payload"]["code"], "permissionDenied");
}

#[tokio::test]
async fn t12_a_resubmission_once_the_grant_is_revoked_is_refused_and_no_grant_is_issued() {
    let (world, result) = after_demo().await;
    let (p, c, _) = world.parties();
    let sent = world
        .request(c, p, &grant_uri(), ask(), Some(&result.thread_id))
        .await;
    let reply = world.exchange(c, p, &sent).await.expect("a reply");
    assert_eq!(reply["payload"]["code"], "permissionDenied");
    assert_eq!(world.grant_count(), 1);
}

#[test]
fn t13_a_reply_is_used_only_from_the_party_asked_on_the_thread_and_signed_unless_library_built() {
    let world = World::new();
    let (p, c, _) = world.parties();
    let reply = |code: &str, signed: bool| {
        let mut doc = json!({
            "id": "urn:uuid:00000000-0000-4000-8000-0000000000f0",
            "type": ERROR_TYPE_URI,
            "issuer": p.vid,
            "recipient": c.vid,
            "issuedAt": "2026-11-25T10:00:01Z",
            "threadId": "t",
            "payload": { "code": code },
        });
        if signed {
            // Presence only: the proof itself is verified separately.
            doc["proof"] = json!({ "type": "DataIntegrityProof" });
        }
        doc
    };
    check_reply(&reply("proofInvalid", false), p, "t").expect("a library-built error, unsigned");
    check_reply(&reply("permissionDenied", true), p, "t").expect("a signed refusal");
    let unsigned = check_reply(&reply("permissionDenied", false), p, "t").unwrap_err();
    assert!(unsigned.to_string().contains("no proof"), "{unsigned}");
    let other_party = check_reply(&reply("proofInvalid", false), c, "t").unwrap_err();
    assert!(other_party.to_string().contains("issuer"), "{other_party}");
    let other_thread = check_reply(&reply("proofInvalid", false), p, "other").unwrap_err();
    assert!(
        other_thread.to_string().contains("threadId"),
        "{other_thread}"
    );
}

#[tokio::test]
async fn t14_another_registered_client_on_a_held_thread_is_refused_and_the_hold_is_unchanged() {
    let world = World::new();
    let (p, c, m) = world.parties();
    // A second registered client, so the check on the hold's issuer is
    // reached (rule 1 passes).
    world.register_client(
        &m.vid,
        json!({
            "name": "Alex's wallet",
            "operator": { "id": "wallet.example", "name": "Wallet Example Ltd" },
        }),
    );
    let first = world.request(c, p, &grant_uri(), ask(), None).await;
    let held = world.exchange(c, p, &first).await.expect("a reply");
    assert_eq!(
        held["type"],
        "https://trusttasks.org/spec/trust-task-next-step/0.1"
    );
    let thread_id = first.thread_id.clone().expect("threadId");
    world.person_approves(&thread_id);
    let resubmission = world
        .request(m, p, &grant_uri(), ask(), Some(&thread_id))
        .await;
    let reply = world.exchange(m, p, &resubmission).await.expect("a reply");
    assert_eq!(reply["type"], ERROR_TYPE_URI);
    assert_eq!(reply["payload"]["code"], "permissionDenied");
    assert_eq!(world.hold(&thread_id), Some((c.vid.clone(), "approved")));
    assert_eq!(world.grant_count(), 0);
}
