//! Error-response payload for the `trust-task-error` framework spec. The SDK
//! emits the `0.4` form — lowerCamelCase codes including `idConflict`, and the
//! `inResponseTo` member that names the document being reported on. The parser
//! also accepts the `0.1` snake_case codes, so a current consumer can read a
//! `0.1` peer.
//!
//! Models the structure defined in SPEC.md §8.2 and §8.3. The set of standard
//! codes is encoded as the [`StandardCode`] enum; task-specific extensions
//! (SPEC.md §8.5) are namespaced strings carried in the [`TrustTaskCode`]
//! `Extended` variant.

use std::error::Error as StdError;
use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use thiserror::Error;

use crate::transport::ConsistencyError;

/// Names the *Trust Task document* an [`ErrorPayload`] reports on (SPEC.md
/// §8.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InResponseTo {
    /// The reported-on document's `type`, **including** any `#request` or
    /// `#response` fragment it carried — that fragment is what tells a consumer
    /// which variant's semantics apply.
    #[serde(rename = "typeUri")]
    pub type_uri: String,

    /// The reported-on document's `id`. Globally unique and never reused
    /// (SPEC.md §4.3), so it names one instance where `threadId` names an
    /// exchange.
    ///
    /// Omitted under `identityMismatch`: per §8.1 the response is addressed to
    /// the transport-authenticated sender rather than the in-band `issuer`, and
    /// that party did not necessarily compose the document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// The `payload` of a `trust-task-error/0.4` document, per SPEC.md §8.2.
///
/// Exchange-level correlation is carried by the surrounding
/// [`TrustTask`](crate::TrustTask)'s `threadId`; *which document* this error
/// reports on is carried here, by [`in_response_to`](Self::in_response_to).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorPayload {
    /// Short identifier for the failure category.
    pub code: TrustTaskCode,

    /// Identifies the *Trust Task document* this error reports on (SPEC.md
    /// §8.2).
    ///
    /// `threadId` correlates the exchange for a party that saw the originating
    /// request, and identifies nothing to anyone else. Without this, a retained
    /// error names neither the specification the failure occurred under nor the
    /// instance that triggered it — and for the standard codes of §8.3 there is
    /// no other signal of origin at all.
    ///
    /// **SHOULD** be populated in general, and **MUST** be where the error will
    /// be retained, replayed, or relied upon beyond the original producer. The
    /// document builders on [`TrustTask`](crate::TrustTask) populate it for you.
    #[serde(
        rename = "inResponseTo",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub in_response_to: Option<InResponseTo>,

    /// Human-readable description. Non-normative; intended for logs and
    /// operator UI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Whether the producer MAY retry the original document. Per SPEC.md §8.4,
    /// "retry" means re-sending bit-for-bit; a new document with a fresh `id`
    /// is not a retry.
    pub retryable: bool,

    /// Earliest time at which the producer SHOULD retry. Meaningful only when
    /// [`retryable`](Self::retryable) is `true`.
    #[serde(
        rename = "retryAfter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub retry_after: Option<DateTime<Utc>>,

    /// Task-specific extension data. The shape is defined by the
    /// originating *Trust Task specification* (SPEC.md §8.5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl ErrorPayload {
    /// Construct a minimal payload for `code`. `retryable` is initialized to
    /// the code's default per the §8.3 table (overridable via
    /// [`with_retryable`](Self::with_retryable)). All other members are absent.
    pub fn new(code: impl Into<TrustTaskCode>) -> Self {
        let code = code.into();
        let retryable = match &code {
            TrustTaskCode::Standard(c) => c.default_retryable(),
            // Per SPEC §8.5, an unrecognized extension code is treated as
            // `task_failed` — which defaults to non-retryable. The producer of
            // the extension MAY override.
            TrustTaskCode::Extended { .. } => false,
        };
        Self {
            code,
            in_response_to: None,
            message: None,
            retryable,
            retry_after: None,
            details: None,
        }
    }

    /// Name the document this error reports on (SPEC.md §8.2).
    ///
    /// Prefer the builders on [`TrustTask`](crate::TrustTask), which populate
    /// this from the request being rejected — including the §8.1 rule that the
    /// `id` is omitted under `identityMismatch`. Use this directly only when
    /// constructing a payload without the originating document in hand.
    pub fn about(mut self, type_uri: impl Into<String>, id: Option<String>) -> Self {
        self.in_response_to = Some(InResponseTo {
            type_uri: type_uri.into(),
            id,
        });
        self
    }

    /// Attach a human-readable message.
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Override the `retryable` flag (default is taken from
    /// [`StandardCode::default_retryable`]).
    pub fn with_retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }

    /// Attach a `retryAfter` hint. Meaningful only when `retryable` is `true`.
    pub fn with_retry_after(mut self, when: DateTime<Utc>) -> Self {
        self.retry_after = Some(when);
        self
    }

    /// Attach task-specific extension data.
    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Resolve this payload's code to a [`StandardCode`] the consumer is
    /// guaranteed to recognize, applying the SPEC §8.5 fallback:
    /// unrecognized extension codes collapse to [`StandardCode::TaskFailed`].
    ///
    /// Standard codes are returned as-is.
    pub fn effective_code(&self) -> StandardCode {
        match &self.code {
            TrustTaskCode::Standard(c) => *c,
            TrustTaskCode::Extended { .. } => StandardCode::TaskFailed,
        }
    }

    /// Apply the SPEC §8.4 retry-semantics check.
    ///
    /// Returns `true` only when:
    ///   * `retryable` is `true`, AND
    ///   * `retry_after` is either absent or already past `now`.
    ///
    /// A `false` return means the producer **MUST NOT** retry yet (or at all,
    /// when `retryable` is `false`). "Retry" here is the strict bit-for-bit
    /// sense from SPEC §8.4 — issuing a new document with a fresh `id` is
    /// always permitted regardless of this value.
    pub fn should_retry_at(&self, now: DateTime<Utc>) -> bool {
        if !self.retryable {
            return false;
        }
        match self.retry_after {
            None => true,
            Some(t) => t <= now,
        }
    }
}

