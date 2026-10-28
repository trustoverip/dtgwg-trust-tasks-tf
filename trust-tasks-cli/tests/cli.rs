//! The `trust-tasks` binary end to end, against a directory registry.
//!
//! Each test builds a registry in a temporary directory, laid out as
//! trusttasks.org is (`specs/index.json`, `specs/<slug>/<M.N>/bundle.json`,
//! `specs/by-digest/sha256-<hex>.json`), from two kinds of bundle:
//!
//! - **real** ones, copied verbatim from `npm run build-bundles` into
//!   `tests/fixtures/bundles/` (`vault/upsert` 0.3 and 0.4, `acl/grant` 0.1
//!   and 0.2) — the CLI hashes them byte for byte, so they are never rewritten;
//! - **constructed** ones (`example/widget`), for the lifecycle cases the
//!   registry has no instance of today: `extends`, `withdrawn`, `retired`, and a
//!   draft edited in place.
//!
//! `full_registry_round_trip` additionally runs against a whole registry when
//! `TRUST_TASKS_TEST_REGISTRY_FULL` names one (CI builds it with the codegen).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/bundles");

fn digest_of(bytes: &[u8]) -> String {
    let h = Sha256::digest(bytes);
    format!(
        "sha256:{}",
        h.iter().map(|b| format!("{b:02x}")).collect::<String>()
    )
}

struct Published {
    entry: Value,
}

struct TestRegistry {
    dir: TempDir,
    /// slug → version → index entry
    entries: BTreeMap<(String, String), Published>,
    by_digest: bool,
}

impl TestRegistry {
    fn new() -> Self {
        TestRegistry {
            dir: TempDir::new().unwrap(),
            entries: BTreeMap::new(),
            by_digest: true,
        }
    }

    fn url(&self) -> String {
        format!("file://{}", self.dir.path().display())
    }

    fn specs(&self) -> PathBuf {
        self.dir.path().join("specs")
    }

    /// Publishes `bytes` at its path and (when enabled) in the by-digest
    /// store, replacing any earlier bytes at the path — as a deploy does.
    fn publish(&mut self, bytes: &[u8]) -> String {
        let b: Value = serde_json::from_slice(bytes).unwrap();
        let slug = b["slug"].as_str().unwrap().to_string();
        let version = b["version"].as_str().unwrap().to_string();
        let path = format!("{slug}/{version}/bundle.json");
        let digest = digest_of(bytes);
        let file = self.specs().join(&path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, bytes).unwrap();
        if self.by_digest {
            let store = self.specs().join("by-digest");
            std::fs::create_dir_all(&store).unwrap();
            let hex = digest.strip_prefix("sha256:").unwrap();
            std::fs::write(store.join(format!("sha256-{hex}.json")), bytes).unwrap();
        }
        let entry = json!({
            "typeUri": b["typeUri"], "slug": slug, "version": version,
            "status": b["status"], "deprecation": b["deprecation"], "extends": b["extends"],
            "digest": digest, "path": path,
        });
        self.entries.insert((slug, version), Published { entry });
        self.write_index();
        digest
    }

    fn publish_fixture(&mut self, slug_version: &str) -> String {
        let bytes = std::fs::read(format!("{FIXTURES}/{slug_version}/bundle.json")).unwrap();
        self.publish(&bytes)
    }

    fn publish_real(&mut self) {
        for sv in [
            "vault/upsert/0.3",
            "vault/upsert/0.4",
            "acl/grant/0.1",
            "acl/grant/0.2",
        ] {
            self.publish_fixture(sv);
        }
    }

    /// Changes a version's status in the index, as the withdrawal batch does.
    fn set_status(&mut self, slug: &str, version: &str, status: &str) {
        let p = self
            .entries
            .get_mut(&(slug.to_string(), version.to_string()))
            .unwrap();
        p.entry["status"] = json!(status);
        self.write_index();
    }

