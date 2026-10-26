//! The identifiers a lockfile and a bundle name: Type URIs (SPEC §6.1),
//! slugs, versions, and the names of third-party sources.
//!
//! A specification's identity is its whole Type URI, never its slug: a slug is
//! unique only within its authority (SPEC §6.1), so `vault/get` from the
//! registry and `vault/get` from a third-party source are different
//! specifications. Everything here reads identity *out of* a Type URI; nothing
//! rebuilds one from a slug except [`official_type_uri`], for the registry's
//! own specifications.

/// The public registry's authority. A Type URI under it (`https://trusttasks.org/…`)
/// is a registry specification; SPEC §6.5 item 1 forbids anything else from
/// claiming one.
pub const OFFICIAL_AUTHORITY: &str = "https://trusttasks.org";

/// The Type URI of the registry specification `slug` at `version`:
/// `https://trusttasks.org/spec/<slug>/<M.N>`.
pub fn official_type_uri(slug: &str, version: &str) -> String {
    format!("{OFFICIAL_AUTHORITY}/spec/{slug}/{version}")
}

/// Whether `type_uri` claims the registry's authority — any URI whose scheme
/// is `https` and whose host is `trusttasks.org`, whatever its path, compared
/// case-insensitively as RFC 3986 compares scheme and host.
pub fn is_official_type_uri(type_uri: &str) -> bool {
    let Some((scheme, rest)) = type_uri.split_once("://") else {
        return false;
    };
    if !scheme.eq_ignore_ascii_case("https") {
        return false;
    }
    let host_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..host_end];
    // Drop any userinfo and port: `x@trusttasks.org:443` still names the host.
    let host = authority.rsplit('@').next().unwrap_or(authority);
    let host = host.split(':').next().unwrap_or(host);
    host.eq_ignore_ascii_case("trusttasks.org")
}

/// The slug and version a Type URI names, read the way SPEC §6.1 says a
/// consumer reads them, in either shape:
///
/// - path form `<prefix>/spec/<slug>/<M.N>`: the slug is the segments between
///   the first `/spec/` and the final segment;
/// - URN form `urn:<nid>:<prefix>:spec:<seg>:…:<M.N>`: the segments between the
///   first `:spec:` and the final segment, rejoined with `/`.
///
/// `None` for anything else: an `http` scheme (non-conformant in every form),
/// a fragment, a path-form URI with an empty prefix, a URN without a
/// namespace and prefix, or a slug or version that fails §6.1 / §5.1.
pub fn type_uri_parts(type_uri: &str) -> Option<(String, String)> {
    if type_uri.contains(['#', '?']) || type_uri.chars().any(char::is_whitespace) {
        return None;
    }
    let (scheme, _) = type_uri.split_once(':')?;
    if !is_scheme(scheme) || scheme.eq_ignore_ascii_case("http") {
        return None;
    }
    let (slug, version) = if scheme.eq_ignore_ascii_case("urn") {
        let (prefix, rest) = type_uri.split_once(":spec:")?;
        // `urn:<nid>:<prefix>` — all three present and non-empty.
        let mut parts = prefix.splitn(3, ':');
        let (_, nid, ns) = (parts.next()?, parts.next()?, parts.next()?);
        if nid.is_empty() || ns.is_empty() {
            return None;
        }
        let (slug, version) = rest.rsplit_once(':')?;
        if slug.contains('/') {
            return None;
        }
        (slug.replace(':', "/"), version.to_string())
    } else {
        let (prefix, rest) = type_uri.split_once("/spec/")?;
        let after_scheme = &prefix[scheme.len() + 1..];
        // `https:/spec/…`, `https:///spec/…` and `did:/spec/…` name nothing.
        if after_scheme.trim_start_matches('/').is_empty() {
            return None;
        }
        if scheme.eq_ignore_ascii_case("https") && !after_scheme.starts_with("//") {
            return None;
        }
        let (slug, version) = rest.rsplit_once('/')?;
        (slug.to_string(), version.to_string())
    };
    (is_spec_slug(&slug) && is_spec_version(&version)).then_some((slug, version))
}

