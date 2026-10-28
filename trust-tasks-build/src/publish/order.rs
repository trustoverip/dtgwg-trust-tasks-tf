//! `propertyOrder`, `defsOrder` and `defOrigins`: the authored order of every
//! `properties` object and of the root `$defs`, and where each spliced
//! definition was written — which the bundle's key-sorted (canonical),
//! self-contained schemas lose.

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Map, Value};

use super::refs::{split_external_ref, Loc, Resolver};
use super::{SpecDir, SpecSchemas};

/// For every schema object in `schema` and `responseSchema` that has
/// `properties`, the authored order of their names — keyed by the JSON
/// Pointer, from the bundle root, of the object owning `properties`
/// (`/schema`, `/schema/$defs/AclEntry`).
///
/// The bundle's schemas are key-sorted (canonical JSON), which loses the order
/// the TypeScript, Go and Dart generators give struct fields. Each pointer is
/// looked up in the document the definition was written in: the payload
/// schema, or — for a `$defs` entry spliced in from a shared document — that
/// document. The names must be exactly the bundled schema's; anything else is
/// a bug in the splice, and stops the build.
pub(crate) fn property_order(
    spec: &SpecDir,
    schemas: &SpecSchemas,
    refs: &Resolver,
) -> Result<Value> {
    let mut sources: BTreeMap<Loc, Ordered> = BTreeMap::new();
    let mut out = Map::new();
    let own = Loc::File(spec.schema_path.clone());
    for (root, value) in [
        ("/schema", Some(&schemas.schema)),
        ("/responseSchema", schemas.response.as_ref()),
    ] {
        let Some(value) = value else { continue };
        let mut owners = Vec::new();
        collect_property_owners(value, &mut Vec::new(), &mut owners);
        for (segments, sorted_names) in owners {
            let source = match segments.as_slice() {
                [defs, name, ..] if defs == "$defs" => schemas
                    .def_origins
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| own.clone()),
                _ => own.clone(),
            };
            if !sources.contains_key(&source) {
                let text = refs.load(&source)?;
                let parsed: Ordered = serde_json::from_str(&text)
                    .with_context(|| format!("parse {}", show(&source)))?;
                sources.insert(source.clone(), parsed);
            }
            let pointer = format!("{root}{}", json_pointer(&segments));
            let authored = sources[&source]
                .at(&segments)
                .and_then(|owner| owner.get("properties"))
                .and_then(Ordered::keys)
                .ok_or_else(|| {
                    anyhow!(
                        "{pointer}: no `properties` at {} in {}",
                        json_pointer(&segments),
                        show(&source)
                    )
                })?;
            let mut check = authored.clone();
            check.sort();
            if check != sorted_names {
                return Err(anyhow!(
                    "{pointer}: the authored members {authored:?} in {} are not the bundled \
                     schema's {sorted_names:?}",
                    show(&source)
                ));
            }
            out.insert(pointer, json!(authored));
        }
    }
    Ok(Value::Object(out))
}

