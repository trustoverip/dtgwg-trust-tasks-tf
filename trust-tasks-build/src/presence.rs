//! Keeping a member's presence through a deserialise-then-serialise round trip.
//!
//! typify models an optional member by the type's "intrinsic default", which
//! loses information wherever a schema gives absent, empty and `null`
//! different meanings:
//!
//! - an optional array or map becomes a bare `Vec` / `Map` / `HashMap` with
//!   `#[serde(default, skip_serializing_if = "…::is_empty")]`, so a present
//!   `[]` or `{}` is read as, and written back as, absent;
//! - an optional member whose schema admits `null` becomes `Option<T>` with
//!   `skip_serializing_if = "Option::is_none"`, so an explicit `null` is read
//!   as, and written back as, absent;
//! - an optional member whose schema is `{"type": "null"}` becomes `()` with
//!   `#[serde(default)]`, so an absent member is written back as `null`;
//! - any other optional member becomes `Option<T>`, and serde reads a `null`
//!   there as `None`.
//!
//! This module rewrites those fields so the three states stay apart wherever
//! the schema gives them apart:
//!
//! | schema of an optional member | field type | absent | `null` | value |
//! |---|---|---|---|---|
//! | array / map | `Option<Vec<T>>` / `Option<Map>` | `None` | `None` | `Some(v)` (`[]` too) |
//! | admits `null` | `Option<Option<T>>` | `None` | `Some(None)` | `Some(Some(v))` |
//! | `{"type": "null"}` | `Option<()>` | `None` | `Some(())` | — |
//! | anything else | `Option<T>` (unchanged) | `None` | `None` | `Some(v)` |
//!
//! Every rewritten field keeps `skip_serializing_if = "Option::is_none"`, so
//! only an absent member is omitted. A member that admits `null` reads through
//! the `present` helper (`deserialize_with`), which maps a present member,
//! `null` included, to `Some`. For `Target::runtime_crate()` the helper is the
//! crate-private `crate::specs::__presence::present` in `trust-tasks-rs`,
//! which adds no public API. For an application it is emitted into each module
//! that needs it, so a generated module names nothing new in the runtime's
//! `__private` surface (and so no new ABI item).
//!
//! A `null` on a member whose schema has no null form is read as absent, not
//! refused: decoding enforces what the Rust type can express, and refusing a
//! `null` the schema forbids is payload-schema validation's job
//! (`PayloadPolicy::Validate`, the `validate` feature), as in the other three
//! runtimes. A per-field refusal (`deserialize_with` on ~14,600 fields of the
//! prebuilt tree) cost about 2 GiB of compiler memory and was dropped.
//!
//! A member whose schema is `true`, `{}` or annotations only accepts any
//! value, `null` included. It is tagged like any other member that admits
//! `null` (a `true` is rewritten to `{}` in typify's copy so it can carry the
//! tag), so it becomes `Option<Option<serde_json::Value>>`: absent is `None`,
//! `null` is `Some(None)`.
//!
//! typify's output does not say which `Option<T>` came from a nullable schema
//! (it wraps a non-nullable optional member in the same `Option`), so
//! [`mark_nullable_optional_members`] tags those members' `description` in
//! the copy of the schema typify sees, and [`apply_presence`] reads the tag
//! back off the field's doc comment and removes it. The wire schema
//! (`PAYLOAD_SCHEMA`) is captured before, and never carries the tag.
//!
//! Each tag carries its own id, and generation fails if one is never read
//! back, so a description typify drops cannot silently turn a member that
//! admits `null` into one that reads its `null` as absent. A tagged member typify renders as
//! anything but an `Option` (a bare `Vec` or map skipped when empty) fails
//! generation too: no rewrite of that field can both keep `[]` and `null`
//! apart and admit `null`.
//!
//! A member with a declared JSON Schema `default` is left as typify renders it
//! (`#[serde(default = "…")]`), so an absent member reads as that default.
//! typify treats a default equal to the type's intrinsic one (`[]`, `{}`,
//! `null`, `false`, `0`, `""`) as no default at all, so such a member is
//! rewritten like any other optional member, and an absent one reads as
//! `None`.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, Result};
use proc_macro2::TokenStream;
use serde_json::Value;
use syn::visit_mut::VisitMut;

/// Start of the tag appended to the `description` of a nullable optional
/// member in the schema typify sees: `{NULLABLE_MARKER}{id}]]`, one id per
/// tagged member. Never in the output: [`apply_presence`] removes every
/// occurrence and [`assert_no_marker`] fails generation if one survives.
pub(crate) const NULLABLE_MARKER: &str = "[[trust-tasks-build:nullable-optional-member:";

/// End of the tag.
const MARKER_END: &str = "]]";

/// One tagged member.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Tag {
    /// The member's JSON name.
    pub(crate) name: String,
    /// Whether typify may render the member as an enum variant's field: it
    /// sits under a `oneOf` / `anyOf` branch, or on an object whose own
    /// `oneOf` / `anyOf` (directly or through an `allOf` branch) typify
    /// merges its members into. Only such a tag can be accounted for by a
    /// variant's field (see [`apply_presence`]).
    pub(crate) in_branch: bool,
}

