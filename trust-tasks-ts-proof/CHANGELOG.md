# Changelog — `@openvtc/trust-tasks-proof`

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The
package versions over its own API. Below 1.0 a breaking change bumps the leading
non-zero component. Publishing is triggered by the
`trust-tasks-ts-proof-v<version>` tag; see `RELEASING.md`.

## 0.1.3 — 2026-10-22


### Fixed

- **trust-tasks-ts-proof**: Ecdsa-jcs-2019 hashes once and accepts high-S, so P-256/P-384 proofs interoperate (#874)

signTrustTask passed the suite digest of hashData to @noble/curves, whose
  `prehash` defaults to true, so it hashed again: TS ECDSA proofs were
  signatures over SHA(SHA(hashData)), which verified only against this
  package. The verifier made the same mistake, so it rejected every standard
  ecdsa-jcs-2019 proof — including the Go backend's — and its default `lowS`
  would also have refused the high-S signatures Go's ecdsa.Sign emits about
  half the time.

  Sign and verify now pass `prehash: false` (the digest is the ECDSA message
  hash), and verify passes `lowS: false`. Ed25519 is unaffected.

## 0.1.2 — 2026-10-01

## 0.1.1 — 2026-09-21

## 0.1.0

### Added

- Initial release: `DataIntegrityProofVerifier`, a `ProofVerifier` for
  `@openvtc/trust-tasks` verifying `eddsa-jcs-2022` and `ecdsa-jcs-2019` with
  issuer binding, and `signTrustTask`, which produces what it verifies. On
  `@noble` — no JSON-LD, byte-compatible with the Rust and Dart proof libraries.
