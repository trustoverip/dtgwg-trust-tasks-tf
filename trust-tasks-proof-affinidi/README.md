# trust-tasks-proof-affinidi

[`ProofVerifier`](../trust-tasks-rs/src/proof.rs) implementation for the [Trust Tasks](https://trusttasks.org/) framework, backed by the [`affinidi-data-integrity`](https://crates.io/crates/affinidi-data-integrity) W3C Data Integrity crate. Verifies `proof` members on Trust Task documents using the EdDSA cryptosuites Affinidi publishes today (`eddsa-rdfc-2022`, `eddsa-jcs-2022`) and any future suites the upstream crate adds.

## Why

The framework's [SPEC §7.2 item 7](../SPEC.md#72-consumer-requirements) requires a conforming consumer to verify `proof` against the in-band `issuer` when present. The framework crate intentionally ships no cryptosuite implementations; this crate is the first concrete `ProofVerifier`.

## Quickstart

```rust,ignore
use trust_tasks_proof_affinidi::AffinidiProofVerifier;
use trust_tasks_rs::ProofVerifier;

// did:key only — offline, no I/O. Good for tests and self-issued docs.
let verifier = AffinidiProofVerifier::for_did_key();
verifier.verify(&inbound_doc).await?;
```

For `did:web` / `did:webvh` / `did:peer` / `did:jwk` and the rest of the methods the Affinidi resolver cache handles, use [`CachedDidResolver`](src/resolver.rs):

```rust,ignore
use std::sync::Arc;
use affinidi_did_resolver_cache_sdk::{config::DIDCacheConfigBuilder, DIDCacheClient};
use trust_tasks_proof_affinidi::{AffinidiProofVerifier, CachedDidResolver};

let client = DIDCacheClient::new(DIDCacheConfigBuilder::default().build()).await?;
let resolver = Arc::new(CachedDidResolver::new(Arc::new(client)));
let verifier = AffinidiProofVerifier::with_resolver(resolver);
```

The default builder runs in local mode (no network) — sufficient for `did:key`, `did:peer`, and `did:jwk`. Add `.with_network_mode(...)` to point at a running resolver cache server for `did:web`, `did:webvh`, and the rest. See [`affinidi-did-resolver-cache-sdk`](https://crates.io/crates/affinidi-did-resolver-cache-sdk) for the full configuration surface.

The adapter currently extracts public keys from `Multikey`-typed verification methods (`publicKeyMultibase`); JWK-bearing methods surface a clean `Resolver` error so callers can stack a custom resolver in front.

## Error mapping

| `DataIntegrityError`     | `VerificationError`            | SPEC §8.3 standard code |
|--------------------------|--------------------------------|-------------------------|
| `UnsupportedCryptoSuite` | `UnsupportedCryptosuite`       | `proof_invalid`         |
| `KeyTypeMismatch`        | `IssuerMismatch` (descriptive) | `proof_invalid`         |
| `InvalidSignature`       | `SignatureInvalid`             | `proof_invalid`         |
| `InvalidPublicKey`       | `MalformedProof`               | `proof_invalid`         |
| `MalformedProof`         | `MalformedProof`               | `proof_invalid`         |
| `Canonicalization`       | `Other`                        | `proof_invalid`         |
| _other_                  | `Other`                        | `proof_invalid`         |

A consumer pipeline that catches `VerificationError` converts it to a `trust-task-error/0.1` document with `code = proof_invalid` (or `code = proof_required` if the proof was missing in the first place — that case is raised by the framework, not by this crate).

## MSRV

1.94, matching `affinidi-data-integrity` 0.6.

## Status

`0.1.0`, tracking SPEC.md `0.1`. Round-trip-tested against `affinidi-data-integrity`'s `sign` (see `tests/round_trip.rs`).
