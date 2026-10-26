//! The registry: `specs/index.json` and the bundles it lists.
//!
//! A registry is a site root — `https://trusttasks.org`, or for a private
//! mirror or an offline test, a `file://` URL or a plain directory laid out the
//! same way — under which:
//!
//! - `specs/index.json` lists every version with its digest and status;
//! - `specs/by-digest/sha256-<hex>.json` holds every bundle ever published, by
//!   digest, append-only;
//! - `specs/<slug>/<M.N>/bundle.json` holds a version's current bytes (a draft
//!   may change in place).

use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;

use crate::names::{check_slug, digest_hex, digest_of, Version};

pub const DEFAULT_REGISTRY: &str = "https://trusttasks.org";

/// The bundle format this CLI reads.
pub const BUNDLE_VERSION: u64 = 1;
const INDEX_VERSION: u64 = 1;
/// Bigger than any bundle or index by orders of magnitude; a bound, not a budget.
const MAX_BODY_BYTES: u64 = 64 << 20;

/// Normalises a registry as written: no trailing `/`.
pub fn normalize(url: &str) -> String {
    url.trim_end_matches('/').to_string()
}

enum Source {
    Http { base: String, agent: ureq::Agent },
    Dir(PathBuf),
}

pub struct Registry {
    source: Source,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexEntry {
    pub type_uri: String,
    pub slug: String,
    pub version: String,
    pub status: String,
    pub deprecation: Option<String>,
    pub extends: Option<String>,
    pub digest: String,
    pub path: String,
}

impl IndexEntry {
    pub fn is_withdrawn(&self) -> bool {
        self.status == "withdrawn"
    }

