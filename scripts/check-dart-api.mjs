#!/usr/bin/env node
// Checks that trust-tasks-dart's public API changes compatibly with the last
// release of `trust_tasks` on pub.dev — or that the PR title declares the break.
//
// Library releases are sized by the PR title (RELEASING.md): `feat(dart)!:`
// makes trust_tasks a breaking release, anything else a patch. Rust has
// cargo-semver-checks to catch a break nobody declared; this is the Dart one,
// run on the pull request so the break is caught when it is made.
//
// Usage:
//   PR_TITLE='feat(dart)!: …' node scripts/check-dart-api.mjs [--old-version X.Y.Z] [--keep]
//
// --old-version compares against that pub.dev release instead of the latest;
// --keep leaves the scratch copies and the change list behind.
//
// The verdict:
//
//   no breaking change                                  → pass
//   breaking change, title has a `!` whose scope list
//     includes `dart` (`feat(dart)!:`, `feat(rs, dart)!:`) → pass, listing the breaks
//   breaking change, no such declaration                → FAIL, listing the breaks
//
// ── How the comparison is done ────────────────────────────────────────────
// The comparison is dart_apitool's (BMW Tech): it builds an API model of each
// side from the analyzer's element model and diffs the two, classifying every
// change as breaking or not (removed declaration, removed/added-required
// parameter, changed type, …; an added declaration or optional parameter is
// not). The workflow activates it globally at a pinned version, and
// scripts/lib/dart_api_diff.dart drives its analyzer and differ through the
// activated package — see that file for why not its CLI (in short: the CLI
// never looks at lib/specs/**, and its report does not say which of ~1,100
// libraries a `Payload` change is in).
//
// The old side is the pub.dev archive of the latest release (what consumers
// have), the new side this tree's trust-tasks-dart/lib. Both are copied to a
// scratch directory with a dependency-free pubspec, so resolving them needs no
// network, and both are rewritten the same way (rewriteForApitool, below) to
// get around two things dart_apitool cannot do.

import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import YAML from "yaml";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const PACKAGE = "trust_tasks";
const PACKAGE_DIR = path.join(ROOT, "trust-tasks-dart");
const SCOPE = "dart";
const DRIVER = path.join(ROOT, "scripts", "lib", "dart_api_diff.dart");

const args = process.argv.slice(2);
const argValue = (name) => {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : undefined;
};

// ── The PR title ───────────────────────────────────────────────────────────
// Same grammar commit-lint.yml enforces: `type(scope, scope)!: summary`. Read
// from the environment, never from a workflow interpolation.
export function declaresBreak(title, scope) {
  const m = /^[a-z]+(?:\(([^)]*)\))?!:/.exec(title ?? "");
  if (!m || m[1] === undefined) return false;
  return m[1]
    .split(",")
    .map((s) => s.trim())
    .includes(scope);
}

// ── Rewrites dart_apitool cannot do without ────────────────────────────────
// Applied to BOTH scratch copies, so the comparison stays like with like. Each
// keeps the declaration's name and the type it carries, which is what a
// consumer compiles against; the rewritten code need not (and does not) still
// compile — dart_apitool reads declarations from the element model.
//
//  1. `extension type const X(String value) { … }` → `final class X { const
//     X(this.value); final String value; … }`. dart_apitool 0.23.2 predates
//     extension types and drops them ("elements … marked to belong to an
//     interface but the interface is not collected"), so the ~2,000 generated
//     closed-value types and the runtime's StandardCode would be invisible: a
//     removed value constant would pass. `final` because nothing outside the
//     package can implement an extension type's interface either, so adding a
//     value is not a break.
//
//  2. `typedef A = B;` → `abstract final class A { B get aliasOf; }`. A typedef
//     naming a class of this package trips a dart_apitool bug: the aliased
//     class's entry points absorb those of every same-named class in other
//     libraries (`typedef Response = DidTemplateRecord;` in eleven vta
//     libraries), and the diff then aborts with "Multiple new items found with
//     the same structure". As a class with one getter, removing the alias or
//     pointing it at another type is still reported as a break; adding one is
//     not.
export function rewriteForApitool(source) {
  return source
    .replace(
      /^extension type const (\w+)\(\s*([\w<>?, ]+?)\s+(\w+)\s*\)\s*\{/gm,
      (_, name, type, field) =>
        `final class ${name} {\n  const ${name}(this.${field});\n  final ${type} ${field};`,
    )
    .replace(
      /^typedef (\w+) = ([^;\n]+);$/gm,
      (_, name, target) => `abstract final class ${name} {\n  ${target} get aliasOf;\n}`,
    );
}

function dartFiles(dir, base = dir, out = []) {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const abs = path.join(dir, e.name);
    if (e.isDirectory()) dartFiles(abs, base, out);
    else if (e.name.endsWith(".dart")) out.push(path.relative(base, abs).split(path.sep).join("/"));
  }
  return out;
}

