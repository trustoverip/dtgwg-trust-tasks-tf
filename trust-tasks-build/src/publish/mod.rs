//! Building registry bundles from a `specs/` tree — the publisher's side.
//!
//! Enabled by the `publish` feature. One implementation serves both
//! publishers:
//!
//! - the registry itself, through the repository's `trust-tasks-codegen
//!   --bundles`, which builds `https://trusttasks.org/specs/…`;
//! - a third party's ecosystem registry, through `trust-tasks registry
//!   build`, whose specifications live under its own authority (SPEC §6.5).
//!
//! A bundle is one specification version, self-contained: its schemas with
//! every cross-file `$ref` inlined ([`Resolver`]), the front-matter policy the
//! runtime enforces ([`read_spec_policy`]), its declared error codes and its
//! lifecycle ([`compute_lifecycles`]).
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
//! # The served layout
//!
//! [`write_registry`] writes, under a site root's `specs/`:
//!
//! ```text
//! specs/index.json                        every version, digest and status
//! specs/<slug>/<M.N>/bundle.json          a version's current bytes
//! specs/by-digest/sha256-<hex>.json       every bundle ever written, append-only
//! ```

mod order;
mod policy;
mod refs;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Map, Value};

#[doc(hidden)]
pub use order::collect_property_owners;
pub use policy::{read_front_matter, read_spec_policy, SpecPolicyDecl, Variants};
pub use refs::{Fetch, Resolver};

use crate::lock::sha256_digest;
use crate::names::{
    is_official_type_uri, is_reserved_slug, is_spec_slug, is_spec_version, type_uri_parts,
};

/// `bundleVersion` — the shape of `bundle.json`.
pub const BUNDLE_VERSION: u64 = crate::bundle::BUNDLE_VERSION;
/// `indexVersion` — the shape of `index.json`.
pub const INDEX_VERSION: u64 = 1;
/// File name of a bundle inside `specs/<slug>/<M.N>/`.
pub const BUNDLE_FILE: &str = "bundle.json";
/// File name of the index inside `specs/`.
pub const INDEX_FILE: &str = "index.json";
/// The append-only content-addressed store inside `specs/`.
pub const BY_DIGEST_DIR: &str = "by-digest";

/// One specification version on disk: `specs/<slug>/<M.N>/payload.schema.json`
/// and the `spec.md` beside it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct SpecDir {
    /// e.g. `"acl/grant"`, exactly as the directories spell it.
    pub slug: String,
    /// e.g. `"0.1"`.
    pub version: String,
    /// Path to `payload.schema.json`.
    pub schema_path: PathBuf,
}

impl SpecDir {
    /// A specification version whose payload schema is at `schema_path`.
    pub fn new(slug: impl Into<String>, version: impl Into<String>, schema_path: PathBuf) -> Self {
        Self {
            slug: slug.into(),
            version: version.into(),
            schema_path,
        }
    }

    /// Path to the adjacent `spec.md`.
    pub fn spec_md_path(&self) -> PathBuf {
        self.schema_path
            .parent()
            .expect("schema path has a parent")
            .join("spec.md")
    }
}

/// Every `payload.schema.json` under `specs_dir`, as `<slug>/<M.N>`, skipping
/// the slugs in `skip`, sorted by slug then version *string*.
pub fn discover_specs(specs_dir: &Path, skip: &[&str]) -> Result<Vec<SpecDir>> {
    let mut out = Vec::new();
    let mut files = Vec::new();
    walk_files(specs_dir, &mut files)?;
    for path in files {
        if path.file_name().is_none_or(|n| n != "payload.schema.json") {
            continue;
        }
        let version_dir = path
            .parent()
            .ok_or_else(|| anyhow!("payload.schema.json with no parent: {}", path.display()))?;
        let slug_dir = version_dir
            .parent()
            .ok_or_else(|| anyhow!("no slug dir for {}", path.display()))?;
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
        if skip.contains(&slug.as_str()) {
            continue;
        }
        out.push(SpecDir {
            slug,
            version,
            schema_path: path,
        });
    }
    out.sort_by(|a, b| (a.slug.as_str(), a.version.as_str()).cmp(&(&b.slug, &b.version)));
    Ok(out)
}

/// Every regular file under `dir`, recursively, not following symlinks.
fn walk_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e).with_context(|| format!("read {}", dir.display())),
    };
    for entry in entries {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() {
            walk_files(&entry.path(), out)?;
        } else if kind.is_file() {
            out.push(entry.path());
        }
    }
    Ok(())
}

