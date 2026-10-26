//! Schema normalisation passes run before typify sees a schema.

use anyhow::{anyhow, Result};
use serde_json::{json, Value};

/// Rename the 2020-12 `$defs` keyword to the Draft-07 `definitions` keyword
/// recursively, and rewrite every `$ref` of the form `#/$defs/X` to
/// `#/definitions/X`. typify 0.5 only honors the older keyword; the two are
/// semantically equivalent for our purposes.
pub(crate) fn migrate_defs_to_definitions(schema: &mut Value) {
    match schema {
        Value::Object(map) => {
            if let Some(defs) = map.remove("$defs") {
                map.insert("definitions".to_string(), defs);
            }
            if let Some(Value::String(s)) = map.get_mut("$ref") {
                if let Some(stripped) = s.strip_prefix("#/$defs/") {
                    *s = format!("#/definitions/{stripped}");
                }
            }
            for v in map.values_mut() {
                migrate_defs_to_definitions(v);
            }
        }
        Value::Array(items) => {
            for v in items {
                migrate_defs_to_definitions(v);
            }
        }
        _ => {}
    }
}

/// Rewrite schema titles so typify produces clean type names:
/// `Payload` for the root, `Response` for the response sub-schema. Returns
/// whether a response anchor was present.
pub(crate) fn normalize_titles(schema: &mut Value) -> Result<bool> {
    let obj = schema
        .as_object_mut()
        .ok_or_else(|| anyhow!("schema root must be an object"))?;
    obj.insert("title".into(), json!("Payload"));

    let mut has_response = false;
    if let Some(defs) = obj.get_mut("$defs").and_then(Value::as_object_mut) {
        if let Some(resp) = defs.get_mut("Response").and_then(Value::as_object_mut) {
            resp.insert("title".into(), json!("Response"));
            has_response = true;
        }
    }
    Ok(has_response)
}

/// Rewrite an `allOf` composition closed with `unevaluatedProperties: false`
/// into the flat, `additionalProperties: false` object it denotes.
///
/// ## Why this exists
///
/// A specification that wants to reuse a shared definition *and* add members
/// of its own has exactly one way to close the result: `unevaluatedProperties`
/// at the outer level. `additionalProperties` cannot do it — both keywords are
/// evaluated against the whole instance from within the subschema that
/// declares them, so an `additionalProperties: false` inside either the shared
/// definition or the composing schema rejects the other's members. That is why
/// `credentials/_shared/0.2`'s `IssuedCredentialBase` is deliberately left
/// open and `vta/credentials/issue/0.2` closes over it.
///
/// typify does not model `unevaluatedProperties`. It flattens the `allOf` into
/// a single struct with every member — the right shape — but emits no
/// `deny_unknown_fields`, because it only maps `additionalProperties: false`.
/// The generated type is then *more permissive than the schema it came from*:
/// a validator rejects an unknown member, the Rust type silently accepts it.
///
/// This pass performs the flattening itself, before typify sees the schema, so
/// the strictness survives the round-trip. The struct typify produces is
/// unchanged; only the attribute is added.
///
/// ## When it declines
///
/// Only a composition whose every `allOf` member is a plain object schema —
/// properties, required, and annotations, with no combinator of its own and no
/// closure — is flattened. Anything else (a nested `allOf`, a `oneOf`, an
/// `if`/`then`, a member that closes itself) is left exactly as it was: the
/// merge would not be sound, and a wrong `deny_unknown_fields` rejects valid
/// documents, which is worse than the permissiveness it would fix.
pub(crate) fn flatten_unevaluated_compositions(schema: &mut Value) {
    let defs = schema.get("definitions").cloned().unwrap_or(Value::Null);
    flatten_node(schema, &defs);
}

/// Walk every object node, flattening where [`flatten_here`] applies.
fn flatten_node(node: &mut Value, defs: &Value) {
    match node {
        Value::Object(map) => {
            flatten_here(map, defs);
            for (_, v) in map.iter_mut() {
                flatten_node(v, defs);
            }
        }
        Value::Array(items) => {
            for v in items.iter_mut() {
                flatten_node(v, defs);
            }
        }
        _ => {}
    }
}

/// Resolve a local `#/definitions/X` reference against the document's
/// definitions. Returns `None` for anything else — an unresolved cross-file
/// `$ref` means the composition cannot be reasoned about here, which is a
/// decline, not an error.
fn resolve_local_ref<'a>(value: &'a Value, defs: &'a Value) -> Option<&'a Value> {
    let r = value.get("$ref")?.as_str()?;
    let name = r.strip_prefix("#/definitions/")?;
    defs.get(name)
}

