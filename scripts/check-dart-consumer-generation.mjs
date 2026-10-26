// Equivalence check: `dart run trust_tasks:generate` (the consumer-side
// generator, input: bundles) against the prebuilt tree under
// trust-tasks-dart/lib/specs/ (scripts/build-dart-bindings.mjs, input: specs/).
//
//   npm run build-bundles                       # bundles + index into website/specs
//   node scripts/check-dart-consumer-generation.mjs [<bundles dir>]
//
// Writes a lock pinning EVERY bundle in the index, vendors them beside it, runs
// the Dart generator over it, and compares each generated library with its
// prebuilt counterpart. Exits 1 on any difference outside the two the bundle
// format makes unavoidable, and says exactly what remains:
//
//   1. The embedded schema text (`payloadSchemaJson`, `responsePayloadSchemaJson`).
//      The bundle's schema is key-sorted (RFC 8785); the prebuilt embeds the
//      authored keyword order. Compared as parsed JSON — must be equal.
//   2. The ORDER of top-level declarations, in one case only. The bundle's
//      `defsOrder` gives the root `$defs` in the order the .mjs's inliner
//      leaves them, and both generators then declare them dependencies-first,
//      walking each definition's `$ref`s in the order its keywords are written.
//      That keyword order is not in the bundle (it is key-sorted, and
//      `propertyOrder` covers `properties` only). It decides the outcome only
//      where one schema object reaches two not-yet-declared definitions through
//      sibling keywords — in the registry, a map whose `propertyNames` and
//      `additionalProperties` each `$ref` a definition, written in either order
//      (both occur). A library may differ in declaration order only when its
//      bundle has such an object; each declaration — every class, its fields
//      in order, its fromJson/toJson — must still be byte-identical, and the
//      multiset of declarations equal. A bundle without `defsOrder` (older than
//      it) is a failure here: rebuild the bundles.
//
// The two header lines (generator and source) differ by design and are not
// compared. Libraries the consumer generator does not write are listed, not
// compared: shared-definition libraries (inlined into each payload library),
// the framework specifications the package ships, and withdrawn versions.

import { promises as fs } from "node:fs";
import fsSync from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const PACKAGE_ROOT = path.join(REPO_ROOT, "trust-tasks-dart");
const PREBUILT = path.join(PACKAGE_ROOT, "lib", "specs");
const bundlesDir = path.resolve(process.argv[2] ?? path.join(REPO_ROOT, "website", "specs"));

const index = JSON.parse(await fs.readFile(path.join(bundlesDir, "index.json"), "utf8"));
const work = await fs.mkdtemp(path.join(os.tmpdir(), "tt-dart-equiv-"));
const outDir = path.join(work, "out");

// ── A lock of every bundle, vendored beside it ─────────────────────────────
let lock = "# Written by check-dart-consumer-generation.mjs.\nversion = 1\nregistry = \"https://trusttasks.org\"\n";
for (const s of index.specs) {
  const dest = path.join(work, "trust-tasks", ...s.slug.split("/"), `${s.version}.bundle.json`);
  await fs.mkdir(path.dirname(dest), { recursive: true });
  await fs.copyFile(path.join(bundlesDir, s.path), dest);
  lock +=
    `\n[[task]]\nslug = ${JSON.stringify(s.slug)}\nversion = ${JSON.stringify(s.version)}\n` +
    `type_uri = ${JSON.stringify(s.typeUri)}\ndigest = ${JSON.stringify(s.digest)}\n`;
}
await fs.writeFile(path.join(work, "trust-tasks.lock"), lock);

const t0 = Date.now();
const run = spawnSync(
  "dart",
  ["run", "trust_tasks:generate", "--lock", path.join(work, "trust-tasks.lock"), "--out", outDir],
  { cwd: PACKAGE_ROOT, encoding: "utf8" },
);
if (run.status !== 0) {
  console.error(run.stdout, run.stderr);
  process.exit(1);
}
const seconds = ((Date.now() - t0) / 1000).toFixed(1);
const notes = run.stderr.split("\n").filter((l) => l.startsWith("note: "));

// ── Comparison ─────────────────────────────────────────────────────────────
async function walk(dir) {
  const out = [];
  for (const e of await fs.readdir(dir, { withFileTypes: true })) {
    const full = path.join(dir, e.name);
    if (e.isDirectory()) out.push(...(await walk(full)));
    else if (e.name.endsWith(".dart")) out.push(full);
  }
  return out;
}

