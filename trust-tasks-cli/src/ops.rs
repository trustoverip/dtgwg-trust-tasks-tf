//! The commands.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{anyhow, bail, Result};

use crate::lock::{Lock, LockEntry};
use crate::manifest::Intent;
use crate::names::{parse_task_arg, short, Requested, Version};
use crate::project::{effective_registry, Project, LOCKFILE, MANIFEST};
use crate::registry::{read_bundle_head, Index, IndexEntry, Registry, Via};

/// Global options.
pub struct Ctx<'a> {
    pub dir: Option<&'a Path>,
    pub registry: Option<&'a str>,
}

/// One line of a command's report: `{verb:>9} {rest}`.
fn say(verb: &str, rest: impl std::fmt::Display) {
    println!("{verb:>9} {rest}");
}

fn note(msg: impl std::fmt::Display) {
    eprintln!("note: {msg}");
}

fn warn(msg: impl std::fmt::Display) {
    eprintln!("warning: {msg}");
}

/// What moving a pin from `from` to `to` means for the code using it.
fn describe_move(index: &Index, slug: &str, from: Version, to: Version) -> String {
    if to > from && index.extends_reaches(slug, to, from) {
        format!("{to} extends {from}: no code change needed")
    } else {
        format!("breaking: {to} does not extend {from}; code written against {from} must change")
    }
}

/// Lifecycle notes for a version being pinned.
fn lifecycle_notes(slug: &str, v: Version, e: &IndexEntry) {
    if e.status == "retired" {
        warn(format_args!(
            "{slug} {v} is retired{}",
            e.deprecation
                .as_deref()
                .map(|d| format!(" ({d})"))
                .unwrap_or_default()
        ));
    } else if let Some(d) = &e.deprecation {
        warn(format_args!("{slug} {v} is deprecated: {d}"));
    }
}

fn fetch(
    registry: &Registry,
    slug: &str,
    v: Version,
    digest: &str,
    entry: Option<&IndexEntry>,
) -> Result<Vec<u8>> {
    let got = registry.fetch_bundle(slug, v, digest, entry)?;
    if got.via == Via::Path {
        note(format_args!(
            "{slug} {v}: not in the registry's by-digest store yet, so fetched specs/{}/{v}/bundle.json and verified it against {} (the by-digest store is new)",
            slug,
            short(digest)
        ));
    }
    if let Some(e) = entry {
        let head = read_bundle_head(&got.bytes)?;
        if head.type_uri != e.type_uri && e.digest == digest {
            bail!(
                "{slug} {v}: the bundle's typeUri {} disagrees with the index's {}",
                head.type_uri,
                e.type_uri
            );
        }
    }
    Ok(got.bytes)
}

fn entry_for(slug: &str, v: Version, e: &IndexEntry) -> LockEntry {
    LockEntry {
        slug: slug.to_string(),
        version: v,
        type_uri: e.type_uri.clone(),
        digest: e.digest.clone(),
    }
}

fn no_such_version(
    slug: &str,
    v: Version,
    versions: &BTreeMap<Version, IndexEntry>,
) -> anyhow::Error {
    let have: Vec<String> = versions.keys().map(Version::to_string).collect();
    anyhow!(
        "{slug} has no version {v} in the registry (it has {})",
        have.join(", ")
    )
}

fn latest_live(index: &Index, slug: &str) -> Result<(Version, IndexEntry)> {
    index.versions(slug)?;
    index
        .latest_live(slug)
        .map(|(v, e)| (v, e.clone()))
        .ok_or_else(|| anyhow!("{slug} has no live version (every version is retired or withdrawn); pin one explicitly as {slug}/<M.N>"))
}

fn open_lock(project: &Project, registry_url: &str) -> Result<Lock> {
    Ok(Lock::load(&project.lock_path())?.unwrap_or_else(|| Lock::new(registry_url.to_string())))
}

// ─── add ────────────────────────────────────────────────────────────────────

