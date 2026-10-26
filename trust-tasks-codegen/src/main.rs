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
//!
//! With `--bundles <out-dir>` it instead writes the registry bundles — one
//! self-contained `bundle.json` per specification version plus an
//! `index.json` — and touches nothing in `trust-tasks-rs`. See [`bundle`].
//!
//! ```sh
//! cargo run -p trust-tasks-codegen -- --bundles website/specs
//! ```

mod bundle;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use proc_macro2::TokenStream;
use quote::quote;
use serde_json::Value;
use trust_tasks_build::publish::{self, compute_lifecycles, Lifecycle};

/// Slug of the framework error spec — modelled by hand in
/// `trust_tasks_rs::ErrorPayload`, so we never emit a generated version.
const SKIP_SLUGS: &[&str] = &["trust-task-error"];

/// Cargo feature that turns on every task family at once. `default` is
/// exactly this one feature, so a consumer who says nothing keeps the
/// pre-0.14 module tree; `default-features = false` plus a handful of
/// family features is what buys the compile-time saving.
const ALL_SPECS_FEATURE: &str = "all-specs";

/// Whether a top-level family is one of the framework-reserved slugs of
/// SPEC §6.1 (`^trust-(task|ceremony)($|-|/)`).
///
/// These are **not** feature-gated. The crate's own hand-written framework
/// machinery depends on them unconditionally (`discovery.rs` does
/// `use crate::specs::trust_task_discovery::v0_3 as wire_v0_3;` at module scope),
/// so gating them would mean gating the framework surface that every consumer
/// needs whatever families they picked. They are also 1.4% of the generated
/// tree, so there is nothing to win by it.
///
/// Derived from the reserved-slug pattern rather than a hard-coded list, so a
/// new `trust-task-*` slug is classified without anybody remembering to.
fn is_reserved_family(family: &str) -> bool {
    for prefix in ["trust-task", "trust-ceremony"] {
        if let Some(rest) = family.strip_prefix(prefix) {
            if rest.is_empty() || rest.starts_with('-') {
                return true;
            }
        }
    }
    false
}

/// The spec families the crate's own unit tests (`#[cfg(test)]` modules in
/// the hand-written `src/*.rs`) use. Every other gated family is compiled out
/// of the unit-test build — see [`unit_test_gate`]. Add a family here before a
/// unit test can name it; until then the test fails to compile with an
/// unresolved `crate::specs::<family>`, which says what to do.
const UNIT_TEST_FAMILIES: &[&str] = &["acl"];

/// The extra attribute line that leaves `family` out of the crate's unit-test
/// build, or `""` for a family the unit tests use.
///
/// `cargo test` compiles the library twice — once as the library that
/// integration tests and dependents link, once more with `cfg(test)` as the
/// unit-test harness — and the generated tree is nearly all of the crate (one
/// module per specification, ~46 MB). The second copy was the largest single
/// cost of a CI run, and its memory peak beside the first is what pushed the
/// test job past the hosted runner's 16 GB. The unit tests only need
/// `UNIT_TEST_FAMILIES`; everything that tests the generated tree as a whole
/// (the `conformance` and `schema_index` integration tests, the dependents'
/// suites) links the full, ordinary library build, which this does not touch.
///
/// A separate `#[cfg]` rather than `all(feature = …, not(test))`, so the
/// `#[cfg(feature = "…")]` line `tests/spec_feature_manifest.rs` reads is
/// unchanged.
fn unit_test_gate(family: &str) -> &'static str {
    if UNIT_TEST_FAMILIES.contains(&family) {
        ""
    } else {
        "#[cfg(not(test))]\n"
    }
}

/// The Cargo feature name for a gated family: the family's directory name,
/// verbatim. Cargo permits `-` in feature names, so `credential-exchange`
/// needs no mangling and the feature a consumer types matches the path
/// segment they see in the registry.
fn feature_name(family: &str) -> &str {
    family
}

/// One spec to generate for.
#[derive(Debug)]
struct Spec {
    /// e.g. `"acl/grant"` (the slug exactly as it appears in the spec).
    slug: String,
    /// e.g. `"0.1"`.
    version: String,
    /// Path to `payload.schema.json`.
    schema_path: PathBuf,
    /// The deprecation note, or `None` for a live version. Set from the
    /// registry-wide lifecycle once every spec is discovered — see
    /// [`compute_lifecycles`].
    deprecation: Option<String>,
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

    /// `"acl"` — the first slug segment, which is the top-level family and
    /// the unit a Cargo feature gates.
    fn family(&self) -> &str {
        self.slug.split('/').next().unwrap_or(&self.slug)
    }

