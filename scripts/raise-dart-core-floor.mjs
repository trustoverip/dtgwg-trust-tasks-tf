#!/usr/bin/env node
// Raise a Dart sibling's `trust_tasks` floor to the core it is released beside.
//
//   node scripts/raise-dart-core-floor.mjs <pubspec.yaml> <core version>
//
// Called by scripts/release-bump-dart.sh (each released sibling) and
// scripts/release-pr.sh (a sibling not yet released), with the core version the
// Release PR carries. When the sibling's `trust_tasks: ^X.Y.Z` under
// `dependencies:` does not admit that version, the line is rewritten to
// `^<core version>` and the old range is printed; otherwise nothing is touched
// and nothing is printed.
//
// Why the Release PR does this: below 1.0 a caret admits one minor, so a core
// release across a minor (`0.9.x` → `0.10.0`) leaves every sibling's `^0.9.3`
// excluding it. scripts/check-sibling-core-ranges.mjs fails on exactly that,
// and before this the Release PR that first took the Dart core across a minor
// (#1085) could not go green. Unlike npm, the Dart siblings build against the
// in-tree core (pubspec_overrides.yaml), so the new floor works in the PR's CI
// at once; on merge, a sibling's verify-dart leg waits on the core reaching
// pub.dev — see RELEASING.md.
//
// Only the constraint changes, so the one-line diff stays inside what
// `release-pr.sh open` accepts in a pubspec.

import fs from "node:fs";

const [file, core] = process.argv.slice(2);
if (!file || !core) {
  console.error("usage: raise-dart-core-floor.mjs <pubspec.yaml> <core version>");
  process.exit(2);
}

const parse = (v) => {
  const m = /^(\d+)\.(\d+)\.(\d+)$/.exec(v);
  return m ? m.slice(1).map(Number) : null;
};
const cmp = (x, y) => x[0] - y[0] || x[1] - y[1] || x[2] - y[2];

const coreV = parse(core);
if (!coreV) {
  console.error(`raise-dart-core-floor: ${JSON.stringify(core)} is not MAJOR.MINOR.PATCH`);
  process.exit(1);
}

const raw = fs.readFileSync(file, "utf8");
// The `trust_tasks:` entry inside the top-level `dependencies:` block — the
// same pattern check-sibling-core-ranges.mjs reads.
const re = /^(dependencies:\n(?:[ \t].*\n|\n)*?[ \t]+trust_tasks:[ \t]*)(\S+)(.*)$/m;
const m = re.exec(raw);
if (!m) process.exit(0); // does not depend on the core

const range = m[2].replace(/^["']|["']$/g, "");
const floor = range.startsWith("^") ? parse(range.slice(1)) : null;
if (!floor) {
  console.error(`raise-dart-core-floor: ${file}: trust_tasks ${JSON.stringify(m[2])} is not a ^MAJOR.MINOR.PATCH range`);
  process.exit(1);
}
const [a, b, c] = floor;
const ceiling = a > 0 ? [a + 1, 0, 0] : b > 0 ? [0, b + 1, 0] : [0, 0, c + 1];
if (cmp(coreV, floor) >= 0 && cmp(coreV, ceiling) < 0) process.exit(0); // admitted
if (cmp(coreV, floor) < 0) {
  // The sibling already needs a newer core than this checkout's. Lowering it
  // would be wrong; leave it for check-sibling-core-ranges to report.
  process.exit(0);
}

fs.writeFileSync(file, raw.replace(re, `$1^${core}$3`));
console.log(range);
