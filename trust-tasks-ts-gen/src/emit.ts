/**
 * One bundle → one TypeScript module.
 *
 * An independent port of the per-module half of `scripts/build-ts-bindings.mjs`
 * in this repository, with its input swapped from the `specs/` tree to a
 * registry bundle. It is deliberately NOT shared with that script (the
 * repository keeps one independent generator per language and holds them to
 * agreement with `check-bindings`; see CLAUDE.md). Where this file and the
 * script must agree byte for byte — the module tail, the `Payload` / `Response`
 * aliasing, the deprecation markers — the code follows the script closely so
 * a reader can diff the two.
 *
 * ── Shared definitions: hoisted when the bundle says which they are ────────
 *
 * The in-repo generator hoists every definition a payload schema reaches by a
 * cross-file `$ref` into one `_shared/components.ts`, imports it into each
 * module, and re-exports it there under its own name. A bundle's `defOrigins`
 * names exactly those definitions (and the shared document each came from), so
 * a module generated from such a bundle does the same: see components.ts, which
 * declares them once for the whole output directory, and `generateModule`
 * below. The payload schema's own `$defs` are left to the compiler, as there.
 *
 * A bundle written before `defOrigins` existed cannot tell a shared definition
 * from the schema's own. Its module declares every definition it reaches
 * itself, once each, under the name the prebuilt module exports it by; see
 * "Declaring each definition once per module" below. TypeScript is
 * structurally typed, so an `Ext` from one such module and the hoisted `Ext`
 * are mutually assignable; what is lost is only the single declaration.
 */

import { compile, type Options } from "json-schema-to-typescript";

import { withAuthoredOrder, type Bundle, type Json, type JsonObject } from "./bundle.js";
import { componentClauses, defRefName, titleTypeName, type ComponentRegistry, type UsedComponent } from "./components.js";

/** What generated a module, for its banner. */
export const GENERATOR = "trust-tasks-gen-ts (@openvtc/trust-tasks-gen)";

// Synthetic property name used to drag `$defs.Response` into the compiler's
// reachability graph. Deliberately ugly so it cannot collide with a real
// payload member; it is stripped from the output before the file is written.
const RESPONSE_PROBE = "__ttResponseProbe";

// Annotation keywords that do not constrain the instance, so a `$defs.Response`
// carrying only these alongside `$ref` is a pure alias for its target.
const RESPONSE_ANNOTATIONS = new Set(["$ref", "$anchor", "title", "description"]);

const isObject = (v: unknown): v is JsonObject => typeof v === "object" && v !== null && !Array.isArray(v);

/**
 * The `$ref` the response probe should point at: the alias's target where
 * `$defs.Response` is a bare alias, so the response type is the target's own
 * name (as in the prebuilt module), else `#/$defs/Response`.
 */
function responseProbeRef(response: JsonObject): JsonObject {
  const isPureAlias =
    typeof response.$ref === "string" && Object.keys(response).every((k) => RESPONSE_ANNOTATIONS.has(k));
  return isPureAlias ? { $ref: response.$ref as string } : { $ref: "#/$defs/Response" };
}

const DECLARATION_RE = /^export (?:interface|type) ([A-Za-z0-9_$]+)/gm;

/**
 * Name of the root interface/type the compiler emitted: the one named from the
 * schema's own `title` when that is present in the output, else the first.
 * An object-rooted schema whose root compiled to a bare alias is refused —
 * that would publish a shared definition as the specification's `Payload`.
 */
function rootTypeName(ts: string, where: string, raw: JsonObject): string {
  const names = [...ts.matchAll(DECLARATION_RE)].map((m) => m[1]!);
  if (names.length === 0) throw new Error(`${where}: compiled output declares no root type`);
  const fromTitle = titleTypeName(raw.title);
  const root = fromTitle && names.includes(fromTitle) ? fromTitle : names[0]!;
  const bareAlias = new RegExp(`^export type ${root} = ([A-Za-z0-9_$]+);\\s*$`, "m");
  if (raw.type === "object" && bareAlias.test(ts)) {
    throw new Error(
      `${where}: root type '${root}' compiled to a bare alias, but the schema's root is an object — ` +
        `the alias would be emitted as this specification's \`Payload\`.`,
    );
  }
  return root;
}

