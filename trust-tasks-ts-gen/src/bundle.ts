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
 * Only `properties` is reordered: it is all `propertyOrder` records, and it is
 * the only member order a generated type can observe. Every other object keeps
 * the bundle's sorted order. A pointer that names nothing, or an order that
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
  return { schema: roots.schema as JsonObject, responseSchema: roots.responseSchema as JsonObject | null };
}
