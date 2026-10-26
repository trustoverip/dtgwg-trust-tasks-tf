//! `trust-tasks.lock` and the bundles vendored beside it.
//!
//! The lockfile is written by the `trust-tasks` CLI, never by hand: one
//! `[[task]]` per pinned specification version, with the SHA-256 of its
//! bundle. The bundles sit in `trust-tasks/<slug>/<M.N>.bundle.json` next to
//! it, byte-identical to the registry's, so `sha256(file) == digest`.
//!
//! ```toml
//! version = 1
//! registry = "https://trusttasks.org"
//!
//! [[task]]
//! slug = "vault/upsert"
//! version = "0.4"
//! type_uri = "https://trusttasks.org/spec/vault/upsert/0.4"
//! digest = "sha256:…"
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::bundle::Bundle;

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
    /// One entry per pinned version, sorted by slug, then version.
    #[serde(default, rename = "task")]
    pub tasks: Vec<LockedTask>,
}

/// One pinned specification version.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct LockedTask {
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
        let mut seen = std::collections::BTreeSet::new();
        for task in &lock.tasks {
            if !seen.insert((task.slug.as_str(), task.version.as_str())) {
                return Err(anyhow!(
                    "{}/{} is locked more than once",
                    task.slug,
                    task.version
                ));
            }
        }
        Ok(lock)
    }
}

impl LockedTask {
    /// Where this task's bundle is vendored under `bundle_dir`:
    /// `<bundle_dir>/<slug>/<M.N>.bundle.json`.
    pub fn bundle_path(&self, bundle_dir: &Path) -> PathBuf {
        let mut path = bundle_dir.to_path_buf();
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
                self.slug,
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
