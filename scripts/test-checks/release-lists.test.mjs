// check-release-lists.mjs: passes on the tree, and fails on each kind of drift
// it exists to catch (#1139, #1137), in a copy of the files it reads.

import { test } from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const SCRIPT = path.join(ROOT, "scripts/check-release-lists.mjs");
const run = (root) =>
  spawnSync("node", [SCRIPT], { env: { ...process.env, ...(root ? { RELEASE_LISTS_ROOT: root } : {}) }, encoding: "utf8" });

/** A copy of exactly the files the check reads. */
function copyTree() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "release-lists-"));
  const copy = (rel) => {
    if (!fs.existsSync(path.join(ROOT, rel))) return;
    fs.mkdirSync(path.dirname(path.join(dir, rel)), { recursive: true });
    fs.copyFileSync(path.join(ROOT, rel), path.join(dir, rel));
  };
  for (const f of [
    ".github/workflows/publish.yml",
    ".github/workflows/ts.yml",
    ".github/workflows/go.yml",
    ".github/workflows/rust.yml",
    ".github/workflows/commit-lint.yml",
    "scripts/release-pr.sh",
    "scripts/release-bump-ts.sh",
    "scripts/release-bump-go.sh",
    "scripts/release-bump-dart.sh",
    "scripts/check-go-api-compat.sh",
    "scripts/check-rs-api-compat.sh",
    "release-plz.toml",
    "Cargo.toml",
    "trust-tasks-go/trusttasks/version.go",
  ]) copy(f);
  const members = [...fs.readFileSync(path.join(ROOT, "Cargo.toml"), "utf8").match(/^members\s*=\s*\[([\s\S]*?)\]/m)[1].matchAll(/"([^"]+)"/g)].map((m) => m[1]);
  for (const m of members) {
    copy(`${m}/Cargo.toml`);
    copy(`${m}/src/lib.rs`);
  }
  for (const d of fs.readdirSync(ROOT)) if (/^trust-tasks-ts/.test(d)) copy(`${d}/package.json`);
  copy("trust-tasks-go/go.mod");
  for (const d of fs.readdirSync(path.join(ROOT, "trust-tasks-go"))) {
    copy(`trust-tasks-go/${d}/go.mod`);
    copy(`trust-tasks-go/${d}/version.go`);
  }
  return dir;
}
const edit = (dir, rel, from, to) => {
  const p = path.join(dir, rel);
  const text = fs.readFileSync(p, "utf8");
  assert.ok(text.includes(from), `${rel} no longer contains ${JSON.stringify(from)} — update this test`);
  fs.writeFileSync(p, text.replace(from, to));
};

test("the lists in the tree agree", () => {
  const r = run();
  assert.equal(r.status, 0, r.stdout + r.stderr);
});

test("the copied tree agrees too (the fixture is complete)", () => {
  const r = run(copyTree());
  assert.equal(r.status, 0, r.stdout + r.stderr);
});

for (const [label, rel, from, to, expect] of [
  ["an npm package missing from TS_PACKAGES", "scripts/release-pr.sh", '  "@openvtc/trust-tasks-gen"\n', "", /TS_PACKAGES/],
  ["a Go module missing from GO_MODULES", "scripts/release-pr.sh", '  "trust-tasks-go/didcomm"\n', "", /GO_MODULES/],
  ["a nested Go module missing from the core's exclude list", "scripts/check-go-api-compat.sh", ' ":(exclude)$PKG_DIR/tsp"', "", /check-go-api-compat\.sh WATCH/],
  ["a crate missing from the API check", "scripts/check-rs-api-compat.sh", '  "trust-tasks-tsp||changed"\n', "", /CRATES/],
  ["a commit-lint scope no script reads", ".github/workflows/commit-lint.yml", "|cli|", "|cli|runtime|", /commit-lint\.yml pkg=/],
  ["a bump-script scope commit-lint refuses", ".github/workflows/commit-lint.yml", "|ts-gen|", "|", /commit-lint\.yml pkg=/],
  ["a crate outside rust.yml's path filter", ".github/workflows/rust.yml", '      - "trust-tasks-tsp/**"\n', "", /rust\.yml/],
]) {
  test(`fails on ${label}`, () => {
    const dir = copyTree();
    edit(dir, rel, from, to);
    const r = run(dir);
    assert.equal(r.status, 1, r.stdout + r.stderr);
    assert.match(r.stderr, expect);
  });
}
