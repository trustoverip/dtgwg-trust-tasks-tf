import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { createPrivateKey, createPublicKey, sign as nodeSign, verify as nodeVerify } from "node:crypto";
import { p256, p384 } from "@noble/curves/nist.js";
import { sha256, sha384 } from "@noble/hashes/sha2.js";
import type { TrustTaskDocument } from "@openvtc/trust-tasks";
import {
  DataIntegrityProofVerifier,
  canonicalBytes,
  fromMultibase,
  signTrustTask,
  signerFromPrivateKey,
  toMultibase,
  type Curve,
} from "../src/index.js";

const fixture = JSON.parse(
  readFileSync(fileURLToPath(new URL("../../test/eddsa-jcs-2022.fixture.json", import.meta.url)), "utf8"),
) as TrustTaskDocument<unknown>;

// A clock after the fixture's `created` (2026-09) so it is not "in the future".
// It never runs behind the real clock either: the sign-then-verify tests stamp
// `created` with the current time, and a fixed instant would put every fresh
// proof in the future once real time passed it (which it did on 2026-10-01).
const now = () => Math.max(Date.parse("2026-10-01T00:00:00Z"), Date.now());
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

// ecdsa-jcs-2019 against node:crypto, an ECDSA that is not noble: ECDSA with
// the suite hash over hashData, r‖s encoded — what Go's ecdsa.Sign produces.
// The round trips above cannot catch a hashing mistake made on both sides.
const NIST = {
  p256: { noble: p256, hash: "sha256", digest: sha256, crv: "P-256", n: BigInt("0xffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551") },
  p384: { noble: p384, hash: "sha384", digest: sha384, crv: "P-384", n: BigInt("0xffffffffffffffffffffffffffffffffffffffffffffffffc7634d81f4372ddf581a0db248b0a77aecec196accc52973") },
} as const;

function hashDataOf(doc: TrustTaskDocument<unknown>, curve: "p256" | "p384"): Uint8Array {
  const { proof, ...body } = doc as Record<string, unknown>;
  const { proofValue: _pv, ...config } = proof as Record<string, unknown>;
  const { digest } = NIST[curve];
  return new Uint8Array([...digest(canonicalBytes(config)), ...digest(canonicalBytes(body))]);
}

function nodeKeys(curve: "p256" | "p384", seed: number) {
  const d = new Uint8Array(curve === "p384" ? 48 : 32).fill(seed);
  d[0] = 1; // as seededSigner does
  const pub = NIST[curve].noble.getPublicKey(d, false); // 0x04 ‖ x ‖ y
  const half = (pub.length - 1) / 2;
  const b64 = (b: Uint8Array) => Buffer.from(b).toString("base64url");
  const jwk = { kty: "EC", crv: NIST[curve].crv, x: b64(pub.slice(1, 1 + half)), y: b64(pub.slice(1 + half)) };
  return {
    privateKey: createPrivateKey({ key: { ...jwk, d: b64(d) }, format: "jwk" }),
    publicKey: createPublicKey({ key: jwk, format: "jwk" }),
  };
}

/** `sig` in its low-S or high-S form — (r, s) and (r, n − s) are both valid. */
function withS(sig: Uint8Array, n: bigint, high: boolean): Uint8Array {
  const half = sig.length / 2;
  const s = BigInt("0x" + Buffer.from(sig.slice(half)).toString("hex"));
  const isHigh = s > n / 2n;
  const out = (isHigh === high ? s : n - s).toString(16).padStart(half * 2, "0");
  return new Uint8Array([...sig.slice(0, half), ...Buffer.from(out, "hex")]);
}

for (const curve of ["p256", "p384"] as const) {
  test(`${curve}: a signature from signTrustTask verifies as standard ECDSA (node:crypto)`, () => {
    const signer = seededSigner(curve, 1);
    const signed = signTrustTask(grant(signer.did), signer);
    const sig = fromMultibase((signed.proof as Record<string, unknown>).proofValue as string);
    const { publicKey } = nodeKeys(curve, 1);
    const ok = nodeVerify(NIST[curve].hash, hashDataOf(signed, curve), { key: publicKey, dsaEncoding: "ieee-p1363" }, sig);
    assert.equal(ok, true);
  });

  test(`${curve}: a standard ECDSA signature (node:crypto) verifies, in low-S and high-S form`, async () => {
    const signer = seededSigner(curve, 1);
    const doc = signTrustTask(grant(signer.did), signer);
    const { privateKey } = nodeKeys(curve, 1);
    const sig = new Uint8Array(nodeSign(NIST[curve].hash, hashDataOf(doc, curve), { key: privateKey, dsaEncoding: "ieee-p1363" }));
    for (const form of [withS(sig, NIST[curve].n, false), withS(sig, NIST[curve].n, true)]) {
      const resigned = structuredClone(doc) as Record<string, unknown>;
      (resigned.proof as Record<string, unknown>).proofValue = toMultibase(form);
      const r = await verifier.verifyDetailed(resigned as TrustTaskDocument<unknown>);
      assert.equal(r.valid, true, r.detail);
    }
  });
}

// ecdsa-jcs-2019.fixture.json is shared with trust-tasks-go/proof/testdata:
// `ts-*` signed here (noble's ECDSA is RFC 6979, so reproducible), `go-*` by
// the Go backend, picked with a high S. Seed 7, as the eddsa fixture.
const ecdsaFixtures = JSON.parse(
  readFileSync(fileURLToPath(new URL("../../test/ecdsa-jcs-2019.fixture.json", import.meta.url)), "utf8"),
) as Record<string, TrustTaskDocument<unknown>>;

for (const [name, doc] of Object.entries(ecdsaFixtures)) {
  test(`interop: verifies the ${name} ecdsa-jcs-2019 fixture`, async () => {
    const r = await verifier.verifyDetailed(doc);
    assert.equal(r.valid, true, r.detail);
  });
}

for (const curve of ["p256", "p384"] as const) {
  test(`interop: reproduces the ts-${curve} ecdsa-jcs-2019 fixture byte for byte`, () => {
    const fixtureDoc = ecdsaFixtures[`ts-${curve}`];
    const signer = seededSigner(curve, 7);
    assert.equal(signer.did, fixtureDoc.issuer);
    const proof = fixtureDoc.proof as Record<string, unknown>;
    const signed = signTrustTask(fixtureDoc, signer, { created: new Date(proof.created as string) });
    assert.deepEqual(signed.proof, proof);
  });
}

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
