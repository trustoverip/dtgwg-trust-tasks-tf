/**
 * The envelope check `consumeInbound` makes before anything reads the
 * document (SPEC §7.2 item 1, as far as the later steps rely on it), and the
 * shared item-11 vectors in `fixtures/consume` at the repository root.
 */

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { describe, it } from "node:test";

import {
  consequentialChecks,
  consumeInbound,
  InMemoryReplayGuard,
  MAX_NESTING_DEPTH,
  notConsequentialChecks,
  respondWith,
  UnauthenticatedTransport,
  validateEnvelope,
  type ConsumeChecks,
  type ConsumeOutcome,
  type SpecPolicy,
  type TrustTaskDocument,
} from "../src/_runtime/index.js";

const ME = "did:web:maintainer.example";
const PEER = "did:web:org.example";
const SPEC: SpecPolicy = {
  typeUri: "https://trusttasks.org/spec/acl/list/0.1",
  isBearer: false,
  isProofRequired: false,
  isRecipientRequired: false,
};
const CLOCK = () => "2026-01-01T00:00:00Z";
const NOW = Date.parse("2026-01-01T00:00:30Z");

const base = (): Record<string, unknown> => ({
  id: "req-1",
  type: SPEC.typeUri,
  issuer: PEER,
  recipient: ME,
  issuedAt: "2026-01-01T00:00:00Z",
  payload: { role: "admin" },
});

/** `depth` levels of arrays, the outermost counting as one. */
function nestedArrays(depth: number): unknown {
  let v: unknown = [];
  for (let i = 1; i < depth; i++) v = [v];
  return v;
}

function consume(
  doc: unknown,
  checks: ConsumeChecks = notConsequentialChecks(),
  spec: SpecPolicy = SPEC,
  myVid: string = ME,
  now: number = NOW,
): Promise<ConsumeOutcome<{ ok: true }>> {
  return consumeInbound<unknown, { ok: true }>({
    transport: new UnauthenticatedTransport(),
    spec,
    proofPolicy: { kind: "rejectIfPresent" },
    payloadPolicy: { kind: "acceptUnvalidated" },
    checks,
    doc: doc as TrustTaskDocument<unknown>,
    myVid,
    now,
    newErrorId: () => "err-1",
    clock: CLOCK,
    handler: (accepted) => respondWith(accepted, "resp-1", { ok: true as const }, CLOCK),
  });
}

describe("envelope member types (SPEC §7.2 item 1)", () => {
  const cases: Array<[string, (d: Record<string, unknown>) => void]> = [
    ["an array id", (d) => (d.id = ["req-1"])],
    ["an object id", (d) => (d.id = { v: "req-1" })],
    ["a numeric id", (d) => (d.id = 1)],
    ["a missing id", (d) => delete d.id],
    ["a non-string type", (d) => (d.type = 7)],
    ["a missing type", (d) => delete d.type],
    ["a missing payload", (d) => delete d.payload],
    ["an array payload", (d) => (d.payload = ["admin"])],
    ["a string payload", (d) => (d.payload = "admin")],
    ["a non-string issuer", (d) => (d.issuer = { id: PEER })],
    ["a non-string recipient", (d) => (d.recipient = [ME])],
    ["a non-string threadId", (d) => (d.threadId = 3)],
    ["a non-string parentThreadId", (d) => (d.parentThreadId = true)],
    ["a non-string issuedAt", (d) => (d.issuedAt = 1767225600000)],
    ["a non-string expiresAt", (d) => (d.expiresAt = {})],
    ["a null issuer", (d) => (d.issuer = null)],
    ["a non-object proof", (d) => (d.proof = "z3kg")],
    ["a non-object ceremony", (d) => (d.ceremony = ["step"])],
    ["a numeric @context", (d) => (d["@context"] = 1)],
  ];

  for (const [name, mutate] of cases) {
    it(`refuses ${name} with malformedRequest, before the handler`, async () => {
      const d = base();
      mutate(d);
      assert.notEqual(validateEnvelope(d), null);
      const outcome = await consume(d);
      assert.equal(outcome.kind, "rejected", JSON.stringify(outcome));
      if (outcome.kind !== "rejected") return;
      assert.equal(outcome.error.payload.code, "malformedRequest");
      // Addressed only from members that are strings: a mistyped one is not
      // echoed into the response.
      const echoed = JSON.stringify(outcome.error);
      for (const value of ["id", "type", "issuer", "recipient", "threadId"]) {
        if (typeof d[value] === "object" && d[value] !== null) {
          assert.ok(!echoed.includes(JSON.stringify(d[value])), `${value} echoed: ${echoed}`);
        }
      }
    });
  }

  it("refuses a value that is not an object at all", async () => {
    for (const value of [null, [], "doc", 3]) {
      const outcome = await consume(value);
      assert.equal(outcome.kind, "rejected");
      if (outcome.kind === "rejected") assert.equal(outcome.error.payload.code, "malformedRequest");
    }
  });

  it("accepts a well-formed document, with every optional member present", async () => {
    const d = {
      ...base(),
      threadId: "t-1",
      parentThreadId: "t-0",
      expiresAt: "2026-01-01T00:01:00Z",
      "@context": ["https://www.w3.org/ns/credentials/v2"],
      ceremony: { enactment: "e-1", step: "s-1" },
      extension: { anything: [1, 2, 3] },
    };
    assert.equal(validateEnvelope(d), null);
    assert.equal((await consume(d)).kind, "handled");
  });

  it("does not let an array id defeat the item-11 record", async () => {
    // Before the envelope check, two parses of the same bytes produced two
    // distinct array keys, so a replay was never recognised.
    const guard = new InMemoryReplayGuard();
    const bytes = JSON.stringify({ ...base(), id: ["same"] });
    for (let i = 0; i < 2; i++) {
      const outcome = await consume(JSON.parse(bytes), consequentialChecks(guard));
      assert.equal(outcome.kind, "rejected");
    }
  });
});