impl From<StandardCode> for ErrorPayload {
    fn from(code: StandardCode) -> Self {
        Self::new(code)
    }
}

impl From<TrustTaskCode> for ErrorPayload {
    fn from(code: TrustTaskCode) -> Self {
        Self::new(code)
    }
}

impl fmt::Display for ErrorPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.message.as_deref() {
            Some(msg) => write!(f, "{}: {}", self.code, msg),
            None => write!(f, "{}", self.code),
        }
    }
}

impl StdError for ErrorPayload {}

/// An error code — either a framework-standard code (SPEC.md §8.3) or an
/// extension code namespaced by a spec's slug (SPEC.md §8.5).
#[derive(Debug, Clone, PartialEq, Eq, SerializeDisplay, DeserializeFromStr)]
pub enum TrustTaskCode {
    /// One of the framework-defined codes recognized by every conforming
    /// consumer.
    Standard(StandardCode),

    /// A specification-extended code in the form `<slug>:<local>`.
    Extended {
        /// The slug owning this extension, e.g. `"kyc-handoff"` or `"acl/grant"`.
        slug: String,
        /// The local code identifier within the slug's namespace.
        local: String,
    },
}

/// The framework-defined standard error codes (SPEC.md §8.3).
///
/// Marked `#[non_exhaustive]` as of 0.7.0: the framework adds a standard code
/// from time to time (`idConflict` in 0.4), and without this every such
/// addition would be a breaking change for any downstream `match`. Downstream
/// code must carry a wildcard arm; in exchange, future codes arrive as a minor
/// bump rather than a major one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum StandardCode {
    /// The document did not validate against the framework schema or the
    /// task-specific payload schema.
    MalformedRequest,
    /// The consumer does not recognize the `type` URI.
    UnsupportedType,
    /// The `type` URI was recognized but its `MAJOR.MINOR` is not supported.
    UnsupportedVersion,
    /// `expiresAt` was in the past at the time of evaluation.
    Expired,
    /// A `proof` was required and was missing.
    ProofRequired,
    /// A `proof` was present but failed verification.
    ProofInvalid,
    /// The requesting party is not authorized to invoke this task.
    PermissionDenied,
    /// The document's `recipient` does not identify the receiving consumer.
    WrongRecipient,
    /// An in-band `issuer` or `recipient` is inconsistent with the transport-
    /// authenticated identity for the same party.
    IdentityMismatch,
    /// The document's `id` matches one the consumer has already accepted, but
    /// its content differs (SPEC.md §7.2 item 11).
    IdConflict,
    /// The recipient party attempted the task and could not complete it.
    TaskFailed,
    /// The recipient party is temporarily unable to process the task.
    Unavailable,
    /// The recipient party encountered an unexpected internal failure.
    InternalError,
}

