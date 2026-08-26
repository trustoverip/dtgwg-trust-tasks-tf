//! DIDComm v2.1 transport binding for the Trust Tasks framework.
//!
//! Wraps `affinidi-messaging-didcomm` so Trust Task documents can ride
//! inside an authcrypt'd DIDComm `Message` and survive any DIDComm-aware
//! transport (mediator pickup, raw HTTPS POST, message queue, paper
//! handoff for that matter).
//!
//! Authcrypt is the only inbound shape this binding accepts. Binding §2
//! makes it a **MUST** and §4 keeps anoncrypt, plaintext, and bare-JWS
//! envelopes out of the framework pipeline entirely: none of them yields
//! the transport-authenticated sender SPEC.md §4.8.1 needs, and a bare
//! JWS additionally has no recipient binding, so one signed message can
//! be delivered to every party in a deployment and each would accept it.
//!
//! ## Binding URI
//!
//! `https://trusttasks.org/binding/didcomm/0.2`
//!
//! The envelope `type` below deliberately did **not** change with the
//! binding's `0.1` → `0.2` minor (binding §1, §7.1) — only the binding
//! identifier did.
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
//! encrypted to the recipient) before transmission. The authcrypt'd
//! `UnpackResult::Encrypted` carries a verified `sender_kid` (a DID URL
//! with a key fragment); the binding strips the fragment and uses the
//! DID as the framework's transport-authenticated `issuer` for SPEC.md
//! §4.8.1 precedence. A `sender_kid` carrying no fragment is an error —
//! see [`DidcommError::UnqualifiedSenderKid`].
//!
//! ## Accepting from many senders
//!
//! A consumer that receives from more than one peer declares a
//! [`SenderAllowlist`] and calls [`unpack_trust_task_from`], which reads
//! the envelope's `skid`, looks the sender up once, and decrypts once.
//!
//! ## The guarded inbound path
//!
//! [`unpack_trust_task`] opens the envelope and stops there. The consumer
//! obligations of SPEC.md §7.2 still have to run over the document it
//! returns, and two of them are stateful: item 11's duplicate-execution
//! record and the freshness bound that lets that record be dropped.
//!
//! [`DidcommConsumer`] is that path with both already wired, **on by
//! default**. It matters most here: §6 of the binding records that this
//! transport guarantees no freshness and that a mediator may re-deliver, so an
//! ordinary queue retry — no attacker involved — grants an ACL entry twice
//! unless the consumer keeps the record. The record is keyed on the document
//! `id` and never on the DIDComm `@id` or `thid`, because a redelivery
//! legitimately carries a fresh transport identifier for the same document.
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
pub mod inbound;
mod pack;

pub use error::DidcommError;
pub use handler::{DidcommHandler, BINDING_URI};
pub use inbound::DidcommConsumer;
pub use pack::{
    advertised_sender_did, pack_trust_task, unpack_trust_task, unpack_trust_task_from,
    SenderAllowlist, ENVELOPE_TYPE,
};