/// The tagged members of one schema, by tag id.
pub(crate) type NullableTags = BTreeMap<usize, Tag>;

fn marker(id: usize) -> String {
    format!("{NULLABLE_MARKER}{id}{MARKER_END}")
}

/// Name of the per-module helper module.
const HELPER_MODULE: &str = "presence";

/// Annotation-only keywords: a schema carrying nothing else accepts any
/// value, `null` included.
const ANNOTATIONS: &[&str] = &[
    "description",
    "title",
    "examples",
    "$comment",
    "deprecated",
    "readOnly",
    "writeOnly",
    "$anchor",
    "$id",
];

/// Tag every optional member whose schema admits `null` (see the module
/// docs). Runs on the typify copy, after `migrate_defs_to_definitions`, so
/// local references read `#/definitions/…`. Returns each tag's id, the JSON
/// name of the member it is on, and whether that member sits under a
/// `oneOf` / `anyOf` branch.
pub(crate) fn mark_nullable_optional_members(schema: &mut Value) -> NullableTags {
    let root = schema.clone();
    let mut tagged = NullableTags::new();
    mark(schema, &root, &mut tagged, false);
    tagged
}

/// Whether an object schema carries a `oneOf` / `anyOf`, directly or in one
/// of its `allOf` branches (typify merges those into the object).
fn has_union(map: &serde_json::Map<String, Value>) -> bool {
    map.contains_key("oneOf")
        || map.contains_key("anyOf")
        || map
            .get("allOf")
            .and_then(Value::as_array)
            .is_some_and(|branches| {
                branches
                    .iter()
                    .any(|b| b.as_object().is_some_and(has_union))
            })
}

