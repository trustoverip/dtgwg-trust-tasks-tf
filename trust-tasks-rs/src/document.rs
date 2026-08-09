//! The framework-level Trust Task document envelope (SPEC.md §4).
//!
//! [`TrustTask<P>`] is generic over the `payload` type so that a caller can
//! either parameterize with a concrete per-spec payload struct (e.g. an
//! `AclGrant`) for compile-time typing, or use [`serde_json::Value`] for
//! opaque/dynamic processing.

use std::error::Error as StdError;
use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ceremony::Ceremony;
use crate::error::{ErrorPayload, RejectReason};
use crate::payload::Payload;
use crate::proof::Proof;
use crate::type_uri::TypeUri;

/// A `trust-task-error/0.1` document — a [`TrustTask`] whose payload is an
/// [`ErrorPayload`]. This type alias is the form most consumer code holds onto
/// when raising or propagating an error response.
pub type ErrorResponse = TrustTask<ErrorPayload>;

/// A single Trust Task document, per SPEC.md §4.2.
///
/// Field naming mirrors the wire form via `#[serde(rename = ...)]`. Unknown
/// top-level members are preserved in [`extra`](Self::extra) on round-trip so
/// that forwarding consumers honor the §7.1 producer guidance to preserve
/// unrecognized members.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustTask<P> {
    /// The *document identifier* — globally unique to this instance.
    pub id: String,

    /// The *thread identifier* correlating this document with others in the
    /// same logical exchange (SPEC.md §4.9).
    #[serde(rename = "threadId", default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,

    /// The `threadId` of the exchange containing this one, where this exchange
    /// is conducted inside another (SPEC.md §4.9.2).
    ///
    /// A navigation aid. It records one level of containment and does **not**
    /// change which exchange attests an event — §4.9.1 governs that, and holds
    /// whether or not this member is present. Like `thread_id` it carries no
    /// normative validation semantics: a consumer MUST NOT reject a document on
    /// the basis of `parentThreadId` alone.
    #[serde(
        rename = "parentThreadId",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub parent_thread_id: Option<String>,

    /// Records that this document is a step of a Trust Ceremony — a flow
    /// composed of several Trust Tasks (SPEC.md §4.11).
    ///
    /// Optional in every sense: no specification declares anything about
    /// ceremonies, a document without it is fully conforming, and a consumer
    /// that does not implement ceremonies processes the document unchanged.
    /// Ignoring it is always safe, because §4.11.4 forbids deriving authority
    /// from it — there is nothing a ceremony-aware consumer may do that an
    /// unaware one omits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ceremony: Option<Ceremony>,

    /// The *Type URI* identifying the specification and version this document
    /// conforms to.
    #[serde(rename = "type")]
    pub type_uri: TypeUri,

    /// VID of the party responsible for the document's content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,

    /// VID of the party expected to act upon the document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipient: Option<String>,

    /// Timestamp recording when the document was produced (SPEC.md §4.2).
    #[serde(rename = "issuedAt", default, skip_serializing_if = "Option::is_none")]
    pub issued_at: Option<DateTime<Utc>>,

    /// Timestamp after which the document is no longer valid (SPEC.md §4.2).
    #[serde(rename = "expiresAt", default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,

    /// The task-specific body, whose internal structure is defined by the
    /// specification identified by [`type_uri`](Self::type_uri).
    pub payload: P,

    /// Optional JSON-LD context (SPEC.md §4.6). When present, the document
    /// MUST be processable as JSON-LD.
    #[serde(rename = "@context", default, skip_serializing_if = "Option::is_none")]
    pub context: Option<JsonLdContext>,

    /// Optional Data Integrity proof binding the document to its issuer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof: Option<Proof>,

    /// Any additional top-level members carried by the document. Preserved on
    /// round-trip per the §7.1 / §7.2 guidance to retain unrecognized members.
    #[serde(flatten)]
    pub extra: std::collections::BTreeMap<String, Value>,
}

