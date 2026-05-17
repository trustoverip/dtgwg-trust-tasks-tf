//! DIDComm v2.1 transport binding for the Trust Tasks framework.
//!
//! Wraps `affinidi-messaging-didcomm` so Trust Task documents can ride
//! inside a DIDComm `Message`, get authcrypt'd or anoncrypt'd in a JWE,
//! and survive any DIDComm-aware transport (mediator pickup, raw HTTPS
//! POST, message queue, paper handoff for that matter).
//!
//! ## Binding URI
//!
//! `https://trusttasks.org/binding/didcomm/0.1`
//!
//! ## Wire shape
//!
//! Each Trust Task document is packed into a DIDComm v2.1 `Message`
//! whose `type` is the framework-reserved URI:
//!
//! ```text
//! https://trusttasks.org/binding/didcomm/0.1/envelope
//! ```
//!
//! The `body` of that DIDComm message is the full `TrustTask<P>` JSON.
//! The outer envelope is then authcrypt'd (sender-authenticated +
//! encrypted to the recipient) or anoncrypt'd (encrypted-only) before
//! transmission. The authcrypt'd `UnpackResult::Encrypted` carries a
//! verified `sender_kid` (a DID URL with a key fragment); the binding
//! strips the fragment and uses the DID as the framework's
//! transport-authenticated `issuer` for SPEC.md §4.8.1 precedence.
//!
//! ## Sketch
//!
//! ```rust,ignore
//! use affinidi_messaging_didcomm::{DIDCommAgent, identity::PrivateIdentity};
//! use trust_tasks_didcomm::{pack_trust_task, unpack_trust_task};
//!
//! // alice (producer):
//! let mut agent = DIDCommAgent::new();
//! agent.add_identity(alice.clone());
//! agent.add_peer(bob.to_resolved());
//! let wire = pack_trust_task(&doc, &agent, &alice.did, &bob.did)?;
//!
//! // bob (consumer):
//! let mut agent = DIDCommAgent::new();
//! agent.add_identity(bob.clone());
//! agent.add_peer(alice.to_resolved());
//! let (doc, handler) = unpack_trust_task::<MyPayload>(&wire, &agent)?;
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod error;
mod handler;
mod pack;

pub use error::DidcommError;
pub use handler::{DidcommHandler, BINDING_URI};
pub use pack::{pack_trust_task, unpack_trust_task, ENVELOPE_TYPE};