/// Members a plain object schema may carry and still be safe to merge.
/// Anything outside this set means the subschema constrains the instance in a
/// way a flat merge would lose.
const MERGEABLE_KEYS: &[&str] = &[
    "$anchor",
    "$comment",
    "$id",
    "$schema",
    "default",
    "description",
    "examples",
    "properties",
    "required",
    "title",
    "type",
];

/// Flatten one node in place, if it is a closed `allOf` composition of plain
/// object schemas. No-op otherwise.
fn flatten_here(map: &mut serde_json::Map<String, Value>, defs: &Value) {
    if map.get("unevaluatedProperties") != Some(&Value::Bool(false)) {
        return;
    }
    let Some(Value::Array(all_of)) = map.get("allOf") else {
        return;
    };

    // Resolve every member first, and only commit once all of them qualify —
    // a partial merge would produce a struct that is neither the composition
    // nor the original.
    let mut resolved = Vec::with_capacity(all_of.len());
    for member in all_of {
        let target = match resolve_local_ref(member, defs) {
            Some(t) => t,
            None if member.get("$ref").is_none() => member,
            None => return,
        };
        let Some(obj) = target.as_object() else {
            return;
        };
        if obj.keys().any(|k| !MERGEABLE_KEYS.contains(&k.as_str())) {
            return;
        }
        resolved.push(obj.clone());
    }

    // The outer schema's own members win a collision: it is the more specific
    // statement, and this is the direction JSON Schema's `allOf` already
    // resolves to (both constraints apply; the narrower one decides).
    let mut properties = serde_json::Map::new();
    let mut required: Vec<Value> = Vec::new();
    for obj in &resolved {
        if let Some(Value::Object(props)) = obj.get("properties") {
            for (k, v) in props {
                properties.insert(k.clone(), v.clone());
            }
        }
        if let Some(Value::Array(req)) = obj.get("required") {
            for r in req {
                if !required.contains(r) {
                    required.push(r.clone());
                }
            }
        }
    }
    if let Some(Value::Object(own)) = map.get("properties") {
        for (k, v) in own {
            properties.insert(k.clone(), v.clone());
        }
    }
    if let Some(Value::Array(own_req)) = map.get("required") {
        for r in own_req {
            if !required.contains(r) {
                required.push(r.clone());
            }
        }
    }

    map.remove("allOf");
    map.remove("unevaluatedProperties");
    map.insert("type".into(), Value::String("object".into()));
    map.insert("properties".into(), Value::Object(properties));
    if !required.is_empty() {
        map.insert("required".into(), Value::Array(required));
    }
    map.insert("additionalProperties".into(), Value::Bool(false));
}

/// Drop a `oneOf` combinator from the schema typify sees when any of its
/// branches carries a compound `not` (wrapped in `anyOf`/`allOf`, or a
/// multi-field `required`) instead of a single `{"required": ["x"]}` guard.
///
/// ## Why this exists
///
/// A spec expressing "exactly one of A, B, C" (with A, B, C themselves
/// possibly excluding more than one sibling — SPEC.md's mutual-exclusion
/// idiom, e.g. `auth/passkey/enroll/invite/update`'s `role` /
/// `expiresAt` / `extendBy`) writes each branch as `{"required": ["a"],
/// "not": {"anyOf": [{"required": ["b"]}, {"required": ["c"]}]}}`. typify
/// (0.6) merges a `oneOf` branch's keywords into the untagged enum variant
/// it synthesizes for that branch, and it handles a bare `"not": {"required":
/// ["x"]}` directly — property `x` is simply absent from that variant's
/// fields. Wrap the same exclusion in `anyOf` and typify's merge instead
/// produces an uninhabited type (`pub enum PayloadVariant0 {}`) for that
/// branch, so a document that should satisfy it can never deserialize — and,
/// worse, a property required only by *that* branch (`role`, excluded by
/// nobody else) gets silently dropped from every *other* branch's variant
/// too, so a document combining it with another branch's discriminator also
/// fails to round-trip.
///
/// Re-expressing the same constraint as `anyOf` (at least one) plus a
/// pairwise `not` (no two at once) — `oneOf`'s own JSON Schema definition —
/// was tried and does not help: typify applies the same broken merge to a
/// `not` wrapping `anyOf`/`allOf` regardless of which top-level combinator
/// carries it, so the replacement produces the identical dropped-property
/// defect one level up.
///
/// The combinator is instead removed outright, with **no** replacement, from
/// only the copy of the schema handed to typify. `generate_one` captures the
/// wire schema (`SCHEMA_JSON`, what `ValidatedPayload::validate_value` and
/// every other language's binding validate against — SPEC.md §7.2 item 2)
/// *before* this pass runs, so the actual mutual-exclusion constraint is
/// still enforced at runtime; only the Rust type becomes as permissive as
/// the TypeScript/Go/Dart bindings already are for this same schema (every
/// affected member optional, no combinator modelled in the type system).
/// That is a real loss of a compile-time guarantee, but every alternative
/// tried produces a Rust type that is not merely less precise, but
/// *outright wrong* — unable to represent, or silently unable to
/// deserialize, a document the schema declares valid. A plain discriminated
/// `oneOf` (every branch's `not`, if any, a single `{"required": ["x"]}`)
/// typify already renders as a clean Rust enum, and is untouched here.
pub(crate) fn desugar_oneof_with_compound_not(schema: &mut Value) {
    desugar_oneof_node(schema);
}