impl<P> TrustTask<P> {
    /// Construct a new document with only the required members populated.
    /// Optional members can be set via field assignment.
    pub fn new(id: impl Into<String>, type_uri: TypeUri, payload: P) -> Self {
        Self {
            id: id.into(),
            thread_id: None,
            parent_thread_id: None,
            ceremony: None,
            type_uri,
            issuer: None,
            recipient: None,
            issued_at: None,
            expires_at: None,
            payload,
            context: None,
            proof: None,
            extra: Default::default(),
        }
    }

    /// Construct a new document, taking the `type` URI from the payload's
    /// [`Payload`] impl. Saves callers from restating the Type URI when they
    /// already hold a typed payload from [`crate::specs`].
    ///
    /// ```rust,ignore
    /// let req = TrustTask::for_payload("req-1", AclGrant { ... });
    /// assert_eq!(req.type_uri, AclGrant::type_uri());
    /// ```
    pub fn for_payload(id: impl Into<String>, payload: P) -> Self
    where
        P: Payload,
    {
        Self::new(id, P::type_uri(), payload)
    }

    /// Apply the SPEC.md §7.2 item 8 / §4.8.2 audience-binding rule: when
    /// `proof` is present and `recipient` is absent in-band, reject the
    /// document with `malformed_request` unless the originating
    /// specification is a *bearer specification* (§4.8.3).
    ///
    /// This check requires the payload type implement [`Payload`] so the
    /// codegen-emitted [`Payload::IS_BEARER`] flag is reachable; callers
    /// holding a `TrustTask<serde_json::Value>` should downcast via
    /// [`crate::Dispatcher`] or by hand before invoking this method.
    ///
    /// A non-bearer specification that signs every document with an
    /// in-band `recipient` (which is the safe default) always passes this
    /// check. A bearer specification opts out of audience binding at the
    /// spec layer and always passes — bearer status is published in the
    /// spec's front matter and codegened into the `Payload` impl, not
    /// chosen by the consumer.
    pub fn enforce_audience_binding(&self) -> Result<(), RejectReason>
    where
        P: Payload,
    {
        if self.proof.is_some() && self.recipient.is_none() && !P::IS_BEARER {
            return Err(RejectReason::MalformedRequest {
                reason: "proof present with no in-band recipient on a non-bearer specification \
                         (SPEC §4.8.2 audience binding)"
                    .to_string(),
            });
        }
        Ok(())
    }

    /// Apply the per-spec consumer checks that depend on the payload type's
    /// codegen-emitted flags — the *typed* subset of SPEC §7.2:
    ///
    /// * **item 5b — recipient-REQUIRED** ([`Payload::IS_RECIPIENT_REQUIRED`]):
    ///   a `recipient`-REQUIRED spec needs the recipient carried in-band, so a
    ///   document without one is `malformedRequest`.
    /// * **item 7 clause A — proof-REQUIRED** ([`Payload::IS_PROOF_REQUIRED`]):
    ///   a `proof`-REQUIRED spec rejects a proofless document with
    ///   `proofRequired`.
    /// * **item 8 — audience binding** ([`Self::enforce_audience_binding`]).
    ///
    /// This is the single source of truth for the flag-driven §7.2 checks. Both
    /// the library [`consume_inbound`](crate::consume_inbound) path and any
    /// binding-specific pipeline (for example the HTTPS server) call it, so the
    /// two cannot diverge on the check set as new flag-driven rules are added.
    /// It does **not** include the non-typed checks (expiry, recipient/transport
    /// cross-check, proof *verification*), which each pipeline applies around
    /// this call per its own transport model.
    pub fn enforce_spec_policy(&self) -> Result<(), RejectReason>
    where
        P: Payload,
    {
        if self.recipient.is_none() && P::IS_RECIPIENT_REQUIRED {
            return Err(RejectReason::MalformedRequest {
                reason: "specification declares recipient REQUIRED but the document \
                         carries no in-band recipient"
                    .to_string(),
            });
        }
        if self.proof.is_none() && P::IS_PROOF_REQUIRED {
            return Err(RejectReason::ProofRequired);
        }
        self.enforce_audience_binding()
    }

