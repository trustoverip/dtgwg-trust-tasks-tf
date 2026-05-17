# Changelog

All notable changes to `trust-tasks-proof-affinidi` are documented in
this file. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); this crate
tracks `trust-tasks-rs`'s `MAJOR.MINOR`.

## [0.1.0] — initial pre-release, tracks `SPEC.md` 0.1

### Added

- `AffinidiProofVerifier` — `trust_tasks_rs::ProofVerifier` implementation
  backed by `affinidi-data-integrity` 0.6. Supports `eddsa-rdfc-2022` and
  `eddsa-jcs-2022` cryptosuites out of the box; tracks whatever
  cryptosuites the upstream crate adds via its feature flags.
- `AffinidiProofVerifier::for_did_key()` — purely-local resolver wrapper
  around `DidKeyResolver`, suitable for self-issued documents and tests.
- `AffinidiProofVerifier::with_resolver(Arc<dyn VerificationMethodResolver>)` —
  plug in a custom resolver (e.g. for `did:web` or `did:webvh`).
- `CachedDidResolver` — wraps `affinidi_did_resolver_cache_sdk::DIDCacheClient`,
  resolves the DID, walks the resulting `Document` for a matching
  `verificationMethod`, decodes its `publicKeyMultibase` via
  `decode_multikey_with_codec`, and maps the multicodec to
  `KeyType` (Ed25519, X25519, P-256, P-384, secp256k1). JWK-bearing
  verification methods surface a typed `Resolver` error so callers can
  stack a custom resolver in front.
- Error mapping from `DataIntegrityError` into the framework's
  `VerificationError` taxonomy — every failure mode lines up with the
  SPEC §8.3 `proof_invalid` standard code path.
- `tests/round_trip.rs` — happy path sign/verify, tampered payload,
  missing proof, unresolvable verification method.
- `tests/cached_resolver.rs` — local-mode `DIDCacheClient` resolves a
  `did:key`, the wrapper verifies the proof end-to-end; unresolvable
  DID does not return `SignatureInvalid` (which would mis-signal
  forgery for a resolver failure).

[0.1.0]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/tag/v0.1.0
