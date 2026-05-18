//! Typed multi-spec dispatch.
//!
//! A consumer that handles N specs can register one [`Payload`] type per spec
//! and dispatch an inbound [`TrustTask<Value>`] to the matching handler. The
//! framework's open registry rules out a closed `enum TrustTaskPayload { … }`
//! because every new spec would be a breaking change; the [`Dispatcher`]
//! preserves the type-safe match while staying additive.
//!
//! ```rust,ignore
//! use trust_tasks_rs::{specs::acl, Dispatcher};
//!
//! let dispatcher = Dispatcher::new()
//!     .on::<acl::grant::v0_1::Payload, _, _>(|doc| handle_grant(doc))
//!     .on::<acl::revoke::v0_1::Payload, _, _>(|doc| handle_revoke(doc));
//!
//! let outcome = dispatcher.dispatch(inbound)?;
//! ```
//!
//! The dispatcher returns [`RejectReason::UnsupportedType`] when no handler
//! matches the inbound document's `type` URI, and
//! [`RejectReason::MalformedRequest`] when a registered URI is found but the
//! payload fails to deserialize into the corresponding `P`.

use std::collections::HashMap;

use chrono::Utc;
use serde_json::Value;

use crate::document::{trust_task_error_type_uri, ErrorResponse, TrustTask};
use crate::error::{ErrorPayload, RejectReason};
use crate::payload::Payload;

type BoxedHandler<R> = Box<dyn Fn(TrustTask<Value>) -> Result<R, RejectReason> + Send + Sync>;

/// Routes a [`TrustTask<Value>`] to the handler registered for its Type URI.
///
/// Construct with [`Dispatcher::new`], register handlers via [`Dispatcher::on`],
/// then call [`Dispatcher::dispatch`] for each inbound document. `R` is the
/// handler's return type — `()`, `Result<TrustTask<Resp>, ErrorResponse>`, or
/// anything else; the dispatcher only adds [`RejectReason`] for the routing-
/// and deserialization-time failures.
pub struct Dispatcher<R> {
    handlers: HashMap<String, BoxedHandler<R>>,
}

impl<R> Default for Dispatcher<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R> Dispatcher<R> {
    /// Build an empty [`Dispatcher`]. Add handlers with [`Self::on`].
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register `handler` for the Type URI declared by `P`.
    ///
    /// On dispatch, the dispatcher looks up the inbound document's `type`
    /// against the registered URIs **in canonical form** ([`TypeUri::for_routing`](crate::TypeUri::for_routing)),
    /// so a producer emitting either the bare URI or the `#request`-fragmented
    /// form per SPEC.md §4.4.1 item 1 routes to the same handler.
    /// `#response`-fragmented URIs are kept distinct and route to whatever
    /// (if anything) is registered for the response variant.
    ///
    /// Registering the same canonical Type URI twice replaces the earlier
    /// handler.
    pub fn on<P, F>(mut self, handler: F) -> Self
    where
        P: Payload + 'static,
        F: Fn(TrustTask<P>) -> R + Send + Sync + 'static,
    {
        let wrapped = move |doc: TrustTask<Value>| -> Result<R, RejectReason> {
            let typed = downcast_payload::<P>(doc)?;
            Ok(handler(typed))
        };
        self.handlers
            .insert(canonical_key(&P::type_uri()), Box::new(wrapped));
        self
    }

    /// Route `doc` to the handler registered for its `type` URI.
    ///
    /// Returns:
    ///
    /// * `Ok(R)` — handler invoked successfully.
    /// * `Err(RejectReason::UnsupportedType)` — no handler registered for
    ///   this Type URI; the caller typically converts this into a
    ///   `trust-task-error/0.1` response via [`TrustTask::reject_with`].
    /// * `Err(RejectReason::MalformedRequest)` — the URI matched but the
    ///   payload failed to deserialize against `P`.
    pub fn dispatch(&self, doc: TrustTask<Value>) -> Result<R, RejectReason> {
        let key = canonical_key(&doc.type_uri);
        match self.handlers.get(&key) {
            Some(handler) => handler(doc),
            None => Err(RejectReason::UnsupportedType { type_uri: key }),
        }
    }

    /// The Type URIs this dispatcher currently routes for, in canonical
    /// form and sorted for stable output. Handy for `unsupported_type`
    /// error responses that list what the consumer *does* implement.
    pub fn registered_uris(&self) -> Vec<&str> {
        let mut v: Vec<&str> = self.handlers.keys().map(String::as_str).collect();
        v.sort_unstable();
        v
    }