// Copies `<src>/lib` and a dependency-free pubspec to `dest`, rewrites it, and
// resolves it. trust_tasks has no dependencies; its dev dependencies are not
// API, and dropping them is what lets `pub get --offline` resolve.
function prepare(src, dest) {
  fs.mkdirSync(dest, { recursive: true });
  fs.cpSync(path.join(src, "lib"), path.join(dest, "lib"), { recursive: true });
  const pubspec = YAML.parse(fs.readFileSync(path.join(src, "pubspec.yaml"), "utf8"));
  if (pubspec.dependencies && Object.keys(pubspec.dependencies).length > 0) {
    throw new Error(
      `${PACKAGE} now has dependencies; teach prepare() to keep them (they are part of the API surface)`,
    );
  }
  fs.writeFileSync(
    path.join(dest, "pubspec.yaml"),
    YAML.stringify({ name: pubspec.name, version: pubspec.version, environment: pubspec.environment }),
  );
  const libs = dartFiles(path.join(dest, "lib"));
  for (const rel of libs) {
    const abs = path.join(dest, "lib", rel);
    const before = fs.readFileSync(abs, "utf8");
    const after = rewriteForApitool(before);
    if (after !== before) fs.writeFileSync(abs, after);
  }
  execFileSync("dart", ["pub", "get", "--offline"], { cwd: dest, stdio: ["ignore", "ignore", "inherit"] });
  return new Set(libs.filter((l) => !l.startsWith("src/")));
}

async function latestPublished() {
  const res = await fetch(`https://pub.dev/api/packages/${PACKAGE}`, {
    headers: { accept: "application/vnd.pub.v2+json" },
  });
  if (!res.ok) throw new Error(`pub.dev answered ${res.status} for ${PACKAGE}`);
  return (await res.json()).latest.version;
}

async function fetchPublished(version, dest) {
  const url = `https://pub.dev/api/archives/${PACKAGE}-${version}.tar.gz`;
  const res = await fetch(url);
  if (!res.ok) throw new Error(`could not download ${url}: ${res.status}`);
  fs.mkdirSync(dest, { recursive: true });
  const tgz = `${dest}.tar.gz`;
  fs.writeFileSync(tgz, Buffer.from(await res.arrayBuffer()));
  execFileSync("tar", ["-xzf", tgz, "-C", dest]);
}

// The globally activated dart_apitool's resolved package config, so the driver
// runs against exactly the version the workflow pinned.
function apitoolPackages() {
  if (process.env.DART_APITOOL_PACKAGES) return process.env.DART_APITOOL_PACKAGES;
  const cache = process.env.PUB_CACHE || path.join(os.homedir(), ".pub-cache");
  const global = path.join(cache, "global_packages", "dart_apitool");
  const config = path.join(global, ".dart_tool", "package_config.json");
  if (!fs.existsSync(config)) {
    throw new Error(
      `dart_apitool is not activated (no ${config}); run \`dart pub global activate dart_apitool <version>\``,
    );
  }
  const lock = YAML.parse(fs.readFileSync(path.join(global, "pubspec.lock"), "utf8"));
  console.log(`dart_apitool ${lock.packages?.dart_apitool?.version ?? "(unknown version)"}`);
  return config;
}

// ── Main ───────────────────────────────────────────────────────────────────
/**
 * Whether a commit merged since the published `version` already makes the next
 * release breaking, by scripts/release-bump-dart.sh's own rule: a `!` scoped to
 * this package (or unscoped), or a `BREAKING CHANGE` trailer, among commits
 * touching what the package is generated from. Without it, every pull request
 * after a declared break would fail until the next release, because pub.dev
 * still has the old API. Returns the deciding subject, or null. Keep it identical
 * to the bump script's `breaking_here`.
 */
