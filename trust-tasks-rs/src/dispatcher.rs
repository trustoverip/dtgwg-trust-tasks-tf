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

use serde_json::Value;

use crate::document::TrustTask;
use crate::error::RejectReason;
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