pub fn add(ctx: &Ctx, args: &[String]) -> Result<()> {
    let mut requests: BTreeMap<String, Requested> = BTreeMap::new();
    for a in args {
        let (slug, req) = parse_task_arg(a)?;
        if requests.insert(slug.clone(), req).is_some() {
            bail!("{slug} is named twice");
        }
    }
    let project = Project::locate(ctx.dir, true)?;
    let mut manifest = project.manifest(true)?;
    let registry_url = effective_registry(&manifest, ctx.registry)?;
    let old = Lock::load(&project.lock_path())?;
    let mut lock = old
        .clone()
        .unwrap_or_else(|| Lock::new(registry_url.clone()));
    if lock.registry != registry_url {
        note(format_args!(
            "{LOCKFILE} now records registry {registry_url} (was {}); existing pins keep their digests",
            lock.registry
        ));
        lock.registry = registry_url.clone();
    }
    let registry = Registry::open(&registry_url)?;
    let index = registry.index()?;

    let mut bundles = BTreeMap::new();
    for (slug, req) in &requests {
        let versions = index.versions(slug)?;
        let (v, entry, intent) = match req {
            Requested::Exact(v) => {
                let e = versions
                    .get(v)
                    .ok_or_else(|| no_such_version(slug, *v, versions))?;
                (*v, e.clone(), Intent::Exact(*v))
            }
            Requested::Bare => {
                let (v, e) = latest_live(&index, slug)?;
                (v, e, Intent::Exact(v))
            }
            Requested::Latest => {
                let (v, e) = latest_live(&index, slug)?;
                (v, e, Intent::Latest)
            }
        };
        if entry.is_withdrawn() {
            let live = index
                .latest_live(slug)
                .map(|(l, _)| format!("; the latest live version is {slug}/{l}"))
                .unwrap_or_default();
            bail!("{slug} {v} is withdrawn: it is no longer offered for new pins{live}");
        }
        lifecycle_notes(slug, v, &entry);

        let previous = lock.remove_slug(slug);
        let new = entry_for(slug, v, &entry);
        let unchanged = previous.len() == 1 && previous[0] == new && project.vendored_ok(&new);
        if unchanged {
            say(
                "Unchanged",
                format_args!("{slug} {v} ({})", short(&new.digest)),
            );
        } else {
            bundles.insert(
                (slug.clone(), v),
                fetch(&registry, slug, v, &new.digest, Some(&entry))?,
            );
            match previous.as_slice() {
                [] => say("Added", format_args!("{slug} {v} ({})", short(&new.digest))),
                [p] if p.version == v => say(
                    "Updated",
                    format_args!("{slug} {v} ({} → {})", short(&p.digest), short(&new.digest)),
                ),
                [p] => say(
                    "Moved",
                    format_args!(
                        "{slug} {} → {v} ({}) — {}",
                        p.version,
                        short(&new.digest),
                        describe_move(&index, slug, p.version, v)
                    ),
                ),
                many => say(
                    "Replaced",
                    format_args!(
                        "{slug} {} → {v} ({})",
                        many.iter()
                            .map(|p| p.version.to_string())
                            .collect::<Vec<_>>()
                            .join(", "),
                        short(&new.digest)
                    ),
                ),
            }
        }
        lock.insert(new);
        manifest.set(slug, vec![intent]);
    }
    project.commit(&manifest, old.as_ref(), &lock, &bundles)
}

// ─── remove ─────────────────────────────────────────────────────────────────

