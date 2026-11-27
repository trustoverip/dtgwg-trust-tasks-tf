#!/usr/bin/env node
// Check that the HCP demos' vendored bundles and locks match the registry
// bundles built from this tree's `specs/`.
//
//   node scripts/check-demo-bundles.mjs [<bundles-dir>]     (default: website/specs)
//
// Run after `npm run build-bundles`. `trust-tasks check` is offline: it checks
// a project's vendored bundles against its own lock, not against the registry.
// The hcp/* 0.1 drafts change in place, so a demo whose lock and bundles were
// never refreshed would stay green against stale schemas. For every
// `examples/hcp-demo*/` with a `trust-tasks.lock`, this checks:
//
//   1. the lock's `registry` is https://trusttasks.org (a refresh against a
//      local registry must reset it);
//   2. every `[[task]]` the lock pins is a bundle the registry build wrote, and
//      the lock's `digest` is `sha256:` of that bundle's bytes;
//   3. every vendored `trust-tasks/<slug>/<M.N>.bundle.json` is the registry
//      bundle byte for byte, and each is one the lock pins (a port may vendor
//      fewer than it pins: the Rust port vendors no framework bundles).
//
// To fix a failure, refresh the port: `trust-tasks update` in its directory with
// TRUST_TASKS_REGISTRY=<repo>/website, then reset the lock's `registry =` line
// (examples/hcp-demo-ts/SCENARIO.md, "Pinned tasks").

import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const REGISTRY = "https://trusttasks.org";

/** The `registry` line and `[[task]]` tables of a trust-tasks.lock (flat TOML, as `trust-tasks` writes it). */
export function parseLock(text) {
  const lock = { registry: undefined, tasks: [] };
  let current = null;
  for (const raw of text.split("\n")) {
    const line = raw.trim();
    if (line === "" || line.startsWith("#")) continue;
    if (line === "[[task]]") {
      current = {};
      lock.tasks.push(current);
      continue;
    }
    const m = /^([A-Za-z_]+)\s*=\s*"([^"]*)"$/.exec(line);
    if (m === null) continue;
    if (current === null) {
      if (m[1] === "registry") lock.registry = m[2];
    } else {
      current[m[1]] = m[2];
    }
  }
  return lock;
}

/** Every `*.bundle.json` under `dir`, as `<slug>/<M.N>` keys. */
function vendored(dir) {
  const out = [];
  const walk = (d) => {
    for (const e of fs.readdirSync(d, { withFileTypes: true })) {
      const p = path.join(d, e.name);
      if (e.isDirectory()) walk(p);
      else if (e.name.endsWith(".bundle.json")) out.push(path.relative(dir, p).split(path.sep).join("/").slice(0, -".bundle.json".length));
    }
  };
  if (fs.existsSync(dir)) walk(dir);
  return out.sort();
}

/** The problems found for every `<examplesDir>/hcp-demo*` against `<bundlesDir>`; empty when all match. */
export function checkDemoBundles({ examplesDir, bundlesDir }) {
  const problems = [];
  const demos = fs
    .readdirSync(examplesDir, { withFileTypes: true })
    .filter((e) => e.isDirectory() && e.name.startsWith("hcp-demo") && fs.existsSync(path.join(examplesDir, e.name, "trust-tasks.lock")))
    .map((e) => e.name)
    .sort();
  if (demos.length === 0) problems.push(`no examples/hcp-demo*/trust-tasks.lock found under ${examplesDir}`);

  for (const demo of demos) {
    const dir = path.join(examplesDir, demo);
    const lock = parseLock(fs.readFileSync(path.join(dir, "trust-tasks.lock"), "utf8"));
    if (lock.registry !== REGISTRY) problems.push(`${demo}: trust-tasks.lock records registry ${lock.registry}, not ${REGISTRY}`);
    const pinned = new Set();
    for (const task of lock.tasks) {
      const key = `${task.slug}/${task.version}`;
      pinned.add(key);
      const bundle = path.join(bundlesDir, task.slug, task.version, "bundle.json");
      if (!fs.existsSync(bundle)) {
        problems.push(`${demo}: the lock pins ${key}, which the registry build has no bundle for`);
        continue;
      }
      const digest = `sha256:${createHash("sha256").update(fs.readFileSync(bundle)).digest("hex")}`;
      if (task.digest !== digest) problems.push(`${demo}: the lock pins ${key} at ${task.digest}, but specs/ builds ${digest}`);
    }
    for (const key of vendored(path.join(dir, "trust-tasks"))) {
      if (!pinned.has(key)) {
        problems.push(`${demo}: trust-tasks/${key}.bundle.json is not pinned by its lock`);
        continue;
      }
      const bundle = path.join(bundlesDir, ...key.split("/"), "bundle.json");
      if (!fs.existsSync(bundle)) continue; // already reported against the lock
      if (!fs.readFileSync(path.join(dir, "trust-tasks", `${key}.bundle.json`)).equals(fs.readFileSync(bundle))) {
        problems.push(`${demo}: trust-tasks/${key}.bundle.json differs from the bundle specs/ builds`);
      }
    }
  }
  return { demos, problems };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const bundlesDir = path.resolve(process.argv[2] ?? path.join(ROOT, "website/specs"));
  if (!fs.existsSync(path.join(bundlesDir, "index.json"))) {
    console.error(`check-demo-bundles: no ${path.join(bundlesDir, "index.json")}; run \`npm run build-bundles\` first`);
    process.exit(2);
  }
  const { demos, problems } = checkDemoBundles({ examplesDir: path.join(ROOT, "examples"), bundlesDir });
  if (problems.length > 0) {
    for (const p of problems) console.error(`check-demo-bundles: ${p}`);
    console.error("check-demo-bundles: refresh each port as examples/hcp-demo-ts/SCENARIO.md (\"Pinned tasks\") describes");
    process.exit(1);
  }
  console.log(`check-demo-bundles: ${demos.length} HCP demo(s) match the bundles specs/ builds (${demos.join(", ")})`);
}
