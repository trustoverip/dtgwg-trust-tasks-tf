#!/usr/bin/env node
// The equivalence test: generate every registry version from its bundle with
// trust-tasks-gen-ts, and compare the result with the prebuilt modules the
// in-repo generator (scripts/build-ts-bindings.mjs) committed under
// trust-tasks-ts/src.
//
//   node scripts/equivalence.mjs [--bundles <dir>] [--work <dir>] [--verbose]
//
// <dir> is what `cargo run -p trust-tasks-codegen -- --bundles <dir>` writes
// (`index.json` + `<slug>/<M.N>/bundle.json`); without --bundles this runs that
// command into the work directory. Run after `npm run build` in this package.
//
// ── What is compared ────────────────────────────────────────────────────────
//
// A module is banner · [imports] · declarations · [re-exports] · tail.
//
//   banner        NOT compared: it names the generator and the source.
//   imports, declarations, re-exports
//                 BYTE-IDENTICAL, but for one normalisation: a shared
//                 definition the prebuilt package names by a historical name
//                 (see KNOWN RENAMES below) is mapped back to it first.
//   tail          BYTE-IDENTICAL, with the two schema constants cut out: the
//                 Type URIs, EXTENDS, DEPRECATION, the Payload / Response
//                 aliases, SPEC and RESPONSE_SPEC.
//   PAYLOAD_SCHEMA, RESPONSE_PAYLOAD_SCHEMA
//                 JSON-EQUAL, with every `properties` object and every root
//                 `$defs` in the same member order. Not byte-identical,
//                 unavoidably: a bundle is RFC 8785 canonical (members sorted),
//                 and `propertyOrder` / `defsOrder` restore only those two
//                 orders — the only ones a generated type can observe.
//   _shared/components.ts
//                 Declaration by declaration, BYTE-IDENTICAL (same
//                 normalisation). The prebuilt module may declare more: the
//                 in-repo generator also hoists definitions only a shared
//                 schema file reaches, which no payload schema — and so no
//                 bundle — carries. Each such declaration must be one no
//                 prebuilt spec module imports.
//   index.ts      The `export * as <Namespace> from "<path>"` lines must equal
//                 the prebuilt barrel's for the same modules (whose deprecated
//                 `export import` spelling is read back as `export * as`).
//   _extends.ts   Byte-identical after the banner.
//   tsc           The generated tree, type-checked whole, as an application
//                 would.
//
// KNOWN RENAMES. The in-repo generator never renames a hoisted definition once
// published (trust-tasks-ts/component-names.lock.json): a name that shipped
// bare keeps it after a second shape of it appears. A bundle cannot carry that
// history, so trust-tasks-gen-ts qualifies every shape of such a name. Each
// rename is checked rather than assumed: the prebuilt name must be locked bare
// while another shape of it is locked qualified, and the two declarations must
// be byte-identical but for the name.
//
// Exit 1 on any difference not explained above.

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { generate } from "../dist/index.js";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, "..", "..");
const PREBUILT = path.join(REPO, "trust-tasks-ts", "src");

const args = process.argv.slice(2);
const flag = (name) => {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : undefined;
};
const verbose = args.includes("--verbose");
const work = path.resolve(flag("--work") ?? fs.mkdtempSync(path.join(process.env.RUNNER_TEMP ?? "/tmp", "tt-ts-equivalence-")));
let bundles = flag("--bundles");
if (!bundles) {
  bundles = path.join(work, "registry-bundles");
  console.log(`building the registry bundles into ${bundles} …`);
  execFileSync("cargo", ["run", "-q", "--release", "-p", "trust-tasks-codegen", "--", "--bundles", bundles], {
    cwd: REPO,
    stdio: "inherit",
  });
}
bundles = path.resolve(bundles);

// ── 1. An application that locks every version in the registry ─────────────

