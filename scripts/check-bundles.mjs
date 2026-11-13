#!/usr/bin/env node
// Check the registry bundles `trust-tasks-codegen --bundles` wrote.
//
//   node scripts/check-bundles.mjs [<bundles-dir>]     (default: website/specs)
//
// A bundle (`<dir>/<slug>/<M.N>/bundle.json`) is one specification version's
// self-contained schemas and policy, and `<dir>/index.json` names every bundle
// by SHA-256 digest (docs/design/consumer-side-generation.md, milestone 1).
// They are built by the Rust generator, from the same code that builds
// `trust-tasks-rs`. This script checks them from the other side, sharing no
// code with it:
//
//   1. every digest in the index is `sha256:` of the bytes of the file it names;
//   2. every bundle is in canonical form — re-serializing the parsed bundle
//      with sorted keys (JSON.stringify is the basis of RFC 8785) reproduces it
//      byte for byte, so a consumer re-canonicalizing gets the same digest;
//   3. the index lists exactly the bundles on disk, and each entry's slug,
//      version, status, deprecation and extends match the bundle;
//   4. every non-withdrawn bundle's `schema` / `responseSchema`, pruned to the
//      `$defs` its root reaches, equal, as parsed JSON, the TypeScript
//      generated module's `PAYLOAD_SCHEMA` / `RESPONSE_PAYLOAD_SCHEMA` — an
//      independent generator's inlining of the same `$ref`s
//      (scripts/build-ts-bindings.mjs). The bundle carries each side's full
//      `$defs` and the generators prune what they emit (#1098); the pruning is
//      re-derived here, not imported. A withdrawn version has no TypeScript
//      module, so only 1–3 and 6 apply to it;
//   5. every bundle's `defsOrder` names exactly the `$defs` of the schema it
//      points at, and for a non-withdrawn bundle, restricted to the pruned
//      `$defs`, is the order of the `$defs` in that TypeScript module's
//      schemas, which its inliner leaves in the order the Go and Dart
//      generators declare them;
//   6. `defOrigins` names an origin for exactly the `$defs` the specification's
//      own `payload.schema.json` does not declare, and each origin is
//      `<$id>#/$defs/<name>` of a shared schema in `specs/` that defines that
//      name (or, for a shared file with no `$id`, its path under
//      `https://trusttasks.org/spec/`).
//
// Run after the bundles are built: `npm run build-bundles && npm run check-bundles`.

import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const DIR = path.resolve(process.argv[2] ?? path.join(ROOT, "website", "specs"));
const TS_SRC = path.join(ROOT, "trust-tasks-ts", "src");

const SPECS = path.join(ROOT, "specs");

/** `$id` → the `$defs` names it declares, over every shared schema in `specs/`. */
function sharedDefinitions() {
  const out = new Map();
  const walk = (dir) => {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const p = path.join(dir, entry.name);
      if (entry.isDirectory()) walk(p);
      else if (entry.name.endsWith(".json") && path.relative(SPECS, p).split(path.sep).some((s) => s.startsWith("_"))) {
        const doc = JSON.parse(fs.readFileSync(p, "utf8"));
        const id =
          doc.$id ??
          `https://trusttasks.org/spec/${path.relative(SPECS, p).split(path.sep).join("/").replace(/(\.schema)?\.json$/, "")}`;
        out.set(id, new Set(Object.keys(doc.$defs ?? {})));
      }
    }
  };
  walk(SPECS);
  return out;
}
const SHARED = sharedDefinitions();

const failures = [];
const fail = (msg) => failures.push(msg);