/** Remove the response probe from compiled output and return the type it resolved to. */
function stripResponseProbe(ts: string, where: string): { ts: string; responseType: string } {
  const probe = new RegExp(`^[ \\t]*${RESPONSE_PROBE}\\?: (.+);[ \\t]*\\r?\\n`, "m");
  const m = probe.exec(ts);
  if (!m) {
    throw new Error(
      `${where}: response probe did not survive compilation — $defs.Response could not be resolved to a TypeScript type`,
    );
  }
  const stripped = ts.replace(probe, "");
  if (probe.test(stripped)) throw new Error(`${where}: response probe matched more than once`);
  return { ts: stripped, responseType: m[1]! };
}

/**
 * Add `@deprecated <note>` to the doc comment of the `export interface|type
 * <name>` declared in `ts`, creating the comment if there is none.
 */
function markDeprecated(ts: string, name: string, note: string): string {
  const decl = new RegExp(`^export (?:interface|type) ${name}\\b`, "m");
  const m = decl.exec(ts);
  if (!m) return ts;
  const before = ts.slice(0, m.index);
  const after = ts.slice(m.index);
  if (/(^|\n) \*\/\n$/.test(before)) {
    const close = before.lastIndexOf(" */");
    return before.slice(0, close) + ` *\n * @deprecated ${note}\n` + before.slice(close) + after;
  }
  return before + `/** @deprecated ${note} */\n` + after;
}

interface TailInput {
  typeUri: string;
  extendsVersion: string | null;
  deprecation: string | null;
  bundle: Bundle;
  schemas: { request: JsonObject; response: JsonObject | null };
}

