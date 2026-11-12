# trust_tasks_proof

W3C [Data Integrity](https://www.w3.org/TR/vc-data-integrity/) proofs for
[Trust Tasks](https://trusttasks.org): a `ProofVerifier` for
[`package:trust_tasks`](https://pub.dev/packages/trust_tasks), and the signer
that produces what it verifies, built on Affinidi's
[`package:ssi`](https://pub.dev/packages/ssi).

`trust_tasks` declares the `ProofVerifier` seam and implements no cryptosuite,
so that it can stay dependency-free. This package is the implementation to put
behind it when you want one that already works — the Dart counterpart of the
Rust [`trust-tasks-proof`](https://crates.io/crates/trust-tasks-proof) crate.
Each verifies what the other signs; for the same Ed25519 key, document and
`created`, the two produce the same `proofValue` byte for byte.

```console
dart pub add trust_tasks trust_tasks_proof
```

Requires Dart 3.6 or later.

## Verifying inside the §7.2 pipeline

```dart
import 'package:trust_tasks/trust_tasks.dart';
import 'package:trust_tasks_proof/trust_tasks_proof.dart';

final outcome = await consumeInbound(
  proofPolicy: ProofPolicy.verify(DataIntegrityProofVerifier.forDidKey()),
  // transport, spec, doc, handler, ... as usual
);
```

A document whose proof fails is rejected with `proofInvalid`, and nothing about
*why* reaches the peer (SPEC §12.4). To log the reason, call `verifyDetailed`
yourself; it returns a `ProofVerification` whose `failure` is one of
`malformedProof`, `issuerMismatch`, `unsupportedCryptosuite` or
`signatureInvalid` — the same taxonomy as the Rust crate.

See [`example/main.dart`](example/main.dart) for a document signed by the Rust
library, verified and then tampered with.

## What is checked

1. The proof is a `DataIntegrityProof` with a non-empty `cryptosuite`,
   `verificationMethod`, `proofPurpose` and `proofValue`.
2. `proofPurpose` names a signing relationship: `authentication`,
   `assertionMethod`, `capabilityInvocation` or `capabilityDelegation`.
   `keyAgreement` and any other value are refused (SPEC §4.7.3).
3. The cryptosuite is one this verifier accepts.
4. `created`, when present, is a `dateTimeStamp` — ending in `Z` or an explicit
   offset — and no more than `clockSkew` (60 seconds by default) in the future.
   An offset-less timestamp is refused rather than guessed at: VC Data
   Integrity §2.1 says to read it as UTC, and `DateTime.parse` would read it as
   local time. `expires`, when present, is a `dateTimeStamp` that has not
   passed, with every field in range: no February 30, hour 24, leap second, or
   offset beyond ±14:00.
5. **The proof is bound to the document's `issuer`.** A valid signature proves
   only that *some* key signed. The DID part of `verificationMethod` (before
   `#`) must equal `issuer` exactly, with no normalization (SPEC §4.8), or the
   document is refused as `issuerMismatch`. Without this, anyone could sign
   with their own key while claiming to be someone else.
6. The issuer's DID document lists the `verificationMethod` under the
   relationship `proofPurpose` names — by absolute reference, by a fragment
   relative to the DID, or embedded — defines it under `verificationMethod`,
   and names the issuer as its controller. A `did:key` lists its key under all
   four signing relationships. A key published only for `keyAgreement`, or for
   another relationship, is refused as `signatureInvalid`.
7. An Ed25519 key is a canonically encoded point of large order. A
   small-order key satisfies the verification equation for any message with a
   chosen signature; ssi 4 refuses it, and ssi 3.9 does not, so this package
   checks it whichever ssi resolves.
8. The signature verifies over the document minus its `proof`, using that key.

Two cases where this package refuses what `trust-tasks-proof` (Rust) accepts:

- **A key embedded only under a relationship.** ssi takes the key from the
  document's `verificationMethod` set, so a method that appears embedded under
  `authentication` (say) and nowhere in `verificationMethod` cannot be used,
  and the proof is refused. When it appears in both, the two must carry the
  same key and both must name the issuer as controller.
- **`did:scid` with a `?src=` locator.** Rust strips the locator before
  comparing the method with its document. This package compares the
  `verificationMethod` with the document exactly, so a reference carrying
  `?src=` does not match the method the document lists under the bare DID.

The verifier never mutates the document it is given and never throws for a bad
one.

## Resolving keys

| Constructor | Resolves | I/O |
|---|---|---|
| `DataIntegrityProofVerifier.forDidKey()` | `did:key` only; every other method fails | none |
| `DataIntegrityProofVerifier.withResolver(resolver)` | whatever `resolver` does — e.g. `ssi`'s `UniversalDIDResolver` for `did:web`, `did:peer` and `did:webvh` | the resolver's |

Resolution runs on input from a peer that has not been authenticated yet. If
you accept `did:web`, prefer a resolver that caches and bounds its fetches.

## Cryptosuites

| Cryptosuite | |
|---|---|
| `eddsa-jcs-2022` | ✓ |
| `ecdsa-jcs-2019` | ✓ P-256. P-384 too, but the Rust crate (through `affinidi-data-integrity` 0.7.11) accepts only P-256 for this suite, so use P-256 when a Rust peer must verify |
| `eddsa-rdfc-2022`, `ecdsa-rdfc-2019` | — a Trust Task document has no JSON-LD `@context`, so there is nothing for RDF canonicalization to expand |
| ML-DSA suites | — see below |

Pass `cryptosuites:` to either constructor to accept a subset.

### Why no ML-DSA

`package:ssi`'s ML-DSA verifiers exist only in `ssi` 4. This package accepts
`ssi` `>=3.9.6 <5.0.0` so that it can share a dependency graph with
[`package:didcomm`](https://pub.dev/packages/didcomm), which pins `ssi` 3. The
ssi APIs used here are identical in both majors, and the test suite runs
against each.

## Signing

```dart
final signed = await signTrustTask(doc, signer); // signer: an ssi DidSigner
```

`signTrustTask` returns a copy of the document with a `proof` over everything
else in it, replacing any proof already there. It refuses — with a
`SignException` — to sign a document that could never verify: one with no
`issuer`, or whose `issuer` is not the DID of the signer's verification method.

The cryptosuite follows from the key: `eddsa-jcs-2022` for Ed25519,
`ecdsa-jcs-2019` for P-256 and P-384. `created` is written in UTC with a `Z`.
`proofPurpose` defaults to `authentication`, the purpose SPEC §4.7.3 recommends
for a Trust Task document; pass `assertionMethod` only where a specification
defines the proof as an attestation. A purpose outside `signingPurposes` is an
`ArgumentError`.

It builds the proof itself instead of calling `ssi`'s generators. Before ssi
4.3.0 those wrote `created` with no zone designator, which this verifier and
the Rust one both refuse, and this package still supports ssi 3.9.

## License

Apache-2.0.
