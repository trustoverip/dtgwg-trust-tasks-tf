import { test } from "node:test";
import assert from "node:assert/strict";
import type { TrustTaskDocument, ResolvedParties, ErrorResponse } from "@openvtc/trust-tasks";
import { respondWith } from "@openvtc/trust-tasks";
import { ed25519, x25519 } from "@noble/curves/ed25519.js";
import {
  packTrustTask,
  unpackTrustTask,
  advertisedSender,
  ENVELOPE_TYPE,
  TspEnvelopeError,
  TspConsumer,
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
