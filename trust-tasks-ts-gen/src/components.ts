/**
 * The shared definitions of the locked tasks, declared once in
 * `_shared/components.ts`.
 *
 * An independent port of the component registry in
 * `scripts/build-ts-bindings.mjs` (`collectComponents`, `canonicalShape`,
 * `buildComponentRegistry`, `emitComponentsModule`, `componentClauses`), with
 * its input swapped from the shared schema files to the bundles' `defOrigins`.
 * It is deliberately NOT shared with that script (see CLAUDE.md). Where the two
 * must agree byte for byte — the names, the components module, the import and
 * re-export clauses — this follows the script closely so a reader can diff
 * them.
 *
 * ── Identity: a bundle's `defOrigins` ──────────────────────────────────────
 *
 * The script keys a component by its file and `$defs` name; a bundle records
 * the same identity as `<that file's $id>#/$defs/<name>`, and one shared
 * definition carries one origin in every bundle that splices it in. Its shape
 * is resolved through the bundle's own `$defs` (every `$ref` there is already
 * local) and compares equal to the script's `canonicalShape` of the file.
 *
 * ── Names: what the lock contains, not the registry ────────────────────────
 *
 * The script names a component bare when its `$defs` name (or `title`) has one
 * shape across the whole registry, and qualifies it with family and version
 * (`AclEntry_AclV0_2`) when it has several. A generator here sees only the
 * locked tasks, so "several" means several *in this lock*: an application
 * pinning only `acl/grant` 0.2 gets `AclEntry` bare. The script additionally
 * keeps every name it has ever published (`component-names.lock.json`), which
 * no bundle carries; so a name that shipped bare before a second shape appeared
 * (`TrustRecord`) is qualified here. Neither touches a spec module's public
 * surface, which re-exports each definition under its own name
 * (`AclGrant_v0_2.AclEntry`) — only the identifiers inside
 * `_shared/components.ts`, which the barrel does not export for that reason.
 */

import { compile, type Options } from "json-schema-to-typescript";

import { OFFICIAL_AUTHORITY, withAuthoredOrder, type Bundle, type Json, type JsonObject } from "./bundle.js";

/** Where the module lands, relative to the output directory. */
export const COMPONENTS_PATH = "_shared/components.ts";

// The synthetic root the components module is compiled through; stripped.
const COMPONENTS_ROOT_TYPE = "TrustTasksSharedComponentsRoot";

const isObject = (v: unknown): v is JsonObject => typeof v === "object" && v !== null && !Array.isArray(v);
const DEF_REF = /^#\/\$defs\/([^/]+)$/;
const DECLARATION_RE = /^export (?:interface|type) ([A-Za-z0-9_$]+)/gm;

/** The `$defs` name a `#/$defs/<name>` reference names, or null. */
export function defRefName(ref: unknown): string | null {
  if (typeof ref !== "string") return null;
  const m = DEF_REF.exec(ref);
  return m ? m[1]!.replace(/~1/g, "/").replace(/~0/g, "~") : null;
}

/** As the compiler mints a type name from a `title` (and as emit.ts does). */
export function titleTypeName(title: Json | undefined): string | null {
  if (typeof title !== "string") return null;
  const parts = title
    .split(/[^A-Za-z0-9]+/)
    .filter(Boolean)
    .map((word) => word[0]!.toUpperCase() + word.slice(1));
  return parts.length ? parts.join("") : null;
}

const pascal = (s: string): string =>
  s
    .split(/[^A-Za-z0-9]+/)
    .filter(Boolean)
    .map((p) => p.charAt(0).toUpperCase() + p.slice(1))
    .join("");

/** One shared definition: one origin, as one bundle spells it. */
interface Component {
  /** `<document $id>#/$defs/<name>`. */
  readonly origin: string;
  readonly defName: string;
  /** What the compiler would call it on its own: its `title`, else the `$defs` name. */
  readonly baseName: string;
  /** The fragment, `properties` in authored order. */
  readonly fragment: JsonObject;
  /** The fully-resolved shape, as a string. */
  readonly shape: string;
  /** Sort key: official first, then the origin document's authority and path. */
  readonly sortKey: string;
  /** The family/version tag that qualifies its name when it must be. */
  readonly tag: string;
  /** Resolve a `$defs` name in the bundle this fragment came from. */
  readonly resolve: (defName: string) => Component;
  tsName?: string;
}