/// RFC 3986 `scheme = ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )`.
fn is_scheme(s: &str) -> bool {
    let mut chars = s.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

/// One slug segment: `^[a-z][a-z0-9]*(-[a-z0-9]+)*$` (SPEC §6.1).
pub fn is_slug_segment(seg: &str) -> bool {
    let bytes = seg.as_bytes();
    let Some(first) = bytes.first() else {
        return false;
    };
    if !first.is_ascii_lowercase() || seg.ends_with('-') || seg.contains("--") {
        return false;
    }
    bytes
        .iter()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
}

/// A slug: one or more `/`-separated segments, each `^[a-z][a-z0-9]*(-[a-z0-9]+)*$`
/// (SPEC §6.1).
pub fn is_spec_slug(slug: &str) -> bool {
    !slug.is_empty() && slug.split('/').all(is_slug_segment)
}

/// `MAJOR.MINOR`, each a non-negative integer without a leading zero (SPEC §5.1).
pub fn is_spec_version(version: &str) -> bool {
    let Some((major, minor)) = version.split_once('.') else {
        return false;
    };
    let num = |p: &str| {
        !p.is_empty()
            && p.bytes().all(|b| b.is_ascii_digit())
            && !(p.len() > 1 && p.starts_with('0'))
            && p.parse::<u32>().is_ok()
    };
    num(major) && num(minor)
}

/// SPEC §6.1's reserved namespace, `^trust-(task|ceremony)($|-|/)`: the
/// framework's own specifications, which every runtime carries and which no
/// third-party source may serve (SPEC §6.5 item 2).
pub fn is_reserved_slug(slug: &str) -> bool {
    ["trust-task", "trust-ceremony"].iter().any(|prefix| {
        slug.strip_prefix(prefix)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with(['-', '/']))
    })
}

/// A source name, as `[sources.<name>]` and `"<name>:<slug>"` write it:
/// `^[a-z][a-z0-9]*(-[a-z0-9]+)*$` — the grammar of one slug segment, so it
/// can stand where a slug's first segment would in every language's
/// generated namespace.
pub fn is_source_name(name: &str) -> bool {
    is_slug_segment(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_spec_6_1_form_is_read() {
        let parts = |s| type_uri_parts(s).map(|(a, b)| format!("{a} {b}"));
        assert_eq!(
            parts("https://trusttasks.org/spec/acl/grant/0.1").as_deref(),
            Some("acl/grant 0.1")
        );
        assert_eq!(
            parts("https://example.com/trust-tasks/spec/acl/grant/0.1").as_deref(),
            Some("acl/grant 0.1")
        );
        assert_eq!(
            parts("did:example:123456789abcdefghi/spec/acl/grant/0.1").as_deref(),
            Some("acl/grant 0.1")
        );
        assert_eq!(
            parts("urn:example:tasks:spec:acl:grant:0.1").as_deref(),
            Some("acl/grant 0.1")
        );
        assert_eq!(
            parts("urn:example:tasks:spec:x:10.0").as_deref(),
            Some("x 10.0")
        );
    }

    #[test]
    fn malformed_and_http_type_uris_are_refused() {
        for bad in [
            "http://example.com/spec/acl/grant/0.1",
            "HTTP://example.com/spec/acl/grant/0.1",
            "https://example.com/spec/acl/grant/0.1#request",
            "https:/spec/acl/grant/0.1",
            "https:///spec/acl/grant/0.1",
            "did:/spec/a/0.1",
            "urn:example:spec:acl:0.1",
            "urn:example:tasks:spec:acl/grant:0.1",
            "https://example.com/spec/Acl/0.1",
            "https://example.com/spec/acl/01.0",
            "https://example.com/spec/acl",
            "example.com/spec/acl/0.1",
        ] {
            assert_eq!(type_uri_parts(bad), None, "{bad}");
        }
    }

    #[test]
    fn the_registry_authority_is_recognised_however_it_is_spelled() {
        assert!(is_official_type_uri("https://trusttasks.org/spec/a/0.1"));
        assert!(is_official_type_uri("HTTPS://TrustTasks.org/spec/a/0.1"));
        assert!(is_official_type_uri("https://trusttasks.org:443/x"));
        assert!(!is_official_type_uri(
            "https://trusttasks.org.evil/spec/a/0.1"
        ));
        assert!(!is_official_type_uri(
            "https://tasks.example.com/spec/a/0.1"
        ));
        assert!(!is_official_type_uri("urn:trusttasks.org:x:spec:a:0.1"));
    }

    #[test]
    fn reserved_slugs_and_source_names() {
        for s in ["trust-task", "trust-task-ok", "trust-ceremony/x"] {
            assert!(is_reserved_slug(s), "{s}");
        }
        for s in ["trust-tasks", "acl/trust-task"] {
            assert!(!is_reserved_slug(s), "{s}");
        }
        for ok in ["acme", "acme-2", "a1"] {
            assert!(is_source_name(ok), "{ok}");
        }
        for bad in [
            "", "Acme", "1acme", "acme_x", "acme--x", "acme-", "a/b", "a:b",
        ] {
            assert!(!is_source_name(bad), "{bad}");
        }
    }
}