/** The constants every module ends with — byte-compatible with the prebuilt modules' tail. */
function emitTail(ts: string, rootType: string, responseType: string | null, where: string, t: TailInput): string {
  const { typeUri, deprecation, bundle, schemas } = t;
  const policy = bundle.policy;
  const aliasDoc = (what: string): string[] =>
    deprecation
      ? [`/**`, ` * Stable alias for this specification's ${what} shape.`, ` *`, ` * @deprecated ${deprecation}`, ` */`]
      : [`/** Stable alias for this specification's ${what} shape. */`];
  const lines: string[] = [
    "",
    `/** Trust Task type URI. */`,
    `export const TYPE_URI = ${JSON.stringify(typeUri)} as const;`,
    "",
    `/**`,
    ` * The earlier version of this specification that this one declares it`,
    ` * extends (front matter \`extends\`, SPEC §5.2 "Declared extension"), or`,
    ` * \`null\`. Only the direct declaration — follow the chain with`,
    ` * \`extendsChain\` / \`readForSpec\`.`,
    ` */`,
    `export const EXTENDS: string | null = ${t.extendsVersion ? JSON.stringify(t.extendsVersion) : "null"};`,
    "",
    `/**`,
    ` * Why this version is deprecated, or \`null\` while it is current. A`,
    ` * superseded or retired version is still supported, and is withdrawn from`,
    ` * this package in a later scheduled batch (SPEC "Superseded and Withdrawn`,
    ` * Versions").`,
    ` */`,
    `export const DEPRECATION: string | null = ${JSON.stringify(deprecation ?? null)};`,
    "",
    ...aliasDoc("request payload"),
  ];
  if (rootType === "Payload") {
    lines.push(`// (the root interface is already named \`Payload\`)`);
  } else if (/^export (?:interface|type) Payload\b/m.test(ts)) {
    throw new Error(`${where}: cannot alias Payload — the name is already taken`);
  } else {
    lines.push(`export type Payload = ${rootType};`);
  }
  lines.push("");

  if (responseType && responseType !== "Response" && /^export (?:interface|type) Response\b/m.test(ts)) {
    throw new Error(`${where}: cannot alias Response — the name is already taken`);
  }
  if (responseType) {
    lines.push(
      `/** Trust Task response type URI (request type URI + "#response"). */`,
      `export const RESPONSE_TYPE_URI = ${JSON.stringify(typeUri + "#response")} as const;`,
      "",
      ...aliasDoc("success-response payload"),
      ...(responseType === "Response"
        ? [`// (the response interface is already named \`Response\`)`]
        : [`export type Response = ${responseType};`]),
      "",
    );
  }

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

  const obj = (uri: string, isProofRequired: boolean, isRecipientRequired: boolean, isIssuedAtRequired: boolean, schemaConst: string): string =>
    [
      `{`,
      `  typeUri: ${uri},`,
      `  isBearer: ${policy.bearer},`,
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
      policy.proofRequired.request,
      policy.recipientRequired.request,
      policy.issuedAtRequired.request,
      "PAYLOAD_SCHEMA",
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
        policy.proofRequired.response,
        policy.recipientRequired.response,
        policy.issuedAtRequired.response,
        schemas.response ? "RESPONSE_PAYLOAD_SCHEMA" : "undefined",
      )}`,
      "",
    );
  }
  return lines.join("\n");
}

/** The compiler options every generated module is produced with (as the in-repo generator's). */
const COMPILE_OPTIONS: Partial<Options> = {
  additionalProperties: false,
  declareExternallyReferenced: true,
  enableConstEnums: false,
  strictIndexSignatures: true,
  unknownAny: true,
  style: { singleQuote: false, semi: true },
};

function compileOptions(bundle: Bundle, source: string | null): Partial<Options> {
  const from = source === null ? "registry bundle" : `bundle from source "${source}"`;
  return {
    ...COMPILE_OPTIONS,
    bannerComment: `/**\n * Generated by ${GENERATOR} — DO NOT EDIT BY HAND.\n * Source: ${bundle.typeUri} (${from})\n */`,
  };
}

/** A generated module: its path relative to the output directory, and its text. */
export interface GeneratedModule {
  /**
   * `<slug>/<M.N>/payload.ts`, or `<source>/<slug>/<M.N>/payload.ts` for a
   * task from a third-party source; `/`-separated.
   */
  readonly path: string;
  /** The third-party source, or `null` for the official registry. */
  readonly source: string | null;
  readonly text: string;
  readonly slug: string;
  readonly version: string;
  readonly typeUri: string;
  readonly extendsVersion: string | null;
}

/* ── Declaring each definition once per module (bundles without defOrigins) ─
 *
 * Left to itself the compiler declares a `$ref`'d definition afresh wherever a
 * reference carries sibling keywords (a `description`, as nearly every one
 * does), numbering the copies: `approve?: AuthorityScope1`, `ext?: Ext1`. The
 * in-repo generator avoids that by rewriting each cross-file `$ref` to the
 * compiler's `tsType` escape hatch and declaring the definition once.
 *
 * This does the same within the module: every `$defs` entry the payload or its
 * response reaches (other than `Response` itself, which the response probe
 * compiles) is referenced by name and declared once, below the root
 * declarations. Since a bundle without `defOrigins` cannot say which `$defs`
 * came from a shared file and which were the schema's own, it treats them all
 * alike — so a schema's own
 * definitions are declared this way too, where the in-repo generator left
 * those to the compiler.
 */

const IDENTIFIER = /^[A-Za-z_$][A-Za-z0-9_$]*$/;
const DEFS_ROOT_TYPE = "TrustTasksModuleDefinitionsRoot";

/** Every `$defs` name referenced anywhere under `node`. */
function referencedDefs(node: Json, out: Set<string>): Set<string> {
  if (Array.isArray(node)) node.forEach((n) => referencedDefs(n, out));
  else if (isObject(node)) {
    const name = defRefName(node.$ref);
    if (name !== null) out.add(name);
    for (const value of Object.values(node)) referencedDefs(value, out);
  }
  return out;
}

/**
 * The definitions to declare once, by `$defs` name → TypeScript name: every
 * `$defs` entry reachable from the root or from `$defs.Response`, except
 * `Response`. The TypeScript name is what the compiler would have called it on
 * its own — a `title` if it has one, else the `$defs` key — which is also the
 * name the prebuilt module re-exports it under.
 */
function moduleDefinitions(raw: JsonObject, where: string): Map<string, string> {
  const defs = isObject(raw.$defs) ? raw.$defs : {};
  const reached = new Set<string>();
  const frontier: string[] = [];
  const visit = (node: Json) => {
    for (const name of referencedDefs(node, new Set())) {
      if (!reached.has(name)) {
        reached.add(name);
        frontier.push(name);
      }
    }
  };
  for (const [key, value] of Object.entries(raw)) if (key !== "$defs") visit(value);
  if (defs.Response !== undefined) visit(defs.Response);
  while (frontier.length > 0) {
    const name = frontier.pop()!;
    if (defs[name] === undefined) throw new Error(`${where}: $ref #/$defs/${name} names no definition`);
    visit(defs[name]!);
  }
  reached.delete("Response");

  const names = new Map<string, string>();
  const taken = new Map<string, string>();
  for (const key of [...reached].sort()) {
    const fragment = defs[key];
    const fromTitle = isObject(fragment) ? titleTypeName(fragment.title) : null;
    const candidates = [fromTitle, IDENTIFIER.test(key) ? key : titleTypeName(key)].filter(
      (n): n is string => typeof n === "string" && IDENTIFIER.test(n),
    );
    const name = candidates.find((n) => !taken.has(n));
    if (name === undefined) {
      throw new Error(
        `${where}: $defs/${key} would be declared as ${candidates[0] ?? JSON.stringify(key)}, which $defs/${taken.get(candidates[0] ?? "")} already is`,
      );
    }
    taken.set(name, key);
    names.set(key, name);
  }
  for (const reserved of ["Payload", "Response", "SPEC", "RESPONSE_SPEC", "TYPE_URI"]) {
    if (taken.has(reserved)) {
      throw new Error(`${where}: $defs/${taken.get(reserved)} would be declared as ${reserved}, which every module also exports`);
    }
  }
  return names;
}

