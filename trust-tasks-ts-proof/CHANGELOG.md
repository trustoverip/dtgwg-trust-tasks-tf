# Changelog — `@openvtc/trust-tasks-proof`

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The
package versions over its own API. Below 1.0 a breaking change bumps the leading
non-zero component. Publishing is triggered by the
`trust-tasks-ts-proof-v<version>` tag; see `RELEASING.md`.

## 0.1.1 — 2026-09-21

## 0.1.0

### Added

- Initial release: `DataIntegrityProofVerifier`, a `ProofVerifier` for
  `@openvtc/trust-tasks` verifying `eddsa-jcs-2022` and `ecdsa-jcs-2019` with
  issuer binding, and `signTrustTask`, which produces what it verifies. On
  `@noble` — no JSON-LD, byte-compatible with the Rust and Dart proof libraries.
