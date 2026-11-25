//! Spell each `$defs` entry the prebuilt tree repeats once, in
//! `trust-tasks-rs/src/specs/shared_defs.rs`, and have every
//! `PAYLOAD_SCHEMA` literal that carries it name it instead.
//!
//! The registry's shared schemas (a family's `_shared/` key-roles or
//! device-binding schema, the framework's `Ext`, …) are inlined into every
//! version that references them, so the same definition sat verbatim in
//! hundreds of `PAYLOAD_SCHEMA` strings: about 9.7 MB of `$defs` text of which
//! 2.2 MB was distinct. Nothing else in the crate grows as fast with every new
//! spec version, and it is what took the `.crate` to within a few hundred KiB
//! of crates.io's 10 MiB upload limit.
//!
//! The constant's **value does not change**. A literal
//!
//! ```text
//! Some("{\"$defs\":{\"A\":{…},\"B\":{…}},\"type\":\"object\"}")
//! ```
//!
//! becomes
//!
//! ```text
//! Some(concat!("{\"$defs\":{", crate::specs::shared_defs::d_A_1a2b3c4d!(), ",\"B\":{…}},\"type\":\"object\"}"))
//! ```
//!
//! where the macro expands to the string literal `"\"A\":{…}"`. `concat!`
//! expands it eagerly, so the result is the same `&'static str`, byte for
//! byte, at compile time — the rewrite is checked by reassembling each one and
//! comparing it with the original before anything is written. A literal that
//! cannot be split that way is left as it was.
//!
//! Only the prebuilt tree does this. A module an application's `build.rs`
//! generates with `trust-tasks-build` is one file with nothing to share
//! with, and `generate_module`'s output is unchanged: the rewrite runs here,
//! after it. It is an encoding of a Rust string constant, not a change to any
//! schema, so the TypeScript, Go and Dart generators (whose packages have no
//! upload limit near their size) are not mirrored; see `docs/DEVELOPMENT.md`.

use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use serde_json::Value;

/// The text before each schema literal in a generated module.
const MARKER: &str = "const PAYLOAD_SCHEMA: Option<&'static str> = Some(";

/// The module the shared definitions live in, under `crate::specs`.
pub(crate) const MODULE: &str = "shared_defs";

/// A definition shorter than this stays inline: the macro call that would
/// replace it is about 60 bytes, so a short one saves nothing.
const MIN_SHARED_LEN: usize = 160;

/// A schema literal split into its pieces: `head`, then each `$defs` entry
/// (`"Name":{…}`) with the commas between them, then `tail`. Joined, they are
/// the literal's value.
struct Split {
    head: &'static str,
    entries: Vec<(String, String)>,
    tail: String,
}

/// Split `schema` (the compact JSON a `PAYLOAD_SCHEMA` holds) into its
/// `$defs` entries, or `None` when it has none or does not reassemble.
fn split(schema: &str) -> Option<Split> {
    let Value::Object(map) = serde_json::from_str::<Value>(schema).ok()? else {
        return None;
    };
    let (first, defs) = map.iter().next()?;
    if first != "$defs" {
        return None;
    }
    let Value::Object(defs) = defs else {
        return None;
    };
    let mut entries = Vec::with_capacity(defs.len());
    for (name, def) in defs {
        let key = serde_json::to_string(name).ok()?;
        let value = serde_json::to_string(def).ok()?;
        entries.push((name.clone(), format!("{key}:{value}")));
    }
    let mut rest = map.clone();
    rest.remove("$defs");
    // The generator ends each schema with a newline; keep whatever follows
    // the document.
    let after = &schema[schema.trim_end().len()..];
    let tail = if rest.is_empty() {
        format!("}}}}{after}")
    } else {
        let rest = serde_json::to_string(&Value::Object(rest)).ok()?;
        format!("}},{}{after}", &rest[1..])
    };
    let out = Split {
        head: "{\"$defs\":{",
        entries,
        tail,
    };
    (out.join() == schema).then_some(out)
}

