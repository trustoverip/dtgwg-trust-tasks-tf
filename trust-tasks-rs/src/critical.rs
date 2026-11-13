//! SPEC §4.5.2 — namespaces a producer marks critical with `extCritical`.
//!
//! An `ext` namespace is ignorable by default (§4.5.1 rule 5). A specification
//! may let a producer list some of them in a sibling `extCritical` array, and
//! a consumer that does not recognize a namespace listed there **MUST NOT**
//! process the document and **MUST** reject it with `unsupportedExtension`
//! (§4.5.2 rule 4). An entry naming a namespace absent from the sibling `ext`
//! is non-conforming and is rejected with `malformedRequest` (rule 2).
//!
//! [`check_critical_extensions`] applies both rules to a payload. The consume
//! pipelines run it after the proof and per-specification checks and before
//! the §7.2 item 11 record, with the namespaces the consumer names in
//! [`ConsumeChecks::with_recognized_extensions`](crate::ConsumeChecks::with_recognized_extensions).
//!
//! # Where a marking is read
//!
//! `extCritical` exists only at the object levels a specification opts in at,
//! so the payload is walked alongside its schema:
//!
//! * a level whose schema declares an `extCritical` property is checked;
//! * a level whose schema closes it (`additionalProperties: false` or
//!   `unevaluatedProperties: false`) without declaring it is refused as
//!   `malformedRequest` when the member is present — what a schema validator
//!   reports, applied even under
//!   [`PayloadPolicy::AcceptUnvalidated`](crate::PayloadPolicy::AcceptUnvalidated)
//!   so the marking is never dropped silently;
//! * a level the schema leaves open is data (a document a vault is asked to
//!   sign, say), and a member named `extCritical` in it is not a marking.
//!
//! Every branch of `allOf` / `anyOf` / `oneOf` is followed, so a level is
//! checked where any branch declares the member. With no schema to walk, every
//! level outside an `ext` namespace's own content is checked.

use std::collections::BTreeSet;

use serde_json::{Map, Value};

use crate::error::RejectReason;

/// How deep the walk follows a payload. Deeper levels are not reached by any
/// specification in the registry.
const MAX_DEPTH: usize = 128;

/// Base work allowance for one walk, in steps (a value visited against a
/// subschema); [`STEPS_PER_VALUE`] more are allowed for every value in the
/// payload. A walk that runs out refuses the document rather than pass a
/// marking it did not reach.
const BASE_STEPS: usize = 100_000;
/// See [`BASE_STEPS`].
const STEPS_PER_VALUE: usize = 64;

/// The member name §4.5.2 defines.
const EXT_CRITICAL: &str = "extCritical";

/// The longest namespace echoed in an `unsupportedExtension` message.
const MAX_ECHOED_NAMESPACE: usize = 128;

/// Apply SPEC §4.5.2 to `payload`.
///
/// `schema_json` is the payload schema of the specification the document is
/// consumed under ([`Payload::PAYLOAD_SCHEMA`](crate::Payload::PAYLOAD_SCHEMA)),
/// and decides which object levels can carry a marking (see the module docs).
/// `recognized` is every namespace this consumer implements.
///
/// Returns [`RejectReason::UnsupportedExtension`] for the first critical
/// namespace not in `recognized`, and [`RejectReason::MalformedRequest`] for a
/// marking that breaks rules 1 and 2 or sits at a level that does not allow
/// it. A payload with no member named `extCritical` anywhere returns `Ok`
/// after one pass over it.
pub fn check_critical_extensions(
    payload: &Value,
    schema_json: Option<&str>,
    recognized: &[&str],
) -> Result<(), RejectReason> {
    let values = match count_values_if_marked(payload) {
        Some(n) => n,
        None => return Ok(()),
    };
    let root = schema_json.and_then(crate::schema_cache::parsed);
    let mut walk = Walk {
        recognized,
        root: root.as_deref(),
        left: BASE_STEPS.saturating_add(STEPS_PER_VALUE.saturating_mul(values)),
    };
    match walk.root {
        Some(root) => walk.guided(payload, &[root], 0),
        None => walk.blind(payload, 0),
    }
}

/// The number of values in `payload` when a member named `extCritical`
/// appears anywhere in it, or `None` when none does.
fn count_values_if_marked(payload: &Value) -> Option<usize> {
    let mut n = 0usize;
    let mut marked = false;
    let mut stack = vec![payload];
    while let Some(v) = stack.pop() {
        n += 1;
        match v {
            Value::Object(members) => {
                marked |= members.contains_key(EXT_CRITICAL);
                stack.extend(members.values());
            }
            Value::Array(elements) => stack.extend(elements.iter()),
            _ => {}
        }
    }
    marked.then_some(n)
}

struct Walk<'a> {
    recognized: &'a [&'a str],
    root: Option<&'a Value>,
    left: usize,
}

