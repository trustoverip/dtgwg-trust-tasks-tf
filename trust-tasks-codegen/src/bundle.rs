//! Registry bundles: one self-contained, content-addressed `bundle.json` per
//! specification version, and an `index.json` naming every one by digest.
//!
//! Milestone 1 of `docs/design/consumer-side-generation.md`. A bundle is
//! everything a code generator needs to emit one specification version's
//! module — its schemas with every cross-file `$ref` already inlined, and the
//! front-matter policy the runtime enforces — so an application can pin one
//! and generate from it with no access to this repository.
//!
//! Built from the **same code paths** [`crate::generate_one`] uses —
//! [`crate::spec_schemas`] for the schemas, [`crate::read_spec_policy`] for the
//! policy, [`crate::compute_lifecycles`] for the deprecation note — so a
//! bundle's `schema` / `responseSchema` are, as parsed JSON, exactly the
//! generated module's `Payload::PAYLOAD_SCHEMA` / `Response::PAYLOAD_SCHEMA`.
//!
//! # Canonical form and digest
//!
//! `bundle.json` is written compact, with every object's members sorted by
//! UTF-16 code unit and strings escaped as RFC 8785 (JCS) escapes them. Numbers
//! are written as integers; a non-integer is admitted only when its shortest
//! round-trip form is also its ECMAScript form, and anything else stops the
//! build — so the bytes are the JCS serialization of the bundle. The digest is
//! `sha256:<lowercase hex>` of exactly those bytes (no trailing newline): a
//! consumer re-serializing a parsed bundle with any JCS implementation gets the
//! same digest.
//!
//! # What is not bundled
//!
//! `trust-task-error` ([`crate::SKIP_SLUGS`]) — every runtime models it by
//! hand, so there is no generated module for a bundle to stand for. `index.json`
//! says so in `excluded`. Withdrawn versions **are** bundled (with `status:
//! "withdrawn"`), so a pinned bundle remains fetchable and verifiable after its
//! version leaves the libraries.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::{
    compute_lifecycles, discover_specs, parse_version, read_spec_policy, spec_schemas, Lifecycle,
    Spec, SKIP_SLUGS,
};

/// `bundleVersion` — the shape of `bundle.json`. Bumped on any change a reader
/// of the previous shape would misread.
pub(crate) const BUNDLE_VERSION: u64 = 1;
/// `indexVersion` — the shape of `index.json`.
pub(crate) const INDEX_VERSION: u64 = 1;
/// File name of a bundle inside `<out>/<slug>/<M.N>/`.
pub(crate) const BUNDLE_FILE: &str = "bundle.json";
/// File name of the index inside `<out>/`.
pub(crate) const INDEX_FILE: &str = "index.json";

/// One specification version's bundle, serialized.
pub(crate) struct BuiltBundle {
    pub(crate) slug: String,
    pub(crate) version: String,
    pub(crate) type_uri: String,
    pub(crate) status: String,
    pub(crate) deprecation: Option<String>,
    pub(crate) extends: Option<String>,
    /// The canonical bytes written to `bundle.json`.
    pub(crate) bytes: Vec<u8>,
    /// `sha256:<hex>` of [`Self::bytes`].
    pub(crate) digest: String,
}

impl BuiltBundle {
    /// `<slug>/<M.N>/bundle.json`, relative to the index.
    pub(crate) fn path(&self) -> String {
        format!("{}/{}/{BUNDLE_FILE}", self.slug, self.version)
    }
}

/// Every discovered specification version, with its front-matter `status` and
/// its lifecycle — withdrawn ones included.
pub(crate) fn discover_all(repo_root: &Path) -> Result<Vec<(Spec, Lifecycle)>> {
    let specs_dir = repo_root.join("specs");
    let lifecycles = compute_lifecycles(&specs_dir)?;
    let mut out = Vec::new();
    for mut spec in discover_specs(&specs_dir)? {
        let lifecycle = lifecycles
            .get(&(spec.slug.clone(), spec.version.clone()))
            .cloned()
            .ok_or_else(|| {
                anyhow!(
                    "{}/{}: has a payload.schema.json but no spec.md to read its status from",
                    spec.slug,
                    spec.version
                )
            })?;
        if let Lifecycle::Deprecated(note) = &lifecycle {
            spec.deprecation = Some(note.clone());
        }
        out.push((spec, lifecycle));
    }
    // Numeric version order within a slug, so `0.10` follows `0.9`.
    out.sort_by(|(a, _), (b, _)| {
        (a.slug.as_str(), parse_version(&a.version)).cmp(&(&b.slug, parse_version(&b.version)))
    });
    Ok(out)
}

