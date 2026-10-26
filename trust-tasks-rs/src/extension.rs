//! Declared extension — reading a later version's document as the version a
//! consumer implements (SPEC §5.2 "Declared extension").
//!
//! A specification version may declare `extends: "<M.K>"` in its front matter:
//! it keeps every member, type, constraint and definition of `M.K` and only
//! *adds*. The build checks the promise. The specification then asks a
//! consumer that implements `M.K` to accept a document of the extending version
//! by ignoring the payload members it does not recognise — and asks it **not**
//! to do the same for a later version that declares nothing, because a draft
//! may break in a MINOR.
//!
//! This module is what makes that a library upgrade rather than an application
//! change. The generated table behind [`extends_for`] carries every
//! declaration, so an application that registered a handler for `x/0.2` accepts
//! an `x/0.3` that extends it once it builds against a library that knows so:
//!
//! * [`Dispatcher`](crate::Dispatcher) and
//!   [`AsyncDispatcher`](crate::AsyncDispatcher) route a document whose exact
//!   version has no handler to the highest registered version its chain
//!   reaches, pruning the payload on the way (see [`prune_payload`]);
//! * [`read_as`] does the same for one payload type, for a consumer that parses
//!   documents itself and calls [`consume_inbound`](crate::consume_inbound);
//! * [`consume_inbound`](crate::consume_inbound) accepts the pruned document
//!   under [`ProofPolicy::VerifyReceived`](crate::ProofPolicy::VerifyReceived)
//!   while handing the verifier the document **as received**.
//!
//! The document keeps the `type` it arrived with. A response built with
//! [`TrustTask::respond_with`] therefore answers `<received type>#response`:
//! the version the producer used. A response shaped for the implemented version
//! is valid for the received one, because the received one only added to it.
//!
//! # Proofs are never verified over a pruned document
//!
//! A proof signs the document the producer sent, extension members included.
//! Pruning changes the payload, so a typed document produced here cannot be
//! re-verified: verify the `serde_json::Value` you received —
//! [`ProofVerifier::verify_received`](crate::ProofVerifier::verify_received),
//! or [`ProofPolicy::VerifyReceived`](crate::ProofPolicy::VerifyReceived) in
//! [`consume_inbound`](crate::consume_inbound) — and dispatch after. Nothing in
//! this module prunes a value it was handed by reference.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::dispatcher::downcast_payload;
use crate::document::TrustTask;
use crate::error::RejectReason;
use crate::payload::Payload;
use crate::type_uri::TypeUri;

/// The longest chain walked. The generator only emits links to a strictly
/// earlier MINOR, so a real chain is far shorter; the bound is what stops a
/// malformed table from looping.
const MAX_CHAIN: usize = 256;

/// How deep [`prune_payload`] follows a schema into a value before leaving the
/// rest as it is.
const MAX_DEPTH: usize = 128;

/// The `MAJOR.MINOR` the specification behind `type_uri` declares it extends
/// (front matter `extends`, SPEC §5.2 "Declared extension"), or `None`.
///
/// Pass the bare request Type URI on the canonical `https://trusttasks.org`
/// authority, as the other per-Type-URI lookups take it. Only the direct
/// declaration: `x/0.3` extending `0.2` answers `Some("0.2")` even when `0.2`
/// itself extends `0.1` — walk the chain by asking again, or use
/// [`extension_reaches`].
///
/// `None` is both "declares no extension" and "a Type URI this build does not
/// know"; for routing the two mean the same thing — no forward acceptance.
/// The table is generated, not feature-gated: a build without the received
/// version's family still knows its chain.
///
/// A version newer than this build's table — one an application generated
/// with `trust-tasks-build` — is known once the application has passed its
/// declaration to [`declare_extends`]. The generated table answers first.
pub fn extends_for(type_uri: &str) -> Option<&'static str> {
    #[cfg(test)]
    if let Some(answer) = test_index::lookup(type_uri) {
        return answer;
    }
    crate::extends_index::declared_extends(type_uri).or_else(|| application_extends(type_uri))
}

