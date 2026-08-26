//! A demo Trust Tasks client over HTTPS.
//!
//! Sends an `acl/grant/0.1` request to a server (default
//! `http://localhost:3000`, override via `TRUST_TASKS_URL`) and prints
//! the response. Pair with the `server_demo` example for end-to-end
//! exercise.
//!
//! `acl/grant/0.1` declares `proof` REQUIRED, so this example **signs**.
//! Both demos derive the same `did:key` from a fixed seed, so they agree
//! on the client's identity without any key exchange: the server maps the
//! `alice` bearer token to it, and SPEC §4.8.1 cross-checks the in-band
//! `issuer` against that transport-authenticated identity.
//!
//! Run with:
//!
//! ```sh
//! # in one terminal:
//! cargo run -p trust-tasks-https --example server_demo
//! # in another terminal:
//! cargo run -p trust-tasks-https --example client_demo
//! ```

use affinidi_secrets_resolver::secrets::Secret;
use trust_tasks_https::{ClientError, HttpsClient};
use trust_tasks_proof::affinidi::SignOptions;
use trust_tasks_proof::ProofExt;
use trust_tasks_rs::{specs::acl::grant::v0_1 as grant, TrustTask};

/// The demo client's identity, derived from a fixed seed so that
/// `server_demo` can arrive at the same value independently.
///
/// `Verifier::for_did_key()` resolves a `did:key` offline, so the pair of
/// demos needs no network and no DID document to verify the proof.
pub(crate) fn demo_identity() -> (Secret, String) {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("TRUST_TASKS_URL").unwrap_or_else(|_| "http://localhost:3000".into());

    let (secret, my_did) = demo_identity();

    // `my_vid` must be the signing identity, not a separate `did:web`:
    // the client checks the response's `recipient` against it, and the
    // server cross-checks the request's in-band `issuer` against the
    // identity its bearer token maps to.
    let client = HttpsClient::builder()
        .server_url(&url)
        .server_vid("did:web:maintainer.example")
        .my_vid(&my_did)
        .my_token("alice")
        .build()?;

    // Generated payload types are `#[non_exhaustive]` and built through the
    // builder typify emits: only the members this request actually carries
    // are named, so a member added to `acl/grant` in a later revision of the
    // specification leaves this call site alone.
    let entry: grant::AclEntry = grant::AclEntry::builder()
        .subject("did:web:carol.example")
        .role("moderator")
        .label("Carol — content moderation".to_string())
        .try_into()?;

    let request = TrustTask::for_payload(
        format!("urn:uuid:{}", uuid::Uuid::new_v4()),
        grant::Payload::builder()
            .entry(entry)
            .reason("onboarding moderator".to_string())
            .try_into()?,
    );

    // Party members and `issuedAt` must be set *before* signing.
    // `HttpsClient::send` fills in any that are still unset, and a member
    // added after signing is a member the proof does not cover — the
    // server would then reject it as `proofInvalid` rather than accept it.
    let mut request = request;
    request.issuer = Some(my_did.clone());
    request.recipient = Some("did:web:maintainer.example".into());
    request.issued_at = Some(chrono::Utc::now());
    request.sign(&secret, SignOptions::new()).await?;

    println!(
        "→ POST {}/trust-tasks\n  request id: {}\n  type: {}\n  signed by: {}",
        url, request.id, request.type_uri, my_did
    );

    match client.send::<grant::Payload>(request).await {
        Ok(resp) => {
            println!(
                "← {} (id: {}, threadId: {})\n  entry: subject={} role={}",
                resp.type_uri,
                resp.id,
                resp.thread_id.as_deref().unwrap_or("<none>"),
                resp.payload.entry.subject,
                &*resp.payload.entry.role,
            );
        }
        Err(ClientError::TrustTaskError { http_status, error }) => {
            println!(
                "← HTTP {} trust-task-error: code={} retryable={} message={:?}",
                http_status, error.payload.code, error.payload.retryable, error.payload.message
            );
        }
        Err(other) => return Err(other.into()),
    }

    Ok(())
}
