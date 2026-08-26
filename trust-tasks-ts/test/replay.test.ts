/**
 * SPEC §7.2 item 11, §8.4 and the §7.2 freshness bound, for the TypeScript
 * pipeline.
 *
 * These mirror `trust-tasks-rs/tests/replay_guard.rs` and the unit tests in
 * `src/replay.rs` / `src/freshness.rs`. The two reference implementations must
 * reach the same verdict on the same document; where a case exists there and
 * not here, the languages can drift apart without anything noticing.
 */

import assert from "node:assert/strict";
import { describe, it } from "node:test";

import {
  canonicalJson,
  consequentialChecks,
  consumeInbound,
  documentDigest,
  EXPIRY_NOT_AFTER_ISSUANCE,
  FUTURE_ISSUED_AT,
  ID_CONFLICT_WIRE_MESSAGE,
  InMemoryReplayGuard,
  notConsequentialChecks,
  PROOF_INVALID_WIRE_MESSAGE,
  sha256Hex,
  STALE_WIRE_MESSAGE,
  StaticTransport,
  validateFreshness,
  type ConsumeOutcome,
  type ReplayGuard,
  type SpecPolicy,
  type TrustTaskDocument,
} from "../src/_runtime/index.js";

const ME = "did:web:maintainer.example";
const PEER = "did:web:org.example";
const NOW = Date.parse("2026-08-26T12:00:00Z");

interface Payload {
  entry: { subject: string; role: string };
}
interface Response {
  entry: { subject: string; role: string };
}

const SPEC: SpecPolicy = {
  typeUri: "https://trusttasks.org/spec/acl/grant/0.1",
  isBearer: false,
  isProofRequired: false,
  isRecipientRequired: true,
};

function doc(over: Partial<TrustTaskDocument<Payload>> = {}): TrustTaskDocument<Payload> {
  return {
    id: "req-1",
    type: SPEC.typeUri,
    issuer: PEER,
    recipient: ME,
    issuedAt: "2026-08-26T12:00:00Z",
    payload: { entry: { subject: "did:web:alice.example", role: "admin" } },
    ...over,
  };
}

/**
 * Counts handler invocations. The count *is* the assertion: item 11 is a
 * statement about how many times the consequential effect happens.
 */
function runner(guard: ReplayGuard) {
  const state = { executions: 0 };
  const run = (
    d: TrustTaskDocument<Payload>,
    now = NOW,
  ): Promise<ConsumeOutcome<Response>> =>
    consumeInbound<Payload, Response>({
      transport: new StaticTransport({ issuer: PEER, recipient: ME }),
      spec: SPEC,
      proofPolicy: { kind: "acceptUnverified" },
      payloadPolicy: { kind: "acceptUnvalidated" },
      checks: consequentialChecks(guard),
      doc: d,
      myVid: ME,
      now,
      newErrorId: () => "err-1",
      clock: () => "2026-08-26T12:00:00.000Z",
      handler: (accepted) => {
        state.executions += 1;
        return {
          id: "resp-1",
          type: `${accepted.type}#response`,
          issuer: ME,
          recipient: PEER,
          issuedAt: "2026-08-26T12:00:00.000Z",
          payload: accepted.payload,
        } as TrustTaskDocument<Response>;
      },
    });
  return { run, state };
}

