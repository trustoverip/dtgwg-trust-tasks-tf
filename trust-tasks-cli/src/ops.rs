//! The commands.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{anyhow, bail, Result};

use crate::lock::{Lock, LockEntry};
use crate::manifest::{Intent, Manifest, SourceLoc};
use crate::names::{parse_task_arg, short, Requested, TaskId, Version};
use crate::project::{effective_registry, Project, LOCKFILE, MANIFEST};
use crate::registry::{check_identity, read_bundle_head, Index, IndexEntry, Registry, Via};

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

/// The registries a command reads — the official one and each source a task
/// names — each opened, and its index read, at most once and only when a
/// task needs it. A command that touches only sourced tasks never contacts
/// the official registry, and the reverse.
struct Origins<'a> {
    root: &'a Path,
    official: String,
    sources: &'a BTreeMap<String, SourceLoc>,
    open: BTreeMap<Option<String>, (Registry, Option<Index>)>,
}

impl<'a> Origins<'a> {
    fn new(root: &'a Path, official: String, sources: &'a BTreeMap<String, SourceLoc>) -> Self {
        Origins {
            root,
            official,
            sources,
            open: BTreeMap::new(),
        }
    }

    fn entry(&mut self, source: Option<&str>) -> Result<&mut (Registry, Option<Index>)> {
        let key = source.map(str::to_string);
        if !self.open.contains_key(&key) {
            let registry = match source {
                None => Registry::open(&self.official)?,
                Some(name) => {
                    let loc = self.sources.get(name).ok_or_else(|| {
                        anyhow!(
                            "source `{name}` is not declared in {MANIFEST} — add [sources.{name}] with `url = \"https://…\"` or `path = \"<dir>\"`"
                        )
                    })?;
                    Registry::open_source(name, loc, self.root)?
                }
            };
            self.open.insert(key.clone(), (registry, None));
        }
        Ok(self.open.get_mut(&key).expect("just inserted"))
    }

    fn registry(&mut self, source: Option<&str>) -> Result<&Registry> {
        Ok(&self.entry(source)?.0)
    }

