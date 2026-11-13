// A Release PR that takes the Dart core across a 0.x minor must raise every
// sibling's `trust_tasks` floor to admit it, and release the sibling for it —
// otherwise check-sibling-core-ranges fails the Release PR (#1085), and a
// sibling whose floor moved but was not released never reaches consumers.
//
// These tests drive scripts/raise-dart-core-floor.mjs directly, and
// scripts/release-bump-dart.sh on a throwaway repository with tags, the way
// release-pr.sh calls it.

import { test } from "node:test";
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

const GIT_ENV = {
  ...process.env,
  GIT_CONFIG_GLOBAL: "/dev/null",
  GIT_CONFIG_NOSYSTEM: "1",
  GIT_AUTHOR_NAME: "Fixture",
  GIT_AUTHOR_EMAIL: "fixture@example.invalid",
  GIT_COMMITTER_NAME: "Fixture",
  GIT_COMMITTER_EMAIL: "fixture@example.invalid",
};

const git = (cwd, ...args) => execFileSync("git", args, { cwd, env: GIT_ENV, encoding: "utf8" });

function write(file, content) {
  fs.mkdirSync(path.dirname(file), { recursive: true });
  fs.writeFileSync(file, content);
}

const pubspec = (name, version, trustTasks) =>
  [
    `name: ${name}`,
    `version: ${version}`,
    "",
    "dependencies:",
    ...(trustTasks ? [`  trust_tasks: ${trustTasks}`] : []),
    '  ssi: ">=3.9.6 <5.0.0"',
    "",
    "dev_dependencies:",
    "  test: ^1.25.0",
    "",
  ].join("\n");

const raise = (file, core) =>
  spawnSync("node", [path.join(ROOT, "scripts/raise-dart-core-floor.mjs"), file, core], { encoding: "utf8" });

function tmpFile(content) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "dart-floor-"));
  const file = path.join(dir, "pubspec.yaml");
  fs.writeFileSync(file, content);
  return file;
}

test("a range that excludes the core is raised to it, and only that line changes", () => {
  const before = pubspec("trust_tasks_proof", "0.1.8", "^0.9.3");
  const file = tmpFile(before);
  const r = raise(file, "0.10.0");
  assert.equal(r.status, 0, r.stderr);
  assert.equal(r.stdout.trim(), "^0.9.3");
  assert.equal(fs.readFileSync(file, "utf8"), before.replace("trust_tasks: ^0.9.3", "trust_tasks: ^0.10.0"));
});

test("a range that admits the core is left alone", () => {
  for (const core of ["0.9.3", "0.9.37"]) {
    const before = pubspec("trust_tasks_proof", "0.1.8", "^0.9.3");
    const file = tmpFile(before);
    const r = raise(file, core);
    assert.equal(r.status, 0, r.stderr);
    assert.equal(r.stdout, "");
    assert.equal(fs.readFileSync(file, "utf8"), before);
  }
});

test("a floor above the core is never lowered", () => {
  const before = pubspec("trust_tasks_proof", "0.1.8", "^0.10.0");
  const file = tmpFile(before);
  const r = raise(file, "0.9.37");
  assert.equal(r.status, 0, r.stderr);
  assert.equal(r.stdout, "");
  assert.equal(fs.readFileSync(file, "utf8"), before);
});

test("a quoted range is read, and a package without the core is skipped", () => {
  const quoted = tmpFile(pubspec("x", "0.1.0", '"^0.9.3"'));
  assert.equal(raise(quoted, "0.10.0").stdout.trim(), "^0.9.3");
  assert.match(fs.readFileSync(quoted, "utf8"), /^ {2}trust_tasks: \^0\.10\.0$/m);

  const none = pubspec("x", "0.1.0", null);
  const file = tmpFile(none);
  const r = raise(file, "0.10.0");
  assert.equal(r.status, 0, r.stderr);
  assert.equal(fs.readFileSync(file, "utf8"), none);
});

test("a range that is not a caret is an error", () => {
  const r = raise(tmpFile(pubspec("x", "0.1.0", "any")), "0.10.0");
  assert.notEqual(r.status, 0);
  assert.match(r.stderr, /not a \^MAJOR\.MINOR\.PATCH range/);
});

