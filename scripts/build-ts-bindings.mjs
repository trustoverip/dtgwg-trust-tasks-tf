// Generate TypeScript bindings for every Trust Tasks payload schema.
//
// Walks specs/<slug>/<version>/payload.schema.json, plus shared schemas under
// _shared/ and _framework/, and emits one .ts file per JSON Schema into
// trust-tasks-ts/src/. The output is hand-readable, ships in the
// @openvtc/trust-tasks npm package, and is consumed by Companions, mobile
// clients, and any other TS implementation.
//
// Each generated request file additionally exports:
//   - TYPE_URI       — the Trust Task type URI (https://trusttasks.org/spec/<slug>/<version>)
//   - RESPONSE_TYPE_URI — the response form (… + "#response"), emitted ONLY when
//     the schema declares $defs.Response. A spec with no success response is
//     fire-and-forget: SPEC.md §4.4.1 says its consumers MUST NOT emit a
//     `#response`-variant document, so handing implementers a ready-made
//     constant for one invites a conformance violation.
//   - Payload / Response — stable type aliases for the request and response
//     payload shapes. json-schema-to-typescript names the emitted interfaces
//     from each schema's `title` (ACLGrantPayload, KeysSignPayload, …), so
//     without these there is no name a consumer can rely on across specs and
//     no way to write code generic over a Trust Task. They mirror the Rust
//     bindings' uniform `Payload` / `Response` structs.
//
// Run from the repo root:
//   npm run build-ts-bindings
//
// Strategy: use json-schema-to-typescript, with every cross-file `$ref`
// **hoisted** into one generated module rather than inlined per spec.
//
// ── Why hoisting, and why it is safe in TypeScript ──────────────────────────
//
// Left to itself the compiler emits a fresh copy of a `$ref`'d definition into
// every module that reaches it, and appends a counter when two copies collide
// inside one file (`Ext`, `Ext1`, `Ext2`). On the tree as it stood before this
// change that produced 481 `Ext` declarations across 341 files — the same
// object, under three different names, with which number you got decided by
// declaration order inside a generated file. A consumer importing `Ext1` from
// one module and `Ext2` from another had two names for one shape and no way to
// write a signature over "the framework extension object".
//
// So every cross-file `$ref` is rewritten to json-schema-to-typescript's
// `tsType` escape hatch naming a type declared once in `_shared/components.ts`,
// and the module imports it. The equivalent hoist was declined for the Rust
// bindings (#283) and the reason does not carry over: in Rust
// `upsert::v0_3::VaultEntry` and `delete::v0_1::VaultEntry` are distinct
// *nominal* types, so collapsing them is an E0119 coherence break for any
// consumer holding a trait impl on each. TypeScript is structurally typed and
// has no coherence rule — two identical `interface Ext` declarations are
// already mutually assignable — so hoisting changes names, not assignability.
// Nothing generated here is branded, uses a `unique symbol`, or relies on
// declaration merging, which are the three TypeScript constructs that would
// make declaration *identity* observable.
//
// ── Grouping is by structure, not by name ──────────────────────────────────
//
// #283's other finding does carry over: a shared name is not a shared shape.
// `VaultEntry` exists in three structurally different versions, `Scope` in two
// (consent's and vta's are unrelated), and 22 names in all denote more than one
// thing. Components are therefore grouped by their fully-resolved schema, and a
// name covering more than one group is emitted once per group under a
// family/version-qualified identifier (`VaultEntry_VaultV0_1`). Only a name
// with exactly one shape across the whole registry keeps the bare form.
//
// Each spec module re-exports what it uses under the definition's own name, so
// `AclEntry` is still `AclEntry` there — what disappears is the numbered
// duplicates. Shared schema modules re-export their own definitions the same
// way, which is what makes `VaultEntryShared_v0_1.VaultEntry` in the barrel
// resolve to something: those modules used to compile to a bare
// `[k: string]: unknown` root and drop every `$def` they exist to publish.
//
// Relative specifiers MUST end in `.js` — in `index.ts`, in the component
// imports, and in the re-exports; see the note where the barrel builds them.

import fsSync, { promises as fs } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import YAML from "yaml";
import { compile } from "json-schema-to-typescript";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const REPO_ROOT = path.resolve(__dirname, "..");
const SPECS_DIR = path.join(REPO_ROOT, "specs");
const OUT_DIR = path.join(REPO_ROOT, "trust-tasks-ts", "src");
const RUNTIME_DIR = "_runtime";

// Where the hoisted cross-file components land. `specs/` has no top-level
// `_shared/` directory, so this cannot collide with a generated spec tree.
const COMPONENTS_DIR = "_shared";
const COMPONENTS_FILE = "components.ts";
const COMPONENTS_PATH = path.join(OUT_DIR, COMPONENTS_DIR, COMPONENTS_FILE);

// The synthetic root the components module is compiled through. Its only job is
// to make every component reachable from one schema root so the compiler emits
// them all; it is stripped from the output before the file is written.
const COMPONENTS_ROOT_TYPE = "TrustTasksSharedComponentsRoot";

// --- Self-contained schema emission (SPEC.md §7.2 item 2) -------------------
//
// The generated modules carry their `payload.schema.json` as a value so a
// consumer can actually perform item 2 at runtime. Types are erased in
// TypeScript, so without this there is no artifact to validate against and
// every REQUIRED payload member is optional in practice.
//
// The on-disk schemas carry cross-file `$ref`s into `_shared/` and
// `_framework/`. Those are resolved for the *type* emission by
// json-schema-to-typescript's own walker, which leaves nothing behind at
// runtime — so the refs are inlined here into a self-contained document,
// mirroring `resolve_cross_file_refs` in trust-tasks-codegen. Both libraries
// must end up validating against the same schema text; `npm run
// check-bindings` is what holds them to it.

