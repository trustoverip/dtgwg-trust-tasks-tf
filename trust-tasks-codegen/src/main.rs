//! Generate Rust payload types for `trust-tasks-rs` from the `specs/` registry.
//!
//! Walks `<repo>/specs/<slug>/<version>/payload.schema.json`, runs each schema
//! through `typify`, and writes one Rust module per (slug, version) into
//! `<repo>/trust-tasks-rs/src/specs/`.
//!
//! Run from anywhere in the workspace, then `cargo fmt` to align the
//! generated output with rustfmt (prettyplease and rustfmt disagree on
//! some line-wrap decisions, and `cargo fmt --check` is enforced in CI):
//!
//! ```sh
//! cargo run -p trust-tasks-codegen
//! cargo fmt
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

    /// Path to the adjacent `spec.md`.
    fn spec_md_path(&self) -> PathBuf {
        self.schema_path
            .parent()
            .expect("schema path has a parent")
            .join("spec.md")
    }
}

/// JSON code-fence examples harvested from a spec's `spec.md`. Empty Vecs
/// mean no examples in that section.
#[derive(Debug, Default)]
struct SpecExamples {
    request: Vec<String>,
    response: Vec<String>,
}

/// Scan a `spec.md`'s YAML front matter for `bearer: true`. Returns
/// `false` when the field is absent, set to `false`, or the file is
/// missing. Per SPEC.md §4.8.3 and §7.3 item 12, the default is non-bearer.
fn read_bearer_flag(spec_md_path: &Path) -> Result<bool> {
    if !spec_md_path.exists() {
        return Ok(false);
    }
    let text = fs::read_to_string(spec_md_path)
        .with_context(|| format!("read {}", spec_md_path.display()))?;

    // Front matter is the first `---`-delimited block at the top of the file.
    let mut lines = text.lines();
    let first = lines.next().unwrap_or("");
    if first.trim() != "---" {
        return Ok(false);
    }
    let mut front_matter = String::new();
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        front_matter.push_str(line);
        front_matter.push('\n');
    }

    let value: serde_yaml::Value = serde_yaml::from_str(&front_matter)
        .with_context(|| format!("parse YAML front matter in {}", spec_md_path.display()))?;
    Ok(value
        .get("bearer")
        .and_then(|v| v.as_bool())
        .unwrap_or(false))
}

/// Spec.md sections often embed illustrative `trust-task-error` responses
/// next to the request/response examples. Drop any harvested example whose
/// top-level `type` does not match this spec's URI — that way the
/// conformance tests only deserialize documents the generated types were
/// actually meant to accept.
fn filter_examples_to_this_spec(spec: &Spec, examples: &mut SpecExamples) {
    let request_uri = spec.type_uri();
    let request_uri_with_frag = format!("{request_uri}#request");
    let response_uri = format!("{request_uri}#response");

    examples.request.retain(|json| match example_type(json) {
        Some(t) => t == request_uri || t == request_uri_with_frag,
        // No `type` field at all → an abbreviated payload-only illustration
        // (e.g. "Compound filter" in acl/list). Not a conformance candidate.
        None => false,
    });
    examples.response.retain(|json| match example_type(json) {
        Some(t) => t == response_uri,
        None => false,
    });
}

fn example_type(json: &str) -> Option<String> {
    let v: Value = serde_json::from_str(json).ok()?;
    v.get("type")?.as_str().map(str::to_string)
}

