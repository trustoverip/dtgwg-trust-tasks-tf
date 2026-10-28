//! `trust-tasks.lock` and the bundles vendored beside it.
//!
//! The lockfile is written by the `trust-tasks` CLI, never by hand: one
//! `[[task]]` per pinned specification version, with the SHA-256 of its
//! bundle. The bundles sit in `trust-tasks/<slug>/<M.N>.bundle.json` next to
//! it — `trust-tasks/@<source>/<slug>/<M.N>.bundle.json` for a task from a
//! third-party source — byte-identical to the registry's, so
//! `sha256(file) == digest`.
//!
//! ```toml
//! version = 1
//! registry = "https://trusttasks.org"
//!
//! [sources.acme]                      # zero or more
//! url = "https://tasks.acme.example"  # or: path = "../acme-registry"
//!
//! [[task]]
//! slug = "vault/upsert"
//! version = "0.4"
//! type_uri = "https://trusttasks.org/spec/vault/upsert/0.4"
//! digest = "sha256:…"
//!
//! [[task]]
//! source = "acme"
//! slug = "payments/settle"
//! version = "1.0"
//! type_uri = "https://tasks.acme.example/spec/payments/settle/1.0"
//! digest = "sha256:…"
//! ```
//!
//! A source's `url` / `path` is where the CLI fetched from; generation never
//! reads it — no network at build time.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::bundle::Bundle;
use crate::names::{
    is_official_type_uri, is_reserved_slug, is_source_name, official_type_uri, type_uri_parts,
};

/// The lockfile shape this crate reads.
pub const LOCK_VERSION: u32 = 1;

/// A parsed `trust-tasks.lock`.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct Lockfile {
    /// The lockfile shape, `1`.
    pub version: u32,
    /// The registry the versions were resolved against.
    #[serde(default)]
    pub registry: Option<String>,
    /// The third-party sources tasks were resolved from, by name.
    #[serde(default)]
    pub sources: BTreeMap<String, LockedSource>,
    /// One entry per pinned version: official tasks first, then by source
    /// name, then slug, then version.
    #[serde(default, rename = "task")]
    pub tasks: Vec<LockedTask>,
}

/// A `[sources.<name>]` table: where a third-party source was fetched from.
/// Exactly one of `url` and `path`. Generation reads neither.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
#[non_exhaustive]
pub struct LockedSource {
    /// An `https://` site root.
    #[serde(default)]
    pub url: Option<String>,
    /// A directory laid out as a registry site root, relative to the lockfile.
    #[serde(default)]
    pub path: Option<String>,
}

/// One pinned specification version.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct LockedTask {
    /// The `[sources.<name>]` the task comes from, or `None` for the official
    /// registry.
    #[serde(default)]
    pub source: Option<String>,
    /// e.g. `vault/upsert`.
    pub slug: String,
    /// `MAJOR.MINOR`, e.g. `0.4`.
    pub version: String,
    /// The request Type URI.
    pub type_uri: String,
    /// `sha256:<lowercase hex>` of the bundle's bytes.
    pub digest: String,
}

impl Lockfile {
    /// Parse a lockfile's text.
    pub fn parse(text: &str) -> Result<Self, crate::Error> {
        Ok(Self::parse_inner(text)?)
    }

    /// Read and parse the lockfile at `path`.
    pub fn read(path: impl AsRef<Path>) -> Result<Self, crate::Error> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        Ok(Self::parse_inner(&text).with_context(|| format!("in {}", path.display()))?)
    }

    fn parse_inner(text: &str) -> Result<Self> {
        let lock: Lockfile = toml::from_str(text).context("parse trust-tasks.lock")?;
        if lock.version != LOCK_VERSION {
            return Err(anyhow!(
                "lockfile version {} is not supported (this trust-tasks-build reads {})",
                lock.version,
                LOCK_VERSION
            ));
        }
        for (name, source) in &lock.sources {
            if !is_source_name(name) {
                return Err(anyhow!(
                    "[sources.{name}]: a source name is lowercase letters, digits and single \
                     hyphens, starting with a letter"
                ));
            }
            if source.url.is_some() == source.path.is_some() {
                return Err(anyhow!(
                    "[sources.{name}]: give exactly one of `url` and `path`"
                ));
            }
        }
        let mut seen = BTreeSet::new();
        let mut type_uris: BTreeMap<&str, String> = BTreeMap::new();
        for task in &lock.tasks {
            let id = task.display_id();
            if !seen.insert((
                task.source.as_deref(),
                task.slug.as_str(),
                task.version.as_str(),
            )) {
                return Err(anyhow!("{id}/{} is locked more than once", task.version));
            }
            if let Some(other) = type_uris.insert(task.type_uri.as_str(), id.clone()) {
                return Err(anyhow!(
                    "{other} and {id} are both locked as {} — one Type URI names one \
                     specification",
                    task.type_uri
                ));
            }
            task.check_identity()?;
            if let Some(source) = &task.source {
                if !lock.sources.contains_key(source) {
                    return Err(anyhow!(
                        "{id}/{}: names source `{source}`, but the lock has no [sources.{source}]",
                        task.version
                    ));
                }
            }
        }
        check_namespaces(&lock.tasks)?;
        Ok(lock)
    }
}

