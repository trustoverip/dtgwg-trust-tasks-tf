# Changelog

All notable changes to `trust-tasks-proof` are documented in this file.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this crate tracks `trust-tasks-rs`'s `MAJOR.MINOR`.

## [0.12.1] - 2026-08-26

### Added

- **`ProofExt` — `.sign()` and `.verify()` on a typed `TrustTask<P>`.** Signing
  was a five-step ritual: serialise the typed document to `serde_json::Value`,
  call `affinidi::sign_trust_task`, check the result, deserialise back, and get
  the ordering right. The new extension trait does that round-trip internally,
  so a producer writes `doc.sign(&secret, SignOptions::new()).await?` and a
  consumer writes `doc.verify(&verifier).await?`.

  It is a wrapper, not a reimplementation — `sign_trust_task` is called verbatim,
  so the canonicalisation contract, the deterministic `eddsa-jcs-2022` default,
  the replace-don't-nest rule and the SPEC §4.7/§4.8 issuer pre-flight are
  unchanged, and a document signed either way is byte-identical (asserted by a
  test). The free functions stay public and unchanged.

- `SignError::ProofRoundTrip`, raised only by `ProofExt::sign` when the emitted
  proof cannot be read back into the framework's typed `Proof`. `SignError` is
  `#[non_exhaustive]`, so this is additive.

## [0.12.0] - 2026-08-26

### Changed

- **`trust-tasks-rs` requirement moved to `0.13`.** That release flips
  `IS_PROOF_REQUIRED` on `vta/memory/list/0.1`'s response, so a consumer rejects
  an unproofed response it used to accept. This crate re-exports the generated
  types, so the leading component moves with it. No change to this crate's own
  API.

## [0.11.0] - 2026-08-26

### Changed

- **`trust-tasks-rs` requirement moved to `0.12`** (SPEC §7.2 item 11 duplicate
  execution, item 13 freshness). Leading component moves with the re-exported
  types. No change to signing or verification behaviour.

## [0.8.0] - 2026-08-16

### Changed

- Requires `trust-tasks-rs` 0.8, which adds the `cancelled` standard error code
  (framework 0.4, SPEC §8.3) and the `trust-task-control/0.1` payload types.
  Additive on the Rust side — `StandardCode` has been `#[non_exhaustive]` since
  0.7.0 — so this crate needed no source change.

## [0.7.0] - 2026-08-15

### Changed

- **BREAKING.** Requires `trust-tasks-rs` 0.7, whose `StandardCode` is now
  `#[non_exhaustive]` and carries the new `idConflict` code (framework 0.4,
  SPEC §8.3). Any `match` over `StandardCode` that this crate's types reach
  needs a wildcard arm.

## [0.6.0] - 2026-08-10

### Changed

- **BREAKING.** Requires `trust-tasks-rs` 0.6, which narrows `DigestMultibase`
  to the multibase headers CID 1.0 requires. The core types cross this crate's
  public API, so a graph mixing 0.5 with this crate will not type-check. No API
  of this crate changed on its own account.

## [0.5.0] - 2026-08-09

### Changed

- **BREAKING.** Requires `trust-tasks-rs` 0.5. That release adds a field to
  `TrustTask<P>` for the framework 0.4 `ceremony` member, and the core types
  cross this crate's public API, so a dependency graph mixing 0.4 with this
  crate will not type-check. No API of this crate changed on its own account.

## [0.4.0] - 2026-08-09

### Changed

- **BREAKING.** Requires `trust-tasks-rs` 0.4. That release changes digest
  payload members from `String` to the validating `DigestMultibase` newtype, and
  the core types cross this crate's public API, so a dependency graph mixing
  `trust-tasks-rs` 0.3 with this crate will not type-check. No API of this crate
  changed on its own account.

## [0.2.2] — 2026-07-29

### Added

- `affinidi::sign_trust_task` — the sign-side counterpart to the stock
  `affinidi::Verifier`. Takes a `serde_json::Value` Trust Task document,
  any upstream `Signer` (an `affinidi-secrets-resolver` `Secret` works
  directly), and an upstream `SignOptions`; signs the document with the
  `proof` member removed (the same canonicalisation contract the verify
  side applies) and returns the document with the proof embedded.
  Defaults to the reference ecosystem's signing profile —
  `proofPurpose: assertionMethod` and the `eddsa-jcs-2022` cryptosuite
  (applied whenever `SignOptions::cryptosuite` is unset, overriding any
  signer-declared default so the wire suite is deterministic).
  - An existing `proof` member is **replaced**, never nested or appended.
  - The document must carry an in-band `issuer` equal to the DID of the
    signer's `verificationMethod`; the SPEC §4.7/§4.8 issuer binding the
    verifier enforces is pre-flighted at sign time (`SignError::MissingIssuer`
    / `SignError::IssuerMismatch`), so emitted documents verify with the
    stock `Verifier` by construction.
