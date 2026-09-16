#!/usr/bin/env node
// Checks that every place naming the Dart packages names the same ones.
//
// Five hand-maintained lists have to agree for a Dart package to release:
//
//   .github/workflows/publish.yml       tag-dart matrix (name, dir, prefix)
//   .github/workflows/publish.yml       release-dart-pr matrix (name)
//   .github/workflows/publish-dart.yml  tag patterns and the `case` arms
//   scripts/release-dart-pr.sh          the `case` arms
//   .github/workflows/dart.yml          the `packages` job (all but the core)
//
// This exists because they did not. #483 added trust_tasks_didcomm with a text
// replacement that matched the tag-dart matrix instead of release-dart-pr's,
// leaving a tag-dart entry with no `dir` and no `prefix`. actionlint passed —
// the YAML was valid — and the first sign was a failed release job.
//
// Like the other guards in this repo it re-reads the files rather than sharing
// a package list with them: a shared list would only assert that it agrees with
// itself.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import YAML from "yaml";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (p) => fs.readFileSync(path.join(ROOT, p), "utf8");
const errors = [];
const fail = (msg) => errors.push(msg);

// ── publish.yml ────────────────────────────────────────────────────────────
const publish = YAML.parse(read(".github/workflows/publish.yml"));
const tagMatrix = publish.jobs["tag-dart"]?.strategy?.matrix?.include ?? [];
const prMatrix = publish.jobs["release-dart-pr"]?.strategy?.matrix?.include ?? [];

const packages = new Map(); // name -> { dir, prefix }
for (const [i, entry] of tagMatrix.entries()) {
  const where = `publish.yml tag-dart matrix entry ${i}`;
  for (const key of ["name", "dir", "prefix"]) {
    if (typeof entry[key] !== "string" || entry[key] === "") {
      fail(`${where} (${entry.name ?? "unnamed"}) has no \`${key}\``);
    }
  }
  if (!entry.name || !entry.dir || !entry.prefix) continue;
  if (packages.has(entry.name)) fail(`${where}: ${entry.name} is listed twice`);
  if (entry.prefix !== `${entry.dir}-v`) {
    fail(`${where}: prefix ${entry.prefix} should be ${entry.dir}-v`);
  }
  const pubspec = path.join(ROOT, entry.dir, "pubspec.yaml");
  if (!fs.existsSync(pubspec)) {
    fail(`${where}: ${entry.dir}/pubspec.yaml does not exist`);
  } else {
    const name = YAML.parse(fs.readFileSync(pubspec, "utf8")).name;
    if (name !== entry.name) {
      fail(`${where}: ${entry.dir}/pubspec.yaml is named ${name}, not ${entry.name}`);
    }
  }
  packages.set(entry.name, { dir: entry.dir, prefix: entry.prefix });
}
if (tagMatrix[0]?.name !== "trust_tasks") {
  fail("publish.yml tag-dart matrix must list trust_tasks first (max-parallel: 1 relies on it)");
}

const same = (label, actual) => {
  const want = [...packages.keys()].sort();
  const got = [...new Set(actual)].sort();
  if (JSON.stringify(want) !== JSON.stringify(got) || got.length !== actual.length) {
    fail(`${label} lists [${actual.join(", ")}]; tag-dart lists [${want.join(", ")}]`);
  }
};

same("publish.yml release-dart-pr matrix", prMatrix.map((e) => e.name));

// ── publish-dart.yml ───────────────────────────────────────────────────────
const publishDartText = read(".github/workflows/publish-dart.yml");
const publishDart = YAML.parse(publishDartText);
const patterns = publishDart.on?.push?.tags ?? [];
same(
  "publish-dart.yml tag patterns",
  patterns.map((p) => {
    const prefix = p.replace(/\[0-9\]\+\.\[0-9\]\+\.\[0-9\]\+$/, "");
    return [...packages].find(([, v]) => v.prefix === prefix)?.[0] ?? `<unknown pattern ${p}>`;
  }),
);
const arms = [...publishDartText.matchAll(/^\s*([a-z-]+-v)\*\)\s*dir=([a-z-]+);\s*name=([a-z_]+);/gm)];
same("publish-dart.yml case arms", arms.map((m) => m[3]));
for (const [, prefix, dir, name] of arms) {
  const pkg = packages.get(name);
  if (pkg && (pkg.prefix !== prefix || pkg.dir !== dir)) {
    fail(`publish-dart.yml case arm for ${name} says ${prefix}/${dir}; tag-dart says ${pkg.prefix}/${pkg.dir}`);
  }
}

// ── release-dart-pr.sh ─────────────────────────────────────────────────────
const script = read("scripts/release-dart-pr.sh");
const scriptArms = [...script.matchAll(/^\s{2}([a-z_]+)\)\n\s+PKG_DIR="([a-z-]+)"/gm)];
same("release-dart-pr.sh case arms", scriptArms.map((m) => m[1]));
for (const [, name, dir] of scriptArms) {
  const pkg = packages.get(name);
  if (pkg && pkg.dir !== dir) {
    fail(`release-dart-pr.sh case arm for ${name} uses ${dir}; tag-dart says ${pkg.dir}`);
  }
}

// ── dart.yml ───────────────────────────────────────────────────────────────
const dartYml = YAML.parse(read(".github/workflows/dart.yml"));
const ciDirs = new Set(
  (dartYml.jobs.packages?.strategy?.matrix?.include ?? []).map((e) => e.package),
);
for (const [name, { dir }] of packages) {
  if (name === "trust_tasks") continue; // tested by its own jobs
  if (!ciDirs.has(dir)) fail(`dart.yml packages job never tests ${dir} (${name})`);
}

if (errors.length) {
  console.error("Dart package lists disagree:\n" + errors.map((e) => `  - ${e}`).join("\n"));
  process.exit(1);
}
console.log(`Dart package lists agree: ${[...packages.keys()].join(", ")}.`);