/** The registry of every shared definition the locked bundles carry. */
export interface ComponentRegistry {
  /** The `_shared/components.ts` text, or `null` when there are no components. */
  readonly text: string | null;
  /** Bundle → its `$defs` name → component, for the bundles that carry origins. */
  readonly forBundle: (bundle: Bundle) => ReadonlyMap<string, ComponentRef>;
  /** Declared name → the declared names its declaration mentions. */
  readonly dependencies: ReadonlyMap<string, ReadonlySet<string>>;
  /** A qualified declared name → the name an importing module binds it to. */
  readonly displayNames: ReadonlyMap<string, string>;
}

/** What a spec module needs to know about one component. */
export interface ComponentRef {
  readonly tsName: string;
  readonly baseName: string;
}

/**
 * The origin document's path below its `/spec/` root, `/`-separated, without
 * `.schema.json` — `acl/_shared/0.2/acl-entry`, `trust-task/0.4/framework`,
 * `governance/_shared/0.1` (whose `$id` names a directory) — and the authority
 * above it (`https://tasks.example.com`, or `urn:example:acme` for a URN).
 */
function documentPath(origin: string): { official: boolean; authority: string; path: string } {
  const doc = origin.slice(0, origin.indexOf("#"));
  const strip = (p: string) => p.replace(/\.schema\.json$/, "");
  if (doc.startsWith(`${OFFICIAL_AUTHORITY}spec/`)) {
    return { official: true, authority: OFFICIAL_AUTHORITY.replace(/\/$/, ""), path: strip(doc.slice(`${OFFICIAL_AUTHORITY}spec/`.length)) };
  }
  const spec = doc.lastIndexOf("/spec/");
  if (spec >= 0) return { official: false, authority: doc.slice(0, spec), path: strip(doc.slice(spec + "/spec/".length)) };
  const urn = doc.indexOf(":spec:");
  if (urn >= 0) return { official: false, authority: doc.slice(0, urn), path: strip(doc.slice(urn + ":spec:".length)).replace(/:/g, "/") };
  return { official: false, authority: doc, path: "" };
}

/** Components sort official first, then by authority, then by document path. */
function sortKeyOf(origin: string): string {
  const { official, authority, path } = documentPath(origin);
  return official ? `0 ${path}` : `1 ${authority} ${path}`;
}

/**
 * The `<Family>V<major>_<minor>` tag of the script's `familyTag`, from an
 * origin's document path: `acl/_shared/0.2/acl-entry` → `AclV0_2`. The last
 * segment is the document's name unless it is a version (a `$id` naming a
 * directory). A document from a third-party source is prefixed with the
 * source's name (`AcmePaymentsV1_0`), so it never qualifies onto an official
 * name.
 *
 * The script reads the framework's documents from `specs/_framework/`, whose
 * `$id`s are `trust-task/<M.N>/framework`, so a qualified framework definition
 * would be tagged `TrustTaskV…` here and `FrameworkV…` there. None is
 * qualified: every framework definition has one shape per name.
 */
function familyTag(origin: string, source: string | null): string {
  const segments = documentPath(origin).path.split("/").filter(Boolean);
  if (segments.length > 0 && !/^\d+\.\d+$/.test(segments[segments.length - 1]!)) segments.pop();
  const version = /^\d+\.\d+$/.test(segments[segments.length - 1] ?? "") ? segments.pop()! : null;
  const family = segments
    .filter((s) => s !== "_shared")
    .flatMap((s) => s.replace(/^_/, "").split("-"))
    .map(pascal)
    .join("");
  return `${source === null ? "" : pascal(source)}${family}${version ? `V${version.replace(/\./g, "_")}` : ""}`;
}

function sortedKeys(value: Json): Json {
  if (Array.isArray(value)) return value.map(sortedKeys);
  if (!isObject(value)) return value;
  const out: JsonObject = {};
  for (const k of Object.keys(value).sort()) out[k] = sortedKeys(value[k]!);
  return out;
}