- `affinidi::SignError` — error taxonomy for the above.
- Re-exports so producers need no direct upstream dep:
  `affinidi::AffinidiSigner` (the upstream `Signer` trait),
  `affinidi::SignOptions`, `affinidi::CryptoSuite`.
- `tests/sign_round_trip.rs` — sign with a `did:key` secret and verify
  with the crate's own stock `Verifier`; option pass-through
  (proofPurpose, explicit cryptosuite incl. RDFC with `@context`);
  deterministic JCS default against a signer declaring RDFC; replace
  semantics for already-signed documents; sign-time issuer-binding
  rejections.

## [0.1.2] — 2026-05-27

### Changed

- Track `trust-tasks-rs` 0.1.2. No public API changes in this crate;
  bump-only release so downstream proof-verifier consumers can `cargo
  update -p trust-tasks-proof` in lockstep with the trust-tasks
  workspace.

## [0.1.0] — initial pre-release, tracks `SPEC.md` 0.1

Renamed from `trust-tasks-proof-affinidi` and restructured as an
umbrella crate with feature-gated backends. The Affinidi-backed
verifier now lives under `trust_tasks_proof::affinidi::Verifier`,
gated by the (default-enabled) `affinidi` Cargo feature.

### Added — crate scaffold

- `default = ["affinidi"]` Cargo feature; disable default features
  for a no-deps umbrella ready to receive other backends.
- `trust_tasks_proof::affinidi` module — gated by the `affinidi`
  feature; rehomes everything that was at the crate root in
  `trust-tasks-proof-affinidi` 0.1.0.

### Added — `affinidi` backend (renamed from `trust-tasks-proof-affinidi`)

- `affinidi::Verifier` (was `AffinidiProofVerifier`) —
  `trust_tasks_rs::ProofVerifier` implementation backed by
  `affinidi-data-integrity` 0.6. Supports `eddsa-rdfc-2022` and
  `eddsa-jcs-2022` cryptosuites out of the box; tracks whatever
  cryptosuites the upstream crate adds via its feature flags.
- `affinidi::Verifier::for_did_key()` — purely-local resolver wrapper
  around `DidKeyResolver`, suitable for self-issued documents and tests.
- `affinidi::Verifier::with_resolver(Arc<dyn VerificationMethodResolver>)`
  — plug in a custom resolver (e.g. for `did:web` or `did:webvh`).
- `affinidi::CachedDidResolver` — wraps
  `affinidi_did_resolver_cache_sdk::DIDCacheClient`, resolves the DID,
  walks the resulting `Document` for a matching `verificationMethod`,
  decodes its `publicKeyMultibase` via `decode_multikey_with_codec`,
  and maps the multicodec to `KeyType` (Ed25519, X25519, P-256, P-384,
  secp256k1). JWK-bearing verification methods surface a typed
  `Resolver` error so callers can stack a custom resolver in front.
- Error mapping from `DataIntegrityError` into the framework's
  `VerificationError` taxonomy — every failure mode lines up with the
  SPEC §8.3 `proof_invalid` standard code path.
- `tests/round_trip.rs` — happy path sign/verify, tampered payload,
  missing proof, unresolvable verification method.
- `tests/cached_resolver.rs` — local-mode `DIDCacheClient` resolves a
  `did:key`, the wrapper verifies the proof end-to-end; unresolvable
  DID does not return `SignatureInvalid` (which would mis-signal
  forgery for a resolver failure).

### Migration from `trust-tasks-proof-affinidi`

```toml
# before
trust-tasks-proof-affinidi = { ... }

# after
trust-tasks-proof = { ... }   # `affinidi` feature is on by default
```

```rust
// before
use trust_tasks_proof_affinidi::{AffinidiProofVerifier, CachedDidResolver};
let verifier = AffinidiProofVerifier::for_did_key();

// after
use trust_tasks_proof::affinidi::{CachedDidResolver, Verifier};
let verifier = Verifier::for_did_key();
```

[0.1.0]: https://github.com/trustoverip/dtgwg-trust-tasks-tf/releases/tag/v0.1.0
