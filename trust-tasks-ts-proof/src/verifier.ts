import type { ProofVerifier, TrustTaskDocument } from "@openvtc/trust-tasks/runtime";
import { ed25519 } from "@noble/curves/ed25519.js";
import { p256, p384 } from "@noble/curves/nist.js";
import { sha256 } from "@noble/hashes/sha2.js";
import { sha384, sha512 } from "@noble/hashes/sha2.js";
import { canonicalBytes } from "./jcs.js";
import { fromMultibase, publicKeyFromDidKey, type Cryptosuite } from "./keys.js";

/** Why a proof failed to verify — the same taxonomy the Rust and Dart proof
 * libraries use, so a failure logged by any of them reads the same. All reach
 * the wire as `proofInvalid`; these and {@link ProofVerification.detail} are for
 * your logs (SPEC §12.4). */
export type ProofFailureKind =
  | "malformedProof"
  | "issuerMismatch"
  | "unsupportedCryptosuite"
  | "signatureInvalid";

/** The result of {@link DataIntegrityProofVerifier.verifyDetailed}. */
export interface ProofVerification {
  valid: boolean;
  /** Set when `valid` is false. */
  failure?: ProofFailureKind;
  /** Diagnostic text for logs; never send it to the peer (SPEC §12.4). */
  detail?: string;
}

/** The cryptosuites this package can verify. */
export const SUPPORTED_CRYPTOSUITES: readonly Cryptosuite[] = ["eddsa-jcs-2022", "ecdsa-jcs-2019"];

/** How far into the verifier's future a proof's `created` may sit. */
export const DEFAULT_CLOCK_SKEW_MS = 60_000;

export interface VerifierOptions {
  /** Restrict the accepted cryptosuites to a subset of {@link SUPPORTED_CRYPTOSUITES}. */
  cryptosuites?: readonly Cryptosuite[];
  /** Allowance applied to a proof's `created`. Defaults to {@link DEFAULT_CLOCK_SKEW_MS}. */
  clockSkewMs?: number;
  /** The clock, for testing. Defaults to `Date.now`. */
  now?: () => number;
}

/** The `proofPurpose` values a proof may carry: the verification relationships
 * a signature can be made under, all of which a `did:key` authorises.
 * `keyAgreement` is deliberately absent — a key-agreement key never authorises a
 * signature (SPEC §4.7.3) — as it is from the Rust, Go and Dart verifiers. */
export const SIGNING_PURPOSES: readonly string[] = [
  "authentication",
  "assertionMethod",
  "capabilityInvocation",
  "capabilityDelegation",
];

/** Why `purpose` cannot be a proof's `proofPurpose`, or `undefined` when it can. */
export function proofPurposeError(purpose: string): string | undefined {
  if (SIGNING_PURPOSES.includes(purpose)) return undefined;
  return purpose === "keyAgreement"
    ? "proofPurpose keyAgreement never authorises a signature"
    : "proofPurpose names no signing verification relationship";
}

/**
 * A {@link ProofVerifier} for W3C Data Integrity proofs, on `@noble`.
 *
 * Verifies `eddsa-jcs-2022` and `ecdsa-jcs-2019`, over the document minus its
 * `proof` member, and binds the proof to the document's `issuer`: a valid
 * signature proves only that *some* key signed, so the key's `did:key` must also
 * be the `issuer` (SPEC §4.7, §7.2 item 7). did:key only — no network, no
 * resolver; the key travels in the identifier.
 */
export class DataIntegrityProofVerifier implements ProofVerifier {
  readonly cryptosuites: ReadonlySet<Cryptosuite>;
  readonly clockSkewMs: number;
  private readonly now: () => number;

  constructor(options: VerifierOptions = {}) {
    const requested = options.cryptosuites ?? SUPPORTED_CRYPTOSUITES;
    for (const c of requested) {
      if (!SUPPORTED_CRYPTOSUITES.includes(c)) {
        throw new Error(`unsupported cryptosuite: ${c}`);
      }
    }
    this.cryptosuites = new Set(requested);
    this.clockSkewMs = options.clockSkewMs ?? DEFAULT_CLOCK_SKEW_MS;
    this.now = options.now ?? Date.now;
  }

  /** {@link ProofVerifier.verify}: true when the proof verifies. */
  async verify<P>(doc: TrustTaskDocument<P>): Promise<boolean> {
    return (await this.verifyDetailed(doc)).valid;
  }