/**
 * A fragment's shape, fully resolved through `defs` and with object keys
 * ordered, as a string — the script's `canonicalShape`. A `$ref` already on
 * the stack is cut with a `$recursion` marker naming the definition.
 */
function canonicalShape(fragment: Json, defs: JsonObject, keyOf: (defName: string) => string): string {
  const visit = (node: Json, stack: ReadonlySet<string>): Json => {
    if (Array.isArray(node)) return node.map((n) => visit(n, stack));
    if (!isObject(node)) return node;
    if (typeof node.$ref === "string") {
      const rest: JsonObject = { ...node };
      delete rest.$ref;
      const defName = defRefName(node.$ref);
      if (defName === null || defs[defName] === undefined) throw new Error(`$ref ${node.$ref} names no definition in the bundle`);
      const key = keyOf(defName);
      if (stack.has(key)) return sortedKeys({ ...rest, $recursion: defName });
      return sortedKeys({ ...(visit(defs[defName]!, new Set([...stack, key])) as JsonObject), ...rest });
    }
    const out: JsonObject = {};
    for (const k of Object.keys(node).sort()) out[k] = visit(node[k]!, stack);
    return out;
  };
  return JSON.stringify(visit(fragment, new Set()));
}

/** The compiler options the components module is produced with (as emit.ts's). */
const COMPILE_OPTIONS: Partial<Options> = {
  additionalProperties: false,
  declareExternallyReferenced: true,
  enableConstEnums: false,
  strictIndexSignatures: true,
  unknownAny: true,
  style: { singleQuote: false, semi: true },
};

/** Remove one top-level declaration, by name, from compiled output. */
function stripDeclaration(ts: string, name: string): string {
  const lines = ts.split("\n");
  const start = lines.findIndex((l) => new RegExp(`^export (?:interface|type) ${name}\\b`).test(l));
  if (start < 0) throw new Error(`compiled output declares no ${name} to strip`);
  let depth = 0;
  let end = start;
  for (; end < lines.length; end++) {
    for (const ch of lines[end]!) {
      if (ch === "{" || ch === "[" || ch === "(") depth++;
      else if (ch === "}" || ch === "]" || ch === ")") depth--;
    }
    if (depth === 0 && (/;\s*$/.test(lines[end]!) || /^\}/.test(lines[end]!))) break;
  }
  lines.splice(start, end - start + 1);
  return lines.join("\n");
}

/** Every `title` below the root: the names json-schema-to-typescript gives nested branch types. */
function nestedTitles(schema: JsonObject): string[] {
  const out: string[] = [];
  const visit = (node: Json, isRoot: boolean): void => {
    if (Array.isArray(node)) return node.forEach((n) => visit(n, false));
    if (!isObject(node)) return;
    if (!isRoot && typeof node.title === "string") out.push(node.title);
    for (const value of Object.values(node)) visit(value, false);
  };
  visit(schema, true);
  return out;
}

/** Suffix every `title` below the root, recording the original name. */
function tagNestedTitles(schema: JsonObject, suffix: string, displayNames: Map<string, string>): void {
  const visit = (node: Json, isRoot: boolean): void => {
    if (Array.isArray(node)) return node.forEach((n) => visit(n, false));
    if (!isObject(node)) return;
    if (!isRoot && typeof node.title === "string") {
      const tagged = `${node.title}${suffix}`;
      displayNames.set(tagged, node.title);
      node.title = tagged;
    }
    for (const value of Object.values(node)) visit(value, false);
  };
  visit(schema, true);
}

export interface RegistryInput {
  readonly bundle: Bundle;
  readonly source: string | null;
}

/**
 * Collect, shape, name and compile every shared definition the given bundles
 * carry. Bundles without `defOrigins` (written before it existed) contribute
 * nothing; their modules declare their definitions themselves (emit.ts).
 */