fn too_complex() -> RejectReason {
    RejectReason::MalformedRequest {
        reason: "payload is too deeply nested or too complex to check its critical extensions \
                 (SPEC §4.5.2)"
            .to_string(),
    }
}

impl<'a> Walk<'a> {
    fn spend(&mut self, depth: usize) -> Result<(), RejectReason> {
        if depth > MAX_DEPTH {
            return Err(too_complex());
        }
        self.left = self.left.checked_sub(1).ok_or_else(too_complex)?;
        Ok(())
    }

    /// No schema: every level is checked, and an `ext` member's content is
    /// skipped, being opaque to the framework (§4.5.1 rule 3).
    fn blind(&mut self, value: &Value, depth: usize) -> Result<(), RejectReason> {
        self.spend(depth)?;
        match value {
            Value::Object(members) => {
                if members.contains_key(EXT_CRITICAL) {
                    self.check_level(members)?;
                }
                for (name, member) in members {
                    if name != "ext" && name != EXT_CRITICAL {
                        self.blind(member, depth + 1)?;
                    }
                }
                Ok(())
            }
            Value::Array(elements) => {
                for element in elements {
                    self.blind(element, depth + 1)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Walk `value` against every schema in `schemas` at once.
    fn guided(
        &mut self,
        value: &Value,
        schemas: &[&'a Value],
        depth: usize,
    ) -> Result<(), RejectReason> {
        self.spend(depth)?;
        if !matches!(value, Value::Object(_) | Value::Array(_)) {
            return Ok(());
        }
        let mut applicable = Vec::new();
        for schema in schemas {
            self.expand(schema, &mut applicable, depth)?;
        }
        if applicable.is_empty() {
            return Ok(());
        }
        match value {
            Value::Object(members) => {
                if members.contains_key(EXT_CRITICAL) {
                    let declared = applicable
                        .iter()
                        .any(|s| property(s, EXT_CRITICAL).is_some());
                    if declared {
                        self.check_level(members)?;
                    } else if applicable.iter().any(|s| closes(s)) {
                        return Err(RejectReason::MalformedRequest {
                            reason: "`extCritical` is not allowed at this level of the payload \
                                     (SPEC §4.5.2)"
                                .to_string(),
                        });
                    }
                }
                for (name, member) in members {
                    if name == EXT_CRITICAL {
                        continue;
                    }
                    let children = member_schemas(&applicable, name);
                    if !children.is_empty() {
                        self.guided(member, &children, depth + 1)?;
                    }
                }
                Ok(())
            }
            Value::Array(elements) => {
                for (i, element) in elements.iter().enumerate() {
                    let children = element_schemas(&applicable, i);
                    if !children.is_empty() {
                        self.guided(element, &children, depth + 1)?;
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Collect into `out` the object schemas that apply where `schema` does:
    /// itself, a local `$ref` target, and every `allOf` / `anyOf` / `oneOf`
    /// branch, recursively.
    fn expand(
        &mut self,
        schema: &'a Value,
        out: &mut Vec<&'a Value>,
        depth: usize,
    ) -> Result<(), RejectReason> {
        self.spend(depth)?;
        let Some(object) = schema.as_object() else {
            return Ok(());
        };
        if out.iter().any(|s| std::ptr::eq(*s, schema)) {
            return Ok(());
        }
        out.push(schema);
        if let (Some(reference), Some(root)) =
            (object.get("$ref").and_then(Value::as_str), self.root)
        {
            if let Some(target) = resolve(reference, root) {
                self.expand(target, out, depth + 1)?;
            }
        }
        for keyword in ["allOf", "anyOf", "oneOf"] {
            if let Some(branches) = object.get(keyword).and_then(Value::as_array) {
                for branch in branches {
                    self.expand(branch, out, depth + 1)?;
                }
            }
        }
        Ok(())
    }

    /// Rules 1 and 2, then rule 4, at one object level.
    fn check_level(&self, level: &Map<String, Value>) -> Result<(), RejectReason> {
        let malformed = |reason: &str| RejectReason::MalformedRequest {
            reason: format!("{reason} (SPEC §4.5.2)"),
        };
        let Some(Value::Array(entries)) = level.get(EXT_CRITICAL) else {
            return Err(malformed("`extCritical` must be an array"));
        };
        if entries.is_empty() {
            return Err(malformed("`extCritical` must name at least one namespace"));
        }
        let ext = match level.get("ext") {
            Some(Value::Object(ext)) => ext,
            _ => {
                return Err(malformed(
                    "`extCritical` is present where there is no sibling `ext` object",
                ))
            }
        };
        let mut seen = BTreeSet::new();
        let mut names = Vec::with_capacity(entries.len());
        for entry in entries {
            let Some(name) = entry.as_str() else {
                return Err(malformed("every `extCritical` entry must be a string"));
            };
            if !is_namespace(name) {
                return Err(malformed(
                    "every `extCritical` entry must be a reverse-DNS namespace",
                ));
            }
            if !seen.insert(name) {
                return Err(malformed("`extCritical` entries must be unique"));
            }
            if !ext.contains_key(name) {
                return Err(malformed(
                    "every `extCritical` entry must name a namespace present in the sibling `ext`",
                ));
            }
            names.push(name);
        }
        match names.into_iter().find(|n| !self.recognized.contains(n)) {
            Some(name) => Err(RejectReason::UnsupportedExtension {
                namespace: name.chars().take(MAX_ECHOED_NAMESPACE).collect(),
            }),
            None => Ok(()),
        }
    }
}

/// The schema `schema` gives the member `name`, if it names one.
fn property<'s>(schema: &'s Value, name: &str) -> Option<&'s Value> {
    schema.get("properties")?.get(name)
}

/// Whether `schema` refuses members it does not name.
fn closes(schema: &Value) -> bool {
    schema.get("additionalProperties") == Some(&Value::Bool(false))
        || schema.get("unevaluatedProperties") == Some(&Value::Bool(false))
}

/// The schemas that apply to member `name` of an object every schema in
/// `applicable` applies to. `patternProperties` are taken whole, since this
/// walk has no regex engine: a schema there is only ever an extra place a
/// marking may be read.
fn member_schemas<'s>(applicable: &[&'s Value], name: &str) -> Vec<&'s Value> {
    let mut out = Vec::new();
    for schema in applicable {
        if let Some(s) = property(schema, name) {
            out.push(s);
            continue;
        }
        if let Some(patterns) = schema.get("patternProperties").and_then(Value::as_object) {
            out.extend(patterns.values());
        }
        if let Some(s @ Value::Object(_)) = schema.get("additionalProperties") {
            out.push(s);
        }
    }
    out
}

/// The schemas that apply to element `index` of an array.
fn element_schemas<'s>(applicable: &[&'s Value], index: usize) -> Vec<&'s Value> {
    let mut out = Vec::new();
    for schema in applicable {
        match schema
            .get("prefixItems")
            .and_then(Value::as_array)
            .and_then(|p| p.get(index))
        {
            Some(s) => out.push(s),
            None => {
                if let Some(s @ Value::Object(_)) = schema.get("items") {
                    out.push(s);
                }
            }
        }
    }
    out
}

/// Resolve a local `$ref` against `root`.
fn resolve<'r>(reference: &str, root: &'r Value) -> Option<&'r Value> {
    let pointer = reference.strip_prefix('#')?;
    if pointer.is_empty() {
        return Some(root);
    }
    root.pointer(pointer)
}