    /// Returns `true` if `expires_at` is set and `now ≥ expiresAt`
    /// (inclusive bound per SPEC.md §4.2). The instant `expiresAt` is
    /// itself treated as expired, matching JWT-style semantics.
    /// SPEC §4.2 permits a consumer to apply a small clock-skew tolerance
    /// (typically ≤ 60s); apply that at the caller by adjusting `now`.
    pub fn is_expired_at(&self, now: DateTime<Utc>) -> bool {
        matches!(self.expires_at, Some(t) if t <= now)
    }

    /// Apply the framework-level rejection rules from SPEC.md §7.2 items 4
    /// and 5:
    ///
    /// * Item 4 — reject when `expiresAt` is set and `now ≥ expiresAt`
    ///   (inclusive bound per the post-0.2 §4.2 wording).
    /// * Item 5 — reject when `recipient` is set and does not identify
    ///   `my_vid`.
    ///
    /// # ⚠ This is *not* the full §7.2 check
    ///
    /// A conforming consumer pipeline runs all six (now eight) items of
    /// §7.2. This method covers items 4 and 5 only:
    ///
    /// | §7.2 item | What it checks                                              | Where it lives                                      |
    /// |-----------|-------------------------------------------------------------|-----------------------------------------------------|
    /// | 1         | Framework schema validation                                 | caller responsibility (e.g. `serde` + feature `validate`) |
    /// | 2         | Payload schema validation                                   | caller (typed `TrustTask<P>` + feature `validate`)  |
    /// | 3         | Unknown `type` URI                                          | [`crate::Dispatcher`] / caller's type registry      |
    /// | **4**     | **Expiry**                                                  | **`validate_basic`**                                |
    /// | **5**     | **Recipient mismatch**                                      | **`validate_basic`**                                |
    /// | 6         | In-band vs transport identity                               | [`TransportHandler::resolve_parties`](crate::TransportHandler::resolve_parties) |
    /// | 7         | Proof handling (`IS_PROOF_REQUIRED` + verification policy)  | [`consume_inbound`](crate::consume_inbound) + [`ProofVerifier`](crate::ProofVerifier) (cryptosuite in a companion crate) |
    /// | 8         | Audience binding (proof+no-recipient on non-bearer specs)   | [`enforce_audience_binding`](Self::enforce_audience_binding) |
    ///
    /// The full §7.2 pipeline is bundled in
    /// [`consume_inbound`](crate::consume_inbound) — items 4–8 in one
    /// call. Direct use of `validate_basic` is for callers that have
    /// their own composition.
    ///
    /// Treat `validate_basic(now, my_vid)?` as **stage 2** of a multi-stage
    /// validation. Calling only this method on an inbound document
    /// produces a non-conforming consumer.
    pub fn validate_basic(&self, now: DateTime<Utc>, my_vid: &str) -> Result<(), RejectReason> {
        if let Some(expires_at) = self.expires_at {
            // SPEC §4.2 / §7.2 item 4: inclusive bound — `now ≥ expiresAt`
            // is expired. Equivalent to `expires_at <= now`.
            if expires_at <= now {
                return Err(RejectReason::Expired { expires_at });
            }
        }
        if let Some(recipient) = self.recipient.as_deref() {
            if recipient != my_vid {
                return Err(RejectReason::WrongRecipient {
                    in_band: recipient.to_string(),
                    expected: my_vid.to_string(),
                });
            }
        }
        Ok(())
    }