/// The declarations applications have passed to [`declare_extends`].
static APPLICATION_EXTENDS: std::sync::RwLock<Vec<&'static [(&'static str, &'static str)]>> =
    std::sync::RwLock::new(Vec::new());

/// Teach [`extends_for`] — and through it the dispatchers' forward-minor
/// routing, [`read_as`] and [`consume_inbound`](crate::consume_inbound) — the
/// `extends` declarations of specification versions this build's generated
/// table does not carry: those an application generated itself from registry
/// bundles with `trust-tasks-build`.
///
/// Each entry is `(bare request Type URI, "MAJOR.MINOR" it extends)`. The file
/// `trust-tasks-build` writes carries every locked version's declarations as
/// `EXTENDS` and a `declare_extends()` that calls this with them, so an
/// application calls that once at start-up:
///
/// ```ignore
/// mod tasks { include!(concat!(env!("OUT_DIR"), "/trust_tasks_generated.rs")); }
/// tasks::declare_extends();
/// ```
///
/// Process-wide and additive. Declaring the same table twice is harmless, and
/// where this build's generated table already answers for a Type URI that
/// answer stands. A declaration is still only followed when it names a
/// strictly earlier MINOR of the same MAJOR (SPEC §5.2).
pub fn declare_extends(declarations: &'static [(&'static str, &'static str)]) {
    let mut tables = APPLICATION_EXTENDS
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if !tables.iter().any(|t| std::ptr::eq(*t, declarations)) {
        tables.push(declarations);
    }
}

fn application_extends(type_uri: &str) -> Option<&'static str> {
    let tables = APPLICATION_EXTENDS
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    tables
        .iter()
        .flat_map(|t| t.iter())
        .find(|(uri, _)| *uri == type_uri)
        .map(|(_, target)| *target)
}

/// The versions `received`'s declared-extension chain reaches, nearest (and so
/// highest) first. Each keeps `received`'s authority, slug and variant.
pub(crate) fn chain(received: &TypeUri) -> Vec<TypeUri> {
    let mut out = Vec::new();
    let mut current = received.clone();
    while out.len() < MAX_CHAIN {
        let key = current.bare().to_string();
        let Some(target) = extends_for(&key) else {
            break;
        };
        let Some((major, minor)) = parse_major_minor(target) else {
            break;
        };
        // Only a strictly earlier MINOR of the same MAJOR is a link SPEC §5.2
        // permits; anything else ends the walk rather than leading it astray.
        if major != current.major() || minor >= current.minor() {
            break;
        }
        current = current.with_version(major, minor);
        out.push(current.clone());
    }
    out
}

fn parse_major_minor(v: &str) -> Option<(u32, u32)> {
    let (major, minor) = v.split_once('.')?;
    Some((major.parse().ok()?, minor.parse().ok()?))
}

/// `true` when `received`'s declared-extension chain reaches `implemented` —
/// the case in which SPEC §5.2 asks a consumer of `implemented` to accept a
/// `received` document.
///
/// Same slug, same variant (`#request` and no fragment are one variant), and a
/// chain of `extends` declarations from `received` down to `implemented`'s
/// `MAJOR.MINOR`. `false` for the identical version: that is not an extension,
/// it is the version itself.
pub fn extension_reaches(received: &TypeUri, implemented: &TypeUri) -> bool {
    let target = implemented.for_routing();
    received.slug() == implemented.slug() && chain(&received.for_routing()).contains(&target)
}

