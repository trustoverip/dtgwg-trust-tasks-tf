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
//!   there as `None`, so a `null` the schema refuses is accepted and written
//!   back as absent.
//!
//! This module rewrites those fields so the three states stay apart, and a
//! `null` is accepted exactly where the schema admits one:
//!
//! | schema of an optional member | field type | absent | `null` | value |
//! |---|---|---|---|---|
//! | array / map | `Option<Vec<T>>` / `Option<Map>` | `None` | refused | `Some(v)` (`[]` too) |
//! | admits `null` | `Option<Option<T>>` | `None` | `Some(None)` | `Some(Some(v))` |
//! | `{"type": "null"}` | `Option<()>` | `None` | `Some(())` | — |
//! | anything else | `Option<T>` (unchanged) | `None` | refused | `Some(v)` |
//!
//! Every optional field reads through one of two helpers (`deserialize_with`)
//! and keeps `skip_serializing_if = "Option::is_none"`, so only an absent
//! member is omitted. `present` maps a present member, `null` included, to
//! `Some` (the member admits `null`); `not_null` refuses a `null` (it does
//! not). For `Target::runtime_crate()` the
//! helpers are the crate-private `crate::specs::presence` in `trust-tasks-rs`,
//! which adds no public API. For an application they are emitted into each
//! module that needs them, so a generated module names nothing new in the
//! runtime's `__private` surface (and so no new ABI item).
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
//! A member with a declared JSON Schema `default` is left as typify renders it
//! (`#[serde(default = "…")]`), so an absent member reads as that default.
//! typify treats a default equal to the type's intrinsic one (`[]`, `{}`,
//! `null`, `false`, `0`, `""`) as no default at all, so such a member is
//! rewritten like any other optional member, and an absent one reads as
//! `None`.

use std::collections::BTreeSet;

use anyhow::{anyhow, Result};
use proc_macro2::TokenStream;
use serde_json::Value;
use syn::visit_mut::VisitMut;

/// Appended to the `description` of a nullable optional member in the schema
/// typify sees. Never in the output: [`apply_presence`] removes every
/// occurrence and [`assert_no_marker`] fails generation if one survives.
pub(crate) const NULLABLE_MARKER: &str = "[[trust-tasks-build:nullable-optional-member]]";

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
/// local references read `#/definitions/…`. Returns the JSON names tagged.
pub(crate) fn mark_nullable_optional_members(schema: &mut Value) -> BTreeSet<String> {
    let root = schema.clone();
    let mut tagged = BTreeSet::new();
    mark(schema, &root, &mut tagged);
    tagged
}

fn mark(node: &mut Value, root: &Value, tagged: &mut BTreeSet<String>) {
    match node {
        Value::Object(map) => {
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
                    let description = match p.get("description").and_then(Value::as_str) {
                        Some(d) => format!("{d}\n\n{NULLABLE_MARKER}"),
                        None => NULLABLE_MARKER.to_string(),
                    };
                    p.insert("description".into(), Value::String(description));
                    tagged.insert(name.clone());
                }
            }
            for value in map.values_mut() {
                mark(value, root, tagged);
            }
        }
        Value::Array(items) => {
            for item in items {
                mark(item, root, tagged);
            }
        }
        _ => {}
    }
}

fn is_null_only(p: &serde_json::Map<String, Value>) -> bool {
    p.get("type").and_then(Value::as_str) == Some("null")
}

/// Whether `schema` accepts JSON `null`. `$ref`s into the document are
/// followed; `depth` bounds a reference cycle.
///
/// A schema admits `null` only when every keyword that can restrict a value's
/// type does: `type`, `const`, `enum`, `oneOf` / `anyOf` (some branch),
/// `allOf` (every branch), `$ref`, and a `not` of a schema that is exactly
/// `null`. Any other keyword (`not` of anything else, `if`, a type-specific
/// keyword such as `minLength`) is taken to admit it. The answer drives
/// whether a `null` is kept or refused, and erring towards "admits" only keeps
/// a `null` the schema might refuse, where erring the other way would refuse a
/// document the schema accepts.
fn admits_null(schema: &Value, root: &Value, depth: usize) -> bool {
    if depth > 64 {
        return false;
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
    if map.get("not").is_some_and(is_exactly_null) {
        return false;
    }
    true
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
/// struct's builder, and emit the helpers the fields use. `serde` is the path
/// generated code names serde by. `shared` is the path of a module that
/// already defines the helpers (`crate::specs::presence` inside
/// `trust-tasks-rs`), or `None` to emit them into this module. `tagged` is
/// what [`mark_nullable_optional_members`] returned. A tag goes unread where
/// typify made the member required (a `oneOf` branch requiring it), which
/// needs no rewrite, or put it in an enum variant, whose fields carry no docs;
/// that case is refused when the variant's field could be either.
pub(crate) fn apply_presence(
    file: &mut syn::File,
    serde: &TokenStream,
    shared: Option<&str>,
    tagged: &BTreeSet<String>,
) -> Result<()> {
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
        // cannot come back there; refuse rather than guess from the name.
        return Err(anyhow!(
            "optional member {name:?} of an enum variant: a member of the same name elsewhere \
             in the schema admits null, and typify writes no doc on a variant's fields, so \
             whether this one admits null cannot be read back; no field rewrite can be chosen \
             safely (rename one of the members, or make the variant a `$defs` type)"
        ));
    }
    // Every doc comment, not only the fields': typify also copies a member's
    // description onto the newtype it makes for a constrained member.
    StripMarker.visit_file_mut(file);

    if rewriter.rewrites.is_empty() && !rewriter.used_present && !rewriter.used_not_null {
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
    // Only the helpers this module uses, so neither is dead code.
    let mut fns: Vec<syn::Item> = Vec::new();
    if rewriter.used_present {
        fns.push(syn::parse_quote! {
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
        });
    }
    if rewriter.used_not_null {
        fns.push(syn::parse_quote! {
            /// Reads a present member as `Some` and refuses a `null`: the
            /// member's schema has no null form, so `null` is not "absent".
            pub(super) fn not_null<'de, D, T>(
                deserializer: D,
            ) -> ::std::result::Result<::std::option::Option<T>, D::Error>
            where
                D: #serde::Deserializer<'de>,
                T: #serde::Deserialize<'de>,
            {
                match <::std::option::Option<T> as #serde::Deserialize<'de>>::deserialize(
                    deserializer,
                )? {
                    ::std::option::Option::Some(value) => {
                        ::std::result::Result::Ok(::std::option::Option::Some(value))
                    }
                    ::std::option::Option::None => ::std::result::Result::Err(
                        <D::Error as #serde::de::Error>::invalid_type(
                            #serde::de::Unexpected::Unit,
                            &"a value (this optional member has no null form: leave it out instead)",
                        ),
                    ),
                }
            }
        });
    }
    let helper: syn::Item = syn::parse_quote! {
        #[doc(hidden)]
        mod presence {
            #(#fns)*
        }
    };
    file.items.push(helper);
    Ok(())
}