impl StandardCode {
    /// The default `retryable` value an emitter SHOULD use per SPEC.md §8.3,
    /// unless task-specific knowledge dictates otherwise.
    ///
    /// `TaskFailed` is `varies` in the spec; we treat its default as `false`.
    pub fn default_retryable(self) -> bool {
        matches!(
            self,
            StandardCode::Unavailable | StandardCode::InternalError
        )
    }

    /// The wire-form string this code is emitted as.
    ///
    /// As of framework 0.2 the standard codes are lowerCamelCase (SPEC.md
    /// §8.3 / Appendix B). `parse_standard` still accepts the framework
    /// 0.1 snake_case spellings so a 0.2 consumer can read a 0.1 peer.
    pub fn as_str(self) -> &'static str {
        match self {
            StandardCode::MalformedRequest => "malformedRequest",
            StandardCode::UnsupportedType => "unsupportedType",
            StandardCode::UnsupportedVersion => "unsupportedVersion",
            StandardCode::Expired => "expired",
            StandardCode::ProofRequired => "proofRequired",
            StandardCode::ProofInvalid => "proofInvalid",
            StandardCode::PermissionDenied => "permissionDenied",
            StandardCode::WrongRecipient => "wrongRecipient",
            StandardCode::IdentityMismatch => "identityMismatch",
            StandardCode::IdConflict => "idConflict",
            StandardCode::TaskFailed => "taskFailed",
            StandardCode::Unavailable => "unavailable",
            StandardCode::InternalError => "internalError",
        }
    }
}

impl fmt::Display for StandardCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Reason a string fails to parse as a [`TrustTaskCode`].
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseCodeError {
    /// The code is empty.
    #[error("error code is empty")]
    Empty,
    /// The namespace portion before `:` is not a valid slug.
    #[error("extension code namespace {0:?} is not a valid slug")]
    InvalidNamespace(String),
    /// The local portion after `:` is empty or malformed.
    #[error("extension code local part {0:?} is invalid")]
    InvalidLocal(String),
}

impl FromStr for TrustTaskCode {
    type Err = ParseCodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ParseCodeError::Empty);
        }
        match s.split_once(':') {
            Some((slug, local)) => {
                validate_slug(slug)
                    .map_err(|_| ParseCodeError::InvalidNamespace(slug.to_string()))?;
                validate_local(local)
                    .map_err(|_| ParseCodeError::InvalidLocal(local.to_string()))?;
                Ok(TrustTaskCode::Extended {
                    slug: slug.to_string(),
                    local: local.to_string(),
                })
            }
            None => Ok(TrustTaskCode::Standard(parse_standard(s)?)),
        }
    }
}

impl fmt::Display for TrustTaskCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrustTaskCode::Standard(c) => f.write_str(c.as_str()),
            TrustTaskCode::Extended { slug, local } => write!(f, "{slug}:{local}"),
        }
    }
}

impl From<StandardCode> for TrustTaskCode {
    fn from(code: StandardCode) -> Self {
        TrustTaskCode::Standard(code)
    }
}

impl TrustTaskCode {
    /// Construct an extended code `<slug>:<local>` after validating both
    /// halves against SPEC.md §8.5's grammar (slug per §6.1, local per
    /// `spec.meta.schema.json`'s `errorCodes[].code` pattern after the
    /// colon). Returns [`ParseCodeError`] when either is malformed.
    ///
    /// Round-trips cleanly through [`Display`](fmt::Display) and
    /// [`FromStr`](std::str::FromStr) — a guarantee the hand-rolled
    /// struct-literal form (`TrustTaskCode::Extended { slug, local }`)
    /// does not enforce.
    pub fn new_extended(
        slug: impl Into<String>,
        local: impl Into<String>,
    ) -> Result<Self, ParseCodeError> {
        let slug = slug.into();
        let local = local.into();
        validate_slug(&slug).map_err(|_| ParseCodeError::InvalidNamespace(slug.clone()))?;
        validate_local(&local).map_err(|_| ParseCodeError::InvalidLocal(local.clone()))?;
        Ok(TrustTaskCode::Extended { slug, local })
    }
}

