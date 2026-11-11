/**
 * The shape check SPEC §7.2 item 1 makes of the document envelope, and the
 * nesting bound that keeps every later walk of the document finite.
 *
 * `JSON.parse(body) as TrustTaskDocument<P>` checks nothing: the cast is a
 * compile-time statement, and the parsed value may carry an array where `id`
 * should be, or nest thousands of levels deep. The Rust, Go and Dart runtimes
 * refuse both while decoding into their typed envelope (serde_json stops at 128
 * levels); TypeScript has no decode step, so {@link consumeInbound} runs this
 * first and refuses `malformedRequest`.
 *
 * Hand-written and dependency-free, like the rest of the runtime.
 */

import type { RejectReason, TrustTaskDocument } from "./document.js";

/**
 * The deepest nesting of objects and arrays a consumed document may have, the
 * document object itself counting as one level. The same figure as the
 * recursion limit of serde_json, which trust-tasks-rs parses documents with.
 */
export const MAX_NESTING_DEPTH = 128;

const STRING_MEMBERS = [
  "id",
  "type",
  "threadId",
  "parentThreadId",
  "issuer",
  "recipient",
  "issuedAt",
  "expiresAt",
] as const;

const OBJECT_MEMBERS = ["payload", "ceremony", "proof"] as const;

const isObject = (v: unknown): v is Record<string, unknown> =>
  typeof v === "object" && v !== null && !Array.isArray(v);

function malformed(detail: string): RejectReason {
  return {
    code: "malformedRequest",
    message: `document is not a well-formed Trust Task document (SPEC §7.2 item 1): ${detail}`,
    retryable: false,
  };
}

/**
 * Whether `value` nests objects and arrays more than `limit` levels deep.
 *
 * Iterative, so it cannot itself run out of stack on the input it exists to
 * refuse.
 */
export function exceedsNestingDepth(value: unknown, limit: number = MAX_NESTING_DEPTH): boolean {
  const stack: Array<[unknown, number]> = [[value, 1]];
  while (stack.length > 0) {
    const [node, depth] = stack.pop()!;
    if (typeof node !== "object" || node === null) continue;
    if (depth > limit) return true;
    const children = Array.isArray(node) ? node : Object.values(node);
    for (const child of children) {
      if (typeof child === "object" && child !== null) stack.push([child, depth + 1]);
    }
  }
  return false;
}

/**
 * Check `doc` against the framework envelope (SPEC §4.2): an object no deeper
 * than {@link MAX_NESTING_DEPTH}, with a string `id` and `type`, an object
 * `payload`, and every other framework member it carries of the type the
 * framework schema gives it. Returns the `malformedRequest` reason to refuse
 * it with, or `null`.
 *
 * Formats are not checked here — a timestamp that does not parse is the
 * freshness check's to refuse, and a Type URI the routing's — only the types
 * every later step relies on.
 */
export function validateEnvelope(doc: unknown): RejectReason | null {
  if (!isObject(doc)) return malformed("not a JSON object");
  if (exceedsNestingDepth(doc)) {
    return malformed(`nested more than ${MAX_NESTING_DEPTH} levels deep`);
  }
  for (const name of ["id", "type", "payload"] as const) {
    if (doc[name] === undefined) return malformed(`\`${name}\` is missing`);
  }
  for (const name of STRING_MEMBERS) {
    if (doc[name] !== undefined && typeof doc[name] !== "string") {
      return malformed(`\`${name}\` is not a string`);
    }
  }
  for (const name of OBJECT_MEMBERS) {
    if (doc[name] !== undefined && !isObject(doc[name])) {
      return malformed(`\`${name}\` is not an object`);
    }
  }
  const context = doc["@context"];
  if (
    context !== undefined &&
    typeof context !== "string" &&
    !Array.isArray(context) &&
    !isObject(context)
  ) {
    return malformed("`@context` is not a string, an array or an object");
  }
  return null;
}

/**
 * The members of a document {@link validateEnvelope} refused that are safe to
 * address a refusal with: the string-valued routing members, and nothing else.
 * A member of the wrong type is left out rather than echoed back.
 */
export function envelopeForRefusal(doc: unknown): TrustTaskDocument<unknown> {
  const out: Record<string, unknown> = { payload: undefined };
  if (isObject(doc)) {
    for (const name of ["id", "type", "threadId", "parentThreadId", "issuer", "recipient"]) {
      if (typeof doc[name] === "string") out[name] = doc[name];
    }
  }
  return out as unknown as TrustTaskDocument<unknown>;
}