#[derive(Default)]
struct Rewriter {
    /// Path the fields name the helpers by: `presence` (emitted into the
    /// module) or a shared module.
    helpers: String,
    rewrites: Rewrites,
    /// Optional enum-variant fields whose JSON name was tagged somewhere in
    /// the schema: their own nullability cannot be read back.
    ambiguous: Vec<String>,
    used_present: bool,
    used_not_null: bool,
}

impl Rewriter {
    fn field(&mut self, owner: Option<&str>, field: &mut syn::Field, tagged: &BTreeSet<String>) {
        let Some(serde) = SerdeAttr::read(&field.attrs) else {
            return;
        };
        let json_name = serde.rename.clone().unwrap_or_else(|| {
            field
                .ident
                .as_ref()
                .map(|i| i.to_string().trim_start_matches("r#").to_string())
                .unwrap_or_default()
        });
        let has_tag = take_marker(&mut field.attrs);
        let ty = &field.ty;
        // Optional as typify rendered it: `Option<T>`, `default`, skipped
        // when `None`. A tagged member typify made required (a `oneOf` branch
        // that requires it) is already written whenever it is present.
        let optional_option = serde.default && serde.skip_if_none() && is_option(ty);
        // (wrap the type in one more `Option`, the schema admits `null`)
        let (wrap, nullable) = if optional_option {
            if has_tag {
                (true, true)
            } else if owner.is_none() && tagged.contains(&json_name) {
                self.ambiguous.push(json_name);
                return;
            } else {
                // Not nullable: the type stays, a `null` is refused.
                (false, false)
            }
        } else if serde.default && serde.skip_if_empty() {
            (true, false)
        } else if serde.default && serde.skip.is_none() && is_unit(ty) {
            (true, true)
        } else {
            return;
        };
        let helper = if nullable {
            self.used_present = true;
            "present"
        } else {
            self.used_not_null = true;
            "not_null"
        };
        serde.rewrite(&mut field.attrs, &format!("{}::{helper}", self.helpers));
        if !wrap {
            return;
        }
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
    fn rewrite(&self, attrs: &mut [syn::Attribute], helper: &str) {
        let attr: syn::Attribute = match &self.rename {
            Some(rename) => syn::parse_quote! {
                #[serde(
                    rename = #rename,
                    default,
                    deserialize_with = #helper,
                    skip_serializing_if = "::std::option::Option::is_none"
                )]
            },
            None => syn::parse_quote! {
                #[serde(
                    default,
                    deserialize_with = #helper,
                    skip_serializing_if = "::std::option::Option::is_none"
                )]
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

/// Remove the marker from a field's doc comment; whether it was there.
fn take_marker(attrs: &mut Vec<syn::Attribute>) -> bool {
    let mut found = false;
    attrs.retain_mut(|attr| match strip_doc(attr) {
        DocStrip::Untouched => true,
        DocStrip::Stripped { now_empty } => {
            found = true;
            !now_empty
        }
    });
    found
}

enum DocStrip {
    Untouched,
    Stripped { now_empty: bool },
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
    let value = text.value();
    if !value.contains(NULLABLE_MARKER) {
        return DocStrip::Untouched;
    }
    let stripped = value
        .replace(&format!("\n\n{NULLABLE_MARKER}"), "")
        .replace(NULLABLE_MARKER, "");
    let now_empty = stripped.trim().is_empty();
    *text = syn::LitStr::new(&stripped, text.span());
    DocStrip::Stripped { now_empty }
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
            if let DocStrip::Stripped { now_empty: true } = strip_doc(attr) {
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
                "thing": {"$ref": "#/definitions/Thing"}
            },
            "definitions": {
                "Thing": {"type": "object", "properties": {"inner": {"type": ["integer", "null"]}}},
                "MaybeThing": {"anyOf": [{"type": "null"}, {"$ref": "#/definitions/Thing"}]}
            }
        }));
        // `unconstrained` and `notString` cannot be shown to refuse `null`, so
        // they keep it; `refAndType` must satisfy both, and `object` refuses it.
        let want: BTreeSet<String> = [
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
}
