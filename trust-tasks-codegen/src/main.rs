//! Generate Rust payload types for `trust-tasks-rs` from the `specs/` registry.
//!
//! Walks `<repo>/specs/<slug>/<version>/payload.schema.json`, runs each schema
//! through `typify`, and writes one Rust module per (slug, version) into
//! `<repo>/trust-tasks-rs/src/specs/`.
//!
//! Run from anywhere in the workspace:
//!
//! ```sh
//! cargo run -p trust-tasks-codegen
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use proc_macro2::TokenStream;
use quote::quote;
use serde_json::{json, Value};
use walkdir::WalkDir;

/// Slug of the framework error spec — modelled by hand in
/// `trust_tasks_rs::ErrorPayload`, so we never emit a generated version.
const SKIP_SLUGS: &[&str] = &["trust-task-error"];

/// One spec to generate for.
#[derive(Debug)]
struct Spec {
    /// e.g. `"acl/grant"` (the slug exactly as it appears in the spec).
    slug: String,
    /// e.g. `"0.1"`.
    version: String,
    /// Path to `payload.schema.json`.
    schema_path: PathBuf,
}

impl Spec {
    /// `["acl", "grant"]` (filesystem path segments — no normalization).
    fn slug_segments(&self) -> Vec<&str> {
        self.slug.split('/').collect()
    }

    /// `["acl", "grant"]` with hyphens converted to underscores so each
    /// segment is a valid Rust identifier (e.g. `change-role` → `change_role`).
    fn module_segments(&self) -> Vec<String> {
        self.slug_segments()
            .iter()
            .map(|s| s.replace('-', "_"))
            .collect()
    }

    /// `"v0_1"` — the leaf module name encoding the version.
    fn version_module(&self) -> String {
        format!("v{}", self.version.replace('.', "_"))
    }

    /// Full Type URI for the request variant.
    fn type_uri(&self) -> String {
        format!("https://trusttasks.org/spec/{}/{}", self.slug, self.version)
    }
}

fn main() -> Result<()> {
    let repo_root = find_repo_root()?;
    let specs_dir = repo_root.join("specs");
    let out_root = repo_root.join("trust-tasks-rs/src/specs");

    let specs = discover_specs(&specs_dir)?;
    if specs.is_empty() {
        return Err(anyhow!("no specs found under {}", specs_dir.display()));
    }

    println!("Discovered {} specs:", specs.len());
    for s in &specs {
        println!("  {}/{}", s.slug, s.version);
    }

    // Wipe any previously generated modules so removals propagate.
    clean_generated_tree(&out_root)?;

    for spec in &specs {
        generate_one(spec, &out_root).with_context(|| {
            format!("failed to generate code for {}/{}", spec.slug, spec.version)
        })?;
    }

    write_mod_tree(&specs, &out_root)?;

    println!(
        "\nGenerated {} payload modules into {}",
        specs.len(),
        out_root.display()
    );
    Ok(())
}

/// Locate the repo root by walking up from CWD looking for the workspace
/// `Cargo.toml` (the one with `[workspace]`).
fn find_repo_root() -> Result<PathBuf> {
    let start = std::env::current_dir()?;
    for ancestor in start.ancestors() {
        let candidate = ancestor.join("Cargo.toml");
        if candidate.is_file() {
            let text = fs::read_to_string(&candidate)?;
            if text.contains("[workspace]") {
                return Ok(ancestor.to_path_buf());
            }
        }
    }
    Err(anyhow!(
        "could not find a workspace Cargo.toml above {}",
        start.display()
    ))
}