    fn write_index(&self) {
        // Ordered by slug, then numerically by version, as the real index is.
        let mut specs: Vec<&Value> = self.entries.values().map(|p| &p.entry).collect();
        specs.sort_by_key(|e| {
            let v = e["version"].as_str().unwrap();
            let (a, b) = v.split_once('.').unwrap();
            (
                e["slug"].as_str().unwrap().to_string(),
                a.parse::<u32>().unwrap(),
                b.parse::<u32>().unwrap(),
            )
        });
        let index = json!({
            "indexVersion": 1, "bundleVersion": 1, "generatedFrom": null,
            "excluded": [{"slug": "trust-task-error", "reason": "modelled by hand in every runtime; no generated module to bundle"}],
            "specs": specs,
        });
        std::fs::create_dir_all(self.specs()).unwrap();
        std::fs::write(
            self.specs().join("index.json"),
            serde_json::to_vec(&index).unwrap(),
        )
        .unwrap();
    }
}

/// A constructed bundle with the members the CLI reads, plus a payload schema
/// so `marker` can make two bundles of the same version differ.
fn widget(version: &str, status: &str, extends: Option<&str>, marker: &str) -> Vec<u8> {
    let v = json!({
        "bundleVersion": 1,
        "deprecation": null,
        "errorCodes": [],
        "extends": extends,
        "policy": {"bearer": false},
        "responseSchema": null,
        "schema": {"description": marker, "type": "object"},
        "slug": "example/widget",
        "status": status,
        "supersededBy": null,
        "targetFrameworkVersion": "0.5.0",
        "typeUri": format!("https://trusttasks.org/spec/example/widget/{version}"),
        "version": version,
    });
    serde_json::to_vec(&v).unwrap()
}

struct App {
    dir: TempDir,
    registry: String,
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

impl Run {
    fn ok(self) -> Self {
        assert_eq!(
            self.code, 0,
            "expected success\nstdout:\n{}\nstderr:\n{}",
            self.stdout, self.stderr
        );
        self
    }
    fn fails(self) -> Self {
        assert_ne!(
            self.code, 0,
            "expected failure\nstdout:\n{}\nstderr:\n{}",
            self.stdout, self.stderr
        );
        self
    }
    fn all(&self) -> String {
        format!("{}{}", self.stdout, self.stderr)
    }
}

impl App {
    fn new(reg: &TestRegistry) -> Self {
        App {
            dir: TempDir::new().unwrap(),
            registry: reg.url(),
        }
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.dir.path().join(rel)
    }

    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.path(rel)).unwrap()
    }

    fn run(&self, args: &[&str]) -> Run {
        let out: Output = Command::new(env!("CARGO_BIN_EXE_trust-tasks"))
            .arg("-C")
            .arg(self.dir.path())
            .args(args)
            .env("TRUST_TASKS_REGISTRY", &self.registry)
            .output()
            .unwrap();
        Run {
            code: out.status.code().unwrap_or(-1),
            stdout: String::from_utf8(out.stdout).unwrap(),
            stderr: String::from_utf8(out.stderr).unwrap(),
        }
    }

    /// Every file under the app, path → bytes.
    fn snapshot(&self) -> BTreeMap<String, Vec<u8>> {
        fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Vec<u8>>) {
            for e in std::fs::read_dir(dir).unwrap() {
                let p = e.unwrap().path();
                if p.is_dir() {
                    walk(root, &p, out);
                } else {
                    let rel = p
                        .strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/");
                    out.insert(rel, std::fs::read(&p).unwrap());
                }
            }
        }
        let mut out = BTreeMap::new();
        walk(self.dir.path(), self.dir.path(), &mut out);
        out
    }
}