/// `in_branch`: `node` sits under a `oneOf` / `anyOf` branch.
fn mark(node: &mut Value, root: &Value, tagged: &mut NullableTags, in_branch: bool) {
    match node {
        Value::Object(map) => {
            let union_here = has_union(map);
            // typify can merge this object's own members into the variants
            // of its `oneOf` / `anyOf`.
            let props_in_branch = in_branch || union_here;
            let required: BTreeSet<String> = map
                .get("required")
                .and_then(Value::as_array)
                .map(|r| {
                    r.iter()
                        .filter_map(|v| v.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default();
            if let Some(Value::Object(props)) = map.get_mut("properties") {
                for (name, prop) in props.iter_mut() {
                    if required.contains(name) || !admits_null(prop, root, 0) {
                        continue;
                    }
                    // `true` cannot carry a tag; `{}` means the same and can.
                    if prop == &Value::Bool(true) {
                        *prop = Value::Object(serde_json::Map::new());
                    }
                    // A bare `{"type": "null"}` is typify's `()`, recognised in
                    // the output without a tag.
                    let Value::Object(p) = prop else { continue };
                    if is_null_only(p) {
                        continue;
                    }
                    let id = tagged.len();
                    let tag = marker(id);
                    let description = match p.get("description").and_then(Value::as_str) {
                        Some(d) => format!("{d}\n\n{tag}"),
                        None => tag,
                    };
                    p.insert("description".into(), Value::String(description));
                    tagged.insert(
                        id,
                        Tag {
                            name: name.clone(),
                            in_branch: props_in_branch,
                        },
                    );
                }
            }
            for (key, value) in map.iter_mut() {
                let child_in_branch = in_branch
                    || matches!(key.as_str(), "oneOf" | "anyOf")
                    || (key == "allOf" && union_here);
                mark(value, root, tagged, child_in_branch);
            }
        }
        Value::Array(items) => {
            for item in items {
                mark(item, root, tagged, in_branch);
            }
        }
        _ => {}
    }
}

fn is_null_only(p: &serde_json::Map<String, Value>) -> bool {
    p.get("type").and_then(Value::as_str) == Some("null")
}

/// Whether `schema` accepts JSON `null`. `$ref`s into the document are
/// followed; `depth` bounds a reference cycle, and a schema nested past it
/// counts as admitting `null`.
///
/// `$ref` follows JSON Schema 2020-12: it is one keyword among its siblings,
/// so a `$ref` with a `type` beside it must satisfy both. (Under Draft-07 and
/// earlier, a `$ref`'s siblings are ignored.) Every schema in the registry is
/// 2020-12; an application's schema in an earlier draft whose `$ref` has
/// constraining siblings gets the 2020-12 reading.
///
/// A schema admits `null` only when every keyword that can restrict a value's
/// type does: `type`, `const`, `enum`, `oneOf` / `anyOf` (some branch),
/// `allOf` (every branch), `$ref`, and a `not` of a schema that is exactly
/// `null`. Any other keyword (`not` of anything else, `if`, a type-specific
/// keyword such as `minLength`) is taken to admit it. The answer drives
/// whether a `null` is kept or read as absent, and erring towards "admits"
/// only keeps a `null` the schema might refuse (payload-schema validation
/// still refuses it), where erring the other way would lose a `null` the
/// schema accepts.
fn admits_null(schema: &Value, root: &Value, depth: usize) -> bool {
    if depth > 64 {
        // The safe direction: keep a `null` rather than lose one.
        return true;
    }
    let map = match schema {
        Value::Bool(b) => return *b,
        Value::Object(map) => map,
        _ => return false,
    };
    if let Some(t) = map.get("type") {
        let ok = match t {
            Value::String(s) => s == "null",
            Value::Array(types) => types.iter().any(|t| t == "null"),
            _ => false,
        };
        if !ok {
            return false;
        }
    }
    if map.get("const").is_some_and(|c| !c.is_null()) {
        return false;
    }
    if let Some(Value::Array(values)) = map.get("enum") {
        if !values.iter().any(Value::is_null) {
            return false;
        }
    }
    for key in ["oneOf", "anyOf"] {
        if let Some(Value::Array(branches)) = map.get(key) {
            if !branches.iter().any(|b| admits_null(b, root, depth + 1)) {
                return false;
            }
        }
    }
    if let Some(Value::String(target)) = map.get("$ref") {
        if resolve(root, target).is_some_and(|t| !admits_null(t, root, depth + 1)) {
            return false;
        }
    }
    if let Some(Value::Array(branches)) = map.get("allOf") {
        if !branches.iter().all(|b| admits_null(b, root, depth + 1)) {
            return false;
        }
    }
    !map.get("not").is_some_and(is_exactly_null)
}

/// `{"type": "null"}` or `{"const": null}`, and nothing else that constrains.
fn is_exactly_null(schema: &Value) -> bool {
    let Value::Object(map) = schema else {
        return false;
    };
    let constraining: Vec<&String> = map
        .keys()
        .filter(|k| !ANNOTATIONS.contains(&k.as_str()))
        .collect();
    match constraining.as_slice() {
        [k] if *k == "type" => map.get("type") == Some(&Value::String("null".into())),
        [k] if *k == "const" => map.get("const") == Some(&Value::Null),
        _ => false,
    }
}

fn resolve<'a>(root: &'a Value, reference: &str) -> Option<&'a Value> {
    let pointer = reference.strip_prefix('#')?;
    root.pointer(pointer)
}

/// The rewritten fields as `(type, field)`, so the builder's copy of each
/// field and its setter can follow. Every rewrite wraps the field's type in
/// one more `Option`, so the builder (which names the module's types through
/// `super::`) wraps its own spelling of the type the same way.
type Rewrites = BTreeSet<(String, String)>;

/// Rewrite the fields described in the module docs, follow each into the
/// struct's builder, and emit the helper the fields use. `serde` is the path
/// generated code names serde by. `shared` is the path of a module that
/// already defines the helper (`crate::specs::__presence` inside
/// `trust-tasks-rs`), or `None` to emit it into this module. `tags` is
/// what [`mark_nullable_optional_members`] returned.
///
/// Every tag must be read back off a field. A field typify made required (a
/// `oneOf` branch requiring the member) reads its tag and needs no rewrite.
/// An enum variant's fields carry no docs, so a tag on a member typify put in
/// a variant cannot come back. Only a tag `mark` placed under a `oneOf` /
/// `anyOf` branch ([`Tag::in_branch`]) can be on such a member: it is
/// accounted for by a variant's required field of that name, and an optional
/// variant field of that name is refused (it could be either). Any other tag
/// left unread fails generation, even where some variant has a field of the
/// same name.
///
/// What this cannot see: where typify copies one description onto two fields
/// (merging an `allOf`, say), the id read back off one copy also accounts for
/// a copy that lost it. The output does not say which fields are copies, so
/// that case is not caught.
pub(crate) fn apply_presence(
    file: &mut syn::File,
    serde: &TokenStream,
    shared: Option<&str>,
    tags: &NullableTags,
) -> Result<()> {
    // Names a variant's field may carry a tag for.
    let tagged: BTreeSet<String> = tags
        .values()
        .filter(|t| t.in_branch)
        .map(|t| t.name.clone())
        .collect();
    let tagged = &tagged;
    let mut rewriter = Rewriter {
        helpers: shared.unwrap_or(HELPER_MODULE).to_string(),
        ..Rewriter::default()
    };
    for item in file.items.iter_mut() {
        match item {
            syn::Item::Struct(s) => {
                let owner = s.ident.to_string();
                if let syn::Fields::Named(fields) = &mut s.fields {
                    for field in fields.named.iter_mut() {
                        rewriter.field(Some(&owner), field, tagged);
                    }
                }
            }
            syn::Item::Enum(e) => {
                for variant in e.variants.iter_mut() {
                    if let syn::Fields::Named(fields) = &mut variant.fields {
                        for field in fields.named.iter_mut() {
                            rewriter.field(None, field, tagged);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    if let Some(name) = rewriter.ambiguous.first() {
        // typify writes no doc comment on an enum variant's fields, so the tag
        // cannot come back there; refuse rather than guess from the name (a
        // wrong "no" would read the member's `null` as absent and lose it).
        return Err(anyhow!(
            "optional member {name:?} of an enum variant: a member of the same name elsewhere \
             in the schema admits null, and typify writes no doc on a variant's fields, so \
             whether this one admits null cannot be read back; no field rewrite can be chosen \
             safely (rename one of the members, or make the variant a `$defs` type)"
        ));
    }
    if let Some((owner, name)) = rewriter.not_option.first() {
        let owner = match owner {
            Some(owner) => format!("of `{owner}`"),
            None => "of an enum variant".to_string(),
        };
        return Err(anyhow!(
            "optional member {name:?} {owner} admits null, but typify rendered it as a bare collection \
             (skipped when empty) rather than an `Option`: no field rewrite can keep `[]` / `{{}}` \
             apart from absent and still admit null (give the member an explicit `type` that \
             includes \"null\", or one that excludes it)"
        ));
    }
    let unread: Vec<String> = tags
        .iter()
        .filter(|(id, tag)| {
            !rewriter.read_back.contains(id)
                && !(tag.in_branch && rewriter.in_variant.contains(&tag.name))
        })
        .map(|(id, tag)| format!("{:?} (tag {id})", tag.name))
        .collect();
    if !unread.is_empty() {
        // typify dropped the member's description, so whether its field
        // admits `null` cannot be read back; guessing "no" would read the
        // member's `null` as absent and lose it.
        return Err(anyhow!(
            "optional member(s) {} admit null, but the tag marking them was not found on any \
             field typify generated, so the field's null handling cannot be chosen safely \
             (typify dropped the member's description: move the member's schema into a `$defs` \
             type, or report this as a trust-tasks-build bug)",
            unread.join(", ")
        ));
    }
    // Every doc comment, not only the fields': typify also copies a member's
    // description onto the newtype it makes for a constrained member.
    StripMarker.visit_file_mut(file);

    if rewriter.rewrites.is_empty() && !rewriter.used_present {
        return Ok(());
    }
    for item in file.items.iter_mut() {
        if let syn::Item::Mod(m) = item {
            if m.ident == "builder" {
                if let Some((_, items)) = &mut m.content {
                    follow_into_builder(items, &rewriter.rewrites);
                }
            }
        }
    }
    if shared.is_some() {
        return Ok(());
    }
    // Only when this module uses it, so it is never dead code.
    if !rewriter.used_present {
        return Ok(());
    }
    let present: syn::Item = syn::parse_quote! {
        /// Reads a present member, `null` included, as `Some`, so an
        /// absent member (`None`, via `#[serde(default)]`) stays distinct.
        pub(super) fn present<'de, D, T>(
            deserializer: D,
        ) -> ::std::result::Result<::std::option::Option<T>, D::Error>
        where
            D: #serde::Deserializer<'de>,
            T: #serde::Deserialize<'de>,
        {
            <T as #serde::Deserialize<'de>>::deserialize(deserializer)
                .map(::std::option::Option::Some)
        }
    };
    let helper: syn::Item = syn::parse_quote! {
        #[doc(hidden)]
        mod presence {
            #present
        }
    };
    file.items.push(helper);
    Ok(())
}

#[derive(Default)]
struct Rewriter {
    /// Path the fields name the helper by: `presence` (emitted into the
    /// module) or a shared module.
    helpers: String,
    rewrites: Rewrites,
    /// Optional enum-variant fields whose JSON name was tagged somewhere in
    /// the schema: their own nullability cannot be read back.
    ambiguous: Vec<String>,
    /// Tagged members typify rendered as something other than an `Option`:
    /// the owning struct (`None` for an enum variant) and the JSON name.
    not_option: Vec<(Option<String>, String)>,
    /// Tag ids read back off a field.
    read_back: BTreeSet<usize>,
    /// JSON names of required enum-variant fields, which account for a tag
    /// on a member of that name (see [`apply_presence`]).
    in_variant: BTreeSet<String>,
    used_present: bool,
}

impl Rewriter {
    fn field(&mut self, owner: Option<&str>, field: &mut syn::Field, tagged: &BTreeSet<String>) {
        // First, so a field without a `#[serde(…)]` (a required member typify
        // gave no rename) still reads its tag back.
        let ids = take_marker(&mut field.attrs);
        let has_tag = !ids.is_empty();
        self.read_back.extend(ids);
        let Some(serde) = SerdeAttr::read(&field.attrs) else {
            if owner.is_none() {
                if let Some(ident) = &field.ident {
                    self.in_variant
                        .insert(ident.to_string().trim_start_matches("r#").to_string());
                }
            }
            return;
        };
        let json_name = serde.rename.clone().unwrap_or_else(|| {
            field
                .ident
                .as_ref()
                .map(|i| i.to_string().trim_start_matches("r#").to_string())
                .unwrap_or_default()
        });
        let ty = &field.ty;
        // Optional as typify rendered it: `Option<T>`, `default`, skipped
        // when `None`. A tagged member typify made required (a `oneOf` branch
        // that requires it) is already written whenever it is present.
        let optional_option = serde.default && serde.skip_if_none() && is_option(ty);
        // Optional through an intrinsic default, but not as an `Option`.
        let optional_other = !optional_option && serde.default && !is_unit(ty);
        if has_tag && optional_other {
            self.not_option.push((owner.map(str::to_owned), json_name));
            return;
        }
        if owner.is_none() && !has_tag && tagged.contains(&json_name) {
            if optional_other {
                self.ambiguous.push(json_name);
                return;
            }
            if !optional_option {
                self.in_variant.insert(json_name.clone());
            }
        }
        // Every rewrite wraps the type in one more `Option`; `nullable`: the
        // schema admits `null`, so it reads through `present`.
        let nullable = if optional_option {
            if has_tag {
                true
            } else if owner.is_none() && tagged.contains(&json_name) {
                self.ambiguous.push(json_name);
                return;
            } else {
                // Not nullable: left as typify renders it (a `null` reads as
                // `None`, as absent; payload-schema validation refuses it).
                return;
            }
        } else if serde.default && serde.skip_if_empty() {
            false
        } else if serde.default && serde.skip.is_none() && is_unit(ty) {
            true
        } else {
            return;
        };
        // A non-nullable array or map needs no helper: `default` reads an
        // absent member as `None`, and serde reads a `null` there as `None`.
        let helper = nullable.then(|| {
            self.used_present = true;
            format!("{}::present", self.helpers)
        });
        serde.rewrite(&mut field.attrs, helper.as_deref());
        field.ty = wrap_in_option(ty);
        if let (Some(owner), Some(ident)) = (owner, &field.ident) {
            self.rewrites.insert((owner.to_string(), ident.to_string()));
        }
    }
}

/// The parts of a field's `#[serde(…)]` attribute that decide its presence.
#[derive(Clone)]
struct SerdeAttr {
    rename: Option<String>,
    default: bool,
    skip: Option<String>,
}

impl SerdeAttr {
    fn read(attrs: &[syn::Attribute]) -> Option<Self> {
        let attr = attrs.iter().find(|a| a.path().is_ident("serde"))?;
        let mut out = SerdeAttr {
            rename: None,
            default: false,
            skip: None,
        };
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                out.rename = Some(meta.value()?.parse::<syn::LitStr>()?.value());
            } else if meta.path.is_ident("default") {
                if meta.input.peek(syn::Token![=]) {
                    // `default = "path"`: a declared default, left alone.
                    let _: syn::LitStr = meta.value()?.parse()?;
                } else {
                    out.default = true;
                }
            } else if meta.path.is_ident("skip_serializing_if") {
                out.skip = Some(meta.value()?.parse::<syn::LitStr>()?.value());
            } else if meta.input.peek(syn::Token![=]) {
                let _: syn::Expr = meta.value()?.parse()?;
            }
            Ok(())
        })
        .ok()?;
        Some(out)
    }

    fn skip_if_none(&self) -> bool {
        self.skip
            .as_deref()
            .is_some_and(|s| s.replace(' ', "") == "::std::option::Option::is_none")
    }

    fn skip_if_empty(&self) -> bool {
        self.skip
            .as_deref()
            .is_some_and(|s| s.replace(' ', "").ends_with("::is_empty"))
    }

    /// Replace the field's `#[serde(…)]` with the presence-keeping form,
    /// keeping its `rename`.
    fn rewrite(&self, attrs: &mut [syn::Attribute], helper: Option<&str>) {
        let attr: syn::Attribute = match (&self.rename, helper) {
            (Some(rename), Some(helper)) => syn::parse_quote! {
                #[serde(
                    rename = #rename,
                    default,
                    deserialize_with = #helper,
                    skip_serializing_if = "::std::option::Option::is_none"
                )]
            },
            (None, Some(helper)) => syn::parse_quote! {
                #[serde(
                    default,
                    deserialize_with = #helper,
                    skip_serializing_if = "::std::option::Option::is_none"
                )]
            },
            (Some(rename), None) => syn::parse_quote! {
                #[serde(
                    rename = #rename,
                    default,
                    skip_serializing_if = "::std::option::Option::is_none"
                )]
            },
            (None, None) => syn::parse_quote! {
                #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
            },
        };
        if let Some(slot) = attrs.iter_mut().find(|a| a.path().is_ident("serde")) {
            *slot = attr;
        }
    }
}

