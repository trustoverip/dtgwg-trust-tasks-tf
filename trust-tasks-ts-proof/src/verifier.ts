import type { ProofVerifier, TrustTaskDocument } from "@openvtc/trust-tasks";
import { ed25519 } from "@noble/curves/ed25519.js";
import { p256, p384 } from "@noble/curves/nist.js";
import { sha256 } from "@noble/hashes/sha2.js";
import { sha384 } from "@noble/hashes/sha2.js";
import { canonicalBytes } from "./jcs.js";
import { fromMultibase, publicKeyFromDidKey, type Cryptosuite } from "./keys.js";

/** Why a proof failed to verify — the same taxonomy the Rust and Dart proof
 * libraries use, so a failure logged by any of them reads the same. All reach
 * the wire as `proofInvalid`; these and {@link ProofVerification.detail} are for
 * your logs (SPEC §10.4). */
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
  /** Diagnostic text for logs; never send it to the peer (SPEC §10.4). */
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

const DATE_TIME_STAMP = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})$/;

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
    const cryptosuite = p.cryptosuite as string;
    if (!this.cryptosuites.has(cryptosuite as Cryptosuite)) {
      return fail("unsupportedCryptosuite", cryptosuite);
    }

    // VC Data Integrity §2.1: `created`, if present, is a dateTimeStamp — UTC or
    // an explicit offset — and not beyond the clock-skew allowance. An
    // offset-less value is refused rather than read as local time, as the Rust
    // and Dart verifiers do.
    const created = p.created;
    if (created !== undefined) {
      if (typeof created !== "string" || !DATE_TIME_STAMP.test(created)) {
        return fail("malformedProof", `proof.created must be a dateTimeStamp with Z or an offset: ${created}`);
      }
      const horizon = this.now() + Math.max(0, this.clockSkewMs);
      if (Date.parse(created) > horizon) {
        return fail("malformedProof", `proof.created is more than ${this.clockSkewMs}ms in the future`);
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

    // Hash the proof configuration (proof minus proofValue) and the document
    // minus its proof, each with the suite's digest, and verify the signature
    // over the concatenation, configuration first.
    const { proofValue: _pv, ...config } = p;
    const { proof: _proof, ...body } = doc as Record<string, unknown>;
    const digest = key.curve === "p384" ? sha384 : sha256;
    const hashData = new Uint8Array([...digest(canonicalBytes(config)), ...digest(canonicalBytes(body))]);

    try {
      const ok =
        key.curve === "ed25519"
          ? ed25519.verify(signature, hashData, key.bytes)
          : (key.curve === "p384" ? p384 : p256).verify(signature, digest(hashData), key.bytes);
      return ok ? { valid: true } : fail("signatureInvalid", "signature does not verify");
    } catch (e) {
      return fail("signatureInvalid", `verification error: ${String(e)}`);
    }
  }
}

function fail(failure: ProofFailureKind, detail: string): ProofVerification {
  return { valid: false, failure, detail };
}
