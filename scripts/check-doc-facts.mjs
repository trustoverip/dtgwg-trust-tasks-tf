#!/usr/bin/env node
// Checks the facts the documentation still states in prose against the files
// that decide them (#1123). #1224 replaced most hard-coded counts with "see the
// table"; what remains is written down because a reader needs it in the
// sentence, so it is checked here instead:
//
//   published   RELEASING.md's "What gets published" table names every package
//               the release tooling publishes — each workspace crate without
//               `publish = false`, and TS_PACKAGES, GO_MODULES and
//               DART_PACKAGES in scripts/release-pr.sh — and no other, except a
//               row that says it is "Not released yet".
//   rs-deps     docs/DEVELOPMENT.md's "workspace event" paragraph names the
//               published crates that do NOT depend on trust-tasks-rs; every
//               other published crate does (a `version` requirement in
//               [dependencies]), and is part of the leading-component bump.
//   layout      every workspace crate under trust-tasks* is named in
//               docs/DEVELOPMENT.md's layout table.
//   node        the Node floor: `.nvmrc`, the root `engines`, and the "Node N"
//               README.md, CONTRIBUTING.md and docs/DEVELOPMENT.md state; and
//               each TypeScript package's "Node.js X.Y or later" against its own
//               `engines`.
//   framework   the current framework version CLAUDE.md, docs/DEVELOPMENT.md and
//               CONTRIBUTING-SPECS.md state, against CURRENT_FRAMEWORK_VERSION
//               in scripts/lib/spec-lifecycle.mjs.
//
// Like the other guards it re-reads the manifests rather than importing a list
// from the release scripts, and reads Cargo.toml directly rather than running
// `cargo metadata`, so it runs in the Node-only checks job. A sentence it reads
// that is reworded so the check no longer finds it fails too: a check that
// cannot find its fact checks nothing.
//
//   npm run check-doc-facts            DOC_FACTS_ROOT=<dir> to check a copy

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = process.env.DOC_FACTS_ROOT
  ? path.resolve(process.env.DOC_FACTS_ROOT)
  : path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (p) => fs.readFileSync(path.join(ROOT, p), "utf8");
const exists = (p) => fs.existsSync(path.join(ROOT, p));
const errors = [];
const fail = (msg) => errors.push(msg);
const missing = (file, what) => fail(`${file}: could not find ${what} — reword the check with the prose, never drop it`);
const sameSet = (label, want, got) => {
  const extra = got.filter((x) => !want.includes(x));
  const lacking = want.filter((x) => !got.includes(x));
  if (extra.length) fail(`${label} names ${extra.join(", ")}, which ${extra.length === 1 ? "is" : "are"} not in the tree`);
  if (lacking.length) fail(`${label} does not name ${lacking.join(", ")}`);
};
const bashArray = (text, name) => {
  const m = text.match(new RegExp(`^${name}=\\(\\n([\\s\\S]*?)\\n\\)$`, "m"));
  return m ? [...m[1].matchAll(/"([^"]+)"/g)].map((x) => x[1]) : null;
};

