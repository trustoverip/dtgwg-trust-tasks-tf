/**
 * Forward-MINOR acceptance along a declared `extends` chain (SPEC §5.2
 * "Declared extension").
 *
 * A specification version may declare `extends: "<M.K>"`: it keeps everything
 * version `M.K` defines and only adds. The registry build checks that promise
 * against the two payload schemas. SPEC: a consumer implementing a version
 * SHOULD accept a document of any later version whose `extends` chain reaches
 * it, by ignoring the payload members it does not recognise; without a
 * declaration it SHOULD NOT.
 *
 * The point of carrying the chain in the library is that an application which
 * implements only `x/0.2` accepts an `x/0.3` document after a library upgrade,
 * with no code change:
 *
 * ```ts
 * const read = readForSpec<XGet_v0_2.Payload>(JSON.parse(body), XGet_v0_2.SPEC);
 * if (read.kind === "refused") return emit(rejectWith(parsed, newId(), toErrorPayload(read.reason)));
 * const outcome = await consumeInbound({ ...opts, spec: XGet_v0_2.SPEC, doc: read.doc });
 * ```
 *
 * `read.doc` keeps the received `type`, so {@link respondWith} answers with
 * `<received type>#response` — valid for the received version, because a
 * response shaped for the version it extends is (the extension is additive).
 *
 * Hand-written. The index it walks, `EXTENDS_BY_TYPE_URI`, is generated.
 */

import { EXTENDS_BY_TYPE_URI } from "../_extends.js";
import { canonicalJson } from "./canonical.js";
import type { RejectReason, SpecPolicy, TrustTaskDocument } from "./document.js";
import {
  formatTypeUri,
  sameSpecification,
  tryParseTypeUri,
  typeUriWithVersion,
  type TypeUri,
} from "./type-uri.js";

const VERSION_RE = /^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/;

/**
 * Where to look up the version a Type URI declares it extends. Either a map
 * from bare request Type URI to version (the shape of the generated
 * `EXTENDS_BY_TYPE_URI`), or a function. Defaults to the generated index plus
 * the tables an application registered with {@link declareExtends}; pass
 * your own to route on knowledge this library release does not carry, or in a
 * test.
 */
export type ExtendsLookup =
  | Readonly<Record<string, string>>
  | ((bareTypeUri: string) => string | null | undefined);

/** `M.N` of a parsed Type URI. */
const versionOf = (u: TypeUri): string => `${u.major}.${u.minor}`;

/**
 * The key an {@link ExtendsLookup} is consulted with: the bare (fragment-free)
 * request Type URI of `u` at `version`, under `u`'s own authority and form.
 * Building it from `u` rather than from the slug is what keeps a chain inside
 * one authority: a third-party `x/0.3` is never looked up as the registry's
 * `x/0.3`.
 */
function bareKey(u: TypeUri): string {
  return formatTypeUri({ ...u, variant: null });
}

/**
 * Tables an application registered with {@link declareExtends}, in the order
 * they were declared.
 */
const APPLICATION_EXTENDS: Readonly<Record<string, string>>[] = [];

/**
 * Teach the default lookup — {@link extendsOf}, {@link extendsChain} and
 * {@link readForSpec} when no `index` is passed — the `extends` declarations
 * of specification versions this package's generated table does not carry:
 * those an application generated itself from registry bundles with
 * `trust-tasks-gen-ts` (`@openvtc/trust-tasks-gen`).
 *
 * Each key is a bare request Type URI and each value the `"MAJOR.MINOR"` it
 * extends — the shape of the generated `EXTENDS_BY_TYPE_URI`. The `index.ts`
 * the generator writes exports a `declareExtends()` that calls this with its
 * table, so an application calls that once at start-up.
 *
 * Process-wide and additive. Declaring the same table twice is harmless, and
 * where this package's generated table already answers for a Type URI that
 * answer stands. A declaration is still only followed when it names a strictly
 * earlier MINOR of the same MAJOR (SPEC §5.2).
 */
