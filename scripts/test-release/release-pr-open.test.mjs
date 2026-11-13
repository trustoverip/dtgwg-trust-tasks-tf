// `release-pr.sh open`, `check-patch-paths.sh` and `check-npm-tarball.sh`
// against changes a release or a withdrawal batch never makes.
//
// `open` runs with the release token, after a read-only job computed the patch.
// These tests build a throwaway repository with a local bare remote and a stub
// `gh`, hand `open` a patch, and check that a patch outside the allowlist is
// refused before anything is applied, committed or pushed — and that an
// ordinary release patch still goes through.

import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

// Isolated from the machine's git configuration (signing, hooks, identity).
const GIT_ENV = {
  ...process.env,
  GIT_CONFIG_GLOBAL: "/dev/null",
  GIT_CONFIG_NOSYSTEM: "1",
  GIT_AUTHOR_NAME: "Fixture",
  GIT_AUTHOR_EMAIL: "fixture@example.invalid",
  GIT_COMMITTER_NAME: "Fixture",
  GIT_COMMITTER_EMAIL: "fixture@example.invalid",
};
delete GIT_ENV.PUSH_TOKEN;
delete GIT_ENV.GH_TOKEN;

const OPEN_ENV = Object.fromEntries(
  Object.entries(GIT_ENV).filter(([k]) => !/^GIT_(AUTHOR|COMMITTER)_/.test(k)),
);

const git = (cwd, ...args) => execFileSync("git", args, { cwd, env: GIT_ENV, encoding: "utf8" });

function write(file, content) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, content);
}

// The manifests a release edits, as the fixture starts and as a test changes them.
const packageJson = (over = {}) =>
  JSON.stringify(
    {
      name: "x",
      version: "0.1.0",
      peerDependencies: { "@openvtc/trust-tasks": "^0.28.0" },
      devDependencies: { typescript: "^5.6.0" },
      ...over,
    },
    null,
    2,
  ) + "\n";
const goVersion = (v, extra = "") => `// Package proof.\npackage proof\n\nconst Version = "${v}"\n${extra}`;
const pubspec = ({ version = "0.1.8", trustTasks = "^0.9.3", description = "Proofs." } = {}) =>
  [
    "name: trust_tasks_proof",
    `description: ${description}`,
    `version: ${version}`,
    "",
    "environment:",
    "  sdk: ^3.6.0",
    "",
    "dependencies:",
    `  trust_tasks: ${trustTasks}`,
    '  ssi: ">=3.9.6 <5.0.0"',
    "",
  ].join("\n");
const dartVersion = (v) => `/// The package version.\nconst String packageVersion = '${v}';\n`;

// A repository at one commit holding the two scripts under test and a small
// package tree, a bare remote, and a `gh` that answers `api user` and
// `pr list` and records every call.
function fixture() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "release-pr-open-"));
  const repo = path.join(dir, "repo");
  const remote = path.join(dir, "remote.git");
  const bin = path.join(dir, "bin");
  const out = path.join(dir, "out");
  fs.mkdirSync(out);
  git(dir, "init", "-q", "--bare", remote);
  git(dir, "init", "-q", "-b", "main", repo);
  fs.mkdirSync(path.join(repo, "scripts"));
  for (const script of ["release-pr.sh", "check-patch-paths.sh", "check-npm-tarball.sh"]) {
    fs.copyFileSync(path.join(ROOT, "scripts", script), path.join(repo, "scripts", script));
    fs.chmodSync(path.join(repo, "scripts", script), 0o755);
  }
  write(path.join(repo, "trust-tasks-ts/package.json"), packageJson());
  write(path.join(repo, "trust-tasks-go/proof/version.go"), goVersion("0.1.4"));
  write(path.join(repo, "trust-tasks-dart-proof/pubspec.yaml"), pubspec());
  write(path.join(repo, "trust-tasks-dart/lib/src/runtime/version.dart"), dartVersion("0.9.37"));
  write(path.join(repo, "trust-tasks-ts/CHANGELOG.md"), "# Changelog\n");
  write(path.join(repo, "specs/a/0.1/spec.md"), "a\n");
  write(path.join(repo, "scripts/other.sh"), "#!/bin/sh\necho hi\n");
  git(repo, "add", "-A");
  git(repo, "commit", "-q", "-m", "base");
  git(repo, "remote", "add", "origin", remote);
  write(
    path.join(bin, "gh"),
    [
      "#!/usr/bin/env bash",
      'echo "gh $*" >> "$GH_LOG"',
      'case "$1 $2" in',
      '  "api user") echo \'{"login":"releaser","id":1,"name":"Releaser"}\' ;;',
      '  "pr list") echo "" ;;',
      "esac",
      "",
    ].join("\n"),
  );
  fs.chmodSync(path.join(bin, "gh"), 0o755);
  return { dir, repo, remote, bin, out };
}