/// Read `doc` as a document of the version `P` implements.
///
/// * `doc`'s `type` **is** `P`'s (bare or `#request`-fragmented alike): the
///   payload is deserialized as it stands — what the dispatchers have always
///   done.
/// * `doc`'s version declares an extension chain reaching `P`'s (SPEC §5.2):
///   the payload is pruned to the members `P`'s schema knows
///   ([`prune_payload`]) and then deserialized. The document keeps the `type`
///   it arrived with, so [`TrustTask::respond_with`] answers the received
///   version.
/// * Otherwise [`RejectReason::UnsupportedVersion`] for the same slug, and
///   [`RejectReason::UnsupportedType`] for another.
///
/// A payload that does not deserialize into `P` — after pruning, in the
/// forward case — is [`RejectReason::MalformedRequest`].
///
/// Takes `doc` by value and returns the typed document only: verify its proof
/// over the value you parsed it from, not over the result (see the module
/// docs).
pub fn read_as<P: Payload>(doc: TrustTask<Value>) -> Result<TrustTask<P>, RejectReason> {
    let implemented = P::type_uri();
    let received = doc.type_uri.for_routing();
    if received == implemented.for_routing() {
        return downcast_payload::<P>(doc);
    }
    if extension_reaches(&doc.type_uri, &implemented) {
        return forward::<P>(doc, P::PAYLOAD_SCHEMA);
    }
    let type_uri = received.to_string();
    if doc.type_uri.slug() == implemented.slug() {
        Err(RejectReason::UnsupportedVersion { type_uri })
    } else {
        Err(RejectReason::UnsupportedType { type_uri })
    }
}

/// Prune `doc`'s payload against `schema` and downcast — the forward half of
/// [`read_as`], shared with the dispatchers.
pub(crate) fn forward<P: Payload>(
    mut doc: TrustTask<Value>,
    schema: Option<&str>,
) -> Result<TrustTask<P>, RejectReason> {
    if let Some(schema) = schema {
        prune_payload(&mut doc.payload, schema);
    }
    downcast_payload::<P>(doc)
}

/// Remove from `payload` the object members `schema_json` does not know, so a
/// payload of an extending version deserializes into the extended version's
/// type (whose structs refuse unknown members wherever its schema says
/// `additionalProperties: false`).
///
/// `schema_json` is the implemented version's
/// [`Payload::PAYLOAD_SCHEMA`] — self-contained, cross-file `$ref`s already
/// inlined into `$defs`. The walk follows the schema into the value:
///
/// * an object member is dropped where the schema at that point says
///   `additionalProperties: false` and lists no such member under
///   `properties`; or `unevaluatedProperties: false` and no `properties` it
///   can evaluate — its own, a `$ref` target's, every `allOf` branch's, and
///   the `oneOf`/`anyOf` branch chosen below — lists it;
/// * a kept member is pruned against its `properties` subschema, or against
///   `additionalProperties` when that is a schema;
/// * an array's elements against `prefixItems` / `items`;
/// * local `$ref`s (`#/…` JSON pointers) are followed, and every `allOf`
///   branch is applied;
/// * for `oneOf` / `anyOf`, the payload is pruned against a branch only when
///   **exactly one** branch admits it on a shallow reading — declared `type`,
///   `const`, `enum`, `required`, and the `const`/`enum` of its properties
///   (the discriminator shape these schemas use). With no such branch, or more
///   than one, that part of the value is left as it is.
///
/// Conservative where it cannot be sure, and so never drops a member the
/// implemented version might have known: nothing is dropped from an object
/// whose schema has `patternProperties` (no regex engine here), or an
/// `if`/`then`/`else` or `dependentSchemas` that could evaluate members
/// (under `unevaluatedProperties`). What is left in place is then the
/// deserializer's to accept or refuse, as it would be with no pruning at all.
/// An unparsable schema prunes nothing.
pub fn prune_payload(payload: &mut Value, schema_json: &str) {
    let Ok(root) = serde_json::from_str::<Value>(schema_json) else {
        return;
    };
    prune(payload, &root, &root, 0);
}

/// Resolve a local `$ref` against `root`.
fn resolve<'a>(reference: &str, root: &'a Value) -> Option<&'a Value> {
    let pointer = reference.strip_prefix('#')?;
    if pointer.is_empty() {
        return Some(root);
    }
    root.pointer(pointer)
}

/// `schema` with any local `$ref` chain followed, for the shallow reads. A
/// `$ref` with siblings is read as the target; the siblings are ignored here
/// (they only matter to [`prune`], which applies both).
fn deref<'a>(mut schema: &'a Value, root: &'a Value) -> &'a Value {
    for _ in 0..MAX_DEPTH {
        match schema.get("$ref").and_then(Value::as_str) {
            Some(r) => match resolve(r, root) {
                Some(target) => schema = target,
                None => break,
            },
            None => break,
        }
    }
    schema
}