export function declareExtends(declarations: Readonly<Record<string, string>>): void {
  if (!APPLICATION_EXTENDS.includes(declarations)) APPLICATION_EXTENDS.push(declarations);
}

const hasOwn = (o: object, k: string): boolean => Object.prototype.hasOwnProperty.call(o, k);

/** The default {@link ExtendsLookup}: the generated table, then {@link declareExtends}'s. */
function defaultLookup(bare: string): string | undefined {
  if (hasOwn(EXTENDS_BY_TYPE_URI, bare)) return EXTENDS_BY_TYPE_URI[bare];
  for (const table of APPLICATION_EXTENDS) if (hasOwn(table, bare)) return table[bare];
  return undefined;
}

function lookup(index: ExtendsLookup, bare: string): string | null {
  const v = typeof index === "function" ? index(bare) : Object.prototype.hasOwnProperty.call(index, bare) ? index[bare] : undefined;
  return typeof v === "string" && VERSION_RE.test(v) ? v : null;
}

/**
 * The version `typeUri` DIRECTLY declares it extends, or `null`. Any
 * `#request` / `#response` fragment is ignored — the declaration is the
 * specification's, covering both variants.
 */
export function extendsOf(typeUri: string, index: ExtendsLookup = defaultLookup): string | null {
  const p = tryParseTypeUri(typeUri);
  if (p === null) return null;
  return lookup(index, bareKey(p));
}

/**
 * Every version `typeUri` extends, transitively, nearest first: for 0.3
 * extends 0.2 extends 0.1, `["0.2", "0.1"]`.
 *
 * `typeUri` may be in any SPEC §6.1 form. Each link is looked up under the
 * same authority, form and slug as `typeUri` — only the version changes — so a
 * chain never crosses authorities: a third-party specification's chain is read
 * from its own declarations, never from a registry specification that happens
 * to share its slug.
 *
 * Defensive against a malformed index: the walk stops at a declaration that is
 * not strictly earlier within the same MAJOR (the build refuses those), and at
 * a cycle.
 */
export function extendsChain(typeUri: string, index: ExtendsLookup = defaultLookup): string[] {
  const start = tryParseTypeUri(typeUri);
  if (start === null) return [];
  const chain: string[] = [];
  let current = start;
  for (;;) {
    const next = lookup(index, bareKey(current));
    if (next === null) break;
    const m = VERSION_RE.exec(next)!;
    const major = Number(m[1]);
    const minor = Number(m[2]);
    if (major !== current.major || minor >= current.minor) break;
    chain.push(next);
    current = typeUriWithVersion(current, major, minor);
  }
  return chain;
}

/**
 * The outcome of {@link readForSpec}.
 *
 * - `accepted` — `doc` is the document to pass to `consumeInbound` and hand to
 *   the handler. When `extended` is `false` it IS `received`, untouched.
 *   When `true` it is a copy whose payload has been pruned to the members the
 *   implemented version's schema knows; its `type` is still the received one,
 *   so a response built with `respondWith` is typed `<received type>#response`.
 * - `refused` — route `reason` per §8.1 (`reject` / `rejectWith`).
 */
export type ReadForSpecResult<P> =
  | {
      kind: "accepted";
      doc: TrustTaskDocument<P>;
      /** The document exactly as received. Proofs are verified over this. */
      received: TrustTaskDocument<unknown>;
      /** Whether the received version is a later one reached along `extends`. */
      extended: boolean;
    }
  | { kind: "refused"; reason: RejectReason };

/** The pruned copy → the document it was read from. Written only by {@link readForSpec}. */
const RECEIVED = new WeakMap<object, { received: TrustTaskDocument<unknown>; readFor: string }>();

/**
 * The document as received, when `doc` is a re-reading of it produced by
 * {@link readForSpec} — otherwise `undefined`.
 *
 * `consumeInbound` uses this to verify a proof (§7.2 item 7) and to digest the
 * document for the item-11 record over what was actually received, never over
 * the pruned copy: a proof covers the members a pruned copy drops, and two
 * documents differing only in an extension member are different documents.
 * A binding-specific pipeline that verifies proofs itself should do the same.
 *
 * Keyed by object identity. A copy of `doc` made by the caller (a spread, a
 * `structuredClone`) is no longer recognised, and its proof is then checked
 * over the pruned payload and fails — closed, not open.
 */
