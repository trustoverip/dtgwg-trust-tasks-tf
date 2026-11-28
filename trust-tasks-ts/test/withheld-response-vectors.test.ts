/**
 * A handler may keep its response out of the SPEC §7.2 item 11 record
 * (`parties.delivery.withholdResponse()`); a duplicate of that document then comes
 * back to it as a re-delivery (`parties.delivery.redelivery`) instead of being
 * answered from the record. Runs the shared vectors in
 * `fixtures/consume/withheld-response.json`, which the Rust, Go and Dart
 * runtimes run too.
 */

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { describe, it } from "node:test";

import {
  consequentialChecks,
  consumeInbound,
  InMemoryReplayGuard,
  respondWith,
  UnauthenticatedTransport,
  type ReplayGuard,
  type SpecPolicy,
  type TrustTaskDocument,
} from "../src/_runtime/index.js";

const CLOCK = () => "2026-11-01T00:00:00Z";

const vectors = JSON.parse(
  readFileSync(new URL("../../../fixtures/consume/withheld-response.json", import.meta.url), "utf8"),
) as {
  spec: string;
  consumer: string;
  now: string;
  documents: Record<string, string>;
  cases: Array<{
    name: string;
    withhold: boolean;
    deliveries: Array<{
      document: string;
      handler: "first" | "redelivery" | null;
      outcome: string;
      priorResponseOf?: number;
    }>;
  }>;
};

const SPEC: SpecPolicy = {
  typeUri: vectors.spec,
  isBearer: false,
  isProofRequired: false,
  isRecipientRequired: false,
};

type Answer = { delivery: "first" | "redelivery" };

async function consume(guard: ReplayGuard, body: string, index: number, withhold: boolean) {
  let seen: "first" | "redelivery" | null = null;
  const outcome = await consumeInbound<unknown, Answer>({
    transport: new UnauthenticatedTransport(),
    spec: SPEC,
    proofPolicy: { kind: "rejectIfPresent" },
    payloadPolicy: { kind: "acceptUnvalidated" },
    checks: consequentialChecks(guard),
    doc: JSON.parse(body) as TrustTaskDocument<unknown>,
    myVid: vectors.consumer,
    now: Date.parse(vectors.now),
    newErrorId: () => "err-1",
    clock: CLOCK,
    handler: async (accepted, parties) => {
      await Promise.resolve();
      const delivery = parties.delivery!;
      seen = delivery.redelivery ? "redelivery" : "first";
      if (withhold) delivery.withholdResponse();
      return respondWith(accepted, `resp-${index}`, { delivery: seen }, CLOCK);
    },
  });
  return { outcome, seen };
}

describe("withheld-response vectors (fixtures/consume/withheld-response.json)", () => {
  for (const c of vectors.cases) {
    it(c.name, async () => {
      const guard = new InMemoryReplayGuard(16);
      const responses: Array<string | undefined> = [];
      for (const [index, d] of c.deliveries.entries()) {
        const { outcome, seen } = await consume(guard, vectors.documents[d.document]!, index, c.withhold);
        const got = outcome.kind === "rejected" ? outcome.error.payload.code : outcome.kind;
        assert.equal(got, d.outcome, `#${index}`);
        assert.equal(seen, d.handler, `#${index}: handler`);
        if (outcome.kind === "handled") {
          assert.equal(outcome.response.payload.delivery, d.handler);
          responses.push(outcome.response.id);
        } else if (outcome.kind === "duplicate") {
          assert.equal(outcome.inFlight, false);
          const prior = outcome.priorResponse as { id: string } | undefined;
          assert.equal(prior?.id, responses[d.priorResponseOf!], `#${index}: replays`);
          responses.push(undefined);
        } else {
          responses.push(undefined);
        }
      }
    });
  }

  it("a guard without recordWithheld absorbs the duplicate in silence", async () => {
    const inner = new InMemoryReplayGuard(16);
    const guard: ReplayGuard = {
      claim: (id, digest, retainUntil, now) => inner.claim(id, digest, retainUntil, now),
      recordResponse: (id, response) => inner.recordResponse(id, response),
      release: (id, digest) => inner.release(id, digest),
    };
    const body = vectors.documents.original!;
    assert.equal((await consume(guard, body, 0, true)).outcome.kind, "handled");
    const again = await consume(guard, body, 1, true);
    assert.deepEqual(again.outcome, { kind: "duplicate", inFlight: false });
    assert.equal(again.seen, null);
  });
});
