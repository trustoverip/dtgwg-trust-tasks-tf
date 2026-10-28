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
// One exception, for the TypeScript core only: when the core version in this
// checkout is not on npm yet — the Release PR that bumps it, until that merges
// and publishes — the siblings are held to the newest PUBLISHED core instead,
// and a range that will not admit the new one is a warning. The siblings build
// and test against the core from npm (their lockfiles pin it), so a range
// raised to a core that does not exist yet cannot be installed: `npm ci` fails
// in every sibling job, on the Release PR and on main alike. Raise the floors
// in the PR right after the release publishes; until that merges, every PR
// fails here again, which is the point. (#963 was the first Release PR to take
// the core across a minor after this check landed, and could not go green.)
// If npm cannot be reached the check stays strict.

import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";
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

/** Versions of an npm package on the registry, or null if npm cannot be asked. */
function publishedVersions(name) {
  try {
    const out = execFileSync("npm", ["view", name, "versions", "--json"], {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
      timeout: 30_000,
    });
    const list = JSON.parse(out);
    return (Array.isArray(list) ? list : [list]).filter((v) => parse(v));
  } catch {
    return null;
  }
}

const cmpVersions = (a, b) => {
  const [x, y] = [parse(a), parse(b)];
  return x[0] - y[0] || x[1] - y[1] || x[2] - y[2];
};
const newest = (versions) => versions.reduce((a, b) => (cmpVersions(a, b) >= 0 ? a : b));

const warnings = [];

// TypeScript.
const tsCheckout = JSON.parse(fs.readFileSync(path.join(ROOT, "trust-tasks-ts/package.json"), "utf8")).version;
let tsCore = tsCheckout;
const tsPublished = process.env.TT_SIBLING_RANGES_OFFLINE ? null : publishedVersions("@openvtc/trust-tasks");
if (tsPublished && tsPublished.length && !tsPublished.includes(tsCheckout)) {
  tsCore = newest(tsPublished);
}
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
  if (tsCore !== tsCheckout && caretAdmits(peer, tsCheckout) === false) {
    warnings.push(
      `${dir}/package.json: @openvtc/trust-tasks ${peer} will not admit ${tsCheckout}, which is not on npm yet — raise it to ^${tsCheckout} once that release publishes`,
    );
  }
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

for (const w of warnings) console.log(process.env.GITHUB_ACTIONS ? `::warning::${w}` : `warning: ${w}`);

if (failures.length) {
  console.error(`check-sibling-core-ranges: ${failures.length} problem(s):`);
  for (const f of failures) console.error(`  - ${f}`);
  process.exit(1);
}
console.log(
  `check-sibling-core-ranges: ${tsCount} TypeScript siblings admit @openvtc/trust-tasks ${tsCore}` +
    (tsCore !== tsCheckout ? ` (the newest on npm; ${tsCheckout} is unpublished)` : "") +
    "; " +
    `${dartCount} Dart siblings admit trust_tasks ${dartCore}.`,
);