export async function buildComponentRegistry(inputs: readonly RegistryInput[]): Promise<ComponentRegistry> {
  // origin → shape → component. One origin normally has one shape; two bundles
  // pinned from different registry snapshots can disagree, if the shared file
  // gained a member in place between them — both are kept, as two types.
  const byOrigin = new Map<string, Map<string, Component>>();
  const perBundle = new Map<Bundle, Map<string, Component>>();

  for (const { bundle, source } of inputs) {
    const origins = bundle.defOrigins;
    if (origins === null || Object.keys(origins).length === 0) continue;
    const defs = (withAuthoredOrder(bundle).schema.$defs ?? {}) as JsonObject;
    const local = new Map<string, Component>();
    const keyOf = (defName: string): string => origins[defName] ?? `#own/${defName}`;
    for (const defName of Object.keys(origins).sort()) {
      const origin = origins[defName]!;
      const fragment = defs[defName];
      if (!isObject(fragment)) throw new Error(`${bundle.slug}/${bundle.version}: $defs/${defName} is not a schema object`);
      const shape = canonicalShape(fragment, defs, keyOf);
      let variants = byOrigin.get(origin);
      if (!variants) byOrigin.set(origin, (variants = new Map()));
      let component = variants.get(shape);
      if (!component) {
        const originDef = origin.slice(origin.indexOf("#/$defs/") + "#/$defs/".length);
        component = {
          origin,
          defName: originDef,
          baseName: titleTypeName(fragment.title) ?? originDef,
          fragment,
          shape,
          sortKey: sortKeyOf(origin),
          tag: familyTag(origin, origin.startsWith(OFFICIAL_AUTHORITY) ? null : source),
          resolve: (name: string) => {
            const target = local.get(name);
            if (!target) throw new Error(`${origin}: $ref #/$defs/${name} leaves the shared definitions of ${bundle.typeUri}`);
            return target;
          },
        };
        variants.set(shape, component);
      }
      local.set(defName, component);
    }
    perBundle.set(bundle, local);
  }

  const components = [...byOrigin.values()].flatMap((v) => [...v.values()]);
  const byName = nameComponents(components);
  const dependencies = new Map<string, Set<string>>();
  const displayNames = new Map<string, string>();
  for (const [name, entry] of byName) if (name !== entry.baseName) displayNames.set(name, entry.baseName);
  const text = byName.size === 0 ? null : await compileComponents(byName, displayNames, dependencies);

  return {
    text,
    forBundle: (bundle) => {
      const local = perBundle.get(bundle);
      const out = new Map<string, ComponentRef>();
      for (const [defName, c] of local ?? []) out.set(defName, { tsName: c.tsName!, baseName: c.baseName });
      return out;
    },
    dependencies,
    displayNames,
  };
}

interface NamedEntry {
  readonly name: string;
  readonly baseName: string;
  readonly fragment: JsonObject;
  readonly representative: Component;
}

/**
 * Give every component a TypeScript name — the script's
 * `buildComponentRegistry`, step 2 (there is no name lock here; see the file
 * header). Components are grouped by base name, then by shape; a group is one
 * TypeScript type. A base name with one shape among the locked tasks is bare;
 * otherwise every shape is qualified with its earliest document's family and
 * version.
 */
function nameComponents(components: readonly Component[]): Map<string, NamedEntry> {
  const byBase = new Map<string, Component[]>();
  for (const c of components) {
    if (!byBase.has(c.baseName)) byBase.set(c.baseName, []);
    byBase.get(c.baseName)!.push(c);
  }
  interface Group {
    baseName: string;
    sources: Component[];
    shapes: number;
  }
  const bySort = (a: Component, b: Component): number =>
    a.sortKey < b.sortKey ? -1 : a.sortKey > b.sortKey ? 1 : a.origin < b.origin ? -1 : a.origin > b.origin ? 1 : a.shape < b.shape ? -1 : a.shape > b.shape ? 1 : 0;
  const groups: Group[] = [];
  for (const [baseName, list] of byBase) {
    const byShape = new Map<string, Component[]>();
    for (const c of list) {
      if (!byShape.has(c.shape)) byShape.set(c.shape, []);
      byShape.get(c.shape)!.push(c);
    }
    for (const sources of byShape.values()) groups.push({ baseName, sources: sources.sort(bySort), shapes: byShape.size });
  }
  groups.sort((a, b) => bySort(a.sources[0]!, b.sources[0]!));

  const byName = new Map<string, NamedEntry>();
  for (const group of groups) {
    const bare = group.shapes === 1 && !byName.has(group.baseName);
    let name = bare ? group.baseName : `${group.baseName}_${group.sources[0]!.tag}`;
    // The script refuses two shapes wanting one qualified name (one family and
    // version). Here that also happens when one origin has two shapes — two
    // snapshots of one shared file in one lock — so number the later ones
    // rather than refuse a lock the CLI wrote.
    if (byName.has(name)) {
      let n = 2;
      while (byName.has(`${name}_${n}`)) n++;
      name = `${name}_${n}`;
    }
    const representative = group.sources[0]!;
    byName.set(name, { name, baseName: group.baseName, fragment: representative.fragment, representative });
    for (const c of group.sources) c.tsName = name;
  }
  for (const reserved of ["Payload", "Response", "SPEC", "RESPONSE_SPEC", "TYPE_URI"]) {
    if (byName.has(reserved)) throw new Error(`a shared definition is named ${reserved}, which every spec module also exports`);
  }
  return byName;
}

