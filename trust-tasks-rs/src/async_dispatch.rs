//! [`AsyncDispatcher`] — the async, context-carrying sibling of
//! [`Dispatcher`](crate::Dispatcher). Private module; the type is
//! re-exported at the crate root and carries the documentation.

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;

use serde_json::Value;

use crate::dispatcher::{build_error_response, canonical_key, downcast_payload, RequestOrigin};
use crate::document::{ErrorResponse, TrustTask};
use crate::error::{ErrorPayload, RejectReason};
use crate::payload::Payload;

/// The boxed future a registered handler produces. `'static` because the
/// handler and its future outlive the dispatch call — the same contract
/// every async router (axum, tower) imposes.
type HandlerFuture<R> = Pin<Box<dyn Future<Output = Result<R, RejectReason>> + Send>>;

type BoxedAsyncHandler<Ctx, R> =
    Box<dyn Fn(TrustTask<Value>, Ctx) -> HandlerFuture<R> + Send + Sync>;

/// Routes a [`TrustTask<Value>`] to an **async** handler registered for
/// its Type URI, carrying a caller-supplied context value alongside it.
///
/// The async, context-carrying sibling of [`Dispatcher`](crate::Dispatcher).
/// `Dispatcher::on` takes `Fn(TrustTask<P>) -> R`: synchronous, and with
/// nowhere to put the request-scoped state a real handler needs. A handler
/// that must `await` a database, a DID resolution or an approval prompt
/// cannot be written against it, which is why every receiver in the wild
/// hand-rolls its own router instead — and why none of them get
/// `unsupportedType` vs `unsupportedVersion` right.
///
/// `AsyncDispatcher` is a *sibling*, not a replacement: `Dispatcher` is
/// unchanged and stays the right tool for a synchronous match.
///
/// ```rust,ignore
/// use trust_tasks_rs::{specs::acl, AsyncDispatcher};
///
/// let dispatcher = AsyncDispatcher::<AppState, Outcome>::new()
///     .on_async::<acl::grant::v0_1::Payload, _, _>(|req, ctx| async move {
///         ctx.db.record_grant(&req.payload.entry).await;
///         Outcome::Granted
///     })
///     .on_async::<acl::revoke::v0_1::Payload, _, _>(|req, ctx| async move {
///         ctx.db.revoke(&req.payload.subject).await;
///         Outcome::Revoked
///     });
///
/// let outcome = dispatcher.dispatch(inbound, state.clone()).await?;
/// ```
///
/// # What falls out by construction
///
/// * **One downcast per message.** The `Value → P` conversion happens once,
///   inside the routing table lookup — not once per arm of an
///   `if doc.type_uri == type_uri_of::<P>()` chain that re-parses
///   `P::TYPE_URI` and re-serialises the document for every spec it does
///   not match.
/// * **`unsupportedType` vs `unsupportedVersion`.** Registering a handler
///   records the Type URI *and* its slug. A document whose slug is known
///   at a `MAJOR.MINOR` nobody registered is
///   [`RejectReason::UnsupportedVersion`] (SPEC §5.2 / §8.3); only an
///   unknown slug is [`RejectReason::UnsupportedType`]. A hand-rolled
///   `match` on the whole URI string cannot tell the two apart, so it
///   answers `unsupportedType` to a producer whose real problem is that it
///   needs to downgrade.
/// * **The typed §7.2 checks.** After the downcast — the first moment the
///   codegen-emitted `Payload` flags are reachable — the dispatcher applies
///   [`TrustTask::enforce_spec_policy`], covering §7.2 items 5b
///   (`recipient` REQUIRED), 7A (`proof` REQUIRED) and 8 (audience
///   binding). This is the same method
///   [`consume_inbound`](crate::consume_inbound) and the HTTPS server call,
///   so the three cannot diverge. It is applied to *request* documents
///   only: a `#response`-variant URI routes straight to its handler,
///   because those items govern what a consumer demands of an inbound
///   request.
///
///   The checks that do **not** need the payload type — expiry, recipient
///   match, the §4.8.1 transport cross-check, proof *verification*, the
///   §7.2 item 11 duplicate-execution record — are unchanged and still
///   belong to [`consume_inbound`](crate::consume_inbound), which a handler
///   can call around its own body.
///
/// # Shape
///
/// `Ctx` is whatever the caller wants each handler to receive alongside the
/// document: a connection pool, the transport's authenticated sender, a
/// request-scoped span. It is passed by value, so `Arc<AppState>` is the
/// usual choice. `R` is the handler's return type, uniform across handlers
/// exactly as it is for [`Dispatcher`](crate::Dispatcher) — commonly
/// `Result<TrustTask<Value>, ErrorResponse>` for a consumer that emits a
/// response document, or `()` for a fire-and-forget receiver.
///
/// The type is `Send + Sync` whatever `Ctx` and `R` are (handlers are held
/// behind `Send + Sync` trait objects), so it can live in an `Arc` on
/// shared state and be dispatched from any `tokio` task.
pub struct AsyncDispatcher<Ctx, R> {
    handlers: HashMap<String, BoxedAsyncHandler<Ctx, R>>,
    /// Slugs (version-independent) of every registered handler, so an
    /// inbound document whose slug is known but whose `MAJOR.MINOR` is not
    /// registered is `unsupportedVersion` rather than `unsupportedType`
    /// (SPEC §5.2 / §8.3).
    slugs: HashSet<String>,
}

