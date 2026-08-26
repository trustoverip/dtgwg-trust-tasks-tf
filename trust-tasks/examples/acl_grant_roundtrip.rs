//! A signed `acl/grant/0.1` round trip over HTTPS, both ends in one process.
//!
//! ```sh
//! cargo run -p trust-tasks --features https,proof-affinidi \
//!     --example acl_grant_roundtrip
//! ```
//!
//! This is the worked example behind [`GETTING-STARTED.md`] at the repo root.
//! The three regions marked `GETTING-STARTED:begin/end` below are extracted
//! verbatim into that document, and `tests/getting_started_snippets.rs` fails
//! if the file and the document disagree — so the snippets a newcomer copies
//! are code that compiles and runs, not code that once did.
//!
//! Everything here is reached through the `trust-tasks` facade
//! (`trust_tasks::https`, `trust_tasks::proof`, `trust_tasks::specs`), which is
//! exactly what a consumer with one dependency line writes.
//!
//! [`GETTING-STARTED.md`]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/GETTING-STARTED.md

// GETTING-STARTED:begin imports
use affinidi_secrets_resolver::secrets::Secret;
use trust_tasks::https::{BearerAuth, ClientError, HttpsClient, HttpsServer};
use trust_tasks::proof::affinidi::{SignOptions, Verifier};
use trust_tasks::proof::ProofExt;
use trust_tasks::specs::acl::grant::v0_1 as grant;
use trust_tasks::TrustTask;
// GETTING-STARTED:end imports

const ADDR: &str = "127.0.0.1:3000";
const SERVER_VID: &str = "did:web:maintainer.example";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (secret, my_did) = demo_identity();

    tokio::spawn(receiver(my_did.clone()));
    wait_until_listening().await?;

    sender(&secret, &my_did).await
}

// ── Receiver ────────────────────────────────────────────────────────────────
// GETTING-STARTED:begin receiver
/// The consuming end: authenticate the peer, verify the proof, handle the task.
async fn receiver(alice_did: String) -> std::io::Result<()> {
    // Bearer token → Verifiable Identifier. This is what the SPEC §4.8.1
    // cross-check compares the document's in-band `issuer` against, so it must
    // be the DID the sender actually signs with.
    let auth = BearerAuth::from_pairs([("alice", alice_did.as_str())]);

    let server = HttpsServer::builder()
        .local_vid("did:web:maintainer.example")
        .with_auth(auth)
        // Without a verifier the server only checks a proof is *present*.
        // `for_did_key()` resolves `did:key` offline — no network, no DID doc.
        .with_verifier(Verifier::for_did_key())
        .on::<grant::Payload, grant::Response, _>(|req, ctx| {
            println!(
                "  [server] acl/grant from {} → subject={} role={}",
                ctx.authenticated_sender.as_deref().unwrap_or("<unauth>"),
                req.payload.entry.subject,
                &*req.payload.entry.role,
            );
            // Generated types are `#[non_exhaustive]`: build with `builder()`.
            Ok(grant::Response::builder()
                .entry(req.payload.entry.clone())
                .try_into()
                .expect("acl/grant response"))
        })
        .build();

    server.serve("127.0.0.1:3000").await?;
    Ok(())
}
// GETTING-STARTED:end receiver

// ── Sender ──────────────────────────────────────────────────────────────────
// GETTING-STARTED:begin sender
/// The producing end: build the document, sign it, send it, read the reply.
async fn sender(secret: &Secret, my_did: &str) -> Result<(), Box<dyn std::error::Error>> {
    // `my_vid` must be the *signing* identity: the server cross-checks the
    // in-band `issuer` against whatever its bearer token maps to (§4.8.1).
    let client = HttpsClient::builder()
        .server_url("http://127.0.0.1:3000")
        .server_vid("did:web:maintainer.example")
        .my_vid(my_did)
        .my_token("alice")
        .build()?;

    // Generated payload types are `#[non_exhaustive]` as of trust-tasks-rs
    // 0.14, so cross-crate construction goes through `X::builder()`. A member
    // added by a later revision of the spec leaves this call site alone.
    let entry: grant::AclEntry = grant::AclEntry::builder()
        .subject("did:web:carol.example")
        .role("moderator")
        .label("Carol — content moderation".to_string())
        .try_into()?;

    let mut request = TrustTask::for_payload(
        format!("urn:uuid:{}", uuid::Uuid::new_v4()),
        grant::Payload::builder()
            .entry(entry)
            .reason("onboarding moderator".to_string())
            .try_into()?,
    );

    // TRAP: party members and `issuedAt` must be set BEFORE signing.
    // `HttpsClient::send` fills in any that are still unset — and a member
    // filled in after the fact is a member the proof does not cover, which the
    // server rejects as `proofInvalid`.
    request.issuer = Some(my_did.to_string());
    request.recipient = Some("did:web:maintainer.example".into());
    request.issued_at = Some(chrono::Utc::now());
    request.sign(secret, SignOptions::new()).await?;

    println!("  [client] POST /trust-tasks  id={}", request.id);

    match client.send::<grant::Payload>(request).await {
        Ok(resp) => println!(
            "  [client] ← {}\n           threadId={} entry: subject={} role={}",
            resp.type_uri,
            resp.thread_id.as_deref().unwrap_or("<none>"),
            resp.payload.entry.subject,
            &*resp.payload.entry.role,
        ),
        Err(ClientError::TrustTaskError { http_status, error }) => println!(
            "  [client] ← HTTP {} trust-task-error: code={} retryable={}",
            http_status, error.payload.code, error.payload.retryable,
        ),
        Err(other) => return Err(other.into()),
    }
    Ok(())
}
// GETTING-STARTED:end sender

// ── Identity ────────────────────────────────────────────────────────────────
// GETTING-STARTED:begin identity
/// A `did:key` derived from a fixed seed, so both ends agree on the sender's
/// identity with no key exchange. Real deployments load a real key here.
fn demo_identity() -> (Secret, String) {
    const SEED: [u8; 32] = [7u8; 32];
    let throwaway = Secret::generate_ed25519(None, Some(&SEED));
    let pk_mb = throwaway
        .get_public_keymultibase()
        .expect("ed25519 public multikey");
    let vm = format!("did:key:{pk_mb}#{pk_mb}");
    let mut secret = Secret::generate_ed25519(Some(&vm), Some(&SEED));
    secret.id = vm.clone();
    let did = vm.split('#').next().expect("did:key prefix").to_string();
    (secret, did)
}
// GETTING-STARTED:end identity

/// Poll until the spawned server has bound its listener. Plumbing for a
/// single-process demo; a real client just connects.
async fn wait_until_listening() -> Result<(), Box<dyn std::error::Error>> {
    for _ in 0..100 {
        if tokio::net::TcpStream::connect(ADDR).await.is_ok() {
            println!("  [server] listening on http://{ADDR}/trust-tasks (vid {SERVER_VID})");
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    Err(format!("server never came up on {ADDR}").into())
}
