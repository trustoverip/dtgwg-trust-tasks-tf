//! Runtime JSON Schema validation for Trust Task payloads.
//!
//! Available behind the `validate` Cargo feature. The codegen inlines each
//! spec's `payload.schema.json` as a string constant and emits a
//! [`ValidatedPayload`] impl so callers can validate any inbound value
//! against the on-the-wire schema, not just the structural shape serde
//! recovers.
//!
//! ```ignore
//! # #[cfg(feature = "validate")]
//! # fn demo() -> Result<(), trust_tasks_rs::validate::ValidationError> {
//! use trust_tasks_rs::{specs::acl::grant::v0_2, validate::ValidatedPayload};
//!
//! let value: serde_json::Value = serde_json::json!({
//!     "entry": {
//!         "subject": "did:web:alice.example",
//!         "role": "admin",
//!         "act": { "scope": "all" },
//!         "keys": { "scope": "all" },
//!         "capabilities": { "scope": "ceiling" }
//!     }
//! });
//! v0_2::Payload::validate_value(&value)?;
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
    /// Validate `value` against [`Payload::PAYLOAD_SCHEMA`].
    ///
    /// Returns [`ValidationError`] with the collected schema-validation
    /// errors when the value does not conform. Where the payload type
    /// carries no schema (`PAYLOAD_SCHEMA` is `None` — the hand-modelled
    /// `trust-task-error`), there is nothing to check and this succeeds:
    /// the Rust type the value deserialized into is itself the constraint.
    ///
    /// The compiled schema is cached process-wide (see [`SchemaValidator`]),
    /// so only the first call for a type pays for compiling it.
    fn validate_value(value: &Value) -> Result<(), ValidationError> {
        match Self::PAYLOAD_SCHEMA {
            Some(schema) => SHARED.check(schema, value),
            None => Ok(()),
        }
    }
}

/// Every [`Payload`] is validatable, because every `Payload` carries its
/// schema (or states that it has none).
///
/// Before 0.9.0 this trait declared its own `SCHEMA_JSON` const and the
/// codegen emitted one impl per generated type. The schema now lives on
/// `Payload` itself — reachable without the `validate` feature, since a
/// `&'static str` costs nothing — and this blanket impl replaces roughly
/// three hundred generated impl blocks.
impl<P: Payload> ValidatedPayload for P {}

/// Compile `schema_json` as a JSON Schema and validate `value` against it.
///
/// Compilation is performed every call. To validate many documents, use a
/// [`SchemaValidator`], which compiles each schema once and keeps it.
///
/// # ⚠ Schema-validation DoS surface (SPEC §12.3)
///
/// `schema_json` MUST be trusted by the caller. The underlying
/// `jsonschema` crate compiles `pattern` keywords through a regex engine
/// that supports backtracking, so a malicious schema can carry a
/// `pattern` such as `"^(a+)+$"` and validation will consume unbounded
/// CPU on otherwise-innocuous strings.
///
/// The codegen-emitted [`ValidatedPayload`] impls satisfy this rule
/// trivially because the schemas are inlined as string constants from the
/// repo at codegen time and frozen at build time. If you call this function directly
/// with a schema obtained from any other source — content-negotiating
/// a URI over the network, accepting a private-spec submission per
/// SPEC §6.5, etc. — you MUST authenticate the schema's source and
/// SHOULD apply a per-validation timeout outside this call.
pub fn against_schema(schema_json: &str, value: &Value) -> Result<(), ValidationError> {
    run(&compile(schema_json)?, value)
}

fn compile(schema_json: &str) -> Result<jsonschema::Validator, ValidationError> {
    let schema_value: Value =
        serde_json::from_str(schema_json).map_err(ValidationError::schema_parse)?;
    jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .build(&schema_value)
        .map_err(|e| ValidationError::schema_compile(e.to_string()))
}