/** Split `<path>#/$defs/<name>` into its two halves, or null if it is not that shape. */
function splitExternalRef(s) {
  const hash = s.indexOf("#");
  if (hash < 0) return null;
  const filePath = s.slice(0, hash);
  const defName = s.slice(hash + 1).replace(/^\/\$defs\//, "");
  if (!filePath || !defName || defName.includes("/") || defName === s.slice(hash + 1)) return null;
  return { filePath, defName };
}

/** Every `$ref` in `node` matching `predicate`, in document order. */
function collectRefs(node, predicate, out = []) {
  if (Array.isArray(node)) {
    for (const item of node) collectRefs(item, predicate, out);
  } else if (node && typeof node === "object") {
    if (typeof node.$ref === "string" && predicate(node.$ref)) out.push(node.$ref);
    for (const value of Object.values(node)) collectRefs(value, predicate, out);
  }
  return out;
}

/** Rewrite every external `$ref` to its local `#/$defs/<name>` form, in place. */
function localizeRefs(node) {
  if (Array.isArray(node)) {
    node.forEach(localizeRefs);
  } else if (node && typeof node === "object") {
    if (typeof node.$ref === "string" && !node.$ref.startsWith("#")) {
      const split = splitExternalRef(node.$ref);
      if (split) node.$ref = `#/$defs/${split.defName}`;
    }
    for (const value of Object.values(node)) localizeRefs(value);
  }
}

/**
 * Inline every cross-file `$ref` into the schema's own `$defs`, so the emitted
 * schema resolves with no filesystem access.
 */
function inlineCrossFileRefs(schema, baseDir) {
  const isExternal = (r) => !r.startsWith("#");
  const frontier = collectRefs(schema, isExternal).map((ref) => ({ ref, ownerDir: baseDir }));
  const seen = new Set();

  while (frontier.length > 0) {
    const { ref, ownerDir } = frontier.pop();
    const split = splitExternalRef(ref);
    if (!split) throw new Error(`external $ref ${JSON.stringify(ref)} is not <path>#/$defs/<name>`);
    const abs = path.resolve(ownerDir, split.filePath);
    const key = `${abs}#/$defs/${split.defName}`;
    if (seen.has(key)) continue;
    seen.add(key);

    const referenced = JSON.parse(fsSync.readFileSync(abs, "utf8"));
    const fragment = referenced?.$defs?.[split.defName];
    if (fragment === undefined) {
      throw new Error(`${abs} has no $defs/${split.defName} (referenced from ${ownerDir})`);
    }

    schema.$defs ??= {};
    const existing = schema.$defs[split.defName];
    if (existing !== undefined) {
      if (JSON.stringify(existing) !== JSON.stringify(fragment)) {
        throw new Error(
          `schema already defines $defs/${split.defName} with a different shape; ` +
            `the cross-file $ref splice would overwrite it`,
        );
      }
    } else {
      schema.$defs[split.defName] = fragment;
    }

    // A spliced fragment can carry refs of its own. External ones resolve
    // against the file it came from, not the original base; internal ones
    // (`#/$defs/X`) point at siblings in that same file, so recast them as
    // external refs against it and let the same path handle both.
    const fragmentDir = path.dirname(abs);
    for (const r of collectRefs(fragment, isExternal)) {
      frontier.push({ ref: r, ownerDir: fragmentDir });
    }
    for (const r of collectRefs(fragment, (x) => x.startsWith("#"))) {
      frontier.push({ ref: `${split.filePath}${r}`, ownerDir: ownerDir });
    }
  }

  localizeRefs(schema);
  return schema;
}

/**
 * The response variant needs a schema of its own: the root describes the
 * *request* payload and `$defs.Response` describes the response. Wrap the
 * latter so `$defs` stays reachable and its internal `$ref`s still resolve.
 *
 * No `$id`: 2020-12 forbids a non-empty fragment in `$id`, so the natural
 * `<base>#response` will not compile under a conforming validator.
 */
function responseSchemaOf(schema) {
  if (schema?.$defs?.Response === undefined) return null;
  const doc = {};
  if (schema.$schema !== undefined) doc.$schema = schema.$schema;
  doc.$ref = "#/$defs/Response";
  doc.$defs = schema.$defs;
  return doc;
}

/* ── Shared component registry ───────────────────────────────────────────────
 *
 * Everything below computes, once per run, the set of `$defs` that live in a
 * `_shared/` or `_framework/` schema and are reachable from at least one
 * payload schema, and gives each distinct *shape* one TypeScript name. See the
 * file header for why hoisting is sound here and was not in Rust.
 */

const schemaCache = new Map();

/** Parse a schema file once per run. Every component read goes through here. */
function readSchemaSync(abs) {
  let doc = schemaCache.get(abs);
  if (doc === undefined) {
    doc = JSON.parse(fsSync.readFileSync(abs, "utf8"));
    schemaCache.set(abs, doc);
  }
  return doc;
}

const isExternalRef = (r) => !r.startsWith("#");
const componentKey = (abs, defName) => `${abs}#/$defs/${defName}`;

/**
 * Resolve one `$ref` against the directory it was written in.
 *
 * Internal refs are re-cast as refs against `ownerAbs` itself, so a `$def` that
 * points at a sibling in its own file and one that points across a file are
 * handled by a single rule.
 */
function resolveRef(ref, ownerAbs) {
  if (ref.startsWith("#")) {
    const split = splitExternalRef(`${path.basename(ownerAbs)}${ref}`);
    if (!split) throw new Error(`internal $ref ${JSON.stringify(ref)} is not #/$defs/<name>`);
    return { abs: ownerAbs, defName: split.defName };
  }
  const split = splitExternalRef(ref);
  if (!split) throw new Error(`external $ref ${JSON.stringify(ref)} is not <path>#/$defs/<name>`);
  return { abs: path.resolve(path.dirname(ownerAbs), split.filePath), defName: split.defName };
}

/**
 * Every `$def` reachable from a payload schema by a cross-file `$ref`, plus
 * everything those definitions reach in turn.
 *
 * The closure stops at the payload schemas' own `$defs`: those are not shared,
 * are never duplicated, and stay inlined in their spec's module.
 */
function collectComponents(payloadPaths) {
  const components = new Map();
  for (const payloadPath of payloadPaths) {
    const doc = readSchemaSync(payloadPath);
    const frontier = collectRefs(doc, isExternalRef).map((ref) => ({ ref, ownerAbs: payloadPath }));
    while (frontier.length > 0) {
      const { ref, ownerAbs } = frontier.pop();
      const { abs, defName } = resolveRef(ref, ownerAbs);
      const key = componentKey(abs, defName);
      if (components.has(key)) continue;

      const fragment = readSchemaSync(abs)?.$defs?.[defName];
      if (fragment === undefined) {
        throw new Error(`${abs} has no $defs/${defName} (referenced from ${ownerAbs})`);
      }
      // The base name is what the compiler would have called this definition on
      // its own: a `title` if the schema carries one, the `$defs` key if not.
      // `auth/_shared/0.1/webauthn.schema.json` keys a definition
      // `CredentialDescriptor` and titles it `PublicKeyCredentialDescriptor`,
      // and seven modules exported the latter — taking the key here would have
      // renamed a type the hoist has no business renaming.
      const baseName = titleTypeName(fragment.title) ?? defName;
      components.set(key, { key, file: abs, defName, baseName, fragment });
      for (const r of collectRefs(fragment, () => true)) frontier.push({ ref: r, ownerAbs: abs });
    }
  }
  return components;
}

/**
 * A component's shape, fully resolved and with object keys ordered, as a string.
 *
 * Two components are the same type exactly when these agree. Comparing the raw
 * `$defs` fragments would not do: identical fragments can carry `$ref`s that
 * resolve to different things, and grouping by *name* would merge the three
 * unrelated `VaultEntry`s. Recursion through a `$ref` already on the stack is
 * cut with a marker rather than followed, so a self-referential definition
 * canonicalizes instead of hanging.
 */
function canonicalShape(fragment, ownerAbs) {
  const visit = (node, owner, stack) => {
    if (Array.isArray(node)) return node.map((n) => visit(n, owner, stack));
    if (!node || typeof node !== "object") return node;
    if (typeof node.$ref === "string") {
      const rest = { ...node };
      delete rest.$ref;
      const { abs, defName } = resolveRef(node.$ref, owner);
      const key = componentKey(abs, defName);
      if (stack.has(key)) return sortedKeys({ ...rest, $recursion: defName });
      const target = readSchemaSync(abs)?.$defs?.[defName];
      if (target === undefined) throw new Error(`${abs} has no $defs/${defName}`);
      return sortedKeys({ ...visit(target, abs, new Set([...stack, key])), ...rest });
    }
    const out = {};
    for (const k of Object.keys(node).sort()) out[k] = visit(node[k], owner, stack);
    return out;
  };
  return JSON.stringify(visit(fragment, ownerAbs, new Set()));
}

function sortedKeys(value) {
  if (Array.isArray(value)) return value.map(sortedKeys);
  if (!value || typeof value !== "object") return value;
  const out = {};
  for (const k of Object.keys(value).sort()) out[k] = sortedKeys(value[k]);
  return out;
}

/**
 * The `<Family>V<major>_<minor>` tag used to disambiguate a name that denotes
 * more than one shape — `VaultEntry_VaultV0_1`.
 *
 * `V` is capital and the family is not separated by an underscore-lowercase
 * pair because json-schema-to-typescript re-normalizes whatever it is handed as
 * a `title` into an identifier, and that normalization eats `_v` (it reads
 * snake_case and camel-cases it). Handing it a name it will not rewrite is what
 * keeps the identifier the compiler emits equal to the one the importing
 * modules ask for; `verifyComponentNames` holds the two together regardless.
 */
function familyTag(abs) {
  const segments = path.relative(SPECS_DIR, abs).split(path.sep).slice(0, -1);
  const version = /^\d+\.\d+$/.test(segments[segments.length - 1]) ? segments.pop() : null;
  const family = segments
    .filter((s) => s !== "_shared")
    .flatMap((s) => s.replace(/^_/, "").split("-"))
    .map((s) => s.charAt(0).toUpperCase() + s.slice(1))
    .join("");
  return `${family}${version ? `V${version.replace(/\./g, "_")}` : ""}`;
}

/**
 * Give every component a TypeScript name.
 *
 * A `$defs` name that denotes exactly one shape across the whole registry keeps
 * it. A name that denotes several is emitted once per shape, and *every* one of
 * them is qualified — leaving one unqualified would imply it is the canonical
 * `VaultEntry` when there is no such thing.
 */
function buildComponentRegistry(payloadPaths) {
  const components = collectComponents(payloadPaths);

  const byDefName = new Map();
  for (const component of components.values()) {
    component.shape = canonicalShape(component.fragment, component.file);
    if (!byDefName.has(component.baseName)) byDefName.set(component.baseName, []);
    byDefName.get(component.baseName).push(component);
  }

  const byName = new Map();
  const multiShaped = [];
  for (const [baseName, list] of byDefName) {
    const groups = new Map();
    for (const component of list) {
      if (!groups.has(component.shape)) groups.set(component.shape, []);
      groups.get(component.shape).push(component);
    }
    const qualify = groups.size > 1;
    if (qualify) multiShaped.push({ baseName, shapes: groups.size });
    for (const group of groups.values()) {
      group.sort((a, b) => (a.file < b.file ? -1 : a.file > b.file ? 1 : 0));
      const name = qualify ? `${baseName}_${familyTag(group[0].file)}` : baseName;
      if (byName.has(name)) {
        throw new Error(
          `two shapes of ${baseName} both want the TypeScript name ${name} — ` +
            `they come from the same family and version (${group[0].file} vs ` +
            `${byName.get(name).sources[0].file}); one of them needs renaming in the schema`,
        );
      }
      byName.set(name, { name, baseName, fragment: group[0].fragment, file: group[0].file, sources: group });
      for (const component of group) component.tsName = name;
    }
  }

  // A hoisted name that collides with one of the module tail's own exports
  // would be shadowed silently, so refuse rather than emit it.
  for (const reserved of ["Payload", "Response", "SPEC", "RESPONSE_SPEC", "TYPE_URI"]) {
    if (byName.has(reserved)) {
      throw new Error(`a shared component is named ${reserved}, which every spec module also exports`);
    }
  }

  // `displayNames` maps an emitted identifier back to the name a module should
  // bind it to. It is seeded with the qualified components and filled in for
  // their nested branch types when the components module is compiled.
  const displayNames = new Map();
  for (const [name, entry] of byName) if (name !== entry.baseName) displayNames.set(name, entry.baseName);

  multiShaped.sort((a, b) => b.shapes - a.shapes || a.baseName.localeCompare(b.baseName));
  return { components, byName, multiShaped, displayNames, dependencies: new Map() };
}

/**
 * Rewrite every `$ref` that leaves this document into a `tsType` naming the
 * hoisted component, and report what was used.
 *
 * `tsType` is json-schema-to-typescript's escape hatch for "emit this type name
 * verbatim". Using it rather than deleting the `$ref` and patching the output
 * text keeps the compiler in charge of arrays, unions, optionality and the
 * JSDoc it derives from each ref site's `description`, which survives because
 * the sibling keywords are left in place.
 *
 * `localNames` maps hoisted name → the name this module will refer to it by.
 * Spec modules use the definition's own `$defs` name (so `AclEntry` is still
 * `AclEntry` in the emitted interface) and alias it at the import.
 */
function rewriteRefsToComponents(schema, ownerAbs, registry, { internal = false, hoistedLocals = false } = {}) {
  const used = new Map();
  const localNames = new Map();
  const claimed = new Map();

  const localNameFor = (component) => {
    if (hoistedLocals) {
      used.set(component.tsName, { component, local: component.tsName });
      return component.tsName;
    }
    const existing = localNames.get(component.tsName);
    if (existing !== undefined) return existing;
    // Two different components with the same `$defs` name in one module (the
    // framework 0.1 and 0.2 `Ext`s, before they deduplicate) cannot both hold
    // the bare name; the loser keeps the qualified one.
    const claimant = claimed.get(component.baseName);
    const local = claimant === undefined ? component.baseName : component.tsName;
    if (claimant === undefined) claimed.set(component.baseName, component.tsName);
    localNames.set(component.tsName, local);
    used.set(component.tsName, { component, local });
    return local;
  };

  const visit = (node, owner) => {
    if (Array.isArray(node)) return node.forEach((n) => visit(n, owner));
    if (!node || typeof node !== "object") return;
    if (typeof node.$ref === "string" && (internal || isExternalRef(node.$ref))) {
      const { abs, defName } = resolveRef(node.$ref, owner);
      const component = registry.components.get(componentKey(abs, defName));
      if (!component) {
        throw new Error(`${owner}: $ref ${node.$ref} resolves to no known shared component`);
      }
      delete node.$ref;
      node.tsType = localNameFor(component);
      return;
    }
    for (const value of Object.values(node)) visit(value, owner);
  };

  visit(schema, ownerAbs);
  return { schema, used };
}

/**
 * The `import type { … }` and `export type { … }` clauses a module needs, or
 * empty strings when it references no shared component.
 *
 * The import covers what the module's own declarations mention. The re-export
 * additionally covers everything those types reach inside the components
 * module — a `oneOf` branch like `Companion`, which the compiler names but no
 * payload schema `$ref`s. Those used to be declared in every module that
 * reached them, and dropping them here would take exports away rather than
 * rename them, so the whole closure comes back out under the names it had.
 */
function componentClauses(used, outPath, ts, registry, alsoUsed = "") {
  if (used.size === 0) return { imports: "", reexports: "" };

  // Only import what actually survived compilation. A shared schema module
  // compiles to its root alone — none of its `$defs` are reachable from that
  // root — so its rewritten refs would otherwise become unused imports.
  //
  // `alsoUsed` carries the response type, which is a genuine use even though
  // `stripResponseProbe` has already removed the line it appeared on: a spec
  // whose `$defs.Response` aliases a shared definition (vta/contexts/get, and
  // seven others) mentions it only in the tail's `export type Response = …`.
  const code = `${ts}\n${alsoUsed}`.replace(/\/\*[\s\S]*?\*\//g, "");
  const direct = [...used.values()].filter(({ local }) => new RegExp(`\\b${local}\\b`).test(code));
  if (direct.length === 0) return { imports: "", reexports: "" };

  // Names this module already declares. Anything the closure would shadow keeps
  // its qualified form rather than silently redefining a local type.
  const declaredHere = new Set(
    [...ts.matchAll(/^export (?:interface|type) ([A-Za-z0-9_$]+)/gm)].map((m) => m[1]),
  );

  const directLocals = new Map(direct.map(({ component, local }) => [component.tsName, local]));
  for (const [tsName, local] of directLocals) {
    if (declaredHere.has(local)) {
      throw new Error(
        `${outPath}: shared component ${tsName} would be imported as ${local}, which this ` +
          `module also declares — the schema's own $defs needs a distinct name`,
      );
    }
  }

  const closure = new Set(directLocals.keys());
  const frontier = [...closure];
  while (frontier.length > 0) {
    for (const dep of registry.dependencies.get(frontier.pop()) ?? []) {
      if (!closure.has(dep)) {
        closure.add(dep);
        frontier.push(dep);
      }
    }
  }

  // The unqualified name is a preference, not a promise: whoever asks first
  // gets it, and anyone it would shadow keeps the qualified form.
  const taken = new Set(directLocals.values());
  const bindings = [...directLocals].map(([tsName, local]) => ({ tsName, local }));
  for (const tsName of [...closure].sort()) {
    if (directLocals.has(tsName)) continue;
    const preferred = registry.displayNames.get(tsName) ?? tsName;
    const local = taken.has(preferred) || declaredHere.has(preferred) ? tsName : preferred;
    taken.add(local);
    bindings.push({ tsName, local });
  }
  bindings.sort((a, b) => a.local.localeCompare(b.local));

  const specifier = JSON.stringify(importSpecifier(outPath, COMPONENTS_PATH));
  const clause = bindings
    .map(({ tsName, local }) => (tsName === local ? local : `${tsName} as ${local}`))
    .join(", ");
  return {
    imports: `\nimport type { ${clause} } from ${specifier};\n`,
    // Re-exported so the module's public surface is unchanged: a consumer that
    // imported `AclEntry` from a spec module still can, it is simply now the
    // one `AclEntry` rather than that module's private copy.
    reexports:
      `\n/** Shared definitions this specification references, re-exported under the names it used to declare them with. */\n` +
      `export type { ${bindings.map(({ local }) => local).join(", ")} };\n`,
  };
}

/** A relative ESM specifier from one generated module to another. Always `.js`. */
function importSpecifier(fromFile, toFile) {
  const rel = path.relative(path.dirname(fromFile), toFile).replace(/\\/g, "/").replace(/\.ts$/, ".js");
  return rel.startsWith(".") ? rel : `./${rel}`;
}

/** Remove one top-level declaration, by name, from compiled output. */
function stripDeclaration(ts, name) {
  const lines = ts.split("\n");
  const start = lines.findIndex((l) => new RegExp(`^export (?:interface|type) ${name}\\b`).test(l));
  if (start < 0) throw new Error(`compiled output declares no ${name} to strip`);
  let depth = 0;
  let end = start;
  for (; end < lines.length; end++) {
    for (const ch of lines[end]) {
      if (ch === "{" || ch === "[" || ch === "(") depth++;
      else if (ch === "}" || ch === "]" || ch === ")") depth--;
    }
    if (depth === 0 && (/;\s*$/.test(lines[end]) || /^\}/.test(lines[end]))) break;
  }
  lines.splice(start, end - start + 1);
  return lines.join("\n");
}

// Slug → ts type renames go here when json-schema-to-typescript's
// auto-naming clashes with reserved words or produces awkward identifiers.
// Empty for now; populate as ergonomic issues come up.
const TYPE_RENAMES = {};

async function walk(dir, pattern) {
  const out = [];
  const entries = await fs.readdir(dir, { withFileTypes: true });
  for (const entry of entries) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      out.push(...(await walk(full, pattern)));
    } else if (pattern.test(entry.name)) {
      out.push(full);
    }
  }
  return out;
}