export function receivedDocumentOf<P>(doc: TrustTaskDocument<P>): TrustTaskDocument<unknown> | undefined {
  return typeof doc === "object" && doc !== null ? RECEIVED.get(doc)?.received : undefined;
}

/**
 * The `spec.typeUri` a {@link readForSpec} re-reading was made for, or
 * `undefined` when `doc` is not one. `consumeInbound` skips its own §7.2 item 3
 * check only when this is the spec it is consuming under.
 */
export function readingSpecOf<P>(doc: TrustTaskDocument<P>): string | undefined {
  return typeof doc === "object" && doc !== null ? RECEIVED.get(doc)?.readFor : undefined;
}

/**
 * Re-read a received document for the specification version the application
 * implements (`spec`, a generated `SPEC` or `RESPONSE_SPEC`).
 *
 * - Same Type URI as `spec.typeUri` — accepted unchanged.
 * - Same specification (authority, form and slug) and variant, later version
 *   whose `extends` chain reaches `spec`'s version — accepted, payload pruned
 *   against `spec.payloadSchema` with {@link prunePayload}, `type` kept as
 *   received.
 * - Same specification and variant, any other version — refused
 *   `unsupportedVersion`. (Without a declaration a later MINOR SHOULD NOT be
 *   accepted: a draft may break in one.)
 * - A different specification or variant — refused `unsupportedType`. The
 *   same slug under another authority is another specification (SPEC §6.1),
 *   so a third-party `x/0.3` never routes to a registry `x/0.1`, or back.
 * - Not a Trust Task document at all — refused `malformedRequest`.
 * - A later version whose payload cannot be pruned within the step limit
 *   {@link prunePayload} describes — refused `malformedRequest`, with a message
 *   saying so. This runs before any proof is checked, so the work it does on
 *   an unauthenticated document is bounded by the payload's size.
 *
 * Proofs are not this function's business, and it neither reads nor alters
 * `proof`: it does not mutate `received`, and it records the pairing so that
 * `consumeInbound` verifies over `received` (see {@link receivedDocumentOf}).
 */
export function readForSpec<P>(
  received: unknown,
  spec: SpecPolicy,
  options: { index?: ExtendsLookup } = {},
): ReadForSpecResult<P> {
  const index = options.index ?? defaultLookup;
  if (!isPlainObject(received) || typeof received.type !== "string" || !("payload" in received)) {
    return refused("malformedRequest", "not a Trust Task document (SPEC §4.2)");
  }
  const doc = received as unknown as TrustTaskDocument<unknown>;
  if (doc.type === spec.typeUri) {
    return { kind: "accepted", doc: doc as TrustTaskDocument<P>, received: doc, extended: false };
  }

  const got = tryParseTypeUri(doc.type);
  const want = tryParseTypeUri(spec.typeUri);
  if (got === null || want === null || !sameSpecification(got, want) || got.variant !== want.variant) {
    return refused("unsupportedType", `unsupported Trust Task type ${JSON.stringify(doc.type)}`);
  }
  const gotVersion = versionOf(got);
  const wantVersion = versionOf(want);
  if (gotVersion === wantVersion || !extendsChain(doc.type, index).includes(wantVersion)) {
    return refused(
      "unsupportedVersion",
      `version ${gotVersion} of ${got.slug} is not supported; this consumer implements ${wantVersion}`,
    );
  }

  let payload: unknown = doc.payload;
  if (spec.payloadSchema !== undefined) {
    const r = pruneWithinLimit(doc.payload, spec.payloadSchema);
    if (!r.ok) {
      return refused(
        "malformedRequest",
        `the ${gotVersion} payload is too complex to re-read as version ${wantVersion} of ${got.slug}: ` +
          `pruning it against that version's schema takes more than ${r.limit} steps`,
      );
    }
    payload = r.value;
  }
  const pruned: TrustTaskDocument<P> = { ...doc, payload: payload as P };
  RECEIVED.set(pruned, { received: doc, readFor: spec.typeUri });
  return { kind: "accepted", doc: pruned, received: doc, extended: true };
}

