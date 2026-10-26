//! What a specification version's `spec.md` front matter says about how its
//! documents are consumed (SPEC §7.2 / §7.3) — the policy a bundle carries.
//!
//! Moved here from the repository's `trust-tasks-codegen` so the registry's
//! bundles and a third party's (`trust-tasks registry build`) are read by one
//! implementation. `scripts/check-bindings-conformance.mjs` re-derives the same
//! policy independently and compares; that independence is the check, so it
//! is not shared with this.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use super::refs::{resolve_cross_file_refs, Resolver};
use super::SpecDir;
use crate::error_codes::{error_code_decls, ErrorCodeDecl};

/// The parsed YAML front matter of `spec_md_path`, or `None` when the file is
/// missing or does not start with a `---` block.
pub fn read_front_matter(spec_md_path: &Path) -> Result<Option<Value>> {
    if !spec_md_path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(spec_md_path)
        .with_context(|| format!("read {}", spec_md_path.display()))?;
    let mut lines = text.lines();
    if lines.next().unwrap_or("").trim() != "---" {
        return Ok(None);
    }
    let mut front_matter = String::new();
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        front_matter.push_str(line);
        front_matter.push('\n');
    }
    let value: Value = serde_saphyr::from_str(&front_matter)
        .with_context(|| format!("parse YAML front matter in {}", spec_md_path.display()))?;
    Ok(Some(value))
}

/// A value declared separately for each document variant.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Variants<T> {
    /// The request document's.
    pub request: T,
    /// The response document's.
    pub response: T,
}

/// Everything the front matter says about consumption, read once.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SpecPolicyDecl {
    /// `bearer: true` (SPEC §4.8.3).
    pub is_bearer: bool,
    /// `proofRequirement` is `REQUIRED`, per variant (SPEC §7.3 item 8).
    pub is_proof_required: Variants<bool>,
    /// `issuedAtRequirement` is `REQUIRED`, per variant (SPEC §7.3 item 17).
    pub is_issued_at_required: Variants<bool>,
    /// `maxDocumentBytes`, per variant.
    pub max_document_bytes: Variants<Option<usize>>,
    /// The request's `recipient` member tracks the recipient party.
    pub recipient_required: bool,
    /// The response swaps the parties, so its `recipient` member tracks the
    /// issuer party (§7.2 item 5).
    pub issuer_required: bool,
    /// `errorCodes`, with any `detailsSchema` made self-contained.
    pub error_codes: Vec<ErrorCodeDecl>,
    /// `extends` (SPEC §5.2).
    pub extends: Option<String>,
}

