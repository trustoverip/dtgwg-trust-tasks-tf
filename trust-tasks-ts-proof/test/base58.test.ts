import { test } from "node:test";
import assert from "node:assert/strict";
import { randomBytes } from "node:crypto";
import type { TrustTaskDocument } from "@openvtc/trust-tasks/runtime";
import {
  DataIntegrityProofVerifier,
  fromMultibase,
  publicKeyFromDidKey,
  signTrustTask,
  signerFromPrivateKey,
  toMultibase,
} from "../src/index.js";

// The bounds the Go, Rust and Dart proof packages apply too: at most 175 base58
// characters, decoding to at most 128 bytes.
const MAX_CHARS = 175;
const MAX_BYTES = 128;

test("base58: values up to the size bound round-trip", () => {
  const cases: Uint8Array[] = [
    new Uint8Array(0),
    Uint8Array.of(0),
    Uint8Array.of(0, 0, 1),
    new Uint8Array(MAX_BYTES).fill(0xff),
    new Uint8Array(MAX_BYTES),
    Uint8Array.from([0, 0, ...new Array(94).fill(0x80)]),
  ];
  for (let i = 0; i < 200; i++) {
    const b = new Uint8Array(randomBytes(1 + (i % MAX_BYTES)));
    if (i % 3 === 0) b[0] = 0;
    cases.push(b);
  }
  for (const bytes of cases) {
    const mb = toMultibase(bytes);
    assert.ok(mb.length - 1 <= MAX_CHARS, `${bytes.length} bytes encode to ${mb.length - 1} characters`);
    assert.deepEqual(fromMultibase(mb), bytes);
  }
});

test("base58: a value over the length or size bound is refused", () => {
  // One character too long.
  assert.throws(() => fromMultibase("z" + "2".repeat(MAX_CHARS + 1)), /longer than the 175/);
  // Within the length bound, still one byte too large.
  assert.throws(() => fromMultibase("z" + "z".repeat(MAX_CHARS)), /more than 128 bytes/);
  assert.throws(() => fromMultibase("z" + "1".repeat(MAX_BYTES + 1)), /more than 128 bytes/);
  // Long values a did:key never carries are not a key.
  assert.equal(publicKeyFromDidKey("did:key:z" + "2".repeat(MAX_CHARS + 1)), null);
  assert.equal(publicKeyFromDidKey("did:key:z" + "2".repeat(4096)), null);
});

// A megabyte-sized multibase value is refused without decoding it, so what a
// verificationMethod or proofValue costs does not grow with its length.
test("base58: a 1 MiB value is refused quickly", () => {
  const huge = "z" + "2".repeat(1 << 20);
  const start = performance.now();
  for (let i = 0; i < 100; i++) {
    assert.throws(() => fromMultibase(huge));
    assert.equal(publicKeyFromDidKey(`did:key:${huge}#${huge}`), null);
  }
  const elapsed = performance.now() - start;
  assert.ok(elapsed < 1000, `refusing 200 oversized values took ${elapsed.toFixed(0)} ms`);
});

test("verifier: an over-long proofValue or did:key is signatureInvalid, quickly", async () => {
  const now = () => Math.max(Date.parse("2026-10-01T00:00:00Z"), Date.now());
  const verifier = new DataIntegrityProofVerifier({ now });
  const signer = signerFromPrivateKey("ed25519", new Uint8Array(32).fill(3));
  const doc = signTrustTask(
    {
      id: "urn:uuid:9b2c1e34-0000-4000-8000-000000000002",
      type: "https://trusttasks.org/spec/acl/grant/0.1",
      issuer: signer.did,
      recipient: "did:web:maintainer.example",
      issuedAt: "2026-01-01T00:00:00Z",
      payload: { entry: { subject: "did:web:alice.example", role: "admin" } },
    } as TrustTaskDocument<unknown>,
    signer,
  ) as Record<string, unknown>;
  assert.equal((await verifier.verifyDetailed(doc as TrustTaskDocument<unknown>)).valid, true);

  const huge = "z" + "2".repeat(1 << 20);
  const longProof = structuredClone(doc);
  (longProof.proof as Record<string, unknown>).proofValue = huge;
  const longKey = structuredClone(doc);
  longKey.issuer = `did:key:${huge}`;
  (longKey.proof as Record<string, unknown>).verificationMethod = `did:key:${huge}#${huge}`;

  const start = performance.now();
  for (const bad of [longProof, longKey]) {
    const r = await verifier.verifyDetailed(bad as TrustTaskDocument<unknown>);
    assert.equal(r.failure, "signatureInvalid", r.detail ?? "");
  }
  const elapsed = performance.now() - start;
  assert.ok(elapsed < 1000, `refusing two oversized proofs took ${elapsed.toFixed(0)} ms`);
});