/// Typed rejection conditions a conforming consumer raises while applying
/// SPEC.md §7.2.
///
/// Each variant carries the context needed to render a meaningful operator
/// message and maps to a single [`StandardCode`] from §8.3. The `From` impl
/// produces an [`ErrorPayload`] with the matching code, retryable default,
/// and a message derived from the variant's fields.
///
/// Use this on the consumer side to turn `?`-propagated errors from
/// `resolve_parties` / `validate_basic` / your own task logic into a
/// well-formed error response.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RejectReason {
    /// The document did not validate against the framework or payload schema.
    #[error("malformed request: {reason}")]
    MalformedRequest {
        /// Human-readable explanation, surfaced as `message` in the response.
        reason: String,
    },

    /// The consumer does not implement this `type` URI.
    #[error("unsupported type: {type_uri}")]
    UnsupportedType {
        /// The unsupported Type URI as it appeared in the document.
        type_uri: String,
    },

    /// The consumer recognizes the type but not at this `MAJOR.MINOR`.
    #[error("unsupported version: {type_uri}")]
    UnsupportedVersion {
        /// The unsupported Type URI as it appeared in the document.
        type_uri: String,
    },

    /// The document's `expiresAt` was in the past.
    #[error("document expired at {expires_at}")]
    Expired {
        /// The expiry timestamp carried in the rejected document.
        expires_at: DateTime<Utc>,
    },

    /// A `proof` was required by the spec or consumer policy but was missing.
    #[error("proof required but not present")]
    ProofRequired,

    /// A `proof` was present but failed verification.
    #[error("proof verification failed: {reason}")]
    ProofInvalid {
        /// Human-readable explanation, surfaced as `message`.
        reason: String,
    },

    /// The requesting party is not authorized to invoke this task.
    #[error("permission denied: {reason}")]
    PermissionDenied {
        /// Human-readable explanation, surfaced as `message`.
        reason: String,
    },

    /// The document's `recipient` is set but does not identify the consumer.
    #[error("wrong recipient: in-band {in_band:?}, expected {expected:?}")]
    WrongRecipient {
        /// Value carried by the document's `recipient` member.
        in_band: String,
        /// VID of the consumer that received the document.
        expected: String,
    },

    /// An in-band party identity contradicts the transport-derived value.
    #[error(transparent)]
    IdentityMismatch(#[from] ConsistencyError),

    /// The task was attempted but could not be completed.
    #[error("task failed: {reason}")]
    TaskFailed {
        /// Human-readable explanation, surfaced as `message`.
        reason: String,
        /// Optional spec-defined extension data; carried verbatim into
        /// [`ErrorPayload::details`].
        details: Option<Value>,
    },

    /// The consumer is temporarily unable to process the task.
    #[error("temporarily unavailable")]
    Unavailable {
        /// Optional retry-after hint, surfaced as `retryAfter`.
        retry_after: Option<DateTime<Utc>>,
    },

    /// The consumer encountered an unexpected internal failure.
    #[error("internal error: {reason}")]
    InternalError {
        /// Human-readable explanation, surfaced as `message`.
        reason: String,
    },
}

impl RejectReason {
    /// The [`StandardCode`] that corresponds to this rejection.
    pub fn code(&self) -> StandardCode {
        match self {
            RejectReason::MalformedRequest { .. } => StandardCode::MalformedRequest,
            RejectReason::UnsupportedType { .. } => StandardCode::UnsupportedType,
            RejectReason::UnsupportedVersion { .. } => StandardCode::UnsupportedVersion,
            RejectReason::Expired { .. } => StandardCode::Expired,
            RejectReason::ProofRequired => StandardCode::ProofRequired,
            RejectReason::ProofInvalid { .. } => StandardCode::ProofInvalid,
            RejectReason::PermissionDenied { .. } => StandardCode::PermissionDenied,
            RejectReason::WrongRecipient { .. } => StandardCode::WrongRecipient,
            RejectReason::IdentityMismatch(_) => StandardCode::IdentityMismatch,
            RejectReason::TaskFailed { .. } => StandardCode::TaskFailed,
            RejectReason::Unavailable { .. } => StandardCode::Unavailable,
            RejectReason::InternalError { .. } => StandardCode::InternalError,
        }
    }