describe("SPEC §7.2 item 11 — duplicate execution", () => {
  it("absorbs a bit-for-bit resend and never executes twice (§8.4)", async () => {
    const { run, state } = runner(new InMemoryReplayGuard(64));

    const first = await run(doc());
    assert.equal(first.kind, "handled");
    assert.equal(state.executions, 1);

    const second = await run(doc());
    assert.equal(second.kind, "duplicate");
    if (second.kind !== "duplicate") return;
    assert.equal(second.inFlight, false);
    assert.equal((second.priorResponse as TrustTaskDocument<Response>).id, "resp-1");
    assert.equal(state.executions, 1, "the consequential effect happened twice");

    // …and again, arbitrarily often.
    assert.equal((await run(doc())).kind, "duplicate");
    assert.equal(state.executions, 1);
  });

  it("rejects differing content under a reused id with idConflict", async () => {
    const { run, state } = runner(new InMemoryReplayGuard(64));

    await run(doc({ payload: { entry: { subject: "did:web:alice.example", role: "reader" } } }));
    const escalated = await run(
      doc({ payload: { entry: { subject: "did:web:alice.example", role: "admin" } } }),
    );

    assert.equal(escalated.kind, "rejected");
    if (escalated.kind !== "rejected") return;
    assert.equal(escalated.error.payload.code, "idConflict");
    assert.equal(escalated.error.payload.retryable, false);
    assert.equal(escalated.error.payload.message, ID_CONFLICT_WIRE_MESSAGE);
    assert.equal(state.executions, 1, "the escalated document executed");
  });

  it("treats a re-signed proof over identical content as a conflict, not a retry", () => {
    // SPEC §4.9.3: item 11 asks *which serialization arrived*, so a re-signed
    // proof makes a different document. This is why the item-11 digest covers
    // `proof`, unlike the §4.9.3 task digest.
    const proof = {
      type: "DataIntegrityProof",
      cryptosuite: "eddsa-jcs-2022",
      verificationMethod: `${PEER}#key-1`,
      created: "2026-08-26T12:00:00Z",
      proofPurpose: "assertionMethod",
      proofValue: "zAAA",
    };
    const signed = doc({ proof });
    const resigned = doc({ proof: { ...proof, proofValue: "zBBB" } });
    const unsigned = doc();

    assert.notEqual(documentDigest(signed), documentDigest(resigned));
    assert.notEqual(documentDigest(signed), documentDigest(unsigned));
  });

  it("releases the record once the acceptance window closes", async () => {
    const guard = new InMemoryReplayGuard(64);
    const { run, state } = runner(guard);

    await run(doc());
    assert.equal(guard.size, 1);

    // `consequentialChecks` uses a five-minute window, so the retention
    // deadline is `issuedAt + 5m`.
    guard.purgeExpired(Date.parse("2026-08-26T12:06:00Z"));
    assert.equal(guard.size, 0, "the record outlived its bound");

    // The same document arriving after the window is refused on freshness,
    // never reaching the (now empty) record.
    const late = await run(doc(), Date.parse("2026-08-26T12:10:00Z"));
    assert.equal(late.kind, "rejected");
    if (late.kind !== "rejected") return;
    assert.equal(late.error.payload.code, "expired");
    assert.equal(state.executions, 1);
  });

  it("does not burn the id of a document refused before the claim", async () => {
    const guard = new InMemoryReplayGuard(64);
    const { run, state } = runner(guard);

    const misaddressed = await run(doc({ recipient: "did:web:someone-else.example" }));
    assert.equal(misaddressed.kind, "rejected");
    assert.equal(guard.size, 0, "a refused document claimed the id");

    assert.equal((await run(doc())).kind, "handled");
    assert.equal(state.executions, 1);
  });

  it("refuses a document it cannot place in any window rather than executing", async () => {
    const guard = new InMemoryReplayGuard(64);
    const { run, state } = runner(guard);

    // No `issuedAt` and no `expiresAt`: §7.2 (*Bounding the record*) forbids
    // executing a consequential task on it — there is nowhere to keep a record.
    const outcome = await run(doc({ issuedAt: undefined }));
    assert.equal(outcome.kind, "rejected");
    if (outcome.kind !== "rejected") return;
    // `consequentialChecks` requires `issuedAt`, so this is caught as
    // malformed before the unboundable path — either way it does not execute.
    assert.equal(outcome.error.payload.code, "malformedRequest");
    assert.equal(state.executions, 0);
  });

  it("fails closed when the record cannot be consulted", async () => {
    const broken: ReplayGuard = {
      claim() {
        throw new Error("redis://replay-1.internal:6379: connection refused");
      },
    };
    const { run, state } = runner(broken);

    const outcome = await run(doc());
    assert.equal(outcome.kind, "rejected");
    if (outcome.kind !== "rejected") return;
    assert.equal(outcome.error.payload.code, "unavailable");
    assert.equal(outcome.error.payload.retryable, true, "the resend must be invited");
    assert.ok(
      !(outcome.error.payload.message ?? "").includes("redis"),
      "store detail reached the wire",
    );
    assert.equal(state.executions, 0, "executed without a usable record");
  });

  it("keeps no record when the caller declares the task not consequential", async () => {
    let executions = 0;
    const run = (): Promise<ConsumeOutcome<Response>> =>
      consumeInbound<Payload, Response>({
        transport: new StaticTransport({ issuer: PEER, recipient: ME }),
        spec: SPEC,
        proofPolicy: { kind: "acceptUnverified" },
        payloadPolicy: { kind: "acceptUnvalidated" },
        checks: notConsequentialChecks(),
        doc: doc(),
        myVid: ME,
        now: NOW,
        newErrorId: () => "err-1",
        handler: () => {
          executions += 1;
          return undefined;
        },
      });

    await run();
    await run();
    // Explicitly opting out is a decision the caller is allowed to make — §7.2
    // item 11 applies to *consequential* tasks. The point of the test is that
    // it is opt-out, not default.
    assert.equal(executions, 2);
  });

  it("requires the checks option rather than defaulting to no record", async () => {
    await assert.rejects(
      () =>
        consumeInbound<Payload, Response>({
          transport: new StaticTransport({ issuer: PEER, recipient: ME }),
          spec: SPEC,
          proofPolicy: { kind: "acceptUnverified" },
          payloadPolicy: { kind: "acceptUnvalidated" },
          doc: doc(),
          myVid: ME,
          now: NOW,
          newErrorId: () => "err-1",
          handler: () => undefined,
        } as never),
      /`checks` is required/,
    );
  });
});