    /// Build the `trust-task-error/0.1` response document for this request,
    /// per the spec's "Reporting consumer" conformance rules.
    ///
    /// Wires:
    ///
    /// * `type` → `https://trusttasks.org/spec/trust-task-error/0.1`
    /// * `threadId` → this request's `threadId`, falling back to its `id`
    ///   per SPEC.md §4.9.
    /// * `issuer` → this request's `recipient` (the rejecting consumer).
    /// * `recipient` → this request's `issuer` (the original producer).
    /// * `issuedAt` → [`Utc::now`].
    ///
    /// `payload` is taken as-is. Pass an [`ErrorPayload`] you constructed
    /// directly, the output of [`ErrorPayload::from`] applied to a
    /// [`RejectReason`], or anything else that converts via [`Into`].
    ///
    /// The caller supplies `id`; the framework does not constrain its form
    /// beyond uniqueness (SPEC.md §4.3). UUIDv4 is the recommended default.
    ///
    /// # ⚠ Identity-mismatch safety
    ///
    /// This method copies `request.issuer` verbatim into the error
    /// response's `recipient`. Under most rejections (`Expired`,
    /// `ProofRequired`, `ProofInvalid`, `TaskFailed`, …) the in-band
    /// `issuer` is a value the consumer has reason to trust — for example,
    /// because [`TransportHandler::resolve_parties`](crate::TransportHandler::resolve_parties)
    /// already accepted it. Under [`RejectReason::IdentityMismatch`],
    /// however, that in-band `issuer` is by definition the contested
    /// identity and MUST NOT be addressed as the error response's
    /// recipient (SPEC.md §8.1, §10.4). For that case, use either
    /// [`Self::reject_with_recipient`] with an explicit transport-
    /// authenticated recipient, or
    /// [`TransportHandler::reject`](crate::TransportHandler::reject),
    /// which applies the §8.1 routing policy automatically.
    pub fn reject_with(
        &self,
        id: impl Into<String>,
        payload: impl Into<ErrorPayload>,
    ) -> ErrorResponse {
        self.reject_with_recipient(id, payload, self.issuer.clone())
    }

    /// Build the `trust-task-error/0.1` response document with an explicit
    /// `recipient`. Use this when the safe default in [`Self::reject_with`]
    /// does not apply — most importantly under
    /// [`RejectReason::IdentityMismatch`], where SPEC.md §8.1 requires the
    /// response to address the transport-authenticated sender rather than
    /// the in-band (contested) issuer.
    ///
    /// `recipient = None` is conformant: SPEC.md §8.1 permits a consumer
    /// faced with an `identity_mismatch` rejection and no transport-
    /// authenticated sender to suppress the response entirely; the caller
    /// can choose to drop the returned `ErrorResponse` in that case.
    pub fn reject_with_recipient(
        &self,
        id: impl Into<String>,
        payload: impl Into<ErrorPayload>,
        recipient: Option<String>,
    ) -> ErrorResponse {
        let thread_id = self.thread_id.clone().or_else(|| Some(self.id.clone()));
        let mut payload = payload.into();
        // §8.2 — name the document this error reports on, so the error means
        // something to a party that did not see the request. Populated here
        // rather than left to the caller because the builder is the only place
        // that reliably has the originating document in hand.
        if payload.in_response_to.is_none() {
            payload.in_response_to = Some(crate::InResponseTo {
                type_uri: self.type_uri.to_string(),
                // §8.1/§8.2 — under `identityMismatch` the response is
                // addressed to the transport-authenticated sender, not the
                // in-band issuer. That party did not necessarily compose the
                // document, so its identifier is not echoed back.
                id: match &payload.code {
                    crate::TrustTaskCode::Standard(crate::StandardCode::IdentityMismatch) => None,
                    _ => Some(self.id.clone()),
                },
            });
        }
        ErrorResponse {
            id: id.into(),
            thread_id,
            // §7.1 — the same enactment is carried onto every document of a
            // step, error responses included, so a rejection stays inside the
            // ceremony it belongs to rather than falling out of it.
            ceremony: self.ceremony.clone(),
            // §4.9.2 — the whole exchange shares one parent, so the error
            // response stays inside the same enclosing exchange.
            parent_thread_id: self.parent_thread_id.clone(),
            type_uri: trust_task_error_type_uri(),
            issuer: self.recipient.clone(),
            recipient,
            issued_at: Some(Utc::now()),
            expires_at: None,
            payload,
            context: None,
            proof: None,
            extra: Default::default(),
        }
    }

