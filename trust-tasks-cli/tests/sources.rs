//! Third-party sources (`[sources.<name>]`, SPEC §6.5), end to end: a
//! publisher builds its registry with `trust-tasks registry build`, and an
//! application pins its tasks beside official ones — from a directory, and
//! over https from a mock server with its own CA.
//!
//! The third party is `acme`, under `https://tasks.acme.example`, with the
//! specifications in `tests/fixtures/acme/specs/`: `payments/settle` 1.0 and
//! 1.1 (1.1 extends 1.0; both `$ref` a local shared file and the registry's
//! `Ext` by absolute URL), and its own `vault/upsert` 0.4 — the registry's
//! slug and version, and a different specification.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write as _};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
const REPO_SPECS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../specs");
const ACME: &str = "https://tasks.acme.example";

fn digest_of(bytes: &[u8]) -> String {
    let h = Sha256::digest(bytes);
    format!(
        "sha256:{}",
        h.iter().map(|b| format!("{b:02x}")).collect::<String>()
    )
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

fn trust_tasks(args: &[&str], env: &[(&str, &str)]) -> Run {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_trust-tasks"));
    cmd.args(args).env_remove("TRUST_TASKS_CA_FILE");
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd.output().unwrap();
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8(out.stdout).unwrap(),
        stderr: String::from_utf8(out.stderr).unwrap(),
    }
}

/// Writes `specs/index.json` (and the by-digest store) for `bundles` under
/// `root`, as a registry deploy does.
fn publish(root: &Path, bundles: &[Vec<u8>]) {
    let specs = root.join("specs");
    let mut entries: Vec<Value> = Vec::new();
    for bytes in bundles {
        let b: Value = serde_json::from_slice(bytes).unwrap();
        let path = format!(
            "{}/{}/bundle.json",
            b["slug"].as_str().unwrap(),
            b["version"].as_str().unwrap()
        );
        let file = specs.join(&path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, bytes).unwrap();
        let digest = digest_of(bytes);
        std::fs::create_dir_all(specs.join("by-digest")).unwrap();
        std::fs::write(
            specs.join(format!("by-digest/sha256-{}.json", &digest[7..])),
            bytes,
        )
        .unwrap();
        entries.push(json!({
            "typeUri": b["typeUri"], "slug": b["slug"], "version": b["version"],
            "status": b["status"], "deprecation": b["deprecation"], "extends": b["extends"],
            "digest": digest, "path": path,
        }));
    }
    let index = json!({
        "indexVersion": 1, "bundleVersion": 1, "generatedFrom": null,
        "excluded": [], "specs": entries,
    });
    std::fs::write(
        specs.join("index.json"),
        serde_json::to_vec(&index).unwrap(),
    )
    .unwrap();
}

/// A bundle of `base` with some members replaced, re-serialized.
fn altered(base: &[u8], changes: Value) -> Vec<u8> {
    let mut b: Value = serde_json::from_slice(base).unwrap();
    for (k, v) in changes.as_object().unwrap() {
        b[k] = v.clone();
    }
    serde_json::to_vec(&b).unwrap()
}

/// An official registry, an acme site built by `registry build`, and an
/// application directory, side by side in one temporary directory.
struct World {
    tmp: TempDir,
}

