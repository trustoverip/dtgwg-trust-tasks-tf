//! The HTTPS-binding [`HttpsHandler`].
//!
//! Per SPEC.md §9.2, each transport binding exposes its identity-resolution
//! and outbound-preparation logic as a [`TransportHandler`]. The HTTPS
//! handler reports the authenticated peer (from the bearer token, on the
//! server side; from the configured server VID, on the client side) and
//! lets the framework apply the §4.8.1 precedence rule unchanged.

use trust_tasks_rs::{TransportContext, TransportHandler};

/// Stable identifier for the HTTPS binding, per SPEC.md §9.2.
pub const BINDING_URI: &str = "https://trusttasks.org/binding/https/0.1";

/// A [`TransportHandler`] for a single HTTPS exchange.
///
/// Constructed per-request on the server (`local` = the server's own VID,
/// `peer` = the bearer-authenticated sender) and per-instance on the
/// client (`local` = the client's own VID, `peer` = the configured server
/// VID).
///
/// Either side can be `None`:
///
/// * `local = None` — the consumer chooses not to assert its identity to
///   the framework's audience-binding rules. Allowed for transports that
///   handle recipient-identity out-of-band.
/// * `peer = None` — the transport authenticated nothing (no bearer header,
///   or an unrecognised token). The framework then falls back entirely to
///   the document's in-band `issuer` plus any `proof` it carries.
#[derive(Debug, Clone)]
pub struct HttpsHandler {
    local: Option<String>,
    peer: Option<String>,
}

impl HttpsHandler {
    /// Construct a handler for an exchange where the local party is
    /// `local` and the transport authenticated `peer` as the counterpart.
    pub fn new(local: impl Into<Option<String>>, peer: impl Into<Option<String>>) -> Self {
        Self {
            local: local.into(),
            peer: peer.into(),
        }
    }

    /// The local party's VID, if configured.
    pub fn local(&self) -> Option<&str> {
        self.local.as_deref()
    }

    /// The authenticated peer's VID, if any.
    pub fn peer(&self) -> Option<&str> {
        self.peer.as_deref()
    }
}

impl TransportHandler for HttpsHandler {
    fn binding_uri(&self) -> &str {
        BINDING_URI
    }

    fn derive_parties(&self) -> TransportContext {
        // From the local party's perspective: the transport-authenticated
        // sender of the document is the *peer*; the transport-authenticated
        // addressee is the *local* party.
        TransportContext {
            issuer: self.peer.clone(),
            recipient: self.local.clone(),
        }
    }
}