    /// Message safe to attach to a wire-serialised [`ErrorPayload`].
    ///
    /// SPEC.md §8.1 (final paragraph) and §10.4 require error-response
    /// messages to be free of consumer-side authentication context. The
    /// [`Display`](std::fmt::Display) implementation (and `to_string()`)
    /// is intentionally chatty for *diagnostic* purposes — it names both the
    /// in-band and the transport-authenticated identities under
    /// [`Self::IdentityMismatch`] and the consumer's own VID under
    /// [`Self::WrongRecipient`], which makes log lines actionable but is
    /// exactly the identity oracle a wire-exposed message must not provide.
    ///
    /// Variants whose `Display` is already consumer-side-neutral
    /// (`Expired`, `ProofInvalid`, …) return the same string the `Display`
    /// would. The variants that *do* leak identity return a sanitised
    /// constant.
    pub fn wire_message(&self) -> String {
        match self {
            // Identity-bearing rejections: sanitised constants. The
            // transport-derived and in-band values stay in logs only.
            RejectReason::IdentityMismatch(_) => {
                "in-band identity does not match transport-derived identity".to_string()
            }
            RejectReason::WrongRecipient { .. } => {
                "document recipient does not identify this consumer".to_string()
            }
            // All other variants carry no consumer-side authentication
            // context — their `Display` is safe for the wire.
            other => other.to_string(),
        }
    }
}

impl From<RejectReason> for ErrorPayload {
    fn from(reason: RejectReason) -> Self {
        let code = reason.code();
        let mut payload = ErrorPayload::new(code).with_message(reason.wire_message());
        match reason {
            RejectReason::Unavailable {
                retry_after: Some(when),
            } => {
                payload = payload.with_retry_after(when);
            }
            RejectReason::TaskFailed {
                details: Some(d), ..
            } => {
                payload = payload.with_details(d);
            }
            _ => {}
        }
        payload
    }
}

fn parse_standard(s: &str) -> Result<StandardCode, ParseCodeError> {
    Ok(match s {
        // Framework 0.2 lowerCamelCase spellings (emitted form) and the
        // framework 0.1 snake_case spellings are both accepted, so a 0.2
        // consumer can still read an error response from a 0.1 peer.
        "malformedRequest" | "malformed_request" => StandardCode::MalformedRequest,
        "unsupportedType" | "unsupported_type" => StandardCode::UnsupportedType,
        "unsupportedVersion" | "unsupported_version" => StandardCode::UnsupportedVersion,
        "expired" => StandardCode::Expired,
        "proofRequired" | "proof_required" => StandardCode::ProofRequired,
        "proofInvalid" | "proof_invalid" => StandardCode::ProofInvalid,
        "permissionDenied" | "permission_denied" => StandardCode::PermissionDenied,
        "wrongRecipient" | "wrong_recipient" => StandardCode::WrongRecipient,
        "idConflict" | "id_conflict" => StandardCode::IdConflict,
        "identityMismatch" | "identity_mismatch" => StandardCode::IdentityMismatch,
        "taskFailed" | "task_failed" => StandardCode::TaskFailed,
        "unavailable" => StandardCode::Unavailable,
        "internalError" | "internal_error" => StandardCode::InternalError,
        // A bare token that doesn't match a standard code is treated as a
        // malformed extension (extensions require a colon-namespaced slug).
        other => return Err(ParseCodeError::InvalidLocal(other.to_string())),
    })
}

fn validate_slug(slug: &str) -> Result<(), ()> {
    if slug.is_empty() {
        return Err(());
    }
    for segment in slug.split('/') {
        validate_segment(segment).ok_or(())?;
    }
    Ok(())
}

fn validate_segment(seg: &str) -> Option<()> {
    let mut chars = seg.chars();
    let first = chars.next()?;
    if !first.is_ascii_lowercase() {
        return None;
    }
    let mut prev_hyphen = false;
    for c in chars {
        match c {
            'a'..='z' | '0'..='9' => prev_hyphen = false,
            '-' => {
                if prev_hyphen {
                    return None;
                }
                prev_hyphen = true;
            }
            _ => return None,
        }
    }
    if prev_hyphen {
        return None;
    }
    Some(())
}