fn desugar_oneof_node(node: &mut Value) {
    match node {
        Value::Object(map) => {
            let drop = matches!(map.get("oneOf"), Some(Value::Array(branches)) if branches.len() >= 2 && branches.iter().any(has_compound_not));
            if drop {
                map.remove("oneOf");
            }
            for (_, v) in map.iter_mut() {
                desugar_oneof_node(v);
            }
        }
        Value::Array(items) => {
            for v in items.iter_mut() {
                desugar_oneof_node(v);
            }
        }
        _ => {}
    }
}

/// A branch's `not` is "simple" — and left to typify's own direct handling —
/// only when it is exactly `{"required": ["x"]}` for a single field `x`.
/// Anything else (wrapped in `anyOf`/`allOf`, more than one required field,
/// or any other shape) is "compound" and triggers dropping the whole
/// `oneOf` in [`desugar_oneof_with_compound_not`].
fn has_compound_not(branch: &Value) -> bool {
    let Some(not) = branch.get("not") else {
        return false;
    };
    let Some(obj) = not.as_object() else {
        // `not: true` / `not: false` — not a shape this pass understands;
        // treat conservatively as compound so the `oneOf` is dropped rather
        // than handed to typify's oneOf-merge heuristics.
        return true;
    };
    if obj.len() != 1 {
        return true;
    }
    !matches!(obj.get("required"), Some(Value::Array(arr)) if arr.len() == 1)
}

/// Drop an `anyOf` from the schema typify sees when it sits on an object
/// schema and every one of its branches is exactly `{"required": [...]}`,
/// naming members that object declares in `properties`.
///
/// ## Why this exists
///
/// That shape is JSON Schema's idiom for "at least one of these members" —
/// `rooms/keys/chain/0.2`'s `links` / `epochFloor`, `rooms/records/curate/0.1`'s
/// `status` / `pinned`. A document carrying *several* of the members is
/// valid; for `rooms/keys/chain/0.2` it is the normal case (a member relays
/// the rungs together with the floor they came with). typify (0.6) renders
/// the `anyOf` as an untagged enum with one struct variant per branch, and
/// each variant carries only the members its own branch requires, under the
/// object's `deny_unknown_fields`. A document holding two of the members
/// then matches no variant — every variant rejects the other member as
/// unknown — so the Rust type cannot deserialize a payload the schema
/// declares valid. The defect is silent at generation time and shows only
/// when a real peer sends both members.
///
/// typify offers no setting that models `anyOf` as "struct plus a
/// constraint", and an `anyOf` cannot be rewritten into a `oneOf` it handles
/// without enumerating every non-empty subset of the members (2ⁿ − 1
/// branches, each still an enum variant), which turns one optional member
/// into a match over combinations and does not scale to a four- or
/// nine-member clause.
///
/// So, as in [`desugar_oneof_with_compound_not`], the combinator is removed
/// with **no** replacement, from only the copy of the schema handed to
/// typify. `generate_one` captures the wire schema (`SCHEMA_JSON`, which
/// `ValidatedPayload::validate_value` and every other language's binding
/// validate against — SPEC.md §7.2 item 2) *before* this pass runs, so "at
/// least one of" is still enforced at runtime. What the Rust type loses is
/// only the compile-time statement of that constraint: the members become
/// ordinary optional fields of one struct, so a value built in Rust with
/// none of them compiles and is refused by validation instead — the same
/// shape the TypeScript, Go and Dart bindings already give these schemas.
///
/// ## When it declines
///
/// Only that exact shape is touched. A branch carrying any other keyword
/// (`properties`, `not`, `type`, …), a `required` naming a member the object
/// does not declare, an `anyOf` on a schema with no `properties` (a
/// `dependentSchemas` entry, a bare `oneOf` branch), and anything beneath a
/// `not` — where dropping a clause would invert what it says — are all left
/// exactly as they were.
pub(crate) fn drop_anyof_of_required_members(schema: &mut Value) {
    drop_anyof_node(schema);
}

