import { test } from "node:test";
import assert from "node:assert/strict";
import type { TrustTaskDocument, ResolvedParties, ErrorResponse } from "@openvtc/trust-tasks/runtime";
import { InMemoryReplayGuard, respondWith } from "@openvtc/trust-tasks/runtime";
import { ed25519, x25519 } from "@noble/curves/ed25519.js";
import {
  packTrustTask,
  unpackTrustTask,
  advertisedSender,
  ENVELOPE_TYPE,
  TspEnvelopeError,
  TspConsumer,
  DOCUMENT_TOO_LARGE_WIRE_MESSAGE,
  type SenderPublicKeys,
  type PackKeys,
  type UnpackKeys,
} from "../src/index.js";

interface Party {
  vid: string;
  sigPriv: Uint8Array;
  sigPub: Uint8Array;
  encPriv: Uint8Array;
  encPub: Uint8Array;
}

function party(vid: string, seed: number): Party {
  const sigPriv = new Uint8Array(32).fill(seed);
  const encPriv = new Uint8Array(32).fill(seed + 100);
  return {
    vid,
    sigPriv,
    sigPub: ed25519.getPublicKey(sigPriv),
    encPriv,
    encPub: x25519.getPublicKey(encPriv),
  };
}

function packKeys(sender: Party, receiver: Party): PackKeys {
  return {
    senderSigningKey: sender.sigPriv,
    senderEncryptionKey: sender.encPriv,
    receiverEncryptionKey: receiver.encPub,
  };
}
function unpackKeys(recipient: Party, sender: Party): UnpackKeys {
  return {
    receiverDecryptionKey: recipient.encPriv,
    senderEncryptionKey: sender.encPub,
    senderSigningKey: sender.sigPub,
  };
}
function senderKeys(p: Party): SenderPublicKeys {
  return { encryptionKey: p.encPub, signingKey: p.sigPub };
}

const echoType = "https://trusttasks.org/spec/example/echo/0.1";
const echoSpec = { typeUri: echoType } as unknown as Parameters<TspConsumer["consume"]>[1];
const fixedIssuedAt = "2026-06-01T12:00:00Z";
const now = () => Date.parse(fixedIssuedAt);

interface Echo {
  text: string;
}

function echoDoc(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    id: "urn:uuid:00000000-0000-4000-8000-000000000001",
    type: echoType,
    issuedAt: fixedIssuedAt,
    payload: { text: "hello" },
    ...overrides,
  };
}

test("round trip: the authenticated sender and recipient", async () => {
  const alice = party("did:example:alice", 1);
  const bob = party("did:example:bob", 2);
  const doc = echoDoc();
  const wire = await packTrustTask(doc, { senderVid: alice.vid, receiverVid: bob.vid, keys: packKeys(alice, bob) });
  assert.equal(advertisedSender(wire), alice.vid);
  const unpacked = await unpackTrustTask(wire, { recipientVid: bob.vid, keys: unpackKeys(bob, alice) });
  assert.deepEqual(unpacked.document, doc);
  assert.equal(unpacked.transport.peer, alice.vid);
  assert.equal(unpacked.transport.local, bob.vid);
});

test("the sealed inner payload is exactly the {type, document} envelope", async () => {
  const alice = party("did:example:alice", 1);
  const bob = party("did:example:bob", 2);
  const doc = echoDoc();
  const wire = await packTrustTask(doc, { senderVid: alice.vid, receiverVid: bob.vid, keys: packKeys(alice, bob) });
  // Not confidential to a non-recipient, but structurally: re-decode via unpack.
  const unpacked = await unpackTrustTask(wire, { recipientVid: bob.vid, keys: unpackKeys(bob, alice) });
  assert.deepEqual(Object.keys(unpacked.document).sort(), Object.keys(doc).sort());
});

test("plaintext does not leak into the sealed message", async () => {
  const alice = party("did:example:alice", 1);
  const bob = party("did:example:bob", 2);
  const wire = await packTrustTask(echoDoc({ payload: { text: "top-secret" } }), {
    senderVid: alice.vid,
    receiverVid: bob.vid,
    keys: packKeys(alice, bob),
  });
  assert.equal(new TextDecoder().decode(wire).includes("top-secret"), false);
});