/** Rewrite each `$ref` to a declared definition into a `tsType` naming it, in place. */
function refsToNames(node: Json, names: ReadonlyMap<string, string>): void {
  if (Array.isArray(node)) node.forEach((n) => refsToNames(n, names));
  else if (isObject(node)) {
    const name = defRefName(node.$ref);
    if (name !== null && names.has(name)) {
      delete node.$ref;
      node.tsType = names.get(name)!;
      return;
    }
    for (const value of Object.values(node)) refsToNames(value, names);
  }
}

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

/**
 * Compile the module's definitions in one pass, through a synthetic root that
 * reaches them all, and return their declarations (the root stripped).
 */
async function compileDefinitions(raw: JsonObject, names: ReadonlyMap<string, string>, where: string): Promise<string> {
  if (names.size === 0) return "";
  const defs = raw.$defs as JsonObject;
  const $defs: JsonObject = {};
  const properties: JsonObject = {};
  for (const [key, name] of [...names].sort(([, a], [, b]) => (a < b ? -1 : a > b ? 1 : 0))) {
    const fragment = structuredClone(defs[key]!);
    refsToNames(fragment, names);
    if (isObject(fragment) && defRefName(fragment.$ref) === "Response") {
      throw new Error(`${where}: $defs/${key} aliases $defs/Response, which this generator cannot declare twice`);
    }
    // `title` is what the compiler names a definition from, so it decides the
    // emitted identifier.
    $defs[name] = isObject(fragment) ? { ...fragment, title: name } : fragment;
    properties[name] = { $ref: `#/$defs/${name}` };
  }
  if (referencedDefs($defs, new Set()).has("Response")) {
    throw new Error(`${where}: a definition references $defs/Response, which this generator cannot declare twice`);
  }
  const ts = stripDeclaration(
    await compile(
      { title: DEFS_ROOT_TYPE, type: "object", additionalProperties: false, properties, $defs } as never,
      "definitions.schema.json",
      { ...COMPILE_OPTIONS, bannerComment: "" },
    ),
    DEFS_ROOT_TYPE,
  );
  const declared = new Set([...ts.matchAll(DECLARATION_RE)].map((m) => m[1]!));
  const missing = [...names.values()].filter((n) => !declared.has(n));
  if (missing.length) {
    throw new Error(`${where}: the compiler renamed definition(s) ${missing.join(", ")} — they would not resolve`);
  }
  return ts.replace(/\n{3,}/g, "\n\n").trim();
}

/** Insert `block` immediately after the generated banner comment. */
function spliceAfterBanner(ts: string, block: string): string {
  if (!block) return ts;
  const end = ts.indexOf("*/");
  if (end < 0) throw new Error("compiled output carries no banner comment to splice after");
  return `${ts.slice(0, end + 2)}\n${block}${ts.slice(end + 2)}`;
}

