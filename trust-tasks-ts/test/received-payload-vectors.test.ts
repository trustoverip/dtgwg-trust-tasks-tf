/**
 * The shared vectors in `fixtures/consume/received-payload.json` at the
 * repository root, which the Rust, Go and Dart runtimes also run: SPEC §7.2
 * item 2 checks the payload as received — a `null` the schema forbids is
 * refused, though a typed decode would read it as absent — and a later version
 * read forward along `extends` pruned, as `readForSpec` reads it.
 *
 * TypeScript has no typed decode: `consumeInbound` validates `doc.payload`,
 * which is the payload as received (pruned, for a `readForSpec` reading).
 */

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { describe, it } from "node:test";

import {
  consumeInbound,
  notConsequentialChecks,
  readForSpec,
  respondWith,
  UnauthenticatedTransport,
  type SpecPolicy,
} from "../src/_runtime/index.js";

interface Vectors {
  implements: { typeUri: string; payloadSchema: string };
  extends: Record<string, string>;
  consumer: string;
  now: string;
  cases: { name: string; document: string; verdict: string; validated: unknown }[];
}

const vectors = JSON.parse(
  readFileSync(new URL("../../../fixtures/consume/received-payload.json", import.meta.url), "utf8"),
) as Vectors;

const spec: SpecPolicy = {
  typeUri: vectors.implements.typeUri,
  isBearer: false,
  isProofRequired: false,
  isRecipientRequired: false,
  payloadSchema: JSON.parse(vectors.implements.payloadSchema),
} as SpecPolicy;

/**
 * Records the payload it is handed and checks the vector schema by hand: an
 * object, `subject` present, every member named in `properties`, every member
 * a string.
 */
function recording(): { seen: unknown; validate: (schema: unknown, payload: unknown) => boolean } {
  const v = {
    seen: undefined as unknown,
    validate(_schema: unknown, payload: unknown): boolean {
      v.seen = structuredClone(payload);
      if (typeof payload !== "object" || payload === null || Array.isArray(payload)) return false;
      const members = payload as Record<string, unknown>;
      if (!("subject" in members)) return false;
      return Object.entries(members).every(
        ([name, value]) => (name === "subject" || name === "role") && typeof value === "string",
      );
    },
  };
  return v;
}

describe("shared received-payload vectors (fixtures/consume/received-payload.json)", () => {
  assert.ok(vectors.cases.length > 0);
  for (const c of vectors.cases) {
    it(c.name, async () => {
      const read = readForSpec<unknown>(JSON.parse(c.document), spec, { index: vectors.extends });
      assert.equal(read.kind, "accepted");
      if (read.kind !== "accepted") return;
      const validator = recording();
      const outcome = await consumeInbound<unknown, Record<string, never>>({
        transport: new UnauthenticatedTransport(),
        spec,
        proofPolicy: { kind: "acceptUnverified" },
        payloadPolicy: { kind: "validate", validate: validator },
        checks: notConsequentialChecks(),
        doc: read.doc,
        myVid: vectors.consumer,
        now: Date.parse(vectors.now),
        newErrorId: () => "urn:uuid:error",
        handler: (d) => respondWith(d, "urn:uuid:answer", {}),
      });
      const verdict =
        outcome.kind === "handled"
          ? "accept"
          : outcome.kind === "rejected"
            ? outcome.error.payload.code
            : outcome.kind;
      assert.equal(verdict, c.verdict);
      assert.deepEqual(validator.seen, c.validated);
    });
  }
});