// Make a change in the working tree, capture it as the patch `prepare` would
// write, and put the tree back.
function patchOf(f, change, summaryRow = "| `x` | npm | `0.1.0` → `0.1.1` | patch |\n") {
  change(f.repo);
  git(f.repo, "add", "-A");
  fs.writeFileSync(path.join(f.out, "changes.patch"), git(f.repo, "diff", "--cached", "--binary"));
  fs.writeFileSync(path.join(f.out, "base"), git(f.repo, "rev-parse", "HEAD"));
  fs.writeFileSync(path.join(f.out, "summary.md"), summaryRow);
  git(f.repo, "reset", "-q", "--hard");
  git(f.repo, "clean", "-fdq");
}

function open(f) {
  const ghLog = path.join(f.dir, "gh.log");
  const res = spawnSync("./scripts/release-pr.sh", ["open", f.out], {
    cwd: f.repo,
    // No GIT_AUTHOR_*/GIT_COMMITTER_*: `open` sets the identity from `gh api user`.
    env: { ...OPEN_ENV, PATH: `${f.bin}${path.delimiter}${process.env.PATH}`, GH_LOG: ghLog },
    encoding: "utf8",
  });
  const gh = fs.existsSync(ghLog) ? fs.readFileSync(ghLog, "utf8") : "";
  const pushed = git(f.dir, "--git-dir", f.remote, "branch", "--list", "release").trim() !== "";
  return { status: res.status, stderr: res.stderr, gh, pushed };
}

function refused(f, pattern) {
  const r = open(f);
  assert.notEqual(r.status, 0, "open should refuse the patch");
  assert.match(r.stderr, pattern);
  assert.equal(r.pushed, false, "nothing may be pushed");
  assert.equal(r.gh, "", "the token must not have been used");
  assert.equal(git(f.repo, "status", "--porcelain"), "", "nothing may be applied");
}

test("open refuses a patch that changes a script", () => {
  const f = fixture();
  patchOf(f, (repo) => write(path.join(repo, "scripts/other.sh"), "#!/bin/sh\necho changed\n"));
  refused(f, /scripts\/other\.sh is never part of this change/);
});

test("open refuses a patch that changes release-pr.sh itself", () => {
  const f = fixture();
  patchOf(f, (repo) => fs.appendFileSync(path.join(repo, "scripts/release-pr.sh"), "\n# appended\n"));
  refused(f, /scripts\/release-pr\.sh is never part of this change/);
});

test("open refuses a patch that adds a workflow", () => {
  const f = fixture();
  patchOf(f, (repo) => write(path.join(repo, ".github/workflows/extra.yml"), "on: push\n"));
  refused(f, /\.github\/workflows\/extra\.yml is never part of this change/);
});

test("open refuses a patch that adds .gitattributes", () => {
  const f = fixture();
  patchOf(f, (repo) => write(path.join(repo, "trust-tasks-ts/.gitattributes"), "* filter=x\n"));
  refused(f, /\.gitattributes is never part of this change/);
});

test("open refuses a patch that turns an allowed file into a symlink", () => {
  const f = fixture();
  patchOf(f, (repo) => {
    const file = path.join(repo, "trust-tasks-ts/CHANGELOG.md");
    fs.rmSync(file);
    fs.symlinkSync("../scripts/other.sh", file);
  });
  refused(f, /symlink/);
});

test("open refuses a patch that touches a file outside the release allowlist", () => {
  const f = fixture();
  patchOf(f, (repo) => write(path.join(repo, "specs/a/0.1/spec.md"), "b\n"));
  refused(f, /specs\/a\/0\.1\/spec\.md is outside the release allowlist/);
});

test("open applies, commits and pushes an ordinary release patch", () => {
  const f = fixture();
  patchOf(f, (repo) => {
    write(path.join(repo, "trust-tasks-ts/package.json"), packageJson({ version: "0.1.1" }));
    fs.appendFileSync(path.join(repo, "trust-tasks-ts/CHANGELOG.md"), "\n## 0.1.1\n");
  });
  const r = open(f);
  assert.equal(r.status, 0, r.stderr);
  assert.equal(r.pushed, true);
  assert.match(r.gh, /pr create/);
  const subject = git(f.dir, "--git-dir", f.remote, "log", "-1", "--format=%s%n%(trailers:key=Signed-off-by)", "release");
  assert.match(subject, /^chore: release x 0\.1\.1\nSigned-off-by: Releaser/);
});

