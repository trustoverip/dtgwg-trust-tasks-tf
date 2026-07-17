//! HTTPS transport binding for the Trust Tasks framework.
//!
//! Implements [SPEC.md §9](../../../SPEC.md#9-transport-bindings) over HTTP/1.1
//! using axum on the server side and reqwest on the client side. The binding
//! authenticates parties via an `Authorization: Bearer <token>` header that
//! maps to a Verifiable Identifier — sufficient for demos, mockups, and
//! end-to-end testing without standing up an mTLS or DIDComm stack.
//!
//! ## Binding URI
//!
//! `https://trusttasks.org/binding/https/0.1`
//!
//! ## On the wire
//!
//! ```text
//! POST /trust-tasks HTTP/1.1
//! Host: example.com
//! Content-Type: application/json
//! Authorization: Bearer <token>
//!
//! {
//!   "id": "...",
//!   "type": "https://trusttasks.org/spec/acl/grant/0.1",
//!   "issuer": "did:web:org.example",
//!   "recipient": "did:web:maintainer.example",
//!   "issuedAt": "2026-05-17T10:00:00Z",
//!   "payload": { ... }
//! }
//! ```
//!
//! The server maps the bearer token to a VID, runs the §7.2 / §4.8.1
//! consumer-side validation pipeline (cross-checking the in-band `issuer`
//! against the authenticated sender), and dispatches the document to the
//! handler registered for its `type` URI.
//!
//! ## Cargo features
//!
//! * `server` (default) — pulls in axum + tokio + tower; exposes
//!   [`HttpsServer`].
//! * `client` (default) — pulls in reqwest; exposes [`HttpsClient`].
//!
//! Disable one or the other if your binary only acts as a producer or only
//! as a consumer.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod auth;
mod error;
mod handler;
mod status;

#[cfg(feature = "client")]
#[cfg_attr(docsrs, doc(cfg(feature = "client")))]
mod client;

#[cfg(feature = "server")]
#[cfg_attr(docsrs, doc(cfg(feature = "server")))]
mod server;

pub use auth::{Auth, BearerAuth};

#[cfg(feature = "jwt")]
#[cfg_attr(docsrs, doc(cfg(feature = "jwt")))]
pub use auth::JwtBearerAuth;
pub use error::TransportError;
pub use handler::{HttpsHandler, BINDING_URI};
pub use status::status_for_code;

#[cfg(feature = "client")]
pub use client::{
    ClientError, HttpsClient, HttpsClientBuilder, DEFAULT_CONNECT_TIMEOUT, DEFAULT_TIMEOUT,
};

#[cfg(feature = "server")]
pub use server::{HttpsServer, HttpsServerBuilder, RequestContext};
