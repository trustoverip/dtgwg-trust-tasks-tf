/**
 * SPEC.md §7.2 conformance tests for the TypeScript consumer pipeline.
 *
 * These deliberately mirror the test set in `trust-tasks-rs/src/consume.rs`.
 * The two reference implementations must reach the same verdict on the same
 * document; where a case exists there and not here, the languages can drift
 * apart without anything noticing.
 */

import assert from "node:assert/strict";
import { describe, it } from "node:test";

import {
  consumeInbound,
  refuse,
  rejectWithRecipient,
  respondWith,
  StaticTransport,
  UnauthenticatedTransport,
  type ConsumeOutcome,
  type ProofPolicy,
  type ProofVerifier,
  type SpecPolicy,
  type TransportContext,
  type TransportHandler,
  type TrustTaskDocument,
} from "../src/_runtime/index.js";

const ME = "did:web:maintainer.example";
const PEER = "did:web:org.example";

/** Stands in for a generated module's `SPEC`. */
const REQUIRED_SPEC: SpecPolicy = {
  typeUri: "https://trusttasks.org/spec/acl/grant/0.1",
  isBearer: false,
  isProofRequired: true,
  isRecipientRequired: true,
};

/** A spec that only RECOMMENDS a proof and does not require a recipient. */
const RELAXED_SPEC: SpecPolicy = {
  typeUri: "https://trusttasks.org/spec/acl/list/0.1",
  isBearer: false,
  isProofRequired: false,
  isRecipientRequired: false,
};

interface Payload {
  role: string;
}
interface Response {
  ok: true;
}

function doc(over: Partial<TrustTaskDocument<Payload>> = {}): TrustTaskDocument<Payload> {
  return {
    id: "req-1",
    type: REQUIRED_SPEC.typeUri,
    issuer: PEER,
    recipient: ME,
    payload: { role: "admin" },
    ...over,
  };
}

const PROOF = {
  type: "DataIntegrityProof",
  cryptosuite: "eddsa-rdfc-2022",
  verificationMethod: `${PEER}#key-1`,
  created: "2026-01-01T00:00:00Z",
  proofPurpose: "assertionMethod",
  proofValue: "z3kg",
};

const alwaysValid: ProofVerifier = { verify: () => true };
const alwaysInvalid: ProofVerifier = { verify: () => false };

const CLOCK = () => "2026-01-01T00:00:00.000Z";

function run(
  over: Partial<TrustTaskDocument<Payload>>,
  opts: {
    spec?: SpecPolicy;
    proofPolicy?: ProofPolicy;
    transport?: TransportHandler;
    handlerShouldNotRun?: boolean;
  } = {},
): Promise<ConsumeOutcome<Response>> {
  return consumeInbound<Payload, Response>({
    transport: opts.transport ?? new UnauthenticatedTransport(),
    spec: opts.spec ?? REQUIRED_SPEC,
    proofPolicy: opts.proofPolicy ?? { kind: "verify", verify: alwaysValid },
    doc: doc(over),
    myVid: ME,
    now: Date.parse("2026-01-01T00:00:00Z"),
    newErrorId: () => "err-1",
    clock: CLOCK,
    handler: (accepted) => {
      if (opts.handlerShouldNotRun) assert.fail("handler must not run");
      return respondWith<Payload, Response>(accepted, "resp-1", { ok: true }, CLOCK);
    },
  });
}

function rejectedCode(outcome: ConsumeOutcome<Response>): string {
  assert.equal(outcome.kind, "rejected", `expected rejected, got ${outcome.kind}`);
  assert.ok(outcome.kind === "rejected");
  return outcome.error.payload.code;
}