#[test]
fn add_vendors_real_bundles_by_digest_and_writes_the_lock() {
    let mut reg = TestRegistry::new();
    reg.publish_real();
    let app = App::new(&reg);

    let r = app
        .run(&["add", "vault/upsert/0.4", "acl/grant@latest"])
        .ok();
    assert!(r.stdout.contains("Added acl/grant 0.2"), "{}", r.stdout);
    assert!(r.stdout.contains("Added vault/upsert 0.4"), "{}", r.stdout);
    assert!(
        !r.stderr.contains("by-digest store"),
        "fetched by digest, no fallback: {}",
        r.stderr
    );

    // The manifest records intent; the lock the resolution, in the documented layout.
    let manifest = app.read("trust-tasks.toml");
    assert!(
        manifest.contains("\"acl/grant\" = \"latest\""),
        "{manifest}"
    );
    assert!(
        manifest.contains("\"vault/upsert\" = \"0.4\""),
        "{manifest}"
    );
    let acl = digest_of(&std::fs::read(format!("{FIXTURES}/acl/grant/0.2/bundle.json")).unwrap());
    let vault =
        digest_of(&std::fs::read(format!("{FIXTURES}/vault/upsert/0.4/bundle.json")).unwrap());
    assert_eq!(
        app.read("trust-tasks.lock"),
        format!(
            "# Written by `trust-tasks`. Do not edit by hand.\n\
             version = 1\n\
             registry = \"{}\"\n\
             \n[[task]]\nslug = \"acl/grant\"\nversion = \"0.2\"\ntype_uri = \"https://trusttasks.org/spec/acl/grant/0.2\"\ndigest = \"{acl}\"\n\
             \n[[task]]\nslug = \"vault/upsert\"\nversion = \"0.4\"\ntype_uri = \"https://trusttasks.org/spec/vault/upsert/0.4\"\ndigest = \"{vault}\"\n",
            reg.url()
        )
    );
    // Vendored bytes are the registry's bytes.
    assert_eq!(
        std::fs::read(app.path("trust-tasks/vault/upsert/0.4.bundle.json")).unwrap(),
        std::fs::read(format!("{FIXTURES}/vault/upsert/0.4/bundle.json")).unwrap()
    );
    app.run(&["check"]).ok();
}

#[test]
fn a_bare_slug_pins_the_latest_live_version_exactly() {
    let mut reg = TestRegistry::new();
    reg.publish_real();
    let app = App::new(&reg);
    let r = app.run(&["add", "vault/upsert"]).ok();
    assert!(r.stdout.contains("Added vault/upsert 0.4"), "{}", r.stdout);
    assert!(app
        .read("trust-tasks.toml")
        .contains("\"vault/upsert\" = \"0.4\""));
}

#[test]
fn add_falls_back_to_the_path_while_the_by_digest_store_is_new() {
    let mut reg = TestRegistry::new();
    reg.by_digest = false;
    reg.publish_real();
    let app = App::new(&reg);
    let r = app.run(&["add", "acl/grant/0.1"]).ok();
    assert!(
        r.stderr
            .contains("not in the registry's by-digest store yet")
            && r.stderr.contains("the by-digest store is new"),
        "{}",
        r.stderr
    );
    app.run(&["check"]).ok();
}

#[test]
fn add_is_idempotent() {
    let mut reg = TestRegistry::new();
    reg.publish_real();
    let app = App::new(&reg);
    app.run(&["add", "vault/upsert/0.3", "acl/grant@latest"])
        .ok();
    let before = app.snapshot();
    let r = app
        .run(&["add", "vault/upsert/0.3", "acl/grant@latest"])
        .ok();
    assert!(r.stdout.contains("Unchanged acl/grant 0.2"), "{}", r.stdout);
    assert!(
        r.stdout.contains("Unchanged vault/upsert 0.3"),
        "{}",
        r.stdout
    );
    assert_eq!(app.snapshot(), before);
    // And an update with nothing new changes nothing either.
    app.run(&["update"]).ok();
    assert_eq!(app.snapshot(), before);
}

#[test]
fn add_keeps_the_authors_comments() {
    let mut reg = TestRegistry::new();
    reg.publish_real();
    let app = App::new(&reg);
    std::fs::write(
        app.path("trust-tasks.toml"),
        "# our tasks\n[tasks]\n\"vault/upsert\" = \"0.3\" # until the migration\n",
    )
    .unwrap();
    app.run(&["update"]).ok();
    app.run(&["add", "acl/grant/0.2"]).ok();
    let m = app.read("trust-tasks.toml");
    assert!(m.starts_with("# our tasks\n"), "{m}");
    assert!(
        m.contains("\"vault/upsert\" = \"0.3\" # until the migration"),
        "{m}"
    );
    assert!(
        m.find("acl/grant").unwrap() < m.find("vault/upsert").unwrap(),
        "sorted: {m}"
    );
}