/// §4.5.1 rule 2: `^[a-z][a-z0-9-]*(\.[a-z0-9-]+)+$`.
fn is_namespace(name: &str) -> bool {
    let mut labels = name.split('.');
    let Some(first) = labels.next() else {
        return false;
    };
    let mut first_chars = first.bytes();
    if !matches!(first_chars.next(), Some(b'a'..=b'z')) {
        return false;
    }
    let label_ok = |b: u8| matches!(b, b'a'..=b'z' | b'0'..=b'9' | b'-');
    if !first_chars.all(label_ok) {
        return false;
    }
    let mut more = 0;
    for label in labels {
        if label.is_empty() || !label.bytes().all(label_ok) {
            return false;
        }
        more += 1;
    }
    more > 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::StandardCode;
    use serde_json::json;

    /// A schema with a marking allowed at the root and in `item`, a closed
    /// `other` level, and an open `data` level.
    fn schema() -> String {
        json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "ext": { "$ref": "#/$defs/Ext" },
                "extCritical": { "$ref": "#/$defs/ExtCritical" },
                "item": { "$ref": "#/$defs/Item" },
                "other": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": { "a": { "type": "string" } }
                },
                "data": { "type": "object" },
                "list": { "type": "array", "items": { "$ref": "#/$defs/Item" } }
            },
            "$defs": {
                "Ext": { "type": "object" },
                "ExtCritical": { "type": "array", "items": { "type": "string" } },
                "Item": {
                    "oneOf": [
                        {
                            "type": "object",
                            "additionalProperties": false,
                            "properties": {
                                "ext": { "$ref": "#/$defs/Ext" },
                                "extCritical": { "$ref": "#/$defs/ExtCritical" }
                            }
                        },
                        { "type": "string" }
                    ]
                }
            }
        })
        .to_string()
    }

    fn check(payload: Value, recognized: &[&str]) -> Result<(), RejectReason> {
        check_critical_extensions(&payload, Some(&schema()), recognized)
    }

    fn code(r: Result<(), RejectReason>) -> Option<StandardCode> {
        r.err().map(|e| e.code())
    }

    #[test]
    fn a_payload_without_a_marking_passes() {
        assert!(check(json!({ "ext": { "org.example.a": 1 } }), &[]).is_ok());
    }

    #[test]
    fn an_unrecognized_critical_namespace_is_refused() {
        let r = check(
            json!({ "ext": { "org.example.a": 1 }, "extCritical": ["org.example.a"] }),
            &[],
        );
        assert_eq!(
            r,
            Err(RejectReason::UnsupportedExtension {
                namespace: "org.example.a".to_string()
            })
        );
        assert_eq!(
            code(r),
            Some(StandardCode::UnsupportedExtension),
            "rule 4 answers unsupportedExtension"
        );
    }

    #[test]
    fn a_recognized_critical_namespace_passes() {
        let payload = json!({
            "ext": { "org.example.a": 1, "org.example.b": 2 },
            "extCritical": ["org.example.a"]
        });
        assert!(check(payload, &["org.example.a"]).is_ok());
    }

    #[test]
    fn nested_and_array_levels_are_checked() {
        let nested = json!({
            "item": { "ext": { "org.example.a": 1 }, "extCritical": ["org.example.a"] }
        });
        assert_eq!(
            code(check(nested, &[])),
            Some(StandardCode::UnsupportedExtension)
        );
        let listed = json!({
            "list": ["x", { "ext": { "org.example.a": 1 }, "extCritical": ["org.example.a"] }]
        });
        assert_eq!(
            code(check(listed, &[])),
            Some(StandardCode::UnsupportedExtension)
        );
    }

    #[test]
    fn rule_violations_are_malformed() {
        for payload in [
            json!({ "ext": { "org.example.a": 1 }, "extCritical": "org.example.a" }),
            json!({ "ext": { "org.example.a": 1 }, "extCritical": [] }),
            json!({ "extCritical": ["org.example.a"] }),
            json!({ "ext": { "org.example.a": 1 }, "extCritical": ["org.example.b"] }),
            json!({ "ext": { "org.example.a": 1 }, "extCritical": ["org.example.a", "org.example.a"] }),
            json!({ "ext": { "Bad": 1 }, "extCritical": ["Bad"] }),
            json!({ "ext": { "org.example.a": 1 }, "extCritical": [7] }),
        ] {
            assert_eq!(
                code(check(payload.clone(), &["org.example.a"])),
                Some(StandardCode::MalformedRequest),
                "{payload}"
            );
        }
    }

    #[test]
    fn a_marking_at_a_closed_level_that_does_not_declare_it_is_malformed() {
        let payload = json!({
            "other": { "a": "x", "ext": { "org.example.a": 1 }, "extCritical": ["org.example.a"] }
        });
        assert_eq!(
            code(check(payload, &["org.example.a"])),
            Some(StandardCode::MalformedRequest)
        );
    }

    #[test]
    fn a_member_named_ext_critical_inside_open_data_is_not_a_marking() {
        let payload = json!({
            "data": { "payload": { "ext": { "org.example.a": 1 }, "extCritical": ["org.example.a"] } }
        });
        assert!(check(payload, &[]).is_ok());
        let in_ext = json!({
            "ext": { "org.example.a": { "extCritical": ["org.example.z"] } }
        });
        assert!(check(in_ext, &[]).is_ok());
    }

    #[test]
    fn without_a_schema_every_level_but_ext_content_is_checked() {
        let nested = json!({
            "x": { "ext": { "org.example.a": 1 }, "extCritical": ["org.example.a"] }
        });
        assert_eq!(
            code(check_critical_extensions(&nested, None, &[])),
            Some(StandardCode::UnsupportedExtension)
        );
        let in_ext = json!({ "ext": { "org.example.a": { "extCritical": ["x"] } } });
        assert!(check_critical_extensions(&in_ext, None, &[]).is_ok());
    }

    #[test]
    fn a_payload_deeper_than_the_walk_is_refused() {
        let mut payload =
            json!({ "ext": { "org.example.a": 1 }, "extCritical": ["org.example.a"] });
        for _ in 0..(MAX_DEPTH + 8) {
            payload = json!({ "x": payload });
        }
        assert_eq!(
            code(check_critical_extensions(
                &payload,
                None,
                &["org.example.a"]
            )),
            Some(StandardCode::MalformedRequest)
        );
    }

    #[test]
    fn namespace_grammar() {
        for ok in ["org.example", "vnd.affinidi.webvh", "a.b", "a-1.b-2"] {
            assert!(is_namespace(ok), "{ok}");
        }
        for bad in [
            "org",
            "Org.example",
            "1org.example",
            "org..example",
            "org.",
            ".org",
            "",
        ] {
            assert!(!is_namespace(bad), "{bad}");
        }
    }
}