/** Decode a single-quoted Dart string literal as the generator writes them. */
function dartLiteral(src) {
  let out = "";
  for (let i = 1; i < src.length - 1; i++) {
    const ch = src[i];
    if (ch !== "\\") {
      out += ch;
      continue;
    }
    const esc = src[++i];
    if (esc === "n") out += "\n";
    else if (esc === "r") out += "\r";
    else if (esc === "t") out += "\t";
    else if (esc === "x") {
      out += String.fromCharCode(parseInt(src.slice(i + 1, i + 3), 16));
      i += 2;
    } else out += esc;
  }
  return out;
}

function deepEqual(a, b) {
  if (a === b) return true;
  if (typeof a !== typeof b || a === null || b === null || typeof a !== "object") return false;
  if (Array.isArray(a) !== Array.isArray(b)) return false;
  const ka = Object.keys(a);
  const kb = Object.keys(b);
  if (ka.length !== kb.length) return false;
  return ka.every((k) => Object.hasOwn(b, k) && deepEqual(a[k], b[k]));
}

const SCHEMA_CONST = /^(const String (?:response)?[pP]ayloadSchemaJson =\s*)('(?:[^'\\]|\\.)*')(;)$/m;

/** Top-level blocks after the two header lines, schema text pulled out. */
function blocks(text) {
  const body = text.split("\n").slice(2).join("\n").replace(/^\n+/, "");
  const schemas = [];
  const parts = body.split(/\n\n(?=\S)/).map((b) =>
    b.replace(SCHEMA_CONST, (_, pre, lit, post) => {
      schemas.push(JSON.parse(dartLiteral(lit)));
      return `${pre}<schema ${schemas.length - 1}>${post}`;
    }),
  );
  return { parts, schemas };
}

// The bundle behind each generated library, to judge a reordering against.
const bundleFor = new Map();
for (const s of index.specs) {
  const dirs = s.slug.split("/").map((p) => p.replace(/^_/, "").replace(/[^A-Za-z0-9]+/g, "_"));
  bundleFor.set([...dirs, `v${s.version.replace(".", "_")}`, "payload.dart"].join("/"), s);
}

/** Does [node] hold an object whose `propertyNames` and `additionalProperties` both `$ref` a definition? */
function hasKeyedMapOfRefs(node) {
  if (Array.isArray(node)) return node.some(hasKeyedMapOfRefs);
  if (node === null || typeof node !== "object") return false;
  const refs = (n) => JSON.stringify(n ?? null).includes('"$ref":"#/$defs/');
  if (refs(node.propertyNames) && typeof node.additionalProperties === "object" && refs(node.additionalProperties)) {
    return true;
  }
  return Object.values(node).some(hasKeyedMapOfRefs);
}

const generated = (await walk(outDir)).map((f) => path.relative(outDir, f)).sort();
const prebuilt = new Set((await walk(PREBUILT)).map((f) => path.relative(PREBUILT, f)));