describe("§7.2 pipeline", () => {
  it("runs the handler when every check passes", async () => {
    const outcome = await run({ proof: PROOF });
    assert.equal(outcome.kind, "handled");
    assert.ok(outcome.kind === "handled");
    assert.equal(outcome.response.id, "resp-1");
    // §4.4.1 — the response carries the #response fragment...
    assert.equal(outcome.response.type, `${REQUIRED_SPEC.typeUri}#response`);
    // ...the parties swap...
    assert.equal(outcome.response.issuer, ME);
    assert.equal(outcome.response.recipient, PEER);
    // ...and §4.9 continues the thread from the request's id.
    assert.equal(outcome.response.threadId, "req-1");
  });

  it("item 4 — rejects an expired document", async () => {
    const outcome = await run({ proof: PROOF, expiresAt: "2025-01-01T00:00:00Z" }, {
      handlerShouldNotRun: true,
    });
    assert.equal(rejectedCode(outcome), "expired");
  });

  it("item 4 — treats the expiry instant itself as expired (inclusive bound)", async () => {
    const outcome = await run({ proof: PROOF, expiresAt: "2026-01-01T00:00:00Z" }, {
      handlerShouldNotRun: true,
    });
    assert.equal(rejectedCode(outcome), "expired");
  });

  it("item 5a — wrong recipient, and the error routes to the original issuer", async () => {
    const outcome = await run({ recipient: "did:web:someone-else.example", proof: PROOF }, {
      handlerShouldNotRun: true,
    });
    assert.equal(rejectedCode(outcome), "wrongRecipient");
    assert.ok(outcome.kind === "rejected");
    assert.equal(outcome.error.recipient, PEER);
  });

  it("item 5b — recipient REQUIRED but absent in-band", async () => {
    // Fires before the proof check, so it wins even though this spec also
    // requires a proof.
    const outcome = await run({ recipient: undefined }, { handlerShouldNotRun: true });
    assert.equal(rejectedCode(outcome), "malformedRequest");
  });

  it("item 7 — proof REQUIRED by the spec but absent", async () => {
    const outcome = await run({}, { handlerShouldNotRun: true });
    assert.equal(rejectedCode(outcome), "proofRequired");
  });

  it("item 7 — a RECOMMENDED spec accepts a proofless document under verify", async () => {
    // Locks in the per-spec discrimination: a regression here would make every
    // spec behave as though proof were REQUIRED.
    const outcome = await run({ proof: undefined }, { spec: RELAXED_SPEC });
    assert.equal(outcome.kind, "handled");
  });

  it("item 7 — a failing verifier maps to proofInvalid", async () => {
    const outcome = await run(
      { proof: PROOF },
      { proofPolicy: { kind: "verify", verify: alwaysInvalid }, handlerShouldNotRun: true },
    );
    assert.equal(rejectedCode(outcome), "proofInvalid");
  });

  it("item 7 — a verifier that throws is a failure, not a crash", async () => {
    const throwing: ProofVerifier = {
      verify: () => {
        throw new Error("resolver unreachable");
      },
    };
    const outcome = await run(
      { proof: PROOF },
      { proofPolicy: { kind: "verify", verify: throwing }, handlerShouldNotRun: true },
    );
    assert.equal(rejectedCode(outcome), "proofInvalid");
  });

  it("item 7 — rejectIfPresent refuses a proof-bearing document without leaking config", async () => {
    // SECURITY: a producer-supplied proof MUST NOT be silently dropped, and the
    // wire message MUST NOT describe the consumer's configuration — that would
    // let an unauthenticated probe fingerprint which endpoints lack a verifier.
    const outcome = await run(
      { proof: PROOF },
      { proofPolicy: { kind: "rejectIfPresent" }, handlerShouldNotRun: true },
    );
    assert.equal(rejectedCode(outcome), "malformedRequest");
    assert.ok(outcome.kind === "rejected");
    const msg = outcome.error.payload.message ?? "";
    assert.match(msg, /policy/);
    assert.match(msg, /§7\.2/);
    assert.doesNotMatch(msg, /verifier/);
    assert.doesNotMatch(msg, /configured/);
  });

  it("item 7 — acceptUnverified passes a proof-bearing document through", async () => {
    const outcome = await run(
      { proof: PROOF },
      { proofPolicy: { kind: "acceptUnverified" }, spec: RELAXED_SPEC },
    );
    assert.equal(outcome.kind, "handled");
  });

  it("item 8 — proof with no in-band recipient on a non-bearer spec is malformed", async () => {
    // §4.8.2 audience binding. Uses the relaxed spec so the recipient-REQUIRED
    // check of item 5b does not pre-empt the one under test.
    const outcome = await run(
      { recipient: undefined, proof: PROOF },
      {
        spec: RELAXED_SPEC,
        proofPolicy: { kind: "acceptUnverified" },
        handlerShouldNotRun: true,
      },
    );
    assert.equal(rejectedCode(outcome), "malformedRequest");
    assert.ok(outcome.kind === "rejected");
    assert.match(outcome.error.payload.message ?? "", /audience binding/);
  });

  it("item 8 — a bearer spec is exempt from audience binding", async () => {
    const bearer: SpecPolicy = { ...RELAXED_SPEC, isBearer: true };
    const outcome = await run(
      { recipient: undefined, proof: PROOF },
      { spec: bearer, proofPolicy: { kind: "acceptUnverified" } },
    );
    assert.equal(outcome.kind, "handled");
  });
});