#[test]
fn unknown_slugs_and_versions_are_clear_errors() {
    let mut reg = TestRegistry::new();
    reg.publish_real();
    let app = App::new(&reg);
    let r = app.run(&["add", "vault/upsert/0.9"]).fails();
    assert!(
        r.stderr
            .contains("vault/upsert has no version 0.9 in the registry (it has 0.3, 0.4)"),
        "{}",
        r.stderr
    );
    let r = app.run(&["add", "vault/upsrt"]).fails();
    assert!(
        r.stderr.contains("did you mean one of: vault/upsert"),
        "{}",
        r.stderr
    );
    let r = app.run(&["add", "trust-task-error"]).fails();
    assert!(
        r.stderr
            .contains("has no registry bundle: modelled by hand"),
        "{}",
        r.stderr
    );
    let r = app.run(&["add", "../../etc"]).fails();
    assert!(
        r.stderr.contains("is not a specification slug"),
        "{}",
        r.stderr
    );
    // Nothing was written by any of them.
    assert!(app.snapshot().is_empty());
}

#[test]
fn check_detects_a_tampered_vendored_bundle() {
    let mut reg = TestRegistry::new();
    reg.publish_real();
    let app = App::new(&reg);
    app.run(&["add", "vault/upsert/0.4"]).ok();
    let f = app.path("trust-tasks/vault/upsert/0.4.bundle.json");
    let mut bytes = std::fs::read(&f).unwrap();
    let i = bytes.len() / 2;
    bytes[i] ^= 0x01;
    std::fs::write(&f, &bytes).unwrap();
    let r = app.run(&["check"]).fails();
    assert!(
        r.stdout
            .contains("trust-tasks/vault/upsert/0.4.bundle.json does not match trust-tasks.lock"),
        "{}",
        r.stdout
    );
    // `fetch` restores the locked bytes, by digest …
    let r = app.run(&["fetch"]).ok();
    assert!(
        r.stdout.contains("Fetched vault/upsert 0.4"),
        "{}",
        r.stdout
    );
    app.run(&["check"]).ok();
    let r = app.run(&["fetch"]).ok();
    assert_eq!(r.stdout, "ok: all 1 vendored bundle(s) present\n");
    // … and so does `update`, which also re-resolves.
    std::fs::write(&f, b"{}").unwrap();
    let r = app.run(&["update"]).ok();
    assert!(
        r.stdout.contains("Restored vault/upsert 0.4"),
        "{}",
        r.stdout
    );
    app.run(&["check"]).ok();
}

#[test]
fn check_detects_missing_stray_and_drift() {
    let mut reg = TestRegistry::new();
    reg.publish_real();
    let app = App::new(&reg);
    app.run(&["add", "vault/upsert/0.4", "acl/grant/0.2"]).ok();

    std::fs::remove_file(app.path("trust-tasks/acl/grant/0.2.bundle.json")).unwrap();
    std::fs::create_dir_all(app.path("trust-tasks/old/task")).unwrap();
    std::fs::write(app.path("trust-tasks/old/task/0.1.bundle.json"), b"{}").unwrap();
    // Hand edits: a new task, and a pin moved without running update.
    let m = app.read("trust-tasks.toml").replace(
        "\"vault/upsert\" = \"0.4\"",
        "\"vault/upsert\" = \"0.3\"\n\"acl/revoke\" = \"latest\"",
    );
    std::fs::write(app.path("trust-tasks.toml"), m).unwrap();

    let r = app.run(&["check"]).fails();
    for want in [
        "trust-tasks/acl/grant/0.2.bundle.json is missing",
        "trust-tasks/old/task/0.1.bundle.json is not in trust-tasks.lock",
        "trust-tasks.toml has acl/revoke = \"latest\" but trust-tasks.lock locks 0 versions of it",
        "trust-tasks.toml pins vault/upsert 0.3 but trust-tasks.lock does not",
        "trust-tasks.lock locks vault/upsert 0.4 but trust-tasks.toml does not pin it",
        "5 problem(s)",
    ] {
        assert!(r.stdout.contains(want), "missing {want:?} in\n{}", r.stdout);
    }
}

#[test]
fn check_is_offline() {
    let mut reg = TestRegistry::new();
    reg.publish_real();
    let app = App::new(&reg);
    app.run(&["add", "vault/upsert/0.4"]).ok();
    drop(reg); // the registry is gone
    app.run(&["check"]).ok();
    app.run(&["list"]).ok();
}

#[test]
fn check_reports_a_lock_resolved_against_another_registry() {
    let mut reg = TestRegistry::new();
    reg.publish_real();
    let app = App::new(&reg);
    app.run(&["add", "vault/upsert/0.4"]).ok();
    let r = app
        .run(&["--registry", "https://mirror.example", "check"])
        .fails();
    assert!(
        r.stdout
            .contains("but the registry is https://mirror.example"),
        "{}",
        r.stdout
    );
}