    /// Live: offered by `latest`. Neither retired nor withdrawn.
    pub fn is_live(&self) -> bool {
        self.status != "withdrawn" && self.status != "retired"
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawIndex {
    index_version: u64,
    #[serde(default)]
    excluded: Vec<Excluded>,
    specs: Vec<IndexEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Excluded {
    pub slug: String,
    pub reason: String,
}

/// `index.json`, keyed by slug, each slug's versions in ascending order.
pub struct Index {
    pub by_slug: BTreeMap<String, BTreeMap<Version, IndexEntry>>,
    pub excluded: Vec<Excluded>,
}

impl Index {
    pub fn parse(bytes: &[u8]) -> Result<Index> {
        let raw: RawIndex = serde_json::from_slice(bytes)
            .context("the registry's specs/index.json is not a valid index")?;
        if raw.index_version != INDEX_VERSION {
            bail!(
                "the registry's index is indexVersion {}; this trust-tasks reads indexVersion {INDEX_VERSION} — upgrade trust-tasks-cli",
                raw.index_version
            );
        }
        let mut by_slug: BTreeMap<String, BTreeMap<Version, IndexEntry>> = BTreeMap::new();
        for e in raw.specs {
            check_slug(&e.slug).with_context(|| format!("registry index entry {}", e.type_uri))?;
            digest_hex(&e.digest)
                .with_context(|| format!("registry index entry {}", e.type_uri))?;
            let v = Version::parse(&e.version).ok_or_else(|| {
                anyhow!(
                    "registry index entry {}: bad version `{}`",
                    e.type_uri,
                    e.version
                )
            })?;
            by_slug.entry(e.slug.clone()).or_default().insert(v, e);
        }
        Ok(Index {
            by_slug,
            excluded: raw.excluded,
        })
    }

    pub fn get(&self, slug: &str, v: Version) -> Option<&IndexEntry> {
        self.by_slug.get(slug)?.get(&v)
    }

    /// The highest live version of `slug`.
    pub fn latest_live(&self, slug: &str) -> Option<(Version, &IndexEntry)> {
        self.by_slug
            .get(slug)?
            .iter()
            .rev()
            .find(|(_, e)| e.is_live())
            .map(|(v, e)| (*v, e))
    }

    /// The versions of `slug`, or an error that says why there are none.
    pub fn versions(&self, slug: &str) -> Result<&BTreeMap<Version, IndexEntry>> {
        if let Some(v) = self.by_slug.get(slug) {
            return Ok(v);
        }
        if let Some(x) = self.excluded.iter().find(|x| x.slug == slug) {
            bail!("`{slug}` has no registry bundle: {}", x.reason);
        }
        let near: Vec<&str> = self
            .by_slug
            .keys()
            .filter(|s| {
                let head = slug.split('/').next().unwrap_or(slug);
                s.starts_with(&format!("{slug}/")) || s.split('/').next() == Some(head)
            })
            .map(String::as_str)
            .take(12)
            .collect();
        if near.is_empty() {
            bail!("`{slug}` is not in the registry");
        }
        bail!(
            "`{slug}` is not in the registry; did you mean one of: {}",
            near.join(", ")
        );
    }

    /// Whether `newer` reaches `older` along declared `extends` — a version that
    /// does can be read as the older one, so moving a pin along it needs no code
    /// change.
    pub fn extends_reaches(&self, slug: &str, newer: Version, older: Version) -> bool {
        let mut cur = newer;
        // A chain is at most as long as the slug has versions; the bound also
        // stops a (malformed) cycle.
        let bound = self.by_slug.get(slug).map_or(0, BTreeMap::len);
        for _ in 0..bound {
            let Some(next) = self
                .get(slug, cur)
                .and_then(|e| e.extends.as_deref())
                .and_then(Version::parse)
            else {
                return false;
            };
            if next == older {
                return true;
            }
            cur = next;
        }
        false
    }
}

/// How a bundle was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Via {
    /// `specs/by-digest/sha256-<hex>.json`.
    Digest,
    /// `specs/<slug>/<M.N>/bundle.json`, because the by-digest store does not
    /// have it (yet).
    Path,
}

pub struct Fetched {
    pub bytes: Vec<u8>,
    pub via: Via,
}

impl Registry {
    pub fn open(url: &str) -> Result<Registry> {
        let url = normalize(url);
        let source = if url.starts_with("https://") || url.starts_with("http://") {
            let config = ureq::Agent::config_builder()
                .timeout_global(Some(Duration::from_secs(120)))
                .user_agent(concat!("trust-tasks-cli/", env!("CARGO_PKG_VERSION")))
                .build();
            Source::Http {
                base: url.clone(),
                agent: config.into(),
            }
        } else {
            let path = url.strip_prefix("file://").unwrap_or(&url);
            if path.is_empty() {
                bail!(
                    "`{url}` is not a registry: give an https:// URL, a file:// URL or a directory"
                );
            }
            Source::Dir(PathBuf::from(path))
        };
        Ok(Registry { source })
    }

    /// `Ok(None)` when the registry has no such file.
    fn get(&self, rel: &str) -> Result<Option<Vec<u8>>> {
        match &self.source {
            Source::Dir(root) => {
                let path = root.join(rel);
                match std::fs::read(&path) {
                    Ok(b) => Ok(Some(b)),
                    Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
                    Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
                }
            }
            Source::Http { base, agent } => {
                let url = format!("{base}/{rel}");
                match agent.get(&url).call() {
                    Ok(mut resp) => {
                        let body = resp
                            .body_mut()
                            .with_config()
                            .limit(MAX_BODY_BYTES)
                            .read_to_vec()
                            .with_context(|| format!("reading {url}"))?;
                        Ok(Some(body))
                    }
                    // S3 answers 403 for a missing key when it may not list the
                    // bucket; either way there is nothing there.
                    Err(ureq::Error::StatusCode(404 | 403)) => Ok(None),
                    Err(e) => Err(anyhow!(e)).with_context(|| format!("fetching {url}")),
                }
            }
        }
    }

    fn describe(&self, rel: &str) -> String {
        match &self.source {
            Source::Http { base, .. } => format!("{base}/{rel}"),
            Source::Dir(root) => root.join(rel).display().to_string(),
        }
    }

    pub fn index(&self) -> Result<Index> {
        let rel = "specs/index.json";
        let bytes = self.get(rel)?.ok_or_else(|| {
            anyhow!(
                "no registry index at {} — a registry is a site root with specs/index.json under it",
                self.describe(rel)
            )
        })?;
        Index::parse(&bytes)
    }

    /// Fetches the bundle whose digest is `digest`, by digest, falling back to
    /// the version's path while the by-digest store is new. The bytes are
    /// verified against `digest` either way.
    ///
    /// `entry` is the index entry for the version; the fallback is taken only
    /// when its digest is the one wanted, because the path serves the
    /// *current* bytes.
    pub fn fetch_bundle(
        &self,
        slug: &str,
        v: Version,
        digest: &str,
        entry: Option<&IndexEntry>,
    ) -> Result<Fetched> {
        let hex = digest_hex(digest)?;
        let by_digest = format!("specs/by-digest/sha256-{hex}.json");
        if let Some(bytes) = self.get(&by_digest)? {
            if digest_of(&bytes) == digest {
                check_bundle(&bytes, slug, v)?;
                return Ok(Fetched {
                    bytes,
                    via: Via::Digest,
                });
            }
            // A JSON body under the content address that is not that content
            // is a broken store, not a missing entry: say so rather than fall
            // back past it. Anything else (an HTML error page served as 200)
            // is treated as absent.
            if serde_json::from_slice::<serde_json::Value>(&bytes).is_ok() {
                bail!(
                    "digest mismatch: {} holds {}, not the {digest} its name promises",
                    self.describe(&by_digest),
                    digest_of(&bytes)
                );
            }
        }
        let Some(entry) = entry.filter(|e| e.digest == digest) else {
            bail!(
                "{slug} {v}: the registry no longer has the bundle with digest {digest} — it is not in the by-digest store ({}), and the version's current bytes have a different digest",
                self.describe(&by_digest)
            );
        };
        let rel = format!("specs/{}", entry.path);
        let bytes = self.get(&rel)?.ok_or_else(|| {
            anyhow!(
                "{slug} {v}: the registry index lists {} but it is missing",
                self.describe(&rel)
            )
        })?;
        let got = digest_of(&bytes);
        if got != digest {
            bail!(
                "digest mismatch: {} is {got}, but the registry index says {digest}",
                self.describe(&rel)
            );
        }
        check_bundle(&bytes, slug, v)?;
        Ok(Fetched {
            bytes,
            via: Via::Path,
        })
    }
}

/// The members of a bundle this CLI reads.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleHead {
    pub bundle_version: u64,
    pub type_uri: String,
    pub slug: String,
    pub version: String,
    pub status: String,
    pub deprecation: Option<String>,
    pub extends: Option<String>,
}

pub fn read_bundle_head(bytes: &[u8]) -> Result<BundleHead> {
    let head: BundleHead = serde_json::from_slice(bytes).context("not a registry bundle")?;
    if head.bundle_version != BUNDLE_VERSION {
        bail!(
            "bundleVersion {} — this trust-tasks reads bundleVersion {BUNDLE_VERSION}; upgrade trust-tasks-cli",
            head.bundle_version
        );
    }
    Ok(head)
}

/// The bundle is for the version it was fetched as.
pub fn check_bundle(bytes: &[u8], slug: &str, v: Version) -> Result<BundleHead> {
    let head = read_bundle_head(bytes).with_context(|| format!("{slug} {v}"))?;
    if head.slug != slug || Version::parse(&head.version) != Some(v) {
        bail!(
            "{slug} {v}: the bundle is for {} {}, not {slug} {v}",
            head.slug,
            head.version
        );
    }
    Ok(head)
}