async function ensureDir(p) {
  await fs.mkdir(p, { recursive: true });
}

function relativeOutPath(schemaPath) {
  // Convert <repo>/specs/<x>/<y>/<file>.schema.json to <out>/<x>/<y>/<file>.ts
  // For payload.schema.json the file becomes "payload" — we rename to the
  // version-bearing form below.
  const rel = path.relative(SPECS_DIR, schemaPath);
  const tsRel = rel
    .replace(/\.schema\.json$/, ".ts")
    .replace(/[\\/]/g, "/");
  return path.join(OUT_DIR, tsRel);
}

function slugFromSchemaPath(schemaPath) {
  // <repo>/specs/<slug>/<version>/payload.schema.json -> { slug, version }
  // Returns null for shared schemas (where the path includes a "_"-prefixed segment).
  const rel = path.relative(SPECS_DIR, schemaPath);
  const segments = rel.split(path.sep);
  if (segments.some((s) => s.startsWith("_"))) return null;
  if (path.basename(schemaPath) !== "payload.schema.json") return null;
  const version = segments[segments.length - 2];
  const slug = segments.slice(0, -2).join("/");
  return { slug, version };
}

/**
 * Read the SPEC §7.2 policy flags out of a spec's front matter.
 *
 * These mirror `Payload::IS_BEARER` / `IS_PROOF_REQUIRED` /
 * `IS_RECIPIENT_REQUIRED` / `IS_ISSUED_AT_REQUIRED` in trust-tasks-rs, and are
 * derived the same way, so
 * a TypeScript consumer reaches the same verdict as a Rust one on the same
 * document. Without them a TypeScript implementation cannot apply §7.2 items
 * 5b, 7 or 8 at all — the requirement is per-specification, and nothing else
 * on the wire carries it.
 *
 * Note the response-variant asymmetry (§7.3 item 5): a response document swaps
 * the parties, so the requirement governing a *response*'s `recipient` member
 * is the one declared for the *issuer* party of the request.
 */
