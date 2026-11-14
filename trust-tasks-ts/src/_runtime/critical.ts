/**
 * SPEC §4.5.2 — namespaces a producer marks critical with `extCritical`.
 *
 * An `ext` namespace is ignorable by default (§4.5.1 rule 5). A specification
 * may let a producer list some of them in a sibling `extCritical` array, and a
 * consumer that does not recognize a namespace listed there MUST NOT process
 * the document and MUST reject it with `unsupportedExtension` (§4.5.2 rule 4).
 * An entry naming a namespace absent from the sibling `ext` is non-conforming
 * and is rejected with `malformedRequest` (rule 2).
 *
 * `extCritical` exists only at the object levels a specification opts in at,
 * so the payload is walked alongside its schema:
 *
 * - a level whose schema declares an `extCritical` property is checked;
 * - a level whose schema closes it (`additionalProperties: false` or
 *   `unevaluatedProperties: false`) without declaring it is refused as
 *   `malformedRequest` when the member is present — what a schema validator
 *   reports, applied even under `acceptUnvalidated` so the marking is never
 *   dropped silently;
 * - a level the schema leaves open is data (a document a vault is asked to
 *   sign, say), and a member named `extCritical` in it is not a marking.
 *
 * Every branch of `allOf` / `anyOf` / `oneOf` is followed, so a level is
 * checked where any branch declares the member. With no schema, every level
 * outside an `ext` namespace's own content is checked. The Rust, Go and Dart
 * runtimes implement the same walk; `fixtures/consume/ext-critical.json` pins
 * the four together.
 */

import type { RejectReason } from "./document.js";

/** How deep the walk follows a payload. */
const MAX_DEPTH = 128;
/** Base work allowance for one walk, in steps. */
const BASE_STEPS = 100_000;
/** Further steps allowed for every value in the payload. */
const STEPS_PER_VALUE = 64;
/** The longest namespace echoed in an `unsupportedExtension` message. */
const MAX_ECHOED_NAMESPACE = 128;

const EXT_CRITICAL = "extCritical";
/** §4.5.1 rule 2. */
const NAMESPACE = /^[a-z][a-z0-9-]*(\.[a-z0-9-]+)+$/;

type Json = unknown;
type Schema = Record<string, unknown>;

class Refusal {
  constructor(readonly reason: RejectReason) {}
}

function malformed(message: string): Refusal {
  return new Refusal({
    code: "malformedRequest",
    message: `${message} (SPEC §4.5.2)`,
    retryable: false,
  });
}

const TOO_COMPLEX = (): Refusal =>
  malformed(
    "payload is too deeply nested or too complex to check its critical extensions",
  );

function isObject(v: unknown): v is Record<string, unknown> {
  return typeof v === "object" && v !== null && !Array.isArray(v);
}

function has(o: Record<string, unknown>, key: string): boolean {
  return Object.prototype.hasOwnProperty.call(o, key);
}

/** The number of values in `payload` when a member named `extCritical` appears anywhere in it, or `null`. */
function countValuesIfMarked(payload: Json): number | null {
  let n = 0;
  let marked = false;
  const stack: Json[] = [payload];
  while (stack.length > 0) {
    const v = stack.pop();
    n += 1;
    if (Array.isArray(v)) {
      for (const e of v) stack.push(e);
    } else if (isObject(v)) {
      if (has(v, EXT_CRITICAL)) marked = true;
      for (const k of Object.keys(v)) stack.push(v[k]);
    }
  }
  return marked ? n : null;
}

function resolve(ref: string, root: Json): Json | undefined {
  if (!ref.startsWith("#")) return undefined;
  const pointer = ref.slice(1);
  if (pointer === "") return root;
  if (!pointer.startsWith("/")) return undefined;
  let node: Json = root;
  for (const raw of pointer.slice(1).split("/")) {
    const token = raw.replace(/~1/g, "/").replace(/~0/g, "~");
    if (Array.isArray(node)) {
      const i = Number(token);
      if (!Number.isInteger(i)) return undefined;
      node = node[i];
    } else if (isObject(node) && has(node, token)) {
      node = node[token];
    } else {
      return undefined;
    }
  }
  return node;
}

function property(schema: Schema, name: string): Json | undefined {
  const props = schema["properties"];
  return isObject(props) && has(props, name) ? props[name] : undefined;
}

function closes(schema: Schema): boolean {
  return schema["additionalProperties"] === false || schema["unevaluatedProperties"] === false;
}

class Walk {
  private left: number;

  constructor(
    private readonly recognized: ReadonlySet<string>,
    private readonly root: Json | undefined,
    values: number,
  ) {
    this.left = BASE_STEPS + STEPS_PER_VALUE * values;
  }

  private spend(depth: number): void {
    if (depth > MAX_DEPTH || this.left <= 0) throw TOO_COMPLEX();
    this.left -= 1;
  }

  blind(value: Json, depth: number): void {
    this.spend(depth);
    if (Array.isArray(value)) {
      for (const e of value) this.blind(e, depth + 1);
    } else if (isObject(value)) {
      if (has(value, EXT_CRITICAL)) this.checkLevel(value);
      for (const k of Object.keys(value)) {
        if (k !== "ext" && k !== EXT_CRITICAL) this.blind(value[k], depth + 1);
      }
    }
  }

