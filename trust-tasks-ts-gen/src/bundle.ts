/**
 * Registry bundles (`bundleVersion: 1`), as vendored beside a lockfile.
 *
 * A bundle is one specification version's self-contained payload schema plus
 * its policy, in RFC 8785 canonical form — so its object members are sorted.
 * `propertyOrder` records the *authored* order of every `properties` object,
 * which is the only member order a generated type can observe; it is restored
 * here before anything is generated. See docs/design/consumer-side-generation.md.
 */

import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";

import type { LockedTask } from "./lock.js";

/** The bundle shape this generator reads. */
export const BUNDLE_VERSION = 1;

export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };
export type JsonObject = { [key: string]: Json };

/** Per-variant boolean policy. */
export interface VariantFlags {
  readonly request: boolean;
  readonly response: boolean;
}

/** The members of a bundle this generator uses (others are carried, unread). */
export interface Bundle {
  readonly bundleVersion: number;
  readonly typeUri: string;
  readonly slug: string;
  readonly version: string;
  readonly status: string;
  readonly deprecation: string | null;
  readonly extends: string | null;
  readonly policy: {
    readonly bearer: boolean;
    readonly proofRequired: VariantFlags;
    readonly issuedAtRequired: VariantFlags;
    readonly recipientRequired: VariantFlags;
  };
  readonly propertyOrder: Readonly<Record<string, readonly string[]>>;
  /**
   * `/schema` and `/responseSchema` → the root `$defs` names in the order the
   * in-repo inliner leaves them. `{}` in a bundle written before it existed.
   */
  readonly defsOrder: Readonly<Record<string, readonly string[]>>;
  /**
   * Each `$defs` entry spliced in from a shared document → that document's
   * `$id` + `#/$defs/<name>`; the payload schema's own definitions have none.
   * `null` when the bundle predates the member — not `{}`, which says "no
   * definition here was shared": a reader of an older bundle cannot tell a
   * shared definition from the schema's own, and falls back accordingly.
   */
  readonly defOrigins: Readonly<Record<string, string>> | null;
  readonly schema: JsonObject;
  readonly responseSchema: JsonObject | null;
}

/** `sha256:<lowercase hex>` of `bytes` — the digest form a lockfile records. */
export function sha256Digest(bytes: Uint8Array): string {
  return `sha256:${createHash("sha256").update(bytes).digest("hex")}`;
}

/**
 * `<bundleDir>/<slug>/<M.N>.bundle.json`, or `<bundleDir>/@<source>/<slug>/<M.N>.bundle.json`
 * for a task from a third-party source (`@` cannot begin a slug, so the two never collide).
 */
export function bundlePath(bundleDir: string, task: LockedTask): string {
  const base = task.source === null ? bundleDir : path.join(bundleDir, `@${task.source}`);
  return path.join(base, ...task.slug.split("/"), `${task.version}.bundle.json`);
}

/** The official registry's authority. Only official tasks' Type URIs may sit under it. */
export const OFFICIAL_AUTHORITY = "https://trusttasks.org/";

const isObject = (v: unknown): v is JsonObject => typeof v === "object" && v !== null && !Array.isArray(v);

function flags(v: unknown, what: string): VariantFlags {
  if (!isObject(v) || typeof v.request !== "boolean" || typeof v.response !== "boolean") {
    throw new Error(`policy.${what} must be {request: boolean, response: boolean}`);
  }
  return { request: v.request, response: v.response };
}

/* ── Text a bundle contributes to generated code ─────────────────────────────
 *
 * A generated module is a pure function of the bundle's schema *data*: every
 * bundle string the generator writes into the module is either a JSON string
 * literal, or text checked here first. The Type URI is written into the banner
 * comment and the lifecycle note into `@deprecated` doc comments, so both are
 * held to their grammars, which cannot end a comment or start a line. The
 * schemas are compiled by json-schema-to-typescript, which reads two keywords
 * of its own (`tsType`, `tsEnumNames`) as TypeScript source; they are not JSON
 * Schema and no registry schema carries them, so a bundle that does is refused
 * — the generator adds `tsType` itself, after this check. A member named
 * `__proto__` is refused too: written into an object literal it sets the
 * literal's prototype instead of defining the member.
 */

