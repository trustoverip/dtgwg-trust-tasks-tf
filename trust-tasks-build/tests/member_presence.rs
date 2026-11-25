//! The types `generate_module` emits keep an optional member's presence: an
//! absent member, a present empty array or map, and an explicit `null` where
//! the schema admits one must each survive a decode-then-encode round trip.
//!
//! typify renders an optional array or map as a bare `Vec` / map skipped when
//! empty (`[]` and `{}` come back absent), an optional nullable member as an
//! `Option` skipped when `None` (`null` comes back absent), and an optional
//! `{"type": "null"}` member as `()` with `#[serde(default)]` (absent comes
//! back as `null`). These tests read the field declarations off a fixture
//! module; `trust-tasks-rs/tests/member_presence.rs` holds the same property
//! on generated `acl` types at run time.

use serde_json::json;
use trust_tasks_build::{generate_module, Bundle, Target};

fn fixture_bundle() -> Bundle {
    let schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://trusttasks.org/spec/example/presence/0.1",
        "title": "Example Presence 0.1 — request payload",
        "type": "object",
        "additionalProperties": false,
        "required": ["requiredNullable"],
        "properties": {
            "list": {"type": "array", "items": {"type": "string"}, "description": "A list."},
            "map": {"type": "object", "additionalProperties": {"type": "string"}},
            "object": {"type": "object", "additionalProperties": true},
            "nullableString": {
                "type": ["string", "null"],
                "maxLength": 8,
                "description": "Explicit null clears it; absent leaves it."
            },
            "nullableRef": {"oneOf": [{"type": "null"}, {"$ref": "#/$defs/Thing"}]},
            "nullableList": {"type": ["array", "null"], "items": {"type": "string"}},
            "nullOnly": {"type": "null", "description": "Reserved; null where present."},
            "plain": {"type": "string"},
            "defaulted": {"type": "boolean", "default": true},
            "requiredNullable": {"type": ["string", "null"]}
        },
        "$defs": {
            "Thing": {
                "type": "object",
                "additionalProperties": false,
                "properties": {"inner": {"type": ["integer", "null"]}}
            }
        }
    });
    let bundle = json!({
        "bundleVersion": 1,
        "typeUri": "https://trusttasks.org/spec/example/presence/0.1",
        "slug": "example/presence",
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

fn module(target: &Target) -> syn::File {
    let source = generate_module(&fixture_bundle(), target).expect("generate");
    syn::parse_file(&source).expect("generated module parses")
}

fn compact(tokens: impl quote::ToTokens) -> String {
    // prettyplease breaks long generics with a trailing comma.
    tokens
        .to_token_stream()
        .to_string()
        .replace(' ', "")
        .replace(",>", ">")
}

/// `(field type, serde attribute)` of `ty.field`, outside `mod builder`.
fn field(file: &syn::File, ty: &str, name: &str) -> (String, String) {
    for item in &file.items {
        let syn::Item::Struct(s) = item else { continue };
        if s.ident != ty {
            continue;
        }
        for f in &s.fields {
            if f.ident.as_ref().is_some_and(|i| i == name) {
                let serde = f
                    .attrs
                    .iter()
                    .find(|a| a.path().is_ident("serde"))
                    .map(compact)
                    .unwrap_or_default();
                return (compact(&f.ty), serde);
            }
        }
    }
    panic!("no field {ty}.{name}");
}

/// The type a builder holds for `ty.field`, inside `mod builder`.
fn builder_field(file: &syn::File, ty: &str, name: &str) -> String {
    for item in &file.items {
        let syn::Item::Mod(m) = item else { continue };
        if m.ident != "builder" {
            continue;
        }
        for item in &m.content.as_ref().unwrap().1 {
            let syn::Item::Struct(s) = item else { continue };
            if s.ident != ty {
                continue;
            }
            for f in &s.fields {
                if f.ident.as_ref().is_some_and(|i| i == name) {
                    return compact(&f.ty);
                }
            }
        }
    }
    panic!("no builder field {ty}.{name}");
}

const OPTION: &str = "::std::option::Option";
const PRESENT: &str = "deserialize_with=\"presence::present\"";

fn assert_presence_kept(file: &syn::File, ty: &str, name: &str, inner: &str) {
    let (field_ty, serde) = field(file, ty, name);
    assert_eq!(field_ty, format!("{OPTION}<{inner}>"), "{ty}.{name}");
    assert!(serde.contains(PRESENT), "{ty}.{name}: {serde}");
    assert!(
        serde.contains("skip_serializing_if=\"::std::option::Option::is_none\""),
        "{ty}.{name}: {serde}"
    );
    assert!(!serde.contains("is_empty"), "{ty}.{name}: {serde}");
    assert_eq!(
        builder_field(file, ty, name),
        format!("::std::result::Result<{OPTION}<{inner}>,::std::string::String>"),
        "builder {ty}.{name}"
    );
}

#[test]
fn an_optional_array_or_map_is_an_option_so_empty_stays_present() {
    let file = module(&Target::runtime_crate());
    assert_presence_kept(
        &file,
        "Payload",
        "list",
        "::std::vec::Vec<::std::string::String>",
    );
    assert_presence_kept(
        &file,
        "Payload",
        "map",
        "::std::collections::HashMap<::std::string::String,::std::string::String>",
    );
    assert_presence_kept(
        &file,
        "Payload",
        "object",
        "::serde_json::Map<::std::string::String,::serde_json::Value>",
    );
}

#[test]
fn an_optional_nullable_member_is_a_double_option_so_null_stays_null() {
    let file = module(&Target::runtime_crate());
    assert_presence_kept(
        &file,
        "Payload",
        "nullable_string",
        &format!("{OPTION}<PayloadNullableString>"),
    );
    assert_presence_kept(
        &file,
        "Payload",
        "nullable_ref",
        &format!("{OPTION}<Thing>"),
    );
    assert_presence_kept(
        &file,
        "Payload",
        "nullable_list",
        &format!("{OPTION}<::std::vec::Vec<::std::string::String>>"),
    );
    // In a `$defs` type as well as the root.
    assert_presence_kept(&file, "Thing", "inner", &format!("{OPTION}<i64>"));
}

#[test]
fn an_optional_null_only_member_is_absent_unless_present() {
    let file = module(&Target::runtime_crate());
    assert_presence_kept(&file, "Payload", "null_only", "()");
}

#[test]
fn other_members_are_left_as_typify_renders_them() {
    let file = module(&Target::runtime_crate());
    let (ty, serde) = field(&file, "Payload", "plain");
    assert_eq!(ty, format!("{OPTION}<::std::string::String>"));
    assert!(!serde.contains(PRESENT));
    // A declared default: absent reads as the default the schema gives it.
    let (ty, serde) = field(&file, "Payload", "defaulted");
    assert_eq!(ty, "bool");
    assert!(!serde.contains(PRESENT));
    // Required and nullable: always written, `null` included.
    let (ty, serde) = field(&file, "Payload", "required_nullable");
    assert_eq!(ty, format!("{OPTION}<::std::string::String>"));
    assert!(!serde.contains(PRESENT) && !serde.contains("skip_serializing_if"));
}

#[test]
fn the_tag_never_reaches_the_output_and_descriptions_survive() {
    for target in [Target::runtime_crate(), Target::application()] {
        let source = generate_module(&fixture_bundle(), &target).expect("generate");
        assert!(!source.contains("trust-tasks-build:nullable"), "{source}");
        assert!(source.contains("///Explicit null clears it; absent leaves it."));
        // The helper names serde through the target's path.
        assert!(source.contains("mod presence"));
    }
    let app = generate_module(&fixture_bundle(), &Target::application()).unwrap();
    assert!(app.contains("::trust_tasks_rs::__private::serde::Deserializer<'de>"));
}