/// The parsed YAML front matter of a `spec.md`.
fn read_front_matter(spec_md_path: &Path) -> Result<Value> {
    let text = fs::read_to_string(spec_md_path)
        .with_context(|| format!("read {}", spec_md_path.display()))?;
    let mut lines = text.lines();
    if lines.next().unwrap_or("").trim() != "---" {
        return Err(anyhow!("{} has no front matter", spec_md_path.display()));
    }
    let mut front_matter = String::new();
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        front_matter.push_str(line);
        front_matter.push('\n');
    }
    serde_saphyr::from_str(&front_matter)
        .with_context(|| format!("parse YAML front matter in {}", spec_md_path.display()))
}

/// Build one specification version's bundle.
pub(crate) fn build_bundle(spec: &Spec) -> Result<BuiltBundle> {
    let schemas = spec_schemas(spec)?;
    let policy = read_spec_policy(spec, schemas.response.is_some())?;
    let front_matter = read_front_matter(&spec.spec_md_path())?;
    let fm_str = |key: &str| {
        front_matter
            .get(key)
            .and_then(Value::as_str)
            .map(str::to_string)
    };
    let status = fm_str("status")
        .ok_or_else(|| anyhow!("{}/{} declares no `status`", spec.slug, spec.version))?;

    let error_codes: Vec<Value> = policy
        .error_codes
        .iter()
        .map(|decl| {
            let mut entry = Map::new();
            entry.insert("code".into(), json!(decl.code));
            entry.insert("meaning".into(), json!(decl.meaning));
            entry.insert("retryable".into(), json!(decl.retryable));
            if let Some(schema) = &decl.details_schema {
                entry.insert("detailsSchema".into(), schema.clone());
            }
            Value::Object(entry)
        })
        .collect();

    let bundle = json!({
        "bundleVersion": BUNDLE_VERSION,
        "typeUri": spec.type_uri(),
        "slug": spec.slug,
        "version": spec.version,
        "status": status,
        "deprecation": spec.deprecation,
        "extends": policy.extends,
        "supersededBy": fm_str("supersededBy"),
        "targetFrameworkVersion": fm_str("targetFrameworkVersion"),
        "policy": {
            "bearer": policy.is_bearer,
            "proofRequired": {
                "request": policy.is_proof_required.request,
                "response": policy.is_proof_required.response,
            },
            "issuedAtRequired": {
                "request": policy.is_issued_at_required.request,
                "response": policy.is_issued_at_required.response,
            },
            // The response swaps the parties, so its `recipient` member is the
            // request's issuer party (SPEC §7.2 item 5).
            "recipientRequired": {
                "request": policy.recipient_required,
                "response": policy.issuer_required,
            },
            "maxDocumentBytes": {
                "request": policy.max_document_bytes.request,
                "response": policy.max_document_bytes.response,
            },
        },
        "errorCodes": error_codes,
        "schema": schemas.schema,
        "responseSchema": schemas.response,
    });

    let bytes = canonical_json(&bundle)
        .with_context(|| format!("canonicalize bundle for {}/{}", spec.slug, spec.version))?
        .into_bytes();
    let digest = sha256_digest(&bytes);
    Ok(BuiltBundle {
        slug: spec.slug.clone(),
        version: spec.version.clone(),
        type_uri: spec.type_uri(),
        status,
        deprecation: spec.deprecation.clone(),
        extends: policy.extends,
        bytes,
        digest,
    })
}

/// Build every bundle in the registry, in index order.
pub(crate) fn build_all(repo_root: &Path) -> Result<Vec<BuiltBundle>> {
    discover_all(repo_root)?
        .iter()
        .map(|(spec, _)| {
            build_bundle(spec)
                .with_context(|| format!("failed to bundle {}/{}", spec.slug, spec.version))
        })
        .collect()
}

/// The canonical bytes of `index.json`.
pub(crate) fn render_index(
    bundles: &[BuiltBundle],
    generated_from: Option<&str>,
) -> Result<String> {
    let specs: Vec<Value> = bundles
        .iter()
        .map(|b| {
            json!({
                "typeUri": b.type_uri,
                "slug": b.slug,
                "version": b.version,
                "status": b.status,
                "deprecation": b.deprecation,
                "extends": b.extends,
                "digest": b.digest,
                "path": b.path(),
            })
        })
        .collect();
    let excluded: Vec<Value> = SKIP_SLUGS
        .iter()
        .map(|slug| {
            json!({
                "slug": slug,
                "reason": "modelled by hand in every runtime; no generated module to bundle",
            })
        })
        .collect();
    canonical_json(&json!({
        "indexVersion": INDEX_VERSION,
        "bundleVersion": BUNDLE_VERSION,
        "generatedFrom": generated_from,
        "excluded": excluded,
        "specs": specs,
    }))
}

