//! Inlining cross-file `$ref`s, so a bundle's schemas are self-contained.
//!
//! A `$ref` of the form `<location>#/$defs/<name>` is resolved by loading the
//! referenced document and splicing its `$defs.<name>` into the referencing
//! schema's own `$defs`, then rewriting the `$ref` to `#/$defs/<name>`. The
//! location is either
//!
//! - a path relative to the file holding the `$ref` — the registry's own
//!   `../../_shared/0.2/acl-entry.schema.json`, or a third party's shared files;
//! - an absolute `https://trusttasks.org/…` URL naming one of the registry's
//!   shared components, which a third-party schema may use: either the served
//!   file (`https://trusttasks.org/specs/acl/_shared/0.2/acl-entry.schema.json`)
//!   or the component's `$id` (`https://trusttasks.org/spec/acl/_shared/0.2/acl-entry`).
//!   Those are read from a checkout of the registry's `specs/` when the
//!   [`Resolver`] has one, and otherwise through its fetch function — the
//!   publisher's network, never a consumer's: the bundle that results carries
//!   the inlined definitions, so nobody downstream dereferences anything.
//!   With a checkout an `$id` is looked up among its shared schemas; without
//!   one it is mapped to a file by convention (`spec/<path>` →
//!   `specs/<path>.schema.json`), which not every component's `$id` follows —
//!   the served-file URL is the form that always works.
//!
//! Refs inside a spliced definition are resolved against the document it came
//! from, transitively.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use crate::names::OFFICIAL_AUTHORITY;

/// Fetches the text at an `https://trusttasks.org/specs/…` URL.
pub type Fetch<'a> = dyn Fn(&str) -> Result<String> + 'a;

/// Where a referenced schema document is: a file, or a registry URL.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Loc {
    File(PathBuf),
    Url(String),
}

impl Loc {
    fn display(&self) -> String {
        match self {
            Loc::File(p) => p.display().to_string(),
            Loc::Url(u) => u.clone(),
        }
    }

    /// The location relative refs inside this document resolve against: the
    /// file's directory, or the URL itself (its last segment is dropped when
    /// joining).
    fn base(&self) -> Result<Loc> {
        Ok(match self {
            Loc::File(p) => Loc::File(
                p.parent()
                    .ok_or_else(|| anyhow!("referenced file has no parent dir"))?
                    .to_path_buf(),
            ),
            Loc::Url(u) => Loc::Url(u.clone()),
        })
    }
}

/// How `$ref`s outside a specification's own tree are resolved: the
/// registry's shared components, by absolute `https://trusttasks.org` URL.
///
/// [`Resolver::local`] resolves only relative files — what the registry's own
/// specifications use — and refuses an absolute URL.
pub struct Resolver<'a> {
    official_specs: Option<PathBuf>,
    fetch: Option<&'a Fetch<'a>>,
    cache: RefCell<BTreeMap<String, String>>,
    /// `$id` → file, over the checkout's shared schemas; built on first use.
    ids: RefCell<Option<BTreeMap<String, PathBuf>>>,
}

impl<'a> Resolver<'a> {
    /// Relative files only.
    pub fn local() -> Self {
        Self {
            official_specs: None,
            fetch: None,
            cache: RefCell::default(),
            ids: RefCell::default(),
        }
    }

    /// Read the registry's shared components from `specs_dir`, a checkout of
    /// the registry's `specs/` directory.
    pub fn with_official_specs(mut self, specs_dir: impl Into<PathBuf>) -> Self {
        self.official_specs = Some(specs_dir.into());
        self
    }

    /// Fetch the registry's shared components with `fetch`, given the URL of
    /// the served file (`https://trusttasks.org/specs/<path>.schema.json`).
    /// Used only when no checkout was given.
    pub fn with_fetch(mut self, fetch: &'a Fetch<'a>) -> Self {
        self.fetch = Some(fetch);
        self
    }

