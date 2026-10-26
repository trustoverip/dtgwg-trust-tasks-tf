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
// ── What is compared, region by region ──────────────────────────────────────
//
// A module is banner · [imports] · declarations · [re-exports] · tail.
//
//   banner        NOT compared: it names the generator and the source.
//   tail          BYTE-IDENTICAL, with the two schema constants cut out: the
//                 Type URIs, EXTENDS, DEPRECATION, the Payload / Response
//                 aliases, SPEC and RESPONSE_SPEC.
//   PAYLOAD_SCHEMA, RESPONSE_PAYLOAD_SCHEMA
//                 JSON-EQUAL, and every `properties` object in the same member
//                 order. Not byte-identical, unavoidably: a bundle is
//                 RFC 8785 canonical (members sorted), and `propertyOrder`
//                 restores the authored order of `properties` only.
//   declarations  The payload/response root declarations are compared BYTE
//                 for byte with the prebuilt module's own declarations. Every
//                 type the two modules export (declared, re-exported, and the
//                 `Payload` / `Response` aliases) is compared by TYPE IDENTITY
//                 — `tsc` checks `Equal<Prebuilt.X, Generated.X>`, the
//                 identity relation: same members, same optionality and
//                 readonly-ness, same unions, recursively; doc comments aside.
//                 The definitions a prebuilt module imports from the hoisted
//                 `_shared/components.ts` are declared in the generated module
//                 itself (a bundle does not say which `$defs` were shared —
//                 see src/emit.ts), so those are compared only this way.
//   exports       The set of type names each module exports must be equal.
//   index.ts      The `export * as <Namespace> from "<path>"` lines must equal
//                 the prebuilt barrel's for the same modules.
//   _extends.ts   Byte-identical after the banner.
//
// Exit 1 on any difference outside "declarations not byte-identical", which is
// reported (with the reason) rather than failed: it is the expected cost of
// declaring a schema's own `$defs` the same way as the shared ones.

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
const notes = { declarationsDiffer: [], counterDuplicates: [], extraDefinitions: [], assignableOnly: [] };

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

// ── 3. Region by region ─────────────────────────────────────────────────────

const TAIL_START = "\n/** Trust Task type URI. */";
const DEFINITIONS_MARK = "\n/* ── Definitions the payload schema references, declared once";
const REEXPORT_RE = /\n\/\*\* Shared definitions this specification references[^\n]*\*\/\nexport type \{ ([^}]*) \};\n/;
const IMPORT_RE = /\nimport type \{[^}]*\} from "[^"]*_shared\/components\.js";\n/;
const DECL_RE = /^export (?:interface|type) ([A-Za-z0-9_$]+)/gm;

function split(text, file) {
  const bannerEnd = text.indexOf("*/") + 2;
  const tailAt = text.indexOf(TAIL_START);
  if (bannerEnd < 2 || tailAt < 0) throw new Error(`${file}: cannot find the banner and tail`);
  return { middle: text.slice(bannerEnd, tailAt), tail: text.slice(tailAt) };
}

/** Cut `export const NAME = <json> as const;` out of `tail`, returning both. */
function cutSchema(tail, name) {
  const start = tail.indexOf(`export const ${name} = `);
  if (start < 0) return { tail, json: null };
  const jsonStart = start + `export const ${name} = `.length;
  const end = tail.indexOf("\n} as const;", jsonStart);
  const json = JSON.parse(tail.slice(jsonStart, end + 2));
  return { tail: tail.slice(0, jsonStart) + "<schema>" + tail.slice(end + 2), json };
}

function canonical(v) {
  if (Array.isArray(v)) return `[${v.map(canonical).join(",")}]`;
  if (v && typeof v === "object") return `{${Object.keys(v).sort().map((k) => `${JSON.stringify(k)}:${canonical(v[k])}`).join(",")}}`;
  return JSON.stringify(v);
}

/** Pointers of `properties` objects whose member order differs. */
function propertyOrderDiffs(a, b, at = "", out = []) {
  if (Array.isArray(a) && Array.isArray(b)) a.forEach((x, i) => propertyOrderDiffs(x, b[i], `${at}/${i}`, out));
  else if (a && b && typeof a === "object" && typeof b === "object") {
    if (a.properties && typeof a.properties === "object" && JSON.stringify(Object.keys(a.properties)) !== JSON.stringify(Object.keys(b.properties ?? {}))) {
      out.push(`${at}/properties`);
    }
    for (const k of Object.keys(a)) propertyOrderDiffs(a[k], b[k], `${at}/${k}`, out);
  }
  return out;
}

