//! Where an application's files live, and writing them.
//!
//! ```text
//! trust-tasks.toml                          intent (the author's)
//! trust-tasks.lock                          resolution (this CLI's)
//! trust-tasks/<slug>/<M.N>.bundle.json      vendored bundles, byte-identical to the registry's
//! ```

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::lock::{Lock, LockEntry};
use crate::manifest::Manifest;
use crate::names::{digest_of, Version};
use crate::registry::{normalize, DEFAULT_REGISTRY};

pub const MANIFEST: &str = "trust-tasks.toml";
pub const LOCKFILE: &str = "trust-tasks.lock";
pub const VENDOR_DIR: &str = "trust-tasks";
pub const BUNDLE_SUFFIX: &str = ".bundle.json";

pub struct Project {
    pub root: PathBuf,
}

impl Project {
    /// `dir` if given; otherwise the nearest directory at or above the current
    /// one holding a `trust-tasks.toml`. With `create`, a directory without one
    /// is accepted (the current one, or `dir`).
    pub fn locate(dir: Option<&Path>, create: bool) -> Result<Project> {
        if let Some(dir) = dir {
            if !dir.is_dir() {
                bail!("{} is not a directory", dir.display());
            }
            let p = Project {
                root: dir.to_path_buf(),
            };
            if !create && !p.manifest_path().exists() {
                bail!(
                    "no {MANIFEST} in {} — `trust-tasks add <slug>` starts one",
                    dir.display()
                );
            }
            return Ok(p);
        }
        let cwd = std::env::current_dir().context("reading the current directory")?;
        for d in cwd.ancestors() {
            if d.join(MANIFEST).is_file() {
                return Ok(Project {
                    root: d.to_path_buf(),
                });
            }
        }
        if create {
            return Ok(Project { root: cwd });
        }
        bail!(
            "no {MANIFEST} in {} or any parent directory — `trust-tasks add <slug>` starts one",
            cwd.display()
        )
    }

    pub fn manifest_path(&self) -> PathBuf {
        self.root.join(MANIFEST)
    }

    pub fn lock_path(&self) -> PathBuf {
        self.root.join(LOCKFILE)
    }

    pub fn vendor_dir(&self) -> PathBuf {
        self.root.join(VENDOR_DIR)
    }

    /// `trust-tasks/<slug>/<M.N>.bundle.json`, relative to the root.
    pub fn vendored_rel(slug: &str, v: Version) -> String {
        format!("{VENDOR_DIR}/{slug}/{v}{BUNDLE_SUFFIX}")
    }

    pub fn vendored(&self, slug: &str, v: Version) -> PathBuf {
        self.root.join(Project::vendored_rel(slug, v))
    }

    /// The vendored bundle for `e` is present and is the locked bytes.
    pub fn vendored_ok(&self, e: &LockEntry) -> bool {
        std::fs::read(self.vendored(&e.slug, e.version)).is_ok_and(|b| digest_of(&b) == e.digest)
    }

    /// The manifest, or a new empty one when `create`.
    pub fn manifest(&self, create: bool) -> Result<Manifest> {
        let path = self.manifest_path();
        if path.exists() {
            Manifest::load(&path)
        } else if create {
            Ok(Manifest::new(path))
        } else {
            bail!("no {MANIFEST} in {}", self.root.display())
        }
    }

    /// Writes the result of a command: new bundles, then the lock, then the
    /// manifest — so an interrupted run leaves a lock that names files already
    /// on disk. Vendored bundles of entries `old` had and `lock` does not are
    /// removed. Unchanged files are not rewritten.
    pub fn commit(
        &self,
        manifest: &Manifest,
        old: Option<&Lock>,
        lock: &Lock,
        bundles: &BTreeMap<(String, Version), Vec<u8>>,
    ) -> Result<()> {
        for ((slug, v), bytes) in bundles {
            write_if_changed(&self.vendored(slug, *v), bytes.as_slice())?;
        }
        write_if_changed(&self.lock_path(), lock.render().as_bytes())?;
        write_if_changed(&self.manifest_path(), manifest.render().as_bytes())?;
        if let Some(old) = old {
            for key in old.entries.keys() {
                if !lock.entries.contains_key(key) {
                    self.remove_vendored(&key.0, key.1)?;
                }
            }
        }
        Ok(())
    }

    /// Deletes a vendored bundle and any directories it leaves empty, up to
    /// `trust-tasks/` itself.
    pub fn remove_vendored(&self, slug: &str, v: Version) -> Result<()> {
        let path = self.vendored(slug, v);
        match std::fs::remove_file(&path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e).with_context(|| format!("removing {}", path.display())),
        }
        let stop = self.vendor_dir();
        let mut dir = path.parent();
        while let Some(d) = dir {
            if d == stop || !d.starts_with(&stop) {
                break;
            }
            // Not empty (or already gone): stop climbing.
            if std::fs::remove_dir(d).is_err() {
                break;
            }
            dir = d.parent();
        }
        Ok(())
    }

    /// Every `*.bundle.json` under `trust-tasks/`, as root-relative paths with
    /// `/` separators, sorted.
    pub fn vendored_files(&self) -> Result<Vec<String>> {
        let mut out = Vec::new();
        let base = self.vendor_dir();
        if base.is_dir() {
            walk(&base, &mut |p| {
                if p.to_string_lossy().ends_with(BUNDLE_SUFFIX) {
                    let rel = p.strip_prefix(&self.root).unwrap_or(p);
                    let rel: Vec<String> = rel
                        .components()
                        .map(|c| c.as_os_str().to_string_lossy().into_owned())
                        .collect();
                    out.push(rel.join("/"));
                }
            })?;
        }
        out.sort();
        Ok(out)
    }
}

fn walk(dir: &Path, f: &mut dyn FnMut(&Path)) -> Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .collect::<std::io::Result<_>>()?;
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let p = e.path();
        if e.file_type()?.is_dir() {
            walk(&p, f)?;
        } else {
            f(&p);
        }
    }
    Ok(())
}

fn write_if_changed(path: &Path, bytes: &[u8]) -> Result<()> {
    if std::fs::read(path).is_ok_and(|b| b == bytes) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    // Write beside, then rename: a reader never sees half a file.
    let tmp = path.with_extension("tmp~");
    std::fs::write(&tmp, bytes).with_context(|| format!("writing {}", tmp.display()))?;
    std::fs::rename(&tmp, path).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

/// The registry to use: `--registry` / `TRUST_TASKS_REGISTRY`, else the
/// manifest's `registry`, else trusttasks.org.
pub fn effective_registry(manifest: &Manifest, flag: Option<&str>) -> Result<String> {
    Ok(normalize(&match flag {
        Some(r) => r.to_string(),
        None => manifest
            .registry()?
            .unwrap_or_else(|| DEFAULT_REGISTRY.to_string()),
    }))
}