/// Where one specification version stands in the lifecycle of SPEC
/// "Superseded and Withdrawn Versions".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lifecycle {
    /// Supported and current.
    Live,
    /// Still supported, but deprecated, carrying this note.
    Deprecated(String),
    /// `status: withdrawn`.
    Withdrawn,
}

/// One version's front-matter lifecycle inputs.
#[derive(Debug, Clone)]
struct VersionStatus {
    version: String,
    order: (u64, u64),
    status: String,
    superseded_by: Option<String>,
}

/// `MAJOR.MINOR` → `(MAJOR, MINOR)`, or `None` for anything else.
pub fn parse_version(v: &str) -> Option<(u64, u64)> {
    let (major, minor) = v.split_once('.')?;
    let all_digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if !all_digits(major) || !all_digits(minor) {
        return None;
    }
    Some((major.parse().ok()?, minor.parse().ok()?))
}

/// Maturity rank of a non-terminal status: draft < candidate < standard.
fn maturity_rank(status: &str) -> Option<u8> {
    match status {
        "draft" => Some(0),
        "candidate" => Some(1),
        "standard" => Some(2),
        _ => None,
    }
}

/// Whether a version still counts as a supported version of its slug.
fn is_live(status: &str) -> bool {
    status != "retired" && status != "withdrawn"
}

/// Every `specs/<slug>/<MAJOR.MINOR>/spec.md`'s `status` and `supersededBy`,
/// grouped by slug.
fn read_version_statuses(specs_dir: &Path) -> Result<BTreeMap<String, Vec<VersionStatus>>> {
    let mut out: BTreeMap<String, Vec<VersionStatus>> = BTreeMap::new();
    let mut files = Vec::new();
    walk_files(specs_dir, &mut files)?;
    for path in files {
        if path.file_name().is_none_or(|n| n != "spec.md") {
            continue;
        }
        let Some(version_dir) = path.parent() else {
            continue;
        };
        let Some(version) = version_dir.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let Some(order) = parse_version(version) else {
            continue;
        };
        let Some(slug_dir) = version_dir.parent() else {
            continue;
        };
        let slug = slug_dir
            .strip_prefix(specs_dir)
            .with_context(|| format!("slug dir {} is not under specs/", slug_dir.display()))?
            .to_string_lossy()
            .replace('\\', "/");
        let value = read_front_matter(&path)?
            .ok_or_else(|| anyhow!("{} has no front matter", path.display()))?;
        let status = value
            .get("status")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("{} declares no `status`", path.display()))?
            .to_string();
        if maturity_rank(&status).is_none() && is_live(&status) {
            return Err(anyhow!("{}: unknown status {status:?}", path.display()));
        }
        let superseded_by = value
            .get("supersededBy")
            .and_then(Value::as_str)
            .map(str::to_string);
        out.entry(slug).or_default().push(VersionStatus {
            version: version.to_string(),
            order,
            status,
            superseded_by,
        });
    }
    Ok(out)
}

/// The lifecycle of every specification version under `specs_dir`, keyed by
/// `(slug, version)`.
///
/// Implements SPEC "Superseded and Withdrawn Versions":
///
/// - `status: withdrawn` → [`Lifecycle::Withdrawn`].
/// - The **live** versions of a slug are those neither `retired` nor
///   `withdrawn`.
/// - A version is **deprecated** when it is `retired`, or **superseded**: a
///   later live version of the same slug exists at an equal or higher maturity
///   (draft < candidate < standard). A later *draft* does not supersede a
///   `standard`.
/// - Its **successor** is, for a retired version declaring `supersededBy`,
///   that target — as written when it names a version, else the named slug's
///   latest live version — and otherwise the latest live version of its own
///   slug. Either may be absent.
/// - The note is `retired; use <S>`, `retired`, or `superseded by <S>`, byte for
///   byte: `check-bindings` compares it across all four libraries.
pub fn compute_lifecycles(specs_dir: &Path) -> Result<BTreeMap<(String, String), Lifecycle>> {
    let by_slug = read_version_statuses(specs_dir)?;
    let latest_live = |slug: &str| -> Option<String> {
        by_slug
            .get(slug)?
            .iter()
            .filter(|v| is_live(&v.status))
            .max_by_key(|v| v.order)
            .map(|v| format!("{slug}/{}", v.version))
    };

    let mut out = BTreeMap::new();
    for (slug, versions) in &by_slug {
        for this in versions {
            let lifecycle = if this.status == "withdrawn" {
                Lifecycle::Withdrawn
            } else if this.status == "retired" {
                let successor = match &this.superseded_by {
                    Some(target) => match target.rsplit_once('/') {
                        Some((_, last)) if parse_version(last).is_some() => Some(target.clone()),
                        _ => latest_live(target),
                    },
                    None => latest_live(slug),
                };
                Lifecycle::Deprecated(match successor {
                    Some(s) => format!("retired; use {s}"),
                    None => "retired".to_string(),
                })
            } else {
                let rank = maturity_rank(&this.status);
                let superseded = versions.iter().any(|later| {
                    later.order > this.order
                        && is_live(&later.status)
                        && maturity_rank(&later.status) >= rank
                });
                match (superseded, latest_live(slug)) {
                    (true, Some(s)) => Lifecycle::Deprecated(format!("superseded by {s}")),
                    _ => Lifecycle::Live,
                }
            };
            out.insert((slug.clone(), this.version.clone()), lifecycle);
        }
    }
    Ok(out)
}

