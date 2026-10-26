#!/usr/bin/env node
// Check that every hand-written sibling package admits the core it is
// developed beside.
//
//   node scripts/check-sibling-core-ranges.mjs
//
// A sibling (`trust-tasks-ts-*` on `@openvtc/trust-tasks`, `trust-tasks-dart-*`
// on `trust_tasks`) declares the core it needs as a caret range, and below 1.0
// a caret admits one minor only: `^0.19.0` is `>=0.19.0 <0.20.0`. Both
// ecosystems have shipped siblings whose range excluded the current core —
// Dart until #895, TypeScript at `^0.19.0` while the core reached 0.27 — so a
// consumer got a months-old core beside its own, or could not install the
// sibling at all. Nothing else notices: each sibling's own CI resolves the
// newest core its range admits and passes.
//
// So this fails when:
//   - a TypeScript sibling lists `@openvtc/trust-tasks` anywhere but
//     `peerDependencies` (plus `devDependencies`, to build against) — a regular
//     dependency installs a second copy of the core, every prebuilt module in
//     it, beside the application's;
//   - a range is not a plain `^MAJOR.MINOR.PATCH`;
//   - a range does not admit the core version in this checkout
//     (`trust-tasks-ts/package.json`, `trust-tasks-dart/pubspec.yaml`).
//
// On the Release PR that bumps the core across a minor, this fails until the
// siblings' floors are raised to it — in a PR before that release merges, so
// the siblings release beside the core that needs them.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const failures = [];
const fail = (msg) => failures.push(msg);

const parse = (v) => {
  const m = /^(\d+)\.(\d+)\.(\d+)$/.exec(v);
  return m ? m.slice(1).map(Number) : null;
};

/** Does the caret range `range` admit `version`? (npm and pub agree on caret.) */
function caretAdmits(range, version) {
  const floor = parse(range.slice(1));
  const v = parse(version);
  if (!range.startsWith("^") || !floor) return null;
  const [a, b, c] = floor;
  const cmp = (x, y) => x[0] - y[0] || x[1] - y[1] || x[2] - y[2];
  if (cmp(v, floor) < 0) return false;
  const ceiling = a > 0 ? [a + 1, 0, 0] : b > 0 ? [0, b + 1, 0] : [0, 0, c + 1];
  return cmp(v, ceiling) < 0;
}

function check(where, range, core, coreVersion) {
  const admits = caretAdmits(range, coreVersion);
  if (admits === null) fail(`${where}: ${core} ${JSON.stringify(range)} is not a ^MAJOR.MINOR.PATCH range`);
  else if (!admits) {
    fail(`${where}: ${core} ${range} does not admit the core in this checkout, ${coreVersion} — raise it to ^${coreVersion}`);
  }
}

const siblings = (prefix) =>
  fs
    .readdirSync(ROOT)
    .filter((d) => d.startsWith(prefix))
    .sort();

// TypeScript.
const tsCore = JSON.parse(fs.readFileSync(path.join(ROOT, "trust-tasks-ts/package.json"), "utf8")).version;
let tsCount = 0;
for (const dir of siblings("trust-tasks-ts-")) {
  const file = path.join(ROOT, dir, "package.json");
  if (!fs.existsSync(file)) continue;
  const pkg = JSON.parse(fs.readFileSync(file, "utf8"));
  const peer = pkg.peerDependencies?.["@openvtc/trust-tasks"];
  const direct = pkg.dependencies?.["@openvtc/trust-tasks"];
  if (direct !== undefined) {
    fail(`${dir}/package.json: @openvtc/trust-tasks is a dependency — make it a peerDependency (and a devDependency to build against), so the application's core is the only one`);
  }
  if (peer === undefined) continue; // does not use the core (trust-tasks-ts-gen)
  tsCount++;
  check(`${dir}/package.json`, peer, "@openvtc/trust-tasks", tsCore);
  const dev = pkg.devDependencies?.["@openvtc/trust-tasks"];
  if (dev !== undefined && dev !== peer) {
    fail(`${dir}/package.json: devDependencies @openvtc/trust-tasks ${dev} differs from peerDependencies ${peer}`);
  }
}

// Dart.
const dartCore = /^version:\s*(\S+)/m.exec(fs.readFileSync(path.join(ROOT, "trust-tasks-dart/pubspec.yaml"), "utf8"))[1];
let dartCount = 0;
for (const dir of siblings("trust-tasks-dart-")) {
  const file = path.join(ROOT, dir, "pubspec.yaml");
  if (!fs.existsSync(file)) continue;
  const m = /^dependencies:\n(?:[ \t].*\n|\n)*?[ \t]+trust_tasks:[ \t]*(\S+)/m.exec(fs.readFileSync(file, "utf8"));
  if (!m) continue;
  dartCount++;
  check(`${dir}/pubspec.yaml`, m[1].replace(/^["']|["']$/g, ""), "trust_tasks", dartCore);
}

if (failures.length) {
  console.error(`check-sibling-core-ranges: ${failures.length} problem(s):`);
  for (const f of failures) console.error(`  - ${f}`);
  process.exit(1);
}
console.log(
  `check-sibling-core-ranges: ${tsCount} TypeScript siblings admit @openvtc/trust-tasks ${tsCore}; ` +
    `${dartCount} Dart siblings admit trust_tasks ${dartCore}.`,
);