/// The commit the registry was built from, when `git` can say.
fn git_head(repo_root: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let head = String::from_utf8(out.stdout).ok()?.trim().to_string();
    (!head.is_empty()).then_some(head)
}

/// `--bundles <out_dir>`: write every bundle and the index, and remove any
/// `bundle.json` under `out_dir` that no longer names a specification version.
pub(crate) fn run(repo_root: &Path, out_dir: &Path) -> Result<()> {
    let started = Instant::now();
    let bundles = build_all(repo_root)?;
    let index = render_index(&bundles, git_head(repo_root).as_deref())?;

    fs::create_dir_all(out_dir).with_context(|| format!("create {}", out_dir.display()))?;
    let mut written: std::collections::BTreeSet<PathBuf> = Default::default();
    for b in &bundles {
        let path = out_dir.join(&b.slug).join(&b.version).join(BUNDLE_FILE);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, &b.bytes).with_context(|| format!("write {}", path.display()))?;
        written.insert(path);
    }
    let mut stale = 0usize;
    for entry in WalkDir::new(out_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file()
            && entry.file_name() == BUNDLE_FILE
            && !written.contains(entry.path())
        {
            fs::remove_file(entry.path())
                .with_context(|| format!("remove stale {}", entry.path().display()))?;
            stale += 1;
        }
    }
    fs::write(out_dir.join(INDEX_FILE), &index)?;

    let total: usize = bundles.iter().map(|b| b.bytes.len()).sum();
    let max = bundles.iter().map(|b| b.bytes.len()).max().unwrap_or(0);
    println!(
        "Wrote {} bundles ({} bytes, avg {}, max {}) and {} ({} bytes) to {} in {:.2?}{}",
        bundles.len(),
        total,
        total / bundles.len().max(1),
        max,
        INDEX_FILE,
        index.len(),
        out_dir.display(),
        started.elapsed(),
        if stale > 0 {
            format!("; removed {stale} stale bundle(s)")
        } else {
            String::new()
        }
    );
    Ok(())
}