/// A specification version's self-contained schemas: the request payload
/// schema with every cross-file `$ref` inlined, and — when it defines a
/// `$defs.Response` — the response document wrapping it.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct SpecSchemas {
    /// The request payload schema.
    pub schema: Value,
    /// The response document schema, or `None`.
    pub response: Option<Value>,
    /// For each `$defs` entry spliced in from another document, where it came
    /// from — where its authored member order is read.
    pub(crate) def_origins: BTreeMap<String, refs::Loc>,
}

/// Build `spec`'s self-contained schemas.
pub fn spec_schemas(spec: &SpecDir, refs: &Resolver) -> Result<SpecSchemas> {
    let mut schema: Value = serde_json::from_str(&fs::read_to_string(&spec.schema_path)?)
        .with_context(|| format!("parse {}", spec.schema_path.display()))?;

    // Inline every cross-file `$ref` (into _shared/ or _framework/, or the
    // registry's shared components by URL) into this schema's own `$defs`, so
    // the result needs no resolver — not typify, and not a runtime validator.
    let base = refs::file_base(&spec.schema_path)?;
    let mut def_origins = BTreeMap::new();
    refs::resolve_cross_file_refs_tracked(&mut schema, &base, refs, &mut def_origins)
        .with_context(|| {
            format!(
                "failed to resolve cross-file $refs for {}/{}",
                spec.slug, spec.version
            )
        })?;

    // The response variant needs a schema of its own: the root schema
    // describes the *request* payload, and `$defs.Response` describes the
    // response. Wrap the latter in a document that keeps `$defs` reachable so
    // its internal `$ref`s still resolve, rather than lifting the subschema
    // out and breaking them.
    let response = schema
        .get("$defs")
        .and_then(|d| d.get("Response"))
        .map(|_| {
            let mut doc = Map::new();
            if let Some(s) = schema.get("$schema") {
                doc.insert("$schema".into(), s.clone());
            }
            // No `$id`: JSON Schema 2020-12 forbids a non-empty fragment in
            // `$id`, so the natural `<base>#response` is not a legal value and
            // a compliant validator refuses to compile the document. The
            // response schema needs no identity of its own — nothing `$ref`s
            // it from outside — and `#/$defs/...` resolves against the
            // document root either way.
            doc.insert("$ref".into(), Value::String("#/$defs/Response".into()));
            doc.insert(
                "$defs".into(),
                schema.get("$defs").cloned().unwrap_or(Value::Null),
            );
            Value::Object(doc)
        });
    Ok(SpecSchemas {
        schema,
        response,
        def_origins,
    })
}

/// One specification version's bundle, serialized.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct BuiltBundle {
    /// The slug.
    pub slug: String,
    /// `MAJOR.MINOR`.
    pub version: String,
    /// The request Type URI the bundle names.
    pub type_uri: String,
    /// Front matter `status`.
    pub status: String,
    /// The lifecycle note, if deprecated.
    pub deprecation: Option<String>,
    /// The declared `extends`.
    pub extends: Option<String>,
    /// The canonical bytes of `bundle.json`.
    pub bytes: Vec<u8>,
    /// `sha256:<hex>` of [`Self::bytes`].
    pub digest: String,
}

