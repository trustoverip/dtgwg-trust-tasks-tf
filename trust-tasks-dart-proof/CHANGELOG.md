# Changelog — `trust_tasks_proof`

All notable changes to the Dart Data Integrity proof package.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against.
Below 1.0 a breaking change bumps the leading non-zero component.

Publishing is triggered by the `trust-tasks-dart-proof-v<version>` tag, because
pub.dev only accepts an automated publish from a tag-triggered workflow. See
`RELEASING.md`.

## 0.1.3 — 2026-10-23


### Fixed

- **dart**: The sibling packages require the current trust_tasks core (#895)

Every published sibling (trust_tasks_https, _proof, _didcomm, and the
  unreleased _tsp and _capability_client) declared `trust_tasks: ^0.1.0`,
  which pub reads as `>=0.1.0 <0.2.0`. A consumer therefore got core 0.1.19,
  missing every fix since, and an app already on trust_tasks 0.9.x could not
  add a sibling at all: the constraints conflict. tag-dart's release check
  did not notice, because it verifies against the newest published core the
  constraint admits. The floor is now ^0.9.3.

  That core releases the replay claim itself when a handler throws (#876),
  so the HTTPS server and the DIDComm and TSP consumers no longer release it
  a second time. Their release also fired when the guard failed to record a
  response after the handler had run, freeing a document whose effect had
  already happened; the core keeps the claim in that case, and so do they now.

  They also pass `responseToJson` (#878), so a persistent ReplayGuard caches
  the success response as JSON and a resend is answered with it.

## 0.1.2 — 2026-09-21

## 0.1.1 — 2026-09-16

## 0.1.0

### Added

- Initial release: `DataIntegrityProofVerifier`, a `ProofVerifier` for
  `package:trust_tasks` verifying `eddsa-jcs-2022` and `ecdsa-jcs-2019` proofs
  with issuer binding, and `signTrustTask`, which produces them; backed by
  Affinidi's `package:ssi`.