fn validate_local(local: &str) -> Result<(), ()> {
    // The local portion of an extended code (`<slug>:<local>`) must start with
    // a lowercase letter, then permits letters, digits, and underscores —
    // matching the `errorCodes` pattern in spec.meta.schema.json. Uppercase is
    // accepted so framework 0.2 lowerCamelCase locals (e.g. `documentRevoked`)
    // parse; underscores are accepted so frozen framework 0.1 snake_case locals
    // (e.g. `document_revoked`) still parse.
    let mut chars = local.chars();
    let first = chars.next().ok_or(())?;
    if !first.is_ascii_lowercase() {
        return Err(());
    }
    for c in chars {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {}
            _ => return Err(()),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_standard_codes() {
        for code in [
            StandardCode::MalformedRequest,
            StandardCode::UnsupportedType,
            StandardCode::UnsupportedVersion,
            StandardCode::Expired,
            StandardCode::ProofRequired,
            StandardCode::ProofInvalid,
            StandardCode::PermissionDenied,
            StandardCode::WrongRecipient,
            StandardCode::IdentityMismatch,
            StandardCode::TaskFailed,
            StandardCode::Unavailable,
            StandardCode::InternalError,
        ] {
            let wire = code.as_str();
            let parsed: TrustTaskCode = wire.parse().unwrap();
            assert_eq!(parsed, TrustTaskCode::Standard(code));
            assert_eq!(parsed.to_string(), wire);
        }
    }

    #[test]
    fn parses_legacy_snake_case_codes_and_re_emits_camel() {
        // Framework 0.1 emitted snake_case codes; a 0.2 consumer must still
        // parse those (lenient parse) while re-emitting the 0.2 camelCase form
        // so the wire moves forward. This pins the whole 0.1-compat arm of
        // `parse_standard`, which was otherwise untested.
        let legacy = [
            ("malformed_request", StandardCode::MalformedRequest),
            ("unsupported_type", StandardCode::UnsupportedType),
            ("unsupported_version", StandardCode::UnsupportedVersion),
            ("proof_required", StandardCode::ProofRequired),
            ("proof_invalid", StandardCode::ProofInvalid),
            ("permission_denied", StandardCode::PermissionDenied),
            ("wrong_recipient", StandardCode::WrongRecipient),
            ("identity_mismatch", StandardCode::IdentityMismatch),
            ("task_failed", StandardCode::TaskFailed),
            ("internal_error", StandardCode::InternalError),
        ];
        for (snake, code) in legacy {
            let parsed: TrustTaskCode = snake.parse().unwrap();
            assert_eq!(parsed, TrustTaskCode::Standard(code), "parse {snake}");
            // Re-emits the 0.2 camelCase spelling, not the snake input.
            assert_eq!(parsed.to_string(), code.as_str());
            assert_ne!(
                parsed.to_string(),
                snake,
                "{snake} should re-emit as camelCase"
            );
        }
    }

    #[test]
    fn deserializes_legacy_snake_error_payload_and_re_emits_camel() {
        // A trust-task-error/0.1 payload with a snake_case code deserializes,
        // its code resolves to the camelCase StandardCode, and re-serializing
        // emits the 0.2 spelling.
        let payload: ErrorPayload =
            serde_json::from_str(r#"{"code":"proof_invalid","retryable":false}"#).unwrap();
        assert_eq!(payload.code, StandardCode::ProofInvalid.into());
        let out = serde_json::to_value(&payload).unwrap();
        assert_eq!(out["code"], "proofInvalid");
    }

    #[test]
    fn parses_extension_code() {
        let parsed: TrustTaskCode = "kyc-handoff:document_revoked".parse().unwrap();
        assert_eq!(
            parsed,
            TrustTaskCode::Extended {
                slug: "kyc-handoff".to_string(),
                local: "document_revoked".to_string(),
            }
        );
        assert_eq!(parsed.to_string(), "kyc-handoff:document_revoked");
    }

    #[test]
    fn parses_hierarchical_extension_code() {
        let parsed: TrustTaskCode = "acl/grant:permission_denied".parse().unwrap();
        assert!(matches!(
            parsed,
            TrustTaskCode::Extended { ref slug, ref local }
            if slug == "acl/grant" && local == "permission_denied"
        ));
    }

    #[test]
    fn rejects_invalid_namespace() {
        let err: ParseCodeError = "Bad:code".parse::<TrustTaskCode>().unwrap_err();
        assert!(matches!(err, ParseCodeError::InvalidNamespace(s) if s == "Bad"));
    }

    #[test]
    fn default_retryable_matches_spec_table() {
        assert!(!StandardCode::MalformedRequest.default_retryable());
        assert!(!StandardCode::Expired.default_retryable());
        assert!(StandardCode::Unavailable.default_retryable());
        assert!(StandardCode::InternalError.default_retryable());
    }

    #[test]
    fn serializes_payload_as_json() {
        let payload = ErrorPayload {
            code: StandardCode::Expired.into(),
            in_response_to: None,
            message: Some("expired".to_string()),
            retryable: false,
            retry_after: None,
            details: None,
        };
        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "code": "expired",
                "message": "expired",
                "retryable": false
            })
        );
    }

    #[test]
    fn new_payload_takes_default_retryable() {
        let p = ErrorPayload::new(StandardCode::Expired);
        assert!(!p.retryable);
        let p = ErrorPayload::new(StandardCode::Unavailable);
        assert!(p.retryable);
    }

    #[test]
    fn builder_methods_compose() {
        let when: DateTime<Utc> = "2026-05-17T00:00:00Z".parse().unwrap();
        let payload = ErrorPayload::new(StandardCode::Unavailable)
            .with_message("nodes draining")
            .with_retry_after(when)
            .with_details(serde_json::json!({ "drain_eta": "30s" }));

        assert_eq!(payload.message.as_deref(), Some("nodes draining"));
        assert_eq!(payload.retry_after, Some(when));
        assert!(payload.retryable);
        assert!(payload.details.is_some());
    }

    #[test]
    fn effective_code_falls_back_for_extensions() {
        // SPEC §8.5: unrecognized extension codes are treated as task_failed
        // by consumers that don't implement the originating spec.
        let payload = ErrorPayload::new(TrustTaskCode::Extended {
            slug: "kyc-handoff".into(),
            local: "document_revoked".into(),
        });
        assert_eq!(payload.effective_code(), StandardCode::TaskFailed);

        let payload = ErrorPayload::new(StandardCode::Expired);
        assert_eq!(payload.effective_code(), StandardCode::Expired);
    }

    #[test]
    fn should_retry_at_respects_retryable_flag() {
        let now: DateTime<Utc> = "2026-05-17T12:00:00Z".parse().unwrap();
        let p = ErrorPayload::new(StandardCode::Expired);
        assert!(!p.should_retry_at(now));
    }

    #[test]
    fn should_retry_at_waits_for_retry_after() {
        let later: DateTime<Utc> = "2026-05-17T12:00:00Z".parse().unwrap();
        let now: DateTime<Utc> = "2026-05-17T11:59:00Z".parse().unwrap();
        let p = ErrorPayload::new(StandardCode::Unavailable).with_retry_after(later);
        assert!(!p.should_retry_at(now));
        assert!(p.should_retry_at(later));
    }

    #[test]
    fn reject_reason_maps_to_correct_code() {
        let cases: &[(RejectReason, StandardCode)] = &[
            (
                RejectReason::MalformedRequest { reason: "x".into() },
                StandardCode::MalformedRequest,
            ),
            (RejectReason::ProofRequired, StandardCode::ProofRequired),
            (
                RejectReason::Unavailable { retry_after: None },
                StandardCode::Unavailable,
            ),
        ];
        for (reason, expected) in cases {
            assert_eq!(reason.code(), *expected);
        }
    }

    #[test]
    fn reject_reason_into_payload_carries_message_and_details() {
        let payload: ErrorPayload = RejectReason::Expired {
            expires_at: "2026-04-12T09:31:00Z".parse().unwrap(),
        }
        .into();
        assert_eq!(payload.code, StandardCode::Expired.into());
        assert!(payload.message.as_deref().unwrap().contains("2026-04-12"));
        assert!(!payload.retryable);

        let payload: ErrorPayload = RejectReason::TaskFailed {
            reason: "downstream rejected".into(),
            details: Some(serde_json::json!({ "trace": "abc" })),
        }
        .into();
        assert!(payload.details.is_some());
    }

    #[test]
    fn consistency_error_flows_into_reject_reason_via_question_mark() {
        fn go() -> Result<(), RejectReason> {
            Err(ConsistencyError::IssuerMismatch {
                in_band: "did:web:a".into(),
                transport: "did:web:b".into(),
            })?;
            Ok(())
        }
        let err = go().unwrap_err();
        assert_eq!(err.code(), StandardCode::IdentityMismatch);
    }
}