fn prune(value: &mut Value, schema: &Value, root: &Value, depth: usize) {
    if depth > MAX_DEPTH {
        return;
    }
    let Some(s) = schema.as_object() else {
        // `true` / `false` schemas say nothing about members.
        return;
    };

    if let Some(target) = s
        .get("$ref")
        .and_then(Value::as_str)
        .and_then(|r| resolve(r, root))
    {
        prune(value, target, root, depth + 1);
    }
    if let Some(branches) = s.get("allOf").and_then(Value::as_array) {
        for branch in branches {
            prune(value, branch, root, depth + 1);
        }
    }
    let chosen = ["oneOf", "anyOf"]
        .iter()
        .filter_map(|k| s.get(*k).and_then(Value::as_array))
        .map(|branches| sole_admitting_branch(value, branches, root))
        .collect::<Vec<_>>();
    for branch in chosen.iter().flatten() {
        prune(value, branch, root, depth + 1);
    }

    match value {
        Value::Object(members) => {
            let properties = s.get("properties").and_then(Value::as_object);
            let opaque = s.contains_key("patternProperties");

            if !opaque && s.get("additionalProperties") == Some(&Value::Bool(false)) {
                members.retain(|k, _| properties.is_some_and(|p| p.contains_key(k)));
            } else if !opaque
                && s.get("unevaluatedProperties") == Some(&Value::Bool(false))
                && !["if", "then", "else", "dependentSchemas"]
                    .iter()
                    .any(|k| s.contains_key(*k))
                // An undecided oneOf/anyOf could evaluate any member.
                && chosen.iter().all(Option::is_some)
            {
                let mut known = BTreeSet::new();
                evaluated_properties(schema, root, &mut known, 0);
                for branch in chosen.iter().flatten() {
                    evaluated_properties(branch, root, &mut known, 0);
                }
                members.retain(|k, _| known.contains(k.as_str()));
            }

            let additional = s.get("additionalProperties").filter(|a| a.is_object());
            for (name, member) in members.iter_mut() {
                match properties.and_then(|p| p.get(name)) {
                    Some(sub) => prune(member, sub, root, depth + 1),
                    None => {
                        if let Some(additional) = additional {
                            prune(member, additional, root, depth + 1);
                        }
                    }
                }
            }
        }
        Value::Array(elements) => {
            let prefix = s
                .get("prefixItems")
                .and_then(Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or_default();
            let items = s.get("items").filter(|i| i.is_object());
            for (i, element) in elements.iter_mut().enumerate() {
                if let Some(sub) = prefix.get(i).or(items) {
                    prune(element, sub, root, depth + 1);
                }
            }
        }
        _ => {}
    }
}

/// Every `properties` name `schema` evaluates at this instance location: its
/// own, its `$ref` target's and its `allOf` branches'. (The chosen
/// `oneOf`/`anyOf` branch is added by the caller, which knows which it is.)
fn evaluated_properties<'a>(
    schema: &'a Value,
    root: &'a Value,
    out: &mut BTreeSet<&'a str>,
    depth: usize,
) {
    if depth > MAX_DEPTH {
        return;
    }
    let Some(s) = schema.as_object() else {
        return;
    };
    if let Some(p) = s.get("properties").and_then(Value::as_object) {
        out.extend(p.keys().map(String::as_str));
    }
    if let Some(target) = s
        .get("$ref")
        .and_then(Value::as_str)
        .and_then(|r| resolve(r, root))
    {
        evaluated_properties(target, root, out, depth + 1);
    }
    if let Some(branches) = s.get("allOf").and_then(Value::as_array) {
        for branch in branches {
            evaluated_properties(branch, root, out, depth + 1);
        }
    }
}

/// The one branch of a `oneOf`/`anyOf` that admits `value` on a shallow
/// reading, or `None` when none or several do.
fn sole_admitting_branch<'a>(
    value: &Value,
    branches: &'a [Value],
    root: &'a Value,
) -> Option<&'a Value> {
    let mut admitting = branches.iter().filter(|b| admits(value, b, root, 0));
    let first = admitting.next()?;
    match admitting.next() {
        Some(_) => None,
        None => Some(first),
    }
}

