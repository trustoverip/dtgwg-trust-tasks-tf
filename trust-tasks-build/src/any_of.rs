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
//! the value once into a `serde_json::Value` and tries the branches in order,
//! each on its own copy of it: the first branch that reads is kept and the
//! rest stay `None`, and a value no branch reads is refused with each
//! branch's error, as `anyOf` requires. First match wins, so at most one
//! field is ever `Some` and `Serialize` can never write a member twice; a
//! value two branches would both read keeps only the first (the derived impl
//! did the same for such a value, since the first branch took the shared
//! members). The second branch's own members are then dropped on re-encode,
//! as typify's open objects drop undeclared members anyway.
//!
//! Only a struct whose every field is such a branch, and which derives both
//! `Deserialize` and `Serialize`, is rewritten: that is the shape typify
//! gives an `anyOf` and nothing else. Any other struct that derives
//! `Deserialize` with a flattened `Option` field (typify's rendering of an
//! `anyOf` beside members of its own, say) would lose data the same way, so
//! the pass refuses it instead of leaving it as it is. The TypeScript, Go and
//! Dart generators do not have the problem (a union type, `json.RawMessage`
//! and `Object?` respectively, none of which reads the value apart) and are
//! not mirrored.

use anyhow::{anyhow, Result};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::visit_mut::VisitMut;

/// Rewrite every top-level `anyOf` struct's `Deserialize` (see the module
/// doc). `serde` and `serde_json` are the paths generated code names those
/// crates by. Returns the names of the structs rewritten, or an error naming
/// every other struct that still derives `Deserialize` over a flattened
/// `Option` field.
pub(crate) fn apply_any_of_deserialize(
    file: &mut syn::File,
    serde: &TokenStream,
    serde_json: &TokenStream,
) -> Result<Vec<String>> {
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
        let errors: Vec<syn::Ident> = (0..fields.len()).map(|i| format_ident!("e{i}")).collect();
        let message = format!(
            "data did not match any anyOf branch of {name}: {}",
            fields
                .iter()
                .zip(&errors)
                .map(|(f, e)| format!("{f}: {{{e}}}"))
                .collect::<Vec<_>>()
                .join("; ")
        );
        // One attempt per branch, in order; the first that reads returns
        // with only its own field set. Every branch but the last reads a
        // clone; the last takes the value itself.
        let last = fields.len() - 1;
        let attempts = fields.iter().zip(&errors).enumerate().map(|(i, (f, e))| {
            let input = if i == last {
                quote!(value)
            } else {
                quote!(value.clone())
            };
            let set = fields.iter().map(|o| {
                if o == f {
                    quote!(#o: ::std::option::Option::Some(branch))
                } else {
                    quote!(#o: ::std::option::Option::None)
                }
            });
            quote! {
                let #e = match #serde_json::from_value(#input) {
                    ::std::result::Result::Ok(branch) => {
                        return ::std::result::Result::Ok(Self { #( #set ),* });
                    }
                    ::std::result::Result::Err(e) => e,
                };
            }
        });
        impls.push((at + 1, syn::parse_quote! {
            impl<'de> #serde::Deserialize<'de> for #name {
                fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                where
                    D: #serde::Deserializer<'de>,
                {
                    let value = <#serde_json::Value as #serde::Deserialize>::deserialize(deserializer)?;
                    #( #attempts )*
                    ::std::result::Result::Err(<D::Error as #serde::de::Error>::custom(
                        ::std::format!(#message),
                    ))
                }
            }
        }));
        rewritten.push(name.to_string());
    }
    // Each right after its struct; from the back, so the indices hold.
    for (at, imp) in impls.into_iter().rev() {
        file.items.insert(at, imp);
    }

    // Anything left that derives `Deserialize` over a flattened `Option`
    // would read the way the anyOf struct did before this pass.
    let mut missed = Missed::default();
    missed.visit_file_mut(file);
    if !missed.0.is_empty() {
        return Err(anyhow!(
            "struct(s) {} derive Deserialize over a flattened Option field, a shape the anyOf \
             pass does not rewrite (see trust-tasks-build/src/any_of.rs): a branch that fails \
             would take the members it shares with the next and lose data",
            missed.0.join(", ")
        ));
    }
    Ok(rewritten)
}

/// Every struct, at any depth, that still derives `Deserialize` and has a
/// `#[serde(flatten)]` field of type `Option<_>`.
#[derive(Default)]
struct Missed(Vec<String>);

impl VisitMut for Missed {
    fn visit_item_struct_mut(&mut self, s: &mut syn::ItemStruct) {
        if derives(&s.attrs, "Deserialize")
            && s.fields
                .iter()
                .any(|f| is_option(&f.ty) && is_flattened(&f.attrs))
        {
            self.0.push(s.ident.to_string());
        }
        syn::visit_mut::visit_item_struct_mut(self, s);
    }
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
        if *ident != format_ident!("subtype_{i}")
            || !is_option(&field.ty)
            || !is_flattened(&field.attrs)
        {
            return None;
        }
        out.push(ident.clone());
    }
    Some(out)
}

fn is_option(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(p)
        if p.qself.is_none() && p.path.segments.last().is_some_and(|s| s.ident == "Option"))
}

fn is_flattened(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("serde")
            && attr
                .parse_args_with(Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated)
                .is_ok_and(|metas| metas.iter().any(|m| m.path().is_ident("flatten")))
    })
}