#[test]
fn a_registry_serving_the_wrong_bytes_is_refused() {
    // By digest: a JSON body under the content address that is other content.
    let mut reg = TestRegistry::new();
    let digest = reg.publish_fixture("vault/upsert/0.4");
    let hex = digest.strip_prefix("sha256:").unwrap();
    let other = std::fs::read(format!("{FIXTURES}/vault/upsert/0.3/bundle.json")).unwrap();
    std::fs::write(
        reg.specs().join(format!("by-digest/sha256-{hex}.json")),
        &other,
    )
    .unwrap();
    let app = App::new(&reg);
    let r = app.run(&["add", "vault/upsert/0.4"]).fails();
    assert!(r.stderr.contains("digest mismatch"), "{}", r.stderr);
    assert!(app.snapshot().is_empty(), "nothing written");

    // By path: the bytes at the path are not the index's digest.
    let mut reg = TestRegistry::new();
    reg.by_digest = false;
    reg.publish_fixture("vault/upsert/0.4");
    std::fs::write(reg.specs().join("vault/upsert/0.4/bundle.json"), &other).unwrap();
    let app = App::new(&reg);
    let r = app.run(&["add", "vault/upsert/0.4"]).fails();
    assert!(
        r.stderr.contains("digest mismatch") && r.stderr.contains("the registry index says"),
        "{}",
        r.stderr
    );
    assert!(app.snapshot().is_empty(), "nothing written");
}

#[test]
fn withdrawn_versions_are_refused() {
    let mut reg = TestRegistry::new();
    reg.publish(&widget("0.4", "draft", None, "a"));
    reg.publish(&widget("0.5", "draft", None, "a"));
    let app = App::new(&reg);
    app.run(&["add", "example/widget/0.4"]).ok();
    let before = app.snapshot();

    reg.set_status("example/widget", "0.4", "withdrawn");
    let r = app.run(&["update"]).fails();
    assert!(
        r.stderr.contains("example/widget 0.4 is withdrawn; update refuses it — move the pin with `trust-tasks add example/widget/0.5`"),
        "{}",
        r.stderr
    );
    // The pin and its vendored bundle stay as they were: it still builds.
    assert_eq!(app.snapshot(), before);
    app.run(&["check"]).ok();

    let r = app.run(&["add", "example/widget/0.4"]).fails();
    assert!(r.stderr.contains("example/widget 0.4 is withdrawn: it is no longer offered for new pins; the latest live version is example/widget/0.5"), "{}", r.stderr);

    // `latest` never resolves to a withdrawn (or retired) version.
    reg.set_status("example/widget", "0.5", "withdrawn");
    reg.publish(&widget("0.6", "retired", None, "a"));
    let r = app.run(&["add", "example/widget@latest"]).fails();
    assert!(r.stderr.contains("has no live version"), "{}", r.stderr);
}

#[test]
fn update_moves_latest_along_extends_and_flags_a_breaking_move() {
    let mut reg = TestRegistry::new();
    reg.publish(&widget("0.4", "draft", None, "a"));
    let app = App::new(&reg);
    app.run(&["add", "example/widget@latest"]).ok();

    reg.publish(&widget("0.5", "draft", Some("0.4"), "a"));
    let r = app.run(&["update"]).ok();
    assert!(
        r.stdout.contains("Moved example/widget 0.4 → 0.5")
            && r.stdout.contains("0.5 extends 0.4: no code change needed"),
        "{}",
        r.stdout
    );
    assert!(app
        .path("trust-tasks/example/widget/0.5.bundle.json")
        .exists());
    assert!(!app
        .path("trust-tasks/example/widget/0.4.bundle.json")
        .exists());
    assert!(app
        .read("trust-tasks.toml")
        .contains("\"example/widget\" = \"latest\""));

    // A chain: 0.7 extends 0.6 extends 0.5 reaches 0.5.
    reg.publish(&widget("0.6", "draft", Some("0.5"), "a"));
    reg.publish(&widget("0.7", "draft", Some("0.6"), "a"));
    let r = app.run(&["update", "example/widget"]).ok();
    assert!(
        r.stdout.contains("0.7 extends 0.5: no code change needed"),
        "{}",
        r.stdout
    );

    reg.publish(&widget("1.0", "draft", None, "a"));
    let r = app.run(&["update"]).ok();
    assert!(
        r.stdout.contains("breaking: 1.0 does not extend 0.7"),
        "{}",
        r.stdout
    );
    app.run(&["check"]).ok();
}