function alreadyBreaking(version) {
  const SCOPES = ["dart", "dart-bindings"];
  const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
  let log;
  try {
    log = execFileSync(
      "git",
      ["log", "--format=%s%n%b", `trust-tasks-dart-v${version}..${process.env.BASE_REF || "HEAD"}`, "--", "specs", "trust-tasks-dart", "scripts/build-dart-bindings.mjs"],
      { cwd: root, encoding: "utf8", maxBuffer: 1 << 28 },
    );
  } catch {
    return null; // no such tag locally (a shallow clone): judge the PR on its own
  }
  if (/^BREAKING[ -]CHANGE/m.test(log)) return "a BREAKING CHANGE trailer";
  for (const line of log.split("\n")) {
    const m = /^[a-z]+(?:\(([^)]*)\))?!:/.exec(line);
    if (!m) continue;
    if (!m[1] || m[1].split(",").map((x) => x.trim()).some((x) => SCOPES.includes(x))) return line;
  }
  return null;
}

async function main() {
  const declared = declaresBreak(process.env.PR_TITLE ?? "", SCOPE);
  const packages = apitoolPackages();
  const oldVersion = argValue("--old-version") ?? (await latestPublished());

  const work = fs.mkdtempSync(path.join(process.env.RUNNER_TEMP || os.tmpdir(), "dart-api-"));
  const published = path.join(work, "published");
  await fetchPublished(oldVersion, published);
  const oldLibs = prepare(published, path.join(work, "old"));
  const newLibs = prepare(PACKAGE_DIR, path.join(work, "new"));
  console.log(
    `comparing ${PACKAGE} ${oldVersion} (pub.dev, ${oldLibs.size} public libraries) with this tree (${newLibs.size})`,
  );

  const out = path.join(work, "changes.json");
  const started = Date.now();
  const run = spawnSync(
    "dart",
    [`--packages=${packages}`, DRIVER, path.join(work, "old"), path.join(work, "new"), out],
    { encoding: "utf8", maxBuffer: 256 * 1024 * 1024 },
  );
  if (run.status !== 0 || !fs.existsSync(out)) {
    process.stdout.write(run.stdout ?? "");
    process.stderr.write(run.stderr ?? "");
    throw new Error(`dart_apitool exited ${run.status}${run.error ? `: ${run.error.message}` : ""}`);
  }
  console.log(`dart_apitool analysed and compared both in ${((Date.now() - started) / 1000).toFixed(0)}s`);

  const changes = JSON.parse(fs.readFileSync(out, "utf8"));
  if (args.includes("--keep")) console.log(`kept ${work}`);
  else fs.rmSync(work, { recursive: true, force: true });

  const breaking = changes.filter((c) => c.breaking);
  const compatible = changes.length - breaking.length;
  const added = [...newLibs].filter((l) => !oldLibs.has(l)).length;

  if (breaking.length === 0) {
    console.log(
      `ok: no breaking change to ${PACKAGE}'s public API since ${oldVersion} ` +
        `(${compatible} compatible change${compatible === 1 ? "" : "s"}, ${added} new librar${added === 1 ? "y" : "ies"})`,
    );
    return 0;
  }

  // A removed library is one line, not one per declaration it held.
  const removedLibs = [...oldLibs].filter((l) => !newLibs.has(l)).sort();
  const removedPaths = new Set(removedLibs.map((l) => `lib/${l}`));
  const lines = [
    ...removedLibs.map((l) => `lib/${l}: library removed`),
    ...breaking
      .filter((c) => !removedPaths.has(c.library))
      .map((c) => `${c.library ?? "(package)"}: ${[...c.context, c.description].join(" › ")} [${c.code}]`)
      .sort(),
  ];
  const list = lines.map((l) => `  - ${l}`).join("\n");
  const n = `${breaking.length} breaking change${breaking.length === 1 ? "" : "s"}`;

  if (declared) {
    console.log(`ok: ${n} to ${PACKAGE} since ${oldVersion}, declared by the title's \`!\` (${SCOPE}):\n${list}`);
    return 0;
  }
  const prior = alreadyBreaking(oldVersion);
  if (prior) {
    console.log(`ok: ${n} to ${PACKAGE} since ${oldVersion}; a change merged since that release already makes the next one breaking (${prior}):\n${list}`);
    return 0;
  }
  console.log(`::error::${n} to ${PACKAGE}'s public API since ${oldVersion}, and the PR title does not declare one.`);
  console.log(list);
  console.log(`
Either make the change compatible, or retitle the PR with \`feat(${SCOPE})!:\`
(a \`!\` whose scope list includes \`${SCOPE}\`) so the release is sized as
breaking — a MINOR of ${PACKAGE} below 1.0. See RELEASING.md.`);
  return 1;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main().then(
    (code) => process.exit(code),
    (err) => {
      console.error(`::error::${err.message}`);
      process.exit(2);
    },
  );
}