const typeChecks = []; // { module, names }
for (const rel of paired) {
  const pText = fs.readFileSync(path.join(PREBUILT, rel), "utf8");
  const gText = fs.readFileSync(path.join(out, rel), "utf8");
  const p = split(pText, rel);
  const g = split(gText, rel);

  // Tail, schemas cut out.
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
    if (canonical(pc.json) !== canonical(gc.json)) fail(`${rel}: ${name} is not JSON-equal to the prebuilt`);
    else {
      const order = propertyOrderDiffs(pc.json, gc.json);
      if (order.length) fail(`${rel}: ${name} member order differs at ${order.join(", ")}`);
    }
  }
  if (pTail !== gTail) fail(`${rel}: the tail (TYPE_URI … SPEC / RESPONSE_SPEC) is not byte-identical`);

  // Declarations.
  const reexport = REEXPORT_RE.exec(p.middle);
  const reexported = reexport ? reexport[1].split(",").map((s) => s.trim()).filter(Boolean) : [];
  const pDecls = p.middle.replace(IMPORT_RE, "\n").replace(REEXPORT_RE, "\n");
  const mark = g.middle.indexOf(DEFINITIONS_MARK);
  const gRoot = mark < 0 ? g.middle : g.middle.slice(0, mark);
  if (pDecls.trim() !== gRoot.trim()) notes.declarationsDiffer.push(rel);

  const pNames = new Set([...[...pDecls.matchAll(DECL_RE)].map((m) => m[1]), ...reexported]);
  const gNames = new Set([...g.middle.matchAll(DECL_RE)].map((m) => m[1]));
  for (const tailName of ["Payload", "Response"]) {
    if (new RegExp(`^export type ${tailName} = `, "m").test(p.tail)) pNames.add(tailName);
    if (new RegExp(`^export type ${tailName} = `, "m").test(g.tail)) gNames.add(tailName);
  }
  const onlyP = [...pNames].filter((n) => !gNames.has(n)).sort();
  const onlyG = [...gNames].filter((n) => !pNames.has(n)).sort();
  const pairs = [...pNames].filter((n) => gNames.has(n)).sort().map((n) => [n, n]);
  // Two expected export-set differences, each still type-checked:
  //  - a counter-suffixed duplicate the compiler minted in the prebuilt module
  //    (`DidTemplateRef1`) for a schema's own definition, which the generated
  //    module declares once — compared with that one declaration;
  //  - a definition the generated module declares once that the prebuilt
  //    module inlined at its use (`Did`, a `$defs` string with no title).
  const definitionNames = new Set(mark < 0 ? [] : [...g.middle.slice(mark).matchAll(DECL_RE)].map((m) => m[1]));
  for (const name of onlyP) {
    const base = /^(.*?)\d+$/.exec(name)?.[1];
    if (base && gNames.has(base) && pNames.has(base)) {
      notes.counterDuplicates.push(`${rel}: ${name} (prebuilt) is ${base} (generated)`);
      pairs.push([name, base]);
    } else fail(`${rel}: exported only by the prebuilt module: ${name}`);
  }
  for (const name of onlyG) {
    if (definitionNames.has(name)) notes.extraDefinitions.push(`${rel}: ${name}`);
    else fail(`${rel}: exported only by the generated module: ${name}`);
  }
  typeChecks.push({ rel, pairs });
}

// ── 4. Type identity, with tsc ──────────────────────────────────────────────