fn last_segment(ty: &syn::Type) -> Option<&syn::PathSegment> {
    match ty {
        syn::Type::Path(p) if p.qself.is_none() => p.path.segments.last(),
        _ => None,
    }
}

fn is_option(ty: &syn::Type) -> bool {
    last_segment(ty).is_some_and(|s| s.ident == "Option")
}

fn is_unit(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty())
}

/// Remove the tags from a field's doc comment; the ids it carried.
fn take_marker(attrs: &mut Vec<syn::Attribute>) -> Vec<usize> {
    let mut found = Vec::new();
    attrs.retain_mut(|attr| match strip_doc(attr) {
        DocStrip::Untouched => true,
        DocStrip::Stripped { now_empty, ids } => {
            found.extend(ids);
            !now_empty
        }
    });
    found
}

enum DocStrip {
    Untouched,
    Stripped { now_empty: bool, ids: Vec<usize> },
}

/// `text` without its tags (and the blank line before each), and the ids.
fn strip_tags(text: &str) -> (String, Vec<usize>) {
    let mut out = String::with_capacity(text.len());
    let mut ids = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find(NULLABLE_MARKER) {
        let after = &rest[start + NULLABLE_MARKER.len()..];
        let digits = after.bytes().take_while(u8::is_ascii_digit).count();
        let Some(tail) = after[digits..].strip_prefix(MARKER_END) else {
            // Not one of ours; keep it, and let `assert_no_marker` refuse it.
            out.push_str(&rest[..start + NULLABLE_MARKER.len()]);
            rest = after;
            continue;
        };
        let head = &rest[..start];
        out.push_str(head.strip_suffix("\n\n").unwrap_or(head));
        ids.extend(after[..digits].parse::<usize>().ok());
        rest = tail;
    }
    out.push_str(rest);
    (out, ids)
}

