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

use tokio::net::TcpListener;
use trust_tasks_https::{BearerAuth, ClientError, HttpsClient, HttpsServer};
use trust_tasks_rs::{
    specs::acl::{grant, list, revoke},
    specs::trust_task_discovery::v0_1 as discovery,
    RejectReason, StandardCode, TrustTask, TypeUri,
};

const SERVER_VID: &str = "did:web:maintainer.example";

/// Build the test server's app router and bind to localhost:0 (kernel
/// chooses a free port). Returns `(addr, JoinHandle)` so the test can
/// connect to the URL the OS picked.
async fn spawn_server() -> SocketAddr {
    let auth = BearerAuth::from_pairs([
        ("alice", "did:web:alice.example"),
        ("eve", "did:web:eve.example"),
    ]);

    let server = HttpsServer::builder()
        .local_vid(SERVER_VID)
        .with_auth(auth)
        .on::<grant::v0_1::Payload, grant::v0_1::Response, _>(|req, _ctx| {
            Ok(grant::v0_1::Response {
                entry: req.payload.entry.clone(),
                ext: None,
            })
        })
        .on::<revoke::v0_1::Payload, revoke::v0_1::Response, _>(|_req, ctx| {
            if ctx.authenticated_sender.is_none() {
                return Err(RejectReason::PermissionDenied {
                    reason: "revoke requires authentication".into(),
                });
            }
            Ok(revoke::v0_1::Response {
                entry: None,
                ext: None,
            })
        })
        // Auto-advertise the two acl handlers (and discovery itself) via
        // trust-task-discovery/0.1.
        .enable_discovery()
        .build();

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = server.into_router();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    addr
}

fn entry() -> grant::v0_1::AclEntry {
    grant::v0_1::AclEntry {
        subject: "did:web:carol.example".into(),
        role: "admin".parse().unwrap(),
        scopes: vec![],
        label: None,
        created_at: None,
        created_by: None,
        updated_at: None,
        updated_by: None,
        expires_at: None,
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

#[tokio::test]
async fn happy_path_acl_grant() {
    let addr = spawn_server().await;
    let client = build_client(addr, "did:web:alice.example", Some("alice"));

    let req = TrustTask::for_payload(
        "urn:uuid:test-grant-1",
        grant::v0_1::Payload {
            entry: entry(),
            reason: None,
            ext: None,
        },
    );

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
    assert_eq!(resp.thread_id.as_deref(), Some("urn:uuid:test-grant-1"));
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

    // We send an acl/list request — the test server didn't register a
    // handler for it, so the dispatcher returns UnsupportedType.
    let req = TrustTask::for_payload(
        "urn:uuid:test-unsupported",
        list::v0_1::Payload {
            role: None,
            scope: None,
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
            "https://trusttasks.org/spec/acl/revoke/0.1",
            "https://trusttasks.org/spec/trust-task-discovery/0.1",
        ],
        "enable_discovery() should advertise the registered acl/grant + acl/revoke handlers \
         plus discovery itself"
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
            "https://trusttasks.org/spec/acl/revoke/0.1",
        ],
        "acl/* should match grant + revoke but not trust-task-discovery"
    );
}

fn uri_of(entry: &discovery::ResponseSupportedTypesItem) -> &str {
    match entry {
        discovery::ResponseSupportedTypesItem::Uri(s) => s.as_str(),
        discovery::ResponseSupportedTypesItem::Object { type_, .. } => type_.as_str(),
    }
}

#[tokio::test]
async fn permission_denied_from_spec_handler() {
    let addr = spawn_server().await;
    // No bearer token — revoke handler rejects.
    let client = build_client(addr, "did:web:alice.example", None);

    let req = TrustTask::for_payload(
        "urn:uuid:test-revoke",
        revoke::v0_1::Payload {
            subject: "did:web:bob.example".into(),
            scopes: vec![],
            reason: None,
            ext: None,
        },
    );

    let err = client
        .send::<revoke::v0_1::Payload, revoke::v0_1::Response>(req)
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
