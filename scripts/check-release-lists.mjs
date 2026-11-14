#!/usr/bin/env node
// Checks that every hand-maintained list naming the npm packages, the Go
// modules and the crates names the same ones, and that commit-lint's `!` scope
// vocabulary is exactly the set of scopes the release tooling recognises.
//
// The Dart lists have their own, older guard (scripts/check-dart-packages.mjs,
// `npm run check-dart-packages`); this is the same idea for the rest (#1139):
//
//   npm   .github/workflows/publish.yml   build-npm matrix (name, dir, prefix)
//         scripts/release-pr.sh           TS_PACKAGES
//         scripts/release-bump-ts.sh      the `case` arms (dir, tag prefix)
//         .github/workflows/ts.yml        some job builds each sibling's dir
//   Go    .github/workflows/publish.yml   verify-go matrix (dir, versionfile,
//                                         tagprefix, module)
//         scripts/release-pr.sh           GO_MODULES
//         scripts/release-bump-go.sh      the `case` arms, and the core's
//                                         `:(exclude)` list of nested modules
//         scripts/check-go-api-compat.sh  the same `:(exclude)` list
//         .github/workflows/go.yml        a job runs in each nested module
//   crates  every workspace member's Cargo.toml (`publish`, a published
//         dependency on trust-tasks-rs), release-plz.toml's `core` group,
//         scripts/check-rs-api-compat.sh  CRATES and CORE_SCOPES,
//         .github/workflows/rust.yml      the pull_request path filter
//   scopes  .github/workflows/commit-lint.yml `pkg=` against the SCOPES of the
//         three bump scripts, check-rs-api-compat.sh and the published crates
//
// Each of these lists has drifted before, or is the shape of a list that has:
// #483 (a Dart matrix entry with no `dir`, found by a failed release), a nested
// Go module missing from the core's exclude list (it would release the core on
// every change to it), a crate in the `core` group no API check covers, and
// `!` scopes commit-lint accepts but no bump script reads (scored as a patch).
//
// Like the other guards in this repo it re-reads the files rather than sharing
// a package list with them: a shared list would only assert that it agrees with
// itself.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import YAML from "yaml";

const ROOT = process.env.RELEASE_LISTS_ROOT
  ? path.resolve(process.env.RELEASE_LISTS_ROOT)
  : path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (p) => fs.readFileSync(path.join(ROOT, p), "utf8");
const exists = (p) => fs.existsSync(path.join(ROOT, p));
const errors = [];
const notes = [];
const fail = (msg) => errors.push(msg);

const sameSet = (label, want, got, wantLabel) => {
  const w = [...new Set(want)].sort();
  const g = [...new Set(got)].sort();
  if (JSON.stringify(w) !== JSON.stringify(g) || got.length !== g.length) {
    fail(`${label} lists [${got.join(", ")}]; ${wantLabel} lists [${want.join(", ")}]`);
  }
};
// A bash array `NAME=( "a" "b" )` from a script.
const bashArray = (text, name) => {
  const m = text.match(new RegExp(`^${name}=\\(\\n([\\s\\S]*?)\\n\\)$`, "m"));
  return m ? [...m[1].matchAll(/"([^"]+)"/g)].map((x) => x[1]) : null;
};
// The `case "$PKG" in` arms of a bump script: arm label -> its body text.
const caseArms = (text) => {
  const arms = new Map();
  const re = /^ {2}([^\s)*][^)\n]*)\)\n([\s\S]*?)\n {4};;$/gm;
  for (const m of text.matchAll(re)) arms.set(m[1], m[2]);
  return arms;
};
const assignment = (body, name) => body.match(new RegExp(`^\\s*${name}="([^"]*)"`, "m"))?.[1];

const publish = YAML.parse(read(".github/workflows/publish.yml"));
const releasePr = read("scripts/release-pr.sh");