/** Sorted-key compact JSON — the RFC 8785 form for the values bundles hold. */
function canonical(value) {
  if (Array.isArray(value)) return `[${value.map(canonical).join(",")}]`;
  if (value && typeof value === "object") {
    // Default sort compares UTF-16 code units, which is what JCS specifies.
    const keys = Object.keys(value).sort();
    return `{${keys.map((k) => `${JSON.stringify(k)}:${canonical(value[k])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

/**
 * Parse `export const <name>: <type> = { … };` (or the older
 * `export const <name> = { … } as const;`) out of a generated TS module.
 * Brace-counting with string literals skipped, so a brace inside a `pattern`
 * or a description does not throw the count off.
 */
function tsConst(src, name) {
  const m = new RegExp(`^export const ${name}(?::[^=\\n]+)? = \\{`, "m").exec(src);
  const start = m ? m.index : -1;
  if (start < 0) return undefined;
  const open = start + m[0].length - 1;
  let depth = 0;
  let inString = false;
  let escaped = false;
  for (let i = open; i < src.length; i++) {
    const ch = src[i];
    if (inString) {
      if (escaped) escaped = false;
      else if (ch === "\\") escaped = true;
      else if (ch === '"') inString = false;
      continue;
    }
    if (ch === '"') inString = true;
    else if (ch === "{") depth++;
    else if (ch === "}" && --depth === 0) return JSON.parse(src.slice(open, i + 1));
  }
  throw new Error(`unterminated ${name}`);
}

/**
 * The root `$defs` names `doc` reaches from its root through `#/$defs/<name>`
 * references, or null when it holds a local `$ref` of another form (`#`, an
 * anchor), which the generators answer by keeping every definition.
 */
function reachedDefs(doc) {
  const defs = doc.$defs ?? {};
  const seen = new Set();
  let other = false;
  const visit = (node) => {
    if (Array.isArray(node)) return node.forEach(visit);
    if (!node || typeof node !== "object") return;
    const ref = node.$ref;
    if (typeof ref === "string" && ref.startsWith("#")) {
      const m = /^#\/\$defs\/([^/]*)/.exec(ref);
      if (!m) other = true;
      else {
        const name = m[1].split("~1").join("/").split("~0").join("~");
        if (Object.hasOwn(defs, name) && !seen.has(name)) {
          seen.add(name);
          visit(defs[name]);
        }
      }
    }
    Object.values(node).forEach(visit);
  };
  const { $defs: _ignored, ...root } = doc;
  visit(root);
  return other ? null : seen;
}

/** `doc` as a generator emits it: only the `$defs` its root reaches (#1098). */
function pruned(doc) {
  if (!doc || !doc.$defs || typeof doc.$defs !== "object") return doc;
  const keep = reachedDefs(doc);
  if (keep === null) return doc;
  const { $defs, ...rest } = doc;
  const kept = Object.fromEntries(Object.entries($defs).filter(([name]) => keep.has(name)));
  return Object.keys(kept).length > 0 ? { ...rest, $defs: kept } : rest;
}

function walkBundles(dir, out = []) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, entry.name);
    if (entry.isDirectory()) walkBundles(p, out);
    else if (entry.name === "bundle.json") out.push(path.relative(DIR, p).split(path.sep).join("/"));
  }
  return out;
}

const indexPath = path.join(DIR, "index.json");
if (!fs.existsSync(indexPath)) {
  console.error(`${indexPath} does not exist — run \`npm run build-bundles\` first.`);
  process.exit(1);
}
const indexBytes = fs.readFileSync(indexPath, "utf8");
const index = JSON.parse(indexBytes);
if (index.indexVersion !== 1) fail(`index.json: unsupported indexVersion ${index.indexVersion}`);
if (canonical(index) !== indexBytes) fail("index.json is not in canonical form");

const listed = new Map();
for (const entry of index.specs) {
  if (listed.has(entry.path)) fail(`index.json lists ${entry.path} twice`);
  listed.set(entry.path, entry);
}
const onDisk = new Set(walkBundles(DIR));
for (const p of onDisk) if (!listed.has(p)) fail(`${p}: on disk but not in index.json`);
for (const p of listed.keys()) if (!onDisk.has(p)) fail(`${p}: in index.json but not on disk`);

