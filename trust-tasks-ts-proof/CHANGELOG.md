# Changelog — `@openvtc/trust-tasks-proof`

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The
package versions over its own API. Below 1.0 a breaking change bumps the leading
non-zero component. Publishing is triggered by the
`trust-tasks-ts-proof-v<version>` tag; see `RELEASING.md`.

## 0.1.6 — 2026-10-28


### Fixed

- **ts-proof, ts-tsp, ts-capability-client**: Admit @openvtc/trust-tasks 0.28 (#965)

The core reached 0.28.0 in #963. The published siblings declare
  @openvtc/trust-tasks ^0.27.8 as a peer dependency, which npm reads as
  <0.28.0, so an application on the current core cannot install them.

  Raise the peer and dev ranges to ^0.28.0 and refresh the lockfiles
  against the published core. All three build, typecheck and pass their
  tests against it. Releasing this releases the three siblings.

## 0.1.5 — 2026-10-27

## 0.1.4 — 2026-10-27


### Fixed

- The TypeScript sibling packages share the application's core and load only its runtime (#947)

@openvtc/trust-tasks-proof, -tsp and -capability-client declared
  "@openvtc/trust-tasks": "^0.19.0" as a regular dependency. npm reads that as
  >=0.19.0 <0.20.0, so an application on core 0.27 that added any of them got a
  second core, 0.19.x, installed beside its own, with every prebuilt module in
  it; their own lockfiles still built and tested against 0.19.8. They also
  imported from the package root, which loads every prebuilt module, where only
  runtime types and functions were used.

  Each now takes the core as a peer dependency at ^0.27.8 (and a dev
  dependency at the same range, to build and test against) and imports from
  @openvtc/trust-tasks/runtime. All three build and pass their tests against
  0.27.8.

  scripts/check-sibling-core-ranges.mjs (npm run check-sibling-core-ranges, in
  ts.yml's `bindings match specs`) fails when a TypeScript sibling lists the
  core anywhere but peerDependencies, or when a TypeScript or Dart sibling's
  caret range does not admit the core version in the checkout. Both ecosystems
  had drifted that way (Dart until #895); on a Release PR that moves the core
  across a 0.x minor it fails until the siblings' floors are raised.

  This is milestone 4 for TypeScript: no hand-written package uses a generated
  specification module outside its tests, so depending on the runtime alone is
  the whole change. The Rust crates did it in #939; the Go and Dart siblings
  already import only the runtime (docs/design/consumer-side-generation.md).

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
