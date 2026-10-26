//! Registry bundles: one self-contained, content-addressed `bundle.json` per
//! specification version, and an `index.json` naming every one by digest.
//!
//! Milestone 1 of `docs/design/consumer-side-generation.md`. A bundle is
//! everything a code generator needs to emit one specification version's
//! module — its schemas with every cross-file `$ref` already inlined, and the
//! front-matter policy the runtime enforces — so an application can pin one
//! and generate from it with no access to this repository.
//!
//! The builder itself is `trust_tasks_build::publish` — the one a third party
//! runs through `trust-tasks registry build` — and [`crate::generate_one`]
//! generates every committed module *from* the bundle it builds, so a bundle's
//! `schema` / `responseSchema` are, as parsed JSON, exactly the generated
//! module's `Payload::PAYLOAD_SCHEMA` / `Response::PAYLOAD_SCHEMA`. This file
//! adds only what is the registry's own: its Type URIs, its `excluded` list
//! and the commit it was built from. See `trust_tasks_build::publish` for the
//! canonical form and the digest.
//!
//! # What is not bundled
//!
//! `trust-task-error` ([`crate::SKIP_SLUGS`]) — every runtime models it by
//! hand, so there is no generated module for a bundle to stand for. `index.json`
//! says so in `excluded`. Withdrawn versions **are** bundled (with `status:
//! "withdrawn"`), so a pinned bundle remains fetchable and verifiable after its
//! version leaves the libraries.

use std::path::Path;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use trust_tasks_build::publish::{self, BuiltBundle, Resolver, INDEX_FILE};

use crate::{compute_lifecycles, discover_specs, Lifecycle, Spec, SKIP_SLUGS};
use trust_tasks_build::publish::parse_version;

/// Every discovered specification version, with its lifecycle — withdrawn
/// ones included.
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

/// Build one specification version's bundle. The registry's own `$ref`s are
/// all relative files, so nothing is fetched.
pub(crate) fn build_bundle(spec: &Spec) -> Result<BuiltBundle> {
    publish::build_bundle(
        &spec.dir(),
        &spec.type_uri(),
        spec.deprecation.as_deref(),
        &Resolver::local(),
    )
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
    let excluded: Vec<(&str, &str)> = SKIP_SLUGS
        .iter()
        .map(|slug| {
            (
                *slug,
                "modelled by hand in every runtime; no generated module to bundle",
            )
        })
        .collect();
    publish::render_index(bundles, generated_from, &excluded)
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
/// The by-digest store is written by `deploy.yml`
/// (`scripts/write-by-digest-store.sh`), not here.
pub(crate) fn run(repo_root: &Path, out_dir: &Path) -> Result<()> {
    let started = Instant::now();
    let bundles = build_all(repo_root)?;
    let index = render_index(&bundles, git_head(repo_root).as_deref())?;
    let written = publish::write_registry(out_dir, &bundles, &index, false)?;

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
        if written.stale_removed > 0 {
            format!("; removed {} stale bundle(s)", written.stale_removed)
        } else {
            String::new()
        }
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use serde_json::{json, Value};
    use trust_tasks_build::publish::{canonical_json, collect_property_owners, BUNDLE_FILE};
    use trust_tasks_build::sha256_digest;
    use walkdir::WalkDir;

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

    /// The JSON value a pointer from the bundle root names.
    fn at<'a>(bundle: &'a Value, pointer: &str) -> &'a Value {
        bundle
            .pointer(pointer)
            .unwrap_or_else(|| panic!("{pointer} names nothing in the bundle"))
    }

    /// `propertyOrder` is the authored order — of the payload schema's own
    /// members, and of a definition inlined from a shared file — and names
    /// exactly the members of the object each pointer names.
    ///
    /// The authored order is checked against the *text* of each source file
    /// (each member's first appearance after the definition starts), not
    /// through the order-keeping parser the bundle writer uses, so the two
    /// cannot agree by sharing a bug.
    #[test]
    fn property_order_is_the_authored_order() {
        let repo_root = find_repo_root().unwrap();
        let spec = spec("acl/grant", "0.2");
        let built = build_bundle(&spec).unwrap();
        let bundle: Value = serde_json::from_slice(&built.bytes).unwrap();
        let order = bundle["propertyOrder"].as_object().unwrap();

        let names = |pointer: &str| -> Vec<String> {
            order[pointer]
                .as_array()
                .unwrap_or_else(|| panic!("no propertyOrder entry {pointer}"))
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect()
        };
        let in_text_order = |text: &str, from: usize, members: &[String]| {
            let mut last = from;
            for name in members {
                let at = text[last..]
                    .find(&format!("\"{name}\""))
                    .unwrap_or_else(|| panic!("{name} does not follow offset {last}"));
                last += at + 1;
            }
        };

        // The payload schema's own members, which key-sorting would reorder
        // (`entry`, `ext`, `reason`).
        assert_eq!(names("/schema"), ["entry", "reason", "ext"]);
        assert_eq!(names("/schema/$defs/Response"), ["entry", "ext"]);

        // A definition spliced in from `acl/_shared/0.2/acl-entry.schema.json`.
        let shared_path = repo_root.join("specs/acl/_shared/0.2/acl-entry.schema.json");
        let shared = fs::read_to_string(&shared_path).unwrap();
        let acl_entry = names("/schema/$defs/AclEntry");
        let mut sorted = acl_entry.clone();
        sorted.sort();
        assert_ne!(acl_entry, sorted, "AclEntry's authored order is not sorted");
        let def_start = shared.find("\"AclEntry\"").unwrap();
        let props_start = def_start + shared[def_start..].find("\"properties\"").unwrap();
        in_text_order(&shared, props_start, &acl_entry);

        // Every pointer names an object whose `properties` are exactly the
        // listed names, and the response document's inlined `$defs` carry the
        // same order as the request's.
        for (pointer, listed) in order {
            let mut listed: Vec<&str> = listed
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect();
            let mut actual: Vec<&str> = at(&bundle, pointer)["properties"]
                .as_object()
                .unwrap_or_else(|| panic!("{pointer} has no properties"))
                .keys()
                .map(String::as_str)
                .collect();
            listed.sort_unstable();
            actual.sort_unstable();
            assert_eq!(listed, actual, "{pointer}");
            if let Some(rest) = pointer.strip_prefix("/responseSchema/$defs/") {
                assert_eq!(order[pointer], order[&format!("/schema/$defs/{rest}")]);
            }
        }
        // And every object with `properties` in either schema has an entry.
        let mut owners = Vec::new();
        collect_property_owners(&bundle["schema"], &mut Vec::new(), &mut owners);
        assert_eq!(
            owners.len(),
            order.keys().filter(|k| k.starts_with("/schema")).count()
        );
    }
}