function readSpecPolicy(schemaPath) {
  const specPath = path.join(path.dirname(schemaPath), "spec.md");
  let meta;
  try {
    const src = fsSync.readFileSync(specPath, "utf8");
    if (!src.startsWith("---")) return null;
    const end = src.indexOf("\n---", 3);
    if (end < 0) return null;
    meta = YAML.parse(src.slice(3, end).replace(/^\r?\n/, ""));
  } catch {
    return null;
  }
  if (!meta) return null;

  const partyRequirement = (member) =>
    (meta.parties || []).find((p) => p && p.member === member)?.requirement === "REQUIRED";

  // §7.3 item 8 is either a single `requirement` covering every variant, or a
  // per-variant `request` / `response` pair. Only REQUIRED obliges a consumer to
  // reject a proofless document, so each reduces to a boolean. A per-variant
  // declaration omitting `response` takes the request's value — the
  // conservative reading, and the only one that cannot weaken a variant by
  // omission.
  const pr = meta.proofRequirement || {};
  const proofRequired =
    typeof pr.requirement === "string"
      ? { request: pr.requirement === "REQUIRED", response: pr.requirement === "REQUIRED" }
      : {
          request: pr.request === "REQUIRED",
          response: (pr.response ?? pr.request) === "REQUIRED",
        };

  // §7.3 item 17, declared with the same shape as item 8 and normalised the
  // same way. Only REQUIRED obliges a consumer to reject a document with no
  // `issuedAt`; the framework baseline of §4.2 is already a SHOULD, which
  // RECOMMENDED merely restates.
  const ir = meta.issuedAtRequirement || {};
  const issuedAtRequired =
    typeof ir.requirement === "string"
      ? { request: ir.requirement === "REQUIRED", response: ir.requirement === "REQUIRED" }
      : {
          request: ir.request === "REQUIRED",
          response: (ir.response ?? ir.request) === "REQUIRED",
        };

  return {
    isBearer: meta.bearer === true,
    isProofRequired: proofRequired.request,
    responseIsProofRequired: proofRequired.response,
    isIssuedAtRequired: issuedAtRequired.request,
    responseIsIssuedAtRequired: issuedAtRequired.response,
    // Request: the party tagged `recipient`. Response: the party tagged
    // `issuer`, because the response addresses the original producer.
    isRecipientRequired: partyRequirement("recipient"),
    responseIsRecipientRequired: partyRequirement("issuer"),
  };
}