test("a message sealed to bob does not open for carol", async () => {
  const alice = party("did:example:alice", 1);
  const bob = party("did:example:bob", 2);
  const carol = party("did:example:carol", 3);
  const wire = await packTrustTask(echoDoc(), { senderVid: alice.vid, receiverVid: bob.vid, keys: packKeys(alice, bob) });
  await assert.rejects(
    unpackTrustTask(wire, { recipientVid: carol.vid, keys: unpackKeys(carol, alice) }),
    (e: unknown) => e instanceof TspEnvelopeError && e.failure === "notForThisReceiver",
  );
});

test("a message sealed to bob's key but naming another receiver VID is refused", async () => {
  const alice = party("did:example:alice", 1);
  const bob = party("did:example:bob", 2);
  const other = "did:example:someone-else";
  const wire = await packTrustTask(echoDoc({ recipient: other }), {
    senderVid: alice.vid,
    receiverVid: other,
    keys: packKeys(alice, bob),
  });
  await assert.rejects(
    unpackTrustTask(wire, { recipientVid: bob.vid, keys: unpackKeys(bob, alice) }),
    (e: unknown) =>
      e instanceof TspEnvelopeError && e.failure === "notForThisReceiver" && e.sender === alice.vid,
  );
});

test("the transport recipient is the configured VID", async () => {
  const alice = party("did:example:alice", 1);
  const bob = party("did:example:bob", 2);
  const wire = await packTrustTask(echoDoc(), { senderVid: alice.vid, receiverVid: bob.vid, keys: packKeys(alice, bob) });
  const { transport } = await unpackTrustTask(wire, { recipientVid: bob.vid, keys: unpackKeys(bob, alice) });
  assert.equal(transport.local, bob.vid);
  assert.equal(transport.peer, alice.vid);
});

test("a tampered message does not open", async () => {
  const alice = party("did:example:alice", 1);
  const bob = party("did:example:bob", 2);
  const wire = await packTrustTask(echoDoc(), { senderVid: alice.vid, receiverVid: bob.vid, keys: packKeys(alice, bob) });
  wire[wire.length - 1] ^= 0xff;
  await assert.rejects(
    unpackTrustTask(wire, { recipientVid: bob.vid, keys: unpackKeys(bob, alice) }),
    (e: unknown) => e instanceof TspEnvelopeError,
  );
});

// ── Consumer ────────────────────────────────────────────────────────────────

function harness() {
  const alice = party("did:example:alice", 1);
  const bob = party("did:example:bob", 2);
  const calls: string[] = [];
  const consumer = new TspConsumer({
    recipientVid: bob.vid,
    decryptionKey: bob.encPriv,
    signingKey: bob.sigPriv,
    resolveSender: (vid) => {
      if (vid !== alice.vid) throw new Error(`unknown ${vid}`);
      return senderKeys(alice);
    },
    now,
  });
  const seal = (doc: Record<string, unknown>) =>
    packTrustTask(doc, { senderVid: alice.vid, receiverVid: bob.vid, keys: packKeys(alice, bob) });
  const handler = (d: TrustTaskDocument<Echo>, _p: ResolvedParties): TrustTaskDocument<Echo> => {
    calls.push(d.payload.text);
    return respondWith<Echo, Echo>(d, "urn:uuid:resp", { text: d.payload.text.toUpperCase() });
  };
  const receive = (wire: Uint8Array) => consumer.receive<Echo, Echo>(wire, echoSpec, handler);
  return { alice, bob, calls, consumer, seal, receive };
}

function codeOf(r: { reply?: Uint8Array }): string | undefined {
  if (!r.reply) return undefined;
  return (JSON.parse(new TextDecoder().decode(r.reply)) as ErrorResponse).payload.code;
}

test("consumer: in-band identity may be omitted; TSP supplies it", async () => {
  const h = harness();
  const r = await h.receive(await h.seal(echoDoc()));
  assert.equal(r.outcome.kind, "handled");
  assert.equal(r.transport.peer, h.alice.vid);
  assert.equal((r.outcome as { response: TrustTaskDocument<Echo> }).response.payload.text, "HELLO");
});