/** Rewrite every `$ref` in a component's fragment to a `tsType` naming the component it resolves to. */
function refsToComponentNames(node: Json, owner: Component): void {
  if (Array.isArray(node)) node.forEach((n) => refsToComponentNames(n, owner));
  else if (isObject(node)) {
    const defName = defRefName(node.$ref);
    if (defName !== null) {
      delete node.$ref;
      node.tsType = owner.resolve(defName).tsName!;
      return;
    }
    for (const value of Object.values(node)) refsToComponentNames(value, owner);
  }
}

/**
 * Compile every component into one module through a synthetic root — the
 * script's `emitComponentsModule` — and index which declaration mentions
 * which (`indexDeclarations`).
 */
async function compileComponents(
  byName: ReadonlyMap<string, NamedEntry>,
  displayNames: Map<string, string>,
  dependencies: Map<string, Set<string>>,
): Promise<string> {
  const names = [...byName.keys()].sort();
  const $defs: JsonObject = {};
  const properties: JsonObject = {};
  const schemas = new Map<string, JsonObject>();
  for (const name of names) {
    const entry = byName.get(name)!;
    const schema = structuredClone(entry.fragment);
    refsToComponentNames(schema, entry.representative);
    schemas.set(name, schema);
  }
  // A qualified component's nested branch types are tagged with its suffix, so
  // they cannot collide. A bare one's keep their own titles, and those can: a
  // lock that reaches one shape of `ConsumerKind` (bare) and the `Service` of
  // `auth/oob` (bare) has two declarations wanting `Service`, and the compiler
  // would number the second. The package never meets this — over the whole
  // registry `ConsumerKind` has two shapes and is qualified — so tag such a
  // component's nested titles with its family and version, which is the name
  // the package gives the same branch.
  const titleUses = new Map<string, number>();
  for (const name of names) {
    if (name === byName.get(name)!.baseName) {
      for (const title of new Set(nestedTitles(schemas.get(name)!))) titleUses.set(title, (titleUses.get(title) ?? 0) + 1);
    }
  }
  for (const name of names) {
    const entry = byName.get(name)!;
    const schema = schemas.get(name)!;
    if (name !== entry.baseName) tagNestedTitles(schema, name.slice(entry.baseName.length), displayNames);
    else if (nestedTitles(schema).some((t) => byName.has(t) || titleUses.get(t)! > 1)) {
      tagNestedTitles(schema, `_${entry.representative.tag}`, displayNames);
    }
    $defs[name] = { ...schema, title: name };
    properties[name] = { $ref: `#/$defs/${name}` };
  }

  const banner =
    "/**\n" +
    " * Generated by trust-tasks-gen-ts (@openvtc/trust-tasks-gen) — DO NOT EDIT BY HAND.\n" +
    " *\n" +
    " * Every shared definition the locked payload schemas reach, declared once.\n" +
    " * Spec modules import from here and re-export what they use under the\n" +
    " * definition's own name — that is the name to use. An identifier here is\n" +
    " * qualified with its family and version when the locked tasks reach more\n" +
    " * than one shape of its name, so it can change as the lock changes.\n" +
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
      } as never,
      "components.schema.json",
      { ...COMPILE_OPTIONS, bannerComment: banner },
    ),
    COMPONENTS_ROOT_TYPE,
  );

  const declared = new Set([...ts.matchAll(DECLARATION_RE)].map((m) => m[1]!));
  const missing = names.filter((n) => !declared.has(n));
  if (missing.length) {
    throw new Error(`the components module does not declare ${missing.join(", ")} — json-schema-to-typescript renamed them`);
  }
  const numbered = [...declared].filter((n) => /\d$/.test(n) && !names.includes(n) && declared.has(n.replace(/\d+$/, "")));
  if (numbered.length) throw new Error(`the components module declares counter-suffixed duplicates: ${numbered.join(", ")}`);

  const lines = ts.replace(/\/\*[\s\S]*?\*\//g, "").split("\n");
  const starts: { name: string; line: number }[] = [];
  lines.forEach((line, i) => {
    const m = /^export (?:interface|type) ([A-Za-z0-9_$]+)/.exec(line);
    if (m) starts.push({ name: m[1]!, line: i });
  });
  starts.forEach(({ name, line }, i) => {
    const body = lines.slice(line, i + 1 < starts.length ? starts[i + 1]!.line : lines.length).join("\n");
    dependencies.set(name, new Set([...body.matchAll(/[A-Za-z0-9_$]+/g)].map((m) => m[0]).filter((id) => id !== name && declared.has(id))));
  });
  return ts.replace(/\n{3,}/g, "\n\n");
}

