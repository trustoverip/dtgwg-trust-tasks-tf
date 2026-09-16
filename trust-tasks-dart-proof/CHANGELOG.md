# Changelog — `trust_tasks_proof`

All notable changes to the Dart Data Integrity proof package.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against.
Below 1.0 a breaking change bumps the leading non-zero component.

Publishing is triggered by the `trust-tasks-dart-proof-v<version>` tag, because
pub.dev only accepts an automated publish from a tag-triggered workflow. See
`RELEASING.md`.

## 0.1.0

### Added

- Initial release: `DataIntegrityProofVerifier`, a `ProofVerifier` for
  `package:trust_tasks` verifying `eddsa-jcs-2022` and `ecdsa-jcs-2019` proofs
  with issuer binding, and `signTrustTask`, which produces them; backed by
  Affinidi's `package:ssi`.
