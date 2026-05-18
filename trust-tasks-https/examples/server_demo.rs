//! A demo Trust Tasks server over HTTPS.
//!
//! Exposes `acl/grant/0.1` and `acl/revoke/0.1` on `http://localhost:3000`.
//! Two bearer tokens are accepted: `alice` (maps to `did:web:alice.example`)
//! and `bob` (maps to `did:web:bob.example`); everything else is treated as
//! unauthenticated.
//!
//! Run with:
//!
//! ```sh
//! cargo run -p trust-tasks-https --example server_demo
//! ```
//!
//! Then point [`client_demo`] at it.

use trust_tasks_https::{BearerAuth, HttpsServer};
use trust_tasks_rs::{
    specs::acl::{grant, revoke},
    RejectReason,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let auth = BearerAuth::from_pairs([
        ("alice", "did:web:alice.example"),
        ("bob", "did:web:bob.example"),
    ]);

    let server = HttpsServer::builder()
        .local_vid("did:web:maintainer.example")
        .with_auth(auth)
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