/// `defsOrder` and `defOrigins`.
///
/// The bundle's `$defs` are key-sorted. The TypeScript, Go and Dart generators
/// declare a schema's definitions in the order their `$ref` inliners leave
/// `$defs` in: the payload schema's own, as authored, then each spliced one in
/// the order a last-in-first-out frontier of `$ref`s (collected in document
/// order) reaches it. This replays that walk over the authored documents —
/// the same frontier [`super::refs`] uses, but in authored rather than sorted
/// member order — and stops the build if the names it reaches are not exactly
/// the bundled `$defs`.
///
/// `defOrigins` names, for each definition spliced in from another document,
/// the definition it is: `<that document's $id>#/$defs/<name>`, so a generator
/// can tell two bundles' copies of one shared component from two definitions
/// that only share a name. A document with no `$id` is named by its path under
/// the specifications root, below the Type URI prefix.
pub(crate) fn defs_order(
    spec: &SpecDir,
    schemas: &SpecSchemas,
    type_uri: &str,
    refs: &Resolver,
) -> Result<(Value, Value)> {
    let bundled: Vec<String> = match schemas.schema.get("$defs") {
        Some(Value::Object(defs)) => defs.keys().cloned().collect(),
        _ => Vec::new(),
    };
    let own_loc = Loc::File(spec.schema_path.clone());
    let own: Ordered = serde_json::from_str(&refs.load(&own_loc)?)
        .with_context(|| format!("parse {}", spec.schema_path.display()))?;
    let mut order: Vec<String> = own.get("$defs").and_then(Ordered::keys).unwrap_or_default();
    let own_count = order.len();

    let external = |r: &str| !r.starts_with('#');
    let mut docs: BTreeMap<Loc, Ordered> = BTreeMap::new();
    let mut origins = Map::new();
    let mut found = Vec::new();
    own.refs(&external, &mut found);
    let base = super::refs::file_base(&spec.schema_path)?;
    let mut frontier: Vec<(String, Loc)> = found.into_iter().map(|r| (r, base.clone())).collect();
    let mut seen: HashSet<String> = HashSet::new();
    while let Some((reference, owner)) = frontier.pop() {
        let (rel, name) = split_external_ref(&reference).ok_or_else(|| {
            anyhow!("external $ref {reference:?} is not of the form <path>#/$defs/<name>")
        })?;
        let target = refs.locate(rel, &owner)?;
        if !seen.insert(format!("{}#/$defs/{name}", show(&target))) {
            continue;
        }
        if !docs.contains_key(&target) {
            let text = refs.load(&target)?;
            let parsed: Ordered =
                serde_json::from_str(&text).with_context(|| format!("parse {}", show(&target)))?;
            docs.insert(target.clone(), parsed);
        }
        let fragment = docs[&target]
            .get("$defs")
            .and_then(|d| d.get(name))
            .ok_or_else(|| anyhow!("{} has no $defs/{name}", show(&target)))?;
        if !order.iter().any(|n| n == name) {
            order.push(name.to_string());
            let id = match docs[&target].get("$id") {
                Some(Ordered::Str(id)) => id.clone(),
                _ => unnamed_origin(spec, type_uri, &target, refs),
            };
            origins.insert(name.to_string(), json!(format!("{id}#/$defs/{name}")));
        }
        let mut more = Vec::new();
        fragment.refs(&external, &mut more);
        let fragment_base = target.base()?;
        frontier.extend(more.into_iter().map(|r| (r, fragment_base.clone())));
        let mut internal = Vec::new();
        fragment.refs(&|r: &str| r.starts_with('#'), &mut internal);
        frontier.extend(
            internal
                .into_iter()
                .map(|local| (format!("{rel}{local}"), owner.clone())),
        );
    }

    let mut check = order.clone();
    check.sort();
    if check != bundled {
        return Err(anyhow!(
            "the authored $defs order {order:?} ({own_count} of them the schema's own) is not \
             the bundled schema's $defs {bundled:?}"
        ));
    }
    let mut by_root = Map::new();
    if !order.is_empty() {
        by_root.insert("/schema".into(), json!(order));
        if schemas.response.is_some() {
            by_root.insert("/responseSchema".into(), json!(order));
        }
    }
    Ok((Value::Object(by_root), Value::Object(origins)))
}

/// The identity of a shared document that declares no `$id`: its path under
/// the specifications root (`.schema.json` / `.json` dropped), below the
/// Type URI prefix the specification's own Type URI was built on — as the
/// registry's `$id`s are (`https://trusttasks.org/spec/acl/_shared/0.2/acl-entry`).
fn unnamed_origin(spec: &SpecDir, type_uri: &str, doc: &Loc, refs: &Resolver) -> String {
    let path = match doc {
        Loc::Url(url) => return url.clone(),
        Loc::File(path) => path,
    };
    let depth = spec.slug.split('/').count() + 1;
    let own_root = spec
        .schema_path
        .parent()
        .and_then(|p| p.ancestors().nth(depth))
        .and_then(|p| p.canonicalize().ok());
    let relative = |root: &Path| {
        path.strip_prefix(root).ok().map(|rel| {
            let rel = rel.to_string_lossy().replace('\\', "/");
            rel.strip_suffix(".schema.json")
                .or_else(|| rel.strip_suffix(".json"))
                .unwrap_or(&rel)
                .to_string()
        })
    };
    if let Some(rel) = own_root.as_deref().and_then(relative) {
        let slash = format!("/{}/{}", spec.slug, spec.version);
        let colon = format!(":{}:{}", spec.slug.replace('/', ":"), spec.version);
        if let Some(prefix) = type_uri.strip_suffix(&slash) {
            return format!("{prefix}/{rel}");
        }
        if let Some(prefix) = type_uri.strip_suffix(&colon) {
            return format!("{prefix}:{}", rel.replace('/', ":"));
        }
        return rel;
    }
    if let Some(rel) = refs
        .official_specs()
        .and_then(|root| root.canonicalize().ok().and_then(|root| relative(&root)))
    {
        return format!("{}/spec/{rel}", crate::names::OFFICIAL_AUTHORITY);
    }
    path.display().to_string()
}

fn show(loc: &Loc) -> String {
    match loc {
        Loc::File(p) => p.display().to_string(),
        Loc::Url(u) => u.clone(),
    }
}

/// The keywords whose value maps *names* to schemas, rather than being one.
const SCHEMA_MAP_KEYWORDS: &[&str] = &[
    "properties",
    "$defs",
    "definitions",
    "patternProperties",
    "dependentSchemas",
];

/// The keywords whose value is data, not a schema, and is not walked.
const DATA_KEYWORDS: &[&str] = &["enum", "const", "default", "examples", "example"];