    /// Where `reference` (the part of a `$ref` before `#`) points, from a
    /// document whose base is `base`.
    fn locate(&self, reference: &str, base: &Loc) -> Result<Loc> {
        if let Some((scheme, _)) = reference.split_once("://") {
            if !scheme.eq_ignore_ascii_case("https") {
                return Err(anyhow!(
                    "$ref {reference:?}: only https:// URLs are resolved (SPEC §6.1 forbids http)"
                ));
            }
            return self.official(reference);
        }
        match base {
            Loc::File(dir) => {
                let abs = dir.join(reference);
                let canonical = fs::canonicalize(&abs).with_context(|| {
                    format!(
                        "$ref target {} (from {}) does not exist",
                        abs.display(),
                        dir.display()
                    )
                })?;
                Ok(Loc::File(canonical))
            }
            Loc::Url(url) => self.official(&join_url(url, reference)?),
        }
    }

    /// An absolute URL naming one of the registry's shared components, mapped
    /// to the checkout's file or to the served file's URL.
    fn official(&self, url: &str) -> Result<Loc> {
        let rest = url
            .strip_prefix(OFFICIAL_AUTHORITY)
            .and_then(|r| r.strip_prefix('/'))
            .ok_or_else(|| {
                anyhow!(
                    "$ref {url:?}: an absolute $ref must name one of the registry's shared \
                     components under {OFFICIAL_AUTHORITY}/ — put any other shared schema \
                     beside your specifications and $ref it by relative path"
                )
            })?;
        // The served file, or the component's `$id` (which, by convention,
        // drops the `.schema.json` and says `spec/` where the site says
        // `specs/`).
        let rel = if let Some(r) = rest.strip_prefix("specs/") {
            r.to_string()
        } else if let Some(r) = rest.strip_prefix("spec/") {
            if let Some(path) = self.by_id(url)? {
                return Ok(Loc::File(path));
            }
            format!("{r}.schema.json")
        } else {
            return Err(anyhow!(
                "$ref {url:?} names no shared component: expected \
                 {OFFICIAL_AUTHORITY}/specs/<path>.schema.json or a component's $id"
            ));
        };
        if rel
            .split('/')
            .any(|s| s.is_empty() || s == "." || s == "..")
        {
            return Err(anyhow!(
                "$ref {url:?}: a path segment is empty, `.` or `..`"
            ));
        }
        match &self.official_specs {
            Some(dir) => {
                let path = dir.join(&rel);
                let canonical = fs::canonicalize(&path).with_context(|| {
                    format!(
                        "$ref {url:?}: {} does not exist in the registry checkout",
                        path.display()
                    )
                })?;
                Ok(Loc::File(canonical))
            }
            None => Ok(Loc::Url(format!("{OFFICIAL_AUTHORITY}/specs/{rel}"))),
        }
    }

    /// The checkout's shared schema whose `$id` is `url`, if there is a
    /// checkout and one is.
    fn by_id(&self, url: &str) -> Result<Option<PathBuf>> {
        let Some(dir) = &self.official_specs else {
            return Ok(None);
        };
        if self.ids.borrow().is_none() {
            let mut ids = BTreeMap::new();
            let mut files = Vec::new();
            super::walk_files(dir, &mut files)?;
            for path in files {
                let shared = path
                    .strip_prefix(dir)
                    .ok()
                    .and_then(Path::parent)
                    .is_some_and(|p| p.iter().any(|c| c.to_string_lossy().starts_with('_')));
                if !shared || !path.to_string_lossy().ends_with(".schema.json") {
                    continue;
                }
                let Ok(text) = fs::read_to_string(&path) else {
                    continue;
                };
                if let Some(id) = serde_json::from_str::<Value>(&text)
                    .ok()
                    .and_then(|v| v.get("$id").and_then(Value::as_str).map(str::to_string))
                {
                    ids.insert(id, fs::canonicalize(&path)?);
                }
            }
            *self.ids.borrow_mut() = Some(ids);
        }
        Ok(self
            .ids
            .borrow()
            .as_ref()
            .and_then(|ids| ids.get(url).cloned()))
    }