/// Scan a `spec.md` for `## Request` / `## Response` headings and collect
/// the `\`\`\`json … \`\`\`` blocks under each.
fn extract_examples(spec_md_path: &Path) -> Result<SpecExamples> {
    if !spec_md_path.exists() {
        return Ok(SpecExamples::default());
    }
    let text = fs::read_to_string(spec_md_path)
        .with_context(|| format!("read {}", spec_md_path.display()))?;

    #[derive(Clone, Copy, PartialEq)]
    enum Section {
        Outside,
        Request,
        Response,
    }

    let mut section = Section::Outside;
    let mut in_fence = false;
    let mut buf = String::new();
    let mut out = SpecExamples::default();

    for raw_line in text.lines() {
        let trimmed = raw_line.trim_end();
        if in_fence {
            if trimmed.trim_start().starts_with("```") {
                let example = std::mem::take(&mut buf);
                match section {
                    Section::Request => out.request.push(example),
                    Section::Response => out.response.push(example),
                    Section::Outside => {}
                }
                in_fence = false;
            } else {
                buf.push_str(raw_line);
                buf.push('\n');
            }
            continue;
        }
        // Heading transitions only at level 2 (`## …`).
        if let Some(rest) = trimmed.strip_prefix("## ") {
            let h = rest.trim();
            section = if h.eq_ignore_ascii_case("Request") {
                Section::Request
            } else if h.eq_ignore_ascii_case("Response") {
                Section::Response
            } else {
                Section::Outside
            };
            continue;
        }
        // Code fence open — only JSON fences in Request/Response sections matter.
        if section != Section::Outside {
            let lang = trimmed.trim_start();
            if let Some(after) = lang.strip_prefix("```") {
                if after.trim().eq_ignore_ascii_case("json") {
                    in_fence = true;
                    buf.clear();
                }
            }
        }
    }
    Ok(out)
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
/// `Cargo.toml` (the one that declares a `[workspace]` table). The
/// previous implementation matched the literal substring `[workspace]`,
/// which would also match a doc comment inside an unrelated `Cargo.toml`
/// (e.g. a vendored dependency) — combined with the destructive
/// [`clean_generated_tree`] this could produce out-of-repo writes. We
/// parse as TOML now and check for the table proper.
fn find_repo_root() -> Result<PathBuf> {
    let start = std::env::current_dir()?;
    for ancestor in start.ancestors() {
        let candidate = ancestor.join("Cargo.toml");
        if !candidate.is_file() {
            continue;
        }
        let text = fs::read_to_string(&candidate)?;
        let parsed: toml::Value = match text.parse() {
            Ok(v) => v,
            Err(_) => continue, // malformed Cargo.toml — skip, keep walking
        };
        if parsed.get("workspace").and_then(|w| w.as_table()).is_some() {
            return Ok(ancestor.to_path_buf());
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
///
/// Refuses to operate on symlinks: `fs::remove_dir_all` on a symlink would
/// happily follow it and delete the *target*. The codegen tree is supposed
/// to contain only regular files and dirs the codegen itself produced; if
/// a developer's checkout has a symlink in there, we'd rather fail than
/// nuke whatever it points at.
fn clean_generated_tree(out_root: &Path) -> Result<()> {
    if !out_root.exists() {
        fs::create_dir_all(out_root)?;
        return Ok(());
    }
    let root_meta = fs::symlink_metadata(out_root)?;
    if root_meta.file_type().is_symlink() {
        return Err(anyhow!(
            "{} is a symlink; refusing to clean it",
            out_root.display()
        ));
    }
    for entry in fs::read_dir(out_root)? {
        let entry = entry?;
        let path = entry.path();
        if path.file_name().and_then(|s| s.to_str()) == Some("mod.rs")
            && !entry.file_type()?.is_symlink()
        {
            continue;
        }
        let meta = fs::symlink_metadata(&path)?;
        if meta.file_type().is_symlink() {
            return Err(anyhow!(
                "{} is a symlink; refusing to remove it",
                path.display()
            ));
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
    let mut examples = extract_examples(&spec.spec_md_path())?;
    filter_examples_to_this_spec(spec, &mut examples);
    let is_bearer = read_bearer_flag(&spec.spec_md_path())?;
    let module_tokens = render_module(spec, body, has_response, &examples, is_bearer, &raw);

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

fn render_module(
    spec: &Spec,
    body: TokenStream,
    has_response: bool,
    examples: &SpecExamples,
    is_bearer: bool,
    schema_json: &str,
) -> TokenStream {
    let type_uri = spec.type_uri();
    let response_uri = format!("{type_uri}#response");
    let slug_doc = format!(" Spec slug: `{}`. Version: `{}`.", spec.slug, spec.version);

    // Per SPEC §4.8.3, only bearer specs override Payload::IS_BEARER. Non-
    // bearer specs (the default) leave the trait's default `false` in place.
    let bearer_const = if is_bearer {
        quote! { const IS_BEARER: bool = true; }
    } else {
        quote! {}
    };

    let response_payload_impl = if has_response {
        quote! {
            impl crate::Payload for Response {
                const TYPE_URI: &'static str = #response_uri;
                #bearer_const
            }
        }
    } else {
        quote! {}
    };

    let validate_request_impl = quote! {
        #[cfg(feature = "validate")]
        impl crate::validate::ValidatedPayload for Payload {
            const SCHEMA_JSON: &'static str = #schema_json;
        }
    };

    let conformance_mod = render_conformance_mod(examples, has_response);

    quote! {
        //! Generated by `trust-tasks-codegen` — do not edit by hand.
        //!
        #![doc = #slug_doc]

        #[allow(unused_imports)]
        use serde::{Deserialize, Serialize};

        #body

        impl crate::Payload for Payload {
            const TYPE_URI: &'static str = #type_uri;
            #bearer_const
        }

        #response_payload_impl

        #validate_request_impl

        #conformance_mod
    }
}

/// Emit `#[cfg(test)] mod conformance` with one test per harvested example.
/// Each test deserializes the JSON into a `TrustTask<Payload>` (or `Response`)
/// and asserts the wire form round-trips.
fn render_conformance_mod(examples: &SpecExamples, has_response: bool) -> TokenStream {
    let request_tests: Vec<TokenStream> = examples
        .request
        .iter()
        .enumerate()
        .map(|(i, json)| {
            let fn_name = quote::format_ident!("request_example_{}", i + 1);
            quote! {
                #[test]
                fn #fn_name() {
                    const JSON: &str = #json;
                    let doc: crate::TrustTask<super::Payload> =
                        serde_json::from_str(JSON).expect("deserialize request example");
                    let rendered = serde_json::to_value(&doc).expect("re-serialize");
                    let expected: serde_json::Value =
                        serde_json::from_str(JSON).expect("re-parse expected");
                    assert_eq!(rendered, expected, "request example failed round-trip");
                }
            }
        })
        .collect();

    let response_tests: Vec<TokenStream> = if has_response {
        examples
            .response
            .iter()
            .enumerate()
            .map(|(i, json)| {
                let fn_name = quote::format_ident!("response_example_{}", i + 1);
                quote! {
                    #[test]
                    fn #fn_name() {
                        const JSON: &str = #json;
                        let doc: crate::TrustTask<super::Response> =
                            serde_json::from_str(JSON).expect("deserialize response example");
                        let rendered = serde_json::to_value(&doc).expect("re-serialize");
                        let expected: serde_json::Value =
                            serde_json::from_str(JSON).expect("re-parse expected");
                        assert_eq!(rendered, expected, "response example failed round-trip");
                    }
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    if request_tests.is_empty() && response_tests.is_empty() {
        return quote! {};
    }

    quote! {
        #[cfg(test)]
        mod conformance {
            //! Round-trip tests harvested from the spec's `spec.md`.

            #(#request_tests)*
            #(#response_tests)*
        }
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