fn drop_anyof_node(node: &mut Value) {
    match node {
        Value::Object(map) => {
            if is_anyof_of_required_members(map) {
                map.remove("anyOf");
            }
            for (k, v) in map.iter_mut() {
                if k != "not" {
                    drop_anyof_node(v);
                }
            }
        }
        Value::Array(items) => {
            for v in items.iter_mut() {
                drop_anyof_node(v);
            }
        }
        _ => {}
    }
}

/// True when `map` is an object schema whose `anyOf` says only "at least one
/// of these declared members": every branch is exactly `{"required": [..]}`
/// with a non-empty list of member names, each declared in `properties`.
fn is_anyof_of_required_members(map: &serde_json::Map<String, Value>) -> bool {
    let Some(Value::Object(properties)) = map.get("properties") else {
        return false;
    };
    let Some(Value::Array(branches)) = map.get("anyOf") else {
        return false;
    };
    !branches.is_empty()
        && branches.iter().all(|branch| {
            let Some(obj) = branch.as_object() else {
                return false;
            };
            if obj.len() != 1 {
                return false;
            }
            matches!(
                obj.get("required"),
                Some(Value::Array(names)) if !names.is_empty()
                    && names
                        .iter()
                        .all(|n| n.as_str().is_some_and(|n| properties.contains_key(n)))
            )
        })
}

#[cfg(test)]
mod flatten_tests {
    use super::*;
    use serde_json::json;

    fn flattened(mut schema: Value) -> Value {
        flatten_unevaluated_compositions(&mut schema);
        schema
    }

    /// The shape `vta/credentials/issue/0.2` uses: a `$ref` to a shared open
    /// definition, the composing schema's own members alongside it, closed
    /// with `unevaluatedProperties`.
    #[test]
    fn merges_a_referenced_base_and_closes_the_result() {
        let out = flattened(json!({
            "definitions": {
                "Base": {
                    "type": "object",
                    "properties": { "a": { "type": "string" }, "b": { "type": "string" } },
                    "required": ["a"]
                },
                "Composed": {
                    "allOf": [{ "$ref": "#/definitions/Base" }],
                    "properties": { "c": { "type": "string" } },
                    "unevaluatedProperties": false
                }
            }
        }));
        let c = &out["definitions"]["Composed"];
        assert_eq!(c["additionalProperties"], json!(false));
        assert!(c.get("allOf").is_none(), "the allOf is consumed");
        assert!(c.get("unevaluatedProperties").is_none());
        assert_eq!(c["properties"]["a"], json!({ "type": "string" }));
        assert_eq!(c["properties"]["c"], json!({ "type": "string" }));
        assert_eq!(c["required"], json!(["a"]));
    }

    /// The outer schema is the more specific statement, so its own definition
    /// of a member decides. Merging the other way would let a shared base
    /// silently widen a member the composing spec deliberately narrowed.
    #[test]
    fn the_composing_schema_wins_a_collision() {
        let out = flattened(json!({
            "definitions": {
                "Base": { "type": "object", "properties": { "a": { "type": "string" } } },
                "Composed": {
                    "allOf": [{ "$ref": "#/definitions/Base" }],
                    "properties": { "a": { "type": "integer" } },
                    "unevaluatedProperties": false
                }
            }
        }));
        assert_eq!(
            out["definitions"]["Composed"]["properties"]["a"],
            json!({ "type": "integer" })
        );
    }