#[test]
fn update_leaves_exact_pins_but_says_what_is_available() {
    let mut reg = TestRegistry::new();
    reg.publish(&widget("0.4", "draft", None, "a"));
    reg.publish(&widget("0.5", "draft", Some("0.4"), "a"));
    let app = App::new(&reg);
    app.run(&["add", "example/widget/0.4"]).ok();
    let r = app.run(&["update"]).ok();
    assert!(
        r.stdout.contains("Unchanged example/widget 0.4"),
        "{}",
        r.stdout
    );
    assert!(
        r.stderr
            .contains("example/widget 0.5 is available (0.5 extends 0.4: no code change needed)"),
        "{}",
        r.stderr
    );

    let r = app.run(&["update", "--to-latest"]).ok();
    assert!(
        r.stdout.contains("Moved example/widget 0.4 → 0.5"),
        "{}",
        r.stdout
    );
    assert!(app
        .read("trust-tasks.toml")
        .contains("\"example/widget\" = \"0.5\""));
    app.run(&["check"]).ok();
}

#[test]
fn a_draft_changed_in_place_keeps_its_pin_until_update() {
    let mut reg = TestRegistry::new();
    let old = reg.publish(&widget("0.4", "draft", None, "first"));
    let app = App::new(&reg);
    app.run(&["add", "example/widget/0.4"]).ok();

    // The registry edits the draft in place; the old digest stays in the store.
    let new = reg.publish(&widget("0.4", "draft", None, "second"));
    assert_ne!(old, new);
    app.run(&["check"]).ok(); // offline: still exactly what the lock pins

    // A vendored bundle deleted now is restored by digest — the old bytes,
    // which the version's path no longer serves.
    let vendored = app.path("trust-tasks/example/widget/0.4.bundle.json");
    std::fs::remove_file(&vendored).unwrap();
    assert!(app.run(&["check"]).fails().stdout.contains("is missing"));
    app.run(&["fetch"]).ok();
    assert!(std::fs::read_to_string(&vendored)
        .unwrap()
        .contains("\"first\""));
    assert!(app.read("trust-tasks.lock").contains(&old));
    app.run(&["check"]).ok();

    // `update` is what takes the new bytes, and says why.
    let r = app.run(&["update"]).ok();
    assert!(
        r.stdout.contains("changed it in place (draft)"),
        "{}",
        r.stdout
    );
    assert!(app.read("trust-tasks.lock").contains(&new));
    app.run(&["check"]).ok();
}

#[test]
fn a_pinned_digest_the_registry_lost_is_an_error() {
    let mut reg = TestRegistry::new();
    reg.by_digest = false;
    reg.publish(&widget("0.4", "draft", None, "first"));
    let app = App::new(&reg);
    app.run(&["add", "example/widget@latest"]).ok();
    // Edited in place, no by-digest store: the old bytes are gone.
    reg.publish(&widget("0.4", "draft", None, "second"));
    std::fs::remove_file(app.path("trust-tasks/example/widget/0.4.bundle.json")).unwrap();
    let r = app.run(&["fetch"]).fails();
    assert!(
        r.stderr
            .contains("example/widget 0.4: the registry no longer has the bundle with digest"),
        "{}",
        r.stderr
    );
    // `update` can still move the pin to the bytes the registry has now.
    let r = app.run(&["update"]).ok();
    assert!(r.stdout.contains("changed it in place"), "{}", r.stdout);
    app.run(&["check"]).ok();
}

