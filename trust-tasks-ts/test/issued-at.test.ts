/**
 * #970: every instant this library stamps is written in whole seconds, and
 * every RFC 3339 precision is still read. Mirrors trust-tasks-rs's
 * tests/issued_at_precision.rs.
 *
 * SPEC §13.1 item 3 calls a full-precision timestamp a fingerprint and lets a
 * producer round `issuedAt` down. `toISOString()` writes milliseconds.
 */

import assert from "node:assert/strict";
import { describe, it } from "node:test";

import {
  CONSEQUENTIAL_FRESHNESS,
  issuedAtNow,
  rejectWith,
  respondWith,
  validateFreshness,
  type TrustTaskDocument,
} from "../src/_runtime/index.js";

const WHOLE_SECONDS = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/;

const request: TrustTaskDocument<Record<string, never>> = {
  id: "5b0e8f3a-91c2-8d4e-a6f0-2c7d93b1e4a5",
  threadId: "5b0e8f3a-91c2-8d4e-a6f0-2c7d93b1e4a5",
  type: "https://real-life.org/trust-tasks/spec/sealed-envelope/0.1",
  issuer: "did:key:z6MkProducer",
  recipient: "did:key:z6MkConsumer",
  issuedAt: "2026-10-27T18:04:11Z",
  payload: {},
};

describe("#970 — instants this library writes", () => {
  it("issuedAtNow is whole seconds, rounded down", () => {
    const before = Date.now();
    const stamped = issuedAtNow();
    assert.match(stamped, WHOLE_SECONDS);
    const at = Date.parse(stamped);
    assert.ok(at <= before + 1000 && at > before - 1000, stamped);
    assert.equal(at % 1000, 0);
  });

  it("rejectWith and respondWith stamp whole seconds by default", () => {
    const error = rejectWith(request, "urn:uuid:2", {
      code: "taskFailed",
      retryable: false,
    });
    assert.match(error.issuedAt!, WHOLE_SECONDS);
    assert.match(respondWith(request, "urn:uuid:3", {}).issuedAt!, WHOLE_SECONDS);
  });
});

describe("#970 — instants this library reads", () => {
  it("takes every RFC 3339 precision, and an offset", () => {
    const issued = Date.parse("2026-10-27T18:04:11Z");
    for (const issuedAt of [
      "2026-10-27T18:04:11Z",
      "2026-10-27T18:04:11.5Z",
      "2026-10-27T18:04:11.720Z",
      "2026-10-27T18:04:11.720263Z",
      "2026-10-27T18:04:11.720263396Z",
      "2026-10-27T20:04:11.720+02:00",
    ]) {
      const reason = validateFreshness({ ...request, issuedAt }, issued + 1000, CONSEQUENTIAL_FRESHNESS);
      assert.equal(reason, null, `${issuedAt}: ${JSON.stringify(reason)}`);
    }
  });
});