/// `sha256:<lowercase hex>` of `bytes`.
pub(crate) fn sha256_digest(bytes: &[u8]) -> String {
    let hash = Sha256::digest(bytes);
    let mut out = String::with_capacity(7 + 64);
    out.push_str("sha256:");
    for byte in hash {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Serialize `value` in the canonical form described in the module docs: the
/// RFC 8785 (JCS) form for every value the registry contains.
///
/// Written out rather than relying on `serde_json::Map` happening to be a
/// `BTreeMap`: that holds only while no crate in the build enables
/// `serde_json/preserve_order`, and feature unification would flip it
/// silently. Byte order and UTF-16 order agree for every key in the registry
/// today, but JCS specifies UTF-16, so that is what is sorted on.
pub(crate) fn canonical_json(value: &Value) -> Result<String> {
    let mut out = String::new();
    write_canonical(value, &mut out)?;
    Ok(out)
}

fn write_canonical(value: &Value, out: &mut String) -> Result<()> {
    match value {
        Value::Null | Value::Bool(_) | Value::String(_) => {
            out.push_str(&serde_json::to_string(value)?);
        }
        Value::Number(n) => {
            if n.is_i64() || n.is_u64() {
                out.push_str(&n.to_string());
            } else {
                // serde_json writes the shortest round-trip form, as
                // ECMAScript does, except that it keeps a `.0` on an integral
                // float and spells exponents differently. Refuse those rather
                // than emit bytes a JCS implementation would not reproduce.
                let text = n.to_string();
                if text.ends_with(".0") || text.contains(['e', 'E']) {
                    return Err(anyhow!(
                        "number {text} has no unambiguous canonical (JCS) form here; \
                         write it as an integer or a plain decimal"
                    ));
                }
                out.push_str(&text);
            }
        }
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(item, out)?;
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
            out.push('{');
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(key)?);
                out.push(':');
                write_canonical(&map[key.as_str()], out)?;
            }
            out.push('}');
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{find_repo_root, generate_one};

    /// The consts one `impl crate::Payload for <ty>` block carries, read back
    /// out of a generated module.
    #[derive(Debug, Default)]
    struct ImplConsts {
        schema: Option<Value>,
        bearer: bool,
        proof: bool,
        issued_at: bool,
        recipient: bool,
        max_bytes: Option<u64>,
    }

    fn lit_str(expr: &syn::Expr) -> Option<String> {
        match expr {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) => Some(s.value()),
            syn::Expr::Call(call) => call.args.first().and_then(lit_str),
            _ => None,
        }
    }

    fn lit_int(expr: &syn::Expr) -> Option<u64> {
        match expr {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(i),
                ..
            }) => i.base10_parse().ok(),
            syn::Expr::Call(call) => call.args.first().and_then(lit_int),
            _ => None,
        }
    }

    fn lit_bool(expr: &syn::Expr) -> bool {
        matches!(
            expr,
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Bool(b),
                ..
            }) if b.value
        )
    }

    /// Read the `Payload` impl for `ty` (`"Payload"` / `"Response"`).
    fn impl_consts(file: &syn::File, ty: &str) -> Option<ImplConsts> {
        let block = file.items.iter().find_map(|item| match item {
            syn::Item::Impl(imp)
                if imp
                    .trait_
                    .as_ref()
                    .and_then(|(_, p, _)| p.segments.last())
                    .is_some_and(|s| s.ident == "Payload")
                    && matches!(&*imp.self_ty, syn::Type::Path(p)
                        if p.path.is_ident(ty)) =>
            {
                Some(imp)
            }
            _ => None,
        })?;
        let mut out = ImplConsts::default();
        for item in &block.items {
            let syn::ImplItem::Const(c) = item else {
                continue;
            };
            match c.ident.to_string().as_str() {
                "PAYLOAD_SCHEMA" => {
                    out.schema = lit_str(&c.expr).map(|s| serde_json::from_str(&s).unwrap())
                }
                "IS_BEARER" => out.bearer = lit_bool(&c.expr),
                "IS_PROOF_REQUIRED" => out.proof = lit_bool(&c.expr),
                "IS_ISSUED_AT_REQUIRED" => out.issued_at = lit_bool(&c.expr),
                "IS_RECIPIENT_REQUIRED" => out.recipient = lit_bool(&c.expr),
                "MAX_DOCUMENT_BYTES" => out.max_bytes = lit_int(&c.expr),
                _ => {}
            }
        }
        Some(out)
    }

    /// A module-level `pub const <name>: Option<&str>`.
    fn module_opt_str(file: &syn::File, name: &str) -> Option<String> {
        file.items.iter().find_map(|item| match item {
            syn::Item::Const(c) if c.ident == name => lit_str(&c.expr),
            _ => None,
        })
    }

    fn spec(slug: &str, version: &str) -> Spec {
        let repo_root = find_repo_root().unwrap();
        discover_all(&repo_root)
            .unwrap()
            .into_iter()
            .map(|(s, _)| s)
            .find(|s| s.slug == slug && s.version == version)
            .unwrap_or_else(|| panic!("no spec {slug}/{version}"))
    }

    /// The bundle carries, as parsed JSON, exactly what the generated module
    /// embeds — schema, response schema, and every policy constant — for a
    /// spread of real specifications.
    #[test]
    fn bundle_agrees_with_generated_module() {
        let cases = [
            ("acl/grant", "0.1"),            // deprecated: superseded by 0.2
            ("acl/grant", "0.2"),            // proof + recipient + issuedAt REQUIRED
            ("vault/upsert", "0.4"),         // errorCodes with detailsSchema $refs
            ("chat/message", "0.1"),         // no $defs.Response
            ("provision/vta/redeem", "0.1"), // maxDocumentBytes
        ];
        let out_root = std::env::temp_dir().join("tt-bundle-test-unused");
        for (slug, version) in cases {
            let spec = spec(slug, version);
            let built = build_bundle(&spec).unwrap();
            let bundle: Value = serde_json::from_slice(&built.bytes).unwrap();
            let generated = generate_one(&spec, &out_root).unwrap();
            let file = syn::parse_file(&generated.contents).unwrap();
            let ctx = format!("{slug}/{version}");

            let request = impl_consts(&file, "Payload").expect("Payload impl");
            assert_eq!(
                request.schema.as_ref(),
                Some(&bundle["schema"]),
                "{ctx} schema"
            );
            let policy = &bundle["policy"];
            assert_eq!(policy["bearer"], json!(request.bearer), "{ctx} bearer");
            assert_eq!(
                policy["proofRequired"]["request"],
                json!(request.proof),
                "{ctx}"
            );
            assert_eq!(
                policy["issuedAtRequired"]["request"],
                json!(request.issued_at),
                "{ctx}"
            );
            assert_eq!(
                policy["recipientRequired"]["request"],
                json!(request.recipient),
                "{ctx}"
            );
            assert_eq!(
                policy["maxDocumentBytes"]["request"],
                json!(request.max_bytes),
                "{ctx}"
            );

            match impl_consts(&file, "Response") {
                Some(response) => {
                    assert_eq!(
                        response.schema.as_ref(),
                        Some(&bundle["responseSchema"]),
                        "{ctx} responseSchema"
                    );
                    assert_eq!(policy["proofRequired"]["response"], json!(response.proof));
                    assert_eq!(
                        policy["issuedAtRequired"]["response"],
                        json!(response.issued_at)
                    );
                    assert_eq!(
                        policy["recipientRequired"]["response"],
                        json!(response.recipient)
                    );
                    assert_eq!(
                        policy["maxDocumentBytes"]["response"],
                        json!(response.max_bytes)
                    );
                }
                None => assert_eq!(bundle["responseSchema"], Value::Null, "{ctx}"),
            }

            assert_eq!(
                bundle["deprecation"],
                json!(module_opt_str(&file, "DEPRECATION")),
                "{ctx} deprecation"
            );
            assert_eq!(
                bundle["extends"],
                json!(module_opt_str(&file, "EXTENDS")),
                "{ctx} extends"
            );
            assert_eq!(built.digest, sha256_digest(&built.bytes));
        }
        // The deprecated case really is one, so the comparison above saw a note.
        assert!(spec("acl/grant", "0.1").deprecation.is_some());
    }

    /// Two runs produce identical files, every digest in the index is the
    /// digest of the file it names, and the index names exactly the bundles on
    /// disk.
    #[test]
    fn bundles_are_deterministic_and_indexed() {
        let repo_root = find_repo_root().unwrap();
        let base = std::env::temp_dir().join(format!("tt-bundles-{}", std::process::id()));
        let (a, b) = (base.join("a"), base.join("b"));
        run(&repo_root, &a).unwrap();
        run(&repo_root, &b).unwrap();

        let files = |root: &Path| -> Vec<(PathBuf, Vec<u8>)> {
            let mut v: Vec<_> = WalkDir::new(root)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .map(|e| {
                    (
                        e.path().strip_prefix(root).unwrap().to_path_buf(),
                        fs::read(e.path()).unwrap(),
                    )
                })
                .collect();
            v.sort();
            v
        };
        let (fa, fb) = (files(&a), files(&b));
        assert_eq!(fa.len(), fb.len());
        assert!(fa == fb, "two runs wrote different bytes");

        let index: Value = serde_json::from_slice(&fs::read(a.join(INDEX_FILE)).unwrap()).unwrap();
        let listed = index["specs"].as_array().unwrap();
        let on_disk = fa
            .iter()
            .filter(|(p, _)| p.file_name().is_some_and(|n| n == BUNDLE_FILE))
            .count();
        assert_eq!(listed.len(), on_disk);
        for entry in listed {
            let bytes = fs::read(a.join(entry["path"].as_str().unwrap())).unwrap();
            assert_eq!(entry["digest"], json!(sha256_digest(&bytes)));
            // Re-serializing the parsed file reproduces it: the bytes are canonical.
            let parsed: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(canonical_json(&parsed).unwrap().into_bytes(), bytes);
            assert!(SKIP_SLUGS.iter().all(|s| entry["slug"] != json!(s)));
        }

        // A stale bundle is removed on the next run.
        let stale = a.join("no-such/9.9").join(BUNDLE_FILE);
        fs::create_dir_all(stale.parent().unwrap()).unwrap();
        fs::write(&stale, b"{}").unwrap();
        run(&repo_root, &a).unwrap();
        assert!(!stale.exists());

        fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn canonical_json_sorts_and_refuses_ambiguous_numbers() {
        let v = json!({"b": 1, "a": [true, null, "x\u{1}"], "\u{e9}": 0.5});
        assert_eq!(
            canonical_json(&v).unwrap(),
            r#"{"a":[true,null,"x\u0001"],"b":1,"é":0.5}"#
        );
        assert!(canonical_json(&json!(1.0)).is_err());
        assert!(canonical_json(&json!(1e21)).is_err());
    }
}
