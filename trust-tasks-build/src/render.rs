//! Rendering one specification version's Rust module: typify's output wrapped
//! in the `Payload` impls, policy constants, `EXTENDS`, `DEPRECATION` and
//! `ERROR_CODES` the runtime reads.

use proc_macro2::TokenStream;
use quote::quote;

use crate::bundle::Bundle;
use crate::error_codes::ErrorCodeDecl;
use crate::Target;

/// Make every multi-paragraph doc attribute in a generated module safe for
/// rustdoc.
///
/// A doc attribute whose text spans lines is printed as `/**{text}*/` — the
/// first line flush against the opening delimiter, every continuation line
/// carrying the indentation of whatever item it documents once rustfmt has run.
/// rustdoc strips only the whitespace *common* to all lines of a doc comment,
/// and the un-indented first line makes that common prefix empty. An item four
/// spaces deep therefore has its second and later paragraphs read as
/// **indented code blocks**, which rustdoc then compiles as Rust doctests and
/// fails on.
///
/// It bit `trust-task-next-step/0.1` and `trust-ceremony-receipt/0.1` through
/// multi-paragraph `description`s. It bit again through a titled inline
/// `oneOf` branch: typify documents an enum variant as `"{title}\n\n{description}"`
/// (`metadata_title_and_description`), so a one-paragraph description became
/// two paragraphs that no schema contained, and 77 doctests failed. A pass over
/// the schema's `description`s cannot see text typify assembles itself, so the
/// fix is applied to what typify emits instead.
///
/// The fix is to start such a doc on its own line. Every line then carries the
/// item's indentation, the common prefix is non-empty, rustdoc strips it, and
/// the paragraphs render as prose. Applied only where a blank line exists, so
/// single-paragraph docs are untouched.
pub(crate) fn indent_safe_docs(file: &mut syn::File) {
    use syn::visit_mut::VisitMut;

    struct IndentSafe;
    impl VisitMut for IndentSafe {
        fn visit_attribute_mut(&mut self, attr: &mut syn::Attribute) {
            if !attr.path().is_ident("doc") {
                return;
            }
            let syn::Meta::NameValue(nv) = &mut attr.meta else {
                return;
            };
            let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(text),
                ..
            }) = &mut nv.value
            else {
                return;
            };
            let value = text.value();
            if value.contains("\n\n") && !value.starts_with('\n') {
                *text = syn::LitStr::new(&format!("\n{value}"), text.span());
            }
        }
    }
    IndentSafe.visit_file_mut(file);
}