    /// The registry and its index.
    fn both(&mut self, source: Option<&str>) -> Result<(&Registry, &Index)> {
        let entry = self.entry(source)?;
        if entry.1.is_none() {
            entry.1 = Some(entry.0.index()?);
        }
        let entry = &*entry;
        Ok((&entry.0, entry.1.as_ref().expect("just loaded")))
    }
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
fn lifecycle_notes(id: &TaskId, v: Version, e: &IndexEntry) {
    if e.status == "retired" {
        warn(format_args!(
            "{id} {v} is retired{}",
            e.deprecation
                .as_deref()
                .map(|d| format!(" ({d})"))
                .unwrap_or_default()
        ));
    } else if let Some(d) = &e.deprecation {
        warn(format_args!("{id} {v} is deprecated: {d}"));
    }
}

fn fetch(
    registry: &Registry,
    id: &TaskId,
    v: Version,
    digest: &str,
    entry: Option<&IndexEntry>,
) -> Result<Vec<u8>> {
    let got = registry.fetch_bundle(id, v, digest, entry)?;
    if got.via == Via::Path {
        note(format_args!(
            "{id} {v}: not in {}'s by-digest store yet, so fetched specs/{}/{v}/bundle.json and verified it against {} (the by-digest store is new)",
            registry.label,
            id.slug,
            short(digest)
        ));
    }
    if let Some(e) = entry {
        let head = read_bundle_head(&got.bytes)?;
        if head.type_uri != e.type_uri && e.digest == digest {
            bail!(
                "{id} {v}: the bundle's typeUri {} disagrees with the index's {}",
                head.type_uri,
                e.type_uri
            );
        }
    }
    Ok(got.bytes)
}

/// The lock entry for `id` at `v` as `e` lists it, once its Type URI has
/// passed the identity rules for where it came from.
fn entry_for(id: &TaskId, v: Version, e: &IndexEntry) -> Result<LockEntry> {
    check_identity(id, v, &e.type_uri)?;
    Ok(LockEntry {
        source: id.source.clone(),
        slug: id.slug.clone(),
        version: v,
        type_uri: e.type_uri.clone(),
        digest: e.digest.clone(),
    })
}

fn no_such_version(
    index: &Index,
    id: &TaskId,
    v: Version,
    versions: &BTreeMap<Version, IndexEntry>,
) -> anyhow::Error {
    let have: Vec<String> = versions.keys().map(Version::to_string).collect();
    anyhow!(
        "{id} has no version {v} in {} (it has {})",
        index.label,
        have.join(", ")
    )
}

fn latest_live(index: &Index, id: &TaskId) -> Result<(Version, IndexEntry)> {
    index.versions(&id.slug)?;
    index
        .latest_live(&id.slug)
        .map(|(v, e)| (v, e.clone()))
        .ok_or_else(|| anyhow!("{id} has no live version (every version is retired or withdrawn); pin one explicitly as {id}/<M.N>"))
}

fn open_lock(project: &Project, registry_url: &str) -> Result<Lock> {
    Ok(Lock::load(&project.lock_path())?.unwrap_or_else(|| Lock::new(registry_url.to_string())))
}

/// Refuses a source named for the first segment of an official task the
/// manifest pins: the two would share a top-level namespace in every
/// language's generated code. Checked against the manifest so the error
/// arrives before anything is fetched; the lock is checked again on commit.
fn check_namespaces(manifest: &Manifest) -> Result<()> {
    let used: BTreeSet<&str> = manifest
        .tasks
        .keys()
        .filter_map(|id| id.source.as_deref())
        .collect();
    for id in manifest.tasks.keys().filter(|id| id.source.is_none()) {
        let family = id.slug.split('/').next().unwrap_or(&id.slug);
        if used.contains(family) {
            bail!(
                "source `{family}` has the name of the official family of {id} — their generated code would share the namespace `{family}`; rename the source in {MANIFEST} (its tasks are named `<source>:<slug>`)"
            );
        }
    }
    Ok(())
}

// ─── add ────────────────────────────────────────────────────────────────────

pub fn add(ctx: &Ctx, args: &[String]) -> Result<()> {
    let mut requests: BTreeMap<TaskId, Requested> = BTreeMap::new();
    for a in args {
        let (id, req) = parse_task_arg(a)?;
        if requests.insert(id.clone(), req).is_some() {
            bail!("{id} is named twice");
        }
    }
    let project = Project::locate(ctx.dir, true)?;
    let mut manifest = project.manifest(true)?;
    for id in requests.keys() {
        if let Some(source) = &id.source {
            manifest.source(source)?;
        }
    }
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
    {
        // The namespaces as they will be once these tasks are added.
        let mut after = Manifest::new(project.manifest_path());
        after.sources = manifest.sources.clone();
        for id in manifest.tasks.keys().chain(requests.keys()) {
            after.tasks.insert(id.clone(), Vec::new());
        }
        check_namespaces(&after)?;
    }
    let sources = manifest.sources.clone();
    let mut origins = Origins::new(&project.root, registry_url.clone(), &sources);

    let mut bundles = BTreeMap::new();
    for (id, req) in &requests {
        let (registry, index) = origins.both(id.source.as_deref())?;
        let versions = index.versions(&id.slug)?;
        let (v, entry, intent) = match req {
            Requested::Exact(v) => {
                let e = versions
                    .get(v)
                    .ok_or_else(|| no_such_version(index, id, *v, versions))?;
                (*v, e.clone(), Intent::Exact(*v))
            }
            Requested::Bare => {
                let (v, e) = latest_live(index, id)?;
                (v, e, Intent::Exact(v))
            }
            Requested::Latest => {
                let (v, e) = latest_live(index, id)?;
                (v, e, Intent::Latest)
            }
        };
        if entry.is_withdrawn() {
            let live = index
                .latest_live(&id.slug)
                .map(|(l, _)| format!("; the latest live version is {id}/{l}"))
                .unwrap_or_default();
            bail!("{id} {v} is withdrawn: it is no longer offered for new pins{live}");
        }
        lifecycle_notes(id, v, &entry);

        let new = entry_for(id, v, &entry)?;
        let previous = lock.remove_task(id);
        let unchanged = previous.len() == 1 && previous[0] == new && project.vendored_ok(&new);
        if unchanged {
            say(
                "Unchanged",
                format_args!("{id} {v} ({})", short(&new.digest)),
            );
        } else {
            bundles.insert(
                (id.clone(), v),
                fetch(registry, id, v, &new.digest, Some(&entry))?,
            );
            match previous.as_slice() {
                [] => say("Added", format_args!("{id} {v} ({})", short(&new.digest))),
                [p] if p.version == v => say(
                    "Updated",
                    format_args!("{id} {v} ({} → {})", short(&p.digest), short(&new.digest)),
                ),
                [p] => say(
                    "Moved",
                    format_args!(
                        "{id} {} → {v} ({}) — {}",
                        p.version,
                        short(&new.digest),
                        describe_move(index, &id.slug, p.version, v)
                    ),
                ),
                many => say(
                    "Replaced",
                    format_args!(
                        "{id} {} → {v} ({})",
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
        manifest.set(id, vec![intent]);
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
        let (id, req) = parse_task_arg(a)?;
        let intents = manifest
            .tasks
            .get(&id)
            .cloned()
            .ok_or_else(|| anyhow!("{id} is not in {MANIFEST}"))?;
        match req {
            Requested::Bare => {
                manifest.remove(&id);
                for e in lock.remove_task(&id) {
                    say("Removed", format_args!("{id} {}", e.version));
                }
            }
            Requested::Exact(v) => {
                if !intents.contains(&Intent::Exact(v)) {
                    bail!(
                        "{id} {v} is not pinned in {MANIFEST} ({id} = {})",
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
                    manifest.remove(&id);
                } else {
                    manifest.set(&id, rest);
                }
                lock.entries.remove(&(id.clone(), v));
                say("Removed", format_args!("{id} {v}"));
            }
            Requested::Latest => bail!("`remove` takes a task or task/<M.N>, not @latest"),
        }
    }
    project.commit(&manifest, old.as_ref(), &lock, &BTreeMap::new())
}

// ─── update ─────────────────────────────────────────────────────────────────

pub fn update(ctx: &Ctx, slugs: &[String], to_latest: bool) -> Result<bool> {
    let project = Project::locate(ctx.dir, false)?;
    let mut manifest = project.manifest(false)?;
    check_namespaces(&manifest)?;
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

    let targets: BTreeSet<TaskId> = if slugs.is_empty() {
        manifest.tasks.keys().cloned().collect()
    } else {
        let mut t = BTreeSet::new();
        for s in slugs {
            let (id, req) = parse_task_arg(s)?;
            if req != Requested::Bare {
                bail!("`update` takes tasks; to pin a version, `trust-tasks add {s}`");
            }
            if !manifest.tasks.contains_key(&id) {
                bail!("{id} is not in {MANIFEST} — `trust-tasks add {id}` adds it");
            }
            t.insert(id);
        }
        t
    };

    let sources = manifest.sources.clone();
    let mut origins = Origins::new(&project.root, registry_url.clone(), &sources);
    let mut bundles = BTreeMap::new();
    let mut refused = 0usize;

    // A full update also drops lock entries the manifest no longer names.
    if slugs.is_empty() {
        let stale: BTreeSet<TaskId> = lock
            .entries
            .keys()
            .map(|(id, _)| id.clone())
            .filter(|id| !manifest.tasks.contains_key(id))
            .collect();
        for id in stale {
            for e in lock.remove_task(&id) {
                say(
                    "Removed",
                    format_args!("{id} {} (no longer in {MANIFEST})", e.version),
                );
            }
        }
    }

    for id in &targets {
        let intents = manifest.tasks[id].clone();
        let (registry, index) = origins.both(id.source.as_deref())?;
        let slug = &id.slug;
        let versions = index.versions(slug)?;
        let previous: Vec<LockEntry> = lock.for_task(id).cloned().collect();
        let mut keep: Vec<LockEntry> = Vec::new();
        let mut new_intents = Vec::new();

        for intent in &intents {
            // The version this intent should now be pinned at.
            let (target, moved_intent) = match intent {
                Intent::Latest => (latest_live(index, id)?.0, Intent::Latest),
                Intent::Exact(v) if to_latest => {
                    let (l, _) = latest_live(index, id)?;
                    let l = l.max(*v);
                    (l, Intent::Exact(l))
                }
                Intent::Exact(v) => (*v, Intent::Exact(*v)),
            };
            new_intents.push(moved_intent);
            // What it was pinned at: the same version, or for `latest` the one
            // lock entry the task had.
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
                    eprintln!("error: {}", no_such_version(index, id, target, versions));
                    refused += 1;
                    keep.extend(current);
                    continue;
                }
            };
            if entry.is_withdrawn() {
                let live = index
                    .latest_live(slug)
                    .map(|(l, _)| format!(" — move the pin with `trust-tasks add {id}/{l}`"))
                    .unwrap_or_default();
                eprintln!("error: {id} {target} is withdrawn; update refuses it{live}");
                refused += 1;
                keep.extend(current);
                continue;
            }
            lifecycle_notes(id, target, entry);
            let new = entry_for(id, target, entry)?;
            match current {
                None => {
                    bundles.insert(
                        (id.clone(), target),
                        fetch(registry, id, target, &new.digest, Some(entry))?,
                    );
                    say(
                        "Added",
                        format_args!("{id} {target} ({})", short(&new.digest)),
                    );
                    keep.push(new);
                }
                Some(cur) if cur.version == target && cur.digest == new.digest => {
                    if !project.vendored_ok(&cur) {
                        bundles.insert(
                            (id.clone(), target),
                            fetch(registry, id, target, &cur.digest, Some(entry))?,
                        );
                        say(
                            "Restored",
                            format_args!("{id} {target} ({})", short(&cur.digest)),
                        );
                    } else {
                        say(
                            "Unchanged",
                            format_args!("{id} {target} ({})", short(&cur.digest)),
                        );
                    }
                    keep.push(cur);
                }
                Some(cur) if cur.version == target => {
                    bundles.insert(
                        (id.clone(), target),
                        fetch(registry, id, target, &new.digest, Some(entry))?,
                    );
                    say(
                        "Updated",
                        format_args!(
                            "{id} {target} ({} → {}) — {} changed it in place ({})",
                            short(&cur.digest),
                            short(&new.digest),
                            registry.label,
                            entry.status
                        ),
                    );
                    keep.push(new);
                }
                Some(cur) => {
                    bundles.insert(
                        (id.clone(), target),
                        fetch(registry, id, target, &new.digest, Some(entry))?,
                    );
                    say(
                        "Moved",
                        format_args!(
                            "{id} {} → {target} ({}) — {}",
                            cur.version,
                            short(&new.digest),
                            describe_move(index, slug, cur.version, target)
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
                                "{id} {l} is available ({how}); `trust-tasks add {id}/{l}` or `update --to-latest` moves the pin"
                            ));
                        }
                    }
                }
            }
        }
        lock.remove_task(id);
        for e in keep {
            lock.insert(e);
        }
        if new_intents != intents {
            manifest.set(id, new_intents);
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

    let official_used = lock.entries.keys().any(|(id, _)| id.source.is_none());
    if official_used && lock.registry != registry_url {
        problems.push(format!(
            "{LOCKFILE} was resolved against {} but the registry is {registry_url} — run `trust-tasks update`",
            lock.registry
        ));
    }
    if let Err(e) = check_namespaces(&manifest) {
        problems.push(format!("{e:#}"));
    }
    // Each source the lock resolved from is where the manifest says it is.
    for (name, loc) in &lock.sources {
        match manifest.sources.get(name) {
            Some(declared) if declared == loc => {}
            Some(declared) => problems.push(format!(
                "{LOCKFILE} resolved source `{name}` from {loc}, but {MANIFEST} now declares {declared} — run `trust-tasks update`"
            )),
            None => problems.push(format!(
                "{LOCKFILE} resolved source `{name}` from {loc}, but {MANIFEST} no longer declares it — run `trust-tasks update`"
            )),
        }
    }

    // Manifest ↔ lock.
    for (id, intents) in &manifest.tasks {
        let locked: Vec<Version> = lock.for_task(id).map(|e| e.version).collect();
        for i in intents {
            match i {
                Intent::Exact(v) if !locked.contains(v) => {
                    problems.push(format!("{MANIFEST} pins {id} {v} but {LOCKFILE} does not — run `trust-tasks update`"))
                }
                Intent::Latest if locked.len() != 1 => problems.push(format!(
                    "{MANIFEST} has {id} = \"latest\" but {LOCKFILE} locks {} versions of it — run `trust-tasks update`",
                    locked.len()
                )),
                _ => {}
            }
        }
        if !intents.contains(&Intent::Latest) {
            for v in &locked {
                if !intents.contains(&Intent::Exact(*v)) {
                    problems.push(format!(
                        "{LOCKFILE} locks {id} {v} but {MANIFEST} does not pin it — run `trust-tasks update`"
                    ));
                }
            }
        }
    }
    for (id, v) in lock.entries.keys() {
        if !manifest.tasks.contains_key(id) {
            problems.push(format!(
                "{LOCKFILE} locks {id} {v} but {MANIFEST} does not name {id} — run `trust-tasks update`"
            ));
        }
    }

    // Every entry's bundle, by digest.
    let mut expected = BTreeSet::new();
    for e in lock.entries.values() {
        let id = e.id();
        let rel = Project::vendored_rel(&id, e.version);
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
                        "{rel} is the bundle for {} ({}), not the locked {id} {}",
                        head.type_uri, head.version, e.version
                    ));
                } else if let Err(err) = check_identity(&id, e.version, &head.type_uri) {
                    problems.push(format!("{rel}: {err:#}"));
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
    // Restore from where the lock resolved each source, which is what its
    // digests were taken from.
    let mut origins = Origins::new(&project.root, registry_url, &lock.sources);
    let mut bundles = BTreeMap::new();
    for e in lock.entries.values() {
        if project.vendored_ok(e) {
            continue;
        }
        let id = e.id();
        let by_digest = origins
            .registry(id.source.as_deref())?
            .fetch_bundle(&id, e.version, &e.digest, None);
        let bytes = match by_digest {
            Ok(got) => got.bytes,
            Err(_) => {
                let (registry, index) = origins.both(id.source.as_deref())?;
                let entry = index.get(&id.slug, e.version);
                fetch(registry, &id, e.version, &e.digest, entry)?
            }
        };
        say(
            "Fetched",
            format_args!("{id} {} ({})", e.version, short(&e.digest)),
        );
        bundles.insert((id, e.version), bytes);
    }
    if bundles.is_empty() {
        println!("ok: all {} vendored bundle(s) present", lock.entries.len());
        return Ok(());
    }
    // Keep the lock's sources: `fetch` never re-resolves.
    let mut keep = manifest;
    keep.sources = lock.sources.clone();
    project.commit(&keep, Some(&lock), &lock, &bundles)
}

// ─── list ───────────────────────────────────────────────────────────────────

pub fn list(ctx: &Ctx, remote: bool) -> Result<()> {
    let project = Project::locate(ctx.dir, false)?;
    let manifest = project.manifest(false)?;
    let lock = Lock::load(&project.lock_path())?
        .ok_or_else(|| anyhow!("no {LOCKFILE} — run `trust-tasks update`"))?;
    let registry_url = effective_registry(&manifest, ctx.registry)?;
    let mut origins = Origins::new(&project.root, registry_url, &manifest.sources);

    let mut header = vec!["TASK", "VERSION", "INTENT", "STATUS", "NOTE"];
    if remote {
        header.insert(4, "LATEST");
    }
    let mut rows: Vec<Vec<String>> = Vec::new();
    for e in lock.entries.values() {
        let id = e.id();
        let intent = match manifest.tasks.get(&id) {
            Some(i) if i.contains(&Intent::Latest) => "latest".to_string(),
            Some(_) => "exact".to_string(),
            None => "(not in manifest)".to_string(),
        };
        let (status, mut note) = match std::fs::read(project.vendored(&id, e.version))
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
        let mut row = vec![id.to_string(), e.version.to_string(), intent, status];
        if remote {
            let (_, index) = origins.both(id.source.as_deref())?;
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
                None => note.push(format!("not in {}'s index", index.label)),
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