test("consumer: the reply seals back to alice, who opens it", async () => {
  const h = harness();
  const r = await h.receive(await h.seal(echoDoc()));
  const wire = await h.consumer.packReply(r);
  const back = await unpackTrustTask(wire, { recipientVid: h.alice.vid, keys: unpackKeys(h.alice, h.bob) });
  assert.equal(back.transport.peer, h.bob.vid);
  assert.equal(back.document.type, `${echoType}#response`);
});

test("consumer: a re-forward is absorbed with the first response", async () => {
  const h = harness();
  const first = await h.receive(await h.seal(echoDoc()));
  const again = await h.receive(await h.seal(echoDoc()));
  assert.equal(again.outcome.kind, "duplicate");
  assert.deepEqual(again.reply, first.reply);
  assert.deepEqual(h.calls, ["hello"]);
});

test("consumer: a different document under the same id is idConflict", async () => {
  const h = harness();
  await h.receive(await h.seal(echoDoc()));
  const r = await h.receive(await h.seal(echoDoc({ payload: { text: "changed" } })));
  assert.equal(codeOf(r), "idConflict");
  assert.deepEqual(h.calls, ["hello"]);
});

test("consumer: an in-band issuer other than the sender is identityMismatch", async () => {
  const h = harness();
  const r = await h.receive(await h.seal(echoDoc({ issuer: "did:web:someone-else.example" })));
  assert.equal(codeOf(r), "identityMismatch");
  const reply = JSON.parse(new TextDecoder().decode(r.reply!)) as Record<string, unknown>;
  assert.equal(reply.recipient, h.alice.vid);
  assert.deepEqual(h.calls, []);
});

test("consumer: a sender off the allowlist is refused before resolution", async () => {
  const h = harness();
  const strict = new TspConsumer({
    recipientVid: h.bob.vid,
    decryptionKey: h.bob.encPriv,
    signingKey: h.bob.sigPriv,
    resolveSender: () => {
      throw new Error("should not be called");
    },
    allowedSenders: new Set(["did:example:nobody"]),
    now,
  });
  await assert.rejects(
    strict.receive<Echo, Echo>(await h.seal(echoDoc()), echoSpec, () => undefined),
    (e: unknown) => e instanceof TspEnvelopeError && e.failure === "notForThisReceiver",
  );
});

test("consumer: a stale document is refused as expired", async () => {
  const h = harness();
  const r = await h.receive(await h.seal(echoDoc({ issuedAt: "2026-05-01T00:00:00Z" })));
  assert.equal(codeOf(r), "expired");
});

test("consumer: a proof with no verifier configured is refused", async () => {
  const h = harness();
  const doc = echoDoc({
    issuer: h.alice.vid,
    recipient: h.bob.vid,
    proof: {
      type: "DataIntegrityProof",
      cryptosuite: "eddsa-jcs-2022",
      verificationMethod: `${h.alice.vid}#k`,
      proofPurpose: "assertionMethod",
      created: fixedIssuedAt,
      proofValue: "z1",
    },
  });
  const r = await h.receive(await h.seal(doc));
  assert.equal(codeOf(r), "malformedRequest");
});

test("consumer: a fire-and-forget success has nothing to reply with", async () => {
  const h = harness();
  const r = await h.consumer.receive<Echo, void>(await h.seal(echoDoc()), echoSpec, () => undefined);
  assert.equal(r.outcome.kind, "accepted");
  assert.equal(r.reply, undefined);
  await assert.rejects(h.consumer.packReply(r));
});

