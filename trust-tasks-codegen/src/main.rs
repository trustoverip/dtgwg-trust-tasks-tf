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

/// Read `payload.invalid-examples.json` next to the spec's
/// `payload.schema.json`, if present.
///
/// The file's top-level shape is a JSON array. Each entry is an object
/// with two members:
///
/// * `note` — a short human-readable description of the bug class this
///   fixture exemplifies (rendered into the generated test's failure
///   message so reviewers can see *why* a fixture was meant to fail
///   when one accidentally starts passing).
/// * `payload` — the deliberately non-conforming payload JSON, which
///   is what `rejects_invalid_examples` actually feeds to the
///   `Payload` parser and the schema validator.
///
/// Returns an empty Vec when the file does not exist — the test
/// emission below treats an empty list as "no test to emit", not as a
/// conformance failure.
fn read_invalid_examples(spec: &Spec) -> Result<Vec<InvalidExample>> {
    let path = spec
        .schema_path
        .parent()
        .ok_or_else(|| anyhow!("schema path has no parent"))?
        .join("payload.invalid-examples.json");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let array: Value =
        serde_json::from_str(&raw).with_context(|| format!("parse {} as JSON", path.display()))?;
    let items = array
        .as_array()
        .ok_or_else(|| anyhow!("{} must contain a top-level JSON array", path.display()))?;
    let mut out = Vec::with_capacity(items.len());
    for (i, item) in items.iter().enumerate() {
        let obj = item.as_object().ok_or_else(|| {
            anyhow!(
                "{}[{}] must be an object with `note` and `payload` members",
                path.display(),
                i
            )
        })?;
        let note = obj
            .get("note")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("{}[{}].note must be a string", path.display(), i))?
            .to_string();
        let payload = obj
            .get("payload")
            .cloned()
            .ok_or_else(|| anyhow!("{}[{}].payload is missing", path.display(), i))?;
        let payload_json = serde_json::to_string_pretty(&payload)?;
        out.push(InvalidExample { note, payload_json });
    }
    Ok(out)
}