fn run(compiled: &jsonschema::Validator, value: &Value) -> Result<(), ValidationError> {
    let messages: Vec<String> = compiled.iter_errors(value).map(|e| e.to_string()).collect();
    if !messages.is_empty() {
        return Err(ValidationError::instance(messages));
    }
    Ok(())
}

/// The cache [`ValidatedPayload::validate_value`] and
/// [`DeclaredErrorCode::check_details`](crate::DeclaredErrorCode::check_details) share.
pub(crate) static SHARED: SchemaValidator = SchemaValidator::new();

/// A [`PayloadValidator`] over the `jsonschema` crate (JSON Schema draft
/// 2020-12) that compiles each schema once and keeps it — the validator
/// [`PayloadPolicy::Validate`] wants.
///
/// The schemas it is handed are `Payload::PAYLOAD_SCHEMA` constants, which
/// never move; the compiled validator is cached by the schema text's address
/// and confirmed against its content, so a document pays for evaluation only,
/// never for parsing and compiling a schema of tens of KB and its `pattern`
/// regexes. One instance serves every task; keep it for the life of the
/// consumer — a `static` works:
///
/// ```rust
/// use trust_tasks_rs::{validate::SchemaValidator, PayloadPolicy};
///
/// static VALIDATOR: SchemaValidator = SchemaValidator::new();
/// let policy = PayloadPolicy::Validate(&VALIDATOR);
/// # let _ = policy;
/// ```
///
/// The SPEC §12.3 caution on [`against_schema`] applies: hand it only
/// schemas whose source you trust.
///
/// [`PayloadValidator`]: crate::PayloadValidator
/// [`PayloadPolicy::Validate`]: crate::PayloadPolicy::Validate
pub struct SchemaValidator {
    compiled: crate::schema_cache::SchemaCache<jsonschema::Validator>,
}

impl SchemaValidator {
    /// An empty cache.
    pub const fn new() -> Self {
        Self {
            compiled: crate::schema_cache::SchemaCache::new(),
        }
    }

    /// Validate `value` against `schema_json`, compiling the schema on first
    /// use.
    pub fn check(&self, schema_json: &str, value: &Value) -> Result<(), ValidationError> {
        let compiled = self.compiled.get_or_try_insert(schema_json, compile)?;
        run(&compiled, value)
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for SchemaValidator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SchemaValidator").finish_non_exhaustive()
    }
}

impl crate::PayloadValidator for SchemaValidator {
    fn validate(&self, schema_json: &str, payload: &Value) -> Result<(), String> {
        self.check(schema_json, payload).map_err(|e| e.to_string())
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PayloadValidator;
    use serde_json::json;

    static SCHEMA: &str = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": ["name"],
        "properties": { "name": { "type": "string", "pattern": "^[a-z]+$" } }
    }"#;

    #[test]
    fn schema_validator_accepts_and_refuses_as_against_schema_does() {
        let validator = SchemaValidator::new();
        for value in [json!({"name": "ok"}), json!({"name": "NO"}), json!({})] {
            assert_eq!(
                validator.check(SCHEMA, &value).is_ok(),
                against_schema(SCHEMA, &value).is_ok(),
                "{value}"
            );
        }
        assert!(validator.validate(SCHEMA, &json!({"name": "ok"})).is_ok());
        let reason = validator.validate(SCHEMA, &json!({})).unwrap_err();
        assert!(
            reason.contains("payload failed schema validation"),
            "{reason}"
        );
    }

    #[test]
    fn schema_validator_compiles_each_schema_once() {
        let validator = SchemaValidator::new();
        for _ in 0..3 {
            validator.check(SCHEMA, &json!({"name": "ok"})).unwrap();
        }
        assert_eq!(validator.compiled.len(), 1);
    }

    #[test]
    fn a_schema_that_does_not_parse_is_an_error_every_time() {
        let validator = SchemaValidator::new();
        for _ in 0..2 {
            let err = validator.check("{", &json!({})).unwrap_err();
            assert!(err.to_string().contains("did not parse"), "{err}");
        }
        assert_eq!(validator.compiled.len(), 0);
    }
}
