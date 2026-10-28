//! `trust-tasks.lock` — the resolution, written only by this CLI.
//!
//! ```toml
//! version = 1
//! registry = "https://trusttasks.org"
//!
//! [sources.acme]
//! url = "https://tasks.acme.example"
//!
//! [[task]]
//! slug = "vault/upsert"
//! version = "0.4"
//! type_uri = "https://trusttasks.org/spec/vault/upsert/0.4"
//! digest = "sha256:<hex>"
//!
//! [[task]]
//! source = "acme"
//! slug = "payments/settle"
//! version = "1.0"
//! type_uri = "https://tasks.acme.example/spec/payments/settle/1.0"
//! digest = "sha256:<hex>"
//! ```
//!
//! `[sources.*]` are the sources at least one task comes from, as the manifest
//! declares them. One `[[task]]` per pinned version: official tasks first,
//! then by source name, then slug, then numerically by version. Parsed with
//! `toml`, rendered by hand so the layout is fixed — and every lock this CLI
//! reads or writes is also held to `trust-tasks-build`'s reading of it, so a
//! lock it writes is one generation accepts.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;

use crate::manifest::{check_source_url, SourceLoc};
use crate::names::{check_source_name, digest_hex, TaskId, Version};

pub const LOCK_VERSION: u64 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockEntry {
    pub source: Option<String>,
    pub slug: String,
    pub version: Version,
    pub type_uri: String,
    pub digest: String,
}