    /// Build the success-response document for this request, per SPEC.md
    /// §4.4.1. The mirror of [`reject_with`](Self::reject_with) for the
    /// success path.
    ///
    /// Wires:
    ///
    /// * `type` → this request's Type URI with `#response` fragment.
    /// * `threadId` → this request's `threadId`, falling back to its `id`
    ///   per SPEC.md §4.9.
    /// * `issuer` → this request's `recipient` (the responding party).
    /// * `recipient` → this request's `issuer` (the original producer).
    /// * `issuedAt` → [`Utc::now`].
    ///
    /// `R` is the response payload type defined by the originating *Trust
    /// Task specification*'s `$anchor: "response"` sub-schema. A spec that
    /// defines no success response is fire-and-forget; do not call this
    /// method for such specs (SPEC.md §4.4.1).
    pub fn respond_with<R>(&self, id: impl Into<String>, payload: R) -> TrustTask<R> {
        let thread_id = self.thread_id.clone().or_else(|| Some(self.id.clone()));
        TrustTask {
            id: id.into(),
            thread_id,
            // §7.1 — as for an error response, the response half of a step
            // belongs to the same enactment as the request.
            ceremony: self.ceremony.clone(),
            // §4.9.2 — the whole exchange shares one parent.
            parent_thread_id: self.parent_thread_id.clone(),
            type_uri: self.type_uri.with_response(),
            issuer: self.recipient.clone(),
            recipient: self.issuer.clone(),
            issued_at: Some(Utc::now()),
            expires_at: None,
            payload,
            context: None,
            proof: None,
            extra: Default::default(),
        }
    }
}

pub(crate) fn trust_task_error_type_uri() -> TypeUri {
    // The `trust-task-error` slug is a framework-defined reserved name, so
    // `TypeUri::canonical` accepts it. The SDK emits `0.3`: it populates the
    // `inResponseTo` member of §8.2, and `0.2`'s payload schema is
    // `additionalProperties: false`, so a document carrying it would not
    // validate as `0.2`. Per §5.2 forward-minor compatibility a `0.2` consumer
    // SHOULD accept it.
    TypeUri::canonical("trust-task-error", 0, 3)
        .expect("trust-task-error/0.3 is a valid framework Type URI")
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}]", self.payload, self.id)
    }
}

impl StdError for ErrorResponse {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.payload)
    }
}