/// Read a specification version's policy. `has_response` is whether its schema
/// defines `$defs.Response`.
pub fn read_spec_policy(
    spec: &SpecDir,
    has_response: bool,
    refs: &Resolver,
) -> Result<SpecPolicyDecl> {
    let md = spec.spec_md_path();
    let fm = read_front_matter(&md)?;
    let fm = fm.as_ref();
    let max_document_bytes = max_document_bytes(fm, &md)?;
    if !has_response && max_document_bytes.response.is_some() {
        return Err(anyhow!(
            "{}/{}: maxDocumentBytes declares a `response` bound, but the schema defines no \
             $defs.Response",
            spec.slug,
            spec.version
        ));
    }
    Ok(SpecPolicyDecl {
        is_bearer: fm
            .and_then(|v| v.get("bearer"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        is_proof_required: requirement(fm, "proofRequirement"),
        is_issued_at_required: requirement(fm, "issuedAtRequirement"),
        max_document_bytes,
        recipient_required: member_required(fm, "recipient"),
        issuer_required: member_required(fm, "issuer"),
        error_codes: error_codes(spec, fm, &md, refs)?,
        extends: extends(spec, fm, &md)?,
    })
}

/// A §7.3 item 8 / item 17 declaration: a single `requirement` covering every
/// variant, or a per-variant `request` / `response` pair. Only `REQUIRED`
/// obliges a consumer to reject, so each variant reduces to a bool. A
/// per-variant declaration that omits `response` takes the request's value —
/// the only reading that cannot weaken a variant by omission.
fn requirement(fm: Option<&Value>, key: &str) -> Variants<bool> {
    let Some(decl) = fm.and_then(|v| v.get(key)) else {
        return Variants::default();
    };
    let level = |k: &str| decl.get(k).and_then(Value::as_str).map(|s| s == "REQUIRED");
    if let Some(all) = level("requirement") {
        return Variants {
            request: all,
            response: all,
        };
    }
    let request = level("request").unwrap_or(false);
    Variants {
        request,
        response: level("response").unwrap_or(request),
    }
}

/// `maxDocumentBytes`. Unlike the proof and `issuedAt` declarations, an absent
/// `response` does **not** take the request's value: the member bounds what a
/// consumer must accept as well as what it may refuse, and a response is
/// usually far smaller than the request. Absent means "no declared bound".
fn max_document_bytes(fm: Option<&Value>, md: &Path) -> Result<Variants<Option<usize>>> {
    let Some(decl) = fm.and_then(|v| v.get("maxDocumentBytes")) else {
        return Ok(Variants::default());
    };
    let bound = |key: &str| -> Result<Option<usize>> {
        match decl.get(key) {
            None => Ok(None),
            Some(v) => v
                .as_u64()
                .and_then(|n| usize::try_from(n).ok())
                .map(Some)
                .ok_or_else(|| {
                    anyhow!(
                        "{}: maxDocumentBytes.{key} must be a non-negative integer",
                        md.display()
                    )
                }),
        }
    };
    let request = bound("request")?;
    let response = bound("response")?;
    if request.is_none() && response.is_none() {
        return Err(anyhow!(
            "{}: maxDocumentBytes declares neither a `request` nor a `response` bound",
            md.display()
        ));
    }
    Ok(Variants { request, response })
}

/// Whether the party filling the framework `member` (`"issuer"` or
/// `"recipient"`) is declared `requirement: REQUIRED`. Per SPEC §7.2 item 5 /
/// §7.3 item 5, only `REQUIRED` obliges the consumer to reject a document
/// lacking that member in-band.
fn member_required(fm: Option<&Value>, member: &str) -> bool {
    let Some(parties) = fm.and_then(|v| v.get("parties")).and_then(Value::as_array) else {
        return false;
    };
    parties.iter().any(|p| {
        p.get("member").and_then(Value::as_str) == Some(member)
            && p.get("requirement").and_then(Value::as_str) == Some("REQUIRED")
    })
}

/// `extends` (SPEC §5.2 "Declared extension"): the earlier `MAJOR.MINOR` of the
/// same slug this version declares it only adds to. Only its shape is checked
/// here — the registry build proves the registry's declarations additive.
fn extends(spec: &SpecDir, fm: Option<&Value>, md: &Path) -> Result<Option<String>> {
    let Some(decl) = fm.and_then(|v| v.get("extends")) else {
        return Ok(None);
    };
    let target = decl
        .as_str()
        .ok_or_else(|| anyhow!("{}: extends must be a \"MAJOR.MINOR\" string", md.display()))?;
    extends_shape(&spec.version, target)
        .map_err(|why| anyhow!("{}: extends: '{target}' {why}", md.display()))?;
    Ok(Some(target.to_string()))
}

/// `Ok` when `target` is a `MAJOR.MINOR` with `version`'s MAJOR and a strictly
/// lower MINOR — the only links SPEC §5.2 permits.
pub(crate) fn extends_shape(version: &str, target: &str) -> std::result::Result<(), &'static str> {
    let parse = |v: &str| -> Option<(u32, u32)> {
        let (major, minor) = v.split_once('.')?;
        Some((major.parse().ok()?, minor.parse().ok()?))
    };
    let (Some((major, minor)), Some((t_major, t_minor))) = (parse(version), parse(target)) else {
        return Err("is not a MAJOR.MINOR version");
    };
    if t_major != major || t_minor >= minor {
        return Err("must name a strictly earlier version with the same MAJOR");
    }
    Ok(())
}

/// `errorCodes` (SPEC §7.3 item 9). A `detailsSchema` may `$ref` a shared
/// schema file relative to the spec; it is inlined, because a consumer
/// validating against it has no base URI to resolve a relative ref against.
fn error_codes(
    spec: &SpecDir,
    fm: Option<&Value>,
    md: &Path,
    refs: &Resolver,
) -> Result<Vec<ErrorCodeDecl>> {
    let Some(value) = fm else {
        return Ok(Vec::new());
    };
    let mut decls = error_code_decls(&spec.slug, value)
        .with_context(|| format!("errorCodes in {}", md.display()))?;
    let base = super::refs::file_base(md)?;
    for decl in &mut decls {
        if let Some(schema) = decl.details_schema.as_mut() {
            resolve_cross_file_refs(schema, &base, refs).with_context(|| {
                format!(
                    "errorCodes[{:?}].detailsSchema in {}",
                    decl.code,
                    md.display()
                )
            })?;
        }
    }
    Ok(decls)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_strictly_earlier_minor_of_the_same_major_is_a_link() {
        assert!(extends_shape("0.3", "0.2").is_ok());
        assert!(extends_shape("0.3", "0.1").is_ok());
        assert!(extends_shape("1.10", "1.9").is_ok());
        assert!(extends_shape("0.3", "0.3").is_err());
        assert!(extends_shape("0.3", "0.4").is_err());
        assert!(extends_shape("1.1", "0.9").is_err());
        assert!(extends_shape("0.3", "0.x").is_err());
        assert!(extends_shape("0.3", "2").is_err());
    }
}
