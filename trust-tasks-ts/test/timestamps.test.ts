/**
 * The shared RFC 3339 vectors in `fixtures/consume/timestamps.json` at the
 * repository root, which the Rust, Go and Dart runtimes also run: every field
 * of `issuedAt` / `expiresAt` must be in range, whatever `Date.parse` would
 * roll over or accept.
 */

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { describe, it } from "node:test";

import { parseRfc3339 } from "../src/_runtime/document.js";
import {
  CONSEQUENTIAL_FRESHNESS,
  validateBasic,
  validateFreshness,
  type TrustTaskDocument,
} from "../src/_runtime/index.js";

interface Vectors {
  valid: { value: string; epochMs: number; why: string }[];
  invalid: { value: string; why: string }[];
}

const vectors = JSON.parse(
  readFileSync(new URL("../../../fixtures/consume/timestamps.json", import.meta.url), "utf8"),
) as Vectors;

describe("shared RFC 3339 vectors (fixtures/consume/timestamps.json)", () => {
  for (const v of vectors.valid) {
    it(`reads ${v.value} (${v.why})`, () => {
      assert.equal(parseRfc3339(v.value), v.epochMs);
    });
  }
  for (const v of vectors.invalid) {
    it(`refuses ${v.value} (${v.why})`, () => {
      assert.ok(Number.isNaN(parseRfc3339(v.value)), `parsed to ${parseRfc3339(v.value)}`);
    });
  }
});

describe("a calendar-invalid timestamp in the pipeline", () => {
  const doc = (over: Partial<TrustTaskDocument<object>>): TrustTaskDocument<object> => ({
    id: "req-1",
    type: "https://trusttasks.org/spec/acl/show/0.2",
    issuer: "did:key:z6MkProducer",
    recipient: "did:key:z6MkConsumer",
    payload: {},
    ...over,
  });
  const now = Date.parse("2026-02-01T00:00:00Z");

  it("an expiresAt of February 30 is malformedRequest, not 2 March", () => {
    const reason = validateBasic(doc({ expiresAt: "2026-02-30T00:00:00Z" }), now, "did:key:z6MkConsumer");
    assert.equal(reason?.code, "malformedRequest");
  });

  it("an issuedAt with hour 24 is refused, not read as the next day", () => {
    // Read as 2026-02-01T00:00:00Z it would be exactly `now`, and fresh.
    const reason = validateFreshness(doc({ issuedAt: "2026-01-31T24:00:00Z" }), now, CONSEQUENTIAL_FRESHNESS);
    assert.equal(reason?.code, "malformedRequest");
  });
});