describe("§4.8.1 party resolution", () => {
  it("fills absent in-band members from the transport", async () => {
    const transport = new StaticTransport({ issuer: PEER, recipient: ME });
    let seen: { issuer?: string; recipient?: string } | undefined;
    const outcome = await consumeInbound<Payload, Response>({
      transport,
      spec: RELAXED_SPEC,
      proofPolicy: { kind: "rejectIfPresent" },
      doc: doc({ issuer: undefined, recipient: ME }),
      myVid: ME,
      now: Date.now(),
      newErrorId: () => "err-1",
      clock: CLOCK,
      handler: (accepted, parties) => {
        seen = parties;
        return respondWith<Payload, Response>(accepted, "resp-1", { ok: true }, CLOCK);
      },
    });
    assert.equal(outcome.kind, "handled");
    assert.equal(seen?.issuer, PEER, "issuer should come from the transport");
    assert.equal(seen?.recipient, ME);
  });

  it("item 6 — in-band and transport disagreeing is an identityMismatch", async () => {
    const transport = new StaticTransport({ issuer: PEER });
    const outcome = await run(
      { issuer: "did:web:attacker.example", proof: PROOF },
      { transport, proofPolicy: { kind: "acceptUnverified" }, handlerShouldNotRun: true },
    );
    assert.equal(rejectedCode(outcome), "identityMismatch");
    assert.ok(outcome.kind === "rejected");
    // §8.1 — addressed to the transport-authenticated sender, NEVER the
    // contested in-band issuer.
    assert.equal(outcome.error.recipient, PEER);
    // §8.1/§10.4 — the message must not echo either identity.
    const msg = outcome.error.payload.message ?? "";
    assert.doesNotMatch(msg, /attacker/);
    assert.doesNotMatch(msg, /org\.example/);
  });

  it("§8.1 — identityMismatch with no transport sender is suppressed, not answered", async () => {
    // An addressed error here would be an oracle: the in-band issuer is the
    // contested identity and the transport authenticated nobody to send to.
    //
    // Reaching this needs a mismatch that survives to item 6. The document's
    // in-band recipient must equal `myVid` — otherwise item 5a rejects it as
    // `wrongRecipient` first, which routes normally and never reaches the
    // suppression path. So the disagreement is on the transport's side: it
    // reports the message as addressed elsewhere, and authenticates no sender.
    const mismatchNoSender: TransportHandler = {
      bindingUri: () => "urn:test:mismatching-no-sender",
      deriveParties: (): TransportContext => ({ recipient: "did:web:other-tenant.example" }),
    };
    const outcome = await run({}, { transport: mismatchNoSender, handlerShouldNotRun: true });
    assert.equal(outcome.kind, "suppressed");
    assert.ok(outcome.kind === "suppressed");
    assert.equal(outcome.reason.code, "identityMismatch");
  });
});

describe("handler refusals", () => {
  it("passes a handler-built error response through verbatim", async () => {
    const outcome = await consumeInbound<Payload, Response>({
      transport: new UnauthenticatedTransport(),
      spec: RELAXED_SPEC,
      proofPolicy: { kind: "rejectIfPresent" },
      doc: doc({ proof: undefined }),
      myVid: ME,
      now: Date.now(),
      newErrorId: () => "err-unused",
      clock: CLOCK,
      handler: (req) =>
        refuse(
          req,
          "err-handler",
          { code: "permissionDenied", message: "not in the maintainer vocabulary", retryable: false },
          CLOCK,
        ),
    });
    assert.equal(outcome.kind, "rejected");
    assert.ok(outcome.kind === "rejected");
    assert.equal(outcome.error.id, "err-handler");
    assert.equal(outcome.error.payload.code, "permissionDenied");
    assert.equal(outcome.error.recipient, PEER);
    assert.equal(outcome.error.issuer, ME);
  });
});