/// The paths in the struct's `#[derive(...)]` lists.
fn derived(attrs: &[syn::Attribute]) -> Vec<syn::Path> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("derive"))
        .filter_map(|attr| {
            attr.parse_args_with(Punctuated::<syn::Path, syn::Token![,]>::parse_terminated)
                .ok()
        })
        .flatten()
        .collect()
}

/// Whether the struct derives a trait whose path ends in `name`.
fn derives(attrs: &[syn::Attribute], name: &str) -> bool {
    derived(attrs)
        .iter()
        .any(|p| p.segments.last().is_some_and(|s| s.ident == name))
}

/// Remove `Deserialize` from the struct's `#[derive(...)]`; whether it did.
/// Only when `Serialize` stays derived: the fields' `#[serde(flatten, ...)]`
/// attributes need a serde derive to be accepted, so a struct deriving
/// `Deserialize` alone is left as it is (and then refused, see
/// [`apply_any_of_deserialize`]).
fn drop_derived_deserialize(attrs: &mut [syn::Attribute]) -> bool {
    if !derives(attrs, "Serialize") {
        return false;
    }
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

    fn run(src: &str) -> (String, Result<Vec<String>>) {
        let mut file: syn::File = syn::parse_str(src).unwrap();
        let names = apply_any_of_deserialize(&mut file, &quote!(::serde), &quote!(::serde_json));
        (prettyplease::unparse(&file), names)
    }

    #[test]
    fn an_any_of_struct_gets_a_first_match_deserialize() {
        let (out, names) = run(r#"
            #[derive(::serde::Deserialize, ::serde::Serialize, Clone, Debug)]
            pub struct X {
                #[serde(flatten, default, skip_serializing_if = "::std::option::Option::is_none")]
                pub subtype_0: ::std::option::Option<A>,
                #[serde(flatten, default, skip_serializing_if = "::std::option::Option::is_none")]
                pub subtype_1: ::std::option::Option<B>,
            }
        "#);
        assert_eq!(names.unwrap(), ["X"]);
        let compact = out.split_whitespace().collect::<String>();
        assert!(
            out.contains("#[derive(::serde::Serialize, Clone, Debug)]"),
            "{out}"
        );
        assert!(
            out.contains("impl<'de> ::serde::Deserialize<'de> for X"),
            "{out}"
        );
        // The first branch reads a copy and returns alone; the last takes
        // the value itself.
        for expected in [
            "lete0=match::serde_json::from_value(value.clone()){",
            "Ok(Self{subtype_0:::std::option::Option::Some(branch),subtype_1:::std::option::Option::None",
            "lete1=match::serde_json::from_value(value){",
            "Ok(Self{subtype_0:::std::option::Option::None,subtype_1:::std::option::Option::Some(branch)",
            "::std::format!(\"datadidnotmatchanyanyOfbranchofX:subtype_0:{e0};subtype_1:{e1}\")",
        ] {
            assert!(compact.contains(expected), "{expected} in\n{out}");
        }
        assert_eq!(out.matches("value.clone()").count(), 1, "{out}");
    }

    #[test]
    fn other_structs_are_left_alone() {
        for src in [
            // An ordinary optional member.
            r#"#[derive(::serde::Deserialize)] pub struct Y {
                #[serde(default)] pub subtype_0: ::std::option::Option<A>,
            }"#,
            // A flattened required member (an `allOf` rendering).
            r#"#[derive(::serde::Deserialize)] pub struct Y {
                #[serde(flatten)] pub subtype_0: A,
            }"#,
            // A flattened map (`registry/*`'s `extra`).
            r#"#[derive(::serde::Deserialize, ::serde::Serialize)] pub struct Y {
                pub name: String,
                #[serde(flatten)] pub extra: ::std::collections::HashMap<String, V>,
            }"#,
            // The anyOf shape without a serde derive at all.
            r#"#[derive(Clone)] pub struct Y {
                pub subtype_0: ::std::option::Option<A>,
            }"#,
        ] {
            let (out, names) = run(src);
            assert!(names.unwrap().is_empty(), "{out}");
            assert!(!out.contains("impl"), "{out}");
        }
    }

    /// A flattened `Option` the pass does not rewrite would lose data the
    /// way the anyOf struct did, so `generate_module` fails on it.
    #[test]
    fn a_flattened_option_the_pass_does_not_rewrite_is_refused() {
        for src in [
            // A mix of a branch and a member of its own.
            r#"#[derive(::serde::Deserialize, ::serde::Serialize)] pub struct Y {
                #[serde(flatten)] pub subtype_0: ::std::option::Option<A>,
                pub other: u8,
            }"#,
            // The anyOf shape deriving `Deserialize` but not `Serialize`:
            // dropping the derive would leave its `#[serde]` field
            // attributes with no serde derive.
            r#"#[derive(::serde::Deserialize)] pub struct Y {
                #[serde(flatten)] pub subtype_0: ::std::option::Option<A>,
            }"#,
            // Inside a module.
            r#"pub mod m { #[derive(::serde::Deserialize)] pub struct Y {
                pub other: u8,
                #[serde(flatten, default)] pub rest: ::std::option::Option<A>,
            } }"#,
        ] {
            let (out, names) = run(src);
            let err = names.expect_err(&out).to_string();
            assert!(
                err.contains("Y derive Deserialize over a flattened Option"),
                "{err}"
            );
            assert!(out.contains("derive(::serde::Deserialize"), "{out}");
        }
    }
}