    /// The text of the document at `loc`.
    pub(crate) fn load(&self, loc: &Loc) -> Result<String> {
        match loc {
            Loc::File(p) => fs::read_to_string(p).with_context(|| format!("read {}", p.display())),
            Loc::Url(url) => {
                if let Some(text) = self.cache.borrow().get(url) {
                    return Ok(text.clone());
                }
                let fetch = self.fetch.ok_or_else(|| {
                    anyhow!(
                        "$ref {url:?} names a registry shared component, and neither a \
                         registry checkout (--official-specs) nor fetching was allowed"
                    )
                })?;
                let text = fetch(url).with_context(|| format!("fetch {url}"))?;
                self.cache.borrow_mut().insert(url.clone(), text.clone());
                Ok(text)
            }
        }
    }
}

/// `reference` resolved against the document at `url` (RFC 3986 §5.2 for the
/// relative paths a schema uses: `..` and `.` segments).
fn join_url(url: &str, reference: &str) -> Result<String> {
    let root = format!("{OFFICIAL_AUTHORITY}/");
    let path = url
        .strip_prefix(&root)
        .ok_or_else(|| anyhow!("{url} is not a registry URL"))?;
    let mut segments: Vec<&str> = path.split('/').collect();
    segments.pop(); // the document's own name
    for seg in reference.split('/') {
        match seg {
            "." | "" => {}
            ".." => {
                segments.pop().ok_or_else(|| {
                    anyhow!("$ref {reference:?} from {url} climbs above the site")
                })?;
            }
            s => segments.push(s),
        }
    }
    Ok(format!("{root}{}", segments.join("/")))
}

/// Inline every cross-file `$ref` in `schema`, which was read from a document
/// whose base is `base`.
pub(crate) fn resolve_cross_file_refs(
    schema: &mut Value,
    base: &Loc,
    refs: &Resolver,
) -> Result<()> {
    resolve_cross_file_refs_tracked(schema, base, refs, &mut BTreeMap::new())
}

