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
//!
//! A third-party source (`[sources.<name>]`) serves the same layout, at an
//! `https://` URL or from a directory, for specifications under its own
//! authority (SPEC §6.5). Whichever it comes from, a bundle is held to the
//! identity it is pinned as: its Type URI names its slug and version, and sits
//! under `https://trusttasks.org/` exactly when it comes from the official
//! registry.

use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;

use crate::manifest::{check_source_url, SourceLoc};
use crate::names::{check_slug, digest_hex, digest_of, TaskId, Version};

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

/// A PEM bundle of root certificates to trust instead of the Mozilla roots —
/// for a source (or a registry mirror) behind an internal CA. When set, it is
/// the whole trust store: include public roots too if trusttasks.org must
/// also be reached.
pub const CA_FILE_ENV: &str = "TRUST_TASKS_CA_FILE";

fn http_agent() -> Result<ureq::Agent> {
    let mut config = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(120)))
        .user_agent(concat!("trust-tasks-cli/", env!("CARGO_PKG_VERSION")));
    if let Some(path) = std::env::var_os(CA_FILE_ENV).filter(|p| !p.is_empty()) {
        let pem = std::fs::read(&path)
            .with_context(|| format!("reading {CA_FILE_ENV}={}", Path::new(&path).display()))?;
        let certs = ureq::tls::parse_pem(&pem)
            .filter_map(|item| match item {
                Ok(ureq::tls::PemItem::Certificate(c)) => Some(Ok(c)),
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<std::result::Result<Vec<_>, _>>()
            .with_context(|| format!("{CA_FILE_ENV}: not a PEM certificate bundle"))?;
        if certs.is_empty() {
            bail!(
                "{CA_FILE_ENV}={} holds no certificates",
                Path::new(&path).display()
            );
        }
        config = config.tls_config(
            ureq::tls::TlsConfig::builder()
                .root_certs(ureq::tls::RootCerts::new_with_certs(&certs))
                .build(),
        );
    }
    Ok(config.build().into())
}

/// The identity rules a pinned version's Type URI is held to, from the index
/// or from the bundle itself: it names the version's slug and version (SPEC
/// §6.1, in any form), and it is under `https://trusttasks.org/` exactly when
/// the task is official — a third-party source claiming the registry's
/// authority is refused (SPEC §6.5 item 1), and so is an "official" bundle
/// outside it.
pub fn check_identity(id: &TaskId, v: Version, type_uri: &str) -> Result<()> {
    match trust_tasks_build::type_uri_parts(type_uri) {
        Some((slug, version)) if slug == id.slug && version == v.to_string() => {}
        Some((slug, version)) => {
            bail!("{id} {v}: its Type URI {type_uri} names {slug} {version}, not {id} {v}")
        }
        None => bail!("{id} {v}: {type_uri} is not a SPEC §6.1 Type URI"),
    }
    match &id.source {
        None if type_uri != trust_tasks_build::official_type_uri(&id.slug, &v.to_string()) => bail!(
            "{id} {v}: the official registry served Type URI {type_uri}, which is not under {}/spec/ — a specification under another authority is pinned from a [sources.<name>] as <name>:{}",
            trust_tasks_build::OFFICIAL_AUTHORITY,
            id.slug
        ),
        Some(source) if trust_tasks_build::is_official_type_uri(type_uri) => bail!(
            "{id} {v}: source `{source}` served Type URI {type_uri}, which claims the public registry's authority ({}/) — a third-party source may not (SPEC §6.5 item 1)",
            trust_tasks_build::OFFICIAL_AUTHORITY
        ),
        _ => Ok(()),
    }
}

enum Source {
    Http { base: String, agent: ureq::Agent },
    Dir(PathBuf),
}

pub struct Registry {
    source: Source,
    /// "the registry", or "source `acme`" — for messages.
    pub label: String,
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
    /// "the registry", or "source `acme`".
    pub label: String,
}

impl Index {
    pub fn parse(bytes: &[u8], label: &str) -> Result<Index> {
        let raw: RawIndex = serde_json::from_slice(bytes)
            .with_context(|| format!("{label}'s specs/index.json is not a valid index"))?;
        if raw.index_version != INDEX_VERSION {
            bail!(
                "{label}'s index is indexVersion {}; this trust-tasks reads indexVersion {INDEX_VERSION} — upgrade trust-tasks-cli",
                raw.index_version
            );
        }
        let mut by_slug: BTreeMap<String, BTreeMap<Version, IndexEntry>> = BTreeMap::new();
        for e in raw.specs {
            check_slug(&e.slug).with_context(|| format!("{label}'s index entry {}", e.type_uri))?;
            digest_hex(&e.digest)
                .with_context(|| format!("{label}'s index entry {}", e.type_uri))?;
            let v = Version::parse(&e.version).ok_or_else(|| {
                anyhow!(
                    "{label}'s index entry {}: bad version `{}`",
                    e.type_uri,
                    e.version
                )
            })?;
            by_slug.entry(e.slug.clone()).or_default().insert(v, e);
        }
        Ok(Index {
            by_slug,
            excluded: raw.excluded,
            label: label.to_string(),
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
        let label = &self.label;
        if let Some(v) = self.by_slug.get(slug) {
            return Ok(v);
        }
        if let Some(x) = self.excluded.iter().find(|x| x.slug == slug) {
            if label == "the registry" {
                bail!("`{slug}` has no registry bundle: {}", x.reason);
            }
            bail!("`{slug}` has no bundle in {label}: {}", x.reason);
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
            bail!("`{slug}` is not in {label}");
        }
        bail!(
            "`{slug}` is not in {label}; did you mean one of: {}",
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
    /// The official registry (or a mirror of it): `--registry`, the manifest's
    /// `registry`, or trusttasks.org.
    pub fn open(url: &str) -> Result<Registry> {
        let url = normalize(url);
        let source = if url.starts_with("https://") || url.starts_with("http://") {
            Source::Http {
                base: url.clone(),
                agent: http_agent()?,
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
        Ok(Registry {
            source,
            label: "the registry".to_string(),
        })
    }

    /// The third-party source `name`: an `https://` site root, or a directory
    /// relative to `project_root`.
    pub fn open_source(name: &str, loc: &SourceLoc, project_root: &Path) -> Result<Registry> {
        let source = match loc {
            SourceLoc::Url(url) => {
                check_source_url(name, url)?;
                Source::Http {
                    base: normalize(url),
                    agent: http_agent()?,
                }
            }
            SourceLoc::Path(p) => Source::Dir(project_root.join(p)),
        };
        Ok(Registry {
            source,
            label: format!("source `{name}`"),
        })
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

    fn is_official(&self) -> bool {
        self.label == "the registry"
    }

    /// "the registry index", or "source `acme`'s index".
    fn index_name(&self) -> String {
        if self.is_official() {
            "the registry index".to_string()
        } else {
            format!("{}'s index", self.label)
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
            let what = if self.is_official() {
                "registry index".to_string()
            } else {
                format!("index for {}", self.label)
            };
            anyhow!(
                "no {what} at {} — a registry is a site root with specs/index.json under it",
                self.describe(rel)
            )
        })?;
        Index::parse(&bytes, &self.label)
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
        id: &TaskId,
        v: Version,
        digest: &str,
        entry: Option<&IndexEntry>,
    ) -> Result<Fetched> {
        let slug = id;
        let label = &self.label;
        let hex = digest_hex(digest)?;
        let by_digest = format!("specs/by-digest/sha256-{hex}.json");
        if let Some(bytes) = self.get(&by_digest)? {
            if digest_of(&bytes) == digest {
                check_bundle(&bytes, id, v)?;
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
                "{slug} {v}: {label} no longer has the bundle with digest {digest} — it is not in the by-digest store ({}), and the version's current bytes have a different digest",
                self.describe(&by_digest)
            );
        };
        let rel = format!("specs/{}", entry.path);
        let bytes = self.get(&rel)?.ok_or_else(|| {
            anyhow!(
                "{slug} {v}: {} lists {} but it is missing",
                self.index_name(),
                self.describe(&rel)
            )
        })?;
        let got = digest_of(&bytes);
        if got != digest {
            bail!(
                "digest mismatch: {} is {got}, but {} says {digest}",
                self.describe(&rel),
                self.index_name()
            );
        }
        check_bundle(&bytes, id, v)?;
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

/// The bundle is for the version it was fetched as, under the authority its
/// origin may serve ([`check_identity`]).
pub fn check_bundle(bytes: &[u8], id: &TaskId, v: Version) -> Result<BundleHead> {
    let head = read_bundle_head(bytes).with_context(|| format!("{id} {v}"))?;
    if head.slug != id.slug || Version::parse(&head.version) != Some(v) {
        bail!(
            "{id} {v}: the bundle is for {} {}, not {id} {v}",
            head.slug,
            head.version
        );
    }
    check_identity(id, v, &head.type_uri)?;
    Ok(head)
}