impl LockEntry {
    pub fn id(&self) -> TaskId {
        TaskId::new(self.source.as_deref(), &self.slug)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lock {
    pub registry: String,
    /// The sources the entries come from.
    pub sources: BTreeMap<String, SourceLoc>,
    /// Keyed by (task, version), which is also the file's order.
    pub entries: BTreeMap<(TaskId, Version), LockEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLock {
    version: u64,
    registry: String,
    #[serde(default)]
    sources: BTreeMap<String, RawSource>,
    #[serde(default)]
    task: Vec<RawEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSource {
    url: Option<String>,
    path: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawEntry {
    #[serde(default)]
    source: Option<String>,
    slug: String,
    version: String,
    type_uri: String,
    digest: String,
}

impl Lock {
    pub fn new(registry: String) -> Lock {
        Lock {
            registry,
            sources: BTreeMap::new(),
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
        let mut sources = BTreeMap::new();
        for (name, s) in raw.sources {
            check_source_name(&name)?;
            let loc = match (s.url, s.path) {
                (Some(u), None) => {
                    check_source_url(&name, &u)?;
                    SourceLoc::Url(u)
                }
                (None, Some(p)) => SourceLoc::Path(p),
                _ => bail!("[sources.{name}]: give exactly one of `url` and `path`"),
            };
            sources.insert(name, loc);
        }
        let mut entries = BTreeMap::new();
        for e in raw.task {
            let id = TaskId::parse(&match &e.source {
                Some(s) => format!("{s}:{}", e.slug),
                None => e.slug.clone(),
            })?;
            digest_hex(&e.digest).with_context(|| format!("{id} {}", e.version))?;
            let v = Version::parse(&e.version)
                .ok_or_else(|| anyhow!("{id}: `{}` is not a MAJOR.MINOR version", e.version))?;
            let key = (id.clone(), v);
            if entries.contains_key(&key) {
                bail!("{id} {v} is locked twice");
            }
            entries.insert(
                key,
                LockEntry {
                    source: e.source,
                    slug: e.slug,
                    version: v,
                    type_uri: e.type_uri,
                    digest: e.digest,
                },
            );
        }
        let lock = Lock {
            registry: raw.registry,
            sources,
            entries,
        };
        lock.validate()?;
        Ok(lock)
    }

    /// The rules generation applies, applied here too — the same Type URI
    /// twice, a sourced entry under the registry's authority (or an official
    /// one outside it), a source named for an official family, a source
    /// with no `[sources.*]` table — by asking `trust-tasks-build` to read
    /// the rendered lock.
    pub fn validate(&self) -> Result<()> {
        trust_tasks_build::Lockfile::parse(&self.render())
            .map(|_| ())
            .map_err(|e| anyhow!("{e}"))
    }

    pub fn render(&self) -> String {
        let mut out = String::from("# Written by `trust-tasks`. Do not edit by hand.\n");
        out.push_str(&format!("version = {LOCK_VERSION}\n"));
        out.push_str(&format!("registry = {}\n", quote(&self.registry)));
        for (name, loc) in &self.sources {
            out.push_str(&format!("\n[sources.{name}]\n"));
            match loc {
                SourceLoc::Url(u) => out.push_str(&format!("url = {}\n", quote(u))),
                SourceLoc::Path(p) => out.push_str(&format!("path = {}\n", quote(p))),
            }
        }
        for e in self.entries.values() {
            out.push_str("\n[[task]]\n");
            if let Some(source) = &e.source {
                out.push_str(&format!("source = {}\n", quote(source)));
            }
            out.push_str(&format!("slug = {}\n", quote(&e.slug)));
            out.push_str(&format!("version = {}\n", quote(&e.version.to_string())));
            out.push_str(&format!("type_uri = {}\n", quote(&e.type_uri)));
            out.push_str(&format!("digest = {}\n", quote(&e.digest)));
        }
        out
    }

    pub fn for_task<'a>(&'a self, id: &'a TaskId) -> impl Iterator<Item = &'a LockEntry> + 'a {
        self.entries
            .iter()
            .filter(move |((i, _), _)| i == id)
            .map(|(_, e)| e)
    }

    pub fn remove_task(&mut self, id: &TaskId) -> Vec<LockEntry> {
        let keys: Vec<_> = self
            .entries
            .keys()
            .filter(|(i, _)| i == id)
            .cloned()
            .collect();
        keys.iter().filter_map(|k| self.entries.remove(k)).collect()
    }

    pub fn insert(&mut self, e: LockEntry) {
        self.entries.insert((e.id(), e.version), e);
    }

    /// Records, for every source an entry comes from, where the manifest says
    /// it is; sources no entry uses are dropped.
    pub fn set_sources(&mut self, declared: &BTreeMap<String, SourceLoc>) -> Result<()> {
        let mut sources = BTreeMap::new();
        for e in self.entries.values() {
            if let Some(name) = &e.source {
                let loc = declared.get(name).ok_or_else(|| {
                    anyhow!("{} is locked but source `{name}` is not declared", e.id())
                })?;
                sources.insert(name.clone(), loc.clone());
            }
        }
        self.sources = sources;
        Ok(())
    }
}

/// A TOML basic string.
fn quote(s: &str) -> String {
    toml::Value::String(s.to_string()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(source: Option<&str>, slug: &str, v: &str, type_uri: String) -> LockEntry {
        LockEntry {
            source: source.map(str::to_string),
            slug: slug.into(),
            version: Version::parse(v).unwrap(),
            type_uri,
            digest: format!("sha256:{}", "a".repeat(64)),
        }
    }

    #[test]
    fn round_trip_is_sorted_and_stable() {
        let mut lock = Lock::new("https://trusttasks.org".into());
        for (slug, v) in [
            ("vault/upsert", "0.10"),
            ("acl/grant", "0.2"),
            ("vault/upsert", "0.9"),
        ] {
            lock.insert(entry(
                None,
                slug,
                v,
                format!("https://trusttasks.org/spec/{slug}/{v}"),
            ));
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
    fn sourced_entries_follow_official_ones_with_their_source() {
        let mut lock = Lock::new("https://trusttasks.org".into());
        lock.sources.insert(
            "acme".into(),
            SourceLoc::Url("https://tasks.acme.example".into()),
        );
        lock.insert(entry(
            Some("acme"),
            "payments/settle",
            "1.0",
            "https://tasks.acme.example/spec/payments/settle/1.0".into(),
        ));
        lock.insert(entry(
            None,
            "vault/get",
            "0.3",
            "https://trusttasks.org/spec/vault/get/0.3".into(),
        ));
        let text = lock.render();
        let vault = text.find("slug = \"vault/get\"").unwrap();
        let acme = text
            .find("source = \"acme\"\nslug = \"payments/settle\"")
            .unwrap();
        assert!(text.contains("\n[sources.acme]\nurl = \"https://tasks.acme.example\"\n"));
        assert!(vault < acme, "{text}");
        assert_eq!(Lock::parse(&text).unwrap(), lock);
    }

    #[test]
    fn rejects_unknown_members_and_versions() {
        assert!(Lock::parse("version = 2\nregistry = \"x\"\n").is_err());
        assert!(Lock::parse("version = 1\nregistry = \"x\"\nextra = 1\n").is_err());
        assert!(
            Lock::parse("version = 1\nregistry = \"x\"\n[sources.acme]\nurl = \"http://a\"\n")
                .is_err()
        );
    }

    #[test]
    fn the_authority_rules_hold_on_read() {
        let d = "a".repeat(64);
        let lock = |body: &str| {
            Lock::parse(&format!(
                "version = 1\nregistry = \"https://trusttasks.org\"\n[sources.acme]\nurl = \"https://a.example\"\n{body}"
            ))
        };
        let task = |source: &str, slug: &str, uri: &str| {
            format!("[[task]]\n{source}slug = \"{slug}\"\nversion = \"1.0\"\ntype_uri = \"{uri}\"\ndigest = \"sha256:{d}\"\n")
        };
        // A sourced entry claiming the registry's authority.
        let e = lock(&task(
            "source = \"acme\"\n",
            "x",
            "https://trusttasks.org/spec/x/1.0",
        ));
        assert!(format!("{:#}", e.unwrap_err()).contains("registry's authority"));
        // An official entry outside it.
        let e = lock(&task("", "x", "https://a.example/spec/x/1.0"));
        assert!(format!("{:#}", e.unwrap_err()).contains("official task's Type URI"));
        // Two entries with one Type URI.
        let e = lock(&format!(
            "{}{}",
            task("source = \"acme\"\n", "x", "https://a.example/spec/x/1.0"),
            task("source = \"acme\"\n", "y", "https://a.example/spec/x/1.0")
        ));
        assert!(format!("{:#}", e.unwrap_err()).contains("one Type URI"));
        // A source named for an official family.
        let e = Lock::parse(&format!(
            "version = 1\nregistry = \"https://trusttasks.org\"\n[sources.vault]\nurl = \"https://a.example\"\n{}{}",
            task("", "vault/get", "https://trusttasks.org/spec/vault/get/1.0"),
            task("source = \"vault\"\n", "x", "https://a.example/spec/x/1.0"),
        ));
        assert!(format!("{:#}", e.unwrap_err()).contains("share one generated namespace"));
    }
}