    /// A member carrying a combinator of its own is not a plain object, so
    /// the merge would drop a constraint. Declining leaves the schema exactly
    /// as it was — permissive, but not wrong.
    #[test]
    fn declines_when_a_member_carries_a_combinator() {
        let input = json!({
            "definitions": {
                "Base": { "oneOf": [{ "type": "object" }, { "type": "null" }] },
                "Composed": {
                    "allOf": [{ "$ref": "#/definitions/Base" }],
                    "unevaluatedProperties": false
                }
            }
        });
        assert_eq!(flattened(input.clone()), input, "left untouched");
    }

    /// An unresolved `$ref` cannot be reasoned about, and guessing would be
    /// how a wrong `deny_unknown_fields` gets emitted.
    #[test]
    fn declines_on_an_unresolvable_ref() {
        let input = json!({
            "definitions": {
                "Composed": {
                    "allOf": [{ "$ref": "../elsewhere.json#/$defs/Base" }],
                    "unevaluatedProperties": false
                }
            }
        });
        assert_eq!(flattened(input.clone()), input);
    }

    /// `unevaluatedProperties: true` is not a closure, so there is nothing to
    /// carry over and nothing to do.
    #[test]
    fn ignores_an_open_composition() {
        let input = json!({
            "definitions": {
                "Base": { "type": "object", "properties": { "a": { "type": "string" } } },
                "Composed": {
                    "allOf": [{ "$ref": "#/definitions/Base" }],
                    "unevaluatedProperties": true
                }
            }
        });
        assert_eq!(flattened(input.clone()), input);
    }

    /// A schema already closed with `additionalProperties` never had the
    /// problem; typify handles it, and this pass must not disturb it.
    #[test]
    fn leaves_an_already_closed_object_alone() {
        let input = json!({
            "definitions": {
                "Plain": {
                    "type": "object",
                    "properties": { "a": { "type": "string" } },
                    "additionalProperties": false
                }
            }
        });
        assert_eq!(flattened(input.clone()), input);
    }
}

#[cfg(test)]
mod desugar_oneof_tests {
    use super::*;
    use serde_json::json;

    fn desugared(mut schema: Value) -> Value {
        desugar_oneof_with_compound_not(&mut schema);
        schema
    }

    /// The defect's own shape: a role-only branch whose `not` excludes two
    /// siblings via `anyOf`, alongside two branches with a single-field
    /// `not` each. The whole `oneOf` is dropped, not just the compound
    /// branch — `oneOf` is one combinator over the whole array.
    #[test]
    fn drops_a_oneof_with_any_compound_not_branch() {
        let input = json!({
            "type": "object",
            "required": ["inviteId"],
            "oneOf": [
                {
                    "required": ["role"],
                    "not": { "anyOf": [{ "required": ["expiresAt"] }, { "required": ["extendBy"] }] }
                },
                { "required": ["expiresAt"], "not": { "required": ["extendBy"] } },
                { "required": ["extendBy"], "not": { "required": ["expiresAt"] } }
            ]
        });
        let out = desugared(input);
        assert!(out.get("oneOf").is_none());
        assert_eq!(out["required"], json!(["inviteId"]));
    }

    /// A `not` with more than one `required` field (implicit AND, not
    /// wrapped in `anyOf`) is compound too — only a *single*-field
    /// `required` is the shape typify handles directly.
    #[test]
    fn a_multi_field_required_not_is_compound() {
        let input = json!({
            "oneOf": [
                { "required": ["a"], "not": { "required": ["b", "c"] } },
                { "required": ["b"] }
            ]
        });
        assert!(desugared(input).get("oneOf").is_none());
    }

    /// The two-way case every other affected schema in `specs/` uses
    /// (`auth/revoke-session/0.1`, `vta/app-state/put/1.0`, …): each
    /// branch's `not` is a single-field `required`, which typify already
    /// renders as a clean discriminated enum. Left untouched.
    #[test]
    fn leaves_a_plain_two_way_oneof_alone() {
        let input = json!({
            "oneOf": [
                { "required": ["sessionId"], "not": { "required": ["all"] } },
                { "required": ["all"], "not": { "required": ["sessionId"] } }
            ]
        });
        assert_eq!(desugared(input.clone()), input);
    }

    /// A branch with no `not` at all (e.g. a plain discriminated union) is
    /// simple by definition and never triggers the drop by itself.
    #[test]
    fn a_branch_with_no_not_is_simple() {
        let input = json!({
            "oneOf": [
                { "required": ["kind"], "properties": { "kind": { "const": "a" } } },
                { "required": ["kind"], "properties": { "kind": { "const": "b" } } }
            ]
        });
        assert_eq!(desugared(input.clone()), input);
    }