    /// `"v0_1"` — the leaf module name encoding the version.
    fn version_module(&self) -> String {
        format!("v{}", self.version.replace('.', "_"))
    }

    /// Full Type URI for the request variant.
    fn type_uri(&self) -> String {
        format!("https://trusttasks.org/spec/{}/{}", self.slug, self.version)
    }

    /// This version as the shared bundle builder sees it.
    fn dir(&self) -> publish::SpecDir {
        publish::SpecDir::new(&self.slug, &self.version, self.schema_path.clone())
    }

    /// The version's policy, read by the shared builder.
    fn policy(&self) -> Result<publish::SpecPolicyDecl> {
        let has_response = fs::read_to_string(&self.schema_path)
            .ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            .and_then(|v| v.get("$defs").and_then(|d| d.get("Response")).cloned())
            .is_some();
        publish::read_spec_policy(&self.dir(), has_response, &publish::Resolver::local())
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
/// * `variant` — optional, `"request"` (the default) or `"response"`.
///   A `"response"` fixture is checked against the spec's `Response`
///   type and its `$anchor: "response"` sub-schema instead, so a spec
///   can pin the negative space of its response payload too. Naming
///   `"response"` in a spec that defines no response is an error.
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
        let response = match obj.get("variant") {
            None => false,
            Some(Value::String(v)) if v == "request" => false,
            Some(Value::String(v)) if v == "response" => true,
            Some(other) => {
                return Err(anyhow!(
                    "{}[{}].variant must be \"request\" or \"response\", found {}",
                    path.display(),
                    i,
                    other
                ));
            }
        };
        let payload_json = serde_json::to_string_pretty(&payload)?;
        out.push(InvalidExample {
            note,
            payload_json,
            response,
        });
    }
    Ok(out)
}

/// One entry from `payload.invalid-examples.json`. See [`read_invalid_examples`].
#[derive(Debug)]
struct InvalidExample {
    note: String,
    payload_json: String,
    /// `true` for a `"variant": "response"` fixture, checked against `Response`.
    response: bool,
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

    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => {}
        [flag, out] if flag == "--bundles" => {
            return bundle::run(&repo_root, &std::env::current_dir()?.join(out));
        }
        _ => {
            return Err(anyhow!(
                "usage: trust-tasks-codegen [--bundles <out-dir>] (got {args:?})"
            ))
        }
    }
    let specs_dir = repo_root.join("specs");
    let out_root = repo_root.join("trust-tasks-rs/src/specs");

    let discovered = discover_specs(&specs_dir)?;
    if discovered.is_empty() {
        return Err(anyhow!("no specs found under {}", specs_dir.display()));
    }

    // A withdrawn version is generated nowhere — no module, no `mod.rs` entry,
    // no schema-index arm, no conformance test — so it is dropped here, before
    // anything downstream sees it. Every other version learns its deprecation.
    let lifecycles = compute_lifecycles(&specs_dir)?;
    let mut specs = Vec::with_capacity(discovered.len());
    let mut withdrawn = 0usize;
    for mut spec in discovered {
        match lifecycles.get(&(spec.slug.clone(), spec.version.clone())) {
            Some(Lifecycle::Withdrawn) => withdrawn += 1,
            Some(Lifecycle::Deprecated(note)) => {
                spec.deprecation = Some(note.clone());
                specs.push(spec);
            }
            Some(Lifecycle::Live) => specs.push(spec),
            None => {
                return Err(anyhow!(
                    "{}/{}: has a payload.schema.json but no spec.md to read its status from",
                    spec.slug,
                    spec.version
                ))
            }
        }
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

    for module in &modules {
        if let Some(parent) = module.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&module.path, &module.contents)?;
    }
    write_conformance_tree(&specs, &modules, &repo_root)?;

    // Every gated family, in the order the feature table and the `mod.rs`
    // declarations both use. Derived from what is on disk, never listed by
    // hand — the whole point of generating the feature table is that a new
    // family cannot silently arrive without one.
    let gated: Vec<String> = {
        let mut set: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for spec in &specs {
            if !is_reserved_family(spec.family()) {
                set.insert(spec.family().to_string());
            }
        }
        set.into_iter().collect()
    };
    if gated.iter().any(|f| feature_name(f) == ALL_SPECS_FEATURE) {
        return Err(anyhow!(
            "a spec family is named {ALL_SPECS_FEATURE:?}, which collides with the umbrella feature"
        ));
    }

    write_mod_tree(&specs, &out_root)?;
    write_schema_index(&specs, &repo_root)?;
    write_extends_index(&specs, &repo_root)?;
    write_cargo_features(&gated, &repo_root)?;

    println!(
        "\nGenerated {} payload modules into {} ({} deprecated, {} withdrawn and skipped)",
        specs.len(),
        out_root.display(),
        specs.iter().filter(|s| s.deprecation.is_some()).count(),
        withdrawn
    );
    println!(
        "Feature-gated {} spec families ({} framework-reserved families are always compiled)",
        gated.len(),
        specs
            .iter()
            .map(Spec::family)
            .filter(|f| is_reserved_family(f))
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    );
    Ok(())
}

