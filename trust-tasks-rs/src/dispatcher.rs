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
//!     .on::<acl::grant::v0_2::Payload, _, _>(|doc| handle_grant(doc))
//!     .on::<acl::revoke::v0_2::Payload, _, _>(|doc| handle_revoke(doc));
//!
//! let outcome = dispatcher.dispatch(inbound)?;
//! ```
//!
//! A document of a version with no handler of its own, whose declared
//! extension chain (SPEC §5.2 "Declared extension") reaches a registered
//! version of the same slug, is routed to the highest such version with its
//! payload pruned to what that version knows — see [`crate::extension`].
//!
//! The dispatcher returns [`RejectReason::UnsupportedType`] when no handler
//! matches the inbound document's `type` URI, and
//! [`RejectReason::MalformedRequest`] when a registered URI is found but the
//! payload fails to deserialize into the corresponding `P`.

use std::collections::{HashMap, HashSet};

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
    /// The specifications (authority and slug, version-independent) of every
    /// registered handler, so an inbound document of a known specification
    /// whose `MAJOR.MINOR` is not registered can be distinguished as
    /// `unsupportedVersion` rather than `unsupportedType` (SPEC §5.2 / §8.3).
    /// Keyed by authority as well as slug: the same slug under another
    /// authority is another specification (SPEC §6.1), so it is
    /// `unsupportedType`.
    slugs: HashSet<String>,
    /// The payload schema of each registered handler's `P`, under the same
    /// key, for pruning a forward-routed document (SPEC §5.2).
    schemas: HashMap<String, Option<&'static str>>,
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
            slugs: HashSet::new(),
            schemas: HashMap::new(),
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
    ///
    /// The handler also receives documents of a **later** version that
    /// declares an extension chain reaching `P`'s (SPEC §5.2 "Declared
    /// extension") when that version has no handler of its own — see
    /// [`Self::dispatch`].
    pub fn on<P, F>(mut self, handler: F) -> Self
    where
        P: Payload + 'static,
        F: Fn(TrustTask<P>) -> R + Send + Sync + 'static,
    {
        let wrapped = move |doc: TrustTask<Value>| -> Result<R, RejectReason> {
            let typed = downcast_payload::<P>(doc)?;
            Ok(handler(typed))
        };
        self.slugs.insert(P::type_uri().specification_key());
        self.schemas
            .insert(canonical_key(&P::type_uri()), P::PAYLOAD_SCHEMA);
        self.handlers
            .insert(canonical_key(&P::type_uri()), Box::new(wrapped));
        self
    }

    /// Route `doc` to the handler registered for its `type` URI.
    ///
    /// Returns:
    ///
    /// * `Ok(R)` — handler invoked successfully.
    /// * `Err(RejectReason::UnsupportedVersion)` — the slug is registered but
    ///   not at this Type URI's `MAJOR.MINOR` (SPEC §5.2 / §8.3), and no
    ///   registered version is reached by its declared extension chain.
    /// * `Err(RejectReason::UnsupportedType)` — the slug is not registered at
    ///   all; the caller typically converts this into a `trust-task-error`
    ///   response via [`TrustTask::reject_with`].
    /// * `Err(RejectReason::MalformedRequest)` — the URI matched but the
    ///   payload failed to deserialize against `P`.
    ///
    /// # Declared extension (SPEC §5.2)
    ///
    /// When no handler is registered for the document's exact `MAJOR.MINOR`
    /// but its version's `extends` chain ([`crate::extension::extends_for`])
    /// reaches a registered version of the same slug, the document is routed
    /// to the **highest** such version. Its payload is first pruned to the
    /// members that version's schema knows
    /// ([`crate::extension::prune_payload`]); its `type` is left as received,
    /// so [`TrustTask::respond_with`] answers the producer's version. An exact
    /// registration always wins over a forward route.
    ///
    /// The dispatcher does not verify proofs, and the document it hands a
    /// handler may be pruned: verify the proof over the document as received
    /// *before* dispatching (as [`consume_inbound`](crate::consume_inbound)
    /// does with [`ProofPolicy::VerifyReceived`](crate::ProofPolicy::VerifyReceived)),
    /// never over the typed document a forward-routed handler receives.
    pub fn dispatch(&self, mut doc: TrustTask<Value>) -> Result<R, RejectReason> {
        let key = canonical_key(&doc.type_uri);
        if let Some(handler) = self.handlers.get(&key) {
            return handler(doc);
        }
        // SPEC §5.2 / §8.3: a recognized slug at an unregistered
        // `MAJOR.MINOR` is `unsupportedVersion` — unless the version declares
        // an extension of one that is registered; an unrecognized slug is
        // `unsupportedType`.
        if !self.slugs.contains(&doc.type_uri.specification_key()) {
            return Err(RejectReason::UnsupportedType { type_uri: key });
        }
        match forward_target(&doc.type_uri, &self.handlers) {
            Some(target) => {
                if let Some(schema) = self.schemas.get(&target).copied().flatten() {
                    crate::extension::prune_payload(&mut doc.payload, schema);
                }
                self.handlers[&target](doc)
            }
            None => Err(RejectReason::UnsupportedVersion { type_uri: key }),
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
    /// `trust-task-error` document including a typed payload and
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
        // to build the error response, and §4.9.2 needs `parentThreadId` so the
        // rejection stays inside the same enclosing exchange. The handler
        // consumes `doc`, so we capture the small bits up front.
        let id = doc.id.clone();
        let thread_id = doc.thread_id.clone();
        let parent_thread_id = doc.parent_thread_id.clone();
        let type_uri = doc.type_uri.to_string();
        let issuer = doc.issuer.clone();
        let recipient = doc.recipient.clone();
        let ceremony = doc.ceremony.clone();

        match self.dispatch(doc) {
            Ok(value) => Ok(value),
            Err(reason) => Err(build_error_response(
                error_id.into(),
                RequestOrigin {
                    id,
                    thread_id,
                    parent_thread_id,
                    ceremony,
                    type_uri,
                    issuer,
                    recipient,
                },
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
/// The parts of an inbound document a routing-time rejection needs, captured
/// before the handler consumes it.
pub(crate) struct RequestOrigin {
    pub(crate) id: String,
    pub(crate) thread_id: Option<String>,
    pub(crate) parent_thread_id: Option<String>,
    /// Carried onto the error response so a rejection stays inside the
    /// enactment it belongs to (SPEC.md §7.1).
    pub(crate) ceremony: Option<crate::Ceremony>,
    pub(crate) type_uri: String,
    pub(crate) issuer: Option<String>,
    pub(crate) recipient: Option<String>,
}

pub(crate) fn build_error_response(
    error_id: String,
    origin: RequestOrigin,
    mut payload: ErrorPayload,
) -> ErrorResponse {
    let RequestOrigin {
        id: request_id,
        thread_id: request_thread_id,
        parent_thread_id: request_parent_thread_id,
        ceremony: request_ceremony,
        type_uri: request_type_uri,
        issuer: request_issuer,
        recipient: request_recipient,
    } = origin;
    // §8.2 — see TrustTask::reject_with_recipient for why this is populated by
    // the builder rather than the caller, and why the id is withheld under
    // identityMismatch.
    if payload.in_response_to.is_none() {
        payload.in_response_to = Some(crate::InResponseTo {
            type_uri: request_type_uri,
            id: match &payload.code {
                crate::TrustTaskCode::Standard(crate::StandardCode::IdentityMismatch) => None,
                _ => Some(request_id.clone()),
            },
        });
    }
    let thread_id = request_thread_id.or(Some(request_id));
    ErrorResponse {
        id: error_id,
        thread_id,
        parent_thread_id: request_parent_thread_id,
        ceremony: request_ceremony,
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

pub(crate) fn canonical_key(uri: &crate::type_uri::TypeUri) -> String {
    uri.for_routing().to_string()
}

/// The registered key a document of `received`'s version forward-routes to:
/// the first — and so highest — link of its declared extension chain that has
/// a handler (SPEC §5.2).
pub(crate) fn forward_target<H>(
    received: &crate::type_uri::TypeUri,
    handlers: &HashMap<String, H>,
) -> Option<String> {
    crate::extension::chain(&received.for_routing())
        .iter()
        .map(canonical_key)
        .find(|k| handlers.contains_key(k))
}

pub(crate) fn downcast_payload<P>(doc: TrustTask<Value>) -> Result<TrustTask<P>, RejectReason>
where
    P: Payload,
{
    let TrustTask {
        id,
        thread_id,
        parent_thread_id,
        ceremony,
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
        parent_thread_id,
        ceremony,
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

#[cfg(all(test, feature = "acl"))] // its specimens are acl payloads
mod tests {
    use super::*;
    use crate::specs::acl::grant::v0_2 as grant;
    use crate::StandardCode;

    fn payload() -> grant::Payload {
        serde_json::from_value(serde_json::json!({
            "entry": {
                "subject": "did:web:alice.example",
                "role": "admin",
                "act": { "scope": "all" },
                "keys": { "scope": "all" },
                "capabilities": { "scope": "ceiling" }
            }
        }))
        .expect("a valid acl/grant/0.2 payload")
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

    // ── Declared extension (SPEC §5.2) ──────────────────────────────────
    //
    // A test must not depend on a specification that declares `extends`, so
    // these build the chain with the test overlay on the generated table
    // (`extension::test_index`, per thread). Two handler versions are needed:
    // `acl/show/0.2` is the generated one, and `0.3` is the test-only
    // `ShowV0_3` below, so the tests never lean on a superseded module that a
    // withdrawal batch removes. The versions 0.7–0.9 are fictional; a received
    // version never needs a module.

    use crate::extension::test_index;
    use crate::specs::acl::show::v0_2 as show_v0_2;

    const SHOW: &str = "https://trusttasks.org/spec/acl/show";

    /// A stand-in for a later `acl/show` version, implemented by hand: an
    /// object with one `subject`, refusing unknown members as the generated
    /// types do, with the matching schema for the dispatcher to prune by.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ShowV0_3 {
        subject: String,
    }

    impl Payload for ShowV0_3 {
        const TYPE_URI: &'static str = "https://trusttasks.org/spec/acl/show/0.3";
        const PAYLOAD_SCHEMA: Option<&'static str> = Some(
            r#"{"type":"object","additionalProperties":false,"properties":{"subject":{"type":"string"}}}"#,
        );
    }

    fn show_doc(version: &str, payload: Value) -> TrustTask<Value> {
        serde_json::from_value(serde_json::json!({
            "id": "req-ext",
            "type": format!("{SHOW}/{version}"),
            "issuer": "did:web:org.example",
            "recipient": "did:web:maintainer.example",
            "payload": payload,
        }))
        .unwrap()
    }

    /// `(handler version, type the handler saw, type its response carries,
    /// subject)`.
    type Seen = (&'static str, String, String, String);

    fn show_dispatcher(with_v0_3: bool) -> Dispatcher<Seen> {
        let d = Dispatcher::<Seen>::new().on::<show_v0_2::Payload, _>(|req| {
            let resp = req.respond_with("resp", ());
            (
                "0.2",
                req.type_uri.to_string(),
                resp.type_uri.to_string(),
                req.payload.subject.to_string(),
            )
        });
        if !with_v0_3 {
            return d;
        }
        d.on::<ShowV0_3, _>(|req| {
            let resp = req.respond_with("resp", ());
            (
                "0.3",
                req.type_uri.to_string(),
                resp.type_uri.to_string(),
                req.payload.subject.clone(),
            )
        })
    }

    #[test]
    fn exact_version_routes_as_before_even_when_a_chain_is_declared() {
        test_index::declare(&format!("{SHOW}/0.3"), Some("0.2"));
        let seen = show_dispatcher(true)
            .dispatch(show_doc(
                "0.3",
                serde_json::json!({ "subject": "did:web:a" }),
            ))
            .unwrap();
        assert_eq!(
            seen.0, "0.3",
            "an exact registration wins over a forward route"
        );
        assert_eq!(seen.2, format!("{SHOW}/0.3#response"));
    }

    #[test]
    fn an_extending_version_routes_to_the_implemented_one_pruned_and_answered_as_received() {
        test_index::declare(&format!("{SHOW}/0.9"), Some("0.8"));
        test_index::declare(&format!("{SHOW}/0.8"), Some("0.2"));
        let doc = show_doc(
            "0.9",
            serde_json::json!({ "subject": "did:web:a", "addedIn0_8": { "deep": true } }),
        );
        // Without pruning the downcast would refuse it: 0.2's `Payload` is
        // `deny_unknown_fields`.
        assert!(matches!(
            downcast_payload::<show_v0_2::Payload>(doc.clone()),
            Err(RejectReason::MalformedRequest { .. })
        ));

        let (version, seen_type, response_type, subject) =
            show_dispatcher(false).dispatch(doc).unwrap();
        assert_eq!(version, "0.2");
        assert_eq!(subject, "did:web:a");
        assert_eq!(
            seen_type,
            format!("{SHOW}/0.9"),
            "the type is kept as received"
        );
        assert_eq!(response_type, format!("{SHOW}/0.9#response"));
    }

    #[test]
    fn the_highest_registered_version_the_chain_reaches_wins() {
        test_index::declare(&format!("{SHOW}/0.9"), Some("0.3"));
        test_index::declare(&format!("{SHOW}/0.3"), Some("0.2"));
        let doc = show_doc("0.9", serde_json::json!({ "subject": "did:web:a", "x": 1 }));
        assert_eq!(
            show_dispatcher(true).dispatch(doc.clone()).unwrap().0,
            "0.3"
        );
        // With only 0.2 registered, the walk goes on down to it.
        assert_eq!(show_dispatcher(false).dispatch(doc).unwrap().0, "0.2");
    }

    #[test]
    fn a_version_whose_chain_does_not_reach_is_still_unsupported_version() {
        // Declares nothing.
        test_index::declare(&format!("{SHOW}/0.7"), None);
        let err = show_dispatcher(true)
            .dispatch(show_doc(
                "0.7",
                serde_json::json!({ "subject": "did:web:a" }),
            ))
            .unwrap_err();
        assert!(
            matches!(err, RejectReason::UnsupportedVersion { .. }),
            "{err:?}"
        );

        // Declares a chain, but to a version nobody registered.
        test_index::declare(&format!("{SHOW}/0.9"), Some("0.8"));
        test_index::declare(&format!("{SHOW}/0.8"), None);
        let err = show_dispatcher(true)
            .dispatch(show_doc(
                "0.9",
                serde_json::json!({ "subject": "did:web:a" }),
            ))
            .unwrap_err();
        assert!(
            matches!(err, RejectReason::UnsupportedVersion { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn read_as_reads_forward_and_refuses_what_the_chain_does_not_reach() {
        test_index::declare(&format!("{SHOW}/0.9"), Some("0.2"));
        test_index::declare(&format!("{SHOW}/0.7"), None);
        let typed = crate::extension::read_as::<show_v0_2::Payload>(show_doc(
            "0.9",
            serde_json::json!({ "subject": "did:web:a", "x": 1 }),
        ))
        .unwrap();
        assert_eq!(typed.type_uri.to_string(), format!("{SHOW}/0.9"));
        assert_eq!(typed.payload.subject.as_str(), "did:web:a");

        let exact = crate::extension::read_as::<show_v0_2::Payload>(show_doc(
            "0.2",
            serde_json::json!({ "subject": "did:web:a" }),
        ));
        assert!(exact.is_ok());

        let err = crate::extension::read_as::<show_v0_2::Payload>(show_doc(
            "0.7",
            serde_json::json!({ "subject": "did:web:a" }),
        ))
        .unwrap_err();
        assert!(
            matches!(err, RejectReason::UnsupportedVersion { .. }),
            "{err:?}"
        );
    }

    // ── Authorities (SPEC §6.1, §6.5) ───────────────────────────────────
    //
    // A slug is unique only within its authority: `acl/show` 0.2 under
    // another https authority, or in the URN form, is another specification,
    // and must never reach the registry's handler — exactly, forward, or by
    // error classification.

    const ELSEWHERE: &str = "https://tasks.example.com/spec/acl/show";
    const URN: &str = "urn:example:tasks:spec:acl:show";

    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ElsewhereShow {
        subject: String,
    }

    impl Payload for ElsewhereShow {
        const TYPE_URI: &'static str = "https://tasks.example.com/spec/acl/show/0.2";
        const PAYLOAD_SCHEMA: Option<&'static str> = Some(
            r#"{"type":"object","additionalProperties":false,"properties":{"subject":{"type":"string"}}}"#,
        );
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct UrnShow {
        subject: String,
    }

    impl Payload for UrnShow {
        const TYPE_URI: &'static str = "urn:example:tasks:spec:acl:show:0.2";
        const PAYLOAD_SCHEMA: Option<&'static str> = Some(
            r#"{"type":"object","additionalProperties":false,"properties":{"subject":{"type":"string"}}}"#,
        );
    }

    fn doc_of(type_uri: &str) -> TrustTask<Value> {
        serde_json::from_value(serde_json::json!({
            "id": "req-authority",
            "type": type_uri,
            "issuer": "did:web:org.example",
            "recipient": "did:web:maintainer.example",
            "payload": { "subject": "did:web:a" },
        }))
        .unwrap()
    }

    fn three_authorities() -> Dispatcher<(&'static str, String)> {
        Dispatcher::new()
            .on::<show_v0_2::Payload, _>(|req| {
                ("registry", req.respond_with("r", ()).type_uri.to_string())
            })
            .on::<ElsewhereShow, _>(|req| ("https", req.respond_with("r", ()).type_uri.to_string()))
            .on::<UrnShow, _>(|req| ("urn", req.respond_with("r", ()).type_uri.to_string()))
    }

    #[test]
    fn one_slug_under_three_authorities_routes_three_ways() {
        let d = three_authorities();
        for (uri, want) in [
            (format!("{SHOW}/0.2"), "registry"),
            (format!("{ELSEWHERE}/0.2"), "https"),
            (format!("{ELSEWHERE}/0.2#request"), "https"),
            (format!("{URN}:0.2"), "urn"),
            (format!("{URN}:0.2#request"), "urn"),
        ] {
            let (seen, response) = d.dispatch(doc_of(&uri)).unwrap();
            assert_eq!(seen, want, "{uri}");
            assert_eq!(
                response,
                format!("{}#response", uri.trim_end_matches("#request"))
            );
        }
        assert_eq!(d.registered_uris().len(), 3);
    }

    #[test]
    fn an_extension_chain_stays_within_its_authority() {
        // Each authority's 0.9 declares it extends 0.2 — of its own spec.
        test_index::declare(&format!("{URN}:0.9"), Some("0.2"));
        test_index::declare(&format!("{ELSEWHERE}/0.9"), Some("0.2"));
        let d = three_authorities();
        assert_eq!(d.dispatch(doc_of(&format!("{URN}:0.9"))).unwrap().0, "urn");
        assert_eq!(
            d.dispatch(doc_of(&format!("{ELSEWHERE}/0.9"))).unwrap().0,
            "https"
        );
        // The registry's 0.9 declares nothing, so it reaches nothing — not the
        // other authorities' 0.2 either.
        assert!(matches!(
            d.dispatch(doc_of(&format!("{SHOW}/0.9"))).unwrap_err(),
            RejectReason::UnsupportedVersion { .. }
        ));
        assert!(crate::extension::read_as::<UrnShow>(doc_of(&format!("{URN}:0.9"))).is_ok());
        assert!(matches!(
            crate::extension::read_as::<show_v0_2::Payload>(doc_of(&format!("{URN}:0.9")))
                .unwrap_err(),
            RejectReason::UnsupportedType { .. }
        ));
    }

    #[test]
    fn the_same_slug_under_an_unknown_authority_is_an_unsupported_type() {
        // Only the registry's acl/show is implemented: another authority's
        // acl/show is not a version of it.
        let d = Dispatcher::<&'static str>::new().on::<show_v0_2::Payload, _>(|_| "registry");
        for uri in [
            "https://other.example/spec/acl/show/0.2".to_string(),
            format!("{URN}:0.2"),
        ] {
            let err = d.dispatch(doc_of(&uri)).unwrap_err();
            assert!(
                matches!(err, RejectReason::UnsupportedType { .. }),
                "{uri}: {err:?}"
            );
        }
        // While the registry's own other versions remain unsupported versions.
        assert!(matches!(
            d.dispatch(doc_of(&format!("{SHOW}/0.7"))).unwrap_err(),
            RejectReason::UnsupportedVersion { .. }
        ));
    }
}