/// [`resolve_cross_file_refs`], recording in `origins` the document each
/// spliced `$defs` entry came from.
pub(crate) fn resolve_cross_file_refs_tracked(
    schema: &mut Value,
    base: &Loc,
    refs: &Resolver,
    origins: &mut BTreeMap<String, Loc>,
) -> Result<()> {
    let mut frontier: Vec<(Value, Loc)> = collect_external_refs(schema, base);
    let mut seen: HashSet<String> = HashSet::new();
    while let Some((ref_value, owner)) = frontier.pop() {
        let ref_str = ref_value
            .as_str()
            .ok_or_else(|| anyhow!("$ref value was not a string"))?
            .to_string();
        let (rel_path, def_name) = split_external_ref(&ref_str).ok_or_else(|| {
            anyhow!("external $ref {ref_str:?} is not of the form <path>#/$defs/<name>")
        })?;
        let target = refs.locate(rel_path, &owner)?;
        let dedupe_key = format!("{}#/$defs/{}", target.display(), def_name);
        if !seen.insert(dedupe_key) {
            continue;
        }
        let referenced: Value = serde_json::from_str(&refs.load(&target)?)
            .with_context(|| format!("parse referenced schema {}", target.display()))?;
        let fragment = referenced
            .get("$defs")
            .and_then(|v| v.get(def_name))
            .ok_or_else(|| {
                anyhow!(
                    "{} has no $defs/{} (referenced from {})",
                    target.display(),
                    def_name,
                    owner.display()
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
            origins.insert(def_name.to_string(), target.clone());
        }

        // The fragment's own external refs (transitive) are resolved against
        // the document it came from, not the original base.
        let referenced_base = target.base()?;
        frontier.extend(collect_external_refs(&fragment, &referenced_base));

        // The fragment may also contain *internal* refs (`#/$defs/X`) pointing
        // at sibling defs in the source document. Without splicing those too,
        // typify panics with `$ref #/definitions/X is missing` after
        // `migrate_defs_to_definitions` rewrites the references. Reformulate
        // each internal ref as a synthetic external ref against the same
        // source document so the splice path treats them uniformly.
        let synthetic_external: Vec<Value> = collect_internal_refs(&fragment)
            .into_iter()
            .map(|local| Value::String(format!("{rel_path}{local}")))
            .collect();
        for synth in synthetic_external {
            frontier.push((synth, owner.clone()));
        }
    }

    // After splicing, rewrite every external $ref string to the local form.
    rewrite_external_refs_local(schema);
    Ok(())
}

/// Every `$ref` in `schema` whose value is external (not `#`-prefixed), with
/// the base to resolve it against.
fn collect_external_refs(schema: &Value, base: &Loc) -> Vec<(Value, Loc)> {
    let mut out = Vec::new();
    walk_external_refs(schema, &mut |r| out.push((r.clone(), base.clone())));
    out
}

/// Every internal `$ref` value (those that start with `#`).
fn collect_internal_refs(schema: &Value) -> Vec<String> {
    let mut out = Vec::new();
    walk_internal_refs(schema, &mut |s| out.push(s.to_string()));
    out
}

fn walk_internal_refs(value: &Value, sink: &mut impl FnMut(&str)) {
    match value {
        Value::Object(map) => {
            if let Some(s) = map.get("$ref").and_then(Value::as_str) {
                if s.starts_with('#') {
                    sink(s);
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

fn walk_external_refs(value: &Value, sink: &mut impl FnMut(&Value)) {
    match value {
        Value::Object(map) => {
            if let Some(r) = map.get("$ref") {
                if r.as_str().is_some_and(|s| !s.starts_with('#')) {
                    sink(r);
                }
            }
            for v in map.values() {
                walk_external_refs(v, sink);
            }
        }
        Value::Array(items) => {
            for v in items {
                walk_external_refs(v, sink);
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

/// Parse `"<location>#/$defs/<name>"` into its two halves.
fn split_external_ref(s: &str) -> Option<(&str, &str)> {
    let hash = s.find('#')?;
    let path = &s[..hash];
    let fragment = &s[hash + 1..];
    let def_name = fragment.strip_prefix("/$defs/")?;
    if path.is_empty() || def_name.is_empty() || def_name.contains('/') {
        return None;
    }
    Some((path, def_name))
}

/// The directory a specification's `payload.schema.json` sits in, as a
/// [`Loc`] base.
pub(crate) fn file_base(schema_path: &Path) -> Result<Loc> {
    Ok(Loc::File(
        schema_path
            .parent()
            .ok_or_else(|| anyhow!("schema path has no parent"))?
            .to_path_buf(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_refs_from_a_registry_url_stay_on_the_registry() {
        let from = "https://trusttasks.org/specs/acl/_shared/0.2/acl-entry.schema.json";
        assert_eq!(
            join_url(from, "../../../_framework/0.5/framework.schema.json").unwrap(),
            "https://trusttasks.org/specs/_framework/0.5/framework.schema.json"
        );
        assert!(join_url(from, "../../../../../x.json").is_err());
    }

    #[test]
    fn absolute_refs_must_name_a_registry_component() {
        let r = Resolver::local();
        let base = Loc::File(PathBuf::from("/nowhere"));
        assert_eq!(
            r.locate(
                "https://trusttasks.org/spec/acl/_shared/0.2/acl-entry",
                &base
            )
            .unwrap(),
            Loc::Url("https://trusttasks.org/specs/acl/_shared/0.2/acl-entry.schema.json".into())
        );
        for bad in [
            "http://trusttasks.org/specs/a.schema.json",
            "https://example.com/specs/a.schema.json",
            "https://trusttasks.org/other/a.schema.json",
            "https://trusttasks.org/specs/../a.schema.json",
        ] {
            assert!(r.locate(bad, &base).is_err(), "{bad}");
        }
        // Without a checkout or a fetch function, nothing is read.
        assert!(r
            .load(&Loc::Url("https://trusttasks.org/specs/a.json".into()))
            .is_err());
    }
}
