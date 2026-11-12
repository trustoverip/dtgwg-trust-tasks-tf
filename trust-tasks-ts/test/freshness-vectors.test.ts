/**
 * The shared acceptance-window vectors in `fixtures/consume/freshness.json` at
 * the repository root, which the Rust, Go and Dart runtimes also run: item 4's
 * expiry check, then the freshness bound, then the record expiry, so that the
 * four runtimes reach the same verdict on the same document and keep its
 * replay record for the same time (SPEC §7.2, *Bounding the record*).
 */

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { describe, it } from "node:test";

import {
  CONSEQUENTIAL_FRESHNESS,
  DEFAULT_FRESHNESS,
  recordExpiry,
  validateBasic,
  validateFreshness,
  type FreshnessPolicy,
  type TrustTaskDocument,
} from "../src/_runtime/index.js";

interface VectorPolicy {
  skewMs: number;
  maxAgeMs?: number;
  expiryHorizonMs?: number;
  requireIssuedAt?: boolean;
}

interface Vectors {
  policies: Record<string, VectorPolicy>;
  cases: {
    name: string;
    policy: string;
    now: string;
    issuedAt?: string;
    expiresAt?: string;
    verdict: string;
    recordExpiry?: string | null;
  }[];
}

const vectors = JSON.parse(
  readFileSync(new URL("../../../fixtures/consume/freshness.json", import.meta.url), "utf8"),
) as Vectors;

function policyOf(v: VectorPolicy): FreshnessPolicy {
  return {
    skewMs: v.skewMs,
    ...(v.maxAgeMs === undefined ? {} : { maxAgeMs: v.maxAgeMs }),
    ...(v.expiryHorizonMs === undefined ? {} : { expiryHorizonMs: v.expiryHorizonMs }),
    requireIssuedAt: v.requireIssuedAt ?? false,
  };
}

/** A policy as the vectors spell it, so an unset member compares equal to absent. */
function normalised(p: FreshnessPolicy): FreshnessPolicy {
  return policyOf(p);
}

describe("shared freshness vectors (fixtures/consume/freshness.json)", () => {
  it("default and consequential are the library's own policies", () => {
    assert.deepEqual(policyOf(vectors.policies.default!), normalised(DEFAULT_FRESHNESS));
    assert.deepEqual(policyOf(vectors.policies.consequential!), normalised(CONSEQUENTIAL_FRESHNESS));
  });

  for (const c of vectors.cases) {
    it(c.name, () => {
      const policy = policyOf(vectors.policies[c.policy]!);
      const now = Date.parse(c.now);
      const doc: TrustTaskDocument<object> = {
        id: "req-1",
        type: "https://trusttasks.org/spec/acl/show/0.2",
        payload: {},
        ...(c.issuedAt === undefined ? {} : { issuedAt: c.issuedAt }),
        ...(c.expiresAt === undefined ? {} : { expiresAt: c.expiresAt }),
      };
      const reason = validateBasic(doc, now, "") ?? validateFreshness(doc, now, policy);
      assert.equal(reason?.code ?? "accept", c.verdict);
      if (reason === null) {
        const expected = c.recordExpiry === null || c.recordExpiry === undefined
          ? undefined
          : Date.parse(c.recordExpiry);
        assert.equal(recordExpiry(doc, policy, now), expected, "record expiry");
      }
    });
  }
});