impl BuiltBundle {
    /// `<slug>/<M.N>/bundle.json`, relative to `specs/`.
    pub fn path(&self) -> String {
        format!("{}/{}/{BUNDLE_FILE}", self.slug, self.version)
    }
}

/// Build one specification version's bundle, naming it `type_uri`, with the
/// lifecycle note `deprecation`.
pub fn build_bundle(
    spec: &SpecDir,
    type_uri: &str,
    deprecation: Option<&str>,
    refs: &Resolver,
) -> Result<BuiltBundle> {
    let schemas = spec_schemas(spec, refs)?;
    let policy = read_spec_policy(spec, schemas.response.is_some(), refs)?;
    let md = spec.spec_md_path();
    let front_matter =
        read_front_matter(&md)?.ok_or_else(|| anyhow!("{} has no front matter", md.display()))?;
    let fm_str = |key: &str| {
        front_matter
            .get(key)
            .and_then(Value::as_str)
            .map(str::to_string)
    };
    let status = fm_str("status")
        .ok_or_else(|| anyhow!("{}/{} declares no `status`", spec.slug, spec.version))?;

    let property_order = order::property_order(spec, &schemas, refs)
        .with_context(|| format!("propertyOrder for {}/{}", spec.slug, spec.version))?;

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
        "typeUri": type_uri,
        "slug": spec.slug,
        "version": spec.version,
        "status": status,
        "deprecation": deprecation,
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
        "propertyOrder": property_order,
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
        type_uri: type_uri.to_string(),
        status,
        deprecation: deprecation.map(str::to_string),
        extends: policy.extends,
        bytes,
        digest,
    })
}

/// The canonical bytes of `index.json`. `excluded` names slugs that have no
/// bundle on purpose, with the reason.
pub fn render_index(
    bundles: &[BuiltBundle],
    generated_from: Option<&str>,
    excluded: &[(&str, &str)],
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
    let excluded: Vec<Value> = excluded
        .iter()
        .map(|(slug, reason)| json!({ "slug": slug, "reason": reason }))
        .collect();
    canonical_json(&json!({
        "indexVersion": INDEX_VERSION,
        "bundleVersion": BUNDLE_VERSION,
        "generatedFrom": generated_from,
        "excluded": excluded,
        "specs": specs,
    }))
}

/// What [`write_registry`] did.
#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct Written {
    /// `bundle.json` files under `specs/` that named no version any more, and
    /// were removed.
    pub stale_removed: usize,
    /// Bundles added to the by-digest store.
    pub by_digest_added: usize,
}

/// Write `bundles` and `index` into `specs_out` (a site's `specs/`): each
/// bundle at `<slug>/<M.N>/bundle.json`, `index.json`, and — with
/// `by_digest` — each bundle again at `by-digest/sha256-<hex>.json`.
///
/// A `bundle.json` under `specs_out` that names no version any more is
/// removed. The by-digest store is append-only: nothing in it is removed, and
/// an existing entry whose bytes are not its name's digest stops the write.
pub fn write_registry(
    specs_out: &Path,
    bundles: &[BuiltBundle],
    index: &str,
    by_digest: bool,
) -> Result<Written> {
    fs::create_dir_all(specs_out).with_context(|| format!("create {}", specs_out.display()))?;
    let mut written: BTreeSet<PathBuf> = BTreeSet::new();
    let mut out = Written::default();
    for b in bundles {
        let path = specs_out.join(&b.slug).join(&b.version).join(BUNDLE_FILE);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, &b.bytes).with_context(|| format!("write {}", path.display()))?;
        written.insert(path);
    }
    let store = specs_out.join(BY_DIGEST_DIR);
    let mut files = Vec::new();
    walk_files(specs_out, &mut files)?;
    for path in files {
        if path.starts_with(&store) {
            continue;
        }
        if path.file_name().is_some_and(|n| n == BUNDLE_FILE) && !written.contains(&path) {
            fs::remove_file(&path).with_context(|| format!("remove stale {}", path.display()))?;
            out.stale_removed += 1;
        }
    }
    if by_digest {
        fs::create_dir_all(&store).with_context(|| format!("create {}", store.display()))?;
        for b in bundles {
            let hex = b.digest.trim_start_matches("sha256:");
            let path = store.join(format!("sha256-{hex}.json"));
            match fs::read(&path) {
                Ok(existing) if sha256_digest(&existing) == b.digest => continue,
                Ok(_) => {
                    return Err(anyhow!(
                        "{} holds bytes that are not {} — the by-digest store is append-only \
                         and must never be rewritten; restore it from the published copy",
                        path.display(),
                        b.digest
                    ))
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e).with_context(|| format!("read {}", path.display())),
            }
            fs::write(&path, &b.bytes).with_context(|| format!("write {}", path.display()))?;
            out.by_digest_added += 1;
        }
    }
    fs::write(specs_out.join(INDEX_FILE), index)
        .with_context(|| format!("write {}", specs_out.join(INDEX_FILE).display()))?;
    Ok(out)
}