// Synthetic property name used to drag `$defs.Response` into the compiler's
// reachability graph. Deliberately ugly so it cannot collide with a real
// payload member; it is stripped from the output before the file is written.
const RESPONSE_PROBE = "__ttResponseProbe";

// Annotation keywords that do not constrain the instance, so a `$defs.Response`
// carrying only these alongside `$ref` is a pure alias for its target.
const RESPONSE_ANNOTATIONS = new Set(["$ref", "$anchor", "title", "description"]);

/**
 * The `$ref` the response probe should point at.
 *
 * Normally `#/$defs/Response`. But where `$defs.Response` is a bare alias for an
 * external definition, we point the probe straight at that external target
 * instead. Going through the local alias makes the resolver splice the foreign
 * subschema into *this* document's context, after which the target's own
 * internal refs (`#/$defs/Scope` in the vta/did-templates shared schema) are
 * looked up against the wrong document and the compile dies. Referencing the
 * external target directly is the same route every other cross-file `$ref` in
 * the schema already takes, and resolves correctly.
 */
function responseProbeRef(response) {
  const isPureAlias =
    typeof response.$ref === "string" &&
    Object.keys(response).every((k) => RESPONSE_ANNOTATIONS.has(k));
  return isPureAlias ? { $ref: response.$ref } : { $ref: "#/$defs/Response" };
}

/**
 * The type name json-schema-to-typescript mints from a schema `title`:
 * split on anything that is not alphanumeric, upper-case each token's first
 * letter, and join. "Audit List — payload" becomes `AuditListPayload`;
 * "VTC Relationships Publish — payload" becomes `VTCRelationshipsPublishPayload`,
 * an already-upper token surviving intact.
 */
function titleTypeName(title) {
  if (typeof title !== "string") return null;
  const parts = title
    .split(/[^A-Za-z0-9]+/)
    .filter(Boolean)
    .map((word) => word[0].toUpperCase() + word.slice(1));
  return parts.length ? parts.join("") : null;
}

/**
 * Name of the root interface/type the compiler emitted.
 *
 * This used to take the first `export interface|type` in the output, on the
 * assumption that the root always comes first. It does not. Where a schema
 * `$ref`s a shared definition — `DigestMultibase`, say — the compiler hoists
 * that definition to its own exported type, sometimes *ahead of* the root, and
 * the first-match rule then aliased `Payload` to the shared definition.
 * Fourteen published specifications shipped `export type Payload =
 * DigestMultibase`, a `string`, in place of their request payload interface.
 *
 * Nothing catches that downstream: the drift check compares the generator's
 * output to itself, `tsc` is satisfied because the alias is well-formed, and
 * no test asserts what `Payload` resolves to. So the rule has to be right here,
 * and it has to fail loudly when it is not — hence the invariant below rather
 * than a quieter heuristic.
 *
 * The root is identified by the name the compiler derives from the schema's own
 * `title`, and is used only when that name is actually present in the output.
 * Positional order is the fallback, for schemas carrying no usable title.
 */
function rootTypeName(ts, schemaPath, raw) {
  const names = [...ts.matchAll(/^export (?:interface|type) ([A-Za-z0-9_$]+)/gm)].map(
    (m) => m[1],
  );
  if (names.length === 0) {
    throw new Error(`${schemaPath}: compiled output declares no root type`);
  }
  const fromTitle = titleTypeName(raw?.title);
  const root = fromTitle && names.includes(fromTitle) ? fromTitle : names[0];

  // An object-rooted schema whose root compiled to a *bare* alias — `= string`,
  // `= DigestMultibase` — means we picked up a hoisted `$ref` instead of the
  // root, the exact failure this function exists to prevent. An object type
  // literal or a union is a legitimate root form and is left alone.
  const bareAlias = new RegExp(`^export type ${root} = ([A-Za-z0-9_$]+);\\s*$`, "m");
  if (raw?.type === "object" && bareAlias.test(ts)) {
    throw new Error(
      `${schemaPath}: root type '${root}' compiled to a bare alias, but the schema's ` +
        `root is an object — the alias would be emitted as this specification's ` +
        `\`Payload\`. Check that the schema's title matches the emitted interface.`,
    );
  }
  return root;
}

/**
 * Remove the synthetic response probe from the compiled output and return the
 * TypeScript type it resolved to.
 *
 * The probe exists because `$defs.Response` is referenced by nothing in the
 * schema — SPEC.md §4.4.1 addresses it out-of-band via the `response` anchor —
 * and json-schema-to-typescript emits only what it can reach from the root. A
 * plain compile therefore silently drops the response half of every
 * request/response specification.
 *
 * Injecting the probe and compiling ONCE (rather than compiling the request and
 * response halves separately) matters for correctness: the compiler
 * de-duplicates structurally distinct types that share a name by appending a
 * counter (`Ext`, `Ext1`, …). Two independent passes number those counters
 * independently, so a name minted in the response pass could quietly denote a
 * different shape than the identical name in the request pass.
 */
function stripResponseProbe(ts, schemaPath) {
  const probe = new RegExp(`^[ \\t]*${RESPONSE_PROBE}\\?: (.+);[ \\t]*\\r?\\n`, "m");
  const m = probe.exec(ts);
  if (!m) {
    throw new Error(
      `${schemaPath}: response probe did not survive compilation — ` +
        `$defs.Response could not be resolved to a TypeScript type`,
    );
  }
  const stripped = ts.replace(probe, "");
  if (probe.test(stripped)) {
    throw new Error(`${schemaPath}: response probe matched more than once`);
  }
  return { ts: stripped, responseType: m[1] };
}