const index = JSON.parse(fs.readFileSync(path.join(bundles, "index.json"), "utf8"));
const app = path.join(work, "app");
fs.rmSync(app, { recursive: true, force: true });
const lockLines = [
  "# every registry version — the equivalence test — plus one third-party fixture",
  "version = 1",
  'registry = "https://trusttasks.org"',
  "",
  "[sources.acme]",
  'url = "https://tasks.example.com"',
];
for (const spec of index.specs) {
  const vendored = path.join(app, "trust-tasks", ...spec.slug.split("/"), `${spec.version}.bundle.json`);
  fs.mkdirSync(path.dirname(vendored), { recursive: true });
  fs.copyFileSync(path.join(bundles, spec.path), vendored);
  lockLines.push("", "[[task]]", `slug = "${spec.slug}"`, `version = "${spec.version}"`, `type_uri = "${spec.typeUri}"`, `digest = "${spec.digest}"`);
}
// A SPEC §6.5 private specification from a third-party source: proves the
// vendored path (`@acme/…`), the source-named directory and namespace, and
// that it generates beside the whole registry without colliding.
const FIXTURE = path.join(HERE, "..", "test", "fixtures", "acme", "payments", "settle", "1.0.bundle.json");
const fixtureBytes = fs.readFileSync(FIXTURE);
const fixture = JSON.parse(fixtureBytes.toString("utf8"));
const vendoredFixture = path.join(app, "trust-tasks", "@acme", "payments", "settle", "1.0.bundle.json");
fs.mkdirSync(path.dirname(vendoredFixture), { recursive: true });
fs.copyFileSync(FIXTURE, vendoredFixture);
const fixtureDigest = `sha256:${(await import("node:crypto")).createHash("sha256").update(fixtureBytes).digest("hex")}`;
lockLines.push("", "[[task]]", 'source = "acme"', `slug = "${fixture.slug}"`, `version = "${fixture.version}"`, `type_uri = "${fixture.typeUri}"`, `digest = "${fixtureDigest}"`);
fs.writeFileSync(path.join(app, "trust-tasks.lock"), lockLines.join("\n") + "\n");

const out = path.join(app, "generated");
const started = Date.now();
const result = await generate({ lock: path.join(app, "trust-tasks.lock"), out, includeFramework: true, log: () => {} });
console.log(`generated ${result.modules.length} modules from ${index.specs.length} bundles in ${((Date.now() - started) / 1000).toFixed(1)} s`);

// ── 2. Pair them with the prebuilt modules ──────────────────────────────────

const failures = [];
const fail = (what) => failures.push(what);

function walk(dir, acc = []) {
  for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) walk(p, acc);
    else if (e.name === "payload.ts") acc.push(p);
  }
  return acc;
}
const relModule = (root, p) => path.relative(root, p).split(path.sep).join("/");
const prebuiltModules = new Set(
  walk(PREBUILT)
    .map((p) => relModule(PREBUILT, p))
    .filter((r) => !r.split("/").some((s) => s.startsWith("_")))
    // Modelled by hand in every runtime; the registry publishes no bundle.
    .filter((r) => !r.startsWith("trust-task-error/")),
);
const SOURCED = "acme/payments/settle/1.0/payload.ts";
const generatedModules = new Set(result.modules.map((m) => m.path).filter((p) => p !== SOURCED));
{
  const sourced = path.join(out, ...SOURCED.split("/"));
  if (!fs.existsSync(sourced)) fail(`${SOURCED}: the sourced fixture was not generated where expected`);
  else if (!fs.readFileSync(sourced, "utf8").includes(`export const TYPE_URI = ${JSON.stringify(fixture.typeUri)} as const;`)) {
    fail(`${SOURCED}: TYPE_URI is not the bundle's typeUri`);
  }
  if (!fs.readFileSync(path.join(out, "index.ts"), "utf8").includes('export * as Acme from "./acme/index.js";')) {
    fail("index.ts: no `Acme` namespace for the sourced fixture");
  }
  if (!fs.readFileSync(path.join(out, "acme", "index.ts"), "utf8").includes('export * as PaymentsSettle_v1_0 from "./payments/settle/1.0/payload.js";')) {
    fail("acme/index.ts: does not export the sourced fixture's module");
  }
}
for (const m of prebuiltModules) if (!generatedModules.has(m)) fail(`${m}: prebuilt, not generated`);
for (const m of generatedModules) if (!prebuiltModules.has(m)) fail(`${m}: generated, not prebuilt`);
const paired = [...prebuiltModules].filter((m) => generatedModules.has(m)).sort();

// ── 3. The shared components module ─────────────────────────────────────────

const DECL_LINE = /^export (?:interface|type) ([A-Za-z0-9_$]+)/;