/// Serialize `value` in the canonical form described in the module docs: the
/// RFC 8785 (JCS) form for every value a registry contains.
///
/// Written out rather than relying on `serde_json::Map` happening to be a
/// `BTreeMap`: that holds only while no crate in the build enables
/// `serde_json/preserve_order`, and feature unification would flip it
/// silently. JCS specifies UTF-16 order, so that is what is sorted on.
pub fn canonical_json(value: &Value) -> Result<String> {
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

/// The Type URI of `slug` at `version` under `prefix` (SPEC §6.1): the URN
/// form `<prefix>:spec:<seg>:…:<M.N>` when `prefix` is a `urn:`, and the path
/// form `<prefix>/spec/<slug>/<M.N>` otherwise.
pub fn type_uri_under(prefix: &str, slug: &str, version: &str) -> String {
    let is_urn = prefix
        .split_once(':')
        .is_some_and(|(scheme, _)| scheme.eq_ignore_ascii_case("urn"));
    if is_urn {
        format!("{prefix}:spec:{}:{version}", slug.replace('/', ":"))
    } else {
        format!("{prefix}/spec/{slug}/{version}")
    }
}

/// A third party's registry, built: every bundle and the index.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ThirdPartyRegistry {
    /// Every specification version's bundle, in index order.
    pub bundles: Vec<BuiltBundle>,
    /// The canonical bytes of `index.json`.
    pub index: String,
}