function emitTail(slugInfo, ts, rootType, responseType, schemaPath, policy, schemas) {
  if (!slugInfo) return ""; // shared schemas are not Trust Tasks
  const { slug, version } = slugInfo;
  const typeUri = `https://trusttasks.org/spec/${slug}/${version}`;
  const lines = [
    "",
    `/** Trust Task type URI. */`,
    `export const TYPE_URI = ${JSON.stringify(typeUri)} as const;`,
    "",
    `/** Stable alias for this specification's request payload shape. */`,
  ];
  // If the compiler already named the root `Payload` the alias is redundant and
  // would be a self-reference; anything else named `Payload` is a genuine clash
  // we must not paper over.
  if (rootType === "Payload") {
    lines.push(`// (the root interface is already named \`Payload\`)`);
  } else if (/^export (?:interface|type) Payload\b/m.test(ts)) {
    throw new Error(`${schemaPath}: cannot alias Payload — the name is already taken`);
  } else {
    lines.push(`export type Payload = ${rootType};`);
  }
  lines.push("");

  if (responseType && responseType !== "Response" && /^export (?:interface|type) Response\b/m.test(ts)) {
    throw new Error(`${schemaPath}: cannot alias Response — the name is already taken`);
  }
  if (responseType) {
    lines.push(
      `/** Trust Task response type URI (request type URI + "#response"). */`,
      `export const RESPONSE_TYPE_URI = ${JSON.stringify(typeUri + "#response")} as const;`,
      "",
      `/** Stable alias for this specification's success-response payload shape. */`,
      ...(responseType === "Response"
        ? [`// (the response interface is already named \`Response\`)`]
        : [`export type Response = ${responseType};`]),
      "",
    );
  }

  if (schemas?.request) {
    lines.push(
      `/**`,
      ` * This specification's payload schema, as a value.`,
      ` *`,
      ` * SPEC.md §7.2 item 2 is performed against this. It is shipped as data`,
      ` * rather than only as a \`.json\` file because TypeScript types are erased`,
      ` * at runtime: without a schema a consumer has nothing to validate, and`,
      ` * every REQUIRED payload member is optional in practice. Cross-file`,
      ` * \`$ref\`s are already inlined, so it needs no resolver.`,
      ` */`,
      `export const PAYLOAD_SCHEMA = ${JSON.stringify(schemas.request, null, 2)} as const;`,
      "",
    );
    if (schemas.response) {
      lines.push(
        `/** As {@link PAYLOAD_SCHEMA}, for the success-response variant. */`,
        `export const RESPONSE_PAYLOAD_SCHEMA = ${JSON.stringify(schemas.response, null, 2)} as const;`,
        "",
      );
    }
  }

  if (policy) {
    const obj = (uri, isProofRequired, isRecipientRequired, isIssuedAtRequired, schemaConst) =>
      [
        `{`,
        `  typeUri: ${uri},`,
        `  isBearer: ${policy.isBearer},`,
        `  isProofRequired: ${isProofRequired},`,
        `  isRecipientRequired: ${isRecipientRequired},`,
        `  isIssuedAtRequired: ${isIssuedAtRequired},`,
        `  payloadSchema: ${schemaConst},`,
        `} as const;`,
      ].join("\n");

    lines.push(
      `/**`,
      ` * SPEC.md §7.2 policy for the request variant, from this specification's`,
      ` * front matter. Pass to \`consumeInbound\` — items 5b, 7 and 8 are`,
      ` * per-specification and cannot be derived from the document alone, and`,
      ` * item 2 needs the schema this carries.`,
      ` */`,
      `export const SPEC = ${obj(
        "TYPE_URI",
        policy.isProofRequired,
        policy.isRecipientRequired,
        policy.isIssuedAtRequired,
        schemas?.request ? "PAYLOAD_SCHEMA" : "undefined",
      )}`,
      "",
    );

    if (responseType) {
      lines.push(
        `/**`,
        ` * SPEC.md §7.2 policy for the success-response variant. \`isRecipientRequired\``,
        ` * tracks the *issuer* party's requirement because a response swaps the`,
        ` * parties (§7.3 item 5).`,
        ` */`,
        `export const RESPONSE_SPEC = ${obj(
          "RESPONSE_TYPE_URI",
          policy.responseIsProofRequired,
          policy.responseIsRecipientRequired,
          policy.responseIsIssuedAtRequired,
          schemas?.response ? "RESPONSE_PAYLOAD_SCHEMA" : "undefined",
        )}`,
        "",
      );
    }
  }
  return lines.join("\n");
}

/** The compiler options every generated module is produced with. */
function compileOptions(sourcePath, cwd) {
  return {
    cwd,
    bannerComment:
      "/**\n * Generated by scripts/build-ts-bindings.mjs — DO NOT EDIT BY HAND.\n * Source: " +
      sourcePath +
      "\n */",
    additionalProperties: false,
    declareExternallyReferenced: true,
    enableConstEnums: false,
    strictIndexSignatures: true,
    unknownAny: true,
    style: { singleQuote: false, semi: true },
  };
}

async function generateOne(schemaPath, registry) {
  const outPath = relativeOutPath(schemaPath);
  await ensureDir(path.dirname(outPath));

  const opts = compileOptions(path.relative(REPO_ROOT, schemaPath), path.dirname(schemaPath));

  const raw = JSON.parse(await fs.readFile(schemaPath, "utf8"));
  const hasResponse = Boolean(raw?.$defs?.Response);

  // The probe is chosen against the original schema — `responseProbeRef` looks
  // for a bare `$ref` — and rewritten with everything else, so a response that
  // aliases a shared definition resolves to the hoisted type like any other.
  const input = hasResponse
    ? { ...raw, properties: { ...raw.properties, [RESPONSE_PROBE]: responseProbeRef(raw.$defs.Response) } }
    : raw;
  const { schema: rewritten, used } = rewriteRefsToComponents(
    structuredClone(input),
    schemaPath,
    registry,
  );

  let ts = await compile(rewritten, path.basename(schemaPath), opts);
  let responseType = null;
  if (hasResponse) ({ ts, responseType } = stripResponseProbe(ts, schemaPath));

  const slugInfo = slugFromSchemaPath(schemaPath);
  const policy = slugInfo ? readSpecPolicy(schemaPath) : null;
  if (slugInfo && !policy) {
    // Every task spec has front matter (the registry build enforces it). A
    // miss here would silently emit a module with no SPEC, and a consumer
    // would then have nothing to apply §7.2 items 5b/7/8 with.
    throw new Error(`${schemaPath}: could not read spec.md front matter for the §7.2 policy`);
  }
  // Build the self-contained schema from a fresh parse: `raw` is handed to
  // other emitters and inlining mutates in place.
  let schemas = null;
  if (slugInfo) {
    const selfContained = inlineCrossFileRefs(
      JSON.parse(await fs.readFile(schemaPath, "utf8")),
      path.dirname(schemaPath),
    );
    schemas = { request: selfContained, response: responseSchemaOf(selfContained) };
  }

  const tail = emitTail(
    slugInfo,
    ts,
    rootTypeName(ts, schemaPath, raw),
    responseType,
    schemaPath,
    policy,
    schemas,
  );

  // Imports go above the compiler's output and the re-exports below it, so
  // `rootTypeName`'s positional fallback still sees the compiler's own first
  // declaration first.
  const { imports, reexports } = componentClauses(used, outPath, ts, registry, responseType ?? "");
  const own = ownDefinitionReexports(schemaPath, outPath, registry);
  await fs.writeFile(outPath, spliceAfterBanner(ts, imports) + reexports + own + tail, "utf8");
  return { outPath, slugInfo };
}

/** Insert `block` immediately after the generated banner comment. */
function spliceAfterBanner(ts, block) {
  if (!block) return ts;
  const end = ts.indexOf("*/");
  if (end < 0) throw new Error("compiled output carries no banner comment to splice after");
  return `${ts.slice(0, end + 2)}\n${block}${ts.slice(end + 2)}`;
}

