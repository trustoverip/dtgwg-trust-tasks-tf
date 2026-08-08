//! [`DidcommV1Handler`] — the framework's [`TransportHandler`] for DIDComm v1.
//!
//! Mirrors `trust-tasks-didcomm`'s handler for v2.1, and reaches the same
//! outcome by a different route. A v2.1 envelope authenticates a **DID**
//! directly: the verified `sender_kid` reduces to one. A v1 envelope contains no
//! DID at all — it authenticates a bare base58 Ed25519 **verkey**, and the
//! verkey-to-DID binding is connection state the agent holds, not something the
//! wire carries.
//!
//! So the transport-authenticated sender SPEC.md §4.8.1 needs is the
//! connection's `theirDid`, resolved by the agent before this handler is built.
//! Where the agent holds no binding for the authenticating verkey the envelope
//! is cryptographically sound but attributable to nobody, and this handler is
//! never constructed — see [`crate::unpack_trust_task`].

use trust_tasks_rs::{TransportContext, TransportHandler};

/// Stable identifier for the DIDComm v1 binding, per SPEC.md §9.2.
pub const BINDING_URI: &str = "https://trusttasks.org/binding/didcomm-v1/0.1";

/// A [`TransportHandler`] for one DIDComm v1 exchange.
///
/// `local` is the DID this party controls; `peer` is the connection's
/// `theirDid` — the DID bound to the verkey that authenticated the envelope.
/// Both are `Option<String>` to match the v2.1 handler's shape, but in practice
/// a consumer only builds one after [`crate::unpack_trust_task`] has established
/// an authenticated sender, so `peer` is populated on the inbound path.
#[derive(Debug, Clone)]
pub struct DidcommV1Handler {
    local: Option<String>,
    peer: Option<String>,
}

impl DidcommV1Handler {
    /// Construct a handler. Either side may be `None`.
    pub fn new(local: impl Into<Option<String>>, peer: impl Into<Option<String>>) -> Self {
        Self {
            local: local.into(),
            peer: peer.into(),
        }
    }

    /// The local party's DID, if set.
    pub fn local(&self) -> Option<&str> {
        self.local.as_deref()
    }

    /// The connection's `theirDid`, if set.
    pub fn peer(&self) -> Option<&str> {
        self.peer.as_deref()
    }
}

impl TransportHandler for DidcommV1Handler {
    fn binding_uri(&self) -> &str {
        BINDING_URI
    }

    fn derive_parties(&self) -> TransportContext {
        TransportContext {
            issuer: self.peer.clone(),
            recipient: self.local.clone(),
        }
    }
}
