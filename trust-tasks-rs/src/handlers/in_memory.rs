use serde::Serialize;

use crate::document::TrustTask;
use crate::transport::{TransportContext, TransportHandler};

/// A simulated transport that authenticates a fixed local-and-peer VID pair.
///
/// On the consumer side it reports the configured peer as the
/// transport-authenticated issuer and the configured local party as the
/// transport-authenticated recipient. On the producer side it strips
/// in-band members that match the configured local-as-issuer /
/// peer-as-recipient assignment — modelling the §9.2 item 1 behavior of a
/// transport that conveys authenticated identity end-to-end.
///
/// Use it for tests and examples that want to exercise the
/// [`TransportHandler`] contract without standing up a real binding.
#[derive(Debug, Clone, Default)]
pub struct InMemoryHandler {
    /// Authenticated identity of the party running this handler.
    pub local: Option<String>,
    /// Authenticated identity of the peer this handler is talking to.
    pub peer: Option<String>,
}

impl InMemoryHandler {
    /// Construct a handler with no configured parties (equivalent to
    /// [`NoopHandler`](crate::handlers::NoopHandler) for consumer-side
    /// resolution).
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the local party identity. Builder-style.
    pub fn with_local(mut self, vid: impl Into<String>) -> Self {
        self.local = Some(vid.into());
        self
    }

    /// Set the peer party identity. Builder-style.
    pub fn with_peer(mut self, vid: impl Into<String>) -> Self {
        self.peer = Some(vid.into());
        self
    }
}

impl TransportHandler for InMemoryHandler {
    fn binding_uri(&self) -> &str {
        "urn:trust-tasks:binding:in-memory"
    }

    fn derive_parties(&self) -> TransportContext {
        TransportContext {
            // The transport's authenticated sender, from this handler's
            // perspective, is the peer.
            issuer: self.peer.clone(),
            // The transport-authenticated addressee is the local party.
            recipient: self.local.clone(),
        }
    }

    fn prepare_outbound<P: Serialize>(&self, doc: &mut TrustTask<P>) {
        // Producer side: drop in-band `issuer` / `recipient` when the transport
        // will convey the same identities end-to-end (§9.2 item 1). When they
        // disagree we leave them alone — the document is the source of truth
        // for who the parties are, and a downstream consumer will detect the
        // mismatch via `resolve_parties`.
        if let (Some(local), Some(in_band)) = (self.local.as_deref(), doc.issuer.as_deref()) {
            if local == in_band {
                doc.issuer = None;
            }
        }
        if let (Some(peer), Some(in_band)) = (self.peer.as_deref(), doc.recipient.as_deref()) {
            if peer == in_band {
                doc.recipient = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_uri::TypeUri;

    fn doc(issuer: Option<&str>, recipient: Option<&str>) -> TrustTask<serde_json::Value> {
        let mut d = TrustTask::new(
            "id-1",
            TypeUri::canonical("kyc-handoff", 1, 0).unwrap(),
            serde_json::json!({}),
        );
        d.issuer = issuer.map(str::to_string);
        d.recipient = recipient.map(str::to_string);
        d
    }

    #[test]
    fn fills_in_absent_parties_from_transport() {
        let h = InMemoryHandler::new()
            .with_local("did:web:bank.example")
            .with_peer("did:web:verifier.example");

        let resolved = h.resolve_parties(&doc(None, None)).unwrap();
        assert_eq!(resolved.issuer.as_deref(), Some("did:web:verifier.example"));
        assert_eq!(resolved.recipient.as_deref(), Some("did:web:bank.example"));
    }

    #[test]
    fn in_band_wins_when_consistent() {
        let h = InMemoryHandler::new()
            .with_local("did:web:bank.example")
            .with_peer("did:web:verifier.example");

        let resolved = h
            .resolve_parties(&doc(
                Some("did:web:verifier.example"),
                Some("did:web:bank.example"),
            ))
            .unwrap();
        assert_eq!(resolved.issuer.as_deref(), Some("did:web:verifier.example"));
        assert_eq!(resolved.recipient.as_deref(), Some("did:web:bank.example"));
    }

    #[test]
    fn flags_issuer_mismatch() {
        let h = InMemoryHandler::new()
            .with_local("did:web:bank.example")
            .with_peer("did:web:verifier.example");

        let err = h
            .resolve_parties(&doc(Some("did:web:attacker.example"), None))
            .unwrap_err();
        assert!(matches!(
            err,
            crate::ConsistencyError::IssuerMismatch { .. }
        ));
    }

    #[test]
    fn prepare_outbound_strips_redundant_in_band() {
        let h = InMemoryHandler::new()
            .with_local("did:web:bank.example")
            .with_peer("did:web:verifier.example");

        let mut d = doc(
            Some("did:web:bank.example"),
            Some("did:web:verifier.example"),
        );
        h.prepare_outbound(&mut d);
        assert!(d.issuer.is_none());
        assert!(d.recipient.is_none());
    }

    #[test]
    fn prepare_outbound_keeps_in_band_when_disagreeing() {
        let h = InMemoryHandler::new()
            .with_local("did:web:bank.example")
            .with_peer("did:web:verifier.example");

        let mut d = doc(Some("did:web:forwarded.example"), None);
        h.prepare_outbound(&mut d);
        assert_eq!(d.issuer.as_deref(), Some("did:web:forwarded.example"));
    }
}