const SCHEME_RE = /^[A-Za-z][A-Za-z0-9+.-]*$/;
const SLUG_SEGMENT = "[a-z][a-z0-9]*(?:-[a-z0-9]+)*";
const SLUG = `${SLUG_SEGMENT}(?:/${SLUG_SEGMENT})*`;
const VERSION = "(?:0|[1-9][0-9]*)\\.(?:0|[1-9][0-9]*)";
// RFC 3986 unreserved, sub-delims except `*`, and `:`, `@`, `/`, `%`.
const AUTHORITY_CHARS = /^[A-Za-z0-9\-._~!$&'()+,;=:@/%]+$/;
const PATH_FORM_TAIL = new RegExp(`^/spec/${SLUG}/${VERSION}$`);
const URN_FORM_TAIL = new RegExp(`^:spec:${SLUG_SEGMENT}(?::${SLUG_SEGMENT})*:${VERSION}$`);
const LIFECYCLE_NOTE = new RegExp(`^(?:retired|retired; use ${SLUG}/${VERSION}|superseded by ${SLUG}/${VERSION})$`);

/** Generator-specific keywords json-schema-to-typescript emits as code. */
const GENERATOR_KEYWORDS = ["tsType", "tsEnumNames"] as const;
/** Keywords whose value maps names to subschemas. */
const SCHEMA_MAP_KEYWORDS = new Set(["properties", "patternProperties", "$defs", "definitions", "dependentSchemas"]);
/** Keywords whose value is a subschema, or an array of them. */
const SCHEMA_KEYWORDS = new Set([
  "items",
  "prefixItems",
  "additionalItems",
  "additionalProperties",
  "unevaluatedItems",
  "unevaluatedProperties",
  "contains",
  "propertyNames",
  "not",
  "if",
  "then",
  "else",
  "allOf",
  "anyOf",
  "oneOf",
]);

/**
 * Throws unless `typeUri` is a bare SPEC §6.1 Type URI — path form
 * `<prefix>/spec/<slug>/<M.N>` or URN form `urn:<nid>:<prefix>:spec:<seg>:…:<M.N>`
 * — whose prefix is made of URI characters other than `*`.
 */
export function checkBundleTypeUri(typeUri: string): void {
  const fail = (why: string): never => {
    throw new Error(`bundle member typeUri ${JSON.stringify(typeUri)} is not a Type URI: ${why}`);
  };
  const colon = typeUri.indexOf(":");
  const scheme = colon < 0 ? "" : typeUri.slice(0, colon);
  if (!SCHEME_RE.test(scheme) || scheme.toLowerCase() === "http") fail("it needs a scheme other than http");
  const urn = scheme.toLowerCase() === "urn";
  const at = typeUri.indexOf(urn ? ":spec:" : "/spec/");
  if (at < 0) fail(urn ? "a urn Type URI is urn:<nid>:<prefix>:spec:<slug>:<M.N>" : "it is not <prefix>/spec/<slug>/<M.N>");
  const prefix = typeUri.slice(0, at);
  if (!AUTHORITY_CHARS.test(prefix)) fail("its prefix holds a character outside the URI character set");
  if (!(urn ? URN_FORM_TAIL : PATH_FORM_TAIL).test(typeUri.slice(at))) fail("its slug or MAJOR.MINOR version is malformed");
}

/** Throws unless `note` is one of the lifecycle notes the registry writes. */
export function checkLifecycleNote(note: string): void {
  if (!LIFECYCLE_NOTE.test(note)) {
    throw new Error(
      `bundle member deprecation ${JSON.stringify(note)} is not a lifecycle note ` +
        "(`retired`, `retired; use <slug>/<M.N>` or `superseded by <slug>/<M.N>`)",
    );
  }
}

const escapePointerToken = (t: string): string => t.replace(/~/g, "~0").replace(/\//g, "~1");

/** Throws if any object under `node` has a member named `__proto__`. */
function refuseProtoKeys(node: unknown, pointer: string): void {
  if (Array.isArray(node)) {
    node.forEach((n, i) => refuseProtoKeys(n, `${pointer}/${i}`));
  } else if (isObject(node)) {
    for (const key of Object.keys(node)) {
      if (key === "__proto__") throw new Error(`bundle ${pointer || "/"} has a member named __proto__`);
      refuseProtoKeys(node[key], `${pointer}/${escapePointerToken(key)}`);
    }
  }
}

/** Throws if a schema object under `schema` carries a generator-specific keyword. */
function refuseGeneratorKeywords(schema: unknown, pointer: string): void {
  if (Array.isArray(schema)) {
    schema.forEach((s, i) => refuseGeneratorKeywords(s, `${pointer}/${i}`));
    return;
  }
  if (!isObject(schema)) return;
  for (const keyword of GENERATOR_KEYWORDS) {
    if (Object.prototype.hasOwnProperty.call(schema, keyword)) {
      throw new Error(`bundle ${pointer}: \`${keyword}\` is a json-schema-to-typescript keyword, not JSON Schema; a bundle must not carry it`);
    }
  }
  for (const [keyword, value] of Object.entries(schema)) {
    const at = `${pointer}/${escapePointerToken(keyword)}`;
    if (SCHEMA_MAP_KEYWORDS.has(keyword) && isObject(value)) {
      for (const [name, sub] of Object.entries(value)) refuseGeneratorKeywords(sub, `${at}/${escapePointerToken(name)}`);
    } else if (SCHEMA_KEYWORDS.has(keyword)) {
      refuseGeneratorKeywords(value, at);
    }
  }
}

/**
 * Throws unless every part of `bundle` the generator writes into a module as
 * text — the Type URI, the lifecycle note, the schemas' member names and
 * keywords — is held to the grammar that keeps it data. {@link parseBundle}
 * runs it; `generateModule` runs it again for a bundle built by hand.
 */
export function checkGeneratedText(bundle: Pick<Bundle, "typeUri" | "deprecation"> & { schema: unknown; responseSchema: unknown }): void {
  checkBundleTypeUri(bundle.typeUri);
  if (bundle.deprecation !== null) checkLifecycleNote(bundle.deprecation);
  refuseProtoKeys(bundle.schema, "/schema");
  refuseProtoKeys(bundle.responseSchema, "/responseSchema");
  refuseGeneratorKeywords(bundle.schema, "/schema");
  refuseGeneratorKeywords(bundle.responseSchema, "/responseSchema");
}

/** Check a parsed bundle's shape, returning it typed. */
export function parseBundle(value: unknown): Bundle {
  if (!isObject(value)) throw new Error("a bundle must be a JSON object");
  if (value.bundleVersion !== BUNDLE_VERSION) {
    throw new Error(
      `unknown bundleVersion ${JSON.stringify(value.bundleVersion)} (this generator reads ${BUNDLE_VERSION}) — upgrade @openvtc/trust-tasks-gen`,
    );
  }
  for (const key of ["typeUri", "slug", "version", "status"] as const) {
    if (typeof value[key] !== "string") throw new Error(`bundle member ${key} must be a string`);
  }
  for (const key of ["deprecation", "extends"] as const) {
    if (value[key] !== null && typeof value[key] !== "string") throw new Error(`bundle member ${key} must be a string or null`);
  }
  checkGeneratedText({
    typeUri: value.typeUri as string,
    deprecation: value.deprecation as string | null,
    schema: value.schema,
    responseSchema: value.responseSchema,
  });
  const policy = value.policy;
  if (!isObject(policy) || typeof policy.bearer !== "boolean") throw new Error("bundle member policy is malformed");
  if (!isObject(value.schema)) throw new Error("bundle member schema must be an object");
  if (value.responseSchema !== null && !isObject(value.responseSchema)) {
    throw new Error("bundle member responseSchema must be an object or null");
  }
  const order = value.propertyOrder ?? {};
  if (!isObject(order) || !Object.values(order).every((v) => Array.isArray(v) && v.every((n) => typeof n === "string"))) {
    throw new Error("bundle member propertyOrder must map JSON Pointers to arrays of member names");
  }
  const defsOrder = value.defsOrder ?? {};
  if (
    !isObject(defsOrder) ||
    !Object.entries(defsOrder).every(
      ([k, v]) => (k === "/schema" || k === "/responseSchema") && Array.isArray(v) && v.every((n) => typeof n === "string"),
    )
  ) {
    throw new Error("bundle member defsOrder must map /schema and /responseSchema to arrays of $defs names");
  }
  const defOrigins = value.defOrigins ?? null;
  if (defOrigins !== null && (!isObject(defOrigins) || !Object.values(defOrigins).every((o) => typeof o === "string" && /#\/\$defs\/[^/]+$/.test(o)))) {
    throw new Error("bundle member defOrigins must map $defs names to <document>#/$defs/<name>");
  }
  if (defOrigins !== null) {
    const defs = isObject(value.schema.$defs) ? value.schema.$defs : {};
    for (const name of Object.keys(defOrigins)) {
      if (!Object.prototype.hasOwnProperty.call(defs, name)) throw new Error(`defOrigins names $defs/${name}, which the schema does not define`);
    }
    if (Object.prototype.hasOwnProperty.call(defOrigins, "Response")) {
      throw new Error("defOrigins gives $defs/Response an origin, but the response is always the payload schema's own");
    }
  }
  return {
    bundleVersion: BUNDLE_VERSION,
    typeUri: value.typeUri as string,
    slug: value.slug as string,
    version: value.version as string,
    status: value.status as string,
    deprecation: value.deprecation as string | null,
    extends: value.extends as string | null,
    policy: {
      bearer: policy.bearer,
      proofRequired: flags(policy.proofRequired, "proofRequired"),
      issuedAtRequired: flags(policy.issuedAtRequired, "issuedAtRequired"),
      recipientRequired: flags(policy.recipientRequired, "recipientRequired"),
    },
    propertyOrder: order as Record<string, string[]>,
    defsOrder: defsOrder as Record<string, string[]>,
    defOrigins: defOrigins as Record<string, string> | null,
    schema: value.schema,
    responseSchema: value.responseSchema as JsonObject | null,
  };
}

/**
 * Read `task`'s vendored bundle, refusing it unless its bytes hash to the
 * locked digest and it names the locked slug, version and Type URI.
 */
export function readLockedBundle(bundleDir: string, task: LockedTask): Bundle {
  const file = bundlePath(bundleDir, task);
  let bytes: Buffer;
  try {
    bytes = readFileSync(file);
  } catch (err) {
    throw new Error(
      `read the vendored bundle for ${task.slug}/${task.version} at ${file} — run \`trust-tasks check\` (${(err as Error).message})`,
    );
  }
  const actual = sha256Digest(bytes);
  if (actual !== task.digest) {
    throw new Error(
      `vendored bundle does not match trust-tasks.lock — run \`trust-tasks check\` ` +
        `(${file}: the lock pins ${task.digest}, the file is ${actual})`,
    );
  }
  let bundle: Bundle;
  try {
    bundle = parseBundle(JSON.parse(bytes.toString("utf8")));
  } catch (err) {
    throw new Error(`${file}: ${(err as Error).message}`);
  }
  // Identity is the bundle's Type URI. An official task's is under the
  // registry's authority; a sourced task's (SPEC §6.5) never is — it may be any
  // SPEC §6.1 form (https or DID URL `<prefix>/spec/<slug>/<M.N>`, or a URN).
  if (task.source === null && !bundle.typeUri.startsWith(OFFICIAL_AUTHORITY)) {
    throw new Error(
      `${file}: ${bundle.typeUri} is not under ${OFFICIAL_AUTHORITY}, but the lock names it as an official task (no \`source\`)`,
    );
  }
  if (task.source !== null && bundle.typeUri.startsWith(OFFICIAL_AUTHORITY)) {
    throw new Error(
      `${file}: ${bundle.typeUri} is under ${OFFICIAL_AUTHORITY}, but the lock names it as from source "${task.source}" — ` +
        `an official specification is locked without a source`,
    );
  }
  if (bundle.slug !== task.slug || bundle.version !== task.version || bundle.typeUri !== task.typeUri) {
    throw new Error(
      `${file} is the bundle of ${bundle.typeUri} (${bundle.slug}/${bundle.version}), but trust-tasks.lock pins it as ` +
        `${task.typeUri} (${task.slug}/${task.version})`,
    );
  }
  return bundle;
}

/** Decode one JSON Pointer reference token. */
const unescapeToken = (t: string): string => t.replace(/~1/g, "/").replace(/~0/g, "~");

/**
 * The bundle's two schemas with every `properties` object put back in its
 * authored member order, as deep copies.
 *
 * `properties` is reordered from `propertyOrder`, the only member order a
 * generated type can observe, and each root `$defs` from `defsOrder` (absent
 * from an older bundle), the order the in-repo generator's schemas carry them
 * in. Every other object keeps the bundle's sorted order. A pointer that names nothing, or an order that
 * does not list exactly the members present, is refused — it would mean the
 * bundle disagrees with itself.
 */
export function withAuthoredOrder(bundle: Bundle): { schema: JsonObject; responseSchema: JsonObject | null } {
  const roots: JsonObject = {
    schema: structuredClone(bundle.schema),
    responseSchema: bundle.responseSchema === null ? null : structuredClone(bundle.responseSchema),
  };
  for (const pointer of Object.keys(bundle.propertyOrder).sort()) {
    const names = bundle.propertyOrder[pointer]!;
    if (!pointer.startsWith("/")) throw new Error(`propertyOrder: ${JSON.stringify(pointer)} is not a JSON Pointer`);
    let node: Json = roots;
    for (const token of pointer.slice(1).split("/").map(unescapeToken)) {
      if (Array.isArray(node) && /^(0|[1-9][0-9]*)$/.test(token)) node = node[Number(token)] ?? null;
      else if (isObject(node) && Object.prototype.hasOwnProperty.call(node, token)) node = node[token]!;
      else throw new Error(`propertyOrder: ${pointer} names nothing in the bundle`);
    }
    if (!isObject(node) || !isObject(node.properties)) {
      throw new Error(`propertyOrder: ${pointer} is not a schema object with \`properties\``);
    }
    const props: JsonObject = node.properties;
    const present = Object.keys(props);
    if (present.length !== names.length || !names.every((n) => Object.prototype.hasOwnProperty.call(props, n))) {
      throw new Error(`propertyOrder: ${pointer} lists [${names.join(", ")}], but its properties are [${present.join(", ")}]`);
    }
    const ordered: JsonObject = {};
    for (const name of names) ordered[name] = props[name]!;
    node.properties = ordered;
  }
  for (const [pointer, names] of Object.entries(bundle.defsOrder)) {
    const root = roots[pointer.slice(1)];
    if (!isObject(root)) {
      if (names.length === 0) continue;
      throw new Error(`defsOrder: ${pointer} names $defs, but the bundle has no ${pointer.slice(1)}`);
    }
    const defs = isObject(root.$defs) ? root.$defs : {};
    const present = Object.keys(defs);
    if (present.length !== names.length || !names.every((n) => Object.prototype.hasOwnProperty.call(defs, n))) {
      throw new Error(`defsOrder: ${pointer} lists [${names.join(", ")}], but its $defs are [${present.join(", ")}]`);
    }
    if (names.length === 0) continue;
    const ordered: JsonObject = {};
    for (const name of names) ordered[name] = defs[name]!;
    root.$defs = ordered;
  }
  return { schema: roots.schema as JsonObject, responseSchema: roots.responseSchema as JsonObject | null };
}