impl World {
    fn new() -> World {
        let w = World {
            tmp: TempDir::new().unwrap(),
        };
        let official: Vec<Vec<u8>> = ["vault/upsert/0.4", "acl/grant/0.2"]
            .iter()
            .map(|sv| std::fs::read(format!("{FIXTURES}/bundles/{sv}/bundle.json")).unwrap())
            .collect();
        publish(&w.path("official"), &official);
        let site = w.path("acme-site");
        trust_tasks(
            &[
                "registry",
                "build",
                "--specs",
                &format!("{FIXTURES}/acme/specs"),
                "--type-uri-prefix",
                ACME,
                "--out",
                site.to_str().unwrap(),
                "--official-specs",
                REPO_SPECS,
            ],
            &[],
        )
        .ok();
        std::fs::create_dir_all(w.path("app")).unwrap();
        w
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.tmp.path().join(rel)
    }

    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.path(rel)).unwrap()
    }

    fn manifest(&self, text: &str) {
        std::fs::write(self.path("app/trust-tasks.toml"), text).unwrap();
    }

    fn run(&self, args: &[&str]) -> Run {
        self.run_env(args, &[])
    }

    fn run_env(&self, args: &[&str], env: &[(&str, &str)]) -> Run {
        let app = self.path("app");
        let registry = format!("file://{}", self.path("official").display());
        let mut all: Vec<(&str, &str)> = vec![("TRUST_TASKS_REGISTRY", &registry)];
        all.extend_from_slice(env);
        let mut full = vec!["-C", app.to_str().unwrap()];
        full.extend_from_slice(args);
        trust_tasks(&full, &all)
    }

    fn acme_bundle(&self, slug_version: &str) -> Vec<u8> {
        std::fs::read(self.path(&format!("acme-site/specs/{slug_version}/bundle.json"))).unwrap()
    }
}

const ACME_PATH_SOURCE: &str = "[sources.acme]\npath = \"../acme-site\"\n\n[tasks]\n";

