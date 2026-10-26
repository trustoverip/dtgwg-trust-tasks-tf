//! Slugs, versions, digests — the identifiers every file in a project names.

use std::cmp::Ordering;
use std::fmt;

use anyhow::{bail, Result};
use sha2::{Digest as _, Sha256};

/// A specification version, `MAJOR.MINOR` (SPEC §5). Ordered numerically, so
/// `0.10` sorts after `0.9`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
}

impl Version {
    pub fn parse(s: &str) -> Option<Version> {
        let (major, minor) = s.split_once('.')?;
        let num = |p: &str| -> Option<u32> {
            if p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            p.parse().ok()
        };
        Some(Version {
            major: num(major)?,
            minor: num(minor)?,
        })
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor).cmp(&(other.major, other.minor))
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// Validates a slug: lowercase path segments of `[a-z0-9-]`, as every slug in
/// the registry is. Also what keeps a slug from escaping `trust-tasks/` when it
/// becomes a path.
pub fn check_slug(slug: &str) -> Result<()> {
    let ok = !slug.is_empty()
        && slug.split('/').all(|seg| {
            !seg.is_empty()
                && !seg.starts_with('-')
                && seg
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        });
    if !ok {
        bail!(
            "`{slug}` is not a specification slug (lowercase path segments of a-z, 0-9 and `-`, e.g. `vault/upsert`)"
        );
    }
    Ok(())
}

/// What a command-line task argument asks for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Requested {
    /// `vault/upsert` — the latest live version, pinned exactly.
    Bare,
    /// `vault/upsert@latest` — follow the latest live version on every update.
    Latest,
    /// `vault/upsert/0.4` (or `vault/upsert@0.4`).
    Exact(Version),
}

/// Parses `<slug>[/<M.N>|@latest|@<M.N>]`.
pub fn parse_task_arg(arg: &str) -> Result<(String, Requested)> {
    let (slug, req) = if let Some((slug, at)) = arg.split_once('@') {
        if at == "latest" {
            (slug.to_string(), Requested::Latest)
        } else if let Some(v) = Version::parse(at) {
            (slug.to_string(), Requested::Exact(v))
        } else {
            bail!("`{arg}`: expected `@latest` or `@<MAJOR>.<MINOR>` after the slug");
        }
    } else if let Some((slug, last)) = arg.rsplit_once('/') {
        match Version::parse(last) {
            Some(v) => (slug.to_string(), Requested::Exact(v)),
            None => (arg.to_string(), Requested::Bare),
        }
    } else {
        (arg.to_string(), Requested::Bare)
    };
    check_slug(&slug)?;
    Ok((slug, req))
}

/// `sha256:<hex>` of `bytes` — the digest form of the index and the lockfile.
pub fn digest_of(bytes: &[u8]) -> String {
    let hash = Sha256::digest(bytes);
    let mut s = String::with_capacity(7 + 64);
    s.push_str("sha256:");
    for b in hash {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// The hex part of a well-formed `sha256:<64 hex>` digest.
pub fn digest_hex(digest: &str) -> Result<&str> {
    match digest.strip_prefix("sha256:") {
        Some(hex)
            if hex.len() == 64
                && hex
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) =>
        {
            Ok(hex)
        }
        _ => bail!("`{digest}` is not a digest of the form sha256:<64 lowercase hex digits>"),
    }
}

/// A digest shortened for messages: `sha256:46925ab44000`.
pub fn short(digest: &str) -> &str {
    digest.get(..19).unwrap_or(digest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_order_numerically() {
        let a = Version::parse("0.9").unwrap();
        let b = Version::parse("0.10").unwrap();
        assert!(a < b);
        assert_eq!(b.to_string(), "0.10");
        assert!(Version::parse("1").is_none());
        assert!(Version::parse("1.x").is_none());
        assert!(Version::parse("+1.2").is_none());
    }

    #[test]
    fn task_args() {
        assert_eq!(
            parse_task_arg("vault/upsert/0.4").unwrap(),
            (
                "vault/upsert".into(),
                Requested::Exact(Version::parse("0.4").unwrap())
            )
        );
        assert_eq!(
            parse_task_arg("vault/upsert@latest").unwrap(),
            ("vault/upsert".into(), Requested::Latest)
        );
        assert_eq!(
            parse_task_arg("vault/upsert").unwrap(),
            ("vault/upsert".into(), Requested::Bare)
        );
        assert_eq!(
            parse_task_arg("trust-task-discovery").unwrap(),
            ("trust-task-discovery".into(), Requested::Bare)
        );
        assert!(parse_task_arg("../etc/passwd").is_err());
        assert!(parse_task_arg("Vault/Upsert").is_err());
        assert!(parse_task_arg("vault/upsert@next").is_err());
    }

    #[test]
    fn digests() {
        let d = digest_of(b"");
        assert_eq!(
            d,
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert!(digest_hex(&d).is_ok());
        assert!(digest_hex("sha256:ABC").is_err());
        assert!(digest_hex("md5:00").is_err());
    }
}