/// A shallow "could `value` be an instance of `schema`": the keywords a
/// discriminated union is told apart by, and nothing that pruning could change.
/// Extra members never count against a branch — they are what pruning removes.
fn admits(value: &Value, schema: &Value, root: &Value, depth: usize) -> bool {
    if depth > 8 {
        return true;
    }
    let schema = deref(schema, root);
    let Some(s) = schema.as_object() else {
        return schema != &Value::Bool(false);
    };
    if let Some(t) = s.get("type") {
        let names: Vec<&str> = match t {
            Value::String(one) => vec![one.as_str()],
            Value::Array(many) => many.iter().filter_map(Value::as_str).collect(),
            _ => vec![],
        };
        if !names.is_empty() && !names.iter().any(|n| has_type(value, n)) {
            return false;
        }
    }
    if let Some(c) = s.get("const") {
        if c != value {
            return false;
        }
    }
    if let Some(e) = s.get("enum").and_then(Value::as_array) {
        if !e.contains(value) {
            return false;
        }
    }
    if let Value::Object(members) = value {
        if let Some(required) = s.get("required").and_then(Value::as_array) {
            if !required
                .iter()
                .filter_map(Value::as_str)
                .all(|r| members.contains_key(r))
            {
                return false;
            }
        }
        if let Some(properties) = s.get("properties").and_then(Value::as_object) {
            for (name, sub) in properties {
                let Some(member) = members.get(name) else {
                    continue;
                };
                let sub = deref(sub, root);
                let discriminates = sub.get("const").is_some() || sub.get("enum").is_some();
                if discriminates && !admits(member, sub, root, depth + 1) {
                    return false;
                }
            }
        }
    }
    if let Some(branches) = s.get("allOf").and_then(Value::as_array) {
        if !branches.iter().all(|b| admits(value, b, root, depth + 1)) {
            return false;
        }
    }
    true
}

fn has_type(value: &Value, name: &str) -> bool {
    match name {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        "number" => value.is_number(),
        "integer" => {
            value.is_i64() || value.is_u64() || value.as_f64().is_some_and(|f| f.fract() == 0.0)
        }
        _ => true,
    }
}

/// A per-thread overlay on the generated table, so the routing can be tested
/// with a constructed chain: no specification declares `extends` yet, and a
/// test must not depend on one that does.
#[cfg(test)]
pub(crate) mod test_index {
    use std::cell::RefCell;
    use std::collections::HashMap;

    thread_local! {
        static OVERLAY: RefCell<HashMap<String, Option<&'static str>>> =
            RefCell::new(HashMap::new());
    }

    /// Answer `extends_for(type_uri)` with `target` on this thread.
    pub(crate) fn declare(type_uri: &str, target: Option<&'static str>) {
        OVERLAY.with(|o| o.borrow_mut().insert(type_uri.to_string(), target));
    }