let compared = 0;
for (const [rel, entry] of listed) {
  if (!onDisk.has(rel)) continue;
  const bytes = fs.readFileSync(path.join(DIR, rel));
  const digest = `sha256:${createHash("sha256").update(bytes).digest("hex")}`;
  if (digest !== entry.digest) fail(`${rel}: digest ${digest} but index.json says ${entry.digest}`);

  const text = bytes.toString("utf8");
  const bundle = JSON.parse(text);
  if (canonical(bundle) !== text) fail(`${rel}: not in canonical form`);
  if (bundle.bundleVersion !== 1) fail(`${rel}: unsupported bundleVersion ${bundle.bundleVersion}`);
  if (rel !== `${bundle.slug}/${bundle.version}/bundle.json`) fail(`${rel}: names ${bundle.slug}/${bundle.version}`);
  if (bundle.typeUri !== `https://trusttasks.org/spec/${bundle.slug}/${bundle.version}`) {
    fail(`${rel}: typeUri ${bundle.typeUri}`);
  }
  for (const key of ["typeUri", "slug", "version", "status", "deprecation", "extends"]) {
    if (bundle[key] !== entry[key]) {
      fail(`${rel}: ${key} is ${JSON.stringify(bundle[key])} but index.json says ${JSON.stringify(entry[key])}`);
    }
  }

  const own = JSON.parse(
    fs.readFileSync(path.join(SPECS, bundle.slug, bundle.version, "payload.schema.json"), "utf8"),
  ).$defs ?? {};
  const origins = bundle.defOrigins ?? {};
  for (const name of Object.keys(bundle.schema.$defs ?? {})) {
    const origin = origins[name];
    if (name in own) {
      if (origin !== undefined) fail(`${rel}: defOrigins names ${name}, which the payload schema declares itself`);
      continue;
    }
    const hash = origin?.lastIndexOf("#/$defs/") ?? -1;
    if (hash < 0 || origin.slice(hash + 8) !== name) {
      fail(`${rel}: defOrigins.${name} is ${JSON.stringify(origin)}, not <$id>#/$defs/${name}`);
    } else if (!SHARED.get(origin.slice(0, hash))?.has(name)) {
      fail(`${rel}: defOrigins.${name} names ${origin}, which no shared schema in specs/ defines`);
    }
  }
  for (const name of Object.keys(origins)) {
    if (!(name in (bundle.schema.$defs ?? {}))) fail(`${rel}: defOrigins names ${name}, which is not in $defs`);
  }

  for (const [pointer, doc] of [["/schema", bundle.schema], ["/responseSchema", bundle.responseSchema]]) {
    const names = doc?.$defs ? Object.keys(doc.$defs).sort() : undefined;
    const recorded = bundle.defsOrder?.[pointer];
    if (JSON.stringify(recorded ? [...recorded].sort() : undefined) !== JSON.stringify(names)) {
      fail(`${rel}: defsOrder${pointer} is ${JSON.stringify(recorded)}, but its $defs are ${JSON.stringify(names)}`);
    }
  }

  if (bundle.status === "withdrawn") continue;
  const tsPath = path.join(TS_SRC, bundle.slug, bundle.version, "payload.ts");
  if (!fs.existsSync(tsPath)) {
    fail(`${rel}: no TypeScript module at ${path.relative(ROOT, tsPath)}`);
    continue;
  }
  const src = fs.readFileSync(tsPath, "utf8");
  const tsRequest = tsConst(src, "PAYLOAD_SCHEMA");
  const tsResponse = tsConst(src, "RESPONSE_PAYLOAD_SCHEMA") ?? null;
  const request = pruned(bundle.schema);
  const response = pruned(bundle.responseSchema);
  if (canonical(tsRequest) !== canonical(request)) {
    fail(`${rel}: schema, pruned to the $defs it reaches, differs from the TypeScript PAYLOAD_SCHEMA`);
  }
  if (canonical(tsResponse) !== canonical(response)) {
    fail(`${rel}: responseSchema, pruned to the $defs it reaches, differs from the TypeScript RESPONSE_PAYLOAD_SCHEMA`);
  }
  for (const [pointer, ts, doc] of [["/schema", tsRequest, request], ["/responseSchema", tsResponse, response]]) {
    const expected = ts?.$defs ? Object.keys(ts.$defs) : undefined;
    const kept = doc?.$defs ?? {};
    const filtered = bundle.defsOrder?.[pointer]?.filter((name) => Object.hasOwn(kept, name));
    const recorded = filtered?.length ? filtered : undefined;
    if (JSON.stringify(recorded) !== JSON.stringify(expected)) {
      fail(`${rel}: defsOrder${pointer} is ${JSON.stringify(recorded)}, the TypeScript module's $defs are ${JSON.stringify(expected)}`);
    }
  }
  compared++;
}

if (failures.length) {
  console.error(`check-bundles: ${failures.length} problem(s):`);
  for (const f of failures.slice(0, 50)) console.error(`  - ${f}`);
  if (failures.length > 50) console.error(`  … and ${failures.length - 50} more`);
  process.exit(1);
}
console.log(
  `check-bundles: ${listed.size} bundles verified (digest, canonical form, index); ` +
    `${compared} schema pairs and their $defs order agree with the TypeScript generator.`,
);
