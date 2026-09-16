# trust_tasks_proof

W3C [Data Integrity](https://www.w3.org/TR/vc-data-integrity/) proof
verification for [Trust Tasks](https://trusttasks.org): a `ProofVerifier` for
[`package:trust_tasks`](https://pub.dev/packages/trust_tasks), built on
Affinidi's [`package:ssi`](https://pub.dev/packages/ssi).

`trust_tasks` declares the `ProofVerifier` seam and implements no cryptosuite,
so that it can stay dependency-free. This package is the implementation to put
behind it when you want one that already works — the Dart counterpart of the
Rust [`trust-tasks-proof`](https://crates.io/crates/trust-tasks-proof) crate,
and it verifies what that crate signs.

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
*why* reaches the peer (SPEC §10.4). To log the reason, call `verifyDetailed`
yourself; it returns a `ProofVerification` whose `failure` is one of
`malformedProof`, `issuerMismatch`, `unsupportedCryptosuite` or
`signatureInvalid` — the same taxonomy as the Rust crate.

See [`example/main.dart`](example/main.dart) for a document signed by the Rust
library, verified and then tampered with.

## What is checked

1. The proof is a `DataIntegrityProof` with a non-empty `cryptosuite`,
   `verificationMethod`, `proofPurpose` and `proofValue`.
2. The cryptosuite is one this verifier accepts.
3. `created`, when present, is a `dateTimeStamp` — ending in `Z` or an explicit
   offset — and no more than `clockSkew` (60 seconds by default) in the future.
   An offset-less timestamp is refused rather than guessed at: VC Data
   Integrity §2.1 says to read it as UTC, and `DateTime.parse` would read it as
   local time.
4. **The proof is bound to the document's `issuer`.** A valid signature proves
   only that *some* key signed. The DID part of `verificationMethod` (before
   `#`) must equal `issuer` exactly, with no normalization (SPEC §4.8), or the
   document is refused as `issuerMismatch`. Without this, anyone could sign
   with their own key while claiming to be someone else.
5. The signature verifies over the document minus its `proof`, using the key
   the issuer's DID resolves to.

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
| `ecdsa-jcs-2019` | ✓ (P-256, P-384) |
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

This package verifies; it does not sign yet. `ssi`'s generators currently stamp
`created` without a zone designator, which this verifier and the Rust one both
refuse — a signing helper will follow once an `ssi` release carries the fix.

## License

Apache-2.0.
