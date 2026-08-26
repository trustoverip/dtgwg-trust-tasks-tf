# trust-tasks-https

HTTPS transport binding for the [Trust Tasks](https://trusttasks.org/) framework. A typed [`HttpsClient`](src/client.rs) (reqwest) and an axum-based [`HttpsServer`](src/server.rs) that together implement [SPEC.md §9](../SPEC.md#9-transport-bindings) over HTTP/1.1 with bearer-token identity authentication — enough for demos, mockups, and end-to-end testing without standing up mTLS or DIDComm.

## Binding URI

`https://trusttasks.org/binding/https/0.2`

## On the wire

```text
POST /trust-tasks HTTP/1.1
Host: example.com
Content-Type: application/json
Authorization: Bearer <token>

{
  "id": "...",
  "type": "https://trusttasks.org/spec/acl/grant/0.1",
  "issuer": "did:web:org.example",
  "recipient": "did:web:maintainer.example",
  "issuedAt": "2026-05-17T10:00:00Z",
  "payload": { ... }
}
```

The server maps `<token>` to a VID, runs the [`§7.2`](../SPEC.md#72-consumer-requirements) consumer-side validation pipeline (cross-checking the in-band `issuer` against the authenticated sender per [§4.8.1](../SPEC.md#481-precedence-of-in-band-over-transport-derived-identity)), and dispatches the document to the handler registered for its `type` URI. Success responses return as `#response`-variant documents inside an HTTP 200 body. Failures return as `trust-task-error` documents with the [status mapping documented in `status.rs`](src/status.rs).

## Security defaults

The binding specification is explicit that [it "does not permit `proof` to be omitted"](../bindings/https/0.2/spec.md): a bearer token authenticates *whoever presents it* to whatever terminates TLS, which is not the same party as whoever composed the document. The runtime enforces that:

| Default | Effect | Opt out with |
|---|---|---|
| `require_attribution = true` | A document arriving with **neither** a transport-authenticated peer **nor** a `proof` is rejected `proofRequired` before any handler runs. Without it, an unauthenticated POST asserting `"issuer": "did:web:victim"` reaches handlers with that string as the caller's identity. | `.require_attribution(false)` — dev and tests only |
| `replay_protection = true` | SPEC §7.2 item 11: the server keeps a duplicate-execution record keyed on the document `id`. A bit-for-bit resend is absorbed and answered with the first execution's result instead of executing again; a *different* document under a reused `id` is refused `idConflict`. The default record is an in-process `InMemoryReplayGuard` — **replicated deployments MUST supply a shared-store guard** via `.with_replay_guard(..)`, since two replicas each holding their own map would each accept the same document once. | `.replay_protection(false)` — dev and tests only |
| Freshness bound (`FreshnessPolicy::consequential`) | `issuedAt` is REQUIRED and a document older than a five-minute acceptance window is refused `expired`. This is not a second feature but the other half of the record above: SPEC §7.2 makes the acceptance window and the record's retention *the same bound*. | `.freshness(..)` |
| Discovery requires authentication | `enable_discovery()` answers an unauthenticated discoverer with `permissionDenied` rather than the full route table (SPEC §10). | `.public_discovery()` |
| Route lookup precedes proof verification | An unknown `type` is rejected before DID resolution, so the endpoint cannot be aimed at an arbitrary host by a stranger. | — |
| `Content-Type: application/json` required | A `text/plain` cross-origin simple POST is refused with 415 instead of reaching the pipeline. | — |
| Request timeout + concurrency limit | Slowloris and flood controls on the router. | `.request_timeout(..)`, `.max_concurrent_requests(..)` |
| Response binding on the client | `HttpsClient::send` checks the response's `threadId`, `type`, `issuer` and `recipient` against the request before returning it. | — |

A guard that cannot be consulted fails **closed**: the server answers `unavailable` (retryable) and does not execute, because a consumer that cannot establish whether a document is a duplicate has not satisfied item 11.

Optionally, `.allowed_did_methods(["key"])` pre-screens `proof.verificationMethod` before the verifier resolves it, and `HttpsClient::builder().with_response_verifier(..)` requires signed responses.

## Quickstart

Server:

```rust,ignore
use trust_tasks_https::{BearerAuth, HttpsServer};
use trust_tasks_rs::specs::acl::grant;

let server = HttpsServer::builder()
    .local_vid("did:web:maintainer.example")
    .with_auth(BearerAuth::from_pairs([
        ("alice", "did:web:alice.example"),
    ]))
    .on::<grant::v0_1::Payload, grant::v0_1::Response, _>(|req, _ctx| {
        Ok(grant::v0_1::Response { entry: req.payload.entry.clone() })
    })
    .build();

server.serve("127.0.0.1:3000").await?;
```

Client:

```rust,ignore
use trust_tasks_https::HttpsClient;
use trust_tasks_rs::{specs::acl::grant, TrustTask};

let client = HttpsClient::builder()
    .server_url("http://localhost:3000")
    .server_vid("did:web:maintainer.example")
    .my_vid("did:web:alice.example")
    .my_token("alice")
    .build()?;

let req = TrustTask::for_payload("urn:uuid:...", grant::v0_1::Payload { ... });
let resp: TrustTask<grant::v0_1::Response> = client.send(req).await?;
```

## Demo

A working end-to-end demo lives in `examples/`. In one terminal:

```sh
cargo run -p trust-tasks-https --example server_demo
```

In another:

```sh
cargo run -p trust-tasks-https --example client_demo
```

The client issues an `acl/grant` request as `did:web:alice.example` over the `alice` bearer token; the server's handler accepts it and the client prints the `#response` document.

## Cargo features

| Feature           | What it enables                         |
|-------------------|-----------------------------------------|
| `client` (default) | `HttpsClient` (pulls in `reqwest`)       |
| `server` (default) | `HttpsServer` (pulls in `axum`, `tokio`) |

A producer-only binary can disable `server`; a consumer-only binary can disable `client`.

## TLS

The crate's name says HTTPS but the server- and client-side examples use plain HTTP localhost for ergonomic local testing. For production use, terminate TLS in front of the axum server (a reverse proxy is the easiest path; `axum-server` with `rustls` works for native termination) and configure `reqwest` with the appropriate root-CA bundle. The framework's [§6.1 requirement](../SPEC.md#61-type-uri) that `type` URIs use `https` is independent of the transport you carry the document over.

## Security notes

The bearer-auth model in this crate is intentionally simple, intended for demos and end-to-end testing. Production deployments SHOULD plug in a real [`Auth`](src/auth.rs) implementation backed by a token-issuing identity provider, JWT verification, or peer-certificate validation. Several of the framework-level security properties (audience binding, identity-mismatch routing, retry semantics, error-message sanitisation) are enforced regardless of which `Auth` you use; see [`SPEC.md` §10](../SPEC.md#10-security-and-privacy-considerations) and [`trust-tasks-rs`](../trust-tasks-rs/README.md).
