//! The `[features]` table must match what is actually in `specs/`.
//!
//! `trust-tasks-codegen` writes the family features into `Cargo.toml` and the
//! matching `#[cfg(feature = …)] pub mod` lines into `src/specs/mod.rs`, so in
//! the normal course of events the three agree. This test exists for the case
//! the repo has been bitten by three times already (see `CLAUDE.md`): a list
//! that is *supposed* to track the tree quietly stops tracking it.
//!
//! Here the failure would be silent in a particularly unhelpful way — a family
//! whose feature went missing simply would not be in the crate, and
//! `codegen-drift` would not notice because that job only diffs
//! `trust-tasks-rs/src/specs`. So this re-derives the family set from `specs/`
//! itself and compares, rather than trusting either generated artifact.
//!
//! Skipped when `specs/` is not on disk, which is the case inside a published
//! `.crate` tarball.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// SPEC §6.1 reserves `^trust-(task|ceremony)($|-|/)`. Those families are
/// always compiled — the crate's own framework code depends on them — so they
/// must NOT carry a feature.
fn is_reserved(family: &str) -> bool {
    ["trust-task", "trust-ceremony"]
        .iter()
        .any(|p| match family.strip_prefix(p) {
            Some(rest) => rest.is_empty() || rest.starts_with('-'),
            None => false,
        })
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir has a parent")
        .to_path_buf()
}

/// Every top-level family under `specs/` that has at least one spec, minus the
/// hand-modelled `trust-task-error` and minus the reserved slugs.
fn families_on_disk(specs_dir: &Path) -> BTreeSet<String> {
    fn has_spec(dir: &Path) -> bool {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return false;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && has_spec(&path) {
                return true;
            }
            if path.file_name().is_some_and(|n| n == "payload.schema.json") {
                return true;
            }
        }
        false
    }

    let mut out = BTreeSet::new();
    for entry in std::fs::read_dir(specs_dir).expect("read specs/").flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        // `_shared` / `_framework` hold referenced fragments, not specs.
        if name.starts_with('_') || name == "trust-task-error" || is_reserved(&name) {
            continue;
        }
        if has_spec(&path) {
            out.insert(name);
        }
    }
    out
}

/// The generated region of `Cargo.toml`, as (`feature = []` names, `all-specs`
/// members, `default` members).
fn manifest_features(manifest: &str) -> (BTreeSet<String>, BTreeSet<String>, BTreeSet<String>) {
    let begin = manifest
        .find("# trust-tasks-codegen:begin")
        .expect("Cargo.toml has a codegen begin marker");
    let end = manifest
        .find("# trust-tasks-codegen:end")
        .expect("Cargo.toml has a codegen end marker");
    let block = &manifest[begin..end];

    let mut declared = BTreeSet::new();
    let mut all_specs = BTreeSet::new();
    let mut default = BTreeSet::new();
    let mut in_all_specs = false;

    for line in block.lines() {
        let line = line.trim();
        if in_all_specs {
            if line == "]" {
                in_all_specs = false;
            } else {
                all_specs.insert(line.trim_matches([' ', ',', '"']).to_string());
            }
            continue;
        }
        if line == "all-specs = [" {
            in_all_specs = true;
        } else if let Some(rest) = line.strip_prefix("default = [") {
            for name in rest.trim_end_matches(']').split(',') {
                let name = name.trim().trim_matches('"');
                if !name.is_empty() {
                    default.insert(name.to_string());
                }
            }
        } else if let Some((name, value)) = line.split_once(" = ") {
            if value == "[]" {
                declared.insert(name.trim_matches('"').to_string());
            }
        }
    }
    (declared, all_specs, default)
}

#[test]
fn every_spec_family_has_a_cargo_feature() {
    let root = repo_root();
    let specs_dir = root.join("specs");
    if !specs_dir.is_dir() {
        eprintln!(
            "skipping: {} not present (packaged crate)",
            specs_dir.display()
        );
        return;
    }

    let on_disk = families_on_disk(&specs_dir);
    assert!(
        !on_disk.is_empty(),
        "found no spec families under {}",
        specs_dir.display()
    );

    let manifest = std::fs::read_to_string(root.join("trust-tasks-rs/Cargo.toml"))
        .expect("read trust-tasks-rs/Cargo.toml");
    let (declared, all_specs, default) = manifest_features(&manifest);

    assert_eq!(
        declared,
        on_disk,
        "the [features] table in trust-tasks-rs/Cargo.toml does not match the families under \
         specs/. Run `cargo run -p trust-tasks-codegen` — the table is generated, not \
         hand-maintained.\n  on disk but not a feature: {:?}\n  a feature but not on disk: {:?}",
        on_disk.difference(&declared).collect::<Vec<_>>(),
        declared.difference(&on_disk).collect::<Vec<_>>(),
    );
    assert_eq!(
        all_specs,
        on_disk,
        "`all-specs` does not enable every family, so `default` no longer means \
         'everything' and a consumer who upgraded would silently lose modules.\n  \
         missing from all-specs: {:?}",
        on_disk.difference(&all_specs).collect::<Vec<_>>(),
    );
    assert_eq!(
        default,
        BTreeSet::from(["all-specs".to_string()]),
        "`default` must be exactly [\"all-specs\"]: that is what keeps this crate \
         non-breaking for a consumer who configures no features."
    );
}

#[test]
fn specs_mod_gates_exactly_the_featured_families() {
    let root = repo_root();
    let specs_dir = root.join("specs");
    if !specs_dir.is_dir() {
        eprintln!(
            "skipping: {} not present (packaged crate)",
            specs_dir.display()
        );
        return;
    }
    let on_disk = families_on_disk(&specs_dir);

    let mod_rs = std::fs::read_to_string(root.join("trust-tasks-rs/src/specs/mod.rs"))
        .expect("read trust-tasks-rs/src/specs/mod.rs");

    let mut gated = BTreeSet::new();
    let mut ungated = BTreeSet::new();
    let mut pending: Option<String> = None;
    for line in mod_rs.lines().map(str::trim) {
        if let Some(rest) = line.strip_prefix("#[cfg(feature = \"") {
            pending = rest.split('"').next().map(str::to_string);
            continue;
        }
        if let Some(rest) = line.strip_prefix("pub mod ") {
            let module = rest.trim_end_matches(';').to_string();
            match pending.take() {
                Some(feature) => {
                    assert_eq!(
                        feature.replace('-', "_"),
                        module,
                        "module `{module}` is gated on feature `{feature}`, which is not its own \
                         family name — a consumer enabling the family would not get the module"
                    );
                    gated.insert(feature);
                }
                None => {
                    ungated.insert(module);
                }
            }
        }
    }

    assert_eq!(
        gated, on_disk,
        "src/specs/mod.rs gates a different set of families than specs/ holds. Run \
         `cargo run -p trust-tasks-codegen`."
    );
    assert!(
        ungated.iter().all(|m| is_reserved(&m.replace('_', "-"))),
        "these modules are compiled unconditionally but are not framework-reserved slugs, so \
         they cost every consumer whatever they select: {:?}",
        ungated
            .iter()
            .filter(|m| !is_reserved(&m.replace('_', "-")))
            .collect::<Vec<_>>()
    );
}