/** The module's path relative to the output directory. */
function modulePath(bundle: Bundle, source: string | null): string {
  return `${source === null ? "" : `${source}/`}${bundle.slug}/${bundle.version}/payload.ts`;
}

/**
 * Generate the module for one (verified) bundle. With a registry and a bundle
 * that carries `defOrigins`, its shared definitions are imported from the
 * registry's `_shared/components.ts`; otherwise they are declared in the module.
 */
export async function generateModule(
  bundle: Bundle,
  source: string | null = null,
  registry: ComponentRegistry | null = null,
): Promise<GeneratedModule> {
  if (registry !== null && bundle.defOrigins !== null) return generateHoistedModule(bundle, source, registry);
  return generateSelfContainedModule(bundle, source);
}

/**
 * The in-repo generator's `generateOne`, for a bundle that says which of its
 * `$defs` are shared: the payload schema is compiled as written — its own
 * `$defs` only, in authored order — with every `$ref` to a shared definition
 * rewritten to the hoisted type's name.
 */
async function generateHoistedModule(bundle: Bundle, source: string | null, registry: ComponentRegistry): Promise<GeneratedModule> {
  const where = `${source === null ? "" : `@${source}/`}${bundle.slug}/${bundle.version}`;
  const outPath = modulePath(bundle, source);
  const origins = bundle.defOrigins!;
  const { schema: full, responseSchema } = withAuthoredOrder(bundle);
  const shared = registry.forBundle(bundle);

  // The schema as its author wrote it: the spliced definitions removed, and
  // `$defs` last, where a schema file keeps it — that is the order the script
  // meets `$ref`s in, which decides which of two same-named definitions gets
  // the bare name below.
  const raw: JsonObject = {};
  for (const [key, value] of Object.entries(full)) if (key !== "$defs") raw[key] = value;
  const own: JsonObject = {};
  for (const [key, value] of Object.entries(isObject(full.$defs) ? full.$defs : {})) {
    if (!Object.prototype.hasOwnProperty.call(origins, key)) own[key] = value;
  }
  if (Object.keys(own).length > 0) raw.$defs = own;
  const response = own.Response;
  const hasResponse = isObject(response);

  const input: JsonObject = structuredClone(
    hasResponse
      ? {
          ...raw,
          properties: { ...(isObject(raw.properties) ? raw.properties : {}), [RESPONSE_PROBE]: responseProbeRef(response) },
        }
      : raw,
  );

  // The script's `rewriteRefsToComponents`: each `$ref` to a shared definition
  // becomes a `tsType` naming it locally. Two different shared definitions
  // with one base name in one module cannot both hold it; the first met keeps
  // it and the other keeps its qualified name.
  const used = new Map<string, UsedComponent>();
  const localNames = new Map<string, string>();
  const claimed = new Map<string, string>();
  const localNameFor = (tsName: string, baseName: string): string => {
    const existing = localNames.get(tsName);
    if (existing !== undefined) return existing;
    const claimant = claimed.get(baseName);
    const local = claimant === undefined ? baseName : tsName;
    if (claimant === undefined) claimed.set(baseName, tsName);
    localNames.set(tsName, local);
    used.set(tsName, { tsName, local });
    return local;
  };
  const rewrite = (node: Json): void => {
    if (Array.isArray(node)) return node.forEach(rewrite);
    if (!isObject(node)) return;
    const name = defRefName(node.$ref);
    if (name !== null && Object.prototype.hasOwnProperty.call(origins, name)) {
      const component = shared.get(name);
      if (!component) throw new Error(`${where}: $defs/${name} is shared, but the registry has no component for it`);
      delete node.$ref;
      node.tsType = localNameFor(component.tsName, component.baseName);
      return;
    }
    for (const value of Object.values(node)) rewrite(value);
  };
  rewrite(input);

  let ts = await compile(input as never, "payload.schema.json", compileOptions(bundle, source));
  let responseType: string | null = null;
  if (hasResponse) ({ ts, responseType } = stripResponseProbe(ts, where));
  if (responseSchema === null && responseType !== null) {
    throw new Error(`${where}: the schema declares $defs.Response but the bundle carries no responseSchema`);
  }

  const rootType = rootTypeName(ts, where, raw);
  const deprecation = bundle.deprecation;
  if (deprecation) {
    ts = markDeprecated(ts, rootType, deprecation);
    if (responseType && /^[A-Za-z0-9_$]+$/.test(responseType) && responseType !== rootType) {
      ts = markDeprecated(ts, responseType, deprecation);
    }
  }
  const tail = emitTail(ts, rootType, responseType, where, {
    typeUri: bundle.typeUri,
    extendsVersion: bundle.extends,
    deprecation,
    bundle,
    schemas: { request: full, response: responseType ? responseSchema : null },
  });

  // Imports above the compiler's output and re-exports below it, as the script.
  const { imports, reexports } = componentClauses(used, outPath, ts, registry, responseType ?? "");
  return {
    path: outPath,
    source,
    text: spliceAfterBanner(ts, imports) + reexports + tail,
    slug: bundle.slug,
    version: bundle.version,
    typeUri: bundle.typeUri,
    extendsVersion: bundle.extends,
  };
}