describe("InMemoryReplayGuard", () => {
  const D = "digest-a";

  it("evicts the least recently used record at capacity", () => {
    const guard = new InMemoryReplayGuard(2);
    guard.claim("a", "da", undefined, NOW);
    guard.claim("b", "db", undefined, NOW);
    guard.claim("a", "da", undefined, NOW); // touch `a`
    guard.claim("c", "dc", undefined, NOW);

    assert.equal(guard.size, 2);
    assert.equal(guard.claim("a", "da", undefined, NOW).kind, "duplicate");
    assert.equal(guard.claim("b", "db", undefined, NOW).kind, "fresh");
  });

  it("does not let a conflicting document displace the record it conflicts with", () => {
    const guard = new InMemoryReplayGuard(8);
    guard.claim("req-1", D, undefined, NOW);
    assert.equal(guard.claim("req-1", "other", undefined, NOW).kind, "conflict");
    // An attacker who could evict the original by sending a variant would then
    // replay the original successfully.
    assert.equal(guard.claim("req-1", D, undefined, NOW).kind, "duplicate");
  });

  it("treats a record past its retention deadline as absent", () => {
    const guard = new InMemoryReplayGuard(8);
    const expiry = Date.parse("2026-08-26T12:05:00Z");
    assert.equal(guard.claim("req-1", D, expiry, NOW).kind, "fresh");
    assert.equal(
      guard.claim("req-1", D, expiry, Date.parse("2026-08-26T12:04:59Z")).kind,
      "duplicate",
    );
    assert.equal(guard.claim("req-1", D, expiry, expiry).kind, "fresh");
  });

  it("releases an unfinished claim but not a completed one", () => {
    const guard = new InMemoryReplayGuard(8);
    guard.claim("req-1", D, undefined, NOW);
    guard.release("req-1", D);
    assert.equal(guard.claim("req-1", D, undefined, NOW).kind, "fresh");

    guard.recordResponse("req-1", { ok: true });
    guard.release("req-1", D);
    assert.equal(
      guard.claim("req-1", D, undefined, NOW).kind,
      "duplicate",
      "a completed execution's record must survive a stray release",
    );
  });

  it("rejects a capacity that would retain nothing", () => {
    // A guard that retains nothing answers `fresh` to everything, which is a
    // silent total defeat of item 11 rather than a visible misconfiguration.
    assert.throws(() => new InMemoryReplayGuard(0), RangeError);
  });
});

