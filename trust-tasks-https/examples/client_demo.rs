//! A demo Trust Tasks client over HTTPS.
//!
//! Sends an `acl/grant/0.1` request to a server (default
//! `http://localhost:3000`, override via `TRUST_TASKS_URL`) and prints
//! the response. Pair with the `server_demo` example for end-to-end
//! exercise.
//!
//! Run with:
//!
//! ```sh
//! # in one terminal:
//! cargo run -p trust-tasks-https --example server_demo
//! # in another terminal:
//! cargo run -p trust-tasks-https --example client_demo
//! ```

use trust_tasks_https::{ClientError, HttpsClient};
use trust_tasks_rs::{specs::acl::grant::v0_1 as grant, TrustTask};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("TRUST_TASKS_URL").unwrap_or_else(|_| "http://localhost:3000".into());

    let client = HttpsClient::builder()
        .server_url(&url)
        .server_vid("did:web:maintainer.example")
        .my_vid("did:web:alice.example")
        .my_token("alice")
        .build()?;

    let request = TrustTask::for_payload(
        format!("urn:uuid:{}", uuid::Uuid::new_v4()),
        grant::Payload {
            entry: grant::AclEntry {
                subject: "did:web:carol.example".into(),
                role: "moderator".into(),
                scopes: vec![],
                label: Some("Carol — content moderation".into()),
                created_at: None,
                created_by: None,
                updated_at: None,
                updated_by: None,
                expires_at: None,
                step_up: None,
                ext: None,
            },
            reason: Some("onboarding moderator".into()),
            ext: None,
        },
    );

    println!(
        "→ POST {}/trust-tasks\n  request id: {}\n  type: {}",
        url, request.id, request.type_uri
    );

    match client
        .send::<grant::Payload, grant::Response>(request)
        .await
    {
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