/**
 * Whether a document whose `type` is `typeUri` may be consumed under `spec`
 * (SPEC §7.2 item 3): `null` when it may, otherwise the reason to refuse it.
 *
 * It may when `typeUri` is `spec.typeUri` — `#request` and no fragment are one
 * variant (§4.4.1) — or a later version of the same specification and variant
 * whose `extends` chain reaches `spec`'s version (§5.2). Otherwise
 * `unsupportedVersion` for the same specification and variant, and
 * `unsupportedType` for anything else: another slug, the same slug under
 * another authority or form (§6.1), another variant, or a `type` that is not a
 * Type URI. `consumeInbound` runs this before any other check.
 */
export function typeRefusalFor(
  typeUri: unknown,
  spec: SpecPolicy,
  options: { index?: ExtendsLookup } = {},
): RejectReason | null {
  if (typeof typeUri === "string" && typeUri === spec.typeUri) return null;
  const shown = JSON.stringify(typeof typeUri === "string" ? typeUri : String(typeUri));
  const got = typeof typeUri === "string" ? tryParseTypeUri(typeUri) : null;
  const want = tryParseTypeUri(spec.typeUri);
  if (
    got === null ||
    want === null ||
    !sameSpecification(got, want) ||
    (got.variant ?? "request") !== (want.variant ?? "request")
  ) {
    return {
      code: "unsupportedType",
      message: `unsupported Trust Task type ${shown}; this consumer implements ${spec.typeUri}`,
      retryable: false,
    };
  }
  const gotVersion = versionOf(got);
  const wantVersion = versionOf(want);
  if (gotVersion === wantVersion) return null;
  if (extendsChain(typeUri as string, options.index ?? defaultLookup).includes(wantVersion)) return null;
  return {
    code: "unsupportedVersion",
    message: `version ${gotVersion} of ${got.slug} is not supported; this consumer implements ${wantVersion}`,
    retryable: false,
  };
}

function refused(code: RejectReason["code"], message: string): ReadForSpecResult<never> {
  return { kind: "refused", reason: { code, message, retryable: false } };
}

// ── Pruning ────────────────────────────────────────────────────────────────

type Obj = Record<string, unknown>;

const MAX_DEPTH = 64;

/**
 * The work one prune may do, in schema steps (a value visited against a
 * subschema, or a branch test): a fixed allowance plus a share for every
 * value in the payload. Ordinary payloads use a small fraction of it. A schema
 * whose `$ref`s, `allOf`s or `oneOf`/`anyOf` branches revisit the same values
 * many times over could otherwise make the walk grow exponentially with the
 * payload's nesting depth, and {@link readForSpec} runs before any proof is
 * checked.
 */
const PRUNE_BASE_STEPS = 100_000;
/** See {@link PRUNE_BASE_STEPS}. */
const PRUNE_STEPS_PER_VALUE = 64;

class PruneLimitReached extends Error {}

/** State shared by one prune: the schema root, the remaining budget and the memo tables. */
interface PruneContext {
  readonly root: Obj;
  steps: number;
  /** value → `oneOf`/`anyOf` list → the branch chosen for it. */
  readonly choices: WeakMap<object, Map<unknown[], Obj | null>>;
  /** value → schema → {@link conforms}'s answer. */
  readonly conformed: WeakMap<object, Map<object, boolean>>;
  readonly patterns: Map<string, RegExp | null>;
}

function spend(ctx: PruneContext): void {
  if (--ctx.steps < 0) throw new PruneLimitReached();
}

/** The number of JSON values in `v` (itself, members and elements, recursively). */
function countValues(v: unknown): number {
  let n = 0;
  const stack: unknown[] = [v];
  while (stack.length > 0) {
    const x = stack.pop();
    n++;
    if (Array.isArray(x)) for (const e of x) stack.push(e);
    else if (isPlainObject(x)) for (const k in x) if (Object.prototype.hasOwnProperty.call(x, k)) stack.push(x[k]);
  }
  return n;
}