    /// The pass recurses into `$defs`/`definitions` and array members (e.g.
    /// a nested `allOf`), not just the schema's own top-level `oneOf`.
    #[test]
    fn recurses_into_nested_schemas() {
        let input = json!({
            "definitions": {
                "Nested": {
                    "oneOf": [
                        {
                            "required": ["a"],
                            "not": { "anyOf": [{ "required": ["b"] }] }
                        },
                        { "required": ["b"], "not": { "required": ["a"] } }
                    ]
                }
            }
        });
        let out = desugared(input);
        assert!(out["definitions"]["Nested"].get("oneOf").is_none());
    }
}

#[cfg(test)]
mod drop_anyof_tests {
    use super::*;
    use serde_json::json;

    fn dropped(mut schema: Value) -> Value {
        drop_anyof_of_required_members(&mut schema);
        schema
    }

    /// The defect's own shape (`rooms/keys/chain/0.2`): "at least one of
    /// `links` / `epochFloor`" on a closed object. The `anyOf` goes; the
    /// members, `required` and closure stay, so typify emits one struct with
    /// both members optional.
    #[test]
    fn drops_an_anyof_of_required_members() {
        let out = dropped(json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["roomId"],
            "properties": {
                "roomId": { "type": "string" },
                "links": { "type": "array" },
                "epochFloor": { "type": "integer" }
            },
            "anyOf": [{ "required": ["links"] }, { "required": ["epochFloor"] }]
        }));
        assert!(out.get("anyOf").is_none());
        assert_eq!(out["required"], json!(["roomId"]));
        assert_eq!(out["additionalProperties"], json!(false));
        assert!(out["properties"].get("links").is_some());
        assert!(out["properties"].get("epochFloor").is_some());
    }

    /// The pass reaches nested object schemas — a `definitions` entry such as
    /// a response — not only the payload root.
    #[test]
    fn recurses_into_definitions() {
        let out = dropped(json!({
            "definitions": {
                "Response": {
                    "type": "object",
                    "properties": { "a": {}, "b": {} },
                    "anyOf": [{ "required": ["a"] }, { "required": ["b"] }]
                }
            }
        }));
        assert!(out["definitions"]["Response"].get("anyOf").is_none());
    }

    /// A branch carrying anything beside `required` constrains more than
    /// presence, so the `anyOf` is not this idiom and is left for typify.
    #[test]
    fn leaves_an_anyof_with_a_richer_branch_alone() {
        let input = json!({
            "type": "object",
            "properties": { "a": {}, "b": {} },
            "anyOf": [
                { "required": ["a"], "properties": { "a": { "const": 1 } } },
                { "required": ["b"] }
            ]
        });
        assert_eq!(dropped(input.clone()), input);
    }

    /// A branch requiring a member the object does not declare is not "at
    /// least one of these members" — declining keeps the pass to the shape it
    /// was written for.
    #[test]
    fn leaves_an_anyof_naming_an_undeclared_member_alone() {
        let input = json!({
            "type": "object",
            "properties": { "a": {} },
            "anyOf": [{ "required": ["a"] }, { "required": ["elsewhere"] }]
        });
        assert_eq!(dropped(input.clone()), input);
    }

    /// No `properties` means no object schema to make the members optional
    /// on — `acl/list`'s `dependentSchemas` entry, or a bare `oneOf` branch
    /// (`persona/contact/lists/put`). Untouched.
    #[test]
    fn leaves_an_anyof_without_properties_alone() {
        let input = json!({
            "type": "object",
            "properties": { "direction": {}, "context": {}, "resource": {} },
            "dependentSchemas": {
                "direction": { "anyOf": [{ "required": ["context"] }, { "required": ["resource"] }] }
            },
            "oneOf": [
                { "required": ["direction"] },
                { "anyOf": [{ "required": ["context"] }, { "required": ["resource"] }] }
            ]
        });
        assert_eq!(dropped(input.clone()), input);
    }

    /// Beneath a `not`, the same clause means "none of these", and removing
    /// it would make the `not` reject everything. Never touched.
    #[test]
    fn never_reaches_beneath_a_not() {
        let input = json!({
            "type": "object",
            "properties": { "a": {}, "b": {}, "c": {} },
            "not": {
                "properties": { "b": {}, "c": {} },
                "anyOf": [{ "required": ["b"] }, { "required": ["c"] }]
            }
        });
        assert_eq!(dropped(input.clone()), input);
    }
}
