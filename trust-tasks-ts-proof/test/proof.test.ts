import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import type { TrustTaskDocument } from "@openvtc/trust-tasks";
import {
  DataIntegrityProofVerifier,
  signTrustTask,
  signerFromPrivateKey,
  type Curve,
} from "../src/index.js";

const fixture = JSON.parse(
  readFileSync(fileURLToPath(new URL("../../test/eddsa-jcs-2022.fixture.json", import.meta.url)), "utf8"),
) as TrustTaskDocument<unknown>;

// A clock after the fixture's `created` (2026-09) so it is not "in the future".
const now = () => Date.parse("2026-10-01T00:00:00Z");
const verifier = new DataIntegrityProofVerifier({ now });

function seededSigner(curve: Curve, seed: number) {
  const key = new Uint8Array(curve === "p384" ? 48 : 32).fill(seed);
  // Ed25519 and NIST take any 32/48-byte scalar; clamp p-curves into range.
  if (curve !== "ed25519") key[0] = 1;
  return signerFromPrivateKey(curve, key);
}

function grant(issuer: string, role = "admin"): TrustTaskDocument<unknown> {
  return {
    id: "urn:uuid:9b2c1e34-0000-4000-8000-000000000001",
    type: "https://trusttasks.org/spec/acl/grant/0.1",
    issuer,
    recipient: "did:web:maintainer.example",
    issuedAt: "2026-01-01T00:00:00Z",
    payload: { entry: { subject: "did:web:alice.example", role } },
  };
}

test("interop: verifies a document signed by the Rust trust-tasks-proof crate", async () => {
  const r = await verifier.verifyDetailed(fixture);
  assert.equal(r.valid, true, r.detail);
});

test("interop: rejects the Rust document once its payload is changed", async () => {
  const tampered = structuredClone(fixture) as Record<string, unknown>;
  (tampered.payload as Record<string, unknown>).entry = { role: "owner", subject: "did:web:alice.example" };
  const r = await verifier.verifyDetailed(tampered as TrustTaskDocument<unknown>);
  assert.equal(r.failure, "signatureInvalid");
});

for (const curve of ["ed25519", "p256", "p384"] as const) {
  test(`${curve}: sign then verify`, async () => {
    const signer = seededSigner(curve, 1);
    const signed = signTrustTask(grant(signer.did), signer);
    assert.equal(await verifier.verify(signed), true);
    assert.equal(signed.proof!.cryptosuite, curve === "ed25519" ? "eddsa-jcs-2022" : "ecdsa-jcs-2019");
  });

  test(`${curve}: a tampered payload no longer verifies`, async () => {
    const signer = seededSigner(curve, 1);
    const signed = signTrustTask(grant(signer.did), signer) as Record<string, unknown>;
    (signed.payload as Record<string, unknown>).entry = { subject: "did:web:alice.example", role: "owner" };
    const r = await verifier.verifyDetailed(signed as TrustTaskDocument<unknown>);
    assert.equal(r.failure, "signatureInvalid");
  });
}

test("interop: reproduces the Rust eddsa-jcs-2022 signature byte for byte", () => {
  // Ed25519 is deterministic; the fixture was signed with seed [7; 32].
  const signer = signerFromPrivateKey("ed25519", new Uint8Array(32).fill(7));
  const rustProof = fixture.proof as Record<string, unknown>;
  assert.equal(signer.did, fixture.issuer);
  const signed = signTrustTask(fixture, signer, { created: new Date(rustProof.created as string) });
  assert.deepEqual(signed.proof, rustProof);
});

test("issuer binding: a valid signature under a different issuer is issuerMismatch", async () => {
  const attacker = seededSigner("ed25519", 2);
  const victim = seededSigner("ed25519", 3);
  const signed = signTrustTask(grant(attacker.did), attacker) as Record<string, unknown>;
  signed.issuer = victim.did; // claim to be the victim
  const r = await verifier.verifyDetailed(signed as TrustTaskDocument<unknown>);
  assert.equal(r.failure, "issuerMismatch");
});

test("issuer binding: a proof with no issuer is issuerMismatch", async () => {
  const signer = seededSigner("ed25519", 1);
  const signed = signTrustTask(grant(signer.did), signer) as Record<string, unknown>;
  delete signed.issuer;
  const r = await verifier.verifyDetailed(signed as TrustTaskDocument<unknown>);
  assert.equal(r.failure, "issuerMismatch");
});

test("malformed proofs are malformedProof", async () => {
  const signer = seededSigner("ed25519", 1);
  const base = signTrustTask(grant(signer.did), signer);
  const cases: Array<(p: Record<string, unknown>) => void> = [
    (p) => (p.type = "Ed25519Signature2020"),
    (p) => delete p.cryptosuite,
    (p) => delete p.verificationMethod,
    (p) => delete p.proofPurpose,
    (p) => delete p.proofValue,
    (p) => (p.created = "2026-01-01T00:00:00"), // no zone designator
  ];
  for (const mutate of cases) {
    const doc = structuredClone(base) as Record<string, unknown>;
    mutate(doc.proof as Record<string, unknown>);
    const r = await verifier.verifyDetailed(doc as TrustTaskDocument<unknown>);
    assert.equal(r.failure, "malformedProof", JSON.stringify(doc.proof));
  }
});

test("a cryptosuite the verifier is not configured for is unsupportedCryptosuite", async () => {
  const edOnly = new DataIntegrityProofVerifier({ cryptosuites: ["eddsa-jcs-2022"], now });
  const signer = seededSigner("p256", 1);
  const signed = signTrustTask(grant(signer.did), signer);
  const r = await edOnly.verifyDetailed(signed);
  assert.equal(r.failure, "unsupportedCryptosuite");
});

test("created is UTC with a Z and no zero fraction", () => {
  const signer = seededSigner("ed25519", 1);
  const signed = signTrustTask(grant(signer.did), signer, { created: new Date("2026-01-01T00:00:00Z") });
  assert.equal((signed.proof as Record<string, unknown>).created, "2026-01-01T00:00:00Z");
});

test("a future created beyond the skew is refused", async () => {
  const signer = seededSigner("ed25519", 1);
  const clock = () => Date.parse("2026-06-01T12:00:00Z");
  const strict = new DataIntegrityProofVerifier({ now: clock });
  const future = signTrustTask(grant(signer.did), signer, { created: new Date(clock() + 5 * 60_000) });
  const r = await strict.verifyDetailed(future);
  assert.equal(r.failure, "malformedProof");
});

test("signing refuses a document that could never verify", () => {
  const signer = seededSigner("ed25519", 1);
  assert.throws(() => signTrustTask(grant("did:web:someone-else.example"), signer));
  const noIssuer = grant(signer.did) as Record<string, unknown>;
  delete noIssuer.issuer;
  assert.throws(() => signTrustTask(noIssuer as TrustTaskDocument<unknown>, signer));
});
