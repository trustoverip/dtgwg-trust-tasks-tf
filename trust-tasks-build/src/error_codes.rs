//! The `errorCodes` a specification version declares (SPEC §7.3 item 9).

use anyhow::{anyhow, Result};
use serde_json::Value;

/// One `errorCodes` declaration from a spec's front matter (SPEC §7.3 item 9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorCodeDecl {
    /// Fully qualified wire form, `<namespace>:<local>`.
    pub code: String,
    /// The declaration's `meaning`, rendered as the constant's rustdoc.
    pub meaning: String,
    /// The declared `retryable`.
    pub retryable: bool,
    /// The declared `detailsSchema`, the JSON Schema an error's `details`
    /// member conforms to (SPEC §7.3 item 9), self-contained (its cross-file
    /// `$ref`s inlined).
    pub details_schema: Option<Value>,
    /// The constant's identifier inside the generated `error_codes` module —
    /// the local part in SCREAMING_SNAKE_CASE.
    pub ident: String,
}

/// The `errorCodes` declarations in parsed front matter `value`, for the spec
/// whose slug is `slug`.
///
/// The build (`scripts/build-registry.mjs`) already validates these against
/// `spec.meta.schema.json` and the §8.5 namespace rule. They are re-checked
/// here because what this function emits is a *constant a consumer puts on the
/// wire*: `DeclaredErrorCode` converts to a `TrustTaskCode` infallibly on the
/// strength of it, so a code that does not parse, or a namespace outside the
/// slug's family, must stop generation rather than reach a library.
pub fn error_code_decls(slug: &str, value: &serde_json::Value) -> Result<Vec<ErrorCodeDecl>> {
    use heck::ToShoutySnakeCase;

    let Some(entries) = value.get("errorCodes") else {
        return Ok(Vec::new());
    };
    if entries.is_null() {
        return Ok(Vec::new());
    }
    let entries = entries
        .as_array()
        .ok_or_else(|| anyhow!("`errorCodes` is not a list"))?;

    // The slug itself plus each proper path prefix of it (§8.5 rule 2).
    let permitted: Vec<&str> = slug
        .match_indices('/')
        .map(|(i, _)| &slug[..i])
        .chain(std::iter::once(slug))
        .collect();

    let mut out: Vec<ErrorCodeDecl> = Vec::with_capacity(entries.len());
    for entry in entries {
        let code = entry
            .get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("an `errorCodes` entry has no string `code`"))?;
        let retryable = entry
            .get("retryable")
            .and_then(|v| v.as_bool())
            .ok_or_else(|| anyhow!("errorCodes[{code:?}] has no boolean `retryable`"))?;
        let meaning = entry
            .get("meaning")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .lines()
            .map(str::trim)
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();
        let details_schema = match entry.get("detailsSchema") {
            None => None,
            Some(schema) if schema.is_object() => Some(schema.clone()),
            Some(_) => {
                return Err(anyhow!(
                    "errorCodes[{code:?}].detailsSchema is not an object (SPEC §7.3 item 9)"
                ));
            }
        };

        let (namespace, local) = code
            .split_once(':')
            .ok_or_else(|| anyhow!("errorCodes[{code:?}] has no `:` (SPEC §8.5)"))?;
        if !permitted.contains(&namespace) {
            return Err(anyhow!(
                "errorCodes[{code:?}] is namespaced {namespace:?}, which is neither the slug \
                 {slug:?} nor a path prefix of it (SPEC §8.5)"
            ));
        }
        let mut chars = local.chars();
        let well_formed = chars.next().is_some_and(|c| c.is_ascii_lowercase())
            && chars.all(|c| c.is_ascii_alphanumeric() || c == '_');
        if !well_formed {
            return Err(anyhow!(
                "errorCodes[{code:?}] has a local part {local:?} outside the \
                 `[a-z][a-zA-Z0-9_]*` grammar of spec.meta.schema.json"
            ));
        }

        let ident = local.to_shouty_snake_case();
        // Two declarations whose local parts differ only in casing
        // (`notFound`, `not_found`), or that share a local part under
        // different namespaces (`keys:notFound`, `keys/revoke:notFound`), would
        // claim one constant. Refuse rather than pick one silently.
        if let Some(clash) = out.iter().find(|d| d.ident == ident) {
            return Err(anyhow!(
                "errorCodes[{code:?}] and errorCodes[{:?}] both map to the constant \
                 `error_codes::{ident}`",
                clash.code
            ));
        }
        out.push(ErrorCodeDecl {
            code: code.to_string(),
            meaning,
            retryable,
            details_schema,
            ident,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod error_code_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_details_schema_is_carried_and_its_absence_is_none() {
        let out = decls(
            "rooms/keys/file-key",
            r#"
errorCodes:
  - code: rooms/keys/file-key:unknownEpoch
    meaning: x
    retryable: false
    detailsSchema:
      type: object
      required: [reason]
      properties:
        reason: { type: string, enum: [notDelivered, beyondChain] }
        heldEpoch: { type: integer, minimum: 1 }
  - code: rooms/keys/file-key:noGroup
    meaning: y
    retryable: false
"#,
        )
        .unwrap();
        assert_eq!(
            out[0].details_schema,
            Some(json!({
                "type": "object",
                "required": ["reason"],
                "properties": {
                    "reason": { "type": "string", "enum": ["notDelivered", "beyondChain"] },
                    "heldEpoch": { "type": "integer", "minimum": 1 }
                }
            }))
        );
        assert_eq!(out[1].details_schema, None);
    }

    #[test]
    fn a_details_schema_that_is_not_an_object_is_refused() {
        let err = decls(
            "keys/revoke",
            "errorCodes:\n  - code: keys/revoke:notFound\n    meaning: x\n    retryable: false\n    detailsSchema: true\n",
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("detailsSchema is not an object"),
            "{err}"
        );
    }

    #[test]
    fn a_constant_carries_its_details_schema_or_none() {
        let with = ErrorCodeDecl {
            code: "rooms/keys/file-key:unknownEpoch".into(),
            meaning: "x".into(),
            retryable: false,
            details_schema: Some(json!({ "type": "object" })),
            ident: "UNKNOWN_EPOCH".into(),
        };
        let without = ErrorCodeDecl {
            code: "rooms/keys/file-key:noGroup".into(),
            details_schema: None,
            ident: "NO_GROUP".into(),
            ..with.clone()
        };
        let out =
            crate::render::render_error_codes(&[with, without], &quote::quote!(crate)).to_string();
        assert!(
            out.contains(
                r#"details_schema : :: core :: option :: Option :: Some ("{\"type\":\"object\"}")"#
            ),
            "{out}"
        );
        assert!(
            out.contains("details_schema : :: core :: option :: Option :: None"),
            "{out}"
        );
    }

    fn decls(slug: &str, yaml: &str) -> Result<Vec<ErrorCodeDecl>> {
        let value: serde_json::Value = serde_saphyr::from_str(yaml).unwrap();
        error_code_decls(slug, &value)
    }

    #[test]
    fn absent_null_and_empty_all_declare_nothing() {
        assert!(decls("acl/grant", "slug: acl/grant").unwrap().is_empty());
        assert!(decls("acl/grant", "errorCodes:").unwrap().is_empty());
        assert!(decls("acl/grant", "errorCodes: []").unwrap().is_empty());
    }

    #[test]
    fn a_declaration_becomes_a_shouty_constant_in_declaration_order() {
        let out = decls(
            "vtc/join-requests/withdraw",
            r#"
errorCodes:
  - code: vtc/join-requests/withdraw:notFound
    meaning: >-
      The applicant has no open request.
    retryable: false
  - code: vtc/join-requests/withdraw:alreadyDecided
    meaning: Terminal.
    retryable: true
"#,
        )
        .unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].ident, "NOT_FOUND");
        assert_eq!(out[0].code, "vtc/join-requests/withdraw:notFound");
        assert_eq!(out[0].meaning, "The applicant has no open request.");
        assert!(!out[0].retryable);
        assert_eq!(out[1].ident, "ALREADY_DECIDED");
        assert!(out[1].retryable);
    }

    /// Frozen framework 0.1 specs declare snake_case locals; they must name the
    /// same shape of constant as a lowerCamelCase one.
    #[test]
    fn a_snake_case_local_names_the_same_shape_of_constant() {
        let out = decls(
            "acl/grant",
            "errorCodes:\n  - code: acl/grant:role_not_recognized\n    meaning: x\n    retryable: false\n",
        )
        .unwrap();
        assert_eq!(out[0].ident, "ROLE_NOT_RECOGNIZED");
    }

    /// SPEC §8.5 rule 2: a family namespace is a proper path prefix of the slug.
    #[test]
    fn a_family_namespace_is_accepted() {
        let out = decls(
            "keys/revoke",
            "errorCodes:\n  - code: keys:notFound\n    meaning: x\n    retryable: false\n",
        )
        .unwrap();
        assert_eq!(out[0].ident, "NOT_FOUND");
    }

    #[test]
    fn an_unrelated_namespace_is_refused() {
        let err = decls(
            "keys/revoke",
            "errorCodes:\n  - code: acl/grant:notFound\n    meaning: x\n    retryable: false\n",
        )
        .unwrap_err();
        assert!(err.to_string().contains("neither the slug"), "{err}");
        // `keys/rev` is a string prefix of the slug but not a path prefix.
        assert!(decls(
            "keys/revoke",
            "errorCodes:\n  - code: keys/rev:notFound\n    meaning: x\n    retryable: false\n",
        )
        .is_err());
    }

    #[test]
    fn a_malformed_local_part_is_refused() {
        for local in ["NotFound", "", "not-found", "1st"] {
            let yaml = format!(
                "errorCodes:\n  - code: \"keys/revoke:{local}\"\n    meaning: x\n    retryable: false\n"
            );
            assert!(decls("keys/revoke", &yaml).is_err(), "{local:?} accepted");
        }
    }

    #[test]
    fn a_missing_retryable_is_refused() {
        assert!(decls(
            "keys/revoke",
            "errorCodes:\n  - code: keys/revoke:notFound\n    meaning: x\n",
        )
        .is_err());
    }

    /// Two declarations that would claim one constant: the generator must not
    /// silently keep one of them.
    #[test]
    fn two_declarations_claiming_one_constant_are_refused() {
        let casing = "errorCodes:\n  - code: keys/revoke:notFound\n    meaning: x\n    retryable: false\n  - code: keys/revoke:not_found\n    meaning: y\n    retryable: false\n";
        assert!(decls("keys/revoke", casing).is_err());
        let namespaces = "errorCodes:\n  - code: keys:notFound\n    meaning: x\n    retryable: false\n  - code: keys/revoke:notFound\n    meaning: y\n    retryable: false\n";
        let err = decls("keys/revoke", namespaces).unwrap_err();
        assert!(err.to_string().contains("error_codes::NOT_FOUND"), "{err}");
    }

    #[test]
    fn an_empty_list_renders_an_empty_slice_and_no_submodule() {
        let out = crate::render::render_error_codes(&[], &quote::quote!(crate)).to_string();
        assert!(out.contains("pub const ERROR_CODES"));
        assert!(!out.contains("mod error_codes"));
    }
}
