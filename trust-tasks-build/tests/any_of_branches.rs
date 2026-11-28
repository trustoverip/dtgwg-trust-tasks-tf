//! A member whose schema is an `anyOf` of object branches sharing members is
//! a struct of flattened `Option`s (typify's rendering), and `generate_module`
//! replaces its derived `Deserialize` with one that reads each branch from
//! its own copy of the value, first match wins, for both targets. The derived
//! one let a failed branch take the members it shares with the next, so the
//! next failed too (`hcp/entries/create` 0.1's `entry`).
//! `trust-tasks-rs/tests/anyof_object_branches.rs` holds the property at run
//! time on the prebuilt `hcp` modules.

use serde_json::json;
use trust_tasks_build::{generate_module, Bundle, Target};

fn bundle() -> Bundle {
    let branch = |own: &str| {
        json!({
            "type": "object",
            "additionalProperties": true,
            "required": ["shared", own],
            "properties": { "shared": {"type": "string"}, own: {"type": "string"} }
        })
    };
    // `Left` excludes `Right` by a `not`, as `hcp/_shared`'s `Disclosure`
    // excludes `Entry`. Without it typify would find the two branches
    // mutually exclusive (each requires a member the other does not
    // declare) and render an untagged enum; a `not` defeats that check.
    let mut left = branch("left");
    left["not"] = json!({"anyOf": [{"required": ["right"]}]});
    let schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://trusttasks.org/spec/example/any-of/0.1",
        "title": "Example AnyOf 0.1 — request payload",
        "type": "object",
        "additionalProperties": false,
        "required": ["item"],
        "properties": {
            "item": {"anyOf": [{"$ref": "#/$defs/Left"}, {"$ref": "#/$defs/Right"}]}
        },
        "$defs": { "Left": left, "Right": branch("right") }
    });
    let bundle = json!({
        "bundleVersion": 1,
        "typeUri": "https://trusttasks.org/spec/example/any-of/0.1",
        "slug": "example/any-of",
        "version": "0.1",
        "status": "draft",
        "deprecation": null,
        "extends": null,
        "policy": {
            "bearer": false,
            "proofRequired": {"request": false, "response": false},
            "issuedAtRequired": {"request": false, "response": false},
            "recipientRequired": {"request": false, "response": false},
            "maxDocumentBytes": {"request": null, "response": null}
        },
        "errorCodes": [],
        "schema": schema,
        "responseSchema": null
    });
    Bundle::from_slice(&serde_json::to_vec(&bundle).unwrap()).expect("fixture bundle")
}

fn compact(tokens: impl quote::ToTokens) -> String {
    tokens.to_token_stream().to_string().replace(' ', "")
}

#[test]
fn an_any_of_of_objects_reads_each_branch_from_its_own_copy() {
    for (target, serde, serde_json) in [
        (Target::runtime_crate(), "serde", "serde_json"),
        (
            Target::application(),
            "::trust_tasks_rs::__private::serde",
            "::trust_tasks_rs::__private::serde_json",
        ),
    ] {
        let source = generate_module(&bundle(), &target).expect("generate");
        let file = syn::parse_file(&source).expect("generated module parses");

        // typify's rendering of the `anyOf`: the struct of `subtype_N`s.
        let item = file
            .items
            .iter()
            .find_map(|i| match i {
                syn::Item::Struct(s)
                    if s.fields
                        .iter()
                        .any(|f| f.ident.as_ref().is_some_and(|id| id == "subtype_0")) =>
                {
                    Some(s)
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("no anyOf struct in:\n{source}"));
        let name = item.ident.to_string();
        let derives: String = item
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("derive"))
            .map(compact)
            .collect();
        assert!(
            derives.contains("Serialize") && !derives.contains("Deserialize"),
            "{derives}"
        );

        let deserialize = file
            .items
            .iter()
            .find_map(|i| match i {
                syn::Item::Impl(imp)
                    if compact(&imp.self_ty) == name
                        && imp.trait_.as_ref().is_some_and(|(_, p, _)| {
                            p.segments.last().is_some_and(|s| s.ident == "Deserialize")
                        }) =>
                {
                    Some(compact(imp))
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("no hand-written Deserialize for {name}"));
        assert!(
            deserialize.contains(&format!("impl<'de>{serde}::Deserialize<'de>for{name}")),
            "{deserialize}"
        );
        // First match wins: `subtype_0` reads a copy, `subtype_1` (the last)
        // takes the value itself, and each returns with only its own field
        // set.
        for expected in [
            format!("lete0=match{serde_json}::from_value(value.clone()"),
            format!("lete1=match{serde_json}::from_value(value"),
            "Ok(Self{subtype_0:::std::option::Option::Some(branch),subtype_1:::std::option::Option::None".to_owned(),
            "Ok(Self{subtype_0:::std::option::Option::None,subtype_1:::std::option::Option::Some(branch)".to_owned(),
            format!("anyOfbranchof{name}:subtype_0:{{e0}};subtype_1:{{e1}}"),
        ] {
            assert!(deserialize.contains(&expected), "{expected} in {deserialize}");
        }
        assert_eq!(
            deserialize.matches("value.clone()").count(),
            1,
            "{deserialize}"
        );
    }
}