/**
 * For a `_shared/` or `_framework/` schema, re-export the definitions it owns.
 *
 * These modules exist to publish their `$defs`, and until this landed they
 * published none: the schema root declares no `type` and no `properties`, so
 * the compiler emitted a lone `[k: string]: unknown` interface and dropped
 * every definition as unreachable. `VaultEntryShared_v0_1.VaultEntry` in the
 * barrel named nothing at all.
 *
 * Definitions no payload schema reaches are not hoisted and so cannot be
 * re-exported here; they are also, by construction, not duplicated anywhere.
 */
function ownDefinitionReexports(schemaPath, outPath, registry) {
  const owned = [...registry.byName.values()]
    .flatMap((entry) => entry.sources)
    .filter((component) => component.file === schemaPath);
  if (owned.length === 0) return "";

  const seen = new Map();
  for (const component of owned.sort((a, b) => a.baseName.localeCompare(b.baseName))) {
    if (seen.has(component.baseName)) continue;
    seen.set(component.baseName, component.tsName);
  }
  const specifier = JSON.stringify(importSpecifier(outPath, COMPONENTS_PATH));
  const clause = [...seen.entries()]
    .map(([baseName, tsName]) => (baseName === tsName ? baseName : `${tsName} as ${baseName}`))
    .join(", ");
  return (
    `\n/**\n * The definitions this shared schema publishes, hoisted to one declaration each.\n * See ${JSON.stringify(importSpecifier(outPath, COMPONENTS_PATH))}.\n */\n` +
    `export type { ${clause} } from ${specifier};\n`
  );
}

/**
 * Compile every hoisted component into one module.
 *
 * The components are reached through a synthetic root object rather than
 * compiled one at a time: json-schema-to-typescript emits only what is
 * reachable from a schema root, and compiling each separately would give the
 * shared definitions *they* reference a fresh copy apiece — the duplication
 * this whole exercise removes. The root itself is stripped afterwards.
 */
async function emitComponentsModule(registry) {
  const names = [...registry.byName.keys()].sort();
  const $defs = {};
  const properties = {};
  for (const name of names) {
    const entry = registry.byName.get(name);
    const { schema } = rewriteRefsToComponents(
      structuredClone(entry.fragment),
      entry.file,
      registry,
      { internal: true, hoistedLocals: true },
    );
    // A `oneOf` branch carrying a `title` becomes a declaration of its own, and
    // two versions of a definition carry the same branch titles — `Companion`
    // in both `ConsumerKind`s — which is how the counter suffixes came back as
    // `Companion1`. Where the definition itself had to be qualified, its
    // branches are qualified with it. `displayNames` remembers what each was
    // called so importing modules can bind it back to the bare name.
    if (name !== entry.baseName) {
      tagNestedTitles(schema, name.slice(entry.baseName.length), registry.displayNames);
    }
    // `title` is what the compiler names a definition from, so it decides the
    // emitted identifier; the `$defs` key alone would not.
    $defs[name] = { ...schema, title: name };
    properties[name] = { $ref: `#/$defs/${name}` };
  }

  const banner =
    "/**\n" +
    " * Generated by scripts/build-ts-bindings.mjs — DO NOT EDIT BY HAND.\n" +
    " *\n" +
    " * Every definition a Trust Task payload schema reaches through a cross-file\n" +
    " * `$ref`, declared once. Spec modules import from here and re-export what they\n" +
    " * use under the definition's own name, so `Ext` is one type rather than one\n" +
    " * `Ext`, `Ext1` or `Ext2` per module.\n" +
    " *\n" +
    " * A name that denotes more than one shape across the registry — `VaultEntry`\n" +
    " * exists in three — is qualified with the family and version of the schema it\n" +
    " * came from. Every shape of such a name is qualified, including the oldest:\n" +
    " * there is no canonical `VaultEntry` to leave unmarked.\n" +
    " */";

  const ts = stripDeclaration(
    await compile(
      {
        $schema: "https://json-schema.org/draft/2020-12/schema",
        title: COMPONENTS_ROOT_TYPE,
        type: "object",
        additionalProperties: false,
        properties,
        $defs,
      },
      COMPONENTS_FILE.replace(/\.ts$/, ".schema.json"),
      { ...compileOptions("specs/**/_shared, specs/_framework", SPECS_DIR), bannerComment: banner },
    ),
    COMPONENTS_ROOT_TYPE,
  );

  verifyComponentNames(ts, names);
  indexDeclarations(ts, registry);
  await ensureDir(path.dirname(COMPONENTS_PATH));
  await fs.writeFile(COMPONENTS_PATH, ts.replace(/\n{3,}/g, "\n\n"), "utf8");
  return COMPONENTS_PATH;
}

/** Suffix every `title` below the root, recording the original name. */
function tagNestedTitles(schema, suffix, displayNames) {
  const visit = (node, isRoot) => {
    if (Array.isArray(node)) return node.forEach((n) => visit(n, false));
    if (!node || typeof node !== "object") return;
    if (!isRoot && typeof node.title === "string") {
      const tagged = `${node.title}${suffix}`;
      displayNames.set(tagged, node.title);
      node.title = tagged;
    }
    for (const value of Object.values(node)) visit(value, false);
  };
  visit(schema, true);
}

/**
 * Record what the components module declares and which of those declarations
 * reference which, so an importing module can pull in the closure of what it
 * uses rather than just the definitions it names.
 *
 * Reading it back out of the compiled text rather than deriving it from the
 * schemas is deliberate: what a module has to import is what the *emitted*
 * types mention, and only the emitted text knows which subschemas the compiler
 * chose to give a name to.
 */
