// check-version-edits.mjs: a pull request other than the Release PR may not
// move a package's version declaration (#1137).

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
const write = (dir, file, content) => {
  fs.mkdirSync(path.dirname(path.join(dir, file)), { recursive: true });
  fs.writeFileSync(path.join(dir, file), content);
};

const cargo = (version, dep = '"0.1"') =>
  `[package]\nname = "trust-tasks-x"\nversion = "${version}"\n\n[dependencies]\ntrust-tasks-rs = { path = "../trust-tasks-rs", version = ${dep} }\n`;

const FILES = {
  "Cargo.toml": '[workspace]\nmembers = [\n    "trust-tasks-x",\n]\n',
  "trust-tasks-x/Cargo.toml": cargo("0.1.0"),
  "trust-tasks-ts/package.json": JSON.stringify({ name: "@openvtc/trust-tasks", version: "0.1.0" }, null, 2) + "\n",
  "trust-tasks-go/trusttasks/version.go": 'package trusttasks\n\nconst Version = "0.1.0"\n',
  "trust-tasks-go/proof/version.go": 'package proof\n\nconst Version = "0.1.0"\n',
  "trust-tasks-dart/pubspec.yaml": "name: trust_tasks\nversion: 0.1.0\n",
  "trust-tasks-dart/lib/src/runtime/version.dart": "const String packageVersion = '0.1.0';\n",
  "examples/consumer-app-ts/package.json": JSON.stringify({ name: "example", version: "0.0.0" }) + "\n",
};

function repo() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "version-edits-"));
  git(dir, "init", "-q", "-b", "main");
  for (const [f, c] of Object.entries(FILES)) write(dir, f, c);
  git(dir, "add", "-A");
  git(dir, "commit", "-q", "-m", "base");
  git(dir, "switch", "-q", "-c", "pr");
  return dir;
}
const run = (dir) =>
  spawnSync("node", [path.join(ROOT, "scripts/check-version-edits.mjs")], {
    cwd: dir,
    env: { ...GIT_ENV, VERSION_BASE_REF: "main" },
    encoding: "utf8",
  });
const commit = (dir, edits) => {
  for (const [f, c] of Object.entries(edits)) write(dir, f, c);
  git(dir, "add", "-A");
  git(dir, "commit", "-q", "-m", "change");
};

for (const [label, file, content] of [
  ["a crate's [package] version", "trust-tasks-x/Cargo.toml", cargo("0.2.0")],
  ["an npm package's version", "trust-tasks-ts/package.json", JSON.stringify({ name: "@openvtc/trust-tasks", version: "0.2.0" }) + "\n"],
  ["the Go core's const Version", "trust-tasks-go/trusttasks/version.go", 'package trusttasks\n\nconst Version = "0.2.0"\n'],
  ["a nested Go module's const Version", "trust-tasks-go/proof/version.go", 'package proof\n\nconst Version = "0.1.1"\n'],
  ["a pubspec version", "trust-tasks-dart/pubspec.yaml", "name: trust_tasks\nversion: 0.1.1\n"],
  ["the Dart packageVersion", "trust-tasks-dart/lib/src/runtime/version.dart", "const String packageVersion = '0.1.1';\n"],
]) {
  test(`fails when ${label} changes`, () => {
    const dir = repo();
    commit(dir, { [file]: content });
    const r = run(dir);
    assert.equal(r.status, 1, r.stdout + r.stderr);
    assert.match(r.stderr, new RegExp(file.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")));
  });
}

test("passes when only a dependency requirement, a reformat or an unwatched manifest changes", () => {
  const dir = repo();
  commit(dir, {
    "trust-tasks-x/Cargo.toml": cargo("0.1.0", '"0.2"'),
    "trust-tasks-ts/package.json": JSON.stringify({ version: "0.1.0", name: "@openvtc/trust-tasks", peerDependencies: { a: "^1" } }) + "\n",
    "trust-tasks-dart/pubspec.yaml": "name: trust_tasks\nversion: '0.1.0'\ndependencies: {}\n",
    "examples/consumer-app-ts/package.json": JSON.stringify({ name: "example", version: "9.9.9" }) + "\n",
  });
  const r = run(dir);
  assert.equal(r.status, 0, r.stdout + r.stderr);
});

test("passes when a new package is added with its first version", () => {
  const dir = repo();
  commit(dir, {
    "trust-tasks-ts-new/package.json": JSON.stringify({ name: "@openvtc/trust-tasks-new", version: "0.1.0" }) + "\n",
    "trust-tasks-go/newmod/version.go": 'package newmod\n\nconst Version = "0.1.0"\n',
  });
  const r = run(dir);
  assert.equal(r.status, 0, r.stdout + r.stderr);
});
