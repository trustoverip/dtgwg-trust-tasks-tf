/**
 * W3C Data Integrity proofs for Trust Tasks: a {@link DataIntegrityProofVerifier}
 * for `@openvtc/trust-tasks`, and {@link signTrustTask}, which produces what it
 * verifies. `eddsa-jcs-2022` and `ecdsa-jcs-2019`, on `@noble` — browser-first,
 * no JSON-LD, byte-compatible with the Rust and Dart proof libraries.
 */
export {
  DataIntegrityProofVerifier,
  SUPPORTED_CRYPTOSUITES,
  DEFAULT_CLOCK_SKEW_MS,
  type ProofFailureKind,
  type ProofVerification,
  type VerifierOptions,
} from "./verifier.js";
export { signTrustTask, signerFromPrivateKey, type Signer, type SignOptions } from "./sign.js";
export { publicKeyFromDidKey, didKeyFromPublicKey, toMultibase, fromMultibase, type Cryptosuite, type Curve, type PublicKey } from "./keys.js";
export { canonicalize, canonicalBytes } from "./jcs.js";
