//! A demo Trust Tasks server over HTTPS.
//!
//! Exposes `acl/grant/0.1` and `acl/revoke/0.1` on `http://localhost:3000`.
//! Two bearer tokens are accepted: `alice` (maps to the `did:key` the
//! `client_demo` signs with) and `bob` (maps to `did:web:bob.example`);
//! everything else is treated as unauthenticated.
//!
//! `alice` maps to a `did:key` rather than a `did:web` because
//! `acl/grant/0.1` declares `proof` REQUIRED: the client signs, and SPEC
//! §4.8.1 requires the in-band `issuer` to agree with the identity the
//! transport authenticated. Mapping `alice` to some other DID would turn
//! the demo's `proofRequired` into `identityMismatch` — a different
//! failure, equally fatal.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p trust-tasks-https --example server_demo
//! ```
//!
//! Then point [`client_demo`] at it.

use affinidi_secrets_resolver::secrets::Secret;
use trust_tasks_https::{BearerAuth, HttpsServer};
use trust_tasks_proof::affinidi::Verifier;
use trust_tasks_rs::{
    specs::acl::{grant, revoke},
    RejectReason,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_, alice_did) = demo_identity();
    let auth = BearerAuth::from_pairs([
        ("alice", alice_did.as_str()),
        ("bob", "did:web:bob.example"),
    ]);

    let server = HttpsServer::builder()
        .local_vid("did:web:maintainer.example")
        .with_auth(auth)
        // Without a verifier the server only checks that a proof is
        // *present*. `for_did_key()` resolves offline, so the demo
        // exercises real verification with no network and no DID document.
        .with_verifier(Verifier::for_did_key())
        // Handler for acl/grant: accepts, echoes the entry back as the
        // canonical post-state. Demonstrates the typed payload + context.
        .on::<grant::v0_1::Payload, grant::v0_1::Response, _>(|req, ctx| {
            println!(
                "acl/grant received from {} for subject {} (role {})",
                ctx.authenticated_sender.as_deref().unwrap_or("<unauth>"),
                req.payload.entry.subject,
                &*req.payload.entry.role,
            );
            Ok(grant::v0_1::Response {
                entry: req.payload.entry.clone(),
                ext: None,
            })
        })
        // Handler for acl/revoke: returns null entry meaning "removed".
        // Demonstrates RejectReason as the error path.
        .on::<revoke::v0_1::Payload, revoke::v0_1::Response, _>(|req, ctx| {
            println!(
                "acl/revoke received from {} for subject {}",
                ctx.authenticated_sender.as_deref().unwrap_or("<unauth>"),
                req.payload.subject,
            );
            if !ctx
                .authenticated_sender
                .as_deref()
                .map(|s| s.starts_with("did:web:"))
                .unwrap_or(false)
            {
                return Err(RejectReason::PermissionDenied {
                    reason: "revoke requires an authenticated did:web sender".into(),
                });
            }
            Ok(revoke::v0_1::Response {
                entry: None,
                ext: None,
            })
        })
        .build();

    println!("Trust Tasks server listening on http://127.0.0.1:3000/trust-tasks");
    server.serve("127.0.0.1:3000").await?;
    Ok(())
}

/// The demo client's identity, derived from the same fixed seed
/// `client_demo` uses, so the two agree without exchanging a key.
///
/// Duplicated rather than shared because each example is its own binary;
/// keep the seed in step with `client_demo::demo_identity`.
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
