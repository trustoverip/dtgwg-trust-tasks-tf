//! End-to-end exercise of the HTTPS binding.
//!
//! Spins up a real `HttpsServer` on an ephemeral port and makes real
//! [`HttpsClient`] calls against it. Covers:
//!
//! * Success path: `acl/grant` → `#response`.
//! * Auth-derived identity-mismatch: in-band issuer ≠ bearer-resolved VID
//!   produces an HTTP 403 + `identity_mismatch` document.
//! * Unsupported type: a Type URI the server has no handler for produces
//!   HTTP 400 + `unsupported_type`.
//! * Spec-handler rejection: `acl/revoke` with an unauthenticated sender
//!   produces `permission_denied`.

use std::net::SocketAddr;

use serde::Serialize;
use tokio::net::TcpListener;
use trust_tasks_https::{BearerAuth, ClientError, HttpsClient, HttpsServer};
use trust_tasks_rs::{
    specs::acl::{grant, list, revoke, show},
    specs::trust_task_discovery::v0_1 as discovery,
    Proof, ProofVerifier, RejectReason, StandardCode, TrustTask, TypeUri, VerificationError,
};

const SERVER_VID: &str = "did:web:maintainer.example";

/// Test verifier that accepts every proof. Used by fixtures that need
/// to exercise the verifier-configured path without standing up a
/// cryptosuite implementation.
struct AcceptAllVerifier;

#[async_trait::async_trait]
impl ProofVerifier for AcceptAllVerifier {
    async fn verify<P>(&self, _doc: &TrustTask<P>) -> Result<(), VerificationError>
    where
        P: Serialize + Send + Sync,
    {
        Ok(())
    }
}

/// Test verifier that rejects every proof with `SignatureInvalid`. Used
/// by the proof-invalid integration test.
struct RejectAllVerifier;

#[async_trait::async_trait]
impl ProofVerifier for RejectAllVerifier {
    async fn verify<P>(&self, _doc: &TrustTask<P>) -> Result<(), VerificationError>
    where
        P: Serialize + Send + Sync,
    {
        Err(VerificationError::SignatureInvalid)
    }
}

/// Which proof-handling strategy the test server uses.
enum VerifierMode {
    /// No verifier configured — server falls back to "reject any
    /// proof-bearing document with `malformed_request`".
    None,
    /// Accept every proof. Used by the happy-path tests against
    /// REQUIRED specs (acl/grant, acl/revoke).
    AcceptAll,
    /// Reject every proof with `SignatureInvalid`. Used to exercise
    /// the `proof_invalid` path.
    RejectAll,
}