/// A source's namespace sits beside the official families' in every
/// language's generated tree, so a source may not be named for the first slug
/// segment of an official task in the same lock.
fn check_namespaces(tasks: &[LockedTask]) -> Result<()> {
    let sources: BTreeSet<&str> = tasks.iter().filter_map(|t| t.source.as_deref()).collect();
    for task in tasks.iter().filter(|t| t.source.is_none()) {
        let family = task.slug.split('/').next().unwrap_or(&task.slug);
        if sources.contains(family) {
            return Err(anyhow!(
                "source `{family}` has the name of the official family of {} {} — the two \
                 would share one generated namespace; rename the source in trust-tasks.toml",
                task.slug,
                task.version
            ));
        }
    }
    Ok(())
}

impl LockedTask {
    /// `vault/get`, or `acme:payments/settle` for a sourced task — how the
    /// manifest and the CLI name it.
    pub fn display_id(&self) -> String {
        match &self.source {
            Some(source) => format!("{source}:{}", self.slug),
            None => self.slug.clone(),
        }
    }

    /// The entry's own identity rules, before any bundle is read: the Type URI
    /// names the locked slug and version, sits under the registry's authority
    /// exactly when the task is official (SPEC §6.5 item 1), and a source
    /// serves no framework slug (SPEC §6.5 item 2).
    pub(crate) fn check_identity(&self) -> Result<()> {
        let id = self.display_id();
        let v = &self.version;
        let Some(source) = &self.source else {
            if self.type_uri != official_type_uri(&self.slug, v) {
                return Err(anyhow!(
                    "{id}/{v}: an official task's Type URI is {}, not {} — a task under \
                     another authority comes from a [sources.<name>]",
                    official_type_uri(&self.slug, v),
                    self.type_uri
                ));
            }
            return Ok(());
        };
        if !is_source_name(source) {
            return Err(anyhow!("{id}/{v}: `{source}` is not a source name"));
        }
        if is_reserved_slug(&self.slug) {
            return Err(anyhow!(
                "{id}/{v}: `{}` is a framework slug (SPEC §6.1 reserves trust-task* and \
                 trust-ceremony*), which no source may serve (SPEC §6.5 item 2)",
                self.slug
            ));
        }
        if is_official_type_uri(&self.type_uri) {
            return Err(anyhow!(
                "{id}/{v}: Type URI {} claims the registry's authority, which a third-party \
                 source may not (SPEC §6.5 item 1)",
                self.type_uri
            ));
        }
        match type_uri_parts(&self.type_uri) {
            Some((slug, version)) if slug == self.slug && version == *v => Ok(()),
            Some((slug, version)) => Err(anyhow!(
                "{id}/{v}: Type URI {} names {slug} {version}, not the locked {} {v}",
                self.type_uri,
                self.slug
            )),
            None => Err(anyhow!(
                "{id}/{v}: {} is not a SPEC §6.1 Type URI",
                self.type_uri
            )),
        }
    }

    /// Where this task's bundle is vendored under `bundle_dir`:
    /// `<bundle_dir>/<slug>/<M.N>.bundle.json`, or
    /// `<bundle_dir>/@<source>/<slug>/<M.N>.bundle.json` for a sourced task
    /// (`@` cannot begin a slug, so the two never collide).
    pub fn bundle_path(&self, bundle_dir: &Path) -> PathBuf {
        let mut path = bundle_dir.to_path_buf();
        if let Some(source) = &self.source {
            path.push(format!("@{source}"));
        }
        for segment in self.slug.split('/') {
            path.push(segment);
        }
        path.join(format!("{}.bundle.json", self.version))
    }

    /// Read this task's vendored bundle, refusing it unless its bytes hash to
    /// the locked digest and it names the locked slug, version and Type URI.
    pub fn read_bundle(&self, bundle_dir: &Path) -> Result<Bundle, crate::Error> {
        Ok(self.read_bundle_inner(bundle_dir)?)
    }

    pub(crate) fn read_bundle_inner(&self, bundle_dir: &Path) -> Result<Bundle> {
        let path = self.bundle_path(bundle_dir);
        let bytes = fs::read(&path).with_context(|| {
            format!(
                "read the vendored bundle for {}/{} at {} — run `trust-tasks check`",
                self.display_id(),
                self.version,
                path.display()
            )
        })?;
        let actual = sha256_digest(&bytes);
        if actual != self.digest {
            return Err(anyhow!(
                "vendored bundle does not match trust-tasks.lock — run `trust-tasks check` \
                 ({}: the lock pins {}, the file is {})",
                path.display(),
                self.digest,
                actual
            ));
        }
        let bundle = Bundle::parse_bytes(&bytes).with_context(|| path.display().to_string())?;
        if bundle.slug != self.slug
            || bundle.version != self.version
            || bundle.type_uri != self.type_uri
        {
            return Err(anyhow!(
                "{} is the bundle of {} ({}/{}), but trust-tasks.lock pins it as {} ({}/{})",
                path.display(),
                bundle.type_uri,
                bundle.slug,
                bundle.version,
                self.type_uri,
                self.slug,
                self.version
            ));
        }
        Ok(bundle)
    }
}

/// `sha256:<lowercase hex>` of `bytes` — the digest form a lockfile records.
pub fn sha256_digest(bytes: &[u8]) -> String {
    let hash = Sha256::digest(bytes);
    let mut out = String::with_capacity(7 + 64);
    out.push_str("sha256:");
    for byte in hash {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}