/// The value of the optional `@context` member, per SPEC.md §4.6 / JSON-LD.
///
/// JSON-LD permits a string, an array of strings or objects, or an object;
/// the framework places no further constraint.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonLdContext {
    /// A single context URL.
    Single(String),
    /// An array of context URLs and/or inline objects.
    Multiple(Vec<Value>),
    /// An inline context object.
    Object(serde_json::Map<String, Value>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct KycHandoff {
        subject: String,
        result: String,
        level: String,
    }

    #[test]
    fn parses_spec_example_one() {
        // SPEC.md §4.2 Example 1.
        let json = r#"{
            "id": "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2",
            "type": "https://trusttasks.org/spec/kyc-handoff/1.0",
            "issuer": "did:web:verifier.example",
            "recipient": "did:web:bank.example",
            "issuedAt": "2026-04-12T09:31:00Z",
            "expiresAt": "2027-04-12T09:31:00Z",
            "payload": {
                "subject": "did:key:z6Mk...",
                "result": "passed",
                "level": "LOA2"
            }
        }"#;

        let doc: TrustTask<KycHandoff> = serde_json::from_str(json).unwrap();
        assert_eq!(doc.id, "4f3c9e2a-1b81-4d3e-9b51-7a3c89e3d1f2");
        assert_eq!(doc.type_uri.slug(), "kyc-handoff");
        assert_eq!(doc.issuer.as_deref(), Some("did:web:verifier.example"));
        assert_eq!(doc.payload.level, "LOA2");
        assert!(doc.thread_id.is_none());
        assert!(doc.proof.is_none());
        assert!(doc.extra.is_empty());
    }

    /// SPEC §4.9.2 — the whole inner exchange shares one parent, so both the
    /// success response and the error response stay inside the enclosing
    /// exchange. A response that dropped it would strand the inner exchange.
    #[test]
    fn parent_thread_id_is_carried_onto_responses() {
        const PARENT: &str = "urn:uuid:9b1d3f60-52a8-4c17-8e44-1d9c7b05f3ae";
        let mut req = TrustTask::new(
            "req-1",
            "https://trusttasks.org/spec/acl/grant/0.1".parse().unwrap(),
            serde_json::json!({}),
        );
        req.thread_id = Some("inner-1".into());
        req.parent_thread_id = Some(PARENT.into());
        req.issuer = Some("did:web:org.example".into());
        req.recipient = Some("did:web:maintainer.example".into());

        let ok = req.respond_with("resp-1", serde_json::json!({}));
        assert_eq!(ok.parent_thread_id.as_deref(), Some(PARENT));
        assert_eq!(ok.thread_id.as_deref(), Some("inner-1"));

        let err = req.reject_with(
            "err-1",
            ErrorPayload::new(crate::TrustTaskCode::from(crate::StandardCode::TaskFailed)),
        );
        assert_eq!(err.parent_thread_id.as_deref(), Some(PARENT));
    }

    /// Absent, not null. An explicit `None` that serialised would imply the
    /// exchange has a null parent rather than no parent.
    #[test]
    fn parent_thread_id_is_omitted_from_the_wire_when_unset() {
        let req = TrustTask::new(
            "req-1",
            "https://trusttasks.org/spec/acl/grant/0.1".parse().unwrap(),
            serde_json::json!({}),
        );
        let wire = serde_json::to_string(&req).unwrap();
        assert!(!wire.contains("parentThreadId"), "wire: {wire}");
    }

    /// Round-trips under the wire name from §4.2, not the Rust field name.
    #[test]
    fn parent_thread_id_round_trips_under_its_wire_name() {
        let json = serde_json::json!({
            "id": "req-1",
            "type": "https://trusttasks.org/spec/acl/grant/0.1",
            "threadId": "inner-1",
            "parentThreadId": "outer-1",
            "payload": {}
        });
        let doc: TrustTask<serde_json::Value> = serde_json::from_value(json).unwrap();
        assert_eq!(doc.parent_thread_id.as_deref(), Some("outer-1"));
        let back = serde_json::to_value(&doc).unwrap();
        assert_eq!(back["parentThreadId"], "outer-1");
    }

    #[test]
    fn round_trips_minimum_document() {
        let doc = TrustTask::new(
            "abc",
            TypeUri::canonical("kyc-handoff", 1, 0).unwrap(),
            KycHandoff {
                subject: "did:key:z6Mk".to_string(),
                result: "passed".to_string(),
                level: "LOA2".to_string(),
            },
        );

        let json = serde_json::to_value(&doc).unwrap();
        // Omitted optional members are not serialized.
        assert!(json.get("threadId").is_none());
        assert!(json.get("issuer").is_none());
        assert!(json.get("@context").is_none());
        assert!(json.get("proof").is_none());

        let back: TrustTask<KycHandoff> = serde_json::from_value(json).unwrap();
        assert_eq!(back, doc);
    }

    #[test]
    fn preserves_unknown_top_level_members() {
        let json = r#"{
            "id": "x",
            "type": "https://trusttasks.org/spec/kyc-handoff/1.0",
            "payload": {"subject":"s","result":"passed","level":"LOA1"},
            "x-experimental": "kept"
        }"#;

        let doc: TrustTask<KycHandoff> = serde_json::from_str(json).unwrap();
        assert_eq!(
            doc.extra.get("x-experimental").and_then(Value::as_str),
            Some("kept")
        );

        let rendered = serde_json::to_value(&doc).unwrap();
        assert_eq!(
            rendered.get("x-experimental").and_then(Value::as_str),
            Some("kept")
        );
    }

    #[test]
    fn detects_expiry() {
        let mut doc = TrustTask::new(
            "abc",
            TypeUri::canonical("kyc-handoff", 1, 0).unwrap(),
            serde_json::json!({}),
        );
        let expiry: DateTime<Utc> = "2026-04-12T09:31:00Z".parse().unwrap();
        doc.expires_at = Some(expiry);

        let before: DateTime<Utc> = "2026-04-12T09:00:00Z".parse().unwrap();
        let after: DateTime<Utc> = "2026-04-12T10:00:00Z".parse().unwrap();
        assert!(!doc.is_expired_at(before));
        assert!(doc.is_expired_at(after));
        // SPEC §4.2 — `now == expiresAt` is expired (inclusive bound).
        assert!(doc.is_expired_at(expiry));
    }
}
