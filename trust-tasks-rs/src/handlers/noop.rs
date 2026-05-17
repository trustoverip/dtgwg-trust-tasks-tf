use crate::transport::{TransportContext, TransportHandler};

/// A transport handler that contributes no identity information.
///
/// Useful for transports that authenticate nothing (e.g. unauthenticated
/// HTTP, an offline file hand-off) — the framework then relies entirely on
/// the document's in-band `issuer`, `recipient`, and any `proof` they carry.
///
/// Modelled after the `unauthenticated transport handler` example from
/// SPEC.md §9.2.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopHandler;

impl NoopHandler {
    /// Construct a [`NoopHandler`].
    pub fn new() -> Self {
        Self
    }
}

impl TransportHandler for NoopHandler {
    fn binding_uri(&self) -> &str {
        "urn:trust-tasks:binding:noop"
    }

    fn derive_parties(&self) -> TransportContext {
        TransportContext::default()
    }
}