#[test]
fn registry_build_writes_the_served_layout() {
    let w = World::new();
    let specs = w.path("acme-site/specs");
    let index: Value =
        serde_json::from_slice(&std::fs::read(specs.join("index.json")).unwrap()).unwrap();
    let listed: Vec<(&str, &str)> = index["specs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["typeUri"].as_str().unwrap(),
                e["status"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        listed,
        [
            (
                "https://tasks.acme.example/spec/payments/settle/1.0",
                "draft"
            ),
            (
                "https://tasks.acme.example/spec/payments/settle/1.1",
                "draft"
            ),
            ("https://tasks.acme.example/spec/vault/upsert/0.4", "draft"),
        ]
    );
    for e in index["specs"].as_array().unwrap() {
        let bytes = std::fs::read(specs.join(e["path"].as_str().unwrap())).unwrap();
        let digest = e["digest"].as_str().unwrap();
        assert_eq!(digest_of(&bytes), digest);
        assert_eq!(
            std::fs::read(specs.join(format!("by-digest/sha256-{}.json", &digest[7..]))).unwrap(),
            bytes
        );
        // The consumer-side reader accepts it, and the inlined schemas are
        // self-contained: the local Money and the registry's Ext.
        let bundle = trust_tasks_build::Bundle::from_slice(&bytes).unwrap();
        if bundle.slug == "payments/settle" {
            let defs = bundle.schema["$defs"].as_object().unwrap();
            assert!(
                defs.contains_key("Money") && defs.contains_key("Ext"),
                "{defs:?}"
            );
            assert!(bundle.policy.proof_required.request);
            assert_eq!(bundle.error_codes.len(), 1);
        }
    }
    let settle_1_0: Value = serde_json::from_slice(&w.acme_bundle("payments/settle/1.0")).unwrap();
    assert_eq!(
        settle_1_0["deprecation"],
        "superseded by payments/settle/1.1"
    );

    // Building again changes nothing, and adds nothing to the store.
    let r = trust_tasks(
        &[
            "registry",
            "build",
            "--specs",
            &format!("{FIXTURES}/acme/specs"),
            "--type-uri-prefix",
            ACME,
            "--out",
            w.path("acme-site").to_str().unwrap(),
            "--official-specs",
            REPO_SPECS,
        ],
        &[],
    )
    .ok();
    assert!(r.stdout.contains("(0 new in by-digest"), "{}", r.stdout);

    // The registry's authority is not a third party's to claim.
    let r = trust_tasks(
        &[
            "registry",
            "build",
            "--specs",
            &format!("{FIXTURES}/acme/specs"),
            "--type-uri-prefix",
            "https://trusttasks.org",
            "--out",
            w.path("x").to_str().unwrap(),
            "--official-specs",
            REPO_SPECS,
        ],
        &[],
    )
    .fails();
    assert!(r.stderr.contains("SPEC §6.5 item 1"), "{}", r.stderr);
}

#[test]
fn a_path_source_pins_beside_official_tasks() {
    let w = World::new();
    w.manifest(ACME_PATH_SOURCE);
    let r = w
        .run(&[
            "add",
            "vault/upsert/0.4",
            "acme:vault/upsert/0.4",
            "acme:payments/settle/1.0",
        ])
        .ok();
    assert!(r.stdout.contains("Added vault/upsert 0.4"), "{}", r.stdout);
    assert!(
        r.stdout.contains("Added acme:vault/upsert 0.4"),
        "{}",
        r.stdout
    );
    assert!(
        r.stdout.contains("Added acme:payments/settle 1.0"),
        "{}",
        r.stdout
    );

    // The lock: the source, then official entries, then the source's, each
    // sourced one naming its source; one slug, two authorities, two Type URIs.
    let official = digest_of(
        &std::fs::read(format!("{FIXTURES}/bundles/vault/upsert/0.4/bundle.json")).unwrap(),
    );
    let settle = digest_of(&w.acme_bundle("payments/settle/1.0"));
    let upsert = digest_of(&w.acme_bundle("vault/upsert/0.4"));
    assert_eq!(
        w.read("app/trust-tasks.lock"),
        format!(
            "# Written by `trust-tasks`. Do not edit by hand.\n\
             version = 1\n\
             registry = \"file://{}\"\n\
             \n[sources.acme]\npath = \"../acme-site\"\n\
             \n[[task]]\nslug = \"vault/upsert\"\nversion = \"0.4\"\ntype_uri = \"https://trusttasks.org/spec/vault/upsert/0.4\"\ndigest = \"{official}\"\n\
             \n[[task]]\nsource = \"acme\"\nslug = \"payments/settle\"\nversion = \"1.0\"\ntype_uri = \"https://tasks.acme.example/spec/payments/settle/1.0\"\ndigest = \"{settle}\"\n\
             \n[[task]]\nsource = \"acme\"\nslug = \"vault/upsert\"\nversion = \"0.4\"\ntype_uri = \"https://tasks.acme.example/spec/vault/upsert/0.4\"\ndigest = \"{upsert}\"\n",
            w.path("official").display()
        )
    );
    let manifest = w.read("app/trust-tasks.toml");
    assert!(
        manifest.contains("\"acme:payments/settle\" = \"1.0\""),
        "{manifest}"
    );
    assert!(
        manifest.contains("\"vault/upsert\" = \"0.4\""),
        "{manifest}"
    );
    // Vendored under `@acme/`, byte for byte.
    assert_eq!(
        std::fs::read(w.path("app/trust-tasks/@acme/payments/settle/1.0.bundle.json")).unwrap(),
        w.acme_bundle("payments/settle/1.0")
    );
    assert!(w
        .path("app/trust-tasks/vault/upsert/0.4.bundle.json")
        .is_file());
    assert!(w
        .path("app/trust-tasks/@acme/vault/upsert/0.4.bundle.json")
        .is_file());
    w.run(&["check"]).ok();

    // Generation reads the vendored paths and puts acme's tasks under `acme`.
    let generated = trust_tasks_build::Config::new()
        .lockfile(w.path("app/trust-tasks.lock"))
        .generate()
        .unwrap()
        .source;
    assert!(
        generated.contains("pub mod acme {\npub mod payments {\npub mod settle {\npub mod v1_0 {")
    );
    assert!(generated.contains("\"https://tasks.acme.example/spec/vault/upsert/0.4\""));
    assert!(generated.contains("\"https://trusttasks.org/spec/vault/upsert/0.4\""));

    let r = w.run(&["list"]).ok();
    assert!(
        r.stdout.contains("acme:payments/settle  1.0"),
        "{}",
        r.stdout
    );
    let r = w.run(&["list", "--remote"]).ok();
    assert!(
        r.stdout.contains("1.1 (extends, no code change)"),
        "{}",
        r.stdout
    );

    // `update` says a newer version extends the pin; `add …@latest` moves it.
    let r = w.run(&["update"]).ok();
    assert!(
        r.stderr.contains(
            "acme:payments/settle 1.1 is available (1.1 extends 1.0: no code change needed)"
        ),
        "{}",
        r.stderr
    );
    let r = w.run(&["add", "acme:payments/settle@latest"]).ok();
    assert!(
        r.stdout.contains("Moved acme:payments/settle 1.0 → 1.1")
            && r.stdout.contains("1.1 extends 1.0: no code change needed"),
        "{}",
        r.stdout
    );
    assert!(!w
        .path("app/trust-tasks/@acme/payments/settle/1.0.bundle.json")
        .exists());

    // `fetch` restores a lost sourced bundle by digest; `check` sees it missing first.
    std::fs::remove_file(w.path("app/trust-tasks/@acme/payments/settle/1.1.bundle.json")).unwrap();
    let r = w.run(&["check"]).fails();
    assert!(
        r.stdout
            .contains("trust-tasks/@acme/payments/settle/1.1.bundle.json is missing"),
        "{}",
        r.stdout
    );
    let r = w.run(&["fetch"]).ok();
    assert!(
        r.stdout.contains("Fetched acme:payments/settle 1.1"),
        "{}",
        r.stdout
    );
    w.run(&["check"]).ok();

    // A lock whose sourced Type URI disagrees with the bundle is a problem.
    let lock = w.read("app/trust-tasks.lock");
    std::fs::write(
        w.path("app/trust-tasks.lock"),
        lock.replace(
            "https://tasks.acme.example/spec/vault/upsert/0.4",
            "https://elsewhere.example/spec/vault/upsert/0.4",
        ),
    )
    .unwrap();
    let r = w.run(&["check"]).fails();
    assert!(
        r.stdout
            .contains("is the bundle for https://tasks.acme.example/spec/vault/upsert/0.4"),
        "{}",
        r.stdout
    );
    std::fs::write(w.path("app/trust-tasks.lock"), lock).unwrap();

    // The manifest moving a source is drift `check` reports.
    let manifest = w.read("app/trust-tasks.toml");
    std::fs::write(
        w.path("app/trust-tasks.toml"),
        manifest.replace("../acme-site", "../acme-site/"),
    )
    .unwrap();
    let r = w.run(&["check"]).fails();
    assert!(
        r.stdout
            .contains("resolved source `acme` from path ../acme-site"),
        "{}",
        r.stdout
    );
    std::fs::write(w.path("app/trust-tasks.toml"), manifest).unwrap();

    // Removing acme's tasks removes their bundles, `@acme/` and the source.
    w.run(&["remove", "acme:payments/settle", "acme:vault/upsert"])
        .ok();
    assert!(!w.path("app/trust-tasks/@acme").exists());
    let lock = w.read("app/trust-tasks.lock");
    assert!(
        !lock.contains("[sources.acme]") && !lock.contains("source ="),
        "{lock}"
    );
    w.run(&["check"]).ok();
}

#[test]
fn sources_are_refused_when_they_break_the_rules() {
    let w = World::new();
    let err = |manifest: &str, args: &[&str]| -> String {
        w.manifest(manifest);
        let _ = std::fs::remove_file(w.path("app/trust-tasks.lock"));
        w.run(args).fails().all()
    };

    // Undeclared, http, and badly named sources.
    let e = err("[tasks]\n", &["add", "acme:payments/settle"]);
    assert!(e.contains("source `acme` is not declared"), "{e}");
    let e = err(
        "[sources.acme]\nurl = \"http://tasks.acme.example\"\n",
        &["add", "acme:payments/settle"],
    );
    assert!(e.contains("must be https://"), "{e}");
    let e = err(
        "[sources.Acme]\npath = \"../acme-site\"\n",
        &["add", "vault/upsert"],
    );
    assert!(e.contains("not a source name"), "{e}");

    // A framework slug from a source (SPEC §6.5 item 2).
    let e = err(ACME_PATH_SOURCE, &["add", "acme:trust-task-discovery"]);
    assert!(e.contains("framework slug"), "{e}");

    // A source named for an official family pinned beside it.
    let e = err(
        "[sources.vault]\npath = \"../acme-site\"\n",
        &["add", "vault/upsert/0.4", "vault:payments/settle/1.0"],
    );
    assert!(e.contains("would share the namespace `vault`"), "{e}");

    // Two pins with one Type URI: two names for the same site.
    let e = err(
        "[sources.acme]\npath = \"../acme-site\"\n[sources.acme2]\npath = \"../acme-site\"\n",
        &[
            "add",
            "acme:payments/settle/1.0",
            "acme2:payments/settle/1.0",
        ],
    );
    assert!(e.contains("one Type URI names one specification"), "{e}");

    // A source serving a bundle under the registry's authority (SPEC §6.5 item 1).
    let settle = w.acme_bundle("payments/settle/1.0");
    publish(
        &w.path("evil-site"),
        &[altered(
            &settle,
            json!({"typeUri": "https://trusttasks.org/spec/payments/settle/1.0"}),
        )],
    );
    let e = err(
        "[sources.evil]\npath = \"../evil-site\"\n",
        &["add", "evil:payments/settle/1.0"],
    );
    assert!(
        e.contains("claims the public registry's authority") && e.contains("SPEC §6.5 item 1"),
        "{e}"
    );

    // A source whose bundle's Type URI names another slug than it claims.
    publish(
        &w.path("muddled-site"),
        &[altered(
            &settle,
            json!({"typeUri": "https://tasks.acme.example/spec/payments/other/1.0"}),
        )],
    );
    let e = err(
        "[sources.muddled]\npath = \"../muddled-site\"\n",
        &["add", "muddled:payments/settle/1.0"],
    );
    assert!(
        e.contains("names payments/other 1.0, not muddled:payments/settle 1.0"),
        "{e}"
    );

    // The official registry serving a bundle under another authority.
    let official = std::fs::read(format!("{FIXTURES}/bundles/acl/grant/0.2/bundle.json")).unwrap();
    let mut index: Value =
        serde_json::from_slice(&std::fs::read(w.path("official/specs/index.json")).unwrap())
            .unwrap();
    let moved = altered(
        &official,
        json!({"typeUri": "https://elsewhere.example/spec/acl/grant/0.2"}),
    );
    std::fs::write(w.path("official/specs/acl/grant/0.2/bundle.json"), &moved).unwrap();
    for e in index["specs"].as_array_mut().unwrap() {
        if e["slug"] == "acl/grant" {
            e["typeUri"] = json!("https://elsewhere.example/spec/acl/grant/0.2");
            e["digest"] = json!(digest_of(&moved));
        }
    }
    std::fs::write(
        w.path("official/specs/index.json"),
        serde_json::to_vec(&index).unwrap(),
    )
    .unwrap();
    let e = err("[tasks]\n", &["add", "acl/grant/0.2"]);
    assert!(
        e.contains("which is not under https://trusttasks.org/spec/"),
        "{e}"
    );
}

// ─── https ─────────────────────────────────────────────────────────────────

/// A CA, and a certificate it issued for 127.0.0.1.
fn test_pki() -> (String, Vec<u8>, Vec<u8>) {
    use rcgen::{BasicConstraints, CertificateParams, IsCa, Issuer, KeyPair, KeyUsagePurpose};
    let ca_key = KeyPair::generate().unwrap();
    let mut ca_params = CertificateParams::new(Vec::<String>::new()).unwrap();
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    ca_params.key_usages = vec![
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::DigitalSignature,
    ];
    let ca_cert = ca_params.self_signed(&ca_key).unwrap();
    let issuer = Issuer::new(ca_params, ca_key);
    let leaf_key = KeyPair::generate().unwrap();
    let leaf = CertificateParams::new(vec!["127.0.0.1".to_string()])
        .unwrap()
        .signed_by(&leaf_key, &issuer)
        .unwrap();
    (ca_cert.pem(), leaf.der().to_vec(), leaf_key.serialize_der())
}

/// Serves the files under `root` over TLS on 127.0.0.1, one request per
/// connection, until the test process exits. Returns the port.
fn serve_https(root: PathBuf, cert: Vec<u8>, key: Vec<u8>) -> u16 {
    use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
    let config = Arc::new(
        rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(
            vec![CertificateDer::from(cert)],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key)),
        )
        .unwrap(),
    );
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else { continue };
            let config = config.clone();
            let root = root.clone();
            std::thread::spawn(move || {
                let Ok(conn) = rustls::ServerConnection::new(config) else {
                    return;
                };
                let mut tls = rustls::StreamOwned::new(conn, stream);
                let mut reader = BufReader::new(&mut tls);
                let mut request_line = String::new();
                if reader.read_line(&mut request_line).is_err() {
                    return; // a client that refused the certificate
                }
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).is_err() || line == "\r\n" || line.is_empty() {
                        break;
                    }
                }
                let path = request_line.split_whitespace().nth(1).unwrap_or("/");
                let body = (!path.contains(".."))
                    .then(|| std::fs::read(root.join(path.trim_start_matches('/'))).ok())
                    .flatten();
                let head = match &body {
                    Some(b) => format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        b.len()
                    ),
                    None => "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string(),
                };
                let _ = tls.write_all(head.as_bytes());
                if let Some(b) = body {
                    let _ = tls.write_all(&b);
                }
                tls.conn.send_close_notify();
                let _ = tls.flush();
            });
        }
    });
    port
}

