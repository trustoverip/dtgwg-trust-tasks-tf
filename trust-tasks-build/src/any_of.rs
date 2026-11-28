//! An `anyOf` of object branches reads each branch from its own copy of the
//! value.
//!
//! typify renders `{"anyOf": [A, B]}` over objects as a struct with one
//! optional flattened field per branch:
//!
//! ```text
//! #[derive(Deserialize, Serialize)]
//! pub struct X {
//!     #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
//!     pub subtype_0: Option<A>,
//!     #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
//!     pub subtype_1: Option<B>,
//! }
//! ```
//!
//! serde's derive reads flattened fields one after another out of a single
//! buffer of the object's entries, and a flattened struct takes each entry it
//! recognises out of that buffer *before* it knows whether it will succeed
//! (`FlatStructAccess` in serde's private `de` module). A failed `Option<A>`
//! reads as `None`, but the members `A` shares with `B` are gone, so `B`
//! fails too. For `hcp/entries/create` 0.1's `entry` (`anyOf` Disclosure |
//! Entry, which share `content`, `assertion`, `classification`, `evidence`),
//! a whole Entry read as `X { subtype_0: None, subtype_1: None }` and wrote
//! back as `{}`; and a value matching no branch read as the same empty struct
//! instead of failing.
//!
//! This pass keeps the struct and its fields exactly as typify writes them
//! (so `Serialize`, the builder and every consumer-visible name are
//! unchanged) and replaces only the derived `Deserialize` with one that reads
//! the value once into a `serde_json::Value` and each branch from its own
//! clone of it, then refuses the value if no branch read it, as `anyOf`
//! requires. A value two branches read keeps both, as before.
//!
//! Only a struct whose every field is such a branch is rewritten: that is the
//! shape typify gives an `anyOf` and nothing else. The TypeScript, Go and Dart
//! generators do not have the problem (a union type, `json.RawMessage` and
//! `Object?` respectively, none of which reads the value apart) and are not
//! mirrored.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;

/// Rewrite every top-level `anyOf` struct's `Deserialize` (see the module
/// doc). `serde` and `serde_json` are the paths generated code names those
/// crates by. Returns the names of the structs rewritten.
pub(crate) fn apply_any_of_deserialize(
    file: &mut syn::File,
    serde: &TokenStream,
    serde_json: &TokenStream,
) -> Vec<String> {
    let mut impls: Vec<(usize, syn::Item)> = Vec::new();
    let mut rewritten = Vec::new();
    for (at, item) in file.items.iter_mut().enumerate() {
        let syn::Item::Struct(s) = item else { continue };
        let Some(fields) = any_of_branches(s) else {
            continue;
        };
        if !drop_derived_deserialize(&mut s.attrs) {
            continue;
        }
        let name = &s.ident;
        let message = format!("data did not match any anyOf branch of {name}");
        let none = fields
            .iter()
            .map(|f| quote!(out.#f.is_none()))
            .reduce(|a, b| quote!(#a && #b))
            .expect("an anyOf struct has a field");
        impls.push((at + 1, syn::parse_quote! {
            impl<'de> #serde::Deserialize<'de> for #name {
                fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                where
                    D: #serde::Deserializer<'de>,
                {
                    let value = <#serde_json::Value as #serde::Deserialize>::deserialize(deserializer)?;
                    let out = Self {
                        #( #fields: #serde_json::from_value(value.clone()).ok(), )*
                    };
                    if #none {
                        return ::std::result::Result::Err(
                            <D::Error as #serde::de::Error>::custom(#message),
                        );
                    }
                    ::std::result::Result::Ok(out)
                }
            }
        }));
        rewritten.push(name.to_string());
    }
    // Each right after its struct; from the back, so the indices hold.
    for (at, imp) in impls.into_iter().rev() {
        file.items.insert(at, imp);
    }
    rewritten
}

/// The field names, when `s` is typify's rendering of an `anyOf`: named
/// fields `subtype_0`, `subtype_1`, ..., each an `Option` with
/// `#[serde(flatten)]`.
fn any_of_branches(s: &syn::ItemStruct) -> Option<Vec<syn::Ident>> {
    let syn::Fields::Named(named) = &s.fields else {
        return None;
    };
    if named.named.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    for (i, field) in named.named.iter().enumerate() {
        let ident = field.ident.as_ref()?;
        let is_option = matches!(&field.ty, syn::Type::Path(p)
            if p.qself.is_none() && p.path.segments.last().is_some_and(|s| s.ident == "Option"));
        if *ident != format_ident!("subtype_{i}") || !is_option || !is_flattened(&field.attrs) {
            return None;
        }
        out.push(ident.clone());
    }
    Some(out)
}

fn is_flattened(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("serde")
            && attr
                .parse_args_with(Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)
                .is_ok_and(|metas| metas.iter().any(|m| m.path().is_ident("flatten")))
    })
}