// ── npm ────────────────────────────────────────────────────────────────────
const npmMatrix = publish.jobs["build-npm"]?.strategy?.matrix?.include ?? [];
const publishNpmMatrix = publish.jobs["publish-npm"]?.strategy?.matrix?.include ?? [];
if (JSON.stringify(npmMatrix) !== JSON.stringify(publishNpmMatrix)) {
  fail("publish.yml build-npm and publish-npm matrices differ (publish-npm should reuse *npm-packages)");
}
const npm = new Map(); // name -> { dir, prefix }
for (const [i, e] of npmMatrix.entries()) {
  const where = `publish.yml build-npm matrix entry ${i} (${e.name ?? "unnamed"})`;
  const missing = ["name", "dir", "prefix"].filter((k) => typeof e[k] !== "string" || !e[k]);
  if (missing.length) {
    fail(`${where} has no ${missing.map((k) => `\`${k}\``).join(", ")}`);
    continue;
  }
  if (npm.has(e.name)) fail(`${where}: ${e.name} is listed twice`);
  if (e.prefix !== `${e.dir}-v`) fail(`${where}: prefix ${e.prefix} should be ${e.dir}-v`);
  if (!exists(`${e.dir}/package.json`)) fail(`${where}: ${e.dir}/package.json does not exist`);
  else {
    const name = JSON.parse(read(`${e.dir}/package.json`)).name;
    if (name !== e.name) fail(`${where}: ${e.dir}/package.json is named ${name}, not ${e.name}`);
  }
  npm.set(e.name, { dir: e.dir, prefix: e.prefix });
}
if (npmMatrix[0]?.name !== "@openvtc/trust-tasks") fail("publish.yml build-npm matrix must list @openvtc/trust-tasks first");

const tsPackages = bashArray(releasePr, "TS_PACKAGES");
if (!tsPackages) fail("scripts/release-pr.sh has no TS_PACKAGES=( … ) array");
else {
  sameSet("release-pr.sh TS_PACKAGES", [...npm.keys()], tsPackages, "publish.yml build-npm");
  if (tsPackages[0] !== "@openvtc/trust-tasks") fail("release-pr.sh TS_PACKAGES must list @openvtc/trust-tasks first");
}

const bumpTs = caseArms(read("scripts/release-bump-ts.sh"));
for (const [name, { dir, prefix }] of npm) {
  const body = bumpTs.get(name);
  if (!body) {
    fail(`release-bump-ts.sh has no case arm for ${name}`);
    continue;
  }
  const d = assignment(body, "PKG_DIR");
  const p = assignment(body, "TAG_PREFIX");
  if (d !== dir) fail(`release-bump-ts.sh case arm for ${name} uses PKG_DIR=${d}; publish.yml says ${dir}`);
  if (p !== prefix) fail(`release-bump-ts.sh case arm for ${name} uses TAG_PREFIX=${p}; publish.yml says ${prefix}`);
}
for (const name of bumpTs.keys()) {
  if (!npm.has(name)) notes.push(`release-bump-ts.sh has a case arm for ${name}, which is not released (no publish.yml matrix entry)`);
}

// Some ts.yml job must run in every sibling package's directory: a package
// nothing tests is released untested.
const tsYmlText = read(".github/workflows/ts.yml");
const tsYml = YAML.parse(tsYmlText);
const tsMatrixDirs = new Set(tsYml.jobs.packages?.strategy?.matrix?.package ?? []);
for (const [name, { dir }] of npm) {
  if (dir === "trust-tasks-ts") continue; // the core has its own jobs
  const ran = tsMatrixDirs.has(dir) || new RegExp(`working-directory:\\s*${dir}\\s*$`, "m").test(tsYmlText);
  if (!ran) fail(`ts.yml never builds or tests ${dir} (${name}): add it to the \`packages\` matrix`);
}

