//! ToIP Trust Spanning Protocol (TSP) transport binding for the Trust Tasks
//! framework.
//!
//! Wraps [`affinidi-tsp`](https://crates.io/crates/affinidi-tsp) so a Trust Task
//! document can ride as the authenticated, encrypted payload of a TSP message —
//! sealed with HPKE authenticated encryption and signed from the producer's
//! *Verifiable Identifier* (VID) to the recipient's VID. On unwrap, the
//! authenticated sender VID becomes the framework's transport-authenticated
//! sender for [SPEC §4.8.1] precedence.
//!
//! ## Binding URI
//!
//! `https://trusttasks.org/binding/tsp/0.1`
//!
//! ## Wire shape
//!
//! The TSP message **payload** (the plaintext TSP seals) is the JSON
//! serialisation of the binding *envelope object*:
//!
//! ```json
//! {
//!   "type": "https://trusttasks.org/binding/tsp/0.1/envelope",
//!   "document": { /* the full TrustTask<P> */ }
//! }
//! ```
//!
//! Unlike the DIDComm binding — which normalises a `sender_kid` to its bare DID —
//! a TSP VID *is* the framework VID, so no transformation is applied: the
//! authenticated `VID_sndr` is surfaced verbatim as the transport-authenticated
//! `issuer`. TSP has no anonymous/unauthenticated sender mode, so every envelope
//! this binding accepts carries a verified sender.
//!
//! This first release covers **Direct** carriage (a TSP message sealed straight
//! from producer to consumer). Routed and Nested carriage ([SPEC binding §5]) —
//! where the mediator relays the sealed message — are handled by the messaging
//! mediator on the wire; the consumer opens the innermost Direct message, which
//! this binding unpacks.
//!
//! ## Sketch
//!
//! ```rust,ignore
//! use affinidi_tsp::PrivateVid;
//! use trust_tasks_tsp::{pack_trust_task, unpack_trust_task};
//!
//! // alice (producer) seals a document for bob:
//! let wire = pack_trust_task(&doc, &alice_private, &bob_resolved)?;
//!
//! // bob (consumer) opens it (alice's VID resolved for verification):
//! let (doc, handler) = unpack_trust_task::<MyPayload>(&wire, &bob_private, &alice_resolved)?;
//! ```
//!
//! [SPEC §4.8.1]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity
//! [SPEC binding §5]: ../../bindings/tsp/0.1/spec.md

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod error;
mod handler;
mod pack;

pub use error::TspError;
pub use handler::{BINDING_URI, TspHandler};
pub use pack::{ENVELOPE_TYPE, pack_trust_task, unpack_trust_task};