let identical = 0;
const reordered = [];
const declName = (p) => (p.match(/^(?:class|typedef|extension type const) (\w+)/m) ?? [])[1];
const failures = [];
for (const rel of generated) {
  if (rel === "extends_index.dart") continue;
  if (!prebuilt.has(rel)) {
    failures.push(`${rel}: generated, but there is no prebuilt library at that path`);
    continue;
  }
  const mine = blocks(await fs.readFile(path.join(outDir, rel), "utf8"));
  const theirs = blocks(await fs.readFile(path.join(PREBUILT, rel), "utf8"));
  prebuilt.delete(rel);

  const schemaProblems = [];
  if (mine.schemas.length !== theirs.schemas.length) {
    schemaProblems.push(`${mine.schemas.length} embedded schemas, prebuilt has ${theirs.schemas.length}`);
  } else {
    mine.schemas.forEach((s, i) => {
      if (!deepEqual(s, theirs.schemas[i])) schemaProblems.push(`embedded schema ${i} is not JSON-equal`);
    });
  }

  const sameSequence =
    mine.parts.length === theirs.parts.length && mine.parts.every((p, i) => p === theirs.parts[i]);
  const sorted = (a) => [...a].sort();
  const sameMultiset =
    mine.parts.length === theirs.parts.length &&
    sorted(mine.parts).every((p, i) => p === sorted(theirs.parts)[i]);

  if (schemaProblems.length > 0 || !sameMultiset) {
    const missing = theirs.parts.filter((p) => !mine.parts.includes(p));
    const extra = mine.parts.filter((p) => !theirs.parts.includes(p));
    failures.push(
      [
        `${rel}:`,
        ...schemaProblems.map((p) => `  ${p}`),
        ...missing.map((p) => `  - prebuilt only:\n${p.split("\n").slice(0, 12).map((l) => `      ${l}`).join("\n")}`),
        ...extra.map((p) => `  + generated only:\n${p.split("\n").slice(0, 12).map((l) => `      ${l}`).join("\n")}`),
      ].join("\n"),
    );
  } else if (sameSequence) identical++;
  else {
    const spec = bundleFor.get(rel);
    const bundle = spec && JSON.parse(await fs.readFile(path.join(bundlesDir, spec.path), "utf8"));
    if (!bundle || !Array.isArray(bundle.defsOrder?.["/schema"])) {
      failures.push(`${rel}: declarations in a different order, and its bundle has no \`defsOrder\` — rebuild the bundles`);
    } else if (!hasKeyedMapOfRefs(bundle.schema)) {
      failures.push(
        `${rel}: declarations in a different order, with no propertyNames/additionalProperties pair to explain it\n` +
          `  generated: ${mine.parts.map(declName).filter(Boolean).join(", ")}\n` +
          `  prebuilt:  ${theirs.parts.map(declName).filter(Boolean).join(", ")}`,
      );
    } else reordered.push(rel);
  }
}

// The extends index: same entries, restricted to what was generated.
const entries = (text) =>
  [...text.matchAll(/^  ('(?:[^'\\]|\\.)*'): ('(?:[^'\\]|\\.)*'),$/gm)].map((m) => `${m[1]}: ${m[2]}`);
const myIndex = entries(await fs.readFile(path.join(outDir, "extends_index.dart"), "utf8"));
const theirIndex = entries(await fs.readFile(path.join(PREBUILT, "extends_index.dart"), "utf8"));
prebuilt.delete("extends_index.dart");
if (myIndex.join("\n") !== theirIndex.join("\n")) {
  failures.push(
    `extends_index.dart: entries differ\n  generated: ${myIndex.join(", ")}\n  prebuilt:  ${theirIndex.join(", ")}`,
  );
}

const notGenerated = [...prebuilt].sort();
const isShared = (p) => /(^|\/)(shared|framework)\//.test(p) || !p.endsWith("/payload.dart");
const isFramework = (p) => /^trust_(task|ceremony)/.test(p);
const shared = notGenerated.filter(isShared);
const framework = notGenerated.filter((p) => !isShared(p) && isFramework(p));
const other = notGenerated.filter((p) => !isShared(p) && !isFramework(p));
for (const p of other) failures.push(`${p}: prebuilt, but the consumer generator wrote nothing for it`);

console.log(`Dart consumer generation vs the prebuilt tree (${index.specs.length} bundles, generated in ${seconds} s)`);
console.log(`  ${identical} libraries byte-identical apart from the header and the embedded schema's key order (JSON-equal)`);
console.log(
  `  ${reordered.length} more identical declaration for declaration, in a different top-level order ` +
    `(a propertyNames/additionalProperties pair whose authored keyword order the bundle does not carry):`,
);
for (const rel of reordered) console.log(`      ${rel}`);
console.log(`  extends_index.dart: ${myIndex.length} entries, ${myIndex.join("\n") === theirIndex.join("\n") ? "equal" : "DIFFERENT"}`);
console.log(`  not generated by design: ${shared.length} shared-definition libraries, ${framework.length} framework libraries`);
console.log(`  skipped with a note: ${notes.length} (${notes.filter((n) => n.includes("withdrawn")).length} withdrawn, ${notes.filter((n) => n.includes("framework")).length} framework)`);
if (failures.length > 0) {
  console.error(`\n${failures.length} unexplained difference(s):\n`);
  for (const f of failures.slice(0, 40)) console.error(f);
  if (failures.length > 40) console.error(`… and ${failures.length - 40} more`);
  process.exitCode = 1;
} else {
  console.log("\nOK — no differences beyond the two the bundle format makes unavoidable.");
}
await fs.rm(work, { recursive: true, force: true });