function indexDeclarations(ts, registry) {
  const lines = ts.replace(/\/\*[\s\S]*?\*\//g, "").split("\n");
  const starts = [];
  lines.forEach((line, i) => {
    const m = /^export (?:interface|type) ([A-Za-z0-9_$]+)/.exec(line);
    if (m) starts.push({ name: m[1], line: i });
  });
  const declared = new Set(starts.map((s) => s.name));
  starts.forEach(({ name, line }, i) => {
    const body = lines.slice(line, i + 1 < starts.length ? starts[i + 1].line : lines.length).join("\n");
    const deps = new Set(
      [...body.matchAll(/[A-Za-z0-9_$]+/g)]
        .map((m) => m[0])
        .filter((id) => id !== name && declared.has(id)),
    );
    registry.dependencies.set(name, deps);
  });
}

/**
 * Assert that the components module declares exactly the names the spec modules
 * will import, and nothing numbered.
 *
 * The compiler re-normalizes each `title` into an identifier and appends a
 * counter to any name it has already used, so the identifier it emits is not
 * guaranteed to be the one asked for. Every spec module's `import type` clause
 * is written from the registry, not read back from this file, so a silent
 * rename here would surface as several hundred TS2305s — or, worse, as a name
 * that happens to resolve to the wrong shape. Fail at the source instead.
 */
function verifyComponentNames(ts, expected) {
  const declared = new Set(
    [...ts.matchAll(/^export (?:interface|type) ([A-Za-z0-9_$]+)/gm)].map((m) => m[1]),
  );
  const missing = expected.filter((n) => !declared.has(n));
  if (missing.length) {
    throw new Error(
      `the components module does not declare ${missing.length} of the names the spec ` +
        `modules import — json-schema-to-typescript renamed them: ${missing.slice(0, 8).join(", ")}` +
        `${missing.length > 8 ? ", …" : ""}`,
    );
  }
  const numbered = [...declared].filter(
    (n) => /\d$/.test(n) && !expected.includes(n) && declared.has(n.replace(/\d+$/, "")),
  );
  if (numbered.length) {
    throw new Error(
      `the components module declares counter-suffixed duplicates, which is the ` +
        `defect this module exists to remove: ${numbered.join(", ")}`,
    );
  }
}

async function emitIndex(generated) {
  // index.ts re-exports every generated module under a stable name.
  // For a payload schema at specs/vault/list/0.1/payload.schema.json the
  // export name is `VaultList_v0_1`.
  const lines = [
    "/** Generated by scripts/build-ts-bindings.mjs — DO NOT EDIT BY HAND. */",
    "",
    "// The hand-written §7.2 consumer pipeline. Re-exported flat (rather than",
    "// namespaced like the generated modules) because it is the framework API,",
    "// not one specification among many.",
    `export * from "./${RUNTIME_DIR}/index.js";`,
    "",
    "// Every cross-file definition, declared once. Namespaced rather than flat:",
    "// 150-odd names in the root export would collide with the runtime's on the",
    "// first shared definition anyone calls `Transport` or `Session`.",
    `export * as SharedComponents from "./${COMPONENTS_DIR}/${COMPONENTS_FILE.replace(/\.ts$/, ".js")}";`,
    "",
  ];
  for (const { outPath, slugInfo } of generated) {
    // `.js`, not extensionless. The package is ESM ("type": "module"), and Node
    // requires an explicit extension on a relative ESM specifier — it does not
    // probe for `.js` the way CommonJS resolution did. TypeScript's `Bundler`
    // moduleResolution accepts the extensionless form and emits it verbatim, so
    // `tsc` stayed quiet while `dist/index.js` shipped specifiers Node cannot
    // resolve: any `import … from "@openvtc/trust-tasks"` died with
    // ERR_MODULE_NOT_FOUND. It went unnoticed because the package was types
    // only until the runtime landed — `import type` is erased, and bundlers
    // tolerate extensionless paths. In TypeScript a `.js` specifier resolves to
    // the `.ts` source, so this is correct at both ends.
    const rel =
      "./" + path.relative(OUT_DIR, outPath).replace(/\\/g, "/").replace(/\.ts$/, ".js");
    if (slugInfo) {
      const id =
        slugInfo.slug
          .split("/")
          .map((s) =>
            s
              .split("-")
              .map((p) => p.charAt(0).toUpperCase() + p.slice(1))
              .join(""),
          )
          .join("") + `_v${slugInfo.version.replace(/\./g, "_")}`;
      lines.push(`export * as ${id} from ${JSON.stringify(rel)};`);
    } else {
      // Shared module — flat re-export, version-qualified with the same
      // `_v<major>_<minor>` suffix the task exports above carry. Without
      // the version the alias is derived from the basename alone, so a
      // shared schema published at both 0.1 and 0.2 (device-binding,
      // policy, sync-event, the vault/* shapes, the framework schema)
      // emits the same identifier twice and breaks `tsc` with TS2300.
      const version = rel
        .replace(/^\.\//, "")
        .split("/")
        .find((s) => /^\d+\.\d+$/.test(s));
      const name = path
        .basename(outPath, ".ts")
        .split("-")
        .map((p) => p.charAt(0).toUpperCase() + p.slice(1))
        .join("");
      const suffix = version ? `_v${version.replace(/\./g, "_")}` : "";
      lines.push(`export * as ${name}Shared${suffix} from ${JSON.stringify(rel)};`);
    }
  }

  // Fail here rather than downstream. The alias is derived from the basename
  // and version only, so it does NOT distinguish families: `config/_shared/
  // 0.1/config` and `vtc/_shared/0.1/config` both want `ConfigShared_v0_1`.
  // Emitting both is valid-looking output that breaks `tsc` with a TS2300 in
  // the generated `index.ts` — a diagnostic that points at the symptom and
  // says nothing about which two specs collided. Catch it at the source.
  const seen = new Map();
  const collisions = [];
  for (const line of lines) {
    const m = /^export \* as (\S+) from "(.+)";$/.exec(line);
    if (!m) continue;
    const [, alias, from] = m;
    if (seen.has(alias)) collisions.push(`${alias}: ${seen.get(alias)} vs ${from}`);
    else seen.set(alias, from);
  }
  if (collisions.length) {
    throw new Error(
      `duplicate TS export alias(es) — rename one of the colliding shared schemas ` +
        `(the alias comes from the file's basename + version, not its family):\n  ` +
        collisions.join("\n  "),
    );
  }

  await fs.writeFile(path.join(OUT_DIR, "index.ts"), lines.join("\n") + "\n", "utf8");
}

// Hand-written directories under src/ that the generator must not touch. The
// underscore prefix matches the `_shared` / `_framework` convention already
// used for non-task directories.
const HAND_WRITTEN = new Set([RUNTIME_DIR]);

async function clean() {
  // Wipe src/ so removed specs don't linger — but preserve the hand-written
  // runtime, which lives alongside the generated tree so that consumers get
  // types and the §7.2 consumer pipeline from one package (as trust-tasks-rs
  // does) rather than having to assemble them from two.
  await ensureDir(OUT_DIR);
  for (const entry of await fs.readdir(OUT_DIR)) {
    if (HAND_WRITTEN.has(entry)) continue;
    await fs.rm(path.join(OUT_DIR, entry), { recursive: true, force: true });
  }
}

async function main() {
  console.log("Trust Tasks TS bindings generator");
  await clean();

  const schemas = await walk(SPECS_DIR, /\.schema\.json$/);
  schemas.sort();

  // Skip two schemas that describe something other than a payload shape:
  //
  //   spec.meta.schema.json — front-matter metadata for spec authors.
  //   trust-task.schema.json — the document envelope (SPEC §4.2). Generating
  //     from it would put a second document type in the package, competing with
  //     the hand-written `TrustTaskDocument<P>` in src/_runtime. The generated
  //     one would be strictly worse — not generic over the payload, so unusable
  //     with consumeInbound — and having both invites picking the wrong one.
  const NOT_PAYLOAD_SCHEMAS = new Set(["spec.meta.schema.json", "trust-task.schema.json"]);
  const filtered = schemas.filter((p) => !NOT_PAYLOAD_SCHEMAS.has(path.basename(p)));

  // The component registry is built from the payload schemas alone: a `$def`
  // no payload reaches is duplicated nowhere and needs no hoisting.
  const registry = buildComponentRegistry(
    filtered.filter((p) => path.basename(p) === "payload.schema.json"),
  );
  console.log(
    `Hoisting ${registry.components.size} cross-file definitions into ` +
      `${registry.byName.size} types (${registry.multiShaped.length} names denote ` +
      `more than one shape and are family/version-qualified)...`,
  );
  for (const { baseName, shapes } of registry.multiShaped) {
    console.log(`  · ${baseName}: ${shapes} shapes`);
  }
  await emitComponentsModule(registry);

  console.log(`Generating bindings for ${filtered.length} schemas...`);
  const generated = [];
  for (const schemaPath of filtered) {
    try {
      const result = await generateOne(schemaPath, registry);
      generated.push(result);
      console.log("  ✓ " + path.relative(REPO_ROOT, schemaPath));
    } catch (err) {
      console.error("  ✗ " + path.relative(REPO_ROOT, schemaPath));
      console.error("    " + err.message);
      process.exitCode = 1;
    }
  }

  await emitIndex(generated);
  console.log(`\nGenerated ${generated.length} TS modules into ${path.relative(REPO_ROOT, OUT_DIR)}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
