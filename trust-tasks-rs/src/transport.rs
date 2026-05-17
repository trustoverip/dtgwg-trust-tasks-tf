//! [`TransportHandler`] trait — the integration point between the framework's
//! transport-agnostic document model and a concrete transport binding (REST,
//! DIDComm, AMQP, ...).
//!
//! The contract encodes SPEC.md §4.8.1 and §9.2:
//!
//! - In-band `issuer` / `recipient` values are authoritative when present.
//! - Transport-derived identity fills in absent members.
//! - When both are present, they MUST be consistent — a mismatch is a
//!   validation failure (the standard error code is `identity_mismatch`,
//!   §8.3).
//!
//! A handler implementation supplies the *transport-derived* values it
//! observed for an inbound message via [`TransportHandler::derive_parties`];
//! the [`TransportHandler::resolve_parties`] default method applies the
//! precedence rule to produce a single [`ResolvedParties`] value that the
//! consumer applies for every subsequent framework rule.

use serde::Serialize;
use thiserror::Error;

use crate::document::{ErrorResponse, TrustTask};
use crate::error::RejectReason;

/// The result of resolving a document's in-band party identity against the
/// transport context.
///
/// Per SPEC.md §4.8.1, these are the values a consumer applies for every
/// subsequent framework rule that references the issuer or recipient.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvedParties {
    /// The party responsible for the document's content, after precedence.
    pub issuer: Option<String>,
    /// The party expected to act upon the document, after precedence.
    pub recipient: Option<String>,
}

/// What the transport handler observed about an inbound message's parties.
///
/// Returned by [`TransportHandler::derive_parties`]. A field is `None` when
/// the transport does not authenticate that party — for example, a plain
/// HTTPS handler with no client certificate sets `issuer = None`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TransportContext {
    /// Transport-authenticated sender identity, if any.
    pub issuer: Option<String>,
    /// Transport-authenticated recipient identity, if any.
    pub recipient: Option<String>,
}

/// Errors raised when in-band and transport-derived identity disagree
/// (SPEC.md §7.2 item 6; standard code `identity_mismatch`, §8.3).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ConsistencyError {
    /// The in-band `issuer` does not match the transport-authenticated sender.
    #[error("in-band issuer {in_band:?} does not match transport-derived {transport:?}")]
    IssuerMismatch {
        /// Value carried in the document's `issuer` member.
        in_band: String,
        /// Value derived from the transport.
        transport: String,
    },
    /// The in-band `recipient` does not match the transport-authenticated
    /// addressee.
    #[error("in-band recipient {in_band:?} does not match transport-derived {transport:?}")]
    RecipientMismatch {
        /// Value carried in the document's `recipient` member.
        in_band: String,
        /// Value derived from the transport.
        transport: String,
    },
}

/// A transport binding's plug-in for the framework.
///
/// Implementations represent one transport (REST + mTLS, DIDComm, TSP, an
/// in-memory test loopback, ...) and supply the transport-derived information
/// the framework needs to apply SPEC.md §4.8.1 precedence.
///
/// The trait is sync and object-safe; transports that perform I/O do it
/// outside the trait (e.g. before constructing the handler with the values it
/// already extracted from a verified envelope).
pub trait TransportHandler {
    /// A stable identifier for this transport binding, suitable for logs and
    /// audit (SPEC.md §9.1, §9.2 — "A *transport binding* specification
    /// SHOULD identify itself by a stable URI").
    fn binding_uri(&self) -> &str;

    /// Identities the transport authenticated for the inbound message under
    /// consideration.
    ///
    /// Implementations construct the handler with whatever the transport
    /// extracted (peer certificate subject, verified DIDComm sender,
    /// authenticated routing key, ...) and return it here. Returning a
    /// fully-empty [`TransportContext`] is valid for transports that provide
    /// no authenticated identity — the framework then falls back entirely to
    /// the in-band members and any `proof` they carry.
    fn derive_parties(&self) -> TransportContext;

    /// Prepare an outbound document for emission over this transport.
    ///
    /// Per SPEC.md §9.2 item 1, a producer-side handler MAY strip `issuer` /
    /// `recipient` when the transport will provide authenticated identity
    /// for those roles end-to-end. The default implementation leaves the
    /// document untouched, which is the conservative choice (in-band values
    /// remain visible to downstream consumers regardless of transport).
    fn prepare_outbound<P: Serialize>(&self, _doc: &mut TrustTask<P>) {}

    /// Apply SPEC.md §4.8.1 precedence to produce the final
    /// [`ResolvedParties`] for an inbound document.
    ///
    /// Returns [`ConsistencyError`] when an in-band member is present and
    /// disagrees with the transport-derived value for the same party. The
    /// caller is responsible for translating the error into an
    /// `identity_mismatch` error response (SPEC.md §8.3).
    fn resolve_parties<P>(&self, doc: &TrustTask<P>) -> Result<ResolvedParties, ConsistencyError> {
        let ctx = self.derive_parties();

        let issuer = match (doc.issuer.as_deref(), ctx.issuer.as_deref()) {
            (Some(in_band), Some(transport)) if in_band != transport => {
                return Err(ConsistencyError::IssuerMismatch {
                    in_band: in_band.to_string(),
                    transport: transport.to_string(),
                });
            }
            (Some(in_band), _) => Some(in_band.to_string()),
            (None, Some(transport)) => Some(transport.to_string()),
            (None, None) => None,
        };

        let recipient = match (doc.recipient.as_deref(), ctx.recipient.as_deref()) {
            (Some(in_band), Some(transport)) if in_band != transport => {
                return Err(ConsistencyError::RecipientMismatch {
                    in_band: in_band.to_string(),
                    transport: transport.to_string(),
                });
            }
            (Some(in_band), _) => Some(in_band.to_string()),
            (None, Some(transport)) => Some(transport.to_string()),
            (None, None) => None,
        };

        Ok(ResolvedParties { issuer, recipient })
    }

    /// Build a `trust-task-error/0.1` response for `doc` that satisfies the
    /// SPEC.md §8.1 routing rules — most importantly, the rule that under
    /// [`RejectReason::IdentityMismatch`] the error MUST address the
    /// transport-authenticated sender and MUST NOT address the contested
    /// in-band issuer.
    ///
    /// Returns `Some(ErrorResponse)` when a routable recipient exists, and
    /// `None` when the rejection is `identity_mismatch` and the transport
    /// has no authenticated sender for the inbound document — per §8.1, the
    /// consumer SHOULD NOT emit a response in that case.
    fn reject<P>(
        &self,
        doc: &TrustTask<P>,
        id: impl Into<String>,
        reason: RejectReason,
    ) -> Option<ErrorResponse> {
        let recipient = match &reason {
            RejectReason::IdentityMismatch(_) => {
                // §8.1: transport-authenticated sender only. If the
                // transport authenticated nothing, the consumer suppresses
                // the response.
                let from_transport = self.derive_parties().issuer;
                from_transport.as_ref()?;
                from_transport
            }
            _ => doc.issuer.clone(),
        };
        Some(doc.reject_with_recipient(id, reason, recipient))
    }
}