    pub(super) fn lookup(type_uri: &str) -> Option<Option<&'static str>> {
        OVERLAY.with(|o| o.borrow().get(type_uri).copied())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn additional_properties_false_drops_unknown_members_through_refs() {
        let schema = json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "entry": { "$ref": "#/$defs/Entry" },
                "list": { "type": "array", "items": { "$ref": "#/$defs/Entry" } },
                "open": { "type": "object" }
            },
            "$defs": {
                "Entry": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": { "subject": { "type": "string" } }
                }
            }
        })
        .to_string();
        let mut payload = json!({
            "entry": { "subject": "a", "added": 1 },
            "list": [{ "subject": "b", "added": 2 }],
            "open": { "kept": true },
            "newTop": "x"
        });
        prune_payload(&mut payload, &schema);
        assert_eq!(
            payload,
            json!({
                "entry": { "subject": "a" },
                "list": [{ "subject": "b" }],
                "open": { "kept": true }
            })
        );
    }

    #[test]
    fn a_discriminated_one_of_prunes_against_its_branch_and_an_ambiguous_one_is_left() {
        let schema = json!({
            "oneOf": [
                { "type": "object", "additionalProperties": false, "required": ["kind"],
                  "properties": { "kind": { "const": "a" }, "x": {} } },
                { "type": "object", "additionalProperties": false, "required": ["kind"],
                  "properties": { "kind": { "const": "b" }, "y": {} } }
            ]
        })
        .to_string();
        let mut payload = json!({ "kind": "b", "y": 1, "z": 2 });
        prune_payload(&mut payload, &schema);
        assert_eq!(payload, json!({ "kind": "b", "y": 1 }));

        // Neither branch: left as it is, for the deserializer to refuse.
        let mut unknown = json!({ "kind": "c", "z": 2 });
        prune_payload(&mut unknown, &schema);
        assert_eq!(unknown, json!({ "kind": "c", "z": 2 }));
    }

    #[test]
    fn unevaluated_properties_false_keeps_what_all_of_evaluates() {
        let schema = json!({
            "allOf": [{ "$ref": "#/$defs/Base" }],
            "properties": { "own": {} },
            "unevaluatedProperties": false,
            "$defs": { "Base": { "properties": { "base": {} } } }
        })
        .to_string();
        let mut payload = json!({ "base": 1, "own": 2, "added": 3 });
        prune_payload(&mut payload, &schema);
        assert_eq!(payload, json!({ "base": 1, "own": 2 }));
    }

    #[test]
    fn pattern_properties_make_an_object_opaque() {
        let schema = json!({
            "additionalProperties": false,
            "patternProperties": { "^x-": {} },
            "properties": { "a": {} }
        })
        .to_string();
        let mut payload = json!({ "a": 1, "x-b": 2, "c": 3 });
        prune_payload(&mut payload, &schema);
        assert_eq!(payload, json!({ "a": 1, "x-b": 2, "c": 3 }));
    }

    #[test]
    fn chain_walks_to_each_earlier_version_and_stops_at_a_bad_link() {
        let base = "https://trusttasks.org/spec/test-ext/chain";
        test_index::declare(&format!("{base}/0.3"), Some("0.2"));
        test_index::declare(&format!("{base}/0.2"), Some("0.1"));
        // A link that does not go strictly down ends the walk.
        test_index::declare(&format!("{base}/0.1"), Some("0.5"));

        let received: TypeUri = format!("{base}/0.3#request").parse().unwrap();
        let links: Vec<String> = chain(&received.for_routing())
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(links, vec![format!("{base}/0.2"), format!("{base}/0.1")]);

        let v01: TypeUri = format!("{base}/0.1").parse().unwrap();
        let v03: TypeUri = format!("{base}/0.3").parse().unwrap();
        assert!(extension_reaches(&received, &v01));
        assert!(!extension_reaches(&v01, &v03), "never backwards");
        assert!(
            !extension_reaches(&v03, &v03),
            "the version itself is not an extension"
        );
        // Response variants walk the same chain and stay responses.
        assert!(extension_reaches(
            &received.with_response(),
            &v01.with_response()
        ));
        assert!(!extension_reaches(&received.with_response(), &v01));
    }

    /// An application's declarations (what `trust-tasks-build`'s
    /// `declare_extends()` passes) extend the generated table: the chain they
    /// describe is walked, and declaring twice changes nothing.
    #[test]
    fn application_declarations_are_walked() {
        static DECLARED: &[(&str, &str)] = &[
            ("https://trusttasks.org/spec/test-app-ext/thing/0.3", "0.2"),
            ("https://trusttasks.org/spec/test-app-ext/thing/0.2", "0.1"),
        ];
        let base = "https://trusttasks.org/spec/test-app-ext/thing";
        assert_eq!(extends_for(&format!("{base}/0.3")), None);
        declare_extends(DECLARED);
        declare_extends(DECLARED);
        assert_eq!(extends_for(&format!("{base}/0.3")), Some("0.2"));
        assert_eq!(
            APPLICATION_EXTENDS
                .read()
                .unwrap()
                .iter()
                .filter(|t| std::ptr::eq(**t, DECLARED))
                .count(),
            1
        );

        let received: TypeUri = format!("{base}/0.3").parse().unwrap();
        let v01: TypeUri = format!("{base}/0.1").parse().unwrap();
        assert!(extension_reaches(&received, &v01));
    }
}