fn discover_specs(specs_dir: &Path) -> Result<Vec<Spec>> {
    let mut out = Vec::new();
    for entry in WalkDir::new(specs_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() != "payload.schema.json" {
            continue;
        }
        let version_dir = entry.path().parent().ok_or_else(|| {
            anyhow!(
                "payload.schema.json with no parent: {}",
                entry.path().display()
            )
        })?;
        let slug_dir = version_dir
            .parent()
            .ok_or_else(|| anyhow!("no slug dir for {}", entry.path().display()))?;

        let version = version_dir
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("non-utf8 version dir: {}", version_dir.display()))?
            .to_string();
        let slug = slug_dir
            .strip_prefix(specs_dir)
            .with_context(|| format!("slug dir {} is not under specs/", slug_dir.display()))?
            .to_string_lossy()
            .replace('\\', "/");

        if SKIP_SLUGS.contains(&slug.as_str()) {
            continue;
        }

        out.push(Spec {
            slug,
            version,
            schema_path: entry.path().to_path_buf(),
        });
    }
    out.sort_by(|a, b| (a.slug.as_str(), a.version.as_str()).cmp(&(&b.slug, &b.version)));
    Ok(out)
}

/// Remove every previously-generated module file/dir under `out_root` except
/// the `mod.rs` and any non-generated siblings (there shouldn't be any).
fn clean_generated_tree(out_root: &Path) -> Result<()> {
    if !out_root.exists() {
        fs::create_dir_all(out_root)?;
        return Ok(());
    }
    for entry in fs::read_dir(out_root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.file_name().and_then(|s| s.to_str()) == Some("mod.rs") {
            continue;
        }
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

fn generate_one(spec: &Spec, out_root: &Path) -> Result<()> {
    let raw = fs::read_to_string(&spec.schema_path)?;
    let mut schema: Value = serde_json::from_str(&raw)
        .with_context(|| format!("parse {}", spec.schema_path.display()))?;

    let has_response = normalize_titles(&mut schema)?;
    // typify (0.5) expects Draft-07 `definitions` rather than 2020-12 `$defs`.
    migrate_defs_to_definitions(&mut schema);

    // Round-trip the normalized schema into the schemars representation
    // typify works with.
    let normalized_text = serde_json::to_string(&schema)?;
    let root_schema: schemars::schema::RootSchema = serde_json::from_str(&normalized_text)?;

    let settings = typify::TypeSpaceSettings::default();
    let mut type_space = typify::TypeSpace::new(&settings);
    type_space
        .add_root_schema(root_schema)
        .with_context(|| format!("typify: add_root_schema for {}/{}", spec.slug, spec.version))?;

    let body = type_space.to_stream();
    let module_tokens = render_module(spec, body, has_response);

    let parsed: syn::File = syn::parse2(module_tokens.clone()).with_context(|| {
        format!(
            "failed to parse generated tokens for {}/{}:\n{}",
            spec.slug, spec.version, module_tokens
        )
    })?;
    let formatted = prettyplease::unparse(&parsed);

    let mut path = out_root.to_path_buf();
    for seg in spec.module_segments() {
        path = path.join(seg);
    }
    fs::create_dir_all(&path)?;
    let leaf = path.join(format!("{}.rs", spec.version_module()));
    fs::write(&leaf, formatted)?;
    Ok(())
}

/// Rename the 2020-12 `$defs` keyword to the Draft-07 `definitions` keyword
/// recursively, and rewrite every `$ref` of the form `#/$defs/X` to
/// `#/definitions/X`. typify 0.5 only honors the older keyword; the two are
/// semantically equivalent for our purposes.
fn migrate_defs_to_definitions(schema: &mut Value) {
    match schema {
        Value::Object(map) => {
            if let Some(defs) = map.remove("$defs") {
                map.insert("definitions".to_string(), defs);
            }
            if let Some(Value::String(s)) = map.get_mut("$ref") {
                if let Some(stripped) = s.strip_prefix("#/$defs/") {
                    *s = format!("#/definitions/{stripped}");
                }
            }
            for v in map.values_mut() {
                migrate_defs_to_definitions(v);
            }
        }
        Value::Array(items) => {
            for v in items {
                migrate_defs_to_definitions(v);
            }
        }
        _ => {}
    }
}

/// Rewrite schema titles so typify produces clean type names:
/// `Payload` for the root, `Response` for the response sub-schema. Returns
/// whether a response anchor was present.
fn normalize_titles(schema: &mut Value) -> Result<bool> {
    let obj = schema
        .as_object_mut()
        .ok_or_else(|| anyhow!("schema root must be an object"))?;
    obj.insert("title".into(), json!("Payload"));

    let mut has_response = false;
    if let Some(defs) = obj.get_mut("$defs").and_then(Value::as_object_mut) {
        if let Some(resp) = defs.get_mut("Response").and_then(Value::as_object_mut) {
            resp.insert("title".into(), json!("Response"));
            has_response = true;
        }
    }
    Ok(has_response)
}

fn render_module(spec: &Spec, body: TokenStream, has_response: bool) -> TokenStream {
    let type_uri = spec.type_uri();
    let response_uri = format!("{type_uri}#response");
    let slug_doc = format!(" Spec slug: `{}`. Version: `{}`.", spec.slug, spec.version);

    let response_impl = if has_response {
        quote! {
            impl crate::Payload for Response {
                const TYPE_URI: &'static str = #response_uri;
            }
        }
    } else {
        quote! {}
    };

    quote! {
        //! Generated by `trust-tasks-codegen` — do not edit by hand.
        //!
        #![doc = #slug_doc]

        #[allow(unused_imports)]
        use serde::{Deserialize, Serialize};

        #body

        impl crate::Payload for Payload {
            const TYPE_URI: &'static str = #type_uri;
        }

        #response_impl
    }
}

#[derive(Default)]
struct ModNode {
    children: BTreeMap<String, ModNode>,
    /// Version module names at this level, e.g. `["v0_1"]`.
    leaves: Vec<String>,
}

/// Write `mod.rs` files at every level of the generated tree.
fn write_mod_tree(specs: &[Spec], out_root: &Path) -> Result<()> {
    let mut root = ModNode::default();
    for spec in specs {
        let mut cursor = &mut root;
        for seg in spec.module_segments() {
            cursor = cursor.children.entry(seg).or_default();
        }
        cursor.leaves.push(spec.version_module());
    }

    write_node(&root, out_root, true)?;
    Ok(())
}

fn write_node(node: &ModNode, dir: &Path, is_root: bool) -> Result<()> {
    let mut decls: Vec<String> = node
        .children
        .keys()
        .map(|n| format!("pub mod {n};"))
        .chain(node.leaves.iter().map(|n| format!("pub mod {n};")))
        .collect();
    decls.sort();
    let body = decls.join("\n");

    if is_root {
        // Preserve the hand-written preamble in specs/mod.rs; only the block
        // between the codegen markers is regenerated.
        let path = dir.join("mod.rs");
        let existing = fs::read_to_string(&path).unwrap_or_default();
        let updated = replace_between_markers(&existing, &body);
        fs::write(&path, updated)?;
    } else {
        fs::create_dir_all(dir)?;
        let path = dir.join("mod.rs");
        let header = "//! Generated by `trust-tasks-codegen` — do not edit by hand.\n\n";
        let content = format!("{header}{body}\n");
        fs::write(&path, content)?;
    }

    for (name, child) in &node.children {
        write_node(child, &dir.join(name), false)?;
    }
    Ok(())
}

const BEGIN_MARKER: &str = "// trust-tasks-codegen:begin";
const END_MARKER: &str = "// trust-tasks-codegen:end";

fn replace_between_markers(existing: &str, body: &str) -> String {
    match (existing.find(BEGIN_MARKER), existing.find(END_MARKER)) {
        (Some(b), Some(e)) if e > b => {
            let before = &existing[..b + BEGIN_MARKER.len()];
            let after = &existing[e..];
            format!("{before}\n{body}\n{after}")
        }
        _ => format!("{existing}\n{BEGIN_MARKER}\n{body}\n{END_MARKER}\n"),
    }
}