fn strip_doc(attr: &mut syn::Attribute) -> DocStrip {
    if !attr.path().is_ident("doc") {
        return DocStrip::Untouched;
    }
    let syn::Meta::NameValue(nv) = &mut attr.meta else {
        return DocStrip::Untouched;
    };
    let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(text),
        ..
    }) = &mut nv.value
    else {
        return DocStrip::Untouched;
    };
    let (stripped, ids) = strip_tags(&text.value());
    if ids.is_empty() {
        return DocStrip::Untouched;
    }
    let now_empty = stripped.trim().is_empty();
    *text = syn::LitStr::new(&stripped, text.span());
    DocStrip::Stripped { now_empty, ids }
}

/// Removes the marker from every remaining doc comment (a newtype typify made
/// for a tagged member carries its description too).
struct StripMarker;

impl StripMarker {
    fn attrs(attrs: &mut [syn::Attribute], name: &syn::Ident) {
        // typify heads an undocumented type with its own name; where the
        // description held only the marker, the heading takes its place.
        let heading = format!("`{name}`");
        for attr in attrs.iter_mut() {
            if let DocStrip::Stripped {
                now_empty: true, ..
            } = strip_doc(attr)
            {
                *attr = syn::parse_quote!(#[doc = #heading]);
            }
        }
    }
}

impl VisitMut for StripMarker {
    fn visit_item_struct_mut(&mut self, item: &mut syn::ItemStruct) {
        Self::attrs(&mut item.attrs, &item.ident);
        syn::visit_mut::visit_item_struct_mut(self, item);
    }
    fn visit_item_enum_mut(&mut self, item: &mut syn::ItemEnum) {
        Self::attrs(&mut item.attrs, &item.ident);
        syn::visit_mut::visit_item_enum_mut(self, item);
    }
    fn visit_attribute_mut(&mut self, attr: &mut syn::Attribute) {
        // Any other doc (a field, a variant): strip in place.
        let _ = strip_doc(attr);
    }
}

fn wrap_in_option(ty: &syn::Type) -> syn::Type {
    syn::parse_quote!(::std::option::Option<#ty>)
}

/// Wrap the first generic argument of the last path segment named `outer`
/// (`Result<T, String>`, `TryInto<T>`) in one more `Option`.
fn wrap_first_argument(path: &mut syn::Path, outer: &str) {
    let Some(seg) = path.segments.last_mut() else {
        return;
    };
    if seg.ident != outer {
        return;
    }
    if let syn::PathArguments::AngleBracketed(args) = &mut seg.arguments {
        if let Some(syn::GenericArgument::Type(t)) = args.args.first_mut() {
            *t = wrap_in_option(t);
        }
    }
}

/// Follow each rewritten field into `mod builder`: the `Result<T, String>`
/// the builder holds and the `TryInto<T>` its setter takes.
fn follow_into_builder(items: &mut [syn::Item], rewrites: &Rewrites) {
    for item in items.iter_mut() {
        match item {
            syn::Item::Struct(s) => {
                let owner = s.ident.to_string();
                let syn::Fields::Named(fields) = &mut s.fields else {
                    continue;
                };
                for field in fields.named.iter_mut() {
                    let Some(ident) = &field.ident else { continue };
                    if rewrites.contains(&(owner.clone(), ident.to_string())) {
                        if let syn::Type::Path(p) = &mut field.ty {
                            wrap_first_argument(&mut p.path, "Result");
                        }
                    }
                }
            }
            syn::Item::Impl(imp) if imp.trait_.is_none() => {
                let Some(owner) = last_segment(&imp.self_ty).map(|s| s.ident.to_string()) else {
                    continue;
                };
                for impl_item in imp.items.iter_mut() {
                    let syn::ImplItem::Fn(f) = impl_item else {
                        continue;
                    };
                    if !rewrites.contains(&(owner.clone(), f.sig.ident.to_string())) {
                        continue;
                    }
                    let Some(wc) = &mut f.sig.generics.where_clause else {
                        continue;
                    };
                    for pred in wc.predicates.iter_mut() {
                        let syn::WherePredicate::Type(pt) = pred else {
                            continue;
                        };
                        for bound in pt.bounds.iter_mut() {
                            if let syn::TypeParamBound::Trait(tb) = bound {
                                wrap_first_argument(&mut tb.path, "TryInto");
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Fail if the marker survived anywhere in the rendered module.
pub(crate) fn assert_no_marker(source: &str, ctx: &str) -> Result<()> {
    if source.contains(NULLABLE_MARKER) {
        return Err(anyhow!(
            "{ctx}: the nullable-member tag survived into the generated module"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tagged(schema: Value) -> BTreeSet<String> {
        let mut s = schema;
        mark_nullable_optional_members(&mut s)
            .into_values()
            .map(|t| t.name)
            .collect()
    }

    #[test]
    fn tags_only_optional_members_that_admit_null() {
        let names = tagged(json!({
            "type": "object",
            "required": ["req"],
            "properties": {
                "req": {"type": ["string", "null"]},
                "union": {"type": ["string", "null"]},
                "branch": {"oneOf": [{"type": "null"}, {"$ref": "#/definitions/Thing"}]},
                "viaRef": {"$ref": "#/definitions/MaybeThing"},
                "constNull": {"enum": ["a", null]},
                "any": {"description": "anything"},
                "anyTrue": true,
                "unconstrained": {"minLength": 1},
                "notNull": {"not": {"type": "null"}},
                "notString": {"not": {"type": "string"}},
                "refAndType": {"$ref": "#/definitions/MaybeThing", "type": "object"},
                "nullOnly": {"type": "null"},
                "plain": {"type": "string"},
                "list": {"type": "array", "items": {"type": "string"}},
                "thing": {"$ref": "#/definitions/Thing"},
                "allOfOneRefuses": {"allOf": [{"type": ["string", "null"]}, {"type": "string"}]},
                "allOfBothAdmit": {"allOf": [{"type": ["string", "null"]}, {"maxLength": 3}]},
                "constString": {"const": "a"},
                "enumNoNull": {"enum": ["a", "b"]},
                "typeArrayNoNull": {"type": ["string", "integer"]},
                "oneOfNoNull": {"oneOf": [{"type": "string"}, {"type": "integer"}]},
                "refCycle": {"$ref": "#/definitions/Loop"}
            },
            "definitions": {
                "Thing": {"type": "object", "properties": {"inner": {"type": ["integer", "null"]}}},
                "MaybeThing": {"anyOf": [{"type": "null"}, {"$ref": "#/definitions/Thing"}]},
                "Loop": {"$ref": "#/definitions/Loop"}
            }
        }));
        // `unconstrained` and `notString` cannot be shown to refuse `null`, so
        // they keep it; `refAndType` must satisfy both, and `object` refuses it.
        // `refCycle` never resolves within the depth bound, which keeps `null`.
        let want: BTreeSet<String> = [
            "allOfBothAdmit",
            "refCycle",
            "union",
            "branch",
            "viaRef",
            "constNull",
            "any",
            "anyTrue",
            "unconstrained",
            "notString",
            "inner",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        assert_eq!(names, want);
    }

    #[test]
    fn each_tag_has_its_own_id_and_strips_cleanly() {
        let mut schema = json!({
            "type": "object",
            "properties": {
                "a": {"type": ["string", "null"], "description": "First."},
                "b": {"type": "object", "properties": {"a": {"type": ["integer", "null"]}}}
            }
        });
        let tags = mark_nullable_optional_members(&mut schema);
        assert_eq!(tags.len(), 2);
        assert!(tags.values().all(|t| t.name == "a" && !t.in_branch));
        let first = schema["properties"]["a"]["description"].as_str().unwrap();
        let (text, ids) = strip_tags(first);
        assert_eq!(text, "First.");
        assert_eq!(ids.len(), 1);
        let nested = schema["properties"]["b"]["properties"]["a"]["description"]
            .as_str()
            .unwrap();
        let (text, other) = strip_tags(nested);
        assert_eq!(text, "");
        assert_ne!(ids, other);
        // Something that only looks like a tag is left for `assert_no_marker`.
        let (text, ids) = strip_tags(&format!("x {NULLABLE_MARKER}y]]"));
        assert!(ids.is_empty() && text.contains(NULLABLE_MARKER));
    }

    #[test]
    fn records_which_tags_sit_under_a_branch() {
        let mut schema = json!({
            "type": "object",
            "properties": {
                "top": {"type": ["string", "null"]},
                "nested": {"type": "object", "properties": {"deep": {"type": ["string", "null"]}}}
            },
            "definitions": {
                "Branched": {
                    "oneOf": [
                        {"type": "object", "properties": {"inBranch": {"type": ["string", "null"]}}}
                    ]
                },
                "Merged": {
                    "type": "object",
                    "properties": {"merged": {"type": ["string", "null"]}},
                    "anyOf": [{"required": ["merged"]}, {"required": ["other"]}]
                },
                "ViaAllOf": {
                    "allOf": [
                        {"type": "object", "properties": {"viaAllOf": {"type": ["string", "null"]}}},
                        {"oneOf": [{"required": ["viaAllOf"]}, {"required": ["other"]}]}
                    ]
                }
            }
        });
        let tags = mark_nullable_optional_members(&mut schema);
        let got: BTreeMap<String, bool> =
            tags.into_values().map(|t| (t.name, t.in_branch)).collect();
        let want: BTreeMap<String, bool> = [
            ("top", false),
            ("deep", false),
            ("inBranch", true),
            ("merged", true),
            ("viaAllOf", true),
        ]
        .into_iter()
        .map(|(n, b)| (n.to_string(), b))
        .collect();
        assert_eq!(got, want);
    }

    fn tag(name: &str, in_branch: bool) -> Tag {
        Tag {
            name: name.to_string(),
            in_branch,
        }
    }

    fn run(mut file: syn::File, tags: &[(usize, Tag)]) -> Result<syn::File> {
        let tags: NullableTags = tags.iter().cloned().collect();
        let serde: TokenStream = quote::quote!(::serde);
        apply_presence(&mut file, &serde, None, &tags).map(|()| file)
    }

    fn run_err(file: syn::File, tags: &[(usize, Tag)]) -> String {
        match run(file, tags) {
            Ok(_) => panic!("expected generation to fail"),
            Err(e) => e.to_string(),
        }
    }

    /// An optional struct member as typify renders it, with no doc comment:
    /// its tag, if it had one, was dropped.
    fn struct_with_untagged_label() -> syn::File {
        syn::parse_quote! {
            pub struct Payload {
                #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
                pub label: ::std::option::Option<::std::string::String>,
            }
        }
    }

    #[test]
    fn an_unread_tag_fails_naming_the_member_and_tag() {
        let err = run_err(struct_with_untagged_label(), &[(0, tag("label", false))]);
        assert!(err.contains("\"label\" (tag 0)"), "{err}");
        assert!(
            err.contains("`$defs`"),
            "the error suggests a remedy: {err}"
        );
    }

    #[test]
    fn a_tag_read_back_rewrites_the_field() {
        let doc = format!("A label.\n\n{}", marker(0));
        let file: syn::File = syn::parse_quote! {
            pub struct Payload {
                #[doc = #doc]
                #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
                pub label: ::std::option::Option<::std::string::String>,
            }
        };
        let out = run(file, &[(0, tag("label", false))]).unwrap();
        let text = quote::quote!(#out).to_string();
        assert!(text.contains("\"presence::present\""), "{text}");
        assert!(!text.contains(NULLABLE_MARKER), "{text}");
    }

    #[test]
    fn a_required_variant_field_accounts_for_a_branch_tag() {
        // typify writes no doc on a variant's fields; a required one (no
        // `#[serde]`, or a `rename` only) accounts for a branch tag.
        let file: syn::File = syn::parse_quote! {
            pub enum Body {
                Plain {
                    label: ::std::option::Option<::std::string::String>,
                },
                Renamed {
                    #[serde(rename = "theLabel")]
                    the_label: ::std::option::Option<::std::string::String>,
                },
            }
        };
        run(file, &[(0, tag("label", true)), (1, tag("theLabel", true))]).unwrap();
    }

    #[test]
    fn a_variant_field_does_not_mask_an_unread_struct_tag() {
        // The struct's `label` lost its tag; a required `label` in an
        // unrelated enum must not account for it.
        let mut file = struct_with_untagged_label();
        let other: syn::File = syn::parse_quote! {
            pub enum Other {
                Only {
                    label: ::std::option::Option<::std::string::String>,
                },
            }
        };
        file.items.extend(other.items);
        let err = run_err(file, &[(0, tag("label", false)), (1, tag("label", true))]);
        assert!(err.contains("\"label\" (tag 0)"), "{err}");
        assert!(!err.contains("tag 1"), "{err}");
    }

    #[test]
    fn an_optional_variant_field_of_a_branch_tagged_name_is_ambiguous() {
        let as_vec: syn::File = syn::parse_quote! {
            pub enum Body {
                Listed {
                    #[serde(default, skip_serializing_if = "::std::vec::Vec::is_empty")]
                    label: ::std::vec::Vec<::std::string::String>,
                },
            }
        };
        let as_option: syn::File = syn::parse_quote! {
            pub enum Body {
                Maybe {
                    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
                    label: ::std::option::Option<::std::string::String>,
                },
            }
        };
        for file in [as_vec, as_option] {
            let err = run_err(file, &[(0, tag("label", true))]);
            assert!(
                err.contains("\"label\" of an enum variant") && err.contains("cannot be read back"),
                "{err}"
            );
        }
    }

    #[test]
    fn a_struct_tag_does_not_make_a_variant_field_ambiguous() {
        // Outside a branch, a tag cannot be on a variant's field: the optional
        // variant field of that name is an unrelated member, left as typify
        // renders it (its `null` reads as absent).
        let doc = marker(0);
        let file: syn::File = syn::parse_quote! {
            pub struct Payload {
                #[doc = #doc]
                #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
                pub label: ::std::option::Option<::std::string::String>,
            }
            pub enum Body {
                Maybe {
                    #[serde(default, skip_serializing_if = "::std::option::Option::is_none")]
                    label: ::std::option::Option<::std::string::String>,
                },
            }
        };
        let out = run(file, &[(0, tag("label", false))]).unwrap();
        let text = quote::quote!(#out).to_string();
        assert!(text.contains("\"presence::present\""), "{text}");
        assert_eq!(text.matches("deserialize_with").count(), 1, "{text}");
    }

    #[test]
    fn a_bare_collection_error_names_the_owning_struct() {
        let doc = marker(0);
        let file: syn::File = syn::parse_quote! {
            pub struct Payload {
                #[doc = #doc]
                #[serde(default, skip_serializing_if = "::std::collections::HashMap::is_empty")]
                pub meta: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
            }
        };
        let err = run_err(file, &[(0, tag("meta", false))]);
        assert!(err.contains("\"meta\" of `Payload`"), "{err}");
    }
}