#[test]
fn remove_deletes_vendored_bundles_and_empty_directories() {
    let mut reg = TestRegistry::new();
    reg.publish_real();
    let app = App::new(&reg);
    app.run(&["add", "vault/upsert/0.4", "acl/grant/0.2"]).ok();
    let r = app.run(&["remove", "acl/grant"]).ok();
    assert!(r.stdout.contains("Removed acl/grant 0.2"), "{}", r.stdout);
    assert!(!app.path("trust-tasks/acl").exists());
    assert!(app
        .path("trust-tasks/vault/upsert/0.4.bundle.json")
        .exists());
    assert!(!app.read("trust-tasks.toml").contains("acl/grant"));
    assert!(!app.read("trust-tasks.lock").contains("acl/grant"));
    app.run(&["check"]).ok();
    let r = app.run(&["remove", "acl/grant"]).fails();
    assert!(
        r.stderr.contains("acl/grant is not in trust-tasks.toml"),
        "{}",
        r.stderr
    );
}

#[test]
fn two_versions_of_one_task() {
    let mut reg = TestRegistry::new();
    reg.publish_real();
    let app = App::new(&reg);
    std::fs::write(
        app.path("trust-tasks.toml"),
        "[tasks]\n\"vault/upsert\" = [\"0.4\", \"0.3\"]\n",
    )
    .unwrap();
    app.run(&["update"]).ok();
    let lock = app.read("trust-tasks.lock");
    assert!(lock.find("version = \"0.3\"").unwrap() < lock.find("version = \"0.4\"").unwrap());
    app.run(&["check"]).ok();
    app.run(&["remove", "vault/upsert/0.3"]).ok();
    assert!(app
        .read("trust-tasks.toml")
        .contains("\"vault/upsert\" = \"0.4\""));
    assert!(!app
        .path("trust-tasks/vault/upsert/0.3.bundle.json")
        .exists());
    app.run(&["check"]).ok();
}

#[test]
fn list_shows_status_deprecation_and_remote_versions() {
    let mut reg = TestRegistry::new();
    reg.publish_real();
    reg.publish(&widget("0.4", "draft", None, "a"));
    let app = App::new(&reg);
    app.run(&["add", "vault/upsert/0.3", "example/widget@latest"])
        .ok();
    reg.publish(&widget("0.5", "draft", Some("0.4"), "a"));

    let r = app.run(&["list"]).ok();
    assert_eq!(
        r.stdout,
        "TASK            VERSION  INTENT  STATUS  NOTE\n\
         example/widget  0.4      latest  draft\n\
         vault/upsert    0.3      exact   draft   superseded by vault/upsert/0.4\n"
    );
    let r = app.run(&["list", "--remote"]).ok();
    assert_eq!(
        r.stdout,
        "TASK            VERSION  INTENT  STATUS  LATEST                         NOTE\n\
         example/widget  0.4      latest  draft   0.5 (extends, no code change)\n\
         vault/upsert    0.3      exact   draft   0.4 (breaking)                 superseded by vault/upsert/0.4\n"
    );
}

#[test]
fn the_manifest_registry_is_used_and_the_flag_overrides_it() {
    let mut reg = TestRegistry::new();
    reg.publish_real();
    let app = App {
        dir: TempDir::new().unwrap(),
        // The environment names a registry that does not exist …
        registry: "file:///nonexistent-registry".into(),
    };
    let r = app.run(&["add", "vault/upsert/0.4"]).fails();
    assert!(
        r.stderr
            .contains("no registry index at /nonexistent-registry/specs/index.json"),
        "{}",
        r.stderr
    );
    // … and the flag beats it.
    app.run(&["--registry", &reg.url(), "add", "vault/upsert/0.4"])
        .ok();
    assert!(app
        .read("trust-tasks.lock")
        .contains(&format!("registry = \"{}\"", reg.url())));
}

#[test]
fn commands_other_than_add_need_a_manifest() {
    let reg = TestRegistry::new();
    let app = App::new(&reg);
    for cmd in [&["check"][..], &["update"], &["list"], &["remove", "a/b"]] {
        let r = app.run(cmd).fails();
        assert!(
            r.stderr.contains("no trust-tasks.toml in"),
            "{cmd:?}: {}",
            r.all()
        );
    }
}