/// One entry from `payload.invalid-examples.json`. See [`read_invalid_examples`].
#[derive(Debug)]
struct InvalidExample {
    note: String,
    payload_json: String,
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

/// The `proof` requirement each document variant carries, per SPEC.md §7.3
/// item 8.
///
/// The declaration is either a single `requirement` covering every variant, or
/// a per-variant `request` / `response` pair. Both are normalised here so the
/// emitter never branches.
#[derive(Debug, Clone, Copy, Default)]
struct ProofRequired {
    request: bool,
    response: bool,
}

/// Scan a `spec.md`'s YAML front matter for the §7.3 item 8 proof declaration.
///
/// Only `REQUIRED` obliges a consumer to reject a proofless document, so each
/// variant reduces to a bool. Returns both `false` when the field is absent or
/// the file is missing. A per-variant declaration that omits `response` takes
/// the request's value — the conservative reading, and the only one that cannot
/// weaken a variant by omission.
fn read_proof_required_flag(spec_md_path: &Path) -> Result<ProofRequired> {
    if !spec_md_path.exists() {
        return Ok(ProofRequired::default());
    }
    let text = fs::read_to_string(spec_md_path)
        .with_context(|| format!("read {}", spec_md_path.display()))?;

    let mut lines = text.lines();
    let first = lines.next().unwrap_or("");
    if first.trim() != "---" {
        return Ok(ProofRequired::default());
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
    let pr = match value.get("proofRequirement") {
        Some(v) => v,
        None => return Ok(ProofRequired::default()),
    };
    let level = |key: &str| {
        pr.get(key)
            .and_then(|v| v.as_str())
            .map(|s| s == "REQUIRED")
    };

    // Single-value form applies to every variant.
    if let Some(all) = level("requirement") {
        return Ok(ProofRequired {
            request: all,
            response: all,
        });
    }
    let request = level("request").unwrap_or(false);
    Ok(ProofRequired {
        request,
        response: level("response").unwrap_or(request),
    })
}

/// Read whether the party filling the framework `member` (`"issuer"` or
/// `"recipient"`) is declared `requirement: REQUIRED` in the spec's front
/// matter. Returns `false` when no party carries that `member`, when its
/// requirement is `RECOMMENDED` / `OPTIONAL`, or when the file is missing. Per
/// SPEC.md §7.2 item 5 / §7.3 item 5, only `REQUIRED` obliges the consumer to
/// reject a document lacking that member in-band.
fn read_member_required_flag(spec_md_path: &Path, member: &str) -> Result<bool> {
    if !spec_md_path.exists() {
        return Ok(false);
    }
    let text = fs::read_to_string(spec_md_path)
        .with_context(|| format!("read {}", spec_md_path.display()))?;
    let mut lines = text.lines();
    if lines.next().unwrap_or("").trim() != "---" {
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
    let Some(parties) = value.get("parties").and_then(|v| v.as_sequence()) else {
        return Ok(false);
    };
    Ok(parties.iter().any(|p| {
        p.get("member").and_then(|v| v.as_str()) == Some(member)
            && p.get("requirement").and_then(|v| v.as_str()) == Some("REQUIRED")
    }))
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

    // Generate everything before deleting anything. `clean_generated_tree` is
    // destructive and `generate_one` can fail on malformed registry input, so
    // doing the clean first meant one bad spec wiped the tree and left the
    // workspace uncompilable — with the error surfacing later, somewhere else,
    // as an unresolvable module.
    let mut modules = Vec::with_capacity(specs.len());
    for spec in &specs {
        let generated = generate_one(spec, &out_root).with_context(|| {
            format!("failed to generate code for {}/{}", spec.slug, spec.version)
        })?;
        modules.push(generated);
    }

    // Every spec generated. Now it is safe to wipe, so removals propagate.
    clean_generated_tree(&out_root)?;

    for (path, contents) in &modules {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, contents)?;
    }

    write_mod_tree(&specs, &out_root)?;
    write_schema_index(&specs, &repo_root)?;

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
        // Parse the whole document into a table. `toml::Value`'s `FromStr`
        // parses a bare value expression (not a document) under toml 1.x,
        // so a full Cargo.toml must go through `from_str::<Table>`.
        let parsed: toml::Table = match toml::from_str(&text) {
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

/// Generate one spec's module, returning the path to write it to and its
/// contents — **without touching the filesystem**.
///
/// Deliberately pure. [`main`] collects every module before
/// [`clean_generated_tree`] runs, so a spec that fails to generate leaves the
/// existing tree exactly as it was. It used to write as it went, after the
/// clean: one malformed `payload.invalid-examples.json` then left 300+ files
/// deleted and the workspace uncompilable, and the only symptom was a
/// Make multi-paragraph `description`s safe for rustdoc.
///
/// typify emits a description as `/**{text}` — the first line flush against the
/// opening delimiter, every continuation line carrying the indentation of
/// whatever item it documents. rustdoc strips only the whitespace *common* to
/// all lines of a doc comment, and the un-indented first line makes that common
/// prefix empty. A field four spaces deep therefore has its second and later
/// paragraphs read as **indented code blocks**, which rustdoc then compiles as
/// Rust doctests and fails on.
///
/// It bit `trust-task-next-step/0.1` and again `trust-ceremony-receipt/0.1`,
/// each time as a `cargo test` failure pointing at generated code rather than at
/// the schema that caused it.
///
/// The fix is to start such a description on its own line. Every line then
/// carries the item's indentation, the common prefix is non-empty, rustdoc
/// strips it, and the paragraphs render as prose. Applied only where a blank
/// line exists, so single-paragraph descriptions are untouched.
fn indent_safe_descriptions(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(desc)) = map.get_mut("description") {
                if desc.contains("\n\n") && !desc.starts_with('\n') {
                    desc.insert(0, '\n');
                }
            }
            for (_, v) in map.iter_mut() {
                indent_safe_descriptions(v);
            }
        }
        Value::Array(items) => {
            for v in items.iter_mut() {
                indent_safe_descriptions(v);
            }
        }
        _ => {}
    }
}

/// downstream `cargo fmt` error about an unresolvable module — pointing
/// nowhere near the file at fault.
fn generate_one(spec: &Spec, out_root: &Path) -> Result<(PathBuf, String)> {
    let mut schema: Value = serde_json::from_str(&fs::read_to_string(&spec.schema_path)?)
        .with_context(|| format!("parse {}", spec.schema_path.display()))?;

    // Resolve any cross-file `$ref`s (e.g. into _shared/ or _framework/)
    // by inlining the referenced `$def` into this schema's local `$defs`.
    // Done before typify sees the schema, and the result becomes the
    // SCHEMA_JSON constant so runtime ValidatedPayload::validate_value
    // does not need a network-style resolver either.
    let base_dir = spec
        .schema_path
        .parent()
        .ok_or_else(|| anyhow!("schema path has no parent"))?
        .to_path_buf();
    resolve_cross_file_refs(&mut schema, &base_dir).with_context(|| {
        format!(
            "failed to resolve cross-file $refs for {}/{}",
            spec.slug, spec.version
        )
    })?;

    // The inlined, self-contained schema is now the on-the-wire SCHEMA_JSON.
    let raw = serde_json::to_string_pretty(&schema)? + "\n";

    // After `raw` is captured: this rewrites descriptions for rustdoc's benefit
    // only, and SCHEMA_JSON must keep the descriptions the registry publishes.
    indent_safe_descriptions(&mut schema);

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
    let invalid_examples = read_invalid_examples(spec)?;
    let is_bearer = read_bearer_flag(&spec.spec_md_path())?;
    let is_proof_required = read_proof_required_flag(&spec.spec_md_path())?;
    // Request `recipient` member tracks the recipient party; the response swaps
    // parties, so its `recipient` member tracks the issuer party (§7.2 item 5).
    let recipient_required = read_member_required_flag(&spec.spec_md_path(), "recipient")?;
    let issuer_required = read_member_required_flag(&spec.spec_md_path(), "issuer")?;
    let module_tokens = render_module(
        spec,
        body,
        has_response,
        &examples,
        &invalid_examples,
        is_bearer,
        is_proof_required,
        recipient_required,
        issuer_required,
        &raw,
    );

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
    Ok((
        path.join(format!("{}.rs", spec.version_module())),
        formatted,
    ))
}

/// Resolve `$ref` strings of the form `<relative-path>#/$defs/<name>` by
/// loading the referenced JSON file and splicing its `$defs.<name>` into
/// the current schema's `$defs.<name>`, then rewriting the `$ref` to the
/// now-local form `#/$defs/<name>`.
///
/// Handles transitive refs (e.g. acl-entry.schema.json's AclEntry contains
/// a $ref to framework.schema.json's Ext) by recursively resolving the
/// spliced fragment against the directory it came from.
///
/// Local `#/$defs/…` and `#/…` refs are left untouched.
fn resolve_cross_file_refs(schema: &mut Value, base_dir: &Path) -> Result<()> {
    use std::collections::HashSet;

    let mut frontier: Vec<(Value, PathBuf)> = collect_external_refs(schema, base_dir);
    let mut seen: HashSet<String> = HashSet::new();
    while let Some((ref_value, owner_dir)) = frontier.pop() {
        let ref_str = ref_value
            .as_str()
            .ok_or_else(|| anyhow!("$ref value was not a string"))?
            .to_string();
        let (rel_path, def_name) = split_external_ref(&ref_str).ok_or_else(|| {
            anyhow!("external $ref {ref_str:?} is not of the form <path>#/$defs/<name>")
        })?;
        let abs_path = owner_dir.join(rel_path);
        let abs_path_canonical = fs::canonicalize(&abs_path).with_context(|| {
            format!(
                "$ref target {} (from {}) does not exist",
                abs_path.display(),
                owner_dir.display()
            )
        })?;
        let dedupe_key = format!("{}#/$defs/{}", abs_path_canonical.display(), def_name);
        if !seen.insert(dedupe_key) {
            continue;
        }
        let referenced: Value = serde_json::from_str(&fs::read_to_string(&abs_path_canonical)?)
            .with_context(|| format!("parse referenced schema {}", abs_path_canonical.display()))?;
        let fragment = referenced
            .get("$defs")
            .and_then(|v| v.get(def_name))
            .ok_or_else(|| {
                anyhow!(
                    "{} has no $defs/{} (referenced from {})",
                    abs_path_canonical.display(),
                    def_name,
                    owner_dir.display()
                )
            })?
            .clone();

        // Splice the fragment into the local schema's $defs.<name>.
        let defs = schema
            .as_object_mut()
            .ok_or_else(|| anyhow!("schema root must be an object"))?
            .entry("$defs")
            .or_insert_with(|| Value::Object(Default::default()))
            .as_object_mut()
            .ok_or_else(|| anyhow!("$defs must be an object"))?;
        if let Some(existing) = defs.get(def_name) {
            if existing != &fragment {
                return Err(anyhow!(
                    "schema already defines $defs/{def_name} with a different shape; \
                     cross-file $ref splice would overwrite it"
                ));
            }
        } else {
            defs.insert(def_name.to_string(), fragment.clone());
        }

        // The fragment's own external refs (transitive) are resolved
        // against the file it came from, not the original base_dir.
        let referenced_dir = abs_path_canonical
            .parent()
            .ok_or_else(|| anyhow!("referenced file has no parent dir"))?
            .to_path_buf();
        frontier.extend(collect_external_refs(&fragment, &referenced_dir));

        // The fragment may also contain *internal* refs (`#/$defs/X`)
        // pointing at sibling defs in the source file. Without
        // splicing those too, typify panics with `$ref
        // #/definitions/X is missing` after `migrate_defs_to_definitions`
        // rewrites the references. Reformulate each internal ref as a
        // synthetic external ref against the same source file so the
        // splice path treats them uniformly.
        let synthetic_external: Vec<Value> = collect_internal_refs(&fragment)
            .into_iter()
            .map(|local| Value::String(format!("{}{}", rel_path, local)))
            .collect();
        for synth in synthetic_external {
            frontier.push((synth, owner_dir.clone()));
        }
    }

    // After splicing, rewrite every external $ref string to the local form.
    rewrite_external_refs_local(schema);
    Ok(())
}

/// Walk `schema` and collect every `$ref` whose value is an external (non-
/// `#`-prefixed) reference. Returns `(ref_value, base_dir_for_resolving_it)`.
fn collect_external_refs(schema: &Value, base_dir: &Path) -> Vec<(Value, PathBuf)> {
    let mut out = Vec::new();
    walk_external_refs(schema, base_dir, &mut |r, dir| {
        out.push((r.clone(), dir.to_path_buf()))
    });
    out
}

/// Walk `schema` and collect every internal `$ref` value (those that
/// start with `#`). Returns the raw fragment strings so the caller
/// can rewrite them into synthetic external refs.
fn collect_internal_refs(schema: &Value) -> Vec<String> {
    let mut out = Vec::new();
    walk_internal_refs(schema, &mut |s| out.push(s.to_string()));
    out
}

fn walk_internal_refs(value: &Value, sink: &mut impl FnMut(&str)) {
    match value {
        Value::Object(map) => {
            if let Some(r) = map.get("$ref") {
                if let Some(s) = r.as_str() {
                    if s.starts_with('#') {
                        sink(s);
                    }
                }
            }
            for v in map.values() {
                walk_internal_refs(v, sink);
            }
        }
        Value::Array(items) => {
            for v in items {
                walk_internal_refs(v, sink);
            }
        }
        _ => {}
    }
}

fn walk_external_refs(value: &Value, base_dir: &Path, sink: &mut impl FnMut(&Value, &Path)) {
    match value {
        Value::Object(map) => {
            if let Some(r) = map.get("$ref") {
                if r.as_str().map(|s| !s.starts_with('#')).unwrap_or(false) {
                    sink(r, base_dir);
                }
            }
            for v in map.values() {
                walk_external_refs(v, base_dir, sink);
            }
        }
        Value::Array(items) => {
            for v in items {
                walk_external_refs(v, base_dir, sink);
            }
        }
        _ => {}
    }
}

/// After splicing, replace every external `$ref` string with its local
/// fragment-only form so typify (and downstream validators) see a self-
/// contained schema.
fn rewrite_external_refs_local(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(s)) = map.get_mut("$ref") {
                if !s.starts_with('#') {
                    if let Some((_, def_name)) = split_external_ref(s) {
                        *s = format!("#/$defs/{def_name}");
                    }
                }
            }
            for v in map.values_mut() {
                rewrite_external_refs_local(v);
            }
        }
        Value::Array(items) => {
            for v in items {
                rewrite_external_refs_local(v);
            }
        }
        _ => {}
    }
}

