// check-doc-facts.mjs: passes on the tree, and fails on each fact it guards
// when the prose and the tree disagree (#1123), in a copy of the files it reads.

import { test } from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const SCRIPT = path.join(ROOT, "scripts/check-doc-facts.mjs");
const run = (root) =>
  spawnSync("node", [SCRIPT], { env: { ...process.env, ...(root ? { DOC_FACTS_ROOT: root } : {}) }, encoding: "utf8" });

/** A copy of exactly the files the check reads. */
function copyTree() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "doc-facts-"));
  const copy = (rel) => {
    fs.mkdirSync(path.dirname(path.join(dir, rel)), { recursive: true });
    fs.copyFileSync(path.join(ROOT, rel), path.join(dir, rel));
  };
  const members = [...fs.readFileSync(path.join(ROOT, "Cargo.toml"), "utf8").match(/^members\s*=\s*\[([\s\S]*?)\]/m)[1].matchAll(/"([^"]+)"/g)].map((m) => m[1]);
  for (const m of members) copy(`${m}/Cargo.toml`);
  for (const d of fs.readdirSync(ROOT).filter((d) => /^trust-tasks-ts(-|$)/.test(d))) {
    for (const f of ["package.json", "README.md"]) if (fs.existsSync(path.join(ROOT, d, f))) copy(`${d}/${f}`);
  }
  for (const f of [
    "Cargo.toml",
    "RELEASING.md",
    "scripts/release-pr.sh",
    "docs/DEVELOPMENT.md",
    ".nvmrc",
    "package.json",
    "README.md",
    "CONTRIBUTING.md",
    "scripts/lib/spec-lifecycle.mjs",
    "CLAUDE.md",
    "CONTRIBUTING-SPECS.md",
  ]) copy(f);
  return dir;
}
function edit(dir, rel, from, to) {
  const file = path.join(dir, rel);
  const text = fs.readFileSync(file, "utf8");
  const next = text.replace(from, to);
  assert.notEqual(next, text, `fixture edit did not apply to ${rel}: ${from}`);
  fs.writeFileSync(file, next);
}
function fails(dir, pattern) {
  const r = run(dir);
  assert.equal(r.status, 1, `expected a failure:\n${r.stdout}${r.stderr}`);
  assert.match(r.stderr, pattern);
}

test("the tree passes", () => {
  const r = run();
  assert.equal(r.status, 0, r.stdout + r.stderr);
  const copy = run(copyTree());
  assert.equal(copy.status, 0, copy.stdout + copy.stderr);
});

test("a published package missing from RELEASING.md's table fails", () => {
  const dir = copyTree();
  edit(dir, "RELEASING.md", /^\| `trust_tasks_capability_client` \|.*\n/m, "");
  fails(dir, /"What gets published" table does not name trust_tasks_capability_client/);
});

test("a row for a package nothing publishes fails, unless it says it is not released yet", () => {
  const dir = copyTree();
  edit(dir, "RELEASING.md", /^(\| `trust_tasks` \|)/m, "| `trust_tasks_gone` | pub.dev |\n$1");
  fails(dir, /names trust_tasks_gone, which is not in the tree/);
  const ok = copyTree();
  edit(ok, "RELEASING.md", /^(\| `trust_tasks` \|)/m, "| `trust_tasks_soon` | pub.dev — **Not released yet** |\n$1");
  assert.equal(run(ok).status, 0);
});

test("a crate that starts depending on trust-tasks-rs leaves DEVELOPMENT.md's list wrong", () => {
  const dir = copyTree();
  edit(dir, "trust-tasks-ceremony/Cargo.toml", /^\[dependencies\]\n/m, '[dependencies]\ntrust-tasks-rs = { path = "../trust-tasks-rs", version = "0.1" }\n');
  fails(dir, /names trust-tasks-ceremony, which is not in the tree/);
});

test("a crate missing from the layout table fails", () => {
  const dir = copyTree();
  edit(dir, "docs/DEVELOPMENT.md", "`-tsp`, ", "");
  fails(dir, /layout table does not name trust-tasks-tsp/);
});

test("a Node floor that disagrees with engines fails", () => {
  const dir = copyTree();
  edit(dir, "README.md", /Requires Node \d+ or later/, "Requires Node 20 or later");
  fails(dir, /README\.md says Node 20; package\.json's engines floor is Node/);
  const ts = copyTree();
  edit(ts, "trust-tasks-ts/README.md", /Requires \*\*Node\.js [\d.]+ or later\*\*/, "Requires **Node.js 18.0 or later**");
  fails(ts, /trust-tasks-ts\/README\.md says Node\.js 18\.0 or later/);
});

test("a stated framework version that is not CURRENT_FRAMEWORK_VERSION fails", () => {
  const dir = copyTree();
  edit(dir, "CLAUDE.md", /\(`CURRENT_FRAMEWORK_VERSION`, `[^`]+`\)/, "(`CURRENT_FRAMEWORK_VERSION`, `0.1.0`)");
  fails(dir, /CLAUDE\.md says the current framework version is 0\.1\.0/);
});

test("a fact reworded out of the check's reach fails rather than passing silently", () => {
  const dir = copyTree();
  edit(dir, "CONTRIBUTING.md", /\(Node \d+, see \.nvmrc\)/, "(see .nvmrc)");
  fails(dir, /CONTRIBUTING\.md: could not find the Node floor/);
});