test("the withdraw allowlist takes spec folders and generated trees only", () => {
  const f = fixture();
  const check = (kind) =>
    spawnSync("./scripts/check-patch-paths.sh", [kind, path.join(f.out, "changes.patch")], {
      cwd: f.repo,
      env: GIT_ENV,
      encoding: "utf8",
    });

  patchOf(f, (repo) => {
    write(path.join(repo, "specs/a/0.1/spec.md"), "withdrawn\n");
    write(path.join(repo, "trust-tasks-go/specs/a/v0_1/payload.go"), "package v0_1\n");
  });
  assert.equal(check("withdraw").status, 0);

  patchOf(f, (repo) => write(path.join(repo, "trust-tasks-ts/package.json"), "{}\n"));
  const r = check("withdraw");
  assert.notEqual(r.status, 0);
  assert.match(r.stderr, /trust-tasks-ts\/package\.json is outside the withdraw allowlist/);
});

// ── What a release may change inside a manifest ─────────────────────────────

// Refused after the patch applied (so after `gh api user` / `pr list`), but
// before the commit: nothing pushed, no PR opened.
function refusedContent(f, pattern) {
  const r = open(f);
  assert.notEqual(r.status, 0, "open should refuse the change");
  assert.match(r.stderr, pattern);
  assert.equal(r.pushed, false, "nothing may be pushed");
  assert.doesNotMatch(r.gh, /pr (create|close)|api .*PATCH/, "no PR may be opened or changed");
}

test("open accepts version and dependency-version changes in every manifest", () => {
  const f = fixture();
  patchOf(f, (repo) => {
    write(
      path.join(repo, "trust-tasks-ts/package.json"),
      packageJson({ version: "0.2.0", peerDependencies: { "@openvtc/trust-tasks": "^0.29.0" } }),
    );
    write(path.join(repo, "trust-tasks-go/proof/version.go"), goVersion("0.2.0"));
    write(path.join(repo, "trust-tasks-dart-proof/pubspec.yaml"), pubspec({ version: "0.2.0", trustTasks: "^0.10.0" }));
    write(path.join(repo, "trust-tasks-dart/lib/src/runtime/version.dart"), dartVersion("0.10.0"));
  });
  const r = open(f);
  assert.equal(r.status, 0, r.stderr);
  assert.equal(r.pushed, true);
});

test("open refuses a package.json that gains a script", () => {
  const f = fixture();
  patchOf(f, (repo) =>
    write(path.join(repo, "trust-tasks-ts/package.json"), packageJson({ version: "0.1.1", scripts: { postinstall: "node x" } })),
  );
  refusedContent(f, /package\.json changes more than its version and dependency versions/);
});

test("open refuses a package.json dependency moved off a version range", () => {
  const f = fixture();
  patchOf(f, (repo) =>
    write(
      path.join(repo, "trust-tasks-ts/package.json"),
      packageJson({ version: "0.1.1", devDependencies: { typescript: "git+https://example.invalid/ts.git" } }),
    ),
  );
  refusedContent(f, /package\.json changes more than its version and dependency versions/);
});

test("open refuses a version.go that changes more than const Version", () => {
  const f = fixture();
  patchOf(f, (repo) =>
    write(path.join(repo, "trust-tasks-go/proof/version.go"), goVersion("0.1.5", "\nfunc init() {}\n")),
  );
  refusedContent(f, /version\.go changes more than `const Version`/);
});

test("open refuses a version.dart that changes more than packageVersion", () => {
  const f = fixture();
  patchOf(f, (repo) =>
    write(path.join(repo, "trust-tasks-dart/lib/src/runtime/version.dart"), dartVersion("0.9.38") + "final x = 1;\n"),
  );
  refusedContent(f, /version\.dart changes more than `packageVersion`/);
});

test("open refuses a pubspec change outside version: and constraints", () => {
  const f = fixture();
  patchOf(f, (repo) =>
    write(path.join(repo, "trust-tasks-dart-proof/pubspec.yaml"), pubspec({ version: "0.1.9", description: "Other." })),
  );
  refusedContent(f, /only `version:` may change at the top level/);
});

test("open refuses a pubspec dependency moved off a version constraint", () => {
  const f = fixture();
  patchOf(f, (repo) =>
    write(path.join(repo, "trust-tasks-dart-proof/pubspec.yaml"), pubspec({ trustTasks: "any" })),
  );
  refusedContent(f, /not a version constraint/);
});