/// Remove `Deserialize` from the struct's `#[derive(...)]`; whether it was
/// there.
fn drop_derived_deserialize(attrs: &mut [syn::Attribute]) -> bool {
    for attr in attrs.iter_mut() {
        if !attr.path().is_ident("derive") {
            continue;
        }
        let Ok(paths) =
            attr.parse_args_with(Punctuated::<syn::Path, syn::Token![,]>::parse_terminated)
        else {
            continue;
        };
        let kept: Punctuated<syn::Path, syn::Token![,]> = paths
            .iter()
            .filter(|p| !p.segments.last().is_some_and(|s| s.ident == "Deserialize"))
            .cloned()
            .collect();
        if kept.len() != paths.len() {
            *attr = syn::parse_quote!(#[derive(#kept)]);
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str) -> (String, Vec<String>) {
        let mut file: syn::File = syn::parse_str(src).unwrap();
        let names = apply_any_of_deserialize(&mut file, &quote!(::serde), &quote!(::serde_json));
        (prettyplease::unparse(&file), names)
    }

    #[test]
    fn an_any_of_struct_gets_a_branch_by_branch_deserialize() {
        let (out, names) = run(r#"
            #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
            pub struct X {
                #[serde(flatten, default, skip_serializing_if = "::std::option::Option::is_none")]
                pub subtype_0: ::std::option::Option<A>,
                #[serde(flatten, default, skip_serializing_if = "::std::option::Option::is_none")]
                pub subtype_1: ::std::option::Option<B>,
            }
        "#);
        assert_eq!(names, ["X"]);
        assert!(
            out.contains("#[derive(::serde::Serialize, Clone, Debug)]"),
            "{out}"
        );
        assert!(
            out.contains("impl<'de> ::serde::Deserialize<'de> for X"),
            "{out}"
        );
        assert!(out.contains("subtype_1: ::serde_json::from_value(value.clone()).ok()"));
        assert!(out.contains("data did not match any anyOf branch of X"));
    }

    #[test]
    fn other_structs_are_left_alone() {
        for src in [
            // An ordinary optional member.
            r#"#[derive(::serde::Deserialize)] pub struct Y {
                #[serde(default)] pub subtype_0: ::std::option::Option<A>,
            }"#,
            // A mix of a branch and a member of its own.
            r#"#[derive(::serde::Deserialize)] pub struct Y {
                #[serde(flatten)] pub subtype_0: ::std::option::Option<A>,
                pub other: u8,
            }"#,
            // A flattened required member (an `allOf` rendering).
            r#"#[derive(::serde::Deserialize)] pub struct Y {
                #[serde(flatten)] pub subtype_0: A,
            }"#,
        ] {
            let (out, names) = run(src);
            assert!(names.is_empty(), "{out}");
            assert!(out.contains("derive(::serde::Deserialize)"), "{out}");
        }
    }
}
