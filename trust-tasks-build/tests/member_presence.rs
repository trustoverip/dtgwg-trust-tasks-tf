//! The types `generate_module` emits keep an optional member's presence: an
//! absent member, a present empty array or map, and an explicit `null` where
//! the schema admits one must each survive a decode-then-encode round trip,
//! and a `null` where the schema has no null form is refused, not read as
//! absent.
//!
//! typify renders an optional array or map as a bare `Vec` / map skipped when
//! empty (`[]` and `{}` come back absent), an optional nullable member as an
//! `Option` skipped when `None` (`null` comes back absent), and an optional
//! `{"type": "null"}` member as `()` with `#[serde(default)]` (absent comes
//! back as `null`), and reads a `null` on any other optional member as `None`.
//! These tests read the field declarations off a fixture module, for both
//! targets; `trust-tasks-rs/tests/member_presence.rs` holds the same property
//! on generated types at run time, and `trust-tasks-rs`'s `specs::presence`
//! tests run the helpers on each field shape.

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
            "emptyByDefault": {"type": "array", "items": {"type": "string"}, "default": []},
            "nullByDefault": {"type": ["string", "null"], "default": null},
            "anything": true,
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
    bundle_of(schema)
}

fn bundle_of(schema: serde_json::Value) -> Bundle {
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

/// Both targets: the repository's own modules name the crate-private helpers
/// in `trust-tasks-rs`, an application's carry their own.
fn targets() -> [(Target, &'static str); 2] {
    [
        (Target::runtime_crate(), "crate::specs::presence"),
        (Target::application(), "presence"),
    ]
}

/// Field types as the runtime crate spells them: an application names
/// serde_json through the runtime's `__private`.
fn unprefixed(ty: String) -> String {
    ty.replace("::trust_tasks_rs::__private", "")
}

/// Which helper a field reads through.
#[derive(Clone, Copy, Debug)]
enum Helper {
    /// The member admits `null`: a present `null` is `Some`.
    Present,
    /// It does not: a `null` is refused.
    NotNull,
}

fn reads_through(serde: &str, helpers: &str, helper: Helper) -> bool {
    let name = match helper {
        Helper::Present => "present",
        Helper::NotNull => "not_null",
    };
    serde.contains(&format!("deserialize_with=\"{helpers}::{name}\""))
}

/// `ty.name` is `Option<inner>`, read through `helper`, skipped only when
/// absent, and its builder holds the same `Option`.
fn assert_presence_kept(
    file: &syn::File,
    ty: &str,
    name: &str,
    inner: &str,
    helpers: &str,
    helper: Helper,
) {
    let (field_ty, serde) = field(file, ty, name);
    assert_eq!(
        unprefixed(field_ty),
        format!("{OPTION}<{inner}>"),
        "{ty}.{name}"
    );
    assert!(
        reads_through(&serde, helpers, helper),
        "{ty}.{name} ({helper:?}): {serde}"
    );
    assert!(
        serde.contains("skip_serializing_if=\"::std::option::Option::is_none\""),
        "{ty}.{name}: {serde}"
    );
    assert!(!serde.contains("is_empty"), "{ty}.{name}: {serde}");
    assert_eq!(
        // The builder names the module's own types through `super::`.
        unprefixed(builder_field(file, ty, name)).replace("super::", ""),
        format!("::std::result::Result<{OPTION}<{inner}>,::std::string::String>"),
        "builder {ty}.{name}"
    );
}

#[test]
fn an_optional_array_or_map_is_an_option_so_empty_stays_present() {
    for (target, h) in targets() {
        let file = module(&target);
        let list = "::std::vec::Vec<::std::string::String>";
        assert_presence_kept(&file, "Payload", "list", list, h, Helper::NotNull);
        assert_presence_kept(
            &file,
            "Payload",
            "map",
            "::std::collections::HashMap<::std::string::String,::std::string::String>",
            h,
            Helper::NotNull,
        );
        assert_presence_kept(
            &file,
            "Payload",
            "object",
            "::serde_json::Map<::std::string::String,::serde_json::Value>",
            h,
            Helper::NotNull,
        );
    }
}

#[test]
fn an_optional_nullable_member_is_a_double_option_so_null_stays_null() {
    for (target, h) in targets() {
        let file = module(&target);
        let p = Helper::Present;
        let string = format!("{OPTION}<PayloadNullableString>");
        assert_presence_kept(&file, "Payload", "nullable_string", &string, h, p);
        let thing = format!("{OPTION}<Thing>");
        assert_presence_kept(&file, "Payload", "nullable_ref", &thing, h, p);
        let list = format!("{OPTION}<::std::vec::Vec<::std::string::String>>");
        assert_presence_kept(&file, "Payload", "nullable_list", &list, h, p);
        // In a `$defs` type as well as the root.
        assert_presence_kept(&file, "Thing", "inner", &format!("{OPTION}<i64>"), h, p);
        // `true` accepts `null` like any other value.
        let any = format!("{OPTION}<::serde_json::Value>");
        assert_presence_kept(&file, "Payload", "anything", &any, h, p);
    }
}

#[test]
fn an_optional_null_only_member_is_absent_unless_present() {
    for (target, h) in targets() {
        let file = module(&target);
        assert_presence_kept(&file, "Payload", "null_only", "()", h, Helper::Present);
    }
}

#[test]
fn an_intrinsic_default_is_no_default_so_the_member_keeps_its_presence() {
    // typify reads a `default` equal to the type's intrinsic one (`[]`,
    // `null`) as no default, so the member is rewritten: absent is `None`.
    for (target, h) in targets() {
        let file = module(&target);
        let list = "::std::vec::Vec<::std::string::String>";
        assert_presence_kept(
            &file,
            "Payload",
            "empty_by_default",
            list,
            h,
            Helper::NotNull,
        );
        let string = format!("{OPTION}<::std::string::String>");
        assert_presence_kept(
            &file,
            "Payload",
            "null_by_default",
            &string,
            h,
            Helper::Present,
        );
    }
}

#[test]
fn any_other_optional_member_keeps_its_type_and_refuses_null() {
    for (target, h) in targets() {
        let file = module(&target);
        let name = "plain";
        let (ty, serde) = field(&file, "Payload", name);
        assert_eq!(
            unprefixed(ty),
            format!("{OPTION}<::std::string::String>"),
            "{name}"
        );
        assert!(reads_through(&serde, h, Helper::NotNull), "{name}: {serde}");
        assert!(serde.contains("skip_serializing_if=\"::std::option::Option::is_none\""));
        // The type is unchanged, so the builder is too.
        assert_eq!(
            unprefixed(builder_field(&file, "Payload", name)),
            format!("::std::result::Result<{OPTION}<::std::string::String>,::std::string::String>"),
        );
    }
}

#[test]
fn declared_defaults_and_required_members_are_left_as_typify_renders_them() {
    for (target, _) in targets() {
        let file = module(&target);
        // A declared default: absent reads as the default the schema gives it.
        let (ty, serde) = field(&file, "Payload", "defaulted");
        assert_eq!(ty, "bool");
        assert!(!serde.contains("deserialize_with"), "{serde}");
        // Required and nullable: always written, `null` included.
        let (ty, serde) = field(&file, "Payload", "required_nullable");
        assert_eq!(ty, format!("{OPTION}<::std::string::String>"));
        assert!(!serde.contains("deserialize_with") && !serde.contains("skip_serializing_if"));
    }
}

#[test]
fn the_tag_never_reaches_the_output_and_descriptions_survive() {
    for (target, _) in targets() {
        let source = generate_module(&fixture_bundle(), &target).expect("generate");
        assert!(!source.contains("trust-tasks-build:nullable"), "{source}");
        assert!(source.contains("///Explicit null clears it; absent leaves it."));
    }
    // Only an application's module carries the helpers, naming serde through
    // the target's path; the runtime crate's share `crate::specs::presence`.
    let app = generate_module(&fixture_bundle(), &Target::application()).unwrap();
    assert!(app.contains("mod presence"));
    assert!(app.contains("fn present<") && app.contains("fn not_null<"));
    assert!(app.contains("::trust_tasks_rs::__private::serde::Deserializer<'de>"));
    let own = generate_module(&fixture_bundle(), &Target::runtime_crate()).unwrap();
    assert!(!own.contains("mod presence"));
}

#[test]
fn a_nullable_member_rendered_as_a_bare_collection_fails_generation() {
    // No `type`, so `null` is admitted; typify still renders the member as a
    // map skipped when empty, which no rewrite can make admit `null`.
    let schema = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$id": "https://trusttasks.org/spec/example/presence/0.1",
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "loose": {"additionalProperties": {"type": "string"}}
        }
    });
    for (target, _) in targets() {
        let err = generate_module(&bundle_of(schema.clone()), &target)
            .expect_err("a nullable member typify renders as a bare map");
        let message = format!("{err:#}");
        assert!(
            message.contains("\"loose\"") && message.contains("bare collection"),
            "{message}"
        );
    }
}

/// The text of the `not_null` refusal in `source`.
fn refusal(source: &str) -> String {
    let start = source
        .find("&\"a value")
        .expect("the not_null refusal message")
        + 2;
    let end = start + source[start..].find('"').unwrap();
    source[start..end].to_string()
}

#[test]
fn the_emitted_and_hand_written_refusals_say_the_same() {
    let app = generate_module(&fixture_bundle(), &Target::application()).unwrap();
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../trust-tasks-rs/src/specs/mod.rs"
    );
    // Only in the repository: a packaged crate has no sibling to compare with.
    let Ok(hand_written) = std::fs::read_to_string(path) else {
        return;
    };
    assert_eq!(refusal(&app), refusal(&hand_written));
}