/// Parse `"<relative-path>#/$defs/<name>"` into its two halves.
fn split_external_ref(s: &str) -> Option<(&str, &str)> {
    let hash = s.find('#')?;
    let path = &s[..hash];
    let fragment = &s[hash + 1..];
    let prefix = "/$defs/";
    let def_name = fragment.strip_prefix(prefix)?;
    if path.is_empty() || def_name.is_empty() || def_name.contains('/') {
        return None;
    }
    Some((path, def_name))
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

#[allow(clippy::too_many_arguments)]
fn render_module(
    spec: &Spec,
    body: TokenStream,
    has_response: bool,
    examples: &SpecExamples,
    invalid_examples: &[InvalidExample],
    is_bearer: bool,
    is_proof_required: ProofRequired,
    recipient_required: bool,
    issuer_required: bool,
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

    let response_payload_impl = if has_response {
        quote! {
            impl crate::Payload for Response {
                const TYPE_URI: &'static str = #response_uri;
                #bearer_const
                #resp_proof_const
                #resp_recipient_const
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

    let conformance_mod = render_conformance_mod(examples, invalid_examples, has_response);

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
            #req_proof_const
            #req_recipient_const
        }

        #response_payload_impl

        #validate_request_impl

        #conformance_mod
    }
}

/// Emit `#[cfg(test)] mod conformance` with one test per harvested example.
/// Each test deserializes the JSON into a `TrustTask<Payload>` (or `Response`)
/// and asserts the wire form round-trips.
///
/// When `invalid_examples` is non-empty AND the crate is built with the
/// `validate` feature, an additional `rejects_invalid_examples` test is
/// emitted that asserts each fixture either fails serde deserialization
/// (as a `Payload`) or fails JSON-Schema validation under
/// `ValidatedPayload::validate_value`.
fn render_conformance_mod(
    examples: &SpecExamples,
    invalid_examples: &[InvalidExample],
    has_response: bool,
) -> TokenStream {
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

    let invalid_test = if invalid_examples.is_empty() {
        quote! {}
    } else {
        let fixtures: Vec<TokenStream> = invalid_examples
            .iter()
            .map(|ex| {
                let note = &ex.note;
                let payload_json = &ex.payload_json;
                quote! { (#note, #payload_json) }
            })
            .collect();
        quote! {
            /// Each fixture in `payload.invalid-examples.json` MUST be
            /// rejected by at least one of: serde deserialization, or
            /// JSON-Schema validation under the `validate` feature. The
            /// fixture file documents the producer-side bug class that
            /// each payload exemplifies; this generated test pins it.
            #[cfg(feature = "validate")]
            #[test]
            fn rejects_invalid_examples() {
                use crate::validate::ValidatedPayload;
                let fixtures: &[(&str, &str)] = &[ #(#fixtures),* ];
                for (i, (note, raw)) in fixtures.iter().enumerate() {
                    let value: serde_json::Value = match serde_json::from_str(raw) {
                        Ok(v) => v,
                        // Parse-level rejection — fine, the fixture is invalid wire JSON.
                        Err(_) => continue,
                    };
                    let serde_ok = serde_json::from_value::<super::Payload>(value.clone()).is_ok();
                    let schema_ok = super::Payload::validate_value(&value).is_ok();
                    assert!(
                        !(serde_ok && schema_ok),
                        "invalid-example #{} ({:?}) was accepted by both serde and JSON Schema; \
                         the fixture's stated failure class is no longer caught:\n{}",
                        i + 1, note, raw
                    );
                }
            }
        }
    };

    if request_tests.is_empty() && response_tests.is_empty() && invalid_examples.is_empty() {
        return quote! {};
    }

    quote! {
        #[cfg(test)]
        mod conformance {
            //! Round-trip tests harvested from the spec's `spec.md`,
            //! plus a `rejects_invalid_examples` test for any fixtures
            //! in `payload.invalid-examples.json` (validate feature).

            #(#request_tests)*
            #(#response_tests)*

            #invalid_test
        }
    }
}

#[derive(Default)]
struct ModNode {
    children: BTreeMap<String, ModNode>,
    /// Version module names at this level, e.g. `["v0_1"]`.
    leaves: Vec<String>,
}

/// Emit `trust-tasks-rs/src/schema_index.rs` — a Type URI → payload schema lookup.
///
/// Without this, a consumer that dispatches on a Type URI has no way to *find* the
/// schema for the payload it is about to run: `ValidatedPayload::SCHEMA_JSON` is a
/// per-type associated const, and a generic gate has no type to name. It could
/// only validate by hand-writing a match arm per task and remembering to add one
/// with every new task — which is to say, it would validate whatever somebody
/// remembered, which is not validation.
fn write_schema_index(specs: &[Spec], repo_root: &Path) -> Result<()> {
    let mut arms = String::new();
    for spec in specs {
        // The error payload is hand-modelled and has no generated module.
        if spec.slug == "trust-task-error" {
            continue;
        }
        let path = spec
            .module_segments()
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("::");
        arms.push_str(&format!(
            "        {:?} => Some(<crate::specs::{}::{}::Payload as crate::validate::ValidatedPayload>::SCHEMA_JSON),\n",
            spec.type_uri(),
            path,
            spec.version_module(),
        ));
    }

    let body = format!(
        r#"//! Generated by `trust-tasks-codegen` — do not edit by hand.
//!
//! Type URI → payload schema, for consumers that dispatch on the URI.

/// The payload schema for `type_uri`, or `None` if this build knows no spec for it.
///
/// `None` is a real answer and callers must decide what it means for them. A
/// consumer that treats "no schema" as "anything goes" has opted out of
/// validation for exactly the tasks it understands least — which is the wrong way
/// round.
#[cfg(feature = "validate")]
pub fn schema_for(type_uri: &str) -> Option<&'static str> {{
    match type_uri {{
{arms}        _ => None,
    }}
}}
"#
    );

    let path = repo_root.join("trust-tasks-rs/src/schema_index.rs");
    fs::write(path, body)?;
    Ok(())
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
