//! DIDComm **v1** transport binding for the Trust Tasks framework.
//!
//! The framework's other DIDComm binding targets v2.1. Credo — and therefore
//! essentially every Aries-lineage wallet — speaks v1 and only v1, so without
//! this there is no way for those stacks to carry a Trust Task at all.
//!
//! Built on [`affinidi_messaging_didcomm_v1`], which supplies the transport
//! primitives; framework semantics stay in [`trust_tasks_rs`]. This crate is the
//! seam between them, and it is deliberately thin — the same shape as
//! `trust-tasks-didcomm` for v2.1, so an implementation can drive both.
//!
//! # What differs from v2.1
//!
//! Two things, both consequences of the wire format rather than choices.
//!
//! **Attribution takes a step more.** A v2.1 envelope authenticates a DID: the
//! verified `sender_kid` reduces to one. A v1 envelope carries no DID at all —
//! it authenticates a bare base58 Ed25519 verkey, and the verkey-to-DID binding
//! is connection state. So there is a third failure mode with no v2.1
//! counterpart: an envelope that is cryptographically sound but attributable to
//! nobody this agent knows. See [`DidcommV1Error::UnknownSenderBinding`].
//!
//! **The document needs somewhere to live.** v2.1 puts it in the message
//! `body`; v1 has no `body`. This binding uses an `~attach` decorator — see
//! [`pack`] for the alternatives and why not.
//!
//! # Thread mapping
//!
//! Per SPEC.md §9.1, a binding that maps its transport's correlation
//! identifiers onto the framework's must say so:
//!
//! | Framework member | `~thread` field |
//! |---|---|
//! | `threadId` | `thid` |
//! | `parentThreadId` | `pthid` |
//!
//! Producers populate the decorator *from* the members. Consumers compare only
//! where **both** are explicitly present, and a disagreement is
//! `malformedRequest` — not `identityMismatch`, which is reserved for a
//! contested party identity. v1's `thid` defaults to the message `@id` and the
//! framework's `threadId` falls back to the document's `id`; those are
//! different identifiers, so an unconditional rule would reject exchanges that
//! conform on both layers.
//!
//! # Status
//!
//! Implements [`bindings/didcomm-v1/0.1`](https://trusttasks.org/bindings/didcomm-v1/0.1),
//! itself a draft written from this crate and offered to the DTG Core
//! Credentials task force to take over. The carriage in [`pack`] is flagged
//! open in §2 of that binding: nothing depends on it yet, so it can still move.
//! Discussion on issue #173.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod handler;
pub mod pack;

pub use error::DidcommV1Error;
pub use handler::{DidcommV1Handler, BINDING_URI};
pub use pack::{build_message, unpack_trust_task, ATTACHMENT_ID};

/// The `~thread` fields this binding maps onto the framework's thread members,
/// documented as data so a consumer can assert the mapping rather than restate
/// it. See the [module docs](self#thread-mapping).
pub const THREAD_MAPPING: [(&str, &str); 2] = [("threadId", "thid"), ("parentThreadId", "pthid")];