  /** Verify `doc`'s proof and say why it failed, if it did. Never throws. */
  async verifyDetailed<P>(doc: TrustTaskDocument<P>): Promise<ProofVerification> {
    const proof = (doc as { proof?: unknown }).proof;
    if (proof === undefined || proof === null || typeof proof !== "object") {
      return fail("malformedProof", proof == null ? "document carries no proof member" : "proof is not an object");
    }
    const p = proof as Record<string, unknown>;
    if (p.type !== "DataIntegrityProof") {
      return fail("malformedProof", "proof type must be DataIntegrityProof");
    }
    for (const member of ["cryptosuite", "verificationMethod", "proofPurpose", "proofValue"] as const) {
      if (typeof p[member] !== "string" || p[member] === "") {
        return fail("malformedProof", `proof.${member} must be a non-empty string`);
      }
    }
    const purposeError = proofPurposeError(p.proofPurpose as string);
    if (purposeError !== undefined) {
      return fail("malformedProof", purposeError);
    }
    const cryptosuite = p.cryptosuite as string;
    if (!this.cryptosuites.has(cryptosuite as Cryptosuite)) {
      return fail("unsupportedCryptosuite", cryptosuite);
    }

    // VC Data Integrity §2.1: `created`, if present, is a dateTimeStamp — UTC or
    // an explicit offset — and not beyond the clock-skew allowance. An
    // offset-less value is refused rather than read as local time, as the Rust
    // and Dart verifiers do.
    // Every field must be in range, as for `expires`: `Date.parse` rolls
    // 2026-02-30 into March and gives NaN for month 13, and NaN compares false
    // with the horizon, so such a value used to pass.
    const created = p.created;
    if (created !== undefined) {
      const at = typeof created === "string" ? parseDateTimeStamp(created) : undefined;
      if (at === undefined) {
        return fail("malformedProof", `proof.created must be a dateTimeStamp with Z or an offset: ${String(created)}`);
      }
      const horizon = this.now() + Math.max(0, this.clockSkewMs);
      if (at > horizon) {
        return fail("malformedProof", `proof.created is more than ${this.clockSkewMs}ms in the future`);
      }
    }

    // `expires`, if present, is a dateTimeStamp that has not yet passed (VC Data
    // Integrity §2.1). No allowance applies: the signer chose the instant.
    const expires = p.expires;
    if (expires !== undefined) {
      const at = typeof expires === "string" ? parseDateTimeStamp(expires) : undefined;
      if (at === undefined) {
        return fail("malformedProof", `proof.expires must be a dateTimeStamp with Z or an offset: ${String(expires)}`);
      }
      if (this.now() > at) {
        return fail("malformedProof", `proof.expires ${expires} has passed`);
      }
    }

    // Bind the proof to the in-band issuer (exact string, no normalization —
    // SPEC §4.8).
    const issuer = (doc as { issuer?: unknown }).issuer;
    if (typeof issuer !== "string") {
      return fail("issuerMismatch", "document carries a proof but no in-band issuer to bind it to");
    }
    const vmDid = (p.verificationMethod as string).split("#", 1)[0];
    if (vmDid !== issuer) {
      return fail("issuerMismatch", `verificationMethod is controlled by ${vmDid}, not the document issuer ${issuer}`);
    }

    const contextError = proofContextError(doc as Record<string, unknown>, p);
    if (contextError !== undefined) {
      return fail("malformedProof", contextError);
    }

    const key = publicKeyFromDidKey(p.verificationMethod as string);
    if (key === null) {
      return fail("signatureInvalid", "verificationMethod is not a did:key this verifier supports");
    }
    // eddsa-jcs-2022 is Ed25519; ecdsa-jcs-2019 is P-256 or P-384.
    if (cryptosuite === "eddsa-jcs-2022" && key.curve !== "ed25519") {
      return fail("malformedProof", "eddsa-jcs-2022 requires an Ed25519 key");
    }
    if (cryptosuite === "ecdsa-jcs-2019" && key.curve === "ed25519") {
      return fail("malformedProof", "ecdsa-jcs-2019 requires a P-256 or P-384 key");
    }

    let signature: Uint8Array;
    try {
      signature = fromMultibase(p.proofValue as string);
    } catch (e) {
      return fail("signatureInvalid", `proofValue is not base58btc multibase: ${String(e)}`);
    }

    // Hash the proof configuration (proof minus proofValue) exactly as
    // received — with its `@context`, when it carries one — and the document
    // minus its proof, each with the suite's digest, and verify the signature
    // over the concatenation, configuration first.
    const { proofValue: _pv, ...config } = p;
    const { proof: _proof, ...body } = doc as Record<string, unknown>;
    const digest = key.curve === "p384" ? sha384 : sha256;
    const hashData = new Uint8Array([...digest(canonicalBytes(config)), ...digest(canonicalBytes(body))]);

    try {
      const ok =
        key.curve === "ed25519"
          ? ed25519VerifyStrict(signature, hashData, key.bytes)
          : // The digest is the ECDSA message hash, so no second hash
            // (`prehash`). High-S is valid ECDSA and what Go's ecdsa.Sign
            // emits half the time, so it is accepted (`lowS`).
            (key.curve === "p384" ? p384 : p256).verify(signature, digest(hashData), key.bytes, {
              prehash: false,
              lowS: false,
            });
      return ok ? { valid: true } : fail("signatureInvalid", "signature does not verify");
    } catch (e) {
      return fail("signatureInvalid", `verification error: ${String(e)}`);
    }
  }
}