/** Top-level declarations of a compiled module, by name, each with its leading JSDoc. */
function declarationBlocks(text) {
  const lines = text.slice(text.indexOf("*/") + 2).split("\n");
  const blocks = new Map();
  let name = null;
  let body = [];
  let pending = [];
  const flush = () => name !== null && blocks.set(name, body.join("\n").trimEnd());
  for (const line of lines) {
    const decl = DECL_LINE.exec(line);
    if (line.startsWith("/**") && name !== null) {
      flush();
      name = null;
      body = [];
      pending = [line];
    } else if (decl) {
      flush();
      name = decl[1];
      body = [...pending, line];
      pending = [];
    } else if (name === null) pending.push(line);
    else body.push(line);
  }
  flush();
  return blocks;
}

const nameLock = JSON.parse(fs.readFileSync(path.join(REPO, "trust-tasks-ts", "component-names.lock.json"), "utf8"));
const lockedNames = new Set(Object.values(nameLock));
const pComponents = declarationBlocks(fs.readFileSync(path.join(PREBUILT, "_shared", "components.ts"), "utf8"));
const gComponentsFile = path.join(out, "_shared", "components.ts");
const gComponents = fs.existsSync(gComponentsFile) ? declarationBlocks(fs.readFileSync(gComponentsFile, "utf8")) : new Map();
if (gComponents.size === 0) fail("_shared/components.ts: not generated, or declares nothing");

// generated name → prebuilt name, for the historical names (see the header).
const renames = new Map();
const renameReasons = [];
for (const [gName, gBlock] of gComponents) {
  if (pComponents.has(gName)) continue;
  const m = /^(.+?)_([A-Z][A-Za-z0-9]*V\d+_\d+)$/.exec(gName);
  const bare = m?.[1];
  const qualifiedSiblings = bare ? [...lockedNames].filter((n) => n.startsWith(`${bare}_`)) : [];
  if (
    bare &&
    pComponents.has(bare) &&
    !gComponents.has(bare) &&
    lockedNames.has(bare) &&
    qualifiedSiblings.length > 0 &&
    pComponents.get(bare) === gBlock.replace(new RegExp(`\\b${gName}\\b`, "g"), bare)
  ) {
    renames.set(gName, bare);
    renameReasons.push(`${gName} is the prebuilt ${bare}: locked bare in component-names.lock.json, beside ${qualifiedSiblings.join(", ")}`);
  } else fail(`_shared/components.ts: ${gName} is declared only by the generated module`);
}
const normalise = (text) => {
  let t = text;
  for (const [gName, pName] of renames) {
    t = t.replace(new RegExp(`\\b${gName} as ${pName}\\b`, "g"), pName).replace(new RegExp(`\\b${gName}\\b`, "g"), pName);
  }
  return t;
};
let componentsIdentical = 0;
for (const [gName, gBlock] of gComponents) {
  const pName = renames.get(gName) ?? gName;
  if (!pComponents.has(pName)) continue;
  if (normalise(gBlock) === pComponents.get(pName)) componentsIdentical++;
  else fail(`_shared/components.ts: ${gName} is not byte-identical to the prebuilt declaration`);
}

// A prebuilt-only declaration is fine only if no prebuilt spec module uses it.
const IMPORT_RE = /\nimport type \{([^}]*)\} from "[^"]*_shared\/components\.js";\n/;
const importedByPrebuilt = new Set();
for (const rel of prebuiltModules) {
  const m = IMPORT_RE.exec(fs.readFileSync(path.join(PREBUILT, rel), "utf8"));
  for (const clause of m ? m[1].split(",") : []) importedByPrebuilt.add(clause.trim().split(/\s+as\s+/)[0]);
}
const covered = new Set([...gComponents.keys()].map((n) => renames.get(n) ?? n));
const prebuiltOnly = [...pComponents.keys()].filter((n) => !covered.has(n)).sort();
for (const name of prebuiltOnly) {
  if (importedByPrebuilt.has(name)) fail(`_shared/components.ts: ${name} is imported by a prebuilt spec module but not generated`);
}

// ── 4. Module by module ─────────────────────────────────────────────────────

const TAIL_START = "\n/** Trust Task type URI. */";

