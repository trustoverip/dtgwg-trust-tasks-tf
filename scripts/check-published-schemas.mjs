// Refuse an in-place change to a RELEASED specification that a consumer or
// the generated libraries can see. Such a change takes a new version folder.
//
// ── Why ──────────────────────────────────────────────────────────────────────
//
// Every schema under `specs/` is compiled into `trust-tasks-rs`,
// `@openvtc/trust-tasks`, `trust-tasks-go` and `trust_tasks`. Once a version
// has been released, an in-place edit that changes its generated types is a
// breaking release of the library — and because the eight Rust crates share
// one release-plz `version_group`, of all of them at once. #858 (a member made
// optional), #758 (a member made required) and #750 (a proof made required)
// each cost the whole group a MINOR that a new version folder would not have:
// a new version is a new module, an additive patch for every library.
//
// So a released version may change in place only additively — a new optional
// member, a value appended to an enum, a branch appended to a oneOf/anyOf, a
// loosened bound, a new `$defs` entry, and any `description`, `$comment` or
// `examples` (scripts/lib/schema-compat.mjs has the full rule). In the front
// matter, `proofRequirement.requirement` may not change and declared
// `errorCodes` may only grow. Everything else fails here, on the pull request,
// rather than as a breaking Release PR after the merge.
//
// This subsumes the enum check it replaces (scripts/check-enum-append-only.mjs,
// added after #819 inserted `signDocument` mid-enum and made 0.32.0 breaking):
// an enum value inserted before an existing one is one of the changes refused.
//
// ── What is compared ─────────────────────────────────────────────────────────
//
// Each `*.json` and `spec.md` under `specs/` that this branch changed (against
// its merge base with `PUBLISHED_BASE_REF`, else `origin/main`) and that
// existed at the LAST RELEASE — the newest `trust-tasks-rs-v*` tag. The
// comparison is against the file as released, not as it is on main: a version
// merged since the last release has reached no library yet, so it can still be
// fixed in place. A file new since the release is not constrained.
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { incompatibilities, frontMatterIncompatibilities } from "./lib/schema-compat.mjs";

const git = (...args) => execFileSync("git", args, { encoding: "utf8", maxBuffer: 1 << 28 }).trim();

const release = git("tag", "--list", "trust-tasks-rs-v*", "--sort=-v:refname").split("\n")[0];
if (!release) {
  // Never fail open silently: without the release tag there is nothing to compare against.
  console.error("::error::No trust-tasks-rs-v* tag found. Fetch tags (actions/checkout fetch-depth: 0) and re-run.");
  process.exit(1);
}
const baseRef = process.env.PUBLISHED_BASE_REF || "origin/main";
const base = git("merge-base", baseRef, "HEAD");
const changed = git("diff", "--name-only", "--diff-filter=M", base, "HEAD", "--", "specs")
  .split("\n")
  .filter((p) => p.endsWith(".json") || p.endsWith("/spec.md"));

const atRelease = (file) => {
  try {
    return git("show", `${release}:${file}`);
  } catch {
    return null; // not in the release: new since, so unconstrained
  }
};
const frontMatter = (text) => {
  const m = text.match(/^---\n([\s\S]*?)\n---/);
  if (!m) return {};
  // Only the two members compared here are read, so a full YAML parser is not needed:
  // `proofRequirement:` → `requirement:` and the `- code:` entries under `errorCodes:`.
  const fm = m[1];
  const requirement = fm.match(/^proofRequirement:\s*\n\s+requirement:\s*["']?(\w+)/m)?.[1];
  const block = fm.match(/^errorCodes:\s*\n((?:[ \t]+.*\n?)*)/m)?.[1] ?? "";
  const errorCodes = [...block.matchAll(/-\s*code:\s*["']?([^"'\s]+)/g)].map((c) => ({ code: c[1] }));
  return { proofRequirement: { requirement }, errorCodes };
};

const violations = [];
let compared = 0;
for (const file of changed) {
  if (!existsSync(file)) continue;
  const released = atRelease(file);
  if (released === null) continue;
  compared++;
  const now = readFileSync(file, "utf8");
  if (file.endsWith("spec.md")) {
    for (const v of frontMatterIncompatibilities(frontMatter(released), frontMatter(now))) violations.push({ file, ...v });
    continue;
  }
  let before;
  let after;
  try {
    before = JSON.parse(released);
    after = JSON.parse(now);
  } catch {
    continue; // an unparseable schema is reported by the registry build
  }
  if (file.endsWith("payload.invalid-examples.json")) continue; // test fixtures, not a schema
  const loadBefore = (f) => {
    const t = atRelease(f);
    if (t === null) throw new Error(`${f} is not in ${release}`);
    return JSON.parse(t);
  };
  const loadAfter = (f) => JSON.parse(readFileSync(f, "utf8"));
  for (const v of incompatibilities(before, after, { beforeFile: file, afterFile: file, loadBefore, loadAfter })) {
    violations.push({ file, ...v });
  }
}

if (violations.length === 0) {
  console.log(
    `Published-schema check: ${changed.length} changed spec file(s), ${compared} of them released in ${release} — ` +
      "every released one changed only additively."
  );
  process.exit(0);
}

console.error(`These released specifications (as of ${release}) changed in a way a consumer or the generated libraries can see:\n`);
for (const v of violations) console.error(`  ${v.file}${v.pointer ? ` ${v.pointer}` : ""}\n    ${v.reason}`);
console.error(
  "\nA released version is compiled into every library, so changing it in place is a BREAKING release\n" +
    "of all of them (one release-plz version_group). Make the change in a NEW VERSION FOLDER instead —\n" +
    "a new version is a new module, an additive patch — with a `## Changes from` section, and leave the\n" +
    "released version as it was. In place, a released version may only gain an optional member, an\n" +
    "appended enum value, an appended oneOf/anyOf branch, a loosened bound, a new $defs entry, or a\n" +
    "description. See CONTRIBUTING-SPECS.md \"Version rules\"."
);
process.exit(1);