impl Split {
    fn join(&self) -> String {
        let mut s = String::from(self.head);
        for (i, (_, e)) in self.entries.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push_str(e);
        }
        s.push_str(&self.tail);
        s
    }
}

/// Each schema literal in `src`: the byte range of the literal token and its
/// value.
fn literals(src: &str) -> Result<Vec<(std::ops::Range<usize>, String)>> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(at) = src[from..].find(MARKER) {
        let mut start = from + at + MARKER.len();
        start += src[start..].len() - src[start..].trim_start().len();
        from = start;
        if !src[start..].starts_with('"') {
            continue;
        }
        let bytes = src.as_bytes();
        let mut i = start + 1;
        while i < bytes.len() && bytes[i] != b'"' {
            i += if bytes[i] == b'\\' { 2 } else { 1 };
        }
        let end = i + 1;
        let lit: syn::LitStr = syn::parse_str(&src[start..end])
            .map_err(|e| anyhow!("unreadable PAYLOAD_SCHEMA literal: {e}"))?;
        out.push((start..end, lit.value()));
        from = end;
    }
    Ok(out)
}

/// FNV-1a: stable across runs and platforms, which `DefaultHasher` is not
/// promised to be. Only names a macro; a collision is detected, not trusted.
fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

fn macro_name(def_name: &str, entry: &str) -> String {
    let ident: String = def_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    format!("d_{ident}_{:08x}", fnv1a(entry) as u32)
}