/// Mark every generated payload type `#[non_exhaustive]`.
///
/// This is the half of the ergonomics contract that the builder cannot
/// provide on its own. Without it a consumer keeps reaching for a struct
/// literal, and the next member added to a schema keeps breaking every one
/// of those literals — the tax this crate's CHANGELOG has recorded
/// repeatedly ("consumers constructing `AclEntry` with a struct literal must
/// add `step_up: None`"). With it, cross-crate construction has to go
/// through `X::builder()` or `X::default()`, both of which absorb a new
/// member silently, and a `match` on a generated enum has to carry a
/// wildcard arm, which absorbs a new enum value silently. `#[non_exhaustive]`
/// binds only *other* crates, so `trust-tasks-rs`' own code — including the
/// `builder` module's `TryFrom` impl and the generated conformance tests —
/// is unaffected.
///
/// Applied to:
///
/// - **named-field structs** (`Payload`, `Response`, and every `$defs`
///   object type) — the struct-literal case, and the one the builder
///   gives an alternative to;
/// - **enums** — matching the precedent set by `StandardCode` in 0.7.0, so
///   that a value added to a schema's `enum` is not a break either.
///
/// Deliberately *not* applied to tuple/newtype structs. typify emits those
/// for constrained scalars (`ExtKey`) and for map wrappers (`Ext(pub
/// HashMap<..>)`); it generates no builder for them, so marking them
/// `#[non_exhaustive]` would remove the only construction path a consumer
/// has and leave nothing in its place. Their shape is fixed by the scalar
/// or map they wrap, so there is no additive change for the attribute to
/// absorb.
///
/// Only top-level items are visited. The nested `error`, `builder` and
/// `defaults` modules typify emits are crate-internal plumbing whose types
/// are constructed by generated code, never by a consumer's struct literal.
pub(crate) fn apply_non_exhaustive(file: &mut syn::File) {
    let attr: syn::Attribute = syn::parse_quote!(#[non_exhaustive]);
    for item in file.items.iter_mut() {
        match item {
            syn::Item::Struct(s) if matches!(s.fields, syn::Fields::Named(_)) => {
                s.attrs.push(attr.clone());
            }
            syn::Item::Enum(e) => e.attrs.push(attr.clone()),
            _ => {}
        }
    }
}

/// Mark a deprecated version's root types — `Payload` and, when the spec has
/// one, `Response` — `#[deprecated(note = …)]`, so a consumer naming either is
/// warned and told the successor. Only the roots: they are what a consumer
/// names to send or receive the task, and every other type in the module is
/// reached through them.
pub(crate) fn apply_deprecation(file: &mut syn::File, note: &str) {
    let attr: syn::Attribute = syn::parse_quote!(#[deprecated(note = #note)]);
    let is_root = |ident: &syn::Ident| ident == "Payload" || ident == "Response";
    for item in file.items.iter_mut() {
        match item {
            syn::Item::Struct(s) if is_root(&s.ident) => s.attrs.push(attr.clone()),
            syn::Item::Enum(e) if is_root(&e.ident) => e.attrs.push(attr.clone()),
            syn::Item::Type(t) if is_root(&t.ident) => t.attrs.push(attr.clone()),
            _ => {}
        }
    }
}

/// Render one specification version's module around typify's `body`.
pub(crate) fn render_module(
    bundle: &Bundle,
    target: &Target,
    body: TokenStream,
    has_response: bool,
    schema_json: &str,
    response_schema_json: Option<&str>,
) -> TokenStream {
    let rt = target.runtime_tokens();
    let serde_path = target.dependency_tokens("serde");
    let generated_by_doc = format!(
        " Generated by `{}` — do not edit by hand.",
        target.generator
    );
    let extends_link_doc = format!(
        " [`{}::extension::extends_for`].",
        target.runtime.trim_start_matches("::")
    );
    let policy = &bundle.policy;
    let is_bearer = policy.bearer;
    let is_proof_required = policy.proof_required;
    let is_issued_at_required = policy.issued_at_required;
    let max_document_bytes = policy.max_document_bytes;
    let recipient_required = policy.recipient_required.request;
    // The response swaps the parties, so its `recipient` member tracks the
    // request's issuer party (SPEC §7.2 item 5).
    let issuer_required = policy.recipient_required.response;
    let error_codes = &bundle.error_codes[..];
    let extends = bundle.extends.as_deref();

    let type_uri = &bundle.type_uri;
    let response_uri = format!("{type_uri}#response");
    let slug_doc = format!(
        " Spec slug: `{}`. Version: `{}`.",
        bundle.slug, bundle.version
    );

    // Per SPEC §4.8.3, only bearer specs override Payload::IS_BEARER. Non-
    // bearer specs (the default) leave the trait's default `false` in place.
    let bearer_const = if is_bearer {
        quote! { const IS_BEARER: bool = true; }
    } else {
        quote! {}
    };

    // Per SPEC §7.3 item 8, only `REQUIRED` overrides
    // Payload::IS_PROOF_REQUIRED; `RECOMMENDED` / `OPTIONAL` (or absent) leave
    // the trait default `false` in place. The two variants are emitted
    // independently: a spec may require a proof on the response it returns
    // without requiring one on the request that triggered it, or the reverse.
    let req_proof_const = if is_proof_required.request {
        quote! { const IS_PROOF_REQUIRED: bool = true; }
    } else {
        quote! {}
    };
    let resp_proof_const = if is_proof_required.response {
        quote! { const IS_PROOF_REQUIRED: bool = true; }
    } else {
        quote! {}
    };

    // Per SPEC §7.3 item 17, only `REQUIRED` overrides
    // Payload::IS_ISSUED_AT_REQUIRED; anything else (or absent) leaves the
    // trait default `false` in place, which is §4.2's SHOULD. Emitted per
    // variant on the same reasoning as the proof consts above: a spec may
    // require a stamped instant on the request that causes the effect without
    // requiring one on the acknowledgement it sends back.
    let req_issued_at_const = if is_issued_at_required.request {
        quote! { const IS_ISSUED_AT_REQUIRED: bool = true; }
    } else {
        quote! {}
    };
    let resp_issued_at_const = if is_issued_at_required.response {
        quote! { const IS_ISSUED_AT_REQUIRED: bool = true; }
    } else {
        quote! {}
    };

    // Front matter `maxDocumentBytes` (SPEC §12.4's "body-size limit
    // appropriate to the Trust Task specification's payload", stated per
    // type). Only a declared variant overrides the trait default `None`.
    let size_const = |bound: Option<usize>| match bound {
        Some(n) => quote! { const MAX_DOCUMENT_BYTES: Option<usize> = Some(#n); },
        None => quote! {},
    };
    let req_size_const = size_const(max_document_bytes.request);
    let resp_size_const = size_const(max_document_bytes.response);

    // Per SPEC §7.2 item 5 / §7.3 item 5, a spec whose `recipient` party is
    // REQUIRED overrides Payload::IS_RECIPIENT_REQUIRED. The request's
    // `recipient` is the recipient party; the response swaps the parties, so its
    // `recipient` is the request's issuer — hence the response impl uses the
    // issuer-party requirement.
    let req_recipient_const = if recipient_required {
        quote! { const IS_RECIPIENT_REQUIRED: bool = true; }
    } else {
        quote! {}
    };
    let resp_recipient_const = if issuer_required {
        quote! { const IS_RECIPIENT_REQUIRED: bool = true; }
    } else {
        quote! {}
    };

    // SPEC §7.2 item 2 — the schema the payload is validated against, carried
    // on the type itself so a consumer never has to find it. Emitted
    // unconditionally: it is a `&'static str` and pulls in no dependency, and
    // gating it behind `validate` was what left the schema unreachable to
    // anyone who had not already opted in.
    let resp_schema_const = match response_schema_json {
        Some(json) => quote! { const PAYLOAD_SCHEMA: Option<&'static str> = Some(#json); },
        None => quote! {},
    };

    // SPEC §4.4.1 — the request payload names the response payload it is
    // answered with, so a transport can infer it. Emitted only for a
    // specification that actually defines a `$defs.Response`: a
    // fire-and-forget specification has no response *document*, and the
    // honest encoding of that is the absence of a `RequestPayload` impl
    // rather than a stand-in type. See the trait docs in
    // `trust-tasks-rs/src/payload.rs` for why neither `()` nor
    // `trust-task-ok` is used to fill the hole.
    let response_payload_impl = if has_response {
        quote! {
            impl #rt::Payload for Response {
                const TYPE_URI: &'static str = #response_uri;
                #bearer_const
                #resp_proof_const
                #resp_issued_at_const
                #resp_size_const
                #resp_recipient_const
                #resp_schema_const
            }

            impl #rt::RequestPayload for Payload {
                type Response = Response;
            }
        }
    } else {
        quote! {}
    };

    let error_codes_items = render_error_codes(error_codes, &rt);

    // SPEC §5.2 "Declared extension" — the direct declaration only. Emitted on
    // every module, `None` included, so "this version declares no extension"
    // is a value a consumer reads rather than an item it infers is missing.
    let extends_value = match extends {
        Some(v) => quote! { Some(#v) },
        None => quote! { None },
    };
    let extends_item = quote! {
        /// The earlier `MAJOR.MINOR` of this slug that this version declares it
        /// extends (front matter `extends`, SPEC §5.2 "Declared extension"), or
        /// `None`. Only the direct declaration; walk a chain with
        #[doc = #extends_link_doc]
        pub const EXTENDS: Option<&str> = #extends_value;
    };

    // SPEC "Superseded and Withdrawn Versions" — the bundle's `deprecation`.
    // `DEPRECATION` is on every module so the answer is a value a consumer can
    // read; the `#[deprecated]` on the root types themselves is added after
    // parsing, by [`apply_deprecation`]. A deprecated module's own generated
    // code (derives, builders, the `Payload` impls) names those types, and
    // rustc warns on same-crate uses too, so the module allows it inside
    // itself — the warning is for the consumer, not for this file.
    let (deprecation_const, allow_deprecated) = match &bundle.deprecation {
        Some(note) => (
            quote! { pub const DEPRECATION: Option<&str> = Some(#note); },
            quote! { #![allow(deprecated)] },
        ),
        None => (
            quote! { pub const DEPRECATION: Option<&str> = None; },
            quote! {},
        ),
    };

    quote! {
        #![doc = #generated_by_doc]
        //!
        #![doc = #slug_doc]
        #allow_deprecated

        #[doc = " Why this specification version is deprecated (SPEC \"Superseded and"]
        #[doc = " Withdrawn Versions\"), or `None` while it is current."]
        #deprecation_const

        #[allow(unused_imports)]
        use #serde_path::{Deserialize, Serialize};

        #body

        impl #rt::Payload for Payload {
            const TYPE_URI: &'static str = #type_uri;
            #bearer_const
            #req_proof_const
            #req_issued_at_const
            #req_size_const
            #req_recipient_const
            const PAYLOAD_SCHEMA: Option<&'static str> = Some(#schema_json);
        }

        #response_payload_impl

        #extends_item

        #error_codes_items
    }
}

/// Emit the SPEC §7.3 item 9 declarations: a `pub const ERROR_CODES` slice on
/// every module, and — when the specification declares any — a `pub mod
/// error_codes` holding one `DeclaredErrorCode` constant per declaration.
///
/// `ERROR_CODES` is emitted even when empty, so "this specification declares no
/// task-specific codes" is a value a consumer can read rather than an absent
/// item it has to infer from a compile error. The submodule is not: an empty
/// module is a namespace with nothing to name.
pub(crate) fn render_error_codes(error_codes: &[ErrorCodeDecl], rt: &TokenStream) -> TokenStream {
    let slice_doc = quote! {
        #[doc = " The extended error codes this specification declares (SPEC §7.3 item 9,"]
        #[doc = " §8.5), in declaration order. Empty when it declares none."]
    };
    if error_codes.is_empty() {
        return quote! {
            #slice_doc
            pub const ERROR_CODES: &[#rt::DeclaredErrorCode] = &[];
        };
    }

    let in_runtime = rt.to_string() == "crate";
    let idents: Vec<proc_macro2::Ident> = error_codes
        .iter()
        .map(|d| quote::format_ident!("{}", d.ident))
        .collect();
    let consts = error_codes.iter().zip(&idents).map(|(d, ident)| {
        let code = &d.code;
        let retryable = d.retryable;
        let head = format!(" `{code}`");
        let meaning_lines: Vec<String> = d.meaning.lines().map(|l| format!(" {l}")).collect();
        let retry_doc = format!(" Declared `retryable: {retryable}`.");
        let (details_doc, details_schema) = match &d.details_schema {
            Some(schema) => {
                // Compact: the constant is read by a validator, not a person,
                // and the spec page renders the schema for people.
                let json =
                    serde_json::to_string(schema).expect("a serde_json::Value always serializes");
                (
                    Some(" Declares a `detailsSchema` for the error's `details` member."),
                    quote! { ::core::option::Option::Some(#json) },
                )
            }
            None => (None, quote! { ::core::option::Option::None }),
        };
        let details_doc = details_doc.into_iter();
        // `DeclaredErrorCode` is `#[non_exhaustive]`, so outside the runtime
        // crate it is built through its hidden const constructor.
        let value = if in_runtime {
            quote! {
                #rt::DeclaredErrorCode {
                    code: #code,
                    retryable: #retryable,
                    details_schema: #details_schema,
                }
            }
        } else {
            quote! {
                #rt::DeclaredErrorCode::__from_declaration(#code, #retryable, #details_schema)
            }
        };
        quote! {
            #[doc = #head]
            #[doc = ""]
            #(#[doc = #meaning_lines])*
            #[doc = ""]
            #[doc = #retry_doc]
            #(#[doc = #details_doc])*
            pub const #ident: #rt::DeclaredErrorCode = #value;
        }
    });

    quote! {
        #slice_doc
        pub const ERROR_CODES: &[#rt::DeclaredErrorCode] = &[#(error_codes::#idents),*];

        /// One constant per extended error code this specification declares
        /// (SPEC §7.3 item 9), named for its local part.
        ///
        /// Emit these rather than a string literal: the code is read from the
        /// specification, so it cannot name a code the specification never
        /// declared.
        pub mod error_codes {
            #(#consts)*
        }
    }
}

#[cfg(test)]
mod indent_safe_doc_tests {
    use super::*;

    fn docs_after(source: &str) -> Vec<String> {
        let mut file: syn::File = syn::parse_str(source).unwrap();
        indent_safe_docs(&mut file);
        let mut docs = Vec::new();
        let syn::Item::Enum(e) = &file.items[0] else {
            panic!("expected an enum");
        };
        for attr in e
            .attrs
            .iter()
            .chain(e.variants.iter().flat_map(|v| v.attrs.iter()))
        {
            if let syn::Meta::NameValue(nv) = &attr.meta {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = &nv.value
                {
                    docs.push(s.value());
                }
            }
        }
        docs
    }

    /// The shape typify gives a titled inline `oneOf` branch: title, blank
    /// line, description. Unrepaired, rustdoc read the description as an
    /// indented code block and compiled it as a doctest.
    #[test]
    fn a_titled_variant_doc_starts_on_its_own_line() {
        let docs = docs_after(
            r#"
            #[doc = "An account's settings."]
            pub enum Settings {
                #[doc = "Settings for aws\n\nEgress: `sts.<region>.amazonaws.com`."]
                Aws { region: String },
            }
            "#,
        );
        assert_eq!(
            docs,
            [
                "An account's settings.",
                "\nSettings for aws\n\nEgress: `sts.<region>.amazonaws.com`.",
            ]
        );
    }

    #[test]
    fn a_doc_already_on_its_own_line_is_left_alone() {
        let docs = docs_after(
            r#"
            #[doc = "\nOne.\n\nTwo."]
            pub enum E { A }
            "#,
        );
        assert_eq!(docs, ["\nOne.\n\nTwo."]);
    }
}