describe("§4.9.2 parentThreadId", () => {
  const PARENT = "urn:uuid:9b1d3f60-52a8-4c17-8e44-1d9c7b05f3ae";

  it("carries the parent onto the success response", () => {
    // The whole inner exchange shares one parent, so a response stays inside
    // the same enclosing exchange.
    const req = doc({ threadId: "inner-1", parentThreadId: PARENT });
    const res = respondWith<Payload, Response>(req, "resp-1", { ok: true }, CLOCK);
    assert.equal(res.parentThreadId, PARENT);
    assert.equal(res.threadId, "inner-1");
  });

  it("carries the parent onto an error response", () => {
    const req = doc({ threadId: "inner-1", parentThreadId: PARENT });
    const err = refuse(req, "err-1", { code: "taskFailed", message: "no", retryable: false }, CLOCK);
    assert.equal(err.parentThreadId, PARENT);
  });

  it("carries the parent through the full pipeline", async () => {
    const outcome = await run({ proof: PROOF, threadId: "inner-1", parentThreadId: PARENT });
    assert.ok(outcome.kind === "handled");
    assert.equal(outcome.response.parentThreadId, PARENT);
  });

  it("omits the member entirely when there is no parent", () => {
    // Not `undefined` — absent. An explicit undefined would serialise to
    // `"parentThreadId": null` under some encoders and imply a null parent.
    const res = respondWith<Payload, Response>(doc({}), "resp-1", { ok: true }, CLOCK);
    assert.ok(!("parentThreadId" in res));
    assert.equal(JSON.stringify(res).includes("parentThreadId"), false);
  });

  it("is not rejected by the pipeline on its own (§4.9.2 has no validation semantics)", async () => {
    // Even a self-referential parent — which a producer MUST NOT emit — is not
    // a consumer-side rejection: consumers MUST NOT reject on parentThreadId
    // alone.
    const outcome = await run({ proof: PROOF, threadId: "same", parentThreadId: "same" });
    assert.equal(outcome.kind, "handled");
  });
});

describe("§8.2 inResponseTo", () => {
  it("names the document the error reports on", () => {
    const err = refuse(doc({}), "err-1", { code: "proofRequired", message: "no proof", retryable: false }, CLOCK);
    assert.deepEqual(err.payload.inResponseTo, {
      typeUri: REQUIRED_SPEC.typeUri,
      id: "req-1",
    });
  });

  it("withholds the originating id under identityMismatch", () => {
    // §8.1 — the response goes to the transport-authenticated sender, not the
    // in-band issuer, and that party did not necessarily compose the document.
    // The typeUri stays: it is not identifying.
    const err = refuse(
      doc({}),
      "err-1",
      { code: "identityMismatch", message: "mismatch", retryable: false },
      CLOCK,
    );
    assert.equal(err.payload.inResponseTo?.typeUri, REQUIRED_SPEC.typeUri);
    assert.equal(err.payload.inResponseTo?.id, undefined);
  });

  it("keeps a caller-supplied value", () => {
    // The builder fills a gap; it does not overwrite a deliberate choice.
    const req = doc({});
    const err = rejectWithRecipient(
      req,
      "err-1",
      { code: "taskFailed", retryable: false, inResponseTo: { typeUri: "https://trusttasks.org/spec/acl/revoke/0.1" } },
      req.issuer,
      CLOCK,
    );
    assert.equal(err.payload.inResponseTo?.typeUri, "https://trusttasks.org/spec/acl/revoke/0.1");
    assert.equal(err.payload.inResponseTo?.id, undefined);
  });

  it("emits trust-task-error/0.4, the version whose schema has the member", () => {
    // 0.2's payload schema is additionalProperties:false, so a document
    // carrying inResponseTo would not validate as 0.2.
    const err = refuse(doc({}), "err-1", { code: "taskFailed", message: "failed", retryable: false }, CLOCK);
    assert.equal(err.type, "https://trusttasks.org/spec/trust-task-error/0.4");
  });

  it("carries through the full pipeline on a framework rejection", async () => {
    const outcome = await run({}, { handlerShouldNotRun: true }); // proofRequired
    assert.ok(outcome.kind === "rejected");
    assert.equal(outcome.error.payload.inResponseTo?.typeUri, REQUIRED_SPEC.typeUri);
    assert.equal(outcome.error.payload.inResponseTo?.id, "req-1");
  });
});
