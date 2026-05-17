//! [`DidcommHandler`] — the framework's [`TransportHandler`] for DIDComm
//! v2.1.
//!
//! Constructed per-exchange (one per inbound message on the consumer
//! side; once per peer on the producer side). The handler reports the
//! locally-controlled DID as `recipient` and the DIDComm-verified peer
//! DID as `issuer`, then lets the framework's default
//! [`TransportHandler::resolve_parties`] apply SPEC.md §4.8.1
//! precedence unchanged.

use trust_tasks_rs::{TransportContext, TransportHandler};

/// Stable identifier for the DIDComm binding, per SPEC.md §9.2.
pub const BINDING_URI: &str = "https://trusttasks.org/binding/didcomm/0.1";

/// A [`TransportHandler`] for one DIDComm v2.1 exchange.
///
/// `local` is the DID this party controls. `peer` is the
/// authcrypt-verified sender DID on the consumer side, or the configured
/// remote DID on the producer side. Both fields are `Option<String>` so
/// the same struct can model anoncrypt'd (sender-unknown) flows by
/// leaving `peer = None` — the framework then falls back entirely to
/// the document's in-band `proof`.
#[derive(Debug, Clone)]
pub struct DidcommHandler {
    local: Option<String>,
    peer: Option<String>,
}

impl DidcommHandler {
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

    /// The verified peer DID, if set.
    pub fn peer(&self) -> Option<&str> {
        self.peer.as_deref()
    }
}

impl TransportHandler for DidcommHandler {
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