// A repository with the core and two siblings, each tagged at its current
// version, then one breaking commit to the core alone.
function repoFixture() {
  const repo = fs.mkdtempSync(path.join(os.tmpdir(), "release-bump-dart-"));
  git(repo, "init", "-q", "-b", "main");
  fs.mkdirSync(path.join(repo, "scripts"));
  for (const script of ["release-bump-dart.sh", "raise-dart-core-floor.mjs"]) {
    fs.copyFileSync(path.join(ROOT, "scripts", script), path.join(repo, "scripts", script));
  }
  fs.copyFileSync(path.join(ROOT, "cliff.toml"), path.join(repo, "cliff.toml"));
  write(path.join(repo, "trust-tasks-dart/pubspec.yaml"), pubspec("trust_tasks", "0.9.37", null));
  write(
    path.join(repo, "trust-tasks-dart/lib/src/runtime/version.dart"),
    "/// The package version.\nconst String packageVersion = '0.9.37';\n",
  );
  write(path.join(repo, "trust-tasks-dart/CHANGELOG.md"), "# Changelog — `trust_tasks`\n");
  write(path.join(repo, "trust-tasks-dart-proof/pubspec.yaml"), pubspec("trust_tasks_proof", "0.1.8", "^0.9.3"));
  write(path.join(repo, "trust-tasks-dart-proof/CHANGELOG.md"), "# Changelog — `trust_tasks_proof`\n\n## 0.1.8 — 2026-11-10\n\n- Earlier.\n");
  write(path.join(repo, "trust-tasks-dart-https/pubspec.yaml"), pubspec("trust_tasks_https", "0.1.9", "^0.10.0"));
  write(path.join(repo, "trust-tasks-dart-https/CHANGELOG.md"), "# Changelog — `trust_tasks_https`\n");
  git(repo, "add", "-A");
  git(repo, "commit", "-q", "-m", "chore: base");
  git(repo, "tag", "trust-tasks-dart-v0.9.37");
  git(repo, "tag", "trust-tasks-dart-proof-v0.1.8");
  git(repo, "tag", "trust-tasks-dart-https-v0.1.9");
  write(path.join(repo, "trust-tasks-dart/lib/a.dart"), "// changed\n");
  git(repo, "add", "-A");
  git(repo, "commit", "-q", "-m", "feat(dart)!: a break in the core");
  return repo;
}

function bump(repo, pkg, core) {
  const summary = path.join(repo, ".summary");
  const env = { ...GIT_ENV, RELEASE_SUMMARY: summary };
  if (core) env.TRUST_TASKS_CORE = core;
  const r = spawnSync("bash", ["scripts/release-bump-dart.sh", pkg], { cwd: repo, env, encoding: "utf8" });
  assert.equal(r.status, 0, r.stderr + r.stdout);
  return fs.existsSync(summary) ? fs.readFileSync(summary, "utf8") : "";
}

test("a sibling with no changes of its own is released for a raised floor", () => {
  const repo = repoFixture();
  assert.match(bump(repo, "trust_tasks"), /`0\.9\.37` → `0\.10\.0` \| breaking/);
  const summary = bump(repo, "trust_tasks_proof", "0.10.0");
  assert.match(summary, /trust_tasks_proof` \| pub\.dev \| `0\.1\.8` → `0\.1\.9` \| patch/);
  const spec = fs.readFileSync(path.join(repo, "trust-tasks-dart-proof/pubspec.yaml"), "utf8");
  assert.match(spec, /^version: 0\.1\.9$/m);
  assert.match(spec, /^ {2}trust_tasks: \^0\.10\.0$/m);
  const log = fs.readFileSync(path.join(repo, "trust-tasks-dart-proof/CHANGELOG.md"), "utf8");
  const top = log.indexOf("## 0.1.9 — ");
  assert.ok(top > 0, log);
  assert.ok(top < log.indexOf("## 0.1.8"), "the new section goes above the last release");
  assert.match(log, /### Dependencies\n\n- Requires `trust_tasks` \^0\.10\.0 \(was \^0\.9\.3\)/);
});

test("a sibling that already admits the core and has no changes is not released", () => {
  const repo = repoFixture();
  bump(repo, "trust_tasks");
  const before = fs.readFileSync(path.join(repo, "trust-tasks-dart-https/pubspec.yaml"), "utf8");
  assert.doesNotMatch(bump(repo, "trust_tasks_https", "0.10.0"), /trust_tasks_https/);
  assert.equal(fs.readFileSync(path.join(repo, "trust-tasks-dart-https/pubspec.yaml"), "utf8"), before);
});