/** The step budget for pruning `payload`. */
function pruneStepLimit(payload: unknown): number {
  return PRUNE_BASE_STEPS + PRUNE_STEPS_PER_VALUE * countValues(payload);
}

function isPlainObject(v: unknown): v is Obj {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}

/**
 * Drop from `payload` the members `schema` does not know, recursively, and
 * return the result. Never mutates `payload`; returns a copy where anything
 * was walked.
 *
 * `schema` is a generated `PAYLOAD_SCHEMA` / `RESPONSE_PAYLOAD_SCHEMA`: JSON
 * Schema 2020-12, self-contained, internal `#/…` refs only.
 *
 * What it does:
 * - An object member is dropped where a schema applying to that object says
 *   `additionalProperties: false` and neither its `properties` nor its
 *   `patternProperties` name the member. `unevaluatedProperties: false` drops
 *   a member no applying schema (including a chosen `oneOf`/`anyOf` branch)
 *   names.
 * - Kept members are pruned against their `properties` / `patternProperties` /
 *   `additionalProperties` subschema; array elements against `prefixItems` /
 *   `items`.
 * - `$ref` (internal JSON-pointer refs) and `allOf` apply conjunctively.
 * - `oneOf` / `anyOf`: each branch is tried — prune against it, then check the
 *   result against a conservative structural matcher (`type`, `const`, `enum`,
 *   `required`, closed objects, recursively). If exactly one branch matches, or
 *   every match prunes to the same value, the value is pruned against it.
 *   Otherwise it is LEFT UNTOUCHED: guessing a branch could drop members the
 *   actual branch knows. A value of a branch the implemented version does not
 *   have (an appended `oneOf` branch) therefore survives intact and fails
 *   item-2 validation, which is the right answer. The branch chosen for a
 *   value, and the matcher's answers, are remembered for the rest of the walk,
 *   so nested `oneOf`s are not re-decided once per enclosing branch.
 *
 * The walk takes at most 100,000 steps plus 64 per value in the payload (a
 * step is a value visited against a subschema, or a branch test). A payload
 * and schema that need more are returned untouched, as an undecidable branch
 * is; {@link readForSpec} refuses such a document instead.
 *
 * Not followed: `if`/`then`/`else`, `dependentSchemas`, `not`, `$anchor` /
 * `$dynamicRef` refs, and refs outside the document — a value under one of
 * these is left as it is.
 */
export function prunePayload(payload: unknown, schema: unknown): unknown {
  const r = pruneWithinLimit(payload, schema);
  return r.ok ? r.value : payload;
}

/** {@link prunePayload}, reporting when the step limit was reached. */
function pruneWithinLimit(
  payload: unknown,
  schema: unknown,
): { ok: true; value: unknown } | { ok: false; limit: number } {
  if (!isPlainObject(schema)) return { ok: true, value: payload };
  const limit = pruneStepLimit(payload);
  const ctx: PruneContext = {
    root: schema,
    steps: limit,
    choices: new WeakMap(),
    conformed: new WeakMap(),
    patterns: new Map(),
  };
  try {
    return { ok: true, value: pruneAt(payload, schema, ctx, 0) };
  } catch (e) {
    if (e instanceof PruneLimitReached) return { ok: false, limit };
    throw e;
  }
}

function resolveRef(ref: string, root: Obj): unknown {
  if (ref === "#") return root;
  if (!ref.startsWith("#/")) return undefined;
  let node: unknown = root;
  for (const raw of ref.slice(2).split("/")) {
    const key = decodeURIComponent(raw).replace(/~1/g, "/").replace(/~0/g, "~");
    if (!isPlainObject(node) && !Array.isArray(node)) return undefined;
    node = (node as Obj)[key];
    if (node === undefined) return undefined;
  }
  return node;
}

