import type { TrustTaskDocument } from "@openvtc/trust-tasks";
import { ed25519 } from "@noble/curves/ed25519.js";
import { p256, p384 } from "@noble/curves/nist.js";
import { sha256, sha384 } from "@noble/hashes/sha2.js";
import { canonicalBytes } from "./jcs.js";
import { didKeyFromPublicKey, toMultibase, type Cryptosuite, type Curve, type PublicKey } from "./keys.js";

/** A signer: a private key of a known curve, with the `did:key` it publishes. */
export interface Signer {
  curve: Curve;
  /** The `did:key:...#...` verification method the proof names. */
  verificationMethod: string;
  /** The document's `issuer` — the `did:key` (no fragment). */
  did: string;
  sign(hashData: Uint8Array): Uint8Array;
}

/** Build a {@link Signer} from a raw private key. */
export function signerFromPrivateKey(curve: Curve, privateKey: Uint8Array): Signer {
  const publicKey: PublicKey =
    curve === "ed25519"
      ? { curve, bytes: ed25519.getPublicKey(privateKey) }
      : { curve, bytes: (curve === "p384" ? p384 : p256).getPublicKey(privateKey, true) };
  const verificationMethod = didKeyFromPublicKey(publicKey);
  const did = verificationMethod.split("#", 1)[0];
  const sign =
    curve === "ed25519"
      ? (h: Uint8Array) => ed25519.sign(h, privateKey)
      : (h: Uint8Array) => {
          const curveApi = curve === "p384" ? p384 : p256;
          const digest = curve === "p384" ? sha384 : sha256;
          return curveApi.sign(digest(h), privateKey);
        };
  return { curve, verificationMethod, did, sign };
}

export interface SignOptions {
  proofPurpose?: string;
  /** ISO-8601; defaults to now. Written in UTC without a zero fraction (as the
   * Rust crate writes it), so the same key over the same document and `created`
   * reproduces the Rust `proofValue`. */
  created?: Date;
}

const CURVE_SUITE: Record<Curve, Cryptosuite> = {
  ed25519: "eddsa-jcs-2022",
  p256: "ecdsa-jcs-2019",
  p384: "ecdsa-jcs-2019",
};

/**
 * Return a copy of `doc` with a `proof` over everything else in it, from
 * `signer`. Refuses — throwing — to sign a document whose `issuer` is absent or
 * is not the signer's `did:key`: such a document could never verify (SPEC
 * §4.7). An existing `proof` is replaced.
 */
export function signTrustTask<P>(
  doc: TrustTaskDocument<P>,
  signer: Signer,
  options: SignOptions = {},
): TrustTaskDocument<P> {
  const { proof: _drop, ...body } = doc as Record<string, unknown>;
  if (typeof body.issuer !== "string") {
    throw new Error("document carries no in-band issuer to bind the proof to");
  }
  if (body.issuer !== signer.did) {
    throw new Error(`document issuer ${body.issuer} is not the signer ${signer.did}`);
  }

  const config: Record<string, unknown> = {
    type: "DataIntegrityProof",
    cryptosuite: CURVE_SUITE[signer.curve],
    created: timestamp(options.created ?? new Date()),
    verificationMethod: signer.verificationMethod,
    proofPurpose: options.proofPurpose ?? "assertionMethod",
  };

  const digest = signer.curve === "p384" ? sha384 : sha256;
  const hashData = new Uint8Array([...digest(canonicalBytes(config)), ...digest(canonicalBytes(body))]);
  const proofValue = toMultibase(signer.sign(hashData));

  return { ...(body as TrustTaskDocument<P>), proof: { ...config, proofValue } as TrustTaskDocument<P>["proof"] };
}

/** UTC, `Z`-terminated, no fraction when it would be all zeros — chrono's form,
 * so a proof signed here and one signed by the Rust crate carry the same bytes. */
function timestamp(d: Date): string {
  return d.toISOString().replace(/\.000Z$/, "Z");
}