    /// Route `doc` to the registered handler, returning either the
    /// handler's success value or an [`ErrorResponse`] already routed
    /// per SPEC.md §8.1.
    ///
    /// Equivalent to calling [`Self::dispatch`] and hand-converting any
    /// [`RejectReason`] via [`TrustTask::reject_with`] — most consumer
    /// code today writes that conversion as a six-line `match`, so this
    /// is the convenience form.
    ///
    /// `error_id` supplies the `id` for the error response; the framework
    /// places no constraints on its form beyond uniqueness (SPEC.md §4.3).
    /// UUIDv4 is the recommended default.
    ///
    /// The dispatcher only emits [`RejectReason::UnsupportedType`] (no
    /// handler registered for this Type URI) and
    /// [`RejectReason::MalformedRequest`] (payload failed to deserialize
    /// into the registered `P`). Both are routed to the original producer
    /// per SPEC.md §8.1 — neither carries the `identity_mismatch`
    /// transport-routing exception, so the safe default in
    /// [`TrustTask::reject_with`] applies.
    ///
    /// `ErrorResponse` is intentionally large (it carries a full
    /// `trust-task-error/0.1` document including a typed payload and
    /// extra-member map). Boxing it in the `Err` variant would just push
    /// the allocation onto every caller; the convenience this method
    /// provides assumes the caller wants the value back unboxed.
    #[allow(clippy::result_large_err)]
    pub fn dispatch_or_reject(
        &self,
        doc: TrustTask<Value>,
        error_id: impl Into<String>,
    ) -> Result<R, ErrorResponse> {
        // §8.1 needs `id`, `threadId`, `issuer`, `recipient`, `type_uri`
        // to build the error response. The handler consumes `doc`, so we
        // capture the small bits up front.
        let id = doc.id.clone();
        let thread_id = doc.thread_id.clone();
        let issuer = doc.issuer.clone();
        let recipient = doc.recipient.clone();

        match self.dispatch(doc) {
            Ok(value) => Ok(value),
            Err(reason) => Err(build_error_response(
                error_id.into(),
                id,
                thread_id,
                issuer,
                recipient,
                ErrorPayload::from(reason),
            )),
        }
    }
}

/// Hand-build the §8.1-routed [`ErrorResponse`] for a routing-time
/// failure. Mirrors [`TrustTask::reject_with`] but does not require an
/// intact `TrustTask` — the dispatcher has already moved the inbound
/// document into the handler by the time we know we need an error
/// response, so we work from the metadata cloned beforehand.
fn build_error_response(
    error_id: String,
    request_id: String,
    request_thread_id: Option<String>,
    request_issuer: Option<String>,
    request_recipient: Option<String>,
    payload: ErrorPayload,
) -> ErrorResponse {
    let thread_id = request_thread_id.or(Some(request_id));
    ErrorResponse {
        id: error_id,
        thread_id,
        type_uri: trust_task_error_type_uri(),
        issuer: request_recipient,
        recipient: request_issuer,
        issued_at: Some(Utc::now()),
        expires_at: None,
        payload,
        context: None,
        proof: None,
        extra: Default::default(),
    }
}

fn canonical_key(uri: &crate::type_uri::TypeUri) -> String {
    uri.for_routing().to_string()
}

fn downcast_payload<P>(doc: TrustTask<Value>) -> Result<TrustTask<P>, RejectReason>
where
    P: Payload,
{
    let TrustTask {
        id,
        thread_id,
        type_uri,
        issuer,
        recipient,
        issued_at,
        expires_at,
        payload,
        context,
        proof,
        extra,
    } = doc;

    let payload: P =
        serde_json::from_value(payload).map_err(|e| RejectReason::MalformedRequest {
            reason: format!("payload does not match {}: {e}", P::TYPE_URI),
        })?;

    Ok(TrustTask {
        id,
        thread_id,
        type_uri,
        issuer,
        recipient,
        issued_at,
        expires_at,
        payload,
        context,
        proof,
        extra,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specs::acl::grant::v0_1 as grant;
    use crate::StandardCode;

    fn payload() -> grant::Payload {
        grant::Payload {
            entry: grant::AclEntry {
                subject: "did:web:alice.example".into(),
                role: "admin".into(),
                scopes: vec![],
                label: None,
                created_at: None,
                created_by: None,
                updated_at: None,
                updated_by: None,
                expires_at: None,
                ext: None,
            },
            reason: None,
            ext: None,
        }
    }

    #[test]
    fn dispatch_or_reject_unsupported_type_routes_back_to_original_issuer() {
        // Dispatcher has no handler at all; any inbound doc → unsupported_type.
        let dispatcher: Dispatcher<()> = Dispatcher::new();
        let mut doc = TrustTask::for_payload("req-9", payload());
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());
        let doc_as_value = serde_json::to_value(&doc).unwrap();
        let doc_as_value: TrustTask<Value> = serde_json::from_value(doc_as_value).unwrap();

        let err = dispatcher
            .dispatch_or_reject(doc_as_value, "err-9")
            .unwrap_err();

        // §8.1 routing: response addresses the original producer.
        assert_eq!(err.id, "err-9");
        assert_eq!(err.thread_id.as_deref(), Some("req-9"));
        assert_eq!(err.issuer.as_deref(), Some("did:web:maintainer.example"));
        assert_eq!(err.recipient.as_deref(), Some("did:web:org.example"));
        assert_eq!(err.payload.code, StandardCode::UnsupportedType.into());
    }

    #[test]
    fn dispatch_or_reject_passes_handler_value_through() {
        let dispatcher =
            Dispatcher::<String>::new().on::<grant::Payload, _>(|_doc| "handled".to_string());

        let mut doc = TrustTask::for_payload("req-10", payload());
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());
        let doc_as_value: TrustTask<Value> =
            serde_json::from_value(serde_json::to_value(&doc).unwrap()).unwrap();

        let outcome = dispatcher
            .dispatch_or_reject(doc_as_value, "err-10")
            .unwrap();

        assert_eq!(outcome, "handled");
    }
}
