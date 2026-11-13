import { base58 } from "@scure/base";

/**
 * A cryptosuite this package can verify, and the curve behind it. Only the JCS
 * suites: a Trust Task document is processed as JSON whether or not it carries
 * the optional `@context` (SPEC §4.7.1), so it is never expanded and the RDF
 * canonicalisation the `-rdfc-` suites need does not apply.
 */
export type Cryptosuite = "eddsa-jcs-2022" | "ecdsa-jcs-2019";

export type Curve = "ed25519" | "p256" | "p384";

/** The multicodec headers a `did:key` uses, as the leading bytes of the
 * base58btc-decoded multikey. */
const MULTICODEC: ReadonlyArray<{ curve: Curve; header: readonly number[] }> = [
  { curve: "ed25519", header: [0xed, 0x01] },
  { curve: "p256", header: [0x80, 0x24] },
  { curve: "p384", header: [0x81, 0x24] },
];

/**
 * The most bytes a base58btc value this package decodes may carry. The largest
 * it reads is a P-384 signature (96 bytes); a `did:key` multikey is at most 51
 * (a 2-byte multicodec header and a 49-byte compressed P-384 point). 128 leaves
 * headroom without letting a value's length set the decoder's cost, which grows
 * with the square of it.
 */
const MAX_BASE58_BYTES = 128;

/** The longest base58btc string that can encode `MAX_BASE58_BYTES` bytes:
 * ceil(128 * log(256) / log(58)) = 175 characters. */
const MAX_BASE58_CHARS = 175;

/**
 * Decode base58btc, refusing a value longer than `MAX_BASE58_CHARS` before
 * decoding it, and one that decodes to more than `MAX_BASE58_BYTES`.
 */
function base58DecodeBounded(value: string): Uint8Array {
  if (value.length > MAX_BASE58_CHARS) {
    throw new Error(`base58: value is ${value.length} characters, longer than the ${MAX_BASE58_CHARS} this decoder accepts`);
  }
  const decoded = base58.decode(value);
  if (decoded.length > MAX_BASE58_BYTES) {
    throw new Error(`base58: value decodes to more than ${MAX_BASE58_BYTES} bytes`);
  }
  return decoded;
}

/** A public key recovered from a `did:key` verification method. */
export interface PublicKey {
  curve: Curve;
  bytes: Uint8Array;
}

/**
 * Decode the public key a `did:key` verification method names, locally — no
 * network. The key is the multibase multikey after the method-specific id, so
 * `did:key:<mb>` and `did:key:<mb>#<mb>` both carry it in `<mb>`.
 *
 * Returns null for a DID that is not a `did:key`, or one whose multikey header
 * is a curve this package does not verify.
 */
export function publicKeyFromDidKey(verificationMethod: string): PublicKey | null {
  const did = verificationMethod.split("#", 1)[0];
  const prefix = "did:key:";
  if (!did.startsWith(prefix)) return null;
  const mb = did.slice(prefix.length);
  if (!mb.startsWith("z")) return null;
  let decoded: Uint8Array;
  try {
    decoded = base58DecodeBounded(mb.slice(1));
  } catch {
    return null;
  }
  for (const { curve, header } of MULTICODEC) {
    if (decoded.length > header.length && header.every((b, i) => decoded[i] === b)) {
      return { curve, bytes: decoded.slice(header.length) };
    }
  }
  return null;
}

/** The multibase-base58btc form of raw `bytes`, as a `proofValue` carries a
 * signature (leading `z`). */
export function toMultibase(bytes: Uint8Array): string {
  return "z" + base58.encode(bytes);
}

/** Decode a multibase-base58btc `z...` value to its raw bytes. Refuses a value
 * of more than 175 base58 characters, or one decoding to more than 128 bytes —
 * more than any key or signature this package reads. */
export function fromMultibase(value: string): Uint8Array {
  if (!value.startsWith("z")) throw new Error("not base58btc multibase (no leading z)");
  return base58DecodeBounded(value.slice(1));
}

/** Build a `did:key` verification method for a raw public key. */
export function didKeyFromPublicKey(key: PublicKey): string {
  const header = MULTICODEC.find((m) => m.curve === key.curve)!.header;
  const mb = toMultibase(Uint8Array.from([...header, ...key.bytes]));
  return `did:key:${mb}#${mb}`;
}