/// Every object under `value` that has a `properties` map: its path segments
/// and its property names, sorted. Schema-aware, so a member that happens to
/// be called `properties` is not mistaken for the keyword.
#[doc(hidden)]
pub fn collect_property_owners(
    value: &Value,
    path: &mut Vec<String>,
    out: &mut Vec<(Vec<String>, Vec<String>)>,
) {
    match value {
        Value::Object(map) => {
            if let Some(Value::Object(props)) = map.get("properties") {
                let mut names: Vec<String> = props.keys().cloned().collect();
                names.sort();
                out.push((path.clone(), names));
            }
            for (key, child) in map {
                if DATA_KEYWORDS.contains(&key.as_str()) {
                    continue;
                }
                path.push(key.clone());
                match child {
                    Value::Object(entries) if SCHEMA_MAP_KEYWORDS.contains(&key.as_str()) => {
                        for (name, schema) in entries {
                            path.push(name.clone());
                            collect_property_owners(schema, path, out);
                            path.pop();
                        }
                    }
                    _ => collect_property_owners(child, path, out),
                }
                path.pop();
            }
        }
        Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                path.push(i.to_string());
                collect_property_owners(item, path, out);
                path.pop();
            }
        }
        _ => {}
    }
}

/// RFC 6901: each segment prefixed with `/`, `~` written `~0` and `/` `~1`.
fn json_pointer(segments: &[String]) -> String {
    segments
        .iter()
        .map(|s| format!("/{}", s.replace('~', "~0").replace('/', "~1")))
        .collect()
}

/// A JSON value that keeps its objects' member order — which
/// `serde_json::Value` does not here, because nothing in this build enables
/// `preserve_order` (and turning it on would reorder every generated module).
#[derive(Debug)]
enum Ordered {
    Object(Vec<(String, Ordered)>),
    Array(Vec<Ordered>),
    Str(String),
    Scalar,
}

impl Ordered {
    fn get(&self, key: &str) -> Option<&Ordered> {
        match self {
            Ordered::Object(members) => members.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    fn at(&self, segments: &[String]) -> Option<&Ordered> {
        segments.iter().try_fold(self, |node, seg| match node {
            Ordered::Array(items) => seg.parse::<usize>().ok().and_then(|i| items.get(i)),
            _ => node.get(seg),
        })
    }

    /// Every `$ref` string under `self` that `keep` accepts, in document
    /// order — the order the TypeScript, Go and Dart generators' `collectRefs`
    /// walks a parsed schema in.
    fn refs(&self, keep: &impl Fn(&str) -> bool, out: &mut Vec<String>) {
        match self {
            Ordered::Object(members) => {
                if let Some(Ordered::Str(r)) = self.get("$ref") {
                    if keep(r) {
                        out.push(r.clone());
                    }
                }
                for (_, value) in members {
                    value.refs(keep, out);
                }
            }
            Ordered::Array(items) => {
                for item in items {
                    item.refs(keep, out);
                }
            }
            Ordered::Str(_) | Ordered::Scalar => {}
        }
    }

    fn keys(&self) -> Option<Vec<String>> {
        match self {
            Ordered::Object(members) => Some(members.iter().map(|(k, _)| k.clone()).collect()),
            _ => None,
        }
    }
}

impl<'de> serde::Deserialize<'de> for Ordered {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> serde::de::Visitor<'de> for V {
            type Value = Ordered;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("any JSON value")
            }
            fn visit_bool<E>(self, _: bool) -> Result<Ordered, E> {
                Ok(Ordered::Scalar)
            }
            fn visit_i64<E>(self, _: i64) -> Result<Ordered, E> {
                Ok(Ordered::Scalar)
            }
            fn visit_u64<E>(self, _: u64) -> Result<Ordered, E> {
                Ok(Ordered::Scalar)
            }
            fn visit_f64<E>(self, _: f64) -> Result<Ordered, E> {
                Ok(Ordered::Scalar)
            }
            fn visit_str<E>(self, v: &str) -> Result<Ordered, E> {
                Ok(Ordered::Str(v.to_string()))
            }
            fn visit_unit<E>(self) -> Result<Ordered, E> {
                Ok(Ordered::Scalar)
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Ordered, A::Error> {
                let mut items = Vec::new();
                while let Some(item) = seq.next_element()? {
                    items.push(item);
                }
                Ok(Ordered::Array(items))
            }
            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Ordered, A::Error> {
                let mut members: Vec<(String, Ordered)> = Vec::new();
                while let Some((key, value)) = map.next_entry::<String, Ordered>()? {
                    // serde_json keeps the last of a duplicated key; so does this.
                    members.retain(|(k, _)| *k != key);
                    members.push((key, value));
                }
                Ok(Ordered::Object(members))
            }
        }
        deserializer.deserialize_any(V)
    }
}