/// Build the test server's app router and bind to localhost:0 (kernel
/// chooses a free port). Returns the address the OS picked.
async fn spawn_server_with(verifier: VerifierMode) -> SocketAddr {
    let auth = BearerAuth::from_pairs([
        ("alice", "did:web:alice.example"),
        ("eve", "did:web:eve.example"),
    ]);

    let mut builder = HttpsServer::builder()
        .local_vid(SERVER_VID)
        .with_auth(auth)
        .on::<grant::v0_1::Payload, grant::v0_1::Response, _>(|req, _ctx| {
            Ok(grant::v0_1::Response {
                entry: req.payload.entry.clone(),
                ext: None,
            })
        })
        .on::<revoke::v0_1::Payload, revoke::v0_1::Response, _>(|_req, _ctx| {
            Ok(revoke::v0_1::Response {
                entry: None,
                ext: None,
            })
        })
        // acl/list is `proofRequirement: RECOMMENDED` — the binding
        // accepts proofless requests for it regardless of verifier
        // configuration. The handler also exercises the
        // PermissionDenied path for the auth-required test.
        .on::<list::v0_1::Payload, list::v0_1::Response, _>(|_req, ctx| {
            if ctx.authenticated_sender.is_none() {
                return Err(RejectReason::PermissionDenied {
                    reason: "list requires authentication".into(),
                });
            }
            // SPEC §4.8.1 resolved parties are available to handlers
            // without re-running resolve_parties — assert the wiring so
            // a regression fails the happy_path_acl_list test below.
            assert_eq!(
                ctx.resolved.issuer.as_deref(),
                Some("did:web:alice.example")
            );
            assert_eq!(ctx.resolved.recipient.as_deref(), Some(SERVER_VID));
            Ok(list::v0_1::Response {
                entries: vec![],
                cursor: None,
                redacted_fields: vec![],
                truncated: false,
                ext: None,
            })
        })
        // Auto-advertise the registered handlers (and discovery itself) via
        // trust-task-discovery/0.1.
        .enable_discovery();

    builder = match verifier {
        VerifierMode::None => builder,
        VerifierMode::AcceptAll => builder.with_verifier(AcceptAllVerifier),
        VerifierMode::RejectAll => builder.with_verifier(RejectAllVerifier),
    };

    let server = builder.build();

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = server.into_router();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

/// Default fixture for tests that don't care about proof handling —
/// `VerifierMode::AcceptAll` covers both proof-bearing and proofless
/// flows on the bundled handlers.
async fn spawn_server() -> SocketAddr {
    spawn_server_with(VerifierMode::AcceptAll).await
}

fn entry() -> grant::v0_1::AclEntry {
    grant::v0_1::AclEntry {
        subject: "did:web:carol.example".into(),
        role: "admin".parse().unwrap(),
        scopes: vec![],
        allowed_keys: None,
        label: None,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
        expires_at: None,
        approve: None,
        step_up: None,
        ext: None,
    }
}

fn build_client(addr: SocketAddr, my_vid: &str, my_token: Option<&str>) -> HttpsClient {
    let mut builder = HttpsClient::builder()
        .server_url(format!("http://{addr}"))
        .server_vid(SERVER_VID)
        .my_vid(my_vid);
    if let Some(t) = my_token {
        builder = builder.my_token(t);
    }
    builder.build().unwrap()
}

/// Happy path via `acl/list` — a `proofRequirement: RECOMMENDED` spec
/// the server accepts without a proof (independent of verifier
/// configuration).
#[tokio::test]
async fn happy_path_acl_list() {
    let addr = spawn_server().await;
    let client = build_client(addr, "did:web:alice.example", Some("alice"));

    let req = TrustTask::for_payload(
        "urn:uuid:test-list-1",
        list::v0_1::Payload {
            role: None,
            scope: None,
            direction: None,
            subject_prefix: None,
            page_size: None,
            cursor: None,
            ext: None,
        },
    );

    let resp = client
        .send::<list::v0_1::Payload, list::v0_1::Response>(req)
        .await
        .unwrap();

    assert_eq!(
        resp.type_uri,
        "https://trusttasks.org/spec/acl/list/0.1#response"
            .parse::<TypeUri>()
            .unwrap()
    );
    assert!(resp.payload.entries.is_empty());
    assert!(!resp.payload.truncated);
    assert_eq!(resp.thread_id.as_deref(), Some("urn:uuid:test-list-1"));
    // Server's response addresses the original producer.
    assert_eq!(resp.recipient.as_deref(), Some("did:web:alice.example"));
}

#[tokio::test]
async fn identity_mismatch_when_in_band_issuer_differs_from_token() {
    let addr = spawn_server().await;
    // Send as alice (bearer = alice) but claim to be carol (in-band issuer).
    let client = build_client(addr, "did:web:carol.example", Some("alice"));

    let req = TrustTask::for_payload(
        "urn:uuid:test-mismatch",
        grant::v0_1::Payload {
            entry: entry(),
            reason: None,
            ext: None,
        },
    );

    let err = client
        .send::<grant::v0_1::Payload, grant::v0_1::Response>(req)
        .await
        .unwrap_err();

    match err {
        ClientError::TrustTaskError { http_status, error } => {
            assert_eq!(http_status, 403);
            assert_eq!(error.payload.code, StandardCode::IdentityMismatch.into());
            // SPEC §10.4: message MUST NOT name either VID.
            let msg = error.payload.message.as_deref().unwrap_or("");
            assert!(!msg.contains("alice"), "wire leak: {msg}");
            assert!(!msg.contains("carol"), "wire leak: {msg}");
        }
        other => panic!("expected TrustTaskError, got {other:?}"),
    }
}

#[tokio::test]
async fn unsupported_type_for_unregistered_uri() {
    let addr = spawn_server().await;
    let client = build_client(addr, "did:web:alice.example", Some("alice"));

    // We send an acl/show request — the test server didn't register a
    // handler for it, so the dispatcher returns UnsupportedType.
    let req = TrustTask::for_payload(
        "urn:uuid:test-unsupported",
        show::v0_1::Payload {
            subject: "did:web:bob.example".parse().unwrap(),
            ext: None,
        },
    );

    let err = client
        .send::<show::v0_1::Payload, show::v0_1::Response>(req)
        .await
        .unwrap_err();

    match err {
        ClientError::TrustTaskError { http_status, error } => {
            assert_eq!(http_status, 400);
            assert_eq!(error.payload.code, StandardCode::UnsupportedType.into());
        }
        other => panic!("expected TrustTaskError, got {other:?}"),
    }
}

#[tokio::test]
async fn discovery_advertises_registered_handlers() {
    let addr = spawn_server().await;
    let client = build_client(addr, "did:web:alice.example", Some("alice"));

    // Empty pattern list ⇒ "give me everything".
    let req = TrustTask::for_payload(
        "urn:uuid:test-discover-all",
        discovery::Payload { patterns: vec![] },
    );

    let resp = client
        .send::<discovery::Payload, discovery::Response>(req)
        .await
        .unwrap();

    let mut got: Vec<&str> = resp.payload.supported_types.iter().map(uri_of).collect();
    got.sort();
    assert_eq!(
        got,
        vec![
            "https://trusttasks.org/spec/acl/grant/0.1",
            "https://trusttasks.org/spec/acl/list/0.1",
            "https://trusttasks.org/spec/acl/revoke/0.1",
            "https://trusttasks.org/spec/trust-task-discovery/0.1",
        ],
        "enable_discovery() should advertise the registered acl/* handlers plus discovery itself"
    );

    // SPEC §4.4.1: the success response carries the #response variant
    // of the request's Type URI.
    assert_eq!(
        resp.type_uri,
        "https://trusttasks.org/spec/trust-task-discovery/0.1#response"
            .parse::<TypeUri>()
            .unwrap()
    );
    assert_eq!(
        resp.thread_id.as_deref(),
        Some("urn:uuid:test-discover-all")
    );
}

#[tokio::test]
async fn discovery_filter_returns_only_matching_slugs() {
    let addr = spawn_server().await;
    let client = build_client(addr, "did:web:alice.example", Some("alice"));

    let req = TrustTask::for_payload(
        "urn:uuid:test-discover-acl",
        discovery::Payload {
            patterns: vec!["acl/*".parse().unwrap()],
        },
    );

    let resp = client
        .send::<discovery::Payload, discovery::Response>(req)
        .await
        .unwrap();

    let mut got: Vec<&str> = resp.payload.supported_types.iter().map(uri_of).collect();
    got.sort();
    assert_eq!(
        got,
        vec![
            "https://trusttasks.org/spec/acl/grant/0.1",
            "https://trusttasks.org/spec/acl/list/0.1",
            "https://trusttasks.org/spec/acl/revoke/0.1",
        ],
        "acl/* should match the three acl handlers but not trust-task-discovery"
    );
}

fn uri_of(entry: &discovery::ResponseSupportedTypesItem) -> &str {
    match entry {
        discovery::ResponseSupportedTypesItem::Uri(s) => s.as_str(),
        discovery::ResponseSupportedTypesItem::Object { type_, .. } => type_.as_str(),
    }
}

/// SPEC §7.2 item 7 (REQUIRED clause). `acl/grant` has
/// `proofRequirement.requirement: REQUIRED` in front matter, so codegen
/// emits `IS_PROOF_REQUIRED = true`. The server MUST reject a proofless
/// `acl/grant` request with `proof_required`, regardless of whether the
/// binding has its own verifier.
#[tokio::test]
async fn proof_required_when_spec_requires_and_doc_lacks_proof() {
    let addr = spawn_server().await;
    let client = build_client(addr, "did:web:alice.example", Some("alice"));

    // No proof on a REQUIRED spec.
    let req = TrustTask::for_payload(
        "urn:uuid:test-proof-required",
        grant::v0_1::Payload {
            entry: entry(),
            reason: None,
            ext: None,
        },
    );

    let err = client
        .send::<grant::v0_1::Payload, grant::v0_1::Response>(req)
        .await
        .unwrap_err();

    match err {
        ClientError::TrustTaskError { http_status, error } => {
            // status.rs maps proof_required → 401 Unauthorized.
            assert_eq!(http_status, 401);
            assert_eq!(error.payload.code, StandardCode::ProofRequired.into());
        }
        other => panic!("expected TrustTaskError, got {other:?}"),
    }
}

/// SPEC §8.1 — under `identity_mismatch`, the response MUST address the
/// transport-authenticated peer, not the contested in-band issuer. The
/// PR added a proof-bearing rejection earlier in the pipeline; this test
/// pins that identity_mismatch still wins (it runs before proof
/// handling), and that no identity oracle is created by the new path.
#[tokio::test]
async fn proof_bearing_with_identity_mismatch_routes_to_transport_peer() {
    let addr = spawn_server().await;
    // Send as alice (bearer = alice) but claim to be carol (in-band issuer).
    let client = build_client(addr, "did:web:carol.example", Some("alice"));

    let mut req = TrustTask::for_payload(
        "urn:uuid:test-proof-and-mismatch",
        grant::v0_1::Payload {
            entry: entry(),
            reason: None,
            ext: None,
        },
    );
    req.proof = Some(Proof {
        proof_type: "DataIntegrityProof".into(),
        cryptosuite: "eddsa-rdfc-2022".into(),
        verification_method: "did:web:carol.example#key-1".into(),
        created: chrono::Utc::now(),
        proof_purpose: "assertionMethod".into(),
        proof_value: "z3kg".into(),
        extra: Default::default(),
    });

    let err = client
        .send::<grant::v0_1::Payload, grant::v0_1::Response>(req)
        .await
        .unwrap_err();

    match err {
        ClientError::TrustTaskError { http_status, error } => {
            assert_eq!(http_status, 403);
            // §8.1: identity_mismatch wins over the proof-rejection
            // path because the in-band issuer is contested and we
            // MUST NOT leak that "your proof was rejected" to a
            // potential impostor.
            assert_eq!(error.payload.code, StandardCode::IdentityMismatch.into());
            // Wire message MUST NOT name either VID.
            let msg = error.payload.message.as_deref().unwrap_or("");
            assert!(!msg.contains("alice"), "wire leak: {msg}");
            assert!(!msg.contains("carol"), "wire leak: {msg}");
        }
        other => panic!("expected TrustTaskError, got {other:?}"),
    }
}

/// SECURITY: with no proof verifier configured, a producer-supplied
/// proof represents an integrity assertion the server cannot honour;
/// silently dropping it would mislead the producer. The server MUST
/// reject with `malformed_request`.
#[tokio::test]
async fn proof_bearing_document_rejected_when_server_has_no_verifier() {
    // Explicitly use the no-verifier fixture — the default fixture
    // configures an AcceptAllVerifier which would accept this proof.
    let addr = spawn_server_with(VerifierMode::None).await;
    let client = build_client(addr, "did:web:alice.example", Some("alice"));

    let mut req = TrustTask::for_payload(
        "urn:uuid:test-proof-rejected",
        grant::v0_1::Payload {
            entry: entry(),
            reason: None,
            ext: None,
        },
    );
    req.proof = Some(Proof {
        proof_type: "DataIntegrityProof".into(),
        cryptosuite: "eddsa-rdfc-2022".into(),
        verification_method: "did:web:alice.example#key-1".into(),
        created: chrono::Utc::now(),
        proof_purpose: "assertionMethod".into(),
        proof_value: "z3kg".into(),
        extra: Default::default(),
    });

    let err = client
        .send::<grant::v0_1::Payload, grant::v0_1::Response>(req)
        .await
        .unwrap_err();

    match err {
        ClientError::TrustTaskError { http_status, error } => {
            assert_eq!(http_status, 400);
            assert_eq!(error.payload.code, StandardCode::MalformedRequest.into());
            let msg = error.payload.message.as_deref().unwrap_or("");
            // Message MUST cite spec + policy but MUST NOT name the
            // server's configuration (no "verifier", no "configured" —
            // those would let a probe fingerprint the deployment).
            assert!(
                msg.contains("policy") && msg.contains("§7.2"),
                "message should cite the spec rule, not internals: {msg}"
            );
            assert!(!msg.contains("verifier"), "wire leak (config): {msg}");
            assert!(!msg.contains("configured"), "wire leak (config): {msg}");
        }
        other => panic!("expected TrustTaskError, got {other:?}"),
    }
}

/// Happy path against the REQUIRED-proof spec `acl/grant`. The
/// fixture's `AcceptAllVerifier` accepts the proof; the dispatch
/// closure's `IS_PROOF_REQUIRED` check is satisfied because the
/// document carries one. End-to-end re-enables the original
/// happy-path test that the earlier IS_PROOF_REQUIRED fix had to
/// disable (no verifier hook existed at that point).
#[tokio::test]
async fn happy_path_acl_grant_with_verifier() {
    let addr = spawn_server().await; // default fixture: AcceptAll
    let client = build_client(addr, "did:web:alice.example", Some("alice"));

    let mut req = TrustTask::for_payload(
        "urn:uuid:test-grant-verified",
        grant::v0_1::Payload {
            entry: entry(),
            reason: None,
            ext: None,
        },
    );
    req.proof = Some(Proof {
        proof_type: "DataIntegrityProof".into(),
        cryptosuite: "eddsa-rdfc-2022".into(),
        verification_method: "did:web:alice.example#key-1".into(),
        created: chrono::Utc::now(),
        proof_purpose: "assertionMethod".into(),
        proof_value: "z3kg".into(),
        extra: Default::default(),
    });

    let resp = client
        .send::<grant::v0_1::Payload, grant::v0_1::Response>(req)
        .await
        .unwrap();

    assert_eq!(
        resp.type_uri,
        "https://trusttasks.org/spec/acl/grant/0.1#response"
            .parse::<TypeUri>()
            .unwrap()
    );
    assert_eq!(&*resp.payload.entry.role, "admin");
    assert_eq!(resp.recipient.as_deref(), Some("did:web:alice.example"));
}

/// Verifier returns `Err` → server rejects with `proof_invalid` and
/// the failure message reaches the wire. Pins both the
/// `RejectReason::ProofInvalid` mapping and the configured-verifier
/// failure path on the binding.
#[tokio::test]
async fn proof_invalid_when_verifier_rejects() {
    let addr = spawn_server_with(VerifierMode::RejectAll).await;
    let client = build_client(addr, "did:web:alice.example", Some("alice"));

    // `acl/list` is RECOMMENDED, so the test isolates the verifier-
    // rejects path from the IS_PROOF_REQUIRED path.
    let mut req = TrustTask::for_payload(
        "urn:uuid:test-proof-invalid",
        list::v0_1::Payload {
            role: None,
            scope: None,
            direction: None,
            subject_prefix: None,
            page_size: None,
            cursor: None,
            ext: None,
        },
    );
    req.proof = Some(Proof {
        proof_type: "DataIntegrityProof".into(),
        cryptosuite: "eddsa-rdfc-2022".into(),
        verification_method: "did:web:alice.example#key-1".into(),
        created: chrono::Utc::now(),
        proof_purpose: "assertionMethod".into(),
        proof_value: "z3kg".into(),
        extra: Default::default(),
    });

    let err = client
        .send::<list::v0_1::Payload, list::v0_1::Response>(req)
        .await
        .unwrap_err();

    match err {
        ClientError::TrustTaskError { http_status, error } => {
            assert_eq!(http_status, 401);
            assert_eq!(error.payload.code, StandardCode::ProofInvalid.into());
            // The verifier's error description surfaces on the wire.
            let msg = error.payload.message.as_deref().unwrap_or("");
            assert!(
                msg.contains("signature"),
                "expected signature-error description: {msg}"
            );
        }
        other => panic!("expected TrustTaskError, got {other:?}"),
    }
}

#[tokio::test]
async fn permission_denied_from_spec_handler() {
    let addr = spawn_server().await;
    // No bearer token — list handler rejects on unauthenticated calls.
    let client = build_client(addr, "did:web:alice.example", None);

    let req = TrustTask::for_payload(
        "urn:uuid:test-list-noauth",
        list::v0_1::Payload {
            role: None,
            scope: None,
            direction: None,
            subject_prefix: None,
            page_size: None,
            cursor: None,
            ext: None,
        },
    );

    let err = client
        .send::<list::v0_1::Payload, list::v0_1::Response>(req)
        .await
        .unwrap_err();

    match err {
        ClientError::TrustTaskError { http_status, error } => {
            assert_eq!(http_status, 403);
            assert_eq!(error.payload.code, StandardCode::PermissionDenied.into());
        }
        other => panic!("expected TrustTaskError, got {other:?}"),
    }
}

// ─── SPEC §10.2 parser hardening (pre-auth DoS controls) ──────────────────

/// An over-limit body is rejected by the router's `DefaultBodyLimit` before
/// it is buffered, parsed, or authenticated — an audited memory-exhaustion
/// control (SPEC §10.2). 512 KiB exceeds the 256 KiB cap.
#[tokio::test]
async fn oversized_body_is_rejected_before_processing() {
    let addr = spawn_server().await;
    let big = vec![b'a'; 512 * 1024];
    let resp = reqwest::Client::new()
        .post(format!("http://{addr}/trust-tasks"))
        .body(big)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::PAYLOAD_TOO_LARGE);
}