/** A bundle without `defOrigins`: every definition declared in the module itself. */
async function generateSelfContainedModule(bundle: Bundle, source: string | null): Promise<GeneratedModule> {
  const where = `${source === null ? "" : `@${source}/`}${bundle.slug}/${bundle.version}`;
  const { schema: raw, responseSchema } = withAuthoredOrder(bundle);
  const defs = isObject(raw.$defs) ? raw.$defs : {};
  const response = defs.Response;
  const hasResponse = isObject(response);
  const names = moduleDefinitions(raw, where);

  const input: JsonObject = structuredClone(
    hasResponse
      ? {
          ...raw,
          properties: { ...(isObject(raw.properties) ? raw.properties : {}), [RESPONSE_PROBE]: responseProbeRef(response) },
        }
      : raw,
  );
  refsToNames(input, names);

  let ts = await compile(input as never, "payload.schema.json", compileOptions(bundle, source));
  let responseType: string | null = null;
  if (hasResponse) ({ ts, responseType } = stripResponseProbe(ts, where));
  if (responseSchema === null && responseType !== null) {
    throw new Error(`${where}: the schema declares $defs.Response but the bundle carries no responseSchema`);
  }

  const declaredHere = new Set([...ts.matchAll(DECLARATION_RE)].map((m) => m[1]!));
  for (const name of names.values()) {
    if (declaredHere.has(name)) {
      throw new Error(`${where}: the definition ${name} has the same name as a type the payload schema declares`);
    }
  }

  const rootType = rootTypeName(ts, where, raw);
  const deprecation = bundle.deprecation;
  if (deprecation) {
    ts = markDeprecated(ts, rootType, deprecation);
    if (responseType && /^[A-Za-z0-9_$]+$/.test(responseType) && responseType !== rootType) {
      ts = markDeprecated(ts, responseType, deprecation);
    }
  }
  const definitions = await compileDefinitions(raw, names, where);
  if (definitions) {
    ts +=
      `\n/* ── Definitions the payload schema references, declared once ──────────── */\n\n` +
      `${definitions}\n`;
  }
  const tail = emitTail(ts, rootType, responseType, where, {
    typeUri: bundle.typeUri,
    extendsVersion: bundle.extends,
    deprecation,
    bundle,
    schemas: { request: raw, response: responseType ? responseSchema : null },
  });

  return {
    path: modulePath(bundle, source),
    source,
    text: ts + tail,
    slug: bundle.slug,
    version: bundle.version,
    typeUri: bundle.typeUri,
    extendsVersion: bundle.extends,
  };
}

/** The barrel's namespace for a module: `acl/grant` 0.2 → `AclGrant_v0_2` (as the package's root barrel). */
export function namespaceFor(slug: string, version: string): string {
  return (
    slug
      .split("/")
      .map((s) =>
        s
          .split("-")
          .map((p) => p.charAt(0).toUpperCase() + p.slice(1))
          .join(""),
      )
      .join("") + `_v${version.replace(/\./g, "_")}`
  );
}

/** Where the runtime is imported from. A subpath, so importing it never loads the prebuilt modules. */
export const RUNTIME_SPECIFIER = "@openvtc/trust-tasks/runtime";