/** `schema` plus everything that applies alongside it: its `$ref` target and `allOf` members, recursively. */
function applicable(schema: unknown, root: Obj, depth: number, out: Obj[] = []): Obj[] {
  if (!isPlainObject(schema) || depth > MAX_DEPTH || out.includes(schema)) return out;
  out.push(schema);
  if (typeof schema.$ref === "string") applicable(resolveRef(schema.$ref, root), root, depth + 1, out);
  if (Array.isArray(schema.allOf)) for (const s of schema.allOf) applicable(s, root, depth + 1, out);
  return out;
}

function matchesPattern(key: string, patterns: unknown, ctx: PruneContext): unknown {
  if (!isPlainObject(patterns)) return undefined;
  for (const [pat, sub] of Object.entries(patterns)) {
    let re = ctx.patterns.get(pat);
    if (re === undefined) {
      try {
        re = new RegExp(pat, "u");
      } catch {
        re = null; // an unusable pattern names nothing
      }
      ctx.patterns.set(pat, re);
    }
    if (re !== null && re.test(key)) return sub;
  }
  return undefined;
}

function ownProps(part: Obj): Obj {
  return isPlainObject(part.properties) ? part.properties : {};
}

function pruneAt(value: unknown, schema: Obj, ctx: PruneContext, depth: number): unknown {
  if (depth > MAX_DEPTH) return value;
  spend(ctx);
  const root = ctx.root;
  const parts = applicable(schema, root, depth);

  // Branch choices are made on the value as it stands, before this level
  // prunes anything, so a closed parent cannot hide a discriminator.
  const branches: Obj[] = [];
  let branchesResolved = true;
  for (const part of parts) {
    for (const key of ["oneOf", "anyOf"] as const) {
      const list = part[key];
      if (!Array.isArray(list)) continue;
      const chosen = chooseBranch(value, list, ctx, depth);
      if (chosen === null) branchesResolved = false;
      else branches.push(chosen);
    }
  }

  let out: unknown = value;
  if (isPlainObject(value)) {
    const obj: Obj = { ...value };
    for (const part of parts) {
      if (part.additionalProperties !== false) continue;
      const props = ownProps(part);
      for (const k of Object.keys(obj)) {
        if (!Object.prototype.hasOwnProperty.call(props, k) && matchesPattern(k, part.patternProperties, ctx) === undefined) {
          delete obj[k];
        }
      }
    }
    if (branchesResolved && parts.some((p) => p.unevaluatedProperties === false)) {
      const evaluated = [...parts, ...branches.flatMap((b) => applicable(b, root, depth + 1))];
      for (const k of Object.keys(obj)) {
        const known = evaluated.some(
          (p) =>
            Object.prototype.hasOwnProperty.call(ownProps(p), k) || matchesPattern(k, p.patternProperties, ctx) !== undefined,
        );
        if (!known) delete obj[k];
      }
    }
    for (const k of Object.keys(obj)) {
      for (const part of parts) {
        const props = ownProps(part);
        let sub: unknown;
        if (Object.prototype.hasOwnProperty.call(props, k)) sub = props[k];
        else sub = matchesPattern(k, part.patternProperties, ctx) ?? part.additionalProperties;
        if (isPlainObject(sub)) obj[k] = pruneAt(obj[k], sub, ctx, depth + 1);
      }
    }
    out = obj;
  } else if (Array.isArray(value)) {
    const arr = [...value];
    for (const part of parts) {
      const prefix = Array.isArray(part.prefixItems) ? part.prefixItems : [];
      for (let i = 0; i < arr.length; i++) {
        const sub = i < prefix.length ? prefix[i] : part.items;
        if (isPlainObject(sub)) arr[i] = pruneAt(arr[i], sub, ctx, depth + 1);
      }
    }
    out = arr;
  }

  for (const branch of branches) out = pruneAt(out, branch, ctx, depth + 1);
  return out;
}