// ── the workspace's crates ───────────────────────────────────────────────────
const members = [...(read("Cargo.toml").match(/^members\s*=\s*\[([\s\S]*?)\]/m)?.[1] ?? "").matchAll(/"([^"]+)"/g)].map((m) => m[1]);
if (!members.length) missing("Cargo.toml", "the [workspace] members");
const crates = members.map((dir) => {
  const toml = read(`${dir}/Cargo.toml`);
  const pkg = toml.match(/^\[package\]\n([\s\S]*?)(?=^\[|(?![\s\S]))/m)?.[1] ?? "";
  const deps = toml.match(/^\[dependencies\]\n([\s\S]*?)(?=^\[|(?![\s\S]))/m)?.[1] ?? "";
  return {
    dir,
    name: pkg.match(/^name\s*=\s*"([^"]+)"/m)?.[1],
    published: !/^publish\s*=\s*false/m.test(pkg),
    // A published dependency (with a `version`), not a path-only or dev one.
    onRs: /^trust-tasks-rs\s*=\s*\{[^}]*\bversion\s*=/m.test(deps),
  };
});
const published = crates.filter((c) => c.published);

// ── published: RELEASING.md's table ──────────────────────────────────────────
{
  const file = "RELEASING.md";
  const text = read(file);
  const section = text.match(/^## What gets published\n([\s\S]*?)(?=^## |(?![\s\S]))/m)?.[1];
  if (!section) missing(file, 'the "## What gets published" section');
  else {
    const rows = section.split("\n").filter((l) => /^\| `/.test(l));
    if (!rows.length) missing(file, "the package table");
    const named = [];
    const unreleased = [];
    for (const row of rows) {
      const [, first, rest] = row.match(/^\|([^|]*)\|(.*)$/);
      const names = [...first.matchAll(/`([^`]+)`/g)].map((m) => m[1]);
      named.push(...names);
      if (/not released yet/i.test(rest)) unreleased.push(...names);
    }
    const releasePr = read("scripts/release-pr.sh");
    const lists = {};
    for (const name of ["TS_PACKAGES", "GO_MODULES", "DART_PACKAGES"]) {
      lists[name] = bashArray(releasePr, name);
      if (!lists[name]) missing("scripts/release-pr.sh", `the ${name}=( … ) array`);
    }
    const want = [...published.map((c) => c.name), ...Object.values(lists).flatMap((l) => l ?? [])];
    sameSet(`${file}'s "What gets published" table`, want, named.filter((n) => !unreleased.includes(n)));
    for (const n of unreleased) if (want.includes(n)) fail(`${file} says ${n} is not released yet, but the release tooling publishes it`);
  }
}

// ── rs-deps: the crates outside the leading-component bump ──────────────────
{
  const file = "docs/DEVELOPMENT.md";
  const text = read(file).replace(/\s+/g, " ");
  const m = text.match(/\(((?:`[^`]+`(?:, | and )?)+) do not depend on `trust-tasks-rs`/);
  if (!m) missing(file, 'the "(… do not depend on `trust-tasks-rs` …)" sentence of "Bumping the leading component"');
  else {
    const named = [...m[1].matchAll(/`([^`]+)`/g)].map((x) => (x[1].startsWith("-") ? `trust-tasks${x[1]}` : x[1]));
    const want = published.filter((c) => !c.onRs && c.name !== "trust-tasks-rs").map((c) => c.name);
    sameSet(`${file}'s list of published crates that do not depend on trust-tasks-rs`, want, named);
  }
}

// ── layout: every crate in the layout table ─────────────────────────────────
{
  const file = "docs/DEVELOPMENT.md";
  const text = read(file);
  const table = text.match(/^\| Path \| What it holds \|\n([\s\S]*?)\n(?!\|)/m)?.[1];
  if (!table) missing(file, 'the "| Path | What it holds |" layout table');
  else {
    const named = new Set();
    for (const row of table.split("\n")) {
      const first = row.split("|")[1] ?? "";
      let base = null;
      for (const [, n] of first.matchAll(/`([^`]+)`/g)) {
        // `-didcomm` after `trust-tasks-https` abbreviates `trust-tasks-didcomm`.
        if (n.startsWith("-") && base) named.add(`${base}${n}`);
        else {
          named.add(n.replace(/\/$/, ""));
          base = n.match(/^(trust-tasks)(?:-|$)/)?.[1] ?? base;
        }
      }
    }
    const want = crates.filter((c) => c.dir.startsWith("trust-tasks")).map((c) => c.dir);
    const lacking = want.filter((d) => !named.has(d));
    if (lacking.length) fail(`${file}'s layout table does not name ${lacking.join(", ")}`);
  }
}

// ── node: the Node floors ───────────────────────────────────────────────────
{
  const nvmrc = read(".nvmrc").trim().replace(/^v/, "");
  const engines = JSON.parse(read("package.json")).engines?.node ?? "";
  const floor = engines.match(/^>=\s*(\d+)/)?.[1];
  if (!floor) missing("package.json", 'an `engines.node` ">=N" floor');
  if (floor && nvmrc.split(".")[0] !== floor) fail(`.nvmrc is ${nvmrc}, but package.json's engines floor is Node ${floor}`);
  for (const [file, re] of [
    ["README.md", /Requires Node (\d+) or later/],
    ["CONTRIBUTING.md", /\(Node (\d+), see \.nvmrc\)/],
    ["docs/DEVELOPMENT.md", /\(Node (\d+), as `\.nvmrc` and `engines` say\)/],
  ]) {
    const m = read(file).match(re);
    if (!m) missing(file, `the Node floor (${re.source})`);
    else if (floor && m[1] !== floor) fail(`${file} says Node ${m[1]}; package.json's engines floor is Node ${floor}`);
  }
  for (const dir of fs.readdirSync(ROOT).filter((d) => /^trust-tasks-ts(-|$)/.test(d)).sort()) {
    if (!exists(`${dir}/package.json`) || !exists(`${dir}/README.md`)) continue;
    const want = JSON.parse(read(`${dir}/package.json`)).engines?.node?.match(/^>=\s*(\d+\.\d+)\.0$/)?.[1];
    const said = read(`${dir}/README.md`).match(/Requires \*\*Node\.js (\d+\.\d+) or later\*\*/)?.[1];
    if (!want) missing(`${dir}/package.json`, 'an `engines.node` ">=X.Y.0" floor');
    else if (!said) missing(`${dir}/README.md`, 'its "Requires **Node.js X.Y or later**" line');
    else if (said !== want) fail(`${dir}/README.md says Node.js ${said} or later; its package.json engines floor is ${want}`);
  }
}

// ── framework: the current framework version ────────────────────────────────
{
  const current = read("scripts/lib/spec-lifecycle.mjs").match(/^export const CURRENT_FRAMEWORK_VERSION = '([^']+)'/m)?.[1];
  if (!current) missing("scripts/lib/spec-lifecycle.mjs", "CURRENT_FRAMEWORK_VERSION");
  for (const [file, re] of [
    ["CLAUDE.md", /\(`CURRENT_FRAMEWORK_VERSION`, `([^`]+)`\)/],
    ["docs/DEVELOPMENT.md", /`CURRENT_FRAMEWORK_VERSION` \(`scripts\/lib\/spec-lifecycle\.mjs`, `([^`]+)`\)/],
    ["CONTRIBUTING-SPECS.md", /current framework version is `([^`]+)`/],
    ["CONTRIBUTING-SPECS.md", /^targetFrameworkVersion: "([^"]+)"\s+# framework version this spec targets/m],
  ]) {
    const m = read(file).replace(/\n(?!\n)/g, " ").match(re) ?? read(file).match(re);
    if (!m) missing(file, `the current framework version (${re.source})`);
    else if (current && m[1] !== current) fail(`${file} says the current framework version is ${m[1]}; CURRENT_FRAMEWORK_VERSION is ${current}`);
  }
}

if (errors.length) {
  console.error(`check-doc-facts: ${errors.length} problem(s)\n`);
  for (const e of errors) console.error(`  - ${e}`);
  process.exit(1);
}
console.log(
  `check-doc-facts: RELEASING.md's table, the crates outside the trust-tasks-rs bump, the layout table, ` +
    `the Node floors and the current framework version agree with the tree.`
);