/// A pathologically nested JSON body within the size budget exceeds
/// `serde_json`'s default 128-level recursion limit, so it fails to parse
/// (→ `malformedRequest`/400) rather than overflowing the stack.
#[tokio::test]
async fn deeply_nested_body_fails_to_parse_not_overflow() {
    let addr = spawn_server().await;
    let body = "[".repeat(1000) + &"]".repeat(1000);
    let resp = reqwest::Client::new()
        .post(format!("http://{addr}/trust-tasks"))
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::BAD_REQUEST);
}

// ─── Regressions from the PR #75 security re-review ───────────────────────

/// SPEC §10.4 — the suppressed identity-mismatch path must be indistinguishable
/// from a plain parse failure: same HTTP status AND same body code. Previously
/// the body was a generic `malformedRequest` but the status stayed 403 (derived
/// from the original IdentityMismatch reason), leaking a 403-vs-400 oracle to an
/// unauthenticated prober.
#[tokio::test]
async fn suppressed_identity_mismatch_is_indistinguishable_from_parse_failure() {
    let addr = spawn_server().await;
    let client = reqwest::Client::new();
    let url = format!("http://{addr}/trust-tasks");

    // No bearer + in-band recipient that mismatches the server VID → the server
    // cannot address a response (no transport sender) → suppressed path.
    let mismatch = client
        .post(&url)
        .header("content-type", "application/json")
        .body(
            serde_json::json!({
                "id": "urn:uuid:probe",
                "type": "https://trusttasks.org/spec/acl/grant/0.1",
                "issuer": "did:web:alice.example",
                "recipient": "did:web:wrong.example",
                "payload": { "entry": { "subject": "did:web:carol.example", "role": "admin" } }
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();
    let mismatch_status = mismatch.status();
    let mismatch_code =
        mismatch.json::<serde_json::Value>().await.unwrap()["payload"]["code"].clone();

    // A garbage body → genuine parse failure.
    let garbage = client
        .post(&url)
        .header("content-type", "application/json")
        .body("not json")
        .send()
        .await
        .unwrap();
    let garbage_status = garbage.status();
    let garbage_code =
        garbage.json::<serde_json::Value>().await.unwrap()["payload"]["code"].clone();

    assert_eq!(mismatch_status, reqwest::StatusCode::BAD_REQUEST);
    assert_eq!(
        mismatch_status, garbage_status,
        "status must not distinguish the two"
    );
    assert_eq!(mismatch_code, serde_json::json!("malformedRequest"));
    assert_eq!(
        mismatch_code, garbage_code,
        "body code must not distinguish the two"
    );
}

/// SPEC §7.2 item 5b — recipient-REQUIRED must be enforced on the HTTPS pipeline
/// too (not only the library `consume_inbound` path). acl/grant declares its
/// recipient REQUIRED; a document with no in-band recipient is malformed even
/// though the transport could fill it.
#[tokio::test]
async fn https_enforces_recipient_required_with_no_in_band_recipient() {
    let addr = spawn_server().await;
    let resp = reqwest::Client::new()
        .post(format!("http://{addr}/trust-tasks"))
        .header("authorization", "Bearer alice")
        .header("content-type", "application/json")
        .body(
            serde_json::json!({
                "id": "urn:uuid:no-recip",
                "type": "https://trusttasks.org/spec/acl/grant/0.1",
                "issuer": "did:web:alice.example",
                "payload": { "entry": { "subject": "did:web:carol.example", "role": "admin" } }
            })
            .to_string(),
        )
        .send()
        .await
        .unwrap();
    let status = resp.status();
    let code = resp.json::<serde_json::Value>().await.unwrap()["payload"]["code"].clone();
    assert_eq!(status, reqwest::StatusCode::BAD_REQUEST);
    assert_eq!(code, serde_json::json!("malformedRequest"));
}

/// The client must never hang on an unresponsive peer: its `reqwest::Client`
/// carries finite timeouts by default, so a server that accepts the
/// connection and then goes silent surfaces as an error.
#[tokio::test]
async fn client_times_out_on_a_silent_server() {
    use std::time::Duration;
    use trust_tasks_https::HttpsClient;
    use trust_tasks_rs::specs::acl::grant::v0_1 as grant;

    // Accepts connections, never answers.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let mut held = Vec::new();
        loop {
            let Ok((socket, _)) = listener.accept().await else {
                break;
            };
            held.push(socket);
        }
    });

    let client = HttpsClient::builder()
        .server_url(format!("http://{addr}"))
        .server_vid("did:web:server.example")
        .my_vid("did:web:alice.example")
        .timeout(Duration::from_millis(200))
        .build()
        .unwrap();

    let request = trust_tasks_rs::TrustTask::for_payload(
        "urn:uuid:timeout-test".to_string(),
        grant::Payload {
            entry: grant::AclEntry {
                subject: "did:web:carol.example".into(),
                role: "moderator".into(),
                scopes: vec![],
                allowed_keys: None,
                label: None,
                created_at: None,
                created_by: None,
                updated_at: None,
                updated_by: None,
                expires_at: None,
                approve: None,
                step_up: None,
                ext: None,
            },
            reason: None,
            ext: None,
        },
    );

    let started = std::time::Instant::now();
    let err = client
        .send::<grant::Payload, grant::Response>(request)
        .await
        .unwrap_err();
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "the call must fail fast, not hang"
    );
    match err {
        trust_tasks_https::ClientError::Http(e) => assert!(e.is_timeout(), "got: {e}"),
        other => panic!("expected Http timeout error, got: {other}"),
    }
}