describe("nesting depth", () => {
  it(`accepts ${MAX_NESTING_DEPTH} levels, the document counting as one`, async () => {
    // document (1) → payload (2) → arrays (3..MAX).
    const d = { ...base(), payload: { deep: nestedArrays(MAX_NESTING_DEPTH - 2) } };
    assert.equal(validateEnvelope(d), null);
    assert.equal((await consume(d)).kind, "handled");
  });

  it(`refuses ${MAX_NESTING_DEPTH + 1} levels with malformedRequest`, async () => {
    const d = { ...base(), payload: { deep: nestedArrays(MAX_NESTING_DEPTH - 1) } };
    const outcome = await consume(d);
    assert.equal(outcome.kind, "rejected");
    if (outcome.kind === "rejected") assert.equal(outcome.error.payload.code, "malformedRequest");
  });

  it("answers a deeply nested payload instead of throwing, under item 11 too", async () => {
    // Deep enough that a recursive canonical form exhausts the stack.
    for (const depth of [5_000, 100_000]) {
      const d = { ...base(), payload: { deep: nestedArrays(depth) } };
      const outcome = await consume(d, consequentialChecks(new InMemoryReplayGuard()));
      assert.equal(outcome.kind, "rejected");
      if (outcome.kind === "rejected") assert.equal(outcome.error.payload.code, "malformedRequest");
    }
  });

  it("refuses deep nesting in any member, not only the payload", async () => {
    const d = { ...base(), extension: nestedArrays(10_000) };
    const outcome = await consume(d, consequentialChecks(new InMemoryReplayGuard()));
    assert.equal(outcome.kind, "rejected");
  });
});

describe("shared item-11 vectors (fixtures/consume)", () => {
  const vectors = JSON.parse(
    readFileSync(new URL("../../../fixtures/consume/item11-digest.json", import.meta.url), "utf8"),
  ) as {
    spec: string;
    consumer: string;
    now: string;
    first: string;
    cases: Array<{ name: string; second: string; expect: string }>;
  };
  const spec: SpecPolicy = { ...SPEC, typeUri: vectors.spec };

  for (const c of vectors.cases) {
    it(c.name, async () => {
      const guard = new InMemoryReplayGuard();
      const run = (body: string) =>
        consume(
          JSON.parse(body),
          consequentialChecks(guard),
          spec,
          vectors.consumer,
          Date.parse(vectors.now),
        );
      assert.equal((await run(vectors.first)).kind, "handled");
      const second = await run(c.second);
      const got =
        second.kind === "rejected" && second.error.payload.code === "idConflict"
          ? "idConflict"
          : second.kind;
      assert.equal(got, c.expect);
    });
  }
});
