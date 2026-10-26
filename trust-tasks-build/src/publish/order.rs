//! `propertyOrder`: the authored order of every `properties` object, which the
//! bundle's key-sorted (canonical) schemas lose.

use std::collections::BTreeMap;

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Map, Value};

use super::refs::{Loc, Resolver};
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
            fn visit_str<E>(self, _: &str) -> Result<Ordered, E> {
                Ok(Ordered::Scalar)
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