test("open refuses a pubspec dependency turned into a git source", () => {
  const f = fixture();
  patchOf(f, (repo) => {
    const file = path.join(repo, "trust-tasks-dart-proof/pubspec.yaml");
    write(file, pubspec().replace("  trust_tasks: ^0.9.3\n", "  trust_tasks:\n    git: https://example.invalid/x.git\n"));
  });
  refusedContent(f, /adds or removes lines|not a version/);
});

// ── check-patch-paths.sh: names, modes and unreadable patches ───────────────

function checkPaths(f, kind = "release") {
  return spawnSync("./scripts/check-patch-paths.sh", [kind, path.join(f.out, "changes.patch")], {
    cwd: f.repo,
    env: GIT_ENV,
    encoding: "utf8",
  });
}

test("a path with a control character is refused even when the allowlist matches it", () => {
  const f = fixture();
  patchOf(f, (repo) => write(path.join(repo, "trust-tasks-go/a\tb/CHANGELOG.md"), "# Changelog\n"));
  const r = checkPaths(f);
  assert.notEqual(r.status, 0);
  assert.match(r.stderr, /control character/);
});

test("a new executable file is refused", () => {
  const f = fixture();
  patchOf(f, (repo) => {
    const file = path.join(repo, "trust-tasks-ts-gen/CHANGELOG.md");
    write(file, "# Changelog\n");
    fs.chmodSync(file, 0o755);
  });
  const r = checkPaths(f);
  assert.notEqual(r.status, 0);
  assert.match(r.stderr, /new executable file/);
});

test("a file made executable is refused", () => {
  const f = fixture();
  patchOf(f, (repo) => fs.chmodSync(path.join(repo, "trust-tasks-ts/CHANGELOG.md"), 0o755));
  const r = checkPaths(f);
  assert.notEqual(r.status, 0);
  assert.match(r.stderr, /made executable/);
});

test("a patch git cannot read is refused as unreadable", () => {
  const f = fixture();
  for (const body of ["not a patch\n", "diff --git a/x b/x\n--- a/x\n+++ b/x\n@@ -1,3 +1,3 @@\n-a\n+b\n"]) {
    fs.writeFileSync(path.join(f.out, "changes.patch"), body);
    const r = checkPaths(f);
    assert.notEqual(r.status, 0);
    assert.match(r.stderr, /git cannot read it/);
  }
});

// ── check-npm-tarball.sh ────────────────────────────────────────────────────

function tarball(f, manifest) {
  const dir = fs.mkdtempSync(path.join(f.dir, "pack-"));
  write(path.join(dir, "package/package.json"), JSON.stringify(manifest));
  const file = path.join(f.dir, `${path.basename(dir)}.tgz`);
  execFileSync("tar", ["-czf", file, "-C", dir, "package"]);
  return file;
}

function checkTarball(f, manifest, name = "x", version = "0.1.1") {
  return spawnSync("./scripts/check-npm-tarball.sh", [tarball(f, manifest), name, version], {
    cwd: f.repo,
    env: GIT_ENV,
    encoding: "utf8",
  });
}

test("the npm tarball check accepts the package decided on", () => {
  const f = fixture();
  const r = checkTarball(f, { name: "x", version: "0.1.1", scripts: { build: "tsc", test: "node --test" } });
  assert.equal(r.status, 0, r.stderr);
});

test("the npm tarball check refuses another name or version", () => {
  const f = fixture();
  assert.match(checkTarball(f, { name: "y", version: "0.1.1" }).stderr, /it is y@0\.1\.1, not x@0\.1\.1/);
  assert.match(checkTarball(f, { name: "x", version: "9.9.9" }).stderr, /it is x@9\.9\.9, not x@0\.1\.1/);
});

test("the npm tarball check refuses a publishConfig", () => {
  const f = fixture();
  const r = checkTarball(f, { name: "x", version: "0.1.1", publishConfig: { registry: "https://example.invalid" } });
  assert.notEqual(r.status, 0);
  assert.match(r.stderr, /publishConfig/);
});

for (const hook of ["preinstall", "install", "postinstall"]) {
  test(`the npm tarball check refuses a ${hook} script`, () => {
    const f = fixture();
    const r = checkTarball(f, { name: "x", version: "0.1.1", scripts: { [hook]: "node x" } });
    assert.notEqual(r.status, 0);
    assert.match(r.stderr, /install-time script/);
  });
}