pub fn remove(ctx: &Ctx, args: &[String]) -> Result<()> {
    let project = Project::locate(ctx.dir, false)?;
    let mut manifest = project.manifest(false)?;
    let registry_url = effective_registry(&manifest, ctx.registry)?;
    let old = Lock::load(&project.lock_path())?;
    let mut lock = old.clone().unwrap_or_else(|| Lock::new(registry_url));
    for a in args {
        let (slug, req) = parse_task_arg(a)?;
        let intents = manifest
            .tasks
            .get(&slug)
            .cloned()
            .ok_or_else(|| anyhow!("{slug} is not in {MANIFEST}"))?;
        match req {
            Requested::Bare => {
                manifest.remove(&slug);
                for e in lock.remove_slug(&slug) {
                    say("Removed", format_args!("{slug} {}", e.version));
                }
            }
            Requested::Exact(v) => {
                if !intents.contains(&Intent::Exact(v)) {
                    bail!(
                        "{slug} {v} is not pinned in {MANIFEST} ({slug} = {})",
                        intents
                            .iter()
                            .map(|i| format!("\"{i}\""))
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                }
                let rest: Vec<Intent> = intents
                    .into_iter()
                    .filter(|i| *i != Intent::Exact(v))
                    .collect();
                if rest.is_empty() {
                    manifest.remove(&slug);
                } else {
                    manifest.set(&slug, rest);
                }
                lock.entries.remove(&(slug.clone(), v));
                say("Removed", format_args!("{slug} {v}"));
            }
            Requested::Latest => bail!("`remove` takes a slug or slug/<M.N>, not @latest"),
        }
    }
    project.commit(&manifest, old.as_ref(), &lock, &BTreeMap::new())
}

// ─── update ─────────────────────────────────────────────────────────────────

pub fn update(ctx: &Ctx, slugs: &[String], to_latest: bool) -> Result<bool> {
    let project = Project::locate(ctx.dir, false)?;
    let mut manifest = project.manifest(false)?;
    let registry_url = effective_registry(&manifest, ctx.registry)?;
    let old = Lock::load(&project.lock_path())?;
    let mut lock = open_lock(&project, &registry_url)?;
    if lock.registry != registry_url {
        note(format_args!(
            "{LOCKFILE} now records registry {registry_url} (was {})",
            lock.registry
        ));
        lock.registry = registry_url.clone();
    }

    let targets: BTreeSet<String> = if slugs.is_empty() {
        manifest.tasks.keys().cloned().collect()
    } else {
        let mut t = BTreeSet::new();
        for s in slugs {
            let (slug, req) = parse_task_arg(s)?;
            if req != Requested::Bare {
                bail!("`update` takes slugs; to pin a version, `trust-tasks add {s}`");
            }
            if !manifest.tasks.contains_key(&slug) {
                bail!("{slug} is not in {MANIFEST} — `trust-tasks add {slug}` adds it");
            }
            t.insert(slug);
        }
        t
    };

    let registry = Registry::open(&registry_url)?;
    let index = registry.index()?;
    let mut bundles = BTreeMap::new();
    let mut refused = 0usize;

    // A full update also drops lock entries the manifest no longer names.
    if slugs.is_empty() {
        let stale: BTreeSet<String> = lock
            .entries
            .keys()
            .map(|(s, _)| s.clone())
            .filter(|s| !manifest.tasks.contains_key(s))
            .collect();
        for s in stale {
            for e in lock.remove_slug(&s) {
                say(
                    "Removed",
                    format_args!("{s} {} (no longer in {MANIFEST})", e.version),
                );
            }
        }
    }

    for slug in &targets {
        let intents = manifest.tasks[slug].clone();
        let versions = index.versions(slug)?;
        let previous: Vec<LockEntry> = lock.for_slug(slug).cloned().collect();
        let mut keep: Vec<LockEntry> = Vec::new();
        let mut new_intents = Vec::new();

        for intent in &intents {
            // The version this intent should now be pinned at.
            let (target, moved_intent) = match intent {
                Intent::Latest => (latest_live(&index, slug)?.0, Intent::Latest),
                Intent::Exact(v) if to_latest => {
                    let (l, _) = latest_live(&index, slug)?;
                    let l = l.max(*v);
                    (l, Intent::Exact(l))
                }
                Intent::Exact(v) => (*v, Intent::Exact(*v)),
            };
            new_intents.push(moved_intent);
            // What it was pinned at: the same version, or for `latest` the one
            // lock entry the slug had.
            let current = previous
                .iter()
                .find(|p| p.version == target)
                .cloned()
                .or_else(|| match intent {
                    Intent::Latest => previous.first().cloned(),
                    Intent::Exact(v) if to_latest => {
                        previous.iter().find(|p| p.version == *v).cloned()
                    }
                    Intent::Exact(_) => None,
                });

            let entry = match versions.get(&target) {
                Some(e) => e,
                None => {
                    eprintln!("error: {}", no_such_version(slug, target, versions));
                    refused += 1;
                    keep.extend(current);
                    continue;
                }
            };
            if entry.is_withdrawn() {
                let live = index
                    .latest_live(slug)
                    .map(|(l, _)| format!(" — move the pin with `trust-tasks add {slug}/{l}`"))
                    .unwrap_or_default();
                eprintln!("error: {slug} {target} is withdrawn; update refuses it{live}");
                refused += 1;
                keep.extend(current);
                continue;
            }
            lifecycle_notes(slug, target, entry);
            let new = entry_for(slug, target, entry);
            match current {
                None => {
                    bundles.insert(
                        (slug.clone(), target),
                        fetch(&registry, slug, target, &new.digest, Some(entry))?,
                    );
                    say(
                        "Added",
                        format_args!("{slug} {target} ({})", short(&new.digest)),
                    );
                    keep.push(new);
                }
                Some(cur) if cur.version == target && cur.digest == new.digest => {
                    if !project.vendored_ok(&cur) {
                        bundles.insert(
                            (slug.clone(), target),
                            fetch(&registry, slug, target, &cur.digest, Some(entry))?,
                        );
                        say(
                            "Restored",
                            format_args!("{slug} {target} ({})", short(&cur.digest)),
                        );
                    } else {
                        say(
                            "Unchanged",
                            format_args!("{slug} {target} ({})", short(&cur.digest)),
                        );
                    }
                    keep.push(cur);
                }
                Some(cur) if cur.version == target => {
                    bundles.insert(
                        (slug.clone(), target),
                        fetch(&registry, slug, target, &new.digest, Some(entry))?,
                    );
                    say(
                        "Updated",
                        format_args!(
                            "{slug} {target} ({} → {}) — the registry changed it in place ({})",
                            short(&cur.digest),
                            short(&new.digest),
                            entry.status
                        ),
                    );
                    keep.push(new);
                }
                Some(cur) => {
                    bundles.insert(
                        (slug.clone(), target),
                        fetch(&registry, slug, target, &new.digest, Some(entry))?,
                    );
                    say(
                        "Moved",
                        format_args!(
                            "{slug} {} → {target} ({}) — {}",
                            cur.version,
                            short(&new.digest),
                            describe_move(&index, slug, cur.version, target)
                        ),
                    );
                    keep.push(new);
                }
            }
        }
        // Say when an exact pin has a newer live version, without moving it.
        if !to_latest {
            if let Some((l, _)) = index.latest_live(slug) {
                for i in &intents {
                    if let Intent::Exact(v) = i {
                        if l > *v && !intents.contains(&Intent::Exact(l)) {
                            let how = if index.extends_reaches(slug, l, *v) {
                                format!("{l} extends {v}: no code change needed")
                            } else {
                                format!("breaking: {l} does not extend {v}")
                            };
                            note(format_args!(
                                "{slug} {l} is available ({how}); `trust-tasks add {slug}/{l}` or `update --to-latest` moves the pin"
                            ));
                        }
                    }
                }
            }
        }
        lock.remove_slug(slug);
        for e in keep {
            lock.insert(e);
        }
        if new_intents != intents {
            manifest.set(slug, new_intents);
        }
    }
    project.commit(&manifest, old.as_ref(), &lock, &bundles)?;
    if refused > 0 {
        eprintln!("error: {refused} pin(s) refused; the rest were updated");
    }
    Ok(refused == 0)
}

// ─── check ──────────────────────────────────────────────────────────────────

pub fn check(ctx: &Ctx) -> Result<bool> {
    let project = Project::locate(ctx.dir, false)?;
    let manifest = project.manifest(false)?;
    let registry_url = effective_registry(&manifest, ctx.registry)?;
    let mut problems: Vec<String> = Vec::new();
    let Some(lock) = Lock::load(&project.lock_path())? else {
        if manifest.tasks.is_empty() {
            println!("ok: {MANIFEST} names no tasks");
            return Ok(true);
        }
        println!("problem: no {LOCKFILE} — run `trust-tasks update`");
        return Ok(false);
    };

    if lock.registry != registry_url {
        problems.push(format!(
            "{LOCKFILE} was resolved against {} but the registry is {registry_url} — run `trust-tasks update`",
            lock.registry
        ));
    }

    // Manifest ↔ lock.
    for (slug, intents) in &manifest.tasks {
        let locked: Vec<Version> = lock.for_slug(slug).map(|e| e.version).collect();
        for i in intents {
            match i {
                Intent::Exact(v) if !locked.contains(v) => {
                    problems.push(format!("{MANIFEST} pins {slug} {v} but {LOCKFILE} does not — run `trust-tasks update`"))
                }
                Intent::Latest if locked.len() != 1 => problems.push(format!(
                    "{MANIFEST} has {slug} = \"latest\" but {LOCKFILE} locks {} versions of it — run `trust-tasks update`",
                    locked.len()
                )),
                _ => {}
            }
        }
        if !intents.contains(&Intent::Latest) {
            for v in &locked {
                if !intents.contains(&Intent::Exact(*v)) {
                    problems.push(format!(
                        "{LOCKFILE} locks {slug} {v} but {MANIFEST} does not pin it — run `trust-tasks update`"
                    ));
                }
            }
        }
    }
    for (slug, v) in lock.entries.keys() {
        if !manifest.tasks.contains_key(slug) {
            problems.push(format!(
                "{LOCKFILE} locks {slug} {v} but {MANIFEST} does not name {slug} — run `trust-tasks update`"
            ));
        }
    }

    // Every entry's bundle, by digest.
    let mut expected = BTreeSet::new();
    for e in lock.entries.values() {
        let rel = Project::vendored_rel(&e.slug, e.version);
        expected.insert(rel.clone());
        let bytes = match std::fs::read(project.root.join(&rel)) {
            Ok(b) => b,
            Err(_) => {
                problems.push(format!(
                    "{rel} is missing — run `trust-tasks fetch` to restore it by digest"
                ));
                continue;
            }
        };
        let got = crate::names::digest_of(&bytes);
        if got != e.digest {
            problems.push(format!(
                "{rel} does not match {LOCKFILE}: it is {got}, the lock says {} — restore it with `trust-tasks fetch` (or `git checkout`)",
                e.digest
            ));
            continue;
        }
        match read_bundle_head(&bytes) {
            Ok(head) => {
                if head.slug != e.slug
                    || Version::parse(&head.version) != Some(e.version)
                    || head.type_uri != e.type_uri
                {
                    problems.push(format!(
                        "{rel} is the bundle for {} ({}), not the locked {} {}",
                        head.type_uri, head.version, e.slug, e.version
                    ));
                }
            }
            Err(err) => problems.push(format!("{rel}: {err:#}")),
        }
    }
    for f in project.vendored_files()? {
        if !expected.contains(&f) {
            problems.push(format!(
                "{f} is not in {LOCKFILE} — delete it, or `trust-tasks add` its task"
            ));
        }
    }

    if problems.is_empty() {
        println!(
            "ok: {} pinned task version(s) match {LOCKFILE}",
            lock.entries.len()
        );
        Ok(true)
    } else {
        for p in &problems {
            println!("problem: {p}");
        }
        println!("{} problem(s)", problems.len());
        Ok(false)
    }
}

// ─── fetch ──────────────────────────────────────────────────────────────────

/// Restores every vendored bundle that is missing or altered, by the digest
/// the lock pins — never moving a pin. The by-digest store is what makes this
/// work after a draft has changed in place; the index is read only when the
/// store does not have a digest.
pub fn fetch_locked(ctx: &Ctx) -> Result<()> {
    let project = Project::locate(ctx.dir, false)?;
    let manifest = project.manifest(false)?;
    let registry_url = effective_registry(&manifest, ctx.registry)?;
    let lock = Lock::load(&project.lock_path())?
        .ok_or_else(|| anyhow!("no {LOCKFILE} — run `trust-tasks update` to resolve {MANIFEST}"))?;
    let registry = Registry::open(&registry_url)?;
    let mut index: Option<Index> = None;
    let mut bundles = BTreeMap::new();
    for e in lock.entries.values() {
        if project.vendored_ok(e) {
            continue;
        }
        let bytes = match registry.fetch_bundle(&e.slug, e.version, &e.digest, None) {
            Ok(got) => got.bytes,
            Err(_) => {
                if index.is_none() {
                    index = Some(registry.index()?);
                }
                let entry = index.as_ref().and_then(|i| i.get(&e.slug, e.version));
                fetch(&registry, &e.slug, e.version, &e.digest, entry)?
            }
        };
        say(
            "Fetched",
            format_args!("{} {} ({})", e.slug, e.version, short(&e.digest)),
        );
        bundles.insert((e.slug.clone(), e.version), bytes);
    }
    if bundles.is_empty() {
        println!("ok: all {} vendored bundle(s) present", lock.entries.len());
        return Ok(());
    }
    project.commit(&manifest, Some(&lock), &lock, &bundles)
}

// ─── list ───────────────────────────────────────────────────────────────────

pub fn list(ctx: &Ctx, remote: bool) -> Result<()> {
    let project = Project::locate(ctx.dir, false)?;
    let manifest = project.manifest(false)?;
    let lock = Lock::load(&project.lock_path())?
        .ok_or_else(|| anyhow!("no {LOCKFILE} — run `trust-tasks update`"))?;
    let index = if remote {
        let url = effective_registry(&manifest, ctx.registry)?;
        Some(Registry::open(&url)?.index()?)
    } else {
        None
    };

    let mut header = vec!["TASK", "VERSION", "INTENT", "STATUS", "NOTE"];
    if remote {
        header.insert(4, "LATEST");
    }
    let mut rows: Vec<Vec<String>> = Vec::new();
    for e in lock.entries.values() {
        let intent = match manifest.tasks.get(&e.slug) {
            Some(i) if i.contains(&Intent::Latest) => "latest".to_string(),
            Some(_) => "exact".to_string(),
            None => "(not in manifest)".to_string(),
        };
        let (status, mut note) = match std::fs::read(project.vendored(&e.slug, e.version))
            .ok()
            .filter(|b| crate::names::digest_of(b) == e.digest)
            .and_then(|b| read_bundle_head(&b).ok())
        {
            Some(h) => {
                let mut n = Vec::new();
                if let Some(d) = h.deprecation {
                    n.push(d);
                }
                if let Some(x) = h.extends {
                    n.push(format!("extends {x}"));
                }
                (h.status, n)
            }
            None => (
                "?".to_string(),
                vec!["vendored bundle missing or altered — run `trust-tasks check`".to_string()],
            ),
        };
        let mut row = vec![e.slug.clone(), e.version.to_string(), intent, status];
        if let Some(index) = &index {
            let latest = match index.latest_live(&e.slug) {
                Some((l, _)) if l > e.version => {
                    let how = if index.extends_reaches(&e.slug, l, e.version) {
                        "extends, no code change"
                    } else {
                        "breaking"
                    };
                    format!("{l} ({how})")
                }
                Some((l, _)) => l.to_string(),
                None => "-".to_string(),
            };
            match index.get(&e.slug, e.version) {
                Some(ie) if ie.is_withdrawn() => note.push("withdrawn upstream".to_string()),
                Some(ie) if ie.digest != e.digest => {
                    note.push("changed upstream — `trust-tasks update`".to_string())
                }
                None => note.push("not in the registry index".to_string()),
                _ => {}
            }
            row.push(latest);
        }
        row.push(note.join("; "));
        rows.push(row);
    }
    if rows.is_empty() {
        println!("no tasks pinned");
        return Ok(());
    }
    let header: Vec<String> = header.into_iter().map(String::from).collect();
    let widths: Vec<usize> = (0..header.len())
        .map(|i| {
            std::iter::once(&header)
                .chain(rows.iter())
                .map(|r| r[i].chars().count())
                .max()
                .unwrap_or(0)
        })
        .collect();
    for r in std::iter::once(&header).chain(rows.iter()) {
        let line: Vec<String> = r
            .iter()
            .zip(&widths)
            .map(|(c, w)| format!("{c:<w$}"))
            .collect();
        println!("{}", line.join("  ").trim_end());
    }
    Ok(())
}
