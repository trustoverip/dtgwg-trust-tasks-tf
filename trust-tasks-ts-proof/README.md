# @openvtc/trust-tasks-proof

W3C [Data Integrity](https://www.w3.org/TR/vc-data-integrity/) proofs for
[Trust Tasks](https://trusttasks.org): a `ProofVerifier` for
[`@openvtc/trust-tasks`](https://www.npmjs.com/package/@openvtc/trust-tasks), and
the signer that produces what it verifies.

Verifies **`eddsa-jcs-2022`** and **`ecdsa-jcs-2019`** on
[`@noble`](https://paulmillr.com/noble/) — no JSON-LD, no WASM, runs in browsers,
Node and React Native. Byte-compatible with the Rust `trust-tasks-proof` crate
and the Dart `trust_tasks_proof` package: for the same Ed25519 key, document and
`created` it reproduces the Rust `proofValue` exactly.

```console
npm install @openvtc/trust-tasks @openvtc/trust-tasks-proof
```

## Verifying

```ts
import { consumeInbound } from "@openvtc/trust-tasks";
import { DataIntegrityProofVerifier } from "@openvtc/trust-tasks-proof";

const outcome = await consumeInbound({
  proofPolicy: { kind: "verify", verify: new DataIntegrityProofVerifier() },
  // transport, spec, doc, handler, ... as usual
});
```

A document whose proof fails is rejected with `proofInvalid`, and nothing about
*why* reaches the peer (SPEC §10.4). To log the reason, call `verifyDetailed`,
which returns a `ProofVerification` whose `failure` is one of `malformedProof`,
`issuerMismatch`, `unsupportedCryptosuite` or `signatureInvalid` — the same
taxonomy as the Rust and Dart libraries.

## What is checked

1. The proof is a `DataIntegrityProof` with a non-empty `cryptosuite`,
   `verificationMethod`, `proofPurpose` and `proofValue`.
2. The cryptosuite is one this verifier accepts.
3. `created`, when present, is a `dateTimeStamp` (ending in `Z` or an explicit
   offset) and no more than `clockSkewMs` (60s default) in the future. An
   offset-less timestamp is refused rather than read as local time.
4. **The proof is bound to the document's `issuer`.** The DID part of
   `verificationMethod` must equal `issuer` exactly, with no normalization (SPEC
   §4.8), or the document is `issuerMismatch`. A valid signature proves only
   that *some* key signed.
5. The signature verifies over the document minus its `proof`.

Keys travel in the `did:key` verification method — no resolver, no network.

## Signing

```ts
import { signerFromPrivateKey, signTrustTask } from "@openvtc/trust-tasks-proof";

const signer = signerFromPrivateKey("ed25519", privateKey); // or "p256" / "p384"
const signed = signTrustTask(doc, signer);
```

`signTrustTask` refuses to sign a document with no `issuer`, or one whose
`issuer` is not the signer's `did:key` — such a document could never verify. It
replaces any existing `proof`.

## Cryptosuites

| Cryptosuite | |
|---|---|
| `eddsa-jcs-2022` | ✓ Ed25519 |
| `ecdsa-jcs-2019` | ✓ P-256, P-384 |
| `eddsa-rdfc-2022`, `ecdsa-rdfc-2019` | — a Trust Task document has no JSON-LD `@context` for RDF canonicalization to expand |

ML-DSA (post-quantum) suites will follow; the `@noble` foundation is chosen so
they can be added without a JSON-LD toolchain.

## License

Apache-2.0.
