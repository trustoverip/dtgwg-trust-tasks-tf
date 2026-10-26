//! `trust-tasks registry build` — the publisher's side of a third-party
//! source.
//!
//! Turns an ecosystem's `specs/<slug>/<M.N>/{spec.md,payload.schema.json}`
//! tree into the layout trusttasks.org serves, so applications can pin its
//! tasks as `[sources.<name>]`:
//!
//! ```text
//! <out>/specs/index.json
//! <out>/specs/<slug>/<M.N>/bundle.json
//! <out>/specs/by-digest/sha256-<hex>.json
//! ```
//!
//! The builder is `trust_tasks_build::publish`, the one the official registry
//! is built with. A schema may `$ref` the registry's shared components by
//! absolute `https://trusttasks.org/…` URL; those are read from
//! `--official-specs` (a checkout of the registry's `specs/`) or fetched here,
//! once, by the publisher — the bundles carry them inlined, so a consumer never
//! dereferences anything.

use std::io::Read as _;
use std::path::Path;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use trust_tasks_build::publish::{build_third_party, write_registry, Resolver};

/// A bound on one fetched shared component; they are a few KB.
const MAX_COMPONENT_BYTES: u64 = 4 << 20;

pub fn registry_build(
    specs: &Path,
    type_uri_prefix: &str,
    out: &Path,
    official_specs: Option<&Path>,
) -> Result<()> {
    if !specs.is_dir() {
        return Err(anyhow!("--specs {} is not a directory", specs.display()));
    }
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(60)))
        .user_agent(concat!("trust-tasks-cli/", env!("CARGO_PKG_VERSION")))
        .build()
        .into();
    let fetch = |url: &str| -> Result<String> {
        let mut resp = agent
            .get(url)
            .call()
            .map_err(|e| anyhow!(e))
            .with_context(|| format!("fetching {url}"))?;
        let mut text = String::new();
        resp.body_mut()
            .as_reader()
            .take(MAX_COMPONENT_BYTES)
            .read_to_string(&mut text)
            .with_context(|| format!("reading {url}"))?;
        eprintln!("note: fetched {url}");
        Ok(text)
    };
    let refs = match official_specs {
        Some(dir) => Resolver::local().with_official_specs(dir),
        None => Resolver::local().with_fetch(&fetch),
    };
    let built = build_third_party(specs, type_uri_prefix, &refs)?;
    let specs_out = out.join("specs");
    let written = write_registry(&specs_out, &built.bundles, &built.index, true)?;
    for b in &built.bundles {
        println!(
            "{:>9} {} {} ({})",
            "Bundled",
            b.type_uri,
            b.status,
            crate::names::short(&b.digest)
        );
    }
    println!(
        "ok: {} bundle(s) and index.json in {} ({} new in by-digest{})",
        built.bundles.len(),
        specs_out.display(),
        written.by_digest_added,
        if written.stale_removed > 0 {
            format!(", {} stale bundle.json removed", written.stale_removed)
        } else {
            String::new()
        }
    );
    println!(
        "Publish {} as the site root (by-digest/ append-only, never deleted), and pin from it with [sources.<name>] url = \"https://…\"",
        out.display()
    );
    Ok(())
}