#[test]
fn an_https_source_is_fetched_and_verified() {
    let w = World::new();
    let (ca_pem, cert, key) = test_pki();
    let port = serve_https(w.path("acme-site"), cert, key);
    let ca = w.path("ca.pem");
    std::fs::write(&ca, ca_pem).unwrap();
    let ca = ca.to_str().unwrap().to_string();
    w.manifest(&format!(
        "[sources.acme]\nurl = \"https://127.0.0.1:{port}\"\n\n[tasks]\n"
    ));

    // An untrusted certificate is refused: TLS is really verified.
    let r = w.run(&["add", "acme:payments/settle/1.0"]).fails();
    assert!(
        r.stderr
            .contains(&format!("https://127.0.0.1:{port}/specs/index.json")),
        "{}",
        r.stderr
    );

    let env = [("TRUST_TASKS_CA_FILE", ca.as_str())];
    let r = w.run_env(&["add", "acme:payments/settle/1.0"], &env).ok();
    assert!(
        r.stdout.contains("Added acme:payments/settle 1.0"),
        "{}",
        r.stdout
    );
    let lock = w.read("app/trust-tasks.lock");
    assert!(
        lock.contains(&format!(
            "[sources.acme]\nurl = \"https://127.0.0.1:{port}\"\n"
        )),
        "{lock}"
    );
    assert!(
        lock.contains("type_uri = \"https://tasks.acme.example/spec/payments/settle/1.0\""),
        "{lock}"
    );
    // `check` needs no network at all.
    w.run(&["check"]).ok();

    // `fetch` restores by digest over https.
    std::fs::remove_file(w.path("app/trust-tasks/@acme/payments/settle/1.0.bundle.json")).unwrap();
    let r = w.run_env(&["fetch"], &env).ok();
    assert!(
        r.stdout.contains("Fetched acme:payments/settle 1.0"),
        "{}",
        r.stdout
    );
    let r = w.run_env(&["list", "--remote"], &env).ok();
    assert!(r.stdout.contains("acme:payments/settle"), "{}", r.stdout);
    w.run(&["check"]).ok();

    // The same bytes as from the directory.
    let by_path: BTreeMap<_, _> = [("p", w.acme_bundle("payments/settle/1.0"))].into();
    assert_eq!(
        std::fs::read(w.path("app/trust-tasks/@acme/payments/settle/1.0.bundle.json")).unwrap(),
        by_path["p"]
    );
}