/// Build the bundles of a third party's `specs/` tree, whose Type URIs sit
/// under `type_uri_prefix` — `https://tasks.example.com`,
/// `did:web:example.com`, `urn:example:tasks` (SPEC §6.1, §6.5).
///
/// What is checked, beyond what building a bundle already requires (front
/// matter with a known `status`, a well-formed policy and error codes,
/// resolvable `$ref`s, an `extends` naming an earlier MINOR):
///
/// - the prefix is not under `https://trusttasks.org/` (SPEC §6.5 item 1) and
///   not `http` (SPEC §6.1), and yields a Type URI that reads back as its own
///   slug and version;
/// - every slug and version satisfies SPEC §6.1 / §5.1, and no slug is a
///   framework slug (SPEC §6.5 item 2);
/// - front matter `slug` / `version`, when given, match the directories, and
///   `supersededBy` appears only on a `retired` or `withdrawn` version;
/// - the payload schema's `$id` is the Type URI and its `$schema` is 2020-12
///   (SPEC §6.3);
/// - an `extends` names a version that exists.
pub fn build_third_party(
    specs_dir: &Path,
    type_uri_prefix: &str,
    refs: &Resolver,
) -> Result<ThirdPartyRegistry> {
    let prefix = type_uri_prefix.trim_end_matches('/');
    if is_official_type_uri(&format!("{prefix}/")) {
        return Err(anyhow!(
            "--type-uri-prefix {prefix}: https://trusttasks.org is the public registry's \
             authority, which a third-party specification must not claim (SPEC §6.5 item 1)"
        ));
    }
    if prefix
        .split_once(':')
        .is_some_and(|(s, _)| s.eq_ignore_ascii_case("http"))
    {
        return Err(anyhow!(
            "--type-uri-prefix {prefix}: a Type URI's scheme must not be http (SPEC §6.1)"
        ));
    }
    let specs = discover_specs(specs_dir, &[])?;
    if specs.is_empty() {
        return Err(anyhow!(
            "no specifications under {} — expected <slug>/<M.N>/payload.schema.json beside a spec.md",
            specs_dir.display()
        ));
    }
    let lifecycles = compute_lifecycles(specs_dir)?;
    let mut ordered: Vec<(SpecDir, Option<String>)> = Vec::new();
    for spec in specs {
        let ctx = format!("{}/{}", spec.slug, spec.version);
        if !is_spec_slug(&spec.slug) {
            return Err(anyhow!(
                "{ctx}: `{}` is not a SPEC §6.1 slug (lowercase segments of a-z, 0-9 and single \
                 hyphens, each starting with a letter)",
                spec.slug
            ));
        }
        if !is_spec_version(&spec.version) {
            return Err(anyhow!(
                "{ctx}: `{}` is not a SPEC §5.1 MAJOR.MINOR version",
                spec.version
            ));
        }
        if is_reserved_slug(&spec.slug) {
            return Err(anyhow!(
                "{ctx}: SPEC §6.1 reserves trust-task* and trust-ceremony* slugs for the \
                 framework, on every authority (SPEC §6.5 item 2)"
            ));
        }
        let deprecation = match lifecycles.get(&(spec.slug.clone(), spec.version.clone())) {
            Some(Lifecycle::Deprecated(note)) => Some(note.clone()),
            Some(_) => None,
            None => {
                return Err(anyhow!(
                    "{ctx}: has a payload.schema.json but no spec.md to read its status from"
                ))
            }
        };
        let md = spec.spec_md_path();
        let fm = read_front_matter(&md)?
            .ok_or_else(|| anyhow!("{}: no YAML front matter", md.display()))?;
        for (key, want) in [("slug", &spec.slug), ("version", &spec.version)] {
            if let Some(given) = fm.get(key) {
                if given.as_str() != Some(want.as_str()) {
                    return Err(anyhow!(
                        "{}: front matter {key} {given} does not match the directory ({want})",
                        md.display()
                    ));
                }
            }
        }
        let status = fm.get("status").and_then(Value::as_str).unwrap_or("");
        if fm.get("supersededBy").is_some() && is_live(status) {
            return Err(anyhow!(
                "{}: supersededBy is only permitted when status is 'retired' or 'withdrawn'",
                md.display()
            ));
        }
        ordered.push((spec, deprecation));
    }
    ordered.sort_by(|(a, _), (b, _)| {
        (a.slug.as_str(), parse_version(&a.version)).cmp(&(&b.slug, parse_version(&b.version)))
    });

    let versions: BTreeSet<(String, String)> = ordered
        .iter()
        .map(|(s, _)| (s.slug.clone(), s.version.clone()))
        .collect();
    let mut bundles = Vec::with_capacity(ordered.len());
    for (spec, deprecation) in &ordered {
        let ctx = format!("{}/{}", spec.slug, spec.version);
        let type_uri = type_uri_under(prefix, &spec.slug, &spec.version);
        match type_uri_parts(&type_uri) {
            Some((s, v)) if s == spec.slug && v == spec.version => {}
            _ => {
                return Err(anyhow!(
                    "{ctx}: --type-uri-prefix {prefix} gives {type_uri}, which is not a SPEC §6.1 \
                     Type URI naming this slug and version"
                ))
            }
        }
        let raw: Value = serde_json::from_str(
            &fs::read_to_string(&spec.schema_path)
                .with_context(|| format!("read {}", spec.schema_path.display()))?,
        )
        .with_context(|| format!("parse {}", spec.schema_path.display()))?;
        if raw.get("$id").and_then(Value::as_str) != Some(type_uri.as_str()) {
            return Err(anyhow!(
                "{}: $id must be the Type URI {type_uri} (SPEC §6.3), not {}",
                spec.schema_path.display(),
                raw.get("$id")
                    .map_or("absent".to_string(), Value::to_string)
            ));
        }
        if raw.get("$schema").and_then(Value::as_str)
            != Some("https://json-schema.org/draft/2020-12/schema")
        {
            return Err(anyhow!(
                "{}: $schema must be https://json-schema.org/draft/2020-12/schema (SPEC §6.3)",
                spec.schema_path.display()
            ));
        }
        let built = build_bundle(spec, &type_uri, deprecation.as_deref(), refs)
            .with_context(|| format!("failed to bundle {ctx}"))?;
        if let Some(target) = &built.extends {
            if !versions.contains(&(spec.slug.clone(), target.clone())) {
                return Err(anyhow!(
                    "{ctx}: extends names {target}, which is not a version of {}",
                    spec.slug
                ));
            }
        }
        bundles.push(built);
    }
    let index = render_index(&bundles, None, &[])?;
    Ok(ThirdPartyRegistry { bundles, index })
}

#[cfg(test)]
mod tests;