  guided(value: Json, schemas: readonly Json[], depth: number): void {
    this.spend(depth);
    if (!Array.isArray(value) && !isObject(value)) return;
    const applicable: Schema[] = [];
    for (const s of schemas) this.expand(s, applicable, depth);
    if (applicable.length === 0) return;
    if (Array.isArray(value)) {
      value.forEach((element, i) => {
        const children = elementSchemas(applicable, i);
        if (children.length > 0) this.guided(element, children, depth + 1);
      });
      return;
    }
    if (has(value, EXT_CRITICAL)) {
      if (applicable.some((s) => property(s, EXT_CRITICAL) !== undefined)) {
        this.checkLevel(value);
      } else if (applicable.some(closes)) {
        throw malformed("`extCritical` is not allowed at this level of the payload");
      }
    }
    for (const k of Object.keys(value)) {
      if (k === EXT_CRITICAL) continue;
      const children = memberSchemas(applicable, k);
      if (children.length > 0) this.guided(value[k], children, depth + 1);
    }
  }

  private expand(schema: Json, out: Schema[], depth: number): void {
    this.spend(depth);
    if (!isObject(schema) || out.includes(schema)) return;
    out.push(schema);
    const ref = schema["$ref"];
    if (typeof ref === "string" && this.root !== undefined) {
      const target = resolve(ref, this.root);
      if (target !== undefined) this.expand(target, out, depth + 1);
    }
    for (const keyword of ["allOf", "anyOf", "oneOf"]) {
      const branches = schema[keyword];
      if (Array.isArray(branches)) {
        for (const b of branches) this.expand(b, out, depth + 1);
      }
    }
  }

  /** Rules 1 and 2, then rule 4, at one object level. */
  private checkLevel(level: Record<string, unknown>): void {
    const entries = level[EXT_CRITICAL];
    if (!Array.isArray(entries)) throw malformed("`extCritical` must be an array");
    if (entries.length === 0) throw malformed("`extCritical` must name at least one namespace");
    const ext = level["ext"];
    if (!isObject(ext)) {
      throw malformed("`extCritical` is present where there is no sibling `ext` object");
    }
    const seen = new Set<string>();
    for (const entry of entries) {
      if (typeof entry !== "string") throw malformed("every `extCritical` entry must be a string");
      if (!NAMESPACE.test(entry)) {
        throw malformed("every `extCritical` entry must be a reverse-DNS namespace");
      }
      if (seen.has(entry)) throw malformed("`extCritical` entries must be unique");
      seen.add(entry);
      if (!has(ext, entry)) {
        throw malformed(
          "every `extCritical` entry must name a namespace present in the sibling `ext`",
        );
      }
    }
    for (const name of seen) {
      if (!this.recognized.has(name)) {
        throw new Refusal({
          code: "unsupportedExtension",
          message: `unsupported critical extension: ${name.slice(0, MAX_ECHOED_NAMESPACE)}`,
          retryable: false,
        });
      }
    }
  }
}

function memberSchemas(applicable: readonly Schema[], name: string): Json[] {
  const out: Json[] = [];
  for (const schema of applicable) {
    const p = property(schema, name);
    if (p !== undefined) {
      out.push(p);
      continue;
    }
    // Taken whole: matching the patterns would only narrow the places a
    // marking may be read, and the other runtimes have no regex engine here.
    const patterns = schema["patternProperties"];
    if (isObject(patterns)) for (const k of Object.keys(patterns)) out.push(patterns[k]);
    const additional = schema["additionalProperties"];
    if (isObject(additional)) out.push(additional);
  }
  return out;
}

function elementSchemas(applicable: readonly Schema[], index: number): Json[] {
  const out: Json[] = [];
  for (const schema of applicable) {
    const prefix = schema["prefixItems"];
    if (Array.isArray(prefix) && index < prefix.length) {
      out.push(prefix[index]);
      continue;
    }
    const items = schema["items"];
    if (isObject(items)) out.push(items);
  }
  return out;
}

/**
 * Apply SPEC §4.5.2 to `payload`, read under `schema` — the payload schema of
 * the specification the document is consumed under (`spec.payloadSchema`), or
 * `undefined` to read every level. `recognized` is every `ext` namespace this
 * consumer implements.
 *
 * Returns `null`, or the refusal: `unsupportedExtension` for the first
 * critical namespace not recognized, `malformedRequest` for a marking that
 * breaks rules 1 and 2 or sits at a level that does not allow it. A payload
 * with no member named `extCritical` anywhere returns `null` after one pass.
 */
export function checkCriticalExtensions(
  payload: unknown,
  schema: unknown,
  recognized: Iterable<string> = [],
): RejectReason | null {
  const values = countValuesIfMarked(payload);
  if (values === null) return null;
  const walk = new Walk(new Set(recognized), schema ?? undefined, values);
  try {
    if (schema === undefined || schema === null) walk.blind(payload, 0);
    else walk.guided(payload, [schema], 0);
    return null;
  } catch (e) {
    if (e instanceof Refusal) return e.reason;
    // A stack overflow or similar: refuse rather than pass what was not read.
    if (e instanceof RangeError) return TOO_COMPLEX().reason;
    throw e;
  }
}
