//! `trust-tasks.lock` — the resolution, written only by this CLI.
//!
//! ```toml
//! version = 1
//! registry = "https://trusttasks.org"
//!
//! [[task]]
//! slug = "vault/upsert"
//! version = "0.4"
//! type_uri = "https://trusttasks.org/spec/vault/upsert/0.4"
//! digest = "sha256:<hex>"
//! ```
//!
//! One `[[task]]` per pinned version, sorted by slug, then numerically by
//! version. Parsed with `toml`, rendered by hand so the layout is fixed.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;

use crate::names::{check_slug, digest_hex, Version};

pub const LOCK_VERSION: u64 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockEntry {
    pub slug: String,
    pub version: Version,
    pub type_uri: String,
    pub digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lock {
    pub registry: String,
    /// Keyed by (slug, version), which is also the file's order.
    pub entries: BTreeMap<(String, Version), LockEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLock {
    version: u64,
    registry: String,
    #[serde(default)]
    task: Vec<RawEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntry {
    slug: String,
    version: String,
    type_uri: String,
    digest: String,
}

impl Lock {
    pub fn new(registry: String) -> Lock {
        Lock {
            registry,
            entries: BTreeMap::new(),
        }
    }

    pub fn load(path: &Path) -> Result<Option<Lock>> {
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e).with_context(|| format!("reading {}", path.display())),
        };
        Lock::parse(&text)
            .with_context(|| format!("{} is not a valid lockfile", path.display()))
            .map(Some)
    }

    pub fn parse(text: &str) -> Result<Lock> {
        let raw: RawLock = toml::from_str(text)?;
        if raw.version != LOCK_VERSION {
            bail!(
                "lockfile version {}; this trust-tasks reads version {LOCK_VERSION} — upgrade trust-tasks-cli",
                raw.version
            );
        }
        let mut entries = BTreeMap::new();
        for e in raw.task {
            check_slug(&e.slug)?;
            digest_hex(&e.digest).with_context(|| format!("{} {}", e.slug, e.version))?;
            let v = Version::parse(&e.version).ok_or_else(|| {
                anyhow!("{}: `{}` is not a MAJOR.MINOR version", e.slug, e.version)
            })?;
            let key = (e.slug.clone(), v);
            if entries.contains_key(&key) {
                bail!("{} {v} is locked twice", e.slug);
            }
            entries.insert(
                key,
                LockEntry {
                    slug: e.slug,
                    version: v,
                    type_uri: e.type_uri,
                    digest: e.digest,
                },
            );
        }
        Ok(Lock {
            registry: raw.registry,
            entries,
        })
    }

    pub fn render(&self) -> String {
        let mut out = String::from("# Written by `trust-tasks`. Do not edit by hand.\n");
        out.push_str(&format!("version = {LOCK_VERSION}\n"));
        out.push_str(&format!("registry = {}\n", quote(&self.registry)));
        for e in self.entries.values() {
            out.push_str("\n[[task]]\n");
            out.push_str(&format!("slug = {}\n", quote(&e.slug)));
            out.push_str(&format!("version = {}\n", quote(&e.version.to_string())));
            out.push_str(&format!("type_uri = {}\n", quote(&e.type_uri)));
            out.push_str(&format!("digest = {}\n", quote(&e.digest)));
        }
        out
    }

    pub fn for_slug<'a>(&'a self, slug: &'a str) -> impl Iterator<Item = &'a LockEntry> + 'a {
        self.entries.values().filter(move |e| e.slug == slug)
    }

    pub fn remove_slug(&mut self, slug: &str) -> Vec<LockEntry> {
        let keys: Vec<_> = self
            .entries
            .keys()
            .filter(|(s, _)| s == slug)
            .cloned()
            .collect();
        keys.iter().filter_map(|k| self.entries.remove(k)).collect()
    }

    pub fn insert(&mut self, e: LockEntry) {
        self.entries.insert((e.slug.clone(), e.version), e);
    }
}

/// A TOML basic string.
fn quote(s: &str) -> String {
    toml::Value::String(s.to_string()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_is_sorted_and_stable() {
        let mut lock = Lock::new("https://trusttasks.org".into());
        for (slug, v) in [
            ("vault/upsert", "0.10"),
            ("acl/grant", "0.2"),
            ("vault/upsert", "0.9"),
        ] {
            let version = Version::parse(v).unwrap();
            lock.insert(LockEntry {
                slug: slug.into(),
                version,
                type_uri: format!("https://trusttasks.org/spec/{slug}/{v}"),
                digest: format!("sha256:{}", "a".repeat(64)),
            });
        }
        let text = lock.render();
        let order: Vec<_> = text
            .lines()
            .filter(|l| l.starts_with("version = \""))
            .collect();
        assert_eq!(
            order,
            [
                "version = \"0.2\"",
                "version = \"0.9\"",
                "version = \"0.10\""
            ]
        );
        let again = Lock::parse(&text).unwrap();
        assert_eq!(again, lock);
        assert_eq!(again.render(), text);
    }

    #[test]
    fn rejects_unknown_members_and_versions() {
        assert!(Lock::parse("version = 2\nregistry = \"x\"\n").is_err());
        assert!(Lock::parse("version = 1\nregistry = \"x\"\nextra = 1\n").is_err());
    }
}