// ── Go ─────────────────────────────────────────────────────────────────────
const GO_ROOT = "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go";
const goMatrix = publish.jobs["verify-go"]?.strategy?.matrix?.include ?? [];
const publishGoMatrix = publish.jobs["publish-go"]?.strategy?.matrix?.include ?? [];
if (JSON.stringify(goMatrix) !== JSON.stringify(publishGoMatrix)) {
  fail("publish.yml verify-go and publish-go matrices differ (publish-go should reuse *go-modules)");
}
const go = new Map(); // name -> entry
for (const [i, e] of goMatrix.entries()) {
  const where = `publish.yml verify-go matrix entry ${i} (${e.name ?? "unnamed"})`;
  const missing = ["name", "dir", "versionfile", "tagprefix", "module"].filter((k) => typeof e[k] !== "string" || !e[k]);
  if (missing.length) {
    fail(`${where} has no ${missing.map((k) => `\`${k}\``).join(", ")}`);
    continue;
  }
  if (go.has(e.name)) fail(`${where}: ${e.name} is listed twice`);
  if (e.name !== e.dir) fail(`${where}: name ${e.name} should equal dir ${e.dir}`);
  // The go command derives a nested module's tag prefix from its directory, so
  // this is a requirement, not a convention.
  if (e.tagprefix !== `${e.dir}/v`) fail(`${where}: tagprefix ${e.tagprefix} should be ${e.dir}/v`);
  const wantModule = e.dir === "trust-tasks-go" ? GO_ROOT : `${GO_ROOT}/${e.dir.replace(/^trust-tasks-go\//, "")}`;
  if (e.module !== wantModule) fail(`${where}: module ${e.module} should be ${wantModule}`);
  if (!exists(`${e.dir}/go.mod`)) fail(`${where}: ${e.dir}/go.mod does not exist`);
  else {
    const mod = read(`${e.dir}/go.mod`).match(/^module\s+(\S+)/m)?.[1];
    if (mod !== e.module) fail(`${where}: ${e.dir}/go.mod declares module ${mod}, not ${e.module}`);
  }
  if (!exists(e.versionfile)) fail(`${where}: ${e.versionfile} does not exist`);
  else if (!/^const Version = "[^"]+"$/m.test(read(e.versionfile))) fail(`${where}: ${e.versionfile} has no \`const Version = "…"\``);
  go.set(e.name, e);
}
if (goMatrix[0]?.name !== "trust-tasks-go") fail("publish.yml verify-go matrix must list trust-tasks-go first");

const goModules = bashArray(releasePr, "GO_MODULES");
if (!goModules) fail("scripts/release-pr.sh has no GO_MODULES=( … ) array");
else {
  sameSet("release-pr.sh GO_MODULES", [...go.keys()], goModules, "publish.yml verify-go");
  if (goModules[0] !== "trust-tasks-go") fail("release-pr.sh GO_MODULES must list trust-tasks-go first");
}

const bumpGoText = read("scripts/release-bump-go.sh");
const bumpGo = caseArms(bumpGoText);
for (const [name, e] of go) {
  const body = bumpGo.get(name);
  if (!body) {
    fail(`release-bump-go.sh has no case arm for ${name}`);
    continue;
  }
  const dir = assignment(body, "PKG_DIR");
  const expand = (v) => v?.replaceAll("$PKG_DIR", dir ?? "");
  const vf = expand(assignment(body, "VERSION_FILE"));
  const tp = assignment(body, "TAG_PREFIX");
  const mod = assignment(body, "MODULE")?.replace("$GO_ROOT", GO_ROOT);
  if (dir !== e.dir) fail(`release-bump-go.sh case arm for ${name} uses PKG_DIR=${dir}; publish.yml says ${e.dir}`);
  if (vf !== e.versionfile) fail(`release-bump-go.sh case arm for ${name} uses VERSION_FILE=${vf}; publish.yml says ${e.versionfile}`);
  if (tp !== e.tagprefix) fail(`release-bump-go.sh case arm for ${name} uses TAG_PREFIX=${tp}; publish.yml says ${e.tagprefix}`);
  if (mod !== e.module) fail(`release-bump-go.sh case arm for ${name} uses MODULE=${mod}; publish.yml says ${e.module}`);
}
for (const name of bumpGo.keys()) {
  if (!go.has(name)) notes.push(`release-bump-go.sh has a case arm for ${name}, which is not released (no publish.yml matrix entry)`);
}

// Every nested module — released or not (tsp and specs are not) — must be excluded from
// the core's watch in BOTH scripts, or a change to it releases the core.
const nested = fs
  .readdirSync(path.join(ROOT, "trust-tasks-go"), { withFileTypes: true })
  .filter((d) => d.isDirectory() && exists(`trust-tasks-go/${d.name}/go.mod`))
  .map((d) => d.name)
  .sort();
const excludes = (text) => [...text.matchAll(/":\(exclude\)\$PKG_DIR\/([a-z0-9_-]+)"/g)].map((m) => m[1]).sort();
sameSet("release-bump-go.sh core WATCH :(exclude)s", nested, excludes(bumpGo.get("trust-tasks-go") ?? ""), "the nested go.mod files under trust-tasks-go");
sameSet("check-go-api-compat.sh WATCH :(exclude)s", nested, excludes(read("scripts/check-go-api-compat.sh")), "the nested go.mod files under trust-tasks-go");

// A nested module CI deliberately does not build yet, with the reason. Each is
// still excluded from the core's watch above; it needs a go.yml job, as every
// other nested module does, before it leaves this list.
const NOT_BUILT_YET = new Map([
  [
    "specs",
    "the generated packages' module (#1112 step 2) cannot require a published core without specs/ until one is released; step 3 adds its job and its release",
  ],
]);
for (const [name, why] of NOT_BUILT_YET) {
  if (!nested.includes(name)) fail(`NOT_BUILT_YET names trust-tasks-go/${name}, which has no go.mod; drop it from the list`);
  else if (go.has(`trust-tasks-go/${name}`)) fail(`trust-tasks-go/${name} is released (publish.yml verify-go) but in NOT_BUILT_YET — a released module needs a go.yml job`);
  else notes.push(`trust-tasks-go/${name} is not built in CI yet: ${why}`);
}

const goYml = read(".github/workflows/go.yml");
for (const name of nested) {
  if (NOT_BUILT_YET.has(name)) continue;
  if (!new RegExp(`working-directory:\\s*trust-tasks-go/${name}\\s*$`, "m").test(goYml)) {
    fail(`go.yml has no job running in trust-tasks-go/${name}; the core's \`./...\` does not reach a nested module`);
  }
}

// ── crates ─────────────────────────────────────────────────────────────────
const rootToml = read("Cargo.toml");
const membersBlock = rootToml.match(/^members\s*=\s*\[([\s\S]*?)\]/m)?.[1] ?? "";
const members = [...membersBlock.matchAll(/"([^"]+)"/g)].map((m) => m[1]);
if (!members.length) fail("Cargo.toml has no [workspace] members");
// The [package] table of a manifest, up to the next table header.
const packageTable = (toml) => toml.match(/^\[package\]\n([\s\S]*?)(?=^\[)/m)?.[1] ?? "";
const crates = new Map(); // name -> { dir, published, lib, onRs }
for (const dir of members) {
  const toml = read(`${dir}/Cargo.toml`);
  const pkg = packageTable(toml);
  const name = pkg.match(/^name\s*=\s*"([^"]+)"/m)?.[1];
  const published = !/^publish\s*=\s*false/m.test(pkg);
  const lib = exists(`${dir}/src/lib.rs`) || /^\[lib\]/m.test(toml);
  // A published dependency on trust-tasks-rs (with a `version`), outside the
  // dev-dependencies: what puts a crate's API in trust-tasks-rs's terms.
  const deps = toml.match(/^\[dependencies\]\n([\s\S]*?)(?=^\[|(?![\s\S]))/m)?.[1] ?? "";
  const onRs = /^trust-tasks-rs\s*=\s*\{[^}]*\bversion\s*=/m.test(deps);
  crates.set(name, { dir, published, lib, onRs });
}

const relPlz = read("release-plz.toml");
const core = new Set();
for (const m of relPlz.matchAll(/\[\[package\]\]\s*\nname\s*=\s*"([^"]+)"\s*\nversion_group\s*=\s*"core"/g)) core.add(m[1]);
if (!core.has("trust-tasks-rs")) fail("release-plz.toml's `core` version group does not include trust-tasks-rs");
for (const name of core) {
  if (!crates.has(name)) fail(`release-plz.toml's core group names ${name}, which is not a workspace member`);
  else if (!crates.get(name).published) fail(`release-plz.toml's core group names ${name}, which sets publish = false`);
}
for (const [name, c] of crates) {
  if (c.published && c.onRs && !core.has(name)) {
    fail(`${name} depends on trust-tasks-rs but is not in release-plz.toml's core version group (see the "ONE COMPATIBILITY UNIT" note there)`);
  }
}

const rsCompat = read("scripts/check-rs-api-compat.sh");
const crateBlock = rsCompat.match(/^CRATES=\(\n([\s\S]*?)\n\)$/m)?.[1];
if (!crateBlock) fail("scripts/check-rs-api-compat.sh has no CRATES=( … ) array");
const apiChecked = [...(crateBlock ?? "").matchAll(/"([a-z0-9-]+)\|/g)].map((m) => m[1]);
// A binary-only crate has no library API to check.
const wantChecked = [...crates].filter(([, c]) => c.published && c.lib).map(([n]) => n);
sameSet("check-rs-api-compat.sh CRATES", wantChecked, apiChecked, "the published library crates");

const coreScopes = (rsCompat.match(/^CORE_SCOPES="([^"]*)"/m)?.[1] ?? "").split(/\s+/).filter(Boolean);
if (!coreScopes.length) fail("scripts/check-rs-api-compat.sh has no CORE_SCOPES");
const shortName = (crate) => (crate === "trust-tasks-rs" ? "rs" : crate.replace(/^trust-tasks-/, ""));
for (const name of core) {
  if (name === "trust-tasks") continue; // the facade: declared through the crates it re-exports
  if (!coreScopes.includes(shortName(name))) fail(`check-rs-api-compat.sh CORE_SCOPES has no \`${shortName(name)}\` for ${name}, a core-group crate`);
}
// The crates versioned on their own must each have their own scope arm.
const scopeArms = new Map([...rsCompat.matchAll(/^\s*(trust-tasks-[a-z0-9-]+)\) echo ([a-z0-9 -]+) ;;$/gm)].map((m) => [m[1], m[2].trim()]));
for (const name of wantChecked) {
  if (core.has(name)) continue;
  const arm = scopeArms.get(name);
  if (!arm) fail(`check-rs-api-compat.sh scopes_for has no arm for ${name}, which versions on its own — it would be judged by CORE_SCOPES`);
  else if (arm !== shortName(name)) fail(`check-rs-api-compat.sh scopes_for gives ${name} the scope \`${arm}\`; expected \`${shortName(name)}\``);
}

// rust.yml is path-filtered: every member's directory must be in the filter,
// or a change to that crate runs no Rust CI (rust.yml:16-19 records it happening).
const rustPaths = YAML.parse(read(".github/workflows/rust.yml")).on?.pull_request?.paths ?? [];
const covered = (dir) =>
  rustPaths.some((p) => p === `${dir}/**` || (p.endsWith("/**") && `${dir}/`.startsWith(p.slice(0, -2))));
for (const [name, { dir }] of crates) {
  if (!covered(dir)) fail(`rust.yml's pull_request paths do not cover ${dir}/ (${name})`);
}

// ── `!` scopes ─────────────────────────────────────────────────────────────
// commit-lint must accept exactly the scopes some release consumer reads. One it
// accepts that none reads (`runtime`, a typo) is scored as a patch; one a bump
// script reads that commit-lint refuses can never be declared.
const lint = read(".github/workflows/commit-lint.yml");
const pkgLine = lint.match(/^\s*pkg='([^']+)'\s*$/m)?.[1];
if (!pkgLine) fail("commit-lint.yml has no `pkg='…'` scope list");
const lintScopes = (pkgLine ?? "").split("|");
for (const s of lintScopes) {
  if (!/^[a-z0-9-]+$/.test(s)) fail(`commit-lint.yml pkg= entry \`${s}\` is not a literal scope; list every scope explicitly`);
}
const scriptScopes = (file) =>
  [...read(file).matchAll(/^\s*SCOPES="([^"]*)"/gm)].flatMap((m) => m[1].split(/\s+/).filter(Boolean));
const recognised = new Set([
  ...scriptScopes("scripts/release-bump-ts.sh"),
  ...scriptScopes("scripts/release-bump-go.sh"),
  ...scriptScopes("scripts/release-bump-dart.sh"),
  ...coreScopes,
  ...scopeArms.values(),
  // release-plz reads a `!` on any commit touching a crate's files, whatever
  // its scope; each published crate's own name is the scope to declare it with.
  ...[...crates].filter(([, c]) => c.published && c.dir.startsWith("trust-tasks")).map(([n]) => shortName(n)).filter((s) => s !== "trust-tasks"),
]);
sameSet("commit-lint.yml pkg=", [...recognised], lintScopes, "the release scripts (bump scripts' SCOPES, check-rs-api-compat.sh, published crates)");

// ── report ─────────────────────────────────────────────────────────────────
for (const n of notes) console.log(`note: ${n}`);
if (errors.length) {
  console.error("Release lists disagree:\n" + errors.map((e) => `  - ${e}`).join("\n"));
  process.exit(1);
}
console.log(
  `Release lists agree: npm [${[...npm.keys()].join(", ")}]; Go [${[...go.keys()].join(", ")}]; ` +
    `crates [${wantChecked.join(", ")}]; ${recognised.size} \`!\` scopes.`
);