/// Rewrite every module's schema literals to name the definitions they share,
/// and return the contents of `shared_defs.rs`.
pub(crate) fn dedup(modules: &mut [&mut String]) -> Result<String> {
    // Count each entry once per literal it appears in.
    let mut uses: BTreeMap<String, (String, usize)> = BTreeMap::new();
    let mut parsed = Vec::with_capacity(modules.len());
    for src in modules.iter() {
        let mut found = Vec::new();
        for (range, value) in literals(src)? {
            if let Some(s) = split(&value) {
                for (name, entry) in &s.entries {
                    if entry.len() >= MIN_SHARED_LEN {
                        uses.entry(entry.clone())
                            .or_insert_with(|| (name.clone(), 0))
                            .1 += 1;
                    }
                }
                found.push((range, s));
            }
        }
        parsed.push(found);
    }

    let mut names: BTreeMap<&str, String> = BTreeMap::new();
    let mut taken: BTreeMap<String, &str> = BTreeMap::new();
    for (entry, (def_name, n)) in &uses {
        if *n < 2 {
            continue;
        }
        let name = macro_name(def_name, entry);
        if let Some(other) = taken.insert(name.clone(), entry) {
            if other != entry {
                return Err(anyhow!("two shared $defs entries hash to macro {name}"));
            }
        }
        names.insert(entry, name);
    }

    for (src, found) in modules.iter_mut().zip(parsed) {
        // Back to front, so earlier ranges stay valid.
        for (range, s) in found.into_iter().rev() {
            if !s
                .entries
                .iter()
                .any(|(_, e)| names.contains_key(e.as_str()))
            {
                continue;
            }
            let mut args: Vec<String> = Vec::new();
            let mut lit = String::from(s.head);
            for (i, (_, entry)) in s.entries.iter().enumerate() {
                if i > 0 {
                    lit.push(',');
                }
                match names.get(entry.as_str()) {
                    Some(name) => {
                        if !lit.is_empty() {
                            args.push(format!("{lit:?}"));
                            lit.clear();
                        }
                        args.push(format!("crate::specs::{MODULE}::{name}!()"));
                    }
                    None => lit.push_str(entry),
                }
            }
            lit.push_str(&s.tail);
            args.push(format!("{lit:?}"));
            src.replace_range(range, &format!("concat!({})", args.join(", ")));
        }
    }

    let mut out = String::from(
        "//! Generated by `trust-tasks-codegen` — do not edit by hand.\n\
         //!\n\
         //! The `$defs` entries more than one `PAYLOAD_SCHEMA` carries, each spelled\n\
         //! once. A schema constant names one with `concat!`, which expands it at\n\
         //! compile time: the constant's value is the full schema, as before. See\n\
         //! `trust-tasks-codegen/src/shared_defs.rs`.\n\
         \n\
         // A family whose feature is off uses none of its definitions.\n\
         #![allow(unused_macros, unused_imports)]\n",
    );
    let mut by_name: Vec<(&String, &str)> = names.iter().map(|(e, n)| (n, *e)).collect();
    by_name.sort();
    for (name, entry) in by_name {
        out.push_str(&format!(
            "\nmacro_rules! {name} {{\n    () => {{\n        {entry:?}\n    }};\n}}\npub(crate) use {name};\n"
        ));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn module(schema: &str) -> String {
        format!(
            "impl crate::Payload for Payload {{\n    {MARKER}\n        {:?},\n    );\n}}\n",
            schema
        )
    }

    fn expand(src: &str, shared: &str) -> String {
        // Resolve each `crate::specs::shared_defs::NAME!()` from `shared` and
        // concatenate, as rustc's `concat!` would.
        use syn::punctuated::Punctuated;
        let start = src.find("concat!(").unwrap();
        let end = src[start..].find("),\n").unwrap() + start + 1;
        let mac: syn::ExprMacro = syn::parse_str(&src[start..end]).unwrap();
        let args = mac
            .mac
            .parse_body_with(Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated)
            .unwrap();
        let lit = |e: &syn::Expr| match e {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) => s.value(),
            other => panic!("not a string literal: {}", quote::quote!(#other)),
        };
        let mut out = String::new();
        for arg in &args {
            match arg {
                syn::Expr::Macro(m) => {
                    let name = m.mac.path.segments.last().unwrap().ident.to_string();
                    let at = shared.find(&format!("macro_rules! {name} {{")).unwrap();
                    let body = shared[at..].lines().nth(2).unwrap().trim();
                    out.push_str(&lit(&syn::parse_str(body).unwrap()));
                }
                other => out.push_str(&lit(other)),
            }
        }
        out
    }

    #[test]
    fn a_shared_definition_is_spelled_once_and_the_value_is_unchanged() {
        let long = "x".repeat(MIN_SHARED_LEN);
        let a = format!(
            r#"{{"$defs":{{"Ext":{{"description":"{long}"}},"Local":{{"type":"string"}}}},"type":"object"}}"#
        );
        let b = format!(r#"{{"$defs":{{"Ext":{{"description":"{long}"}}}},"required":["x"]}}"#);
        let c = r#"{"type":"object"}"#.to_string();
        let (mut ma, mut mb, mut mc) = (module(&a), module(&b), module(&c));
        let original_c = mc.clone();
        let shared = dedup(&mut [&mut ma, &mut mb, &mut mc]).unwrap();
        assert_eq!(shared.matches("macro_rules!").count(), 1, "{shared}");
        assert!(ma.contains("concat!("), "{ma}");
        assert_eq!(expand(&ma, &shared), a);
        assert_eq!(expand(&mb, &shared), b);
        assert_eq!(mc, original_c, "a literal with no $defs is left alone");
    }

    #[test]
    fn a_definition_used_once_stays_inline() {
        let long = "x".repeat(MIN_SHARED_LEN);
        let a = format!(r#"{{"$defs":{{"Only":{{"description":"{long}"}}}},"type":"object"}}"#);
        let mut ma = module(&a);
        let original = ma.clone();
        let shared = dedup(&mut [&mut ma]).unwrap();
        assert_eq!(ma, original);
        assert!(!shared.contains("macro_rules!"));
    }
}