/// Rewrite the generated block of `trust-tasks-rs/Cargo.toml`'s `[features]`
/// table: one feature per top-level spec family, an `all-specs` umbrella
/// enabling every one of them, and `default = ["all-specs"]`.
///
/// Generated rather than hand-maintained on purpose. `CLAUDE.md` records three
/// separate incidents of a hand-kept list drifting from the tree; a family
/// whose feature was forgotten would simply vanish from the crate, and the
/// `codegen-drift` job would not see it because it only diffs `src/specs`.
/// `trust-tasks-rs/tests/spec_feature_manifest.rs` closes that gap by
/// re-deriving the family set from `specs/` and asserting the table matches.
fn write_cargo_features(gated: &[String], repo_root: &Path) -> Result<()> {
    let mut body = String::new();
    body.push_str("# One feature per top-level spec family. `default` is the umbrella, so an\n");
    body.push_str("# unconfigured consumer gets every family exactly as before; the saving is\n");
    body.push_str("# `default-features = false, features = [\"vault\"]` and friends.\n");
    body.push_str("default = [\"");
    body.push_str(ALL_SPECS_FEATURE);
    body.push_str("\"]\n");
    body.push_str(ALL_SPECS_FEATURE);
    body.push_str(" = [\n");
    for family in gated {
        body.push_str(&format!("    {:?},\n", feature_name(family)));
    }
    body.push_str("]\n");
    for family in gated {
        // Every family name is a valid TOML bare key (the registry restricts
        // slugs to `[a-z0-9-]`), so no quoting is needed on the left-hand side.
        let name = feature_name(family);
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(anyhow!(
                "family {name:?} is not a valid Cargo feature / TOML bare key"
            ));
        }
        body.push_str(&format!("{name} = []\n"));
    }

    let path = repo_root.join("trust-tasks-rs/Cargo.toml");
    let existing = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    if !existing.contains(TOML_BEGIN_MARKER) {
        return Err(anyhow!(
            "{} has no `{}` marker; the [features] table cannot be regenerated",
            path.display(),
            TOML_BEGIN_MARKER
        ));
    }
    let updated = replace_between(
        &existing,
        TOML_BEGIN_MARKER,
        TOML_END_MARKER,
        body.trim_end(),
    );
    fs::write(&path, updated).with_context(|| format!("write {}", path.display()))?;
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