/** `_extends.ts`: the locked versions' `extends` declarations, in the package's own `_extends.ts` shape. */
export function emitExtendsIndex(modules: readonly GeneratedModule[]): string {
  const entries = modules
    .filter((m) => m.extendsVersion)
    .map((m) => [m.typeUri, m.extendsVersion!] as const)
    .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));
  return [
    `/** Generated by ${GENERATOR} — DO NOT EDIT BY HAND. */`,
    "",
    "/**",
    " * Bare request Type URI → the version it declares it extends (front matter",
    " * `extends`, SPEC §5.2 \"Declared extension\"). Only versions that declare one",
    " * are listed. Each entry is a DIRECT declaration; the runtime's",
    " * `extendsChain` follows them transitively.",
    " */",
    "export const EXTENDS_BY_TYPE_URI: Readonly<Record<string, string>> = Object.freeze({",
    ...entries.map(([uri, v]) => `  ${JSON.stringify(uri)}: ${JSON.stringify(v)},`),
    "});",
    "",
  ].join("\n");
}

/** A source's namespace in the barrel: `acme` → `Acme`, `acme-corp` → `AcmeCorp`. */
export function sourceNamespace(source: string): string {
  return source
    .split("-")
    .map((p) => p.charAt(0).toUpperCase() + p.slice(1))
    .join("");
}

/** `export * as <Namespace> from "<path>"` for each module, refusing two that want one name. */
function namespaceLines(modules: readonly GeneratedModule[], relTo: (m: GeneratedModule) => string): string[] {
  const lines: string[] = [];
  const seen = new Map<string, string>();
  for (const m of modules) {
    const id = namespaceFor(m.slug, m.version);
    const rel = relTo(m);
    const clash = seen.get(id);
    if (clash) throw new Error(`two locked versions both want the export name ${id}: ${clash} and ${rel}`);
    seen.set(id, rel);
    lines.push(`export * as ${id} from ${JSON.stringify(rel)};`);
  }
  return lines;
}

/**
 * `<source>/index.ts`: the namespaces of one third-party source's modules,
 * which the top-level barrel re-exports as one namespace named for the source.
 */
export function emitSourceIndex(source: string, modules: readonly GeneratedModule[]): string {
  const prefix = `${source}/`;
  return [
    `/** Generated by ${GENERATOR} — DO NOT EDIT BY HAND. */`,
    "",
    `// Tasks locked from the third-party source "${source}" (SPEC §6.5).`,
    ...namespaceLines(
      modules.filter((m) => m.source === source),
      (m) => `./${m.path.slice(prefix.length).replace(/\.ts$/, ".js")}`,
    ),
    "",
  ].join("\n");
}

/**
 * `index.ts`: one namespace per official module, one per third-party source
 * (holding that source's modules), the `extends` table, and `declareExtends()`.
 */
export function emitIndex(modules: readonly GeneratedModule[]): string {
  const lines = [
    `/** Generated by ${GENERATOR} — DO NOT EDIT BY HAND. */`,
    "",
    `import { declareExtends as declareExtendsInRuntime } from ${JSON.stringify(RUNTIME_SPECIFIER)};`,
    `import { EXTENDS_BY_TYPE_URI } from "./_extends.js";`,
    "",
    ...namespaceLines(
      modules.filter((m) => m.source === null),
      (m) => `./${m.path.replace(/\.ts$/, ".js")}`,
    ),
  ];
  const sources = [...new Set(modules.map((m) => m.source).filter((s): s is string => s !== null))].sort();
  for (const source of sources) {
    lines.push(`export * as ${sourceNamespace(source)} from ${JSON.stringify(`./${source}/index.js`)};`);
  }
  lines.push(
    "",
    "export { EXTENDS_BY_TYPE_URI };",
    "",
    "/**",
    " * Register these versions' `extends` declarations with the runtime, so",
    " * `readForSpec` (and `extendsOf` / `extendsChain`) route a later MINOR to the",
    " * version this application implements (SPEC §5.2 \"Declared extension\").",
    " * Call once at start-up; calling it again is harmless.",
    " */",
    "export function declareExtends(): void {",
    "  declareExtendsInRuntime(EXTENDS_BY_TYPE_URI);",
    "}",
    "",
  );
  return lines.join("\n");
}