describe("SPEC §4.2 / §7.2 — freshness bounds", () => {
  const skew = { skewMs: 60_000 };

  it("rejects an issuedAt beyond the skew tolerance, and accepts one inside it", () => {
    assert.equal(validateFreshness(doc({ issuedAt: "2026-08-26T12:00:30Z" }), NOW, skew), null);

    const bad = validateFreshness(doc({ issuedAt: "2026-08-26T12:05:00Z" }), NOW, skew);
    assert.equal(bad?.code, "malformedRequest");
    assert.equal(bad?.message, FUTURE_ISSUED_AT);
  });

  it("does not name the consumer's clock in the future-dated message", () => {
    // Echoing the delta would be a remote `ntpdate` for an unauthenticated
    // sender (SPEC §10.4).
    const msg = validateFreshness(doc({ issuedAt: "2031-01-01T00:00:00Z" }), NOW, skew)?.message;
    assert.ok(!msg?.includes("2026"), `wire message leaked the clock: ${msg}`);
    assert.ok(!msg?.includes("2031"), `wire message echoed the input: ${msg}`);
  });

  it("rejects an expiresAt at or before issuedAt", () => {
    for (const expiresAt of ["2026-08-26T11:59:00Z", "2026-08-26T11:59:30Z"]) {
      const d = doc({ issuedAt: "2026-08-26T11:59:30Z", expiresAt });
      const err = validateFreshness(d, NOW, skew);
      assert.equal(err?.code, "malformedRequest");
      assert.equal(err?.message, EXPIRY_NOT_AFTER_ISSUANCE);
    }
    const ok = doc({ issuedAt: "2026-08-26T11:59:30Z", expiresAt: "2026-08-26T12:30:00Z" });
    assert.equal(validateFreshness(ok, NOW, skew), null);
  });

  it("bounds the acceptance window with maxAgeMs", () => {
    const policy = { skewMs: 60_000, maxAgeMs: 5 * 60_000 };
    assert.equal(validateFreshness(doc({ issuedAt: "2026-08-26T11:58:00Z" }), NOW, policy), null);

    const stale = validateFreshness(doc({ issuedAt: "2026-08-26T11:30:00Z" }), NOW, policy);
    assert.equal(stale?.code, "expired");
    assert.equal(stale?.message, STALE_WIRE_MESSAGE);
  });

  it("refuses a document with no timestamp at all once a window is configured", () => {
    const bare = doc({ issuedAt: undefined });
    // No window: the pre-existing behaviour is preserved.
    assert.equal(validateFreshness(bare, NOW, skew), null);

    const windowed = { skewMs: 60_000, maxAgeMs: 5 * 60_000 };
    assert.equal(validateFreshness(bare, NOW, windowed)?.code, "expired");

    // …unless the producer supplied an `expiresAt`, which is a window of its own.
    const bounded = doc({ issuedAt: undefined, expiresAt: "2026-08-26T12:30:00Z" });
    assert.equal(validateFreshness(bounded, NOW, windowed), null);
  });
});

describe("canonicalization and digest", () => {
  it("matches the published SHA-256 vectors", () => {
    // A hand-rolled hash is only worth having if it is pinned to the
    // standard's own answers (FIPS 180-4 / NIST CAVP).
    assert.equal(
      sha256Hex(""),
      "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    );
    assert.equal(
      sha256Hex("abc"),
      "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    );
    assert.equal(
      sha256Hex("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
      "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
    );
    assert.equal(
      sha256Hex("a".repeat(1_000_000)),
      "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0",
    );
  });

  it("orders members and ignores the whitespace of the received bytes", () => {
    // A consumer keyed on received octets would answer a legitimate §8.4 retry
    // — re-indented in transit, or re-serialized by an intermediary — with
    // `idConflict`.
    const a = JSON.parse('{ "b" : 1 , "a": { "z": 1, "y": 2 } }') as unknown;
    const b = JSON.parse('{"a":{"y":2,"z":1},"b":1}') as unknown;
    assert.equal(canonicalJson(a), canonicalJson(b));
    assert.equal(canonicalJson(a), '{"a":{"y":2,"z":1},"b":1}');
  });

  it("computes the same digest for the same document regardless of member order", () => {
    const a = doc();
    const b = { payload: a.payload, type: a.type, recipient: ME, issuer: PEER, id: "req-1", issuedAt: a.issuedAt };
    assert.equal(documentDigest(a), documentDigest(b as TrustTaskDocument<Payload>));
  });
});

describe("SPEC §10.4 — error messages carry no consumer internals", () => {
  it("reports proofInvalid with a constant, not the verifier's text", async () => {
    const outcome = await consumeInbound<Payload, Response>({
      transport: new StaticTransport({ issuer: PEER, recipient: ME }),
      spec: SPEC,
      proofPolicy: {
        kind: "verify",
        verify: {
          verify() {
            throw new Error(
              "resolve did:web:org.example: connection refused; " +
                "verificationMethod did:web:org.example#key-1 not present in DID document",
            );
          },
        },
      },
      payloadPolicy: { kind: "acceptUnvalidated" },
      checks: notConsequentialChecks(),
      doc: doc({
        proof: {
          type: "DataIntegrityProof",
          cryptosuite: "eddsa-jcs-2022",
          verificationMethod: `${PEER}#key-1`,
          created: "2026-08-26T12:00:00Z",
          proofPurpose: "assertionMethod",
          proofValue: "zAAA",
        },
      }),
      myVid: ME,
      now: NOW,
      newErrorId: () => "err-1",
      handler: () => undefined,
    });

    assert.equal(outcome.kind, "rejected");
    if (outcome.kind !== "rejected") return;
    assert.equal(outcome.error.payload.code, "proofInvalid");
    const msg = outcome.error.payload.message ?? "";
    assert.equal(msg, PROOF_INVALID_WIRE_MESSAGE);
    for (const leak of ["resolve", "did:web:", "DID document", "connection refused"]) {
      assert.ok(!msg.includes(leak), `verifier detail on the wire: ${msg}`);
    }
  });
});