function split(text, file) {
  const bannerEnd = text.indexOf("*/") + 2;
  const tailAt = text.indexOf(TAIL_START);
  if (bannerEnd < 2 || tailAt < 0) throw new Error(`${file}: cannot find the banner and tail`);
  return { middle: text.slice(bannerEnd, tailAt), tail: text.slice(tailAt) };
}

/** Cut `export const NAME = <json> as const;` out of `tail`, returning both. */
function cutSchema(tail, name) {
  const start = tail.indexOf(`export const ${name} = `);
  if (start < 0) return { tail, json: null, text: null };
  const jsonStart = start + `export const ${name} = `.length;
  const end = tail.indexOf("\n} as const;", jsonStart);
  const text = tail.slice(jsonStart, end + 2);
  return { tail: tail.slice(0, jsonStart) + "<schema>" + tail.slice(end + 2), json: JSON.parse(text), text };
}

function canonical(v) {
  if (Array.isArray(v)) return `[${v.map(canonical).join(",")}]`;
  if (v && typeof v === "object") return `{${Object.keys(v).sort().map((k) => `${JSON.stringify(k)}:${canonical(v[k])}`).join(",")}}`;
  return JSON.stringify(v);
}

/** Pointers of `properties` objects and root `$defs` whose member order differs. */
function memberOrderDiffs(a, b, at = "", out = []) {
  if (Array.isArray(a) && Array.isArray(b)) a.forEach((x, i) => memberOrderDiffs(x, b[i], `${at}/${i}`, out));
  else if (a && b && typeof a === "object" && typeof b === "object") {
    for (const key of at === "" ? ["properties", "$defs"] : ["properties"]) {
      if (a[key] && typeof a[key] === "object" && JSON.stringify(Object.keys(a[key])) !== JSON.stringify(Object.keys(b[key] ?? {}))) {
        out.push(`${at}/${key}`);
      }
    }
    for (const k of Object.keys(a)) memberOrderDiffs(a[k], b[k], `${at}/${k}`, out);
  }
  return out;
}

let middlesIdentical = 0;
let middlesNormalised = 0;
let schemasByteIdentical = 0;
let schemasCompared = 0;
for (const rel of paired) {
  const pText = fs.readFileSync(path.join(PREBUILT, rel), "utf8");
  const gText = fs.readFileSync(path.join(out, rel), "utf8");
  const p = split(pText, rel);
  const g = split(gText, rel);

  if (p.middle === g.middle) middlesIdentical++;
  else if (p.middle === normalise(g.middle)) middlesNormalised++;
  else fail(`${rel}: the declarations (imports, types, re-exports) are not byte-identical`);

  let pTail = p.tail;
  let gTail = g.tail;
  for (const name of ["PAYLOAD_SCHEMA", "RESPONSE_PAYLOAD_SCHEMA"]) {
    const pc = cutSchema(pTail, name);
    const gc = cutSchema(gTail, name);
    pTail = pc.tail;
    gTail = gc.tail;
    if ((pc.json === null) !== (gc.json === null)) {
      fail(`${rel}: ${name} is ${pc.json === null ? "absent from the prebuilt module" : "absent from the generated module"}`);
      continue;
    }
    if (pc.json === null) continue;
    schemasCompared++;
    if (pc.text === gc.text) schemasByteIdentical++;
    if (canonical(pc.json) !== canonical(gc.json)) fail(`${rel}: ${name} is not JSON-equal to the prebuilt`);
    else {
      const order = memberOrderDiffs(pc.json, gc.json);
      if (order.length) fail(`${rel}: ${name} member order differs at ${order.join(", ")}`);
    }
  }
  if (pTail !== gTail) fail(`${rel}: the tail (TYPE_URI … SPEC / RESPONSE_SPEC) is not byte-identical`);
}

// ── 5. The generated tree type-checks whole ─────────────────────────────────