/** A relative ESM specifier from a generated module to the components module. Always `.js`. */
function componentsSpecifier(modulePath: string): string {
  const from = modulePath.split("/").slice(0, -1);
  const to = COMPONENTS_PATH.replace(/\.ts$/, ".js").split("/");
  let common = 0;
  while (common < from.length && common < to.length - 1 && from[common] === to[common]) common++;
  const rel = [...from.slice(common).map(() => ".."), ...to.slice(common)].join("/");
  return rel.startsWith(".") ? rel : `./${rel}`;
}

/** One component a spec module uses: the hoisted name, and the name the module binds it to. */
export interface UsedComponent {
  readonly tsName: string;
  readonly local: string;
}

/**
 * The `import type { … }` and `export type { … }` clauses a spec module needs —
 * the script's `componentClauses`. Only what survived compilation is imported;
 * the re-export covers the closure of what that mentions inside the components
 * module, each under its display name unless that would shadow something.
 */
export function componentClauses(
  used: ReadonlyMap<string, UsedComponent>,
  modulePath: string,
  ts: string,
  registry: ComponentRegistry,
  alsoUsed = "",
): { imports: string; reexports: string } {
  if (used.size === 0) return { imports: "", reexports: "" };
  const code = `${ts}\n${alsoUsed}`.replace(/\/\*[\s\S]*?\*\//g, "");
  const direct = [...used.values()].filter(({ local }) => new RegExp(`\\b${local}\\b`).test(code));
  if (direct.length === 0) return { imports: "", reexports: "" };

  const declaredHere = new Set([...ts.matchAll(DECLARATION_RE)].map((m) => m[1]!));
  const directLocals = new Map(direct.map(({ tsName, local }) => [tsName, local]));
  for (const [tsName, local] of directLocals) {
    if (declaredHere.has(local)) {
      throw new Error(
        `${modulePath}: shared definition ${tsName} would be imported as ${local}, which this module also declares — ` +
          `the schema's own $defs needs a distinct name`,
      );
    }
  }

  const closure = new Set(directLocals.keys());
  const frontier = [...closure];
  while (frontier.length > 0) {
    for (const dep of registry.dependencies.get(frontier.pop()!) ?? []) {
      if (!closure.has(dep)) {
        closure.add(dep);
        frontier.push(dep);
      }
    }
  }

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

  const specifier = JSON.stringify(componentsSpecifier(modulePath));
  const clause = bindings.map(({ tsName, local }) => (tsName === local ? local : `${tsName} as ${local}`)).join(", ");
  return {
    imports: `\nimport type { ${clause} } from ${specifier};\n`,
    reexports:
      `\n/** Shared definitions this specification references, re-exported under the names it used to declare them with. */\n` +
      `export type { ${bindings.map(({ local }) => local).join(", ")} };\n`,
  };
}