/// Every specification version under `specs_dir` except [`SKIP_SLUGS`] — the
/// shared builder's discovery, with this crate's per-version state.
fn discover_specs(specs_dir: &Path) -> Result<Vec<Spec>> {
    Ok(publish::discover_specs(specs_dir, SKIP_SLUGS)?
        .into_iter()
        .map(|dir| Spec {
            slug: dir.slug,
            version: dir.version,
            schema_path: dir.schema_path,
            deprecation: None,
        })
        .collect())
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
/// downstream `cargo fmt` error about an unresolvable module — pointing
/// nowhere near the file at fault.
fn generate_one(spec: &Spec, out_root: &Path) -> Result<GeneratedSpec> {
    // Through the registry bundle, and the generator an application's
    // `build.rs` runs on it (`trust-tasks-build`): the module committed here is
    // therefore, byte for byte, what an application generates from the
    // version's published bundle — only the paths to the runtime differ.
    let built = bundle::build_bundle(spec)?;
    let parsed = trust_tasks_build::Bundle::from_slice(&built.bytes)?;
    let formatted =
        trust_tasks_build::generate_module(&parsed, &trust_tasks_build::Target::runtime_crate())?;
    let has_response = parsed.response_schema.is_some();

    let mut examples = extract_examples(&spec.spec_md_path())?;
    filter_examples_to_this_spec(spec, &mut examples);
    let invalid_examples = read_invalid_examples(spec)?;
    if !has_response && invalid_examples.iter().any(|ex| ex.response) {
        return Err(anyhow!(
            "{}/{}: payload.invalid-examples.json has a \"variant\": \"response\" fixture, \
             but the schema defines no $defs.Response",
            spec.slug,
            spec.version
        ));
    }

    let conformance = render_conformance_file(spec, &examples, &invalid_examples, has_response)
        .map(|tokens| {
            syn::parse2::<syn::File>(tokens.clone())
                .map(|file| prettyplease::unparse(&file))
                .with_context(|| {
                    format!(
                        "failed to parse generated conformance tests for {}/{}:\n{}",
                        spec.slug, spec.version, tokens
                    )
                })
        })
        .transpose()?;

    let mut path = out_root.to_path_buf();
    for seg in spec.module_segments() {
        path = path.join(seg);
    }
    Ok(GeneratedSpec {
        path: path.join(format!("{}.rs", spec.version_module())),
        contents: formatted,
        conformance,
    })
}

/// One spec's generated output: its payload module under `src/specs`, and —
/// when it has any examples to test — its conformance tests, which are written
/// under `tests/conformance` rather than into the module. See
/// [`write_conformance_tree`] for why.
struct GeneratedSpec {
    path: PathBuf,
    contents: String,
    conformance: Option<String>,
}

/// Emit one spec's file of the `conformance` integration test, with one test
/// per harvested example, or `None` when the spec has nothing to test. Each test
/// deserializes the JSON into a `TrustTask<Payload>` (or `Response`) and
/// asserts the wire form round-trips.
///
/// The tests name the module by its public path, `trust_tasks_rs::specs::…`,
/// because they are compiled outside the crate — see [`write_conformance_tree`].
///
/// When `invalid_examples` is non-empty AND the crate is built with the
/// `validate` feature, an additional `rejects_invalid_examples` test is
/// emitted that asserts each fixture either fails serde deserialization
/// (as a `Payload`) or fails JSON-Schema validation under
/// `ValidatedPayload::validate_value`.
fn render_conformance_file(
    spec: &Spec,
    examples: &SpecExamples,
    invalid_examples: &[InvalidExample],
    has_response: bool,
) -> Option<TokenStream> {
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
                    let doc: trust_tasks_rs::TrustTask<spec::Payload> =
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
                        let doc: trust_tasks_rs::TrustTask<spec::Response> =
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

    let request_invalid: Vec<&InvalidExample> =
        invalid_examples.iter().filter(|ex| !ex.response).collect();
    let response_invalid: Vec<&InvalidExample> =
        invalid_examples.iter().filter(|ex| ex.response).collect();

    let invalid_test = if request_invalid.is_empty() {
        quote! {}
    } else {
        let fixtures: Vec<TokenStream> = request_invalid
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
                use trust_tasks_rs::validate::ValidatedPayload;
                let fixtures: &[(&str, &str)] = &[ #(#fixtures),* ];
                for (i, (note, raw)) in fixtures.iter().enumerate() {
                    let value: serde_json::Value = match serde_json::from_str(raw) {
                        Ok(v) => v,
                        // Parse-level rejection — fine, the fixture is invalid wire JSON.
                        Err(_) => continue,
                    };
                    let serde_ok = serde_json::from_value::<spec::Payload>(value.clone()).is_ok();
                    let schema_ok = spec::Payload::validate_value(&value).is_ok();
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

    let response_invalid_test = if response_invalid.is_empty() {
        quote! {}
    } else {
        let fixtures: Vec<TokenStream> = response_invalid
            .iter()
            .map(|ex| {
                let note = &ex.note;
                let payload_json = &ex.payload_json;
                quote! { (#note, #payload_json) }
            })
            .collect();
        quote! {
            /// Each `"variant": "response"` fixture in
            /// `payload.invalid-examples.json` MUST be rejected as a
            /// response payload by at least one of: serde deserialization
            /// into `Response`, or JSON-Schema validation against the
            /// `$anchor: "response"` sub-schema (validate feature).
            #[cfg(feature = "validate")]
            #[test]
            fn rejects_invalid_response_examples() {
                use trust_tasks_rs::validate::ValidatedPayload;
                let fixtures: &[(&str, &str)] = &[ #(#fixtures),* ];
                for (i, (note, raw)) in fixtures.iter().enumerate() {
                    let value: serde_json::Value = match serde_json::from_str(raw) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    let serde_ok = serde_json::from_value::<spec::Response>(value.clone()).is_ok();
                    let schema_ok = spec::Response::validate_value(&value).is_ok();
                    assert!(
                        !(serde_ok && schema_ok),
                        "invalid response example #{} ({:?}) was accepted by both serde and JSON Schema; \
                         the fixture's stated failure class is no longer caught:\n{}",
                        i + 1, note, raw
                    );
                }
            }
        }
    };

    if request_tests.is_empty() && response_tests.is_empty() && invalid_examples.is_empty() {
        return None;
    }

    let module_path: syn::Path = syn::parse_str(&format!(
        "trust_tasks_rs::specs::{}::{}",
        spec.module_segments().join("::"),
        spec.version_module()
    ))
    .expect("module segments are Rust identifiers");
    let slug_doc = format!(" `{}/{}`", spec.slug, spec.version);

    Some(quote! {
        //! Generated by `trust-tasks-codegen` — do not edit by hand.
        //!
        //! Round-trip tests harvested from the spec's `spec.md`, plus a
        //! `rejects_invalid_examples` test for any fixtures in
        //! `payload.invalid-examples.json` (validate feature), for
        #![doc = #slug_doc]

        // Unused when every test here is behind the `validate` feature.
        #[allow(unused_imports)]
        use #module_path as spec;

        #(#request_tests)*
        #(#response_tests)*

        #invalid_test

        #response_invalid_test
    })
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
/// schema for the payload it is about to run: `Payload::PAYLOAD_SCHEMA` is a
/// per-type associated const, and a generic gate has no type to name. It could
/// only validate by hand-writing a match arm per task and remembering to add one
/// with every new task — which is to say, it would validate whatever somebody
/// remembered, which is not validation.
fn write_schema_index(specs: &[Spec], repo_root: &Path) -> Result<()> {
    let mut arms = String::new();
    // Request-side only: the flag-driven rules are about a document a consumer
    // *received*, and a `#response` has no audience-binding or proof
    // requirement of its own to enforce.
    let mut policy_arms = String::new();
    // One arm per task, keyed on the bare request Type URI: an error code is
    // declared by the specification, not by one of its variants.
    let mut error_code_arms = String::new();
    // Only the Type URIs whose specification declares a bound: `None` from the
    // wildcard means "no declared bound", and for a Type URI this build does
    // not know that is the same answer — the consumer's own transport bound.
    let mut size_arms = String::new();
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
        // A family the consumer did not select has no module to name, so the
        // arm has to go with it. `schema_for` then answers `None` for that
        // Type URI, which is exactly what its doc comment says `None` means:
        // "this build knows no spec for it".
        let gate = if is_reserved_family(spec.family()) {
            String::new()
        } else {
            format!(
                "        #[cfg(feature = {:?})]\n{}",
                feature_name(spec.family()),
                match unit_test_gate(spec.family()) {
                    "" => String::new(),
                    gate => format!("        {gate}"),
                }
            )
        };
        arms.push_str(&gate);
        arms.push_str(&format!(
            "        {:?} => <crate::specs::{}::{}::Payload as crate::Payload>::PAYLOAD_SCHEMA,\n",
            spec.type_uri(),
            path,
            spec.version_module(),
        ));
        policy_arms.push_str(&gate);
        policy_arms.push_str(&format!(
            "        {:?} => Some(crate::SpecPolicy::of::<crate::specs::{}::{}::Payload>()),\n",
            spec.type_uri(),
            path,
            spec.version_module(),
        ));
        error_code_arms.push_str(&gate);
        error_code_arms.push_str(&format!(
            "        {:?} => Some(crate::specs::{}::{}::ERROR_CODES),\n",
            spec.type_uri(),
            path,
            spec.version_module(),
        ));
        let max_document_bytes = spec.policy()?.max_document_bytes;
        if max_document_bytes.request.is_some() {
            size_arms.push_str(&gate);
            size_arms.push_str(&format!(
                "        {:?} => <crate::specs::{}::{}::Payload as crate::Payload>::MAX_DOCUMENT_BYTES,\n",
                spec.type_uri(),
                path,
                spec.version_module(),
            ));
        }
        if max_document_bytes.response.is_some() {
            size_arms.push_str(&gate);
            size_arms.push_str(&format!(
                "        {:?} => <crate::specs::{}::{}::Response as crate::Payload>::MAX_DOCUMENT_BYTES,\n",
                format!("{}#response", spec.type_uri()),
                path,
                spec.version_module(),
            ));
        }
        // The response side, under the same `#response` URI its `Response`
        // type already declares as `TYPE_URI`. Indexing only requests left a
        // consumer able to validate what it received and not what it sent
        // back — and a producer's own output is the half it can actually fix.
        // `$defs.Response` is what makes the generator emit a `Response`
        // type; read it from the same file rather than trusting a flag that
        // is not on this struct.
        let has_response = fs::read_to_string(&spec.schema_path)
            .ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            .and_then(|v| v.get("$defs").and_then(|d| d.get("Response")).cloned())
            .is_some();
        if has_response {
            arms.push_str(&gate);
            arms.push_str(&format!(
                "        {:?} => <crate::specs::{}::{}::Response as crate::Payload>::PAYLOAD_SCHEMA,\n",
                format!("{}#response", spec.type_uri()),
                path,
                spec.version_module(),
            ));
        }
    }

    // One policy arm per *request* schema arm, checked here rather than in a
    // runtime test: the crate exposes no list of every Type URI to iterate, and
    // this is the invariant that matters — both matches are emitted from this
    // one loop, so a generator edited on one side and not the other is a bug in
    // this function, catchable in this function.
    let request_arms = arms
        .lines()
        .filter(|l| l.contains("=>") && !l.contains("#response\""))
        .count();
    let policy_arm_count = policy_arms.lines().filter(|l| l.contains("=>")).count();
    anyhow::ensure!(
        request_arms == policy_arm_count,
        "schema_index: {request_arms} request schema arms but {policy_arm_count} policy arms — \
         the two matches must cover the same Type URIs"
    );
    let error_code_arm_count = error_code_arms.lines().filter(|l| l.contains("=>")).count();
    anyhow::ensure!(
        request_arms == error_code_arm_count,
        "schema_index: {request_arms} request schema arms but {error_code_arm_count} error-code \
         arms — the two matches must cover the same Type URIs"
    );

    let body = format!(
        r#"//! Generated by `trust-tasks-codegen` — do not edit by hand.
//!
//! Type URI → payload schema and consumer policy, for consumers that dispatch
//! on the URI.
//!
//! Both sides are indexed: the request under the bare Type URI, the response
//! under the same URI suffixed `#response`, which is what the generated
//! `Response` type already declares as its own `TYPE_URI`.

// A deprecated specification version is indexed until it is withdrawn, and
// indexing it means naming its deprecated `Payload` / `Response` types.
#![allow(deprecated)]

/// The payload schema for `type_uri`, or `None` if this build knows no spec for it.
///
/// Pass a bare Type URI for a request schema, or one suffixed `#response` for
/// the response schema of the same task. A task with no response side has no
/// `#response` entry.
///
/// `None` is a real answer and callers must decide what it means for them. A
/// consumer that treats "no schema" as "anything goes" has opted out of
/// validation for exactly the tasks it understands least — which is the wrong way
/// round.
///
/// Available without the `validate` feature: the schemas are `&'static str`
/// constants and cost nothing to carry. Only *evaluating* one needs a JSON
/// Schema implementation, which the caller supplies via
/// [`crate::PayloadValidator`].
pub fn schema_for(type_uri: &str) -> Option<&'static str> {{
    match type_uri {{
{arms}        _ => None,
    }}
}}

/// The flag-driven consumer policy for `type_uri`, or `None` if this build
/// knows no spec for it.
///
/// [`TrustTask::enforce_spec_policy`](crate::TrustTask::enforce_spec_policy)
/// reads the same constants off `P` and is what a consumer holding a typed
/// document should call. This exists for the other shape, which the rest of
/// this module already serves: a consumer that dispatches on the Type URI and
/// holds a `TrustTask<serde_json::Value>`, and so has no `P` to read them from.
/// Without it, such a consumer must hand-maintain its own URI → payload-type
/// table — a second source of truth for data this file already generates.
///
/// Request side only. The rules are about a document a consumer *received*;
/// a `#response` carries no audience-binding or proof requirement of its own,
/// so there are no `#response` entries here and `None` is the answer for one.
///
/// [`SpecPolicy::enforce`](crate::SpecPolicy::enforce) applies it, and is the
/// same code path the typed method takes.
pub fn spec_policy_for(type_uri: &str) -> Option<crate::SpecPolicy> {{
    match type_uri {{
{policy_arms}        _ => None,
    }}
}}

/// The extended error codes the specification behind `type_uri` declares
/// (SPEC §7.3 item 9, §8.5), or `None` if this build knows no spec for it.
///
/// Pass the bare request Type URI. Error codes belong to the specification, not
/// to one of its variants, so there are no `#response` entries.
///
/// `Some(&[])` and `None` are different answers: the first is a specification
/// that declares no task-specific codes, the second a Type URI this build does
/// not know. A census that treats them alike would pass every task it has never
/// heard of.
///
/// The same slice is `ERROR_CODES` on the generated module; this exists for a
/// consumer holding only the URI — a conformance census over every task it
/// dispatches, say, which could otherwise only compare the codes it emits
/// against a list it maintains by hand.
pub fn error_codes_for(type_uri: &str) -> Option<&'static [crate::DeclaredErrorCode]> {{
    match type_uri {{
{error_code_arms}        _ => None,
    }}
}}

/// The document size bound the specification behind `type_uri` declares
/// (front matter `maxDocumentBytes`), in bytes of the serialized Trust Task
/// document, or `None` where none is declared.
///
/// Pass a bare Type URI for the request bound, or one suffixed `#response` for
/// the response bound. The same value is `Payload::MAX_DOCUMENT_BYTES` on the
/// generated type; this exists for a dispatcher that learns the `type` before
/// it has parsed the rest of the document — an HTTP `Trust-Task` header, say —
/// and has to decide how much body to read.
///
/// `None` covers both a specification that declares no bound and a Type URI
/// this build does not know. For a size bound those are the same answer: the
/// consumer's own transport limit applies (SPEC §12.4).
pub fn max_document_bytes_for(type_uri: &str) -> Option<usize> {{
    #[allow(clippy::match_single_binding)]
    match type_uri {{
{size_arms}        _ => None,
    }}
}}

/// The version the specification behind `type_uri` declares it extends (SPEC
/// §5.2 "Declared extension"), or `None`. Re-exported from
/// [`crate::extension`], which also walks the chain; it is here so every
/// per-Type-URI lookup has one home.
pub use crate::extension::extends_for;
"#
    );

    let path = repo_root.join("trust-tasks-rs/src/schema_index.rs");
    fs::write(path, body)?;
    Ok(())
}

/// Emit `trust-tasks-rs/src/extends_index.rs` — bare request Type URI → the
/// `MAJOR.MINOR` its specification declares it extends (front matter
/// `extends`, SPEC §5.2 "Declared extension").
///
/// Not part of `schema_index`, and not feature-gated per family, on purpose:
/// the dispatchers route along it, and they are compiled whether or not the
/// `validate` feature (which gates `schema_index`) or the received version's
/// family module is. The arms are string literals rather than references to
/// each module's `EXTENDS`, so a build without a family still knows that
/// family's chain — the received version never needs a module of its own to
/// be routed to the version the consumer implements. `check-bindings` holds
/// the per-module constants to the front matter; the generator holds this
/// table to the same reading, since both come from the shared front-matter
/// reader (`publish::read_spec_policy`).
fn write_extends_index(specs: &[Spec], repo_root: &Path) -> Result<()> {
    let mut arms = String::new();
    for spec in specs {
        if let Some(target) = spec.policy()?.extends {
            arms.push_str(&format!(
                "        {:?} => Some({:?}),\n",
                spec.type_uri(),
                target
            ));
        }
    }
    let body = format!(
        r#"//! Generated by `trust-tasks-codegen` — do not edit by hand.
//!
//! Bare request Type URI → the `MAJOR.MINOR` its specification declares it
//! extends (SPEC §5.2 "Declared extension"). Read through
//! [`crate::extension::extends_for`].

/// The direct `extends` declaration of the specification behind the bare
/// request `type_uri`, or `None`.
pub(crate) fn declared_extends(type_uri: &str) -> Option<&'static str> {{
    #[allow(clippy::match_single_binding)]
    match type_uri {{
{arms}        _ => None,
    }}
}}
"#
    );
    fs::write(repo_root.join("trust-tasks-rs/src/extends_index.rs"), body)?;
    Ok(())
}

/// Write the `conformance` integration test: `tests/conformance/main.rs`
/// declaring one module per specification that has examples, and each
/// specification's tests at `tests/conformance/<slug segments>/<version>.rs`.
///
/// These used to be a `#[cfg(test)] mod conformance` inside each generated
/// module, which made them unit tests — and a unit test needs the `cfg(test)`
/// build of the whole library, a second compilation of the entire generated
/// tree. As an integration test they link the ordinary library build that the
/// dependents already need. They only ever used public items.
///
/// The directory is wholly generated, so it is removed and rewritten, which is
/// how a removed specification's tests go with it.
fn write_conformance_tree(
    specs: &[Spec],
    modules: &[GeneratedSpec],
    repo_root: &Path,
) -> Result<()> {
    let dir = repo_root.join("trust-tasks-rs/tests/conformance");
    if fs::symlink_metadata(&dir).is_ok_and(|m| m.file_type().is_symlink()) {
        return Err(anyhow!(
            "{} is a symlink; refusing to clean it",
            dir.display()
        ));
    }
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }

    let mut root = ModNode::default();
    let mut gates: BTreeMap<String, String> = BTreeMap::new();
    for (spec, module) in specs.iter().zip(modules) {
        let Some(tests) = &module.conformance else {
            continue;
        };
        let segments = spec.module_segments();
        let mut path = dir.clone();
        for seg in &segments {
            path = path.join(seg);
        }
        fs::create_dir_all(&path)?;
        fs::write(path.join(format!("{}.rs", spec.version_module())), tests)?;

        if !is_reserved_family(spec.family()) {
            gates.insert(segments[0].clone(), spec.family().to_string());
        }
        let mut cursor = &mut root;
        for seg in segments {
            cursor = cursor.children.entry(seg).or_default();
        }
        cursor.leaves.push(spec.version_module());
    }

    // Inline modules contribute their names to the path a nested `mod x;` is
    // looked up at, so one file can declare the whole tree.
    fn render(node: &ModNode, gates: &BTreeMap<String, String>, depth: usize, out: &mut String) {
        let indent = "    ".repeat(depth);
        let mut names: Vec<&String> = node.children.keys().chain(node.leaves.iter()).collect();
        names.sort();
        for name in names {
            if let Some(family) = gates.get(name) {
                out.push_str(&format!(
                    "{indent}#[cfg(feature = {:?})]\n",
                    feature_name(family)
                ));
            }
            match node.children.get(name) {
                Some(child) => {
                    out.push_str(&format!("{indent}mod {name} {{\n"));
                    render(child, &BTreeMap::new(), depth + 1, out);
                    out.push_str(&format!("{indent}}}\n"));
                }
                None => out.push_str(&format!("{indent}mod {name};\n")),
            }
        }
    }
    let mut body = String::from(
        "//! Generated by `trust-tasks-codegen` — do not edit by hand.\n\
         //!\n\
         //! The round-trip tests harvested from every specification's examples. One\n\
         //! integration test rather than a `#[cfg(test)]` module per specification, so\n\
         //! they link the ordinary library build instead of a second, unit-test\n\
         //! compilation of the whole generated tree.\n\
         \n\
         // A deprecated specification version is still tested until it is\n\
         // withdrawn, and testing it means naming its deprecated types.\n\
         #![allow(deprecated)]\n\n",
    );
    render(&root, &gates, 0, &mut body);
    fs::write(dir.join("main.rs"), body)?;
    Ok(())
}