const checkDir = path.join(work, "typecheck");
fs.rmSync(checkDir, { recursive: true, force: true });
fs.mkdirSync(checkDir, { recursive: true });
const checkLines = [
  "type Equal<A, B> = (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false;",
  "type Assignable<A, B> = [A] extends [B] ? ([B] extends [A] ? true : false) : false;",
  "",
];
// line number (1-based) → { what: "module: Name", kind: "identity" | "assignable" }
const lineOwner = new Map();
typeChecks.forEach(({ rel }, i) => {
  const spec = (root) => JSON.stringify(path.join(root, rel).replace(/\.ts$/, ".js"));
  checkLines.push(`import type * as P${i} from ${spec(PREBUILT)};`, `import type * as G${i} from ${spec(out)};`);
});
checkLines.push("");
let assertions = 0;
typeChecks.forEach(({ rel, pairs }, i) => {
  for (const [pName, gName] of pairs) {
    const what = pName === gName ? `${rel}: ${pName}` : `${rel}: ${pName} (prebuilt) / ${gName} (generated)`;
    checkLines.push(`export const c${assertions}: Equal<P${i}.${pName}, G${i}.${gName}> = true;`);
    lineOwner.set(checkLines.length, { what, kind: "identity" });
    checkLines.push(`export const a${assertions++}: Assignable<P${i}.${pName}, G${i}.${gName}> = true;`);
    lineOwner.set(checkLines.length, { what, kind: "assignable" });
  }
});
// A self-test the comparison can fail: two different shapes must not compare equal.
checkLines.push(
  "interface SelfTestA { a: string; b?: number }",
  "interface SelfTestB { a: string; b: number }",
  "export const selfTestDiffers: Equal<SelfTestA, SelfTestB> = false;",
  "export const selfTestSame: Equal<SelfTestA, { a: string; b?: number }> = true;",
  "",
);
fs.writeFileSync(path.join(checkDir, "equivalence.ts"), checkLines.join("\n"));
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
      // The generated tree whole (index.ts and _extends.ts included), so it is
      // type-checked as an application would, as well as compared.
      include: ["equivalence.ts", path.join(out, "**", "*.ts")],
    },
    null,
    2,
  ),
);
const tsc = path.join(REPO, "trust-tasks-ts", "node_modules", "typescript", "bin", "tsc");
let tscOutput = "";
const tscStarted = Date.now();
try {
  execFileSync(process.execPath, ["--max-old-space-size=8192", tsc, "-p", path.join(checkDir, "tsconfig.json")], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
} catch (err) {
  tscOutput = `${err.stdout ?? ""}${err.stderr ?? ""}`;
}
console.log(`tsc: ${assertions} type-identity assertions over ${typeChecks.length} module pairs in ${((Date.now() - tscStarted) / 1000).toFixed(1)} s`);
// An identity failure whose mutual-assignability check passes is a difference
// in how the compiler spelled one type (see the report); one whose
// assignability fails too is a real difference in what the type accepts.
const identityFailed = new Set();
const assignableFailed = new Set();
for (const line of tscOutput.split("\n").filter((l) => /error TS/.test(l))) {
  const m = /equivalence\.ts\((\d+),\d+\)/.exec(line);
  const owner = m ? lineOwner.get(Number(m[1])) : undefined;
  if (owner) (owner.kind === "identity" ? identityFailed : assignableFailed).add(owner.what);
  else fail(`tsc: ${line.trim()}`);
}
for (const what of assignableFailed) fail(`${what}: not mutually assignable with the prebuilt module's type`);
for (const what of identityFailed) {
  if (!assignableFailed.has(what)) notes.assignableOnly.push(what);
}

// ── 5. index.ts and _extends.ts ─────────────────────────────────────────────

const barrelLines = (text) => text.split("\n").filter((l) => /^export \* as \S+ from "\.\/(?!_)[^"]+\/payload\.js";$/.test(l));
const pBarrel = barrelLines(fs.readFileSync(path.join(PREBUILT, "index.ts"), "utf8")).filter((l) => !l.includes('"./trust-task-error/'));
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
console.log(`modules compared:                      ${paired.length}`);
console.log(`tails byte-identical (schemas aside):  ${paired.length - failures.filter((f) => f.includes("tail")).length}`);
console.log(`root declarations byte-identical:      ${paired.length - notes.declarationsDiffer.length}`);
console.log(`root declarations differing in text:   ${notes.declarationsDiffer.length} (reported, not failed — type identity is what is held)`);
if (verbose) for (const m of notes.declarationsDiffer) console.log(`  · ${m}`);
console.log(`types compared (identity + assignability): ${assertions}`);
console.log(`  identical:                           ${assertions - notes.assignableOnly.length - assignableFailed.size}`);
console.log(`  mutually assignable, not identical:  ${notes.assignableOnly.length}`);
for (const m of notes.assignableOnly) console.log(`    · ${m}`);
console.log(`counter-suffixed prebuilt duplicates compared with their one generated declaration: ${notes.counterDuplicates.length}`);
if (verbose) for (const m of notes.counterDuplicates) console.log(`    · ${m}`);
console.log(`definitions the generated module declares that the prebuilt inlined: ${notes.extraDefinitions.length}`);
if (verbose) for (const m of notes.extraDefinitions) console.log(`    · ${m}`);
console.log(`sourced fixture (@acme ${fixture.typeUri}): generated at acme/…, namespace Acme`);
if (failures.length) {
  console.log(`\n${failures.length} difference(s):`);
  for (const f of failures.slice(0, verbose ? Infinity : 200)) console.log(`  ✗ ${f}`);
  if (!verbose && failures.length > 200) console.log(`  … ${failures.length - 200} more (--verbose)`);
  process.exit(1);
}
console.log("\nequivalent ✓");
