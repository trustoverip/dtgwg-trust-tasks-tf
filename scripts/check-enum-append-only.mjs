// Refuse any change to an existing JSON Schema `enum` other than appending to it.
//
// ── Why ──────────────────────────────────────────────────────────────────────
//
// Every `enum` of strings becomes a Rust enum in `trust-tasks-rs`. The
// generated enum has implicit discriminants and derives `Ord`, so a value
// inserted anywhere but the end shifts every variant after it: code that casts
// one (`as isize`) or sorts by one sees a different answer. cargo-semver-checks
// reports it as `enum_no_repr_variant_discriminant_changed`, and because the
// eight Rust crates share one release-plz `version_group`, one such insertion
// makes the whole group a breaking minor release — which every downstream
// workspace can only adopt in lockstep with everything else that pins it.
//
// It happened twice in two days: `signDocument` inserted into device
// `Capability` (#819) made 0.32.0 breaking, and `pcsVoter` inserted the same
// way (#834) would have made 0.33.0 breaking, until #835 appended it instead.
// Appending is always possible and is never breaking, so this check makes it
// the only way: removing a value, reordering values, or inserting one before
// an existing value all fail here, on the pull request, rather than in the
// Release PR after the merge.
//
// ── What is compared ─────────────────────────────────────────────────────────
//
// Each `*.json` under `specs/` that changed between the base and HEAD and
// existed at the base. Both trees are walked together; wherever both have an
// `enum` array at the same JSON pointer, the base's values must be an
// unchanged prefix of HEAD's. New files, and enums new in an existing file,
// are not constrained.
//
// The base is `ENUM_BASE_REF` (CI passes the pull request's base commit), else
// `origin/main`; the comparison is against its merge base with HEAD, so a
// branch that is behind main is not blamed for main's own changes.

import { execFileSync } from "node:child_process";
import { readFileSync, existsSync } from "node:fs";

const git = (...args) =>
  execFileSync("git", args, { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 }).trim();

const baseRef = process.env.ENUM_BASE_REF || "origin/main";
const base = git("merge-base", baseRef, "HEAD");
const changed = git("diff", "--name-only", "--diff-filter=M", base, "HEAD", "--", "specs")
  .split("\n")
  .filter((p) => p.endsWith(".json"));

const violations = [];

function walk(file, pointer, before, after) {
  if (before === null || after === null || typeof before !== "object" || typeof after !== "object") {
    return;
  }
  if (Array.isArray(before) && Array.isArray(after)) {
    before.forEach((b, i) => walk(file, `${pointer}/${i}`, b, after[i] ?? null));
    return;
  }
  if (Array.isArray(before.enum) && Array.isArray(after.enum)) {
    const prefix = after.enum.slice(0, before.enum.length);
    const same = before.enum.length <= after.enum.length &&
      before.enum.every((v, i) => JSON.stringify(v) === JSON.stringify(prefix[i]));
    if (!same) {
      violations.push({ file, pointer: `${pointer}/enum`, before: before.enum, after: after.enum });
    }
  }
  for (const key of Object.keys(before)) {
    if (key in after) {
      walk(file, `${pointer}/${key.replace(/~/g, "~0").replace(/\//g, "~1")}`, before[key], after[key]);
    }
  }
}

for (const file of changed) {
  if (!existsSync(file)) continue;
  let before;
  try {
    before = JSON.parse(git("show", `${base}:${file}`));
  } catch {
    continue; // not at the base: a new file
  }
  walk(file, "", before, JSON.parse(readFileSync(file, "utf8")));
}

if (violations.length === 0) {
  console.log(`Enum check: ${changed.length} changed schema file(s) against ${base.slice(0, 9)} — every existing enum only grew at its end.`);
  process.exit(0);
}

console.error("Enum values may only be APPENDED. These enums changed otherwise:\n");
for (const v of violations) {
  const added = v.after.filter((x) => !v.before.includes(x));
  const removed = v.before.filter((x) => !v.after.includes(x));
  console.error(`  ${v.file} ${v.pointer}`);
  if (removed.length) console.error(`    removed: ${JSON.stringify(removed)}`);
  if (added.length) console.error(`    added:   ${JSON.stringify(added)}`);
  console.error(`    base:    ${JSON.stringify(v.before)}`);
  console.error(`    head:    ${JSON.stringify(v.after)}\n`);
}
console.error(
  "A value inserted or moved anywhere but the end shifts the discriminant of every\n" +
  "generated Rust variant after it (cargo-semver-checks enum_no_repr_variant_discriminant_changed),\n" +
  "which makes the whole core crate group a BREAKING release. Move new values to the end\n" +
  "of the array. Removing a value is breaking on the wire too: give it a new version folder.",
);
process.exit(1);
