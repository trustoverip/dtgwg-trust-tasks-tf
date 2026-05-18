//! Runtime JSON Schema validation for Trust Task payloads.
//!
//! Available behind the `validate` Cargo feature. The codegen embeds each
//! spec's `payload.schema.json` via `include_str!` and emits an
//! [`ValidatedPayload`] impl so callers can validate any inbound value
//! against the on-the-wire schema, not just the structural shape serde
//! recovers.
//!
//! ```ignore
//! # #[cfg(feature = "validate")]
//! # fn demo() -> Result<(), trust_tasks_rs::validate::ValidationError> {
//! use trust_tasks_rs::{specs::acl::grant::v0_1, validate::ValidatedPayload};
//!
//! let value: serde_json::Value = serde_json::json!({
//!     "entry": { "subject": "did:web:alice.example", "role": "admin" }
//! });
//! v0_1::Payload::validate_value(&value)?;
//! # Ok(())
//! # }
//! ```
//!
//! This is belt-and-suspenders: most schema constraints fall out of typify's
//! generated struct shapes, but cross-cutting checks (regex `pattern`,
//! `minItems` on optional arrays, conditional schemas) only hold up under a
//! true schema validator.

use std::fmt;

use serde_json::Value;
use thiserror::Error;

use crate::payload::Payload;

/// A payload type whose original JSON Schema is embedded at compile time and
/// available for runtime validation.
///
/// Implemented automatically by the codegen for every generated request and
/// response payload type.
pub trait ValidatedPayload: Payload {
    /// Raw text of the `payload.schema.json` (or extracted sub-schema)
    /// describing values of this type.
    const SCHEMA_JSON: &'static str;

    /// Validate `value` against [`SCHEMA_JSON`](Self::SCHEMA_JSON).
    ///
    /// Returns [`ValidationError`] with the collected schema-validation
    /// errors when the value does not conform. The default implementation
    /// dispatches to [`against_schema`]; override only if a payload needs
    /// special preprocessing.
    fn validate_value(value: &Value) -> Result<(), ValidationError> {
        against_schema(Self::SCHEMA_JSON, value)
    }
}

/// Compile `schema_json` as a JSON Schema and validate `value` against it.
///
/// Compilation is performed every call. Callers in a hot path should keep a
/// compiled schema around themselves; this helper is intended for the
/// once-per-request pattern that consumer pipelines use.
///
/// # ⚠ Schema-validation DoS surface (SPEC §10.3)
///
/// `schema_json` MUST be trusted by the caller. The underlying
/// `jsonschema` crate compiles `pattern` keywords through a regex engine
/// that supports backtracking, so a malicious schema can carry a
/// `pattern` such as `"^(a+)+$"` and validation will consume unbounded
/// CPU on otherwise-innocuous strings.
///
/// The codegen-emitted [`ValidatedPayload`] impls satisfy this rule
/// trivially because the schemas are `include_str!`-embedded from the
/// repo and frozen at build time. If you call this function directly
/// with a schema obtained from any other source — content-negotiating
/// a URI over the network, accepting a private-spec submission per
/// SPEC §6.5, etc. — you MUST authenticate the schema's source and
/// SHOULD apply a per-validation timeout outside this call.
pub fn against_schema(schema_json: &str, value: &Value) -> Result<(), ValidationError> {
    let schema_value: Value =
        serde_json::from_str(schema_json).map_err(ValidationError::schema_parse)?;
    let compiled = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .build(&schema_value)
        .map_err(|e| ValidationError::schema_compile(e.to_string()))?;

    let messages: Vec<String> = compiled.iter_errors(value).map(|e| e.to_string()).collect();
    if !messages.is_empty() {
        return Err(ValidationError::instance(messages));
    }
    Ok(())
}

/// One or more schema-validation problems encountered while checking a value.
#[derive(Debug, Error)]
pub struct ValidationError {
    kind: ErrorKind,
    messages: Vec<String>,
}

#[derive(Debug)]
enum ErrorKind {
    /// The embedded schema JSON failed to parse.
    SchemaParse,
    /// The schema parsed but failed to compile under Draft 2020-12.
    SchemaCompile,
    /// The instance failed validation against a well-formed schema.
    Instance,
}

impl ValidationError {
    fn schema_parse(e: serde_json::Error) -> Self {
        Self {
            kind: ErrorKind::SchemaParse,
            messages: vec![e.to_string()],
        }
    }

    fn schema_compile(message: String) -> Self {
        Self {
            kind: ErrorKind::SchemaCompile,
            messages: vec![message],
        }
    }

    fn instance(messages: Vec<String>) -> Self {
        Self {
            kind: ErrorKind::Instance,
            messages,
        }
    }

    /// The collected per-error messages from the validator, in source order.
    pub fn messages(&self) -> &[String] {
        &self.messages
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self.kind {
            ErrorKind::SchemaParse => "schema JSON did not parse",
            ErrorKind::SchemaCompile => "schema did not compile",
            ErrorKind::Instance => "payload failed schema validation",
        };
        write!(f, "{label}: {}", self.messages.join("; "))
    }
}