/// Against a whole registry, when CI has built one
/// (`cargo run -p trust-tasks-codegen -- --bundles <dir>/specs` plus the
/// by-digest store): every bundle the registry serves is fetchable and
/// verifiable, not just the fixtures.
#[test]
fn full_registry_round_trip() {
    let Ok(full) = std::env::var("TRUST_TASKS_TEST_REGISTRY_FULL") else {
        eprintln!("skipped: TRUST_TASKS_TEST_REGISTRY_FULL is not set");
        return;
    };
    let app = App {
        dir: TempDir::new().unwrap(),
        registry: full,
    };
    let r = app
        .run(&[
            "add",
            "vault/upsert",
            "acl/grant@latest",
            "trust-task-discovery",
        ])
        .ok();
    assert!(
        !r.stderr.contains("by-digest store yet"),
        "the store is complete: {}",
        r.stderr
    );
    app.run(&["check"]).ok();
    let before = app.snapshot();
    app.run(&["update"]).ok();
    assert_eq!(app.snapshot(), before);
    app.run(&["list", "--remote"]).ok();
    app.run(&["remove", "acl/grant"]).ok();
    app.run(&["check"]).ok();
}

/// `generate` verifies the lock, then runs the language's generator from the
/// project directory with absolute `--lock` / `--out` and the arguments after
/// `--`. A stand-in `npx` on PATH records what it was given.
#[cfg(unix)]
#[test]
fn generate_runs_the_generator_after_check() {
    use std::os::unix::fs::PermissionsExt;

    let mut reg = TestRegistry::new();
    reg.publish_real();
    let app = App::new(&reg);
    app.run(&["add", "vault/upsert/0.4"]).ok();

    let bin = TempDir::new().unwrap();
    let record = bin.path().join("called");
    let npx = bin.path().join("npx");
    std::fs::write(
        &npx,
        format!(
            "#!/bin/sh\npwd > '{0}'\nprintf '%s\\n' \"$@\" >> '{0}'\n",
            record.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&npx, std::fs::Permissions::from_mode(0o755)).unwrap();
    let generate = |args: &[&str], path: &std::ffi::OsStr| {
        let out = Command::new(env!("CARGO_BIN_EXE_trust-tasks"))
            .arg("-C")
            .arg(app.dir.path())
            .args(args)
            .env("TRUST_TASKS_REGISTRY", &app.registry)
            .env("PATH", path)
            .output()
            .unwrap();
        Run {
            code: out.status.code().unwrap_or(-1),
            stdout: String::from_utf8(out.stdout).unwrap(),
            stderr: String::from_utf8(out.stderr).unwrap(),
        }
    };

    generate(
        &[
            "generate", "--lang", "ts", "--out", "/abs/out", "--", "--extra",
        ],
        bin.path().as_os_str(),
    )
    .ok();
    let called = std::fs::read_to_string(&record).unwrap();
    let lines: Vec<&str> = called.lines().collect();
    let root = app.dir.path().canonicalize().unwrap();
    assert_eq!(Path::new(lines[0]).canonicalize().unwrap(), root);
    assert_eq!(lines[1..3], ["--no-install", "trust-tasks-gen-ts"]);
    assert_eq!(lines[3], "--lock");
    assert_eq!(
        Path::new(lines[4]).canonicalize().unwrap(),
        root.join("trust-tasks.lock")
    );
    assert_eq!(lines[5..], ["--out", "/abs/out", "--extra"]);

    // --out is required; a missing tool says what to install.
    let r = generate(&["generate", "--lang", "ts"], bin.path().as_os_str()).fails();
    assert!(r.stderr.contains("--out <dir> is required"), "{}", r.stderr);
    let empty = TempDir::new().unwrap();
    let r = generate(
        &["generate", "--lang", "ts", "--out", "x"],
        empty.path().as_os_str(),
    )
    .fails();
    assert!(
        r.stderr.contains("npm i -D @openvtc/trust-tasks-gen"),
        "{}",
        r.stderr
    );

    // Rust generates in build.rs; the command says so after checking.
    let r = generate(&["generate", "--lang", "rs"], empty.path().as_os_str()).ok();
    assert!(
        r.stdout.contains("trust_tasks_build::Config"),
        "{}",
        r.stdout
    );

    // A lock that does not verify generates nothing.
    std::fs::remove_file(&record).unwrap();
    std::fs::write(app.path("trust-tasks/vault/upsert/0.4.bundle.json"), b"{}").unwrap();
    generate(
        &["generate", "--lang", "ts", "--out", "/abs/out"],
        bin.path().as_os_str(),
    )
    .fails();
    assert!(
        !record.exists(),
        "the generator ran on a lock that does not verify"
    );
}
