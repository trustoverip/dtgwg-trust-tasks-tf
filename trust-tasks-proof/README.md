# trust-tasks-proof

Pluggable [`ProofVerifier`](../trust-tasks-rs/src/proof.rs) implementations for the [Trust Tasks](https://trusttasks.org/) framework.

The framework's `ProofVerifier` trait is the seam where W3C Data Integrity cryptosuite implementations plug in; this crate is the umbrella that hosts those implementations behind Cargo features so a single dependency line opts you in to a specific backend without dragging in the others.

## Backends

| Cargo feature | Module       | Backed by                                            | Cryptosuites                            |
|---------------|--------------|------------------------------------------------------|-----------------------------------------|
| `affinidi` ✱  | `affinidi`   | [`affinidi-data-integrity`](https://crates.io/crates/affinidi-data-integrity) | `eddsa-rdfc-2022`, `eddsa-jcs-2022`     |

✱ = enabled by default.

## Why

The framework's [SPEC §7.2 item 7](../SPEC.md#72-consumer-requirements) requires a conforming consumer to verify `proof` against the in-band `issuer` when present. `trust-tasks-rs` intentionally ships no cryptosuites — this crate hosts the concrete implementations.

## Quickstart (`affinidi` backend)

```rust,ignore
use trust_tasks_proof::affinidi::Verifier;
use trust_tasks_rs::ProofVerifier;

// did:key only — offline, no I/O. Good for tests and self-issued docs.
let verifier = Verifier::for_did_key();
verifier.verify(&inbound_doc).await?;
```

For `did:web` / `did:webvh` / `did:peer` / `did:jwk` and the rest of the methods the Affinidi resolver cache handles, use [`CachedDidResolver`](src/affinidi/resolver.rs):

```rust,ignore
use std::sync::Arc;
use affinidi_did_resolver_cache_sdk::{config::DIDCacheConfigBuilder, DIDCacheClient};
use trust_tasks_proof::affinidi::{CachedDidResolver, Verifier};

let client = DIDCacheClient::new(DIDCacheConfigBuilder::default().build()).await?;
let resolver = Arc::new(CachedDidResolver::new(Arc::new(client)));
let verifier = Verifier::with_resolver(resolver);
```

The default builder runs in local mode (no network) — sufficient for `did:key`, `did:peer`, and `did:jwk`. Add `.with_network_mode(...)` to point at a running resolver cache server for `did:web`, `did:webvh`, and the rest. See [`affinidi-did-resolver-cache-sdk`](https://crates.io/crates/affinidi-did-resolver-cache-sdk) for the full configuration surface.

The adapter currently extracts public keys from `Multikey`-typed verification methods (`publicKeyMultibase`); JWK-bearing methods surface a clean `Resolver` error so callers can stack a custom resolver in front.

## Opting out of the default backend

```toml
[dependencies]
trust-tasks-proof = { git = "...", default-features = false }
# add other backends here as they're introduced
```

With no features enabled the crate compiles to an empty surface — useful while you wait for a non-Affinidi backend to land, or if you're consuming the trait from `trust-tasks-rs` directly and only want the umbrella as a future home for adapters.

## Error mapping (`affinidi` backend)

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