/**
 * Why `proof`'s `@context` disagrees with `doc`'s, or `undefined` when it does
 * not. eddsa-jcs-2022 / ecdsa-jcs-2019 §3.3.2 step 4: "If proofOptions.@context
 * exists: Check that the securedDocument.@context starts with all values
 * contained in the proofOptions.@context in the same order. Otherwise, set
 * verified to false". A proof without `@context` is checked as received (step
 * 4 does not apply).
 *
 * This verifier requires the two to be **equal**, not only the proof's to be a
 * prefix. The cryptosuite verifies a longer document `@context` by hashing the
 * proof's in its place (step 4.2) and returning that rewritten document as the
 * verified one; {@link ProofVerifier.verify} returns no document, so the caller
 * would go on to act on `@context` entries no signature covers. The Rust, Go
 * and Dart verifiers apply the same rule.
 */
function proofContextError(doc: Record<string, unknown>, proof: Record<string, unknown>): string | undefined {
  if (!Object.prototype.hasOwnProperty.call(proof, "@context")) return undefined;
  if (!Object.prototype.hasOwnProperty.call(doc, "@context")) {
    return "proof carries @context but the document has none";
  }
  let a: Uint8Array;
  let b: Uint8Array;
  try {
    a = canonicalBytes(proof["@context"]);
    b = canonicalBytes(doc["@context"]);
  } catch (e) {
    return `@context cannot be canonicalized: ${String(e)}`;
  }
  if (a.length !== b.length || !a.every((byte, i) => byte === b[i])) {
    return "proof @context does not equal the document's @context";
  }
  return undefined;
}

/**
 * Ed25519 as the Go, Rust and Dart backends verify it: RFC 8032 point decoding
 * (`zip215: false` — @noble's ed25519 defaults to ZIP-215, which accepts
 * non-canonical encodings), a public key of small order refused (noble's strict
 * mode does this), and the **cofactorless** equation `[S]B = R + [k]A` checked
 * by comparing encodings. noble checks the cofactored equation, which also
 * accepts an R carrying a small-order component that the other three refuse.
 */
function ed25519VerifyStrict(signature: Uint8Array, message: Uint8Array, publicKey: Uint8Array): boolean {
  if (!ed25519.verify(signature, message, publicKey, { zip215: false })) return false;
  const Point = ed25519.Point;
  const order = Point.Fn.ORDER;
  const A = Point.fromBytes(publicKey);
  const R = signature.subarray(0, 32);
  const S = leToBigInt(signature.subarray(32, 64));
  const k = leToBigInt(sha512(new Uint8Array([...R, ...publicKey, ...message]))) % order;
  const expected = Point.BASE.multiplyUnsafe(S).subtract(A.multiplyUnsafe(k)).toBytes();
  return expected.length === R.length && expected.every((b, i) => b === R[i]);
}

function leToBigInt(bytes: Uint8Array): bigint {
  let n = 0n;
  for (let i = bytes.length - 1; i >= 0; i--) n = (n << 8n) | BigInt(bytes[i]);
  return n;
}

const DATE_TIME_STAMP_PARTS = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(\.\d+)?(?:Z|([+-])(\d{2}):(\d{2}))$/;

/** `value` as epoch milliseconds when it is a dateTimeStamp with every field in
 * range — month 1-12, a day the month has (leap years included), hour 0-23,
 * minute and second 0-59 (no leap second), an offset of at most 14:00 with
 * minutes 0-59 — else `undefined`. `Date.parse` alone takes 24:00 and rolls
 * 2027-02-30 into March; the Go, Rust and Dart verifiers apply these ranges. */
export function parseDateTimeStamp(value: string): number | undefined {
  const m = DATE_TIME_STAMP_PARTS.exec(value);
  if (m === null) return undefined;
  const [year, month, day, hour, minute, second] = m.slice(1, 7).map(Number);
  const daysInMonth = new Date(Date.UTC(year, month, 0)).getUTCDate();
  if (month < 1 || month > 12 || day < 1 || day > daysInMonth || hour > 23 || minute > 59 || second > 59) {
    return undefined;
  }
  let offsetMinutes = 0;
  if (m[8] !== undefined) {
    const oh = Number(m[9]);
    const om = Number(m[10]);
    if (oh > 14 || om > 59 || (oh === 14 && om !== 0)) return undefined;
    offsetMinutes = (m[8] === "+" ? 1 : -1) * (oh * 60 + om);
  }
  const fractionMs = m[7] === undefined ? 0 : Number(`0${m[7]}`) * 1000;
  const utc = new Date(0);
  utc.setUTCFullYear(year, month - 1, day);
  utc.setUTCHours(hour, minute, second, 0);
  return utc.getTime() + fractionMs - offsetMinutes * 60_000;
}

function fail(failure: ProofFailureKind, detail: string): ProofVerification {
  return { valid: false, failure, detail };
}