/// Write `mod.rs` files at every level of the generated tree.
///
/// The root `mod.rs` is the only level that carries `#[cfg(feature = …)]`:
/// gating happens per top-level family, so once a family is in, everything
/// under it is in.
fn write_mod_tree(specs: &[Spec], out_root: &Path) -> Result<()> {
    let mut root = ModNode::default();
    // Root module name (`credential_exchange`) → family directory name
    // (`credential-exchange`), for the families that get a feature.
    let mut gates: BTreeMap<String, String> = BTreeMap::new();
    for spec in specs {
        let segments = spec.module_segments();
        if !is_reserved_family(spec.family()) {
            if let Some(root_module) = segments.first() {
                gates.insert(root_module.clone(), spec.family().to_string());
            }
        }
        let mut cursor = &mut root;
        for seg in segments {
            cursor = cursor.children.entry(seg).or_default();
        }
        cursor.leaves.push(spec.version_module());
    }

    write_node(&root, out_root, true, &gates)?;
    Ok(())
}

fn write_node(
    node: &ModNode,
    dir: &Path,
    is_root: bool,
    gates: &BTreeMap<String, String>,
) -> Result<()> {
    let decl = |name: &String| match gates.get(name) {
        Some(family) => format!(
            "#[cfg(feature = {:?})]\n{}pub mod {name};",
            feature_name(family),
            unit_test_gate(family)
        ),
        None => format!("pub mod {name};"),
    };
    let mut names: Vec<&String> = node.children.keys().chain(node.leaves.iter()).collect();
    names.sort();
    let body = names.into_iter().map(decl).collect::<Vec<_>>().join("\n");

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

    // Only the root level gates: a family's feature covers its whole subtree.
    for (name, child) in &node.children {
        write_node(child, &dir.join(name), false, &BTreeMap::new())?;
    }
    Ok(())
}

const BEGIN_MARKER: &str = "// trust-tasks-codegen:begin";
const END_MARKER: &str = "// trust-tasks-codegen:end";

/// TOML comments start with `#`, so `trust-tasks-rs/Cargo.toml` carries its own
/// pair of markers around the generated `[features]` entries.
const TOML_BEGIN_MARKER: &str = "# trust-tasks-codegen:begin";
const TOML_END_MARKER: &str = "# trust-tasks-codegen:end";

fn replace_between_markers(existing: &str, body: &str) -> String {
    replace_between(existing, BEGIN_MARKER, END_MARKER, body)
}

fn replace_between(existing: &str, begin: &str, end: &str, body: &str) -> String {
    match (existing.find(begin), existing.find(end)) {
        (Some(b), Some(e)) if e > b => {
            let before = &existing[..b + begin.len()];
            let after = &existing[e..];
            format!("{before}\n{body}\n{after}")
        }
        _ => format!("{existing}\n{begin}\n{body}\n{end}\n"),
    }
}