/** The single branch `value` belongs to, or `null` when that cannot be told. */
function chooseBranch(value: unknown, list: unknown[], ctx: PruneContext, depth: number): Obj | null {
  // The choice depends only on the value and the list, so it is made once per
  // value: without this, a value under k nested oneOfs was re-decided once for
  // every branch of every enclosing oneOf.
  const memo = typeof value === "object" && value !== null ? ctx.choices : undefined;
  const known = memo?.get(value as object)?.get(list);
  if (known !== undefined) return known;
  const hits: { branch: Obj; pruned: string }[] = [];
  for (const branch of list) {
    if (!isPlainObject(branch)) continue;
    spend(ctx);
    const pruned = pruneAt(value, branch, ctx, depth + 1);
    if (conforms(pruned, branch, ctx, depth + 1)) hits.push({ branch, pruned: canonicalJson(pruned) });
  }
  // Several branches accept it (the matcher is lenient, so this happens for
  // `oneOf` too). Harmless only if they all prune it the same way.
  const chosen = hits.length > 0 && hits.every((h) => h.pruned === hits[0]!.pruned) ? hits[0]!.branch : null;
  if (memo !== undefined) {
    let byList = memo.get(value as object);
    if (byList === undefined) memo.set(value as object, (byList = new Map()));
    byList.set(list, chosen);
  }
  return chosen;
}

function typeOk(value: unknown, t: unknown): boolean {
  switch (t) {
    case "null": return value === null;
    case "boolean": return typeof value === "boolean";
    case "string": return typeof value === "string";
    case "number": return typeof value === "number" && Number.isFinite(value);
    case "integer": return typeof value === "number" && Number.isInteger(value);
    case "array": return Array.isArray(value);
    case "object": return isPlainObject(value);
    default: return true;
  }
}

function same(a: unknown, b: unknown): boolean {
  try {
    return canonicalJson(a) === canonicalJson(b);
  } catch {
    return false;
  }
}

/**
 * A conservative structural match — enough to tell `oneOf` branches apart by
 * their discriminators, not a validator. Unrecognised keywords pass. Answers
 * for an object or array value are remembered for the rest of the walk.
 */
function conforms(value: unknown, schema: unknown, ctx: PruneContext, depth: number): boolean {
  if (typeof schema === "boolean") return schema;
  if (depth > MAX_DEPTH) return true;
  const memo =
    typeof value === "object" && value !== null && typeof schema === "object" && schema !== null
      ? ctx.conformed
      : undefined;
  const known = memo?.get(value as object)?.get(schema as object);
  if (known !== undefined) return known;
  spend(ctx);
  const answer = conformsUncached(value, schema, ctx, depth);
  if (memo !== undefined) {
    let bySchema = memo.get(value as object);
    if (bySchema === undefined) memo.set(value as object, (bySchema = new Map()));
    bySchema.set(schema as object, answer);
  }
  return answer;
}

function conformsUncached(value: unknown, schema: unknown, ctx: PruneContext, depth: number): boolean {
  for (const part of applicable(schema, ctx.root, depth)) {
    if (part.type !== undefined) {
      const types = Array.isArray(part.type) ? part.type : [part.type];
      if (!types.some((t) => typeOk(value, t))) return false;
    }
    if ("const" in part && !same(value, part.const)) return false;
    if (Array.isArray(part.enum) && !part.enum.some((e) => same(value, e))) return false;
    if (isPlainObject(value)) {
      if (Array.isArray(part.required) && !part.required.every((k) => typeof k === "string" && k in value)) return false;
      const props = ownProps(part);
      for (const [k, v] of Object.entries(value)) {
        if (Object.prototype.hasOwnProperty.call(props, k)) {
          if (!conforms(v, props[k], ctx, depth + 1)) return false;
        } else if (matchesPattern(k, part.patternProperties, ctx) === undefined && part.additionalProperties === false) {
          return false;
        }
      }
    }
    if (Array.isArray(value) && isPlainObject(part.items)) {
      if (!value.every((v) => conforms(v, part.items, ctx, depth + 1))) return false;
    }
    // Lenient: at least one branch, for `oneOf` as for `anyOf` — this matcher
    // ignores too many keywords to count exclusivity.
    for (const key of ["oneOf", "anyOf"] as const) {
      const list = part[key];
      if (Array.isArray(list) && !list.some((b) => conforms(value, b, ctx, depth + 1))) return false;
    }
  }
  return true;
}