impl<Ctx, R> Default for AsyncDispatcher<Ctx, R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Ctx, R> AsyncDispatcher<Ctx, R> {
    /// Build an empty [`AsyncDispatcher`]. Add handlers with
    /// [`Self::on_async`].
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            slugs: HashSet::new(),
        }
    }

    /// Register an async `handler` for the Type URI declared by `P`.
    ///
    /// The handler receives the downcast [`TrustTask<P>`] and the `Ctx`
    /// value supplied to [`Self::dispatch`], and returns any future
    /// resolving to `R` — so it can `await` whatever it needs to.
    ///
    /// Routing matches [`Dispatcher::on`](crate::Dispatcher::on): lookup is
    /// against the **canonical** form
    /// ([`TypeUri::for_routing`](crate::TypeUri::for_routing)), so a
    /// producer emitting either the bare URI or the `#request`-fragmented
    /// form per SPEC.md §4.4.1 item 1 reaches the same handler, while
    /// `#response`-fragmented URIs stay distinct. Registering the same
    /// canonical Type URI twice replaces the earlier handler.
    ///
    /// Before the handler runs, a request-variant document is checked with
    /// [`TrustTask::enforce_spec_policy`] — see the type-level docs.
    pub fn on_async<P, F, Fut>(mut self, handler: F) -> Self
    where
        P: Payload + 'static,
        F: Fn(TrustTask<P>, Ctx) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: Send + 'static,
    {
        let wrapped = move |doc: TrustTask<Value>, ctx: Ctx| -> HandlerFuture<R> {
            // The downcast is synchronous and happens exactly once, before
            // any future is created — so a routing or policy failure never
            // costs an allocation-and-poll cycle, and the handler's future
            // borrows nothing from the untyped document.
            let is_request = !doc.type_uri.is_response();
            let typed = match downcast_payload::<P>(doc) {
                Ok(typed) => typed,
                Err(reason) => return Box::pin(std::future::ready(Err(reason))),
            };
            // SPEC §7.2 items 5b + 7A + 8 — the flag-driven checks that only
            // exist once `P` is known. Same method as `consume_inbound` and
            // the HTTPS server, so the three cannot drift apart.
            if is_request {
                if let Err(reason) = typed.enforce_spec_policy() {
                    return Box::pin(std::future::ready(Err(reason)));
                }
            }
            let fut = handler(typed, ctx);
            Box::pin(async move { Ok(fut.await) })
        };
        self.slugs.insert(P::type_uri().slug().to_string());
        self.handlers
            .insert(canonical_key(&P::type_uri()), Box::new(wrapped));
        self
    }

    /// Route `doc` to the handler registered for its `type` URI, passing
    /// `ctx` alongside it, and await the result.
    ///
    /// Returns:
    ///
    /// * `Ok(R)` — handler invoked successfully.
    /// * `Err(RejectReason::UnsupportedVersion)` — the slug is registered
    ///   but not at this Type URI's `MAJOR.MINOR` (SPEC §5.2 / §8.3).
    /// * `Err(RejectReason::UnsupportedType)` — the slug is not registered
    ///   at all.
    /// * `Err(RejectReason::MalformedRequest)` — the URI matched but the
    ///   payload failed to deserialize against `P`.
    /// * any [`RejectReason`] [`TrustTask::enforce_spec_policy`] raises —
    ///   [`ProofRequired`](RejectReason::ProofRequired) most often.
    pub async fn dispatch(&self, doc: TrustTask<Value>, ctx: Ctx) -> Result<R, RejectReason> {
        let key = canonical_key(&doc.type_uri);
        match self.handlers.get(&key) {
            Some(handler) => handler(doc, ctx).await,
            // SPEC §5.2 / §8.3: a recognized slug at an unregistered
            // `MAJOR.MINOR` is `unsupportedVersion`; an unrecognized slug is
            // `unsupportedType`.
            None if self.slugs.contains(doc.type_uri.slug()) => {
                Err(RejectReason::UnsupportedVersion { type_uri: key })
            }
            None => Err(RejectReason::UnsupportedType { type_uri: key }),
        }
    }

    /// Route `doc` to the registered handler, returning either the
    /// handler's value or an [`ErrorResponse`] already routed per SPEC.md
    /// §8.1.
    ///
    /// The async mirror of
    /// [`Dispatcher::dispatch_or_reject`](crate::Dispatcher::dispatch_or_reject);
    /// see it for the routing rationale. `error_id` supplies the `id` for
    /// the error response — UUIDv4 is the recommended default (SPEC.md
    /// §4.3).
    ///
    /// None of the rejections this method can produce carry the
    /// `identity_mismatch` transport-routing exception, so the §8.1 safe
    /// default (address the original producer) applies to all of them.
    ///
    /// `ErrorResponse` is intentionally large; boxing it in the `Err`
    /// variant would push the allocation onto every caller.
    #[allow(clippy::result_large_err)]
    pub async fn dispatch_or_reject(
        &self,
        doc: TrustTask<Value>,
        ctx: Ctx,
        error_id: impl Into<String>,
    ) -> Result<R, ErrorResponse> {
        // §8.1 needs `id`, `threadId`, `issuer`, `recipient` and `type`, and
        // §4.9.2 needs `parentThreadId`, to build the error response. The
        // handler consumes `doc`, so capture the small bits up front.
        let origin = RequestOrigin {
            id: doc.id.clone(),
            thread_id: doc.thread_id.clone(),
            parent_thread_id: doc.parent_thread_id.clone(),
            ceremony: doc.ceremony.clone(),
            type_uri: doc.type_uri.to_string(),
            issuer: doc.issuer.clone(),
            recipient: doc.recipient.clone(),
        };

        match self.dispatch(doc, ctx).await {
            Ok(value) => Ok(value),
            Err(reason) => Err(build_error_response(
                error_id.into(),
                origin,
                ErrorPayload::from(reason),
            )),
        }
    }

    /// The Type URIs this dispatcher currently routes for, in canonical
    /// form and sorted for stable output. Handy for an `unsupportedType`
    /// error response that lists what the consumer *does* implement.
    pub fn registered_uris(&self) -> Vec<&str> {
        let mut v: Vec<&str> = self.handlers.keys().map(String::as_str).collect();
        v.sort_unstable();
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::specs::acl::grant::v0_1 as grant;
    use crate::StandardCode;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn payload() -> grant::Payload {
        grant::Payload {
            entry: grant::AclEntry {
                allowed_keys: None,
                subject: "did:web:alice.example".into(),
                role: "admin".into(),
                scopes: vec![],
                label: None,
                created_at: None,
                created_by: None,
                updated_at: None,
                updated_by: None,
                expires_at: None,
                approve: None,
                step_up: None,
                ext: None,
            },
            reason: None,
            ext: None,
        }
    }

    /// A well-formed inbound `acl/grant` in untyped form. `acl/grant/0.1`
    /// declares `proof` REQUIRED, so a document that is meant to reach a
    /// handler carries one; the framework only checks presence here
    /// (verification is `ProofVerifier`'s job).
    fn inbound(type_uri: &str, with_proof: bool) -> TrustTask<Value> {
        let mut doc = TrustTask::for_payload("req-1", payload());
        doc.issuer = Some("did:web:org.example".into());
        doc.recipient = Some("did:web:maintainer.example".into());
        if with_proof {
            doc.proof = Some(crate::Proof {
                proof_type: "DataIntegrityProof".into(),
                cryptosuite: "eddsa-jcs-2022".into(),
                verification_method: "did:web:org.example#key-1".into(),
                created: chrono::Utc::now(),
                proof_purpose: "assertionMethod".into(),
                proof_value: "z000".into(),
                extra: Default::default(),
            });
        }
        let mut value = serde_json::to_value(&doc).unwrap();
        value["type"] = Value::String(type_uri.to_string());
        serde_json::from_value(value).unwrap()
    }

    const GRANT_V0_1: &str = "https://trusttasks.org/spec/acl/grant/0.1";

    /// The headline: a handler that actually awaits, and a context value
    /// that reaches it.
    #[tokio::test]
    async fn async_handler_awaits_and_returns_a_response() {
        let calls = Arc::new(AtomicUsize::new(0));

        let dispatcher =
            AsyncDispatcher::<Arc<AtomicUsize>, TrustTask<grant::Response>>::new()
                .on_async::<grant::Payload, _, _>(|req, ctx: Arc<AtomicUsize>| async move {
                    // A real await point — stands in for the database round-trip
                    // the synchronous `Dispatcher` cannot express.
                    tokio::task::yield_now().await;
                    ctx.fetch_add(1, Ordering::SeqCst);
                    req.respond_with(
                        "resp-1",
                        grant::Response {
                            entry: req.payload.entry.clone(),
                            ext: None,
                        },
                    )
                });

        let response = dispatcher
            .dispatch(inbound(GRANT_V0_1, true), Arc::clone(&calls))
            .await
            .expect("handler ran");

        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "context reached the handler"
        );
        assert!(response.type_uri.is_response());
        assert_eq!(response.thread_id.as_deref(), Some("req-1"));
        assert_eq!(response.payload.entry.subject, "did:web:alice.example");
        // §8.1-style role reversal on the response document.
        assert_eq!(
            response.issuer.as_deref(),
            Some("did:web:maintainer.example")
        );
        assert_eq!(response.recipient.as_deref(), Some("did:web:org.example"));
    }

    /// An unregistered slug is `unsupportedType`.
    #[tokio::test]
    async fn unknown_slug_is_unsupported_type() {
        let dispatcher = AsyncDispatcher::<(), ()>::new()
            .on_async::<grant::Payload, _, _>(|_req, _ctx| async {});

        let doc = inbound("https://trusttasks.org/spec/never-heard-of-it/1.0", true);
        let err = dispatcher.dispatch(doc, ()).await.unwrap_err();

        assert!(
            matches!(err, RejectReason::UnsupportedType { .. }),
            "expected UnsupportedType, got {err:?}"
        );
    }

    /// A **known** slug at an unregistered `MAJOR.MINOR` is
    /// `unsupportedVersion` (SPEC §5.2 / §8.3) — the answer every
    /// hand-rolled router in the wild gets wrong, because a `match` on the
    /// whole URI string cannot see that the slug was recognised.
    #[tokio::test]
    async fn known_slug_at_unknown_version_is_unsupported_version() {
        let dispatcher = AsyncDispatcher::<(), ()>::new()
            .on_async::<grant::Payload, _, _>(|_req, _ctx| async {});

        let doc = inbound("https://trusttasks.org/spec/acl/grant/9.9", true);
        let err = dispatcher.dispatch(doc, ()).await.unwrap_err();

        assert!(
            matches!(err, RejectReason::UnsupportedVersion { .. }),
            "expected UnsupportedVersion, got {err:?}"
        );
    }

    /// And the distinction survives the §8.1 error-response builder, which
    /// is the form a transport binding actually emits.
    #[tokio::test]
    async fn dispatch_or_reject_carries_the_version_distinction_onto_the_wire() {
        let dispatcher = AsyncDispatcher::<(), ()>::new()
            .on_async::<grant::Payload, _, _>(|_req, _ctx| async {});

        let err = dispatcher
            .dispatch_or_reject(
                inbound("https://trusttasks.org/spec/acl/grant/9.9", true),
                (),
                "err-1",
            )
            .await
            .unwrap_err();

        assert_eq!(err.payload.code, StandardCode::UnsupportedVersion.into());
        // §8.1 routing: addressed back to the original producer.
        assert_eq!(err.id, "err-1");
        assert_eq!(err.thread_id.as_deref(), Some("req-1"));
        assert_eq!(err.issuer.as_deref(), Some("did:web:maintainer.example"));
        assert_eq!(err.recipient.as_deref(), Some("did:web:org.example"));

        let err = dispatcher
            .dispatch_or_reject(
                inbound("https://trusttasks.org/spec/nope/1.0", true),
                (),
                "err-2",
            )
            .await
            .unwrap_err();
        assert_eq!(err.payload.code, StandardCode::UnsupportedType.into());
    }

    /// `acl/grant/0.1` declares `proof` REQUIRED, so a proofless document
    /// is `proofRequired` — raised by the dispatcher itself, at the one
    /// point where the payload type's codegen flags are reachable. The
    /// handler never runs.
    #[tokio::test]
    async fn typed_spec_policy_runs_before_the_handler() {
        let calls = Arc::new(AtomicUsize::new(0));
        let dispatcher = AsyncDispatcher::<Arc<AtomicUsize>, ()>::new()
            .on_async::<grant::Payload, _, _>(|_req, ctx: Arc<AtomicUsize>| async move {
                ctx.fetch_add(1, Ordering::SeqCst);
            });

        let err = dispatcher
            .dispatch(inbound(GRANT_V0_1, false), Arc::clone(&calls))
            .await
            .unwrap_err();

        assert!(
            matches!(err, RejectReason::ProofRequired),
            "expected ProofRequired, got {err:?}"
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0, "handler must not have run");
    }

    /// A payload that does not deserialize into `P` is `malformedRequest`,
    /// not a panic — and the downcast happens once, not once per registered
    /// spec.
    #[tokio::test]
    async fn payload_that_does_not_match_is_malformed_request() {
        let dispatcher = AsyncDispatcher::<(), ()>::new()
            .on_async::<grant::Payload, _, _>(|_req, _ctx| async {});

        let mut doc = inbound(GRANT_V0_1, true);
        doc.payload = serde_json::json!({ "entry": "not an object" });

        let err = dispatcher.dispatch(doc, ()).await.unwrap_err();
        assert!(
            matches!(err, RejectReason::MalformedRequest { .. }),
            "expected MalformedRequest, got {err:?}"
        );
    }

    /// The bare and `#request`-fragmented forms of the same URI route to
    /// the same handler (SPEC §4.4.1 item 1), and `registered_uris` reports
    /// the canonical form.
    #[tokio::test]
    async fn request_fragment_and_bare_uri_route_together() {
        let dispatcher = AsyncDispatcher::<(), &'static str>::new()
            .on_async::<grant::Payload, _, _>(|_req, _ctx| async { "handled" });

        assert_eq!(dispatcher.registered_uris(), vec![GRANT_V0_1]);

        for uri in [
            GRANT_V0_1,
            "https://trusttasks.org/spec/acl/grant/0.1#request",
        ] {
            assert_eq!(
                dispatcher.dispatch(inbound(uri, true), ()).await.unwrap(),
                "handled",
                "{uri} should route to the registered handler"
            );
        }
    }

    /// Usable from a spawned `tokio` task — the property a receiver needs
    /// to hold the dispatcher on shared state.
    #[tokio::test]
    async fn dispatcher_is_shareable_across_tasks() {
        let dispatcher = Arc::new(
            AsyncDispatcher::<(), &'static str>::new()
                .on_async::<grant::Payload, _, _>(|_req, _ctx| async { "handled" }),
        );

        let d = Arc::clone(&dispatcher);
        let handle = tokio::spawn(async move { d.dispatch(inbound(GRANT_V0_1, true), ()).await });

        assert_eq!(handle.await.unwrap().unwrap(), "handled");
    }
}