const checkDir = path.join(work, "typecheck");
fs.rmSync(checkDir, { recursive: true, force: true });
fs.mkdirSync(checkDir, { recursive: true });
fs.writeFileSync(
  path.join(checkDir, "tsconfig.json"),
  JSON.stringify(
    {
      compilerOptions: {
        target: "ES2022",
        module: "ESNext",
        moduleResolution: "Bundler",
        strict: true,
        noEmit: true,
        skipLibCheck: true,
        types: [],
        paths: { "@openvtc/trust-tasks/runtime": [path.join(REPO, "trust-tasks-ts", "src", "_runtime", "index.ts")] },
      },
      include: [path.join(out, "**", "*.ts")],
    },
    null,
    2,
  ),
);
const tsc = path.join(REPO, "trust-tasks-ts", "node_modules", "typescript", "bin", "tsc");
const tscStarted = Date.now();
try {
  execFileSync(process.execPath, ["--max-old-space-size=8192", tsc, "-p", path.join(checkDir, "tsconfig.json")], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
} catch (err) {
  for (const line of `${err.stdout ?? ""}${err.stderr ?? ""}`.split("\n").filter((l) => /error TS/.test(l))) fail(`tsc: ${line.trim()}`);
}
console.log(`tsc: the generated tree type-checked in ${((Date.now() - tscStarted) / 1000).toFixed(1)} s`);

// ── 5. index.ts and _extends.ts ─────────────────────────────────────────────

const barrelLines = (text) => text.split("\n").filter((l) => /^export \* as \S+ from "\.\/(?!_)[^"]+\/payload\.js";$/.test(l));
// The prebuilt barrel deprecates every non-framework namespace (milestone 5 of
// docs/design/consumer-side-generation.md) by spelling its re-export
// `import * as _X from "…"; /** @deprecated … */ export import X = _X;` —
// TypeScript ignores @deprecated on `export * as`. Read that form back as the
// `export * as X from "…"` it stands for; the generated barrel, which is the
// replacement, is not deprecated.
const undeprecated = (text) =>
  text.replace(
    /^import \* as _(\S+) from ("[^"]+");\n\/\*\* @deprecated The prebuilt specification modules will be removed: [^\n]*\*\/\nexport import \1 = _\1;$/gm,
    "export * as $1 from $2;",
  );
const pBarrel = barrelLines(undeprecated(fs.readFileSync(path.join(PREBUILT, "index.ts"), "utf8"))).filter((l) => !l.includes('"./trust-task-error/'));
const gBarrel = barrelLines(fs.readFileSync(path.join(out, "index.ts"), "utf8"));
if (pBarrel.join("\n") !== gBarrel.join("\n")) {
  const ps = new Set(pBarrel);
  const gs = new Set(gBarrel);
  fail(`index.ts: namespace exports differ — only prebuilt: ${pBarrel.filter((l) => !gs.has(l)).slice(0, 5).join(" | ") || "none"}; only generated: ${gBarrel.filter((l) => !ps.has(l)).slice(0, 5).join(" | ") || "none"}`);
}
const afterBanner = (text) => text.slice(text.indexOf("*/") + 2);
if (afterBanner(fs.readFileSync(path.join(PREBUILT, "_extends.ts"), "utf8")) !== afterBanner(fs.readFileSync(path.join(out, "_extends.ts"), "utf8"))) {
  fail("_extends.ts: not byte-identical after the banner");
}

// ── Report ──────────────────────────────────────────────────────────────────

console.log("");
console.log(`modules compared:                          ${paired.length}`);
console.log(`  declarations byte-identical:             ${middlesIdentical}`);
console.log(`  … after the known renames:               ${middlesNormalised}`);
console.log(`  tails byte-identical (schemas aside):    ${paired.length - failures.filter((f) => f.includes("the tail")).length}`);
console.log(`  schema constants JSON-equal, member order equal: ${schemasCompared} (byte-identical: ${schemasByteIdentical})`);
console.log(`_shared/components.ts declarations:        ${gComponents.size} generated, ${pComponents.size} prebuilt`);
console.log(`  byte-identical:                          ${componentsIdentical - renames.size}`);
console.log(`  … after the known renames:               ${renames.size}`);
for (const r of renameReasons) console.log(`    · ${r}`);
console.log(`  prebuilt only, reached by no payload schema: ${prebuiltOnly.length}${prebuiltOnly.length ? ` (${prebuiltOnly.join(", ")})` : ""}`);
console.log(`sourced fixture (@acme ${fixture.typeUri}): generated at acme/…, namespace Acme`);
if (failures.length) {
  console.log(`\n${failures.length} difference(s):`);
  for (const f of failures.slice(0, verbose ? Infinity : 200)) console.log(`  ✗ ${f}`);
  if (!verbose && failures.length > 200) console.log(`  … ${failures.length - 200} more (--verbose)`);
  process.exit(1);
}
console.log("\nequivalent ✓");
