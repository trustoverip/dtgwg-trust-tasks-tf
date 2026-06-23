//! [`TspHandler`] — the framework's [`TransportHandler`] for the TSP binding.
//!
//! Constructed per-exchange by [`unpack_trust_task`](crate::unpack_trust_task)
//! (one per inbound message). The handler reports the locally-controlled VID as
//! `recipient` and the TSP-authenticated peer VID as `issuer`, then lets the
//! framework's default [`TransportHandler::resolve_parties`] apply SPEC §4.8.1
//! precedence unchanged. Because a TSP VID *is* a framework VID, no
//! normalisation is applied — comparison is exact string equality.

use trust_tasks_rs::{TransportContext, TransportHandler};

/// Stable identifier for the TSP binding, per SPEC §9.3.
pub const BINDING_URI: &str = "https://trusttasks.org/binding/tsp/0.1";

/// A [`TransportHandler`] for one TSP exchange.
///
/// `local` is the VID this party controls (the `VID_rcvr` it unwrapped the
/// message for). `peer` is the TSP-authenticated sender VID (`VID_sndr`). Both
/// are `Option<String>` to satisfy the trait's shape, but for a successfully
/// unpacked TSP message both are always `Some` — TSP has no unauthenticated
/// sender mode.
#[derive(Debug, Clone)]
pub struct TspHandler {
    local: Option<String>,
    peer: Option<String>,
}

impl TspHandler {
    /// Construct a handler from the recipient and authenticated-sender VIDs.
    pub fn new(local: impl Into<Option<String>>, peer: impl Into<Option<String>>) -> Self {
        Self {
            local: local.into(),
            peer: peer.into(),
        }
    }

    /// The local party's VID, if set.
    pub fn local(&self) -> Option<&str> {
        self.local.as_deref()
    }

    /// The TSP-authenticated peer VID, if set.
    pub fn peer(&self) -> Option<&str> {
        self.peer.as_deref()
    }
}

impl TransportHandler for TspHandler {
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