// SPEC §5.2 (#1117): a consumer of rooms/records/put/0.4 reads a 0.5 document —
// which declares extends: "0.4" in the registry — as 0.4, its payload pruned to
// what 0.4's schema knows, and answers it as 0.5. An undeclared later version
// of the same slug is unsupportedVersion; another slug is unsupportedType.
test("consumer: a later version declaring extends is read forward", async () => {
  const base = "https://trusttasks.org/spec/rooms/records/put/";
  const spec = {
    typeUri: `${base}0.4`,
    isBearer: false,
    isProofRequired: false,
    isRecipientRequired: false,
    payloadSchema: { type: "object", additionalProperties: false, properties: { text: { type: "string" } } },
  };
  const run = async (type: string) => {
    const h = harness();
    let seen: Record<string, unknown> | undefined;
    const r = await h.consumer.receive<Record<string, unknown>, Echo>(
      await h.seal(echoDoc({ type, payload: { text: "hello", extra: "0.5 only" } })),
      spec,
      (d) => {
        seen = d.payload;
        return respondWith<Record<string, unknown>, Echo>(d, "urn:uuid:resp", { text: "ok" });
      },
    );
    return { r, seen };
  };

  const forward = await run(`${base}0.5`);
  assert.equal(forward.r.outcome.kind, "handled");
  assert.deepEqual(forward.seen, { text: "hello" });
  assert.equal((forward.r.outcome as { response: TrustTaskDocument<Echo> }).response.type, `${base}0.5#response`);

  assert.equal(codeOf((await run(`${base}0.99`)).r), "unsupportedVersion");
  assert.equal(codeOf((await run("https://trusttasks.org/spec/rooms/records/get/0.4")).r), "unsupportedType");
});

// SPEC §12.4 (#1121): the declared maxDocumentBytes is held against the
// received document — refused malformedRequest to the authenticated sender,
// before the item-11 record or the handler; within it, never refused for size.
test("consumer: a document over its declared maxDocumentBytes is refused before the record", async () => {
  const doc = echoDoc();
  const size = new TextEncoder().encode(JSON.stringify(doc)).length;
  const run = async (bound: number) => {
    const h = harness();
    const guard = new InMemoryReplayGuard();
    const consumer = new TspConsumer({
      recipientVid: h.bob.vid,
      decryptionKey: h.bob.encPriv,
      signingKey: h.bob.sigPriv,
      resolveSender: () => senderKeys(h.alice),
      guard,
      now,
    });
    const calls: string[] = [];
    const r = await consumer.receive<Echo, Echo>(
      await h.seal(doc),
      { ...echoSpec, maxDocumentBytes: bound } as typeof echoSpec,
      (d) => {
        calls.push(d.payload.text);
        return respondWith<Echo, Echo>(d, "urn:uuid:resp", { text: "ok" });
      },
    );
    return { r, calls, guard, h };
  };

  const over = await run(size - 1);
  assert.equal(over.r.outcome.kind, "rejected");
  assert.equal(codeOf(over.r), "malformedRequest");
  const error = JSON.parse(new TextDecoder().decode(over.r.reply!)) as ErrorResponse;
  assert.ok(error.recipient === undefined || error.recipient === over.h.alice.vid);
  // Sealed back to the TSP-authenticated sender, who opens it.
  const sealed = await over.h.consumer.packReply(over.r);
  const back = await unpackTrustTask(sealed, { recipientVid: over.h.alice.vid, keys: unpackKeys(over.h.alice, over.h.bob) });
  assert.equal(back.document.type, (error as unknown as { type: string }).type);
  assert.equal(error.payload.message, DOCUMENT_TOO_LARGE_WIRE_MESSAGE);
  assert.deepEqual(over.calls, []);
  assert.equal(over.guard.size, 0, "a refused document writes no record");

  const at = await run(size);
  assert.equal(at.r.outcome.kind, "handled");
  assert.deepEqual(at.calls, ["hello"]);
  assert.equal(at.guard.size, 1);
});

test("consumer: a document read forward is not held to the implemented version's bound", async () => {
  const base = "https://trusttasks.org/spec/rooms/records/put/";
  const spec = {
    typeUri: `${base}0.4`,
    isBearer: false,
    isProofRequired: false,
    isRecipientRequired: false,
    payloadSchema: { type: "object", properties: { text: { type: "string" } } },
    maxDocumentBytes: 16,
  } as unknown as typeof echoSpec;
  const respond = (d: TrustTaskDocument<Echo>) => respondWith<Echo, Echo>(d, "urn:uuid:resp", { text: "ok" });

  const h = harness();
  const forward = await h.consumer.receive<Echo, Echo>(await h.seal(echoDoc({ type: `${base}0.5` })), spec, respond);
  assert.equal(forward.outcome.kind, "handled");

  // The same bound refuses a document of the implemented version itself.
  const h2 = harness();
  const same = await h2.consumer.receive<Echo, Echo>(await h2.seal(echoDoc({ type: `${base}0.4` })), spec, respond);
  assert.equal(codeOf(same), "malformedRequest");
});
