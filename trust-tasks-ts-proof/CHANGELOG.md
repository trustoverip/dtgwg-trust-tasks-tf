# Changelog — `@openvtc/trust-tasks-proof`

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The
package versions over its own API. Below 1.0 a breaking change bumps the leading
non-zero component. Publishing is triggered by the
`trust-tasks-ts-proof-v<version>` tag; see `RELEASING.md`.

## 0.2.2 — 2026-11-14

## 0.2.1 — 2026-11-13


### Fixed

- **rs,ts,go,dart,go-didcomm,go-tsp,ts-tsp**: Settle the open runtime parity rows (#1251)

* fix(rs,ts,go,dart): settle the open runtime parity rows

  Replay guard: a capacity below one takes the default capacity in all four
  libraries. Rust's InMemoryReplayGuard::new(0) no longer panics and the
  TypeScript constructor no longer throws RangeError; Go and Dart already
  defaulted. Moving the two to the default only relaxes them.

- **ts-proof,ts-tsp,ts-capability-client**: Require @openvtc/trust-tasks 0.29 (#1237)

## 0.2.0 — 2026-11-12


### Documentation

- Fix drifted counts, release runbook, licences and stale examples across the four libraries (#1224)

* docs(getting-started): move the worked example to acl/grant/0.3 and stamp issuedAt with issued_at_now()

  The example pinned acl/grant/0.2, superseded by 0.3, and stamped issuedAt
  with chrono::Utc::now(), the full-precision form #970 removed from the
  libraries. The TS consumer example wrote toISOString() milliseconds into
  document fields the same way. The dependency snippet now uses cargo add,
  so it cannot name a stale version, and §4.4 describes the shared version
  group instead of versions from 0.14.



### Fixed

- **proof,ts-proof,go-proof,dart-proof**: Follow the JCS cryptosuites' @context rule in all four proof backends (#1233)

Signers copy a document's @context into the proof (eddsa-jcs-2022 /
  ecdsa-jcs-2019 section 3.3.1 step 2). Verifiers hash the proof
  configuration as received, and refuse a proof whose @context is not
  exactly the document's (section 3.3.2 step 4, held to equality because a
  ProofVerifier returns no verified document). The Dart verifier no longer
  lets ssi copy the document's @context into the configuration, which made
  it refuse what the other three signers emit. Documents without @context
  get byte-identical proofs. Six shared vectors in fixtures/proof pin the
  behaviour in every library.

- **ts,go,dart,rs**: Whole-second capability-client issuedAt, tested Dart core floors, TS default export condition, Go schema caching (#1214)

* fix(dart-capability-client,go-capabilityclient,go-didcomm): stamp issuedAt in whole seconds and fail on entropy failure

  The Dart capability client stamped issuedAt with toIso8601String (full
  precision); it now uses the core's systemClock. The Go port now stamps
  through trusttasks.SystemClock. The TS port already stamped whole seconds.

  The Go capabilityclient and didcomm UUIDv4 generators ignored crypto/rand
  errors and would mint an all-zero id; they now panic, as tsp's does.

- **rs,ts,go,dart,proof,ts-proof,go-proof,dart-proof**: Refuse out-of-range timestamp fields instead of rolling them over (#1210)

issuedAt and expiresAt, and a proof's created, are now read with every
  field range-checked after the shared pattern: month 1-12, a day the month
  has, hour 0-23, minute and second 0-59 (no leap second), and an offset of
  at most 23:59 (14:00 for a proof's dateTimeStamp, as for expires).

  Before, the four runtimes disagreed on the same bytes. TypeScript's
  Date.parse read 2026-02-30 as 2 March and 24:00 as the next day; Dart's
  DateTime.tryParse rolled 2026-13-45T25:61:61Z over into 2027 and took any
  offset; Rust's chrono read :60 as a leap second and accepted a space for
  the T; Go's time.Parse accepted +24:00 and +23:60. In the TypeScript proof
  verifier a regex-valid but unparseable created (month 13) gave NaN, which
  never compares later than the clock-skew horizon, so it was accepted.

  New shared vectors: fixtures/consume/timestamps.json, run by all four
  runtimes, and five correctly signed created-* proof vectors.

- **go-proof, ts-proof, dart-proof**: Check proofPurpose and expires, and sign with authentication by default (#1168)

* fix(go-proof, ts-proof, dart-proof)!: check proofPurpose and expires, and sign with authentication by default

  The Go, TypeScript and Dart proof verifiers accepted any non-empty
  proofPurpose, including keyAgreement, and none of the four read a proof's
  expires. They now apply the rule trust-tasks-proof (Rust) already applies
  (SPEC §4.7.3):

  - proofPurpose must be authentication, assertionMethod,
    capabilityInvocation or capabilityDelegation; keyAgreement and any other
    value are malformedProof.
  - Dart's withResolver path checks that the issuer's DID document lists the
    exact verificationMethod under the relationship proofPurpose names (by
    absolute reference, relative fragment, or embedded), defines it under
    verificationMethod, and names the issuer as its controller. The document
    is resolved once per verification and shared with ssi.
  - expires, when present, must be a dateTimeStamp (Z or an explicit offset)
    that has not passed; no clock-skew allowance. All four runtimes.
  - The Go, TypeScript and Dart signers default proofPurpose to
    authentication, as the Rust signer does, and refuse a purpose naming no
    signing relationship.
  - TypeScript verifies Ed25519 with RFC 8032 decoding ({ zip215: false }),
    as Go, Rust and Dart do.

  Three shared vectors (fixtures/proof) are correctly signed and must fail in
  every runtime: proofPurpose keyAgreement, an unrecognised proofPurpose, and
  a passed expires.

- **proof, ts-proof, dart-proof, dart-didcomm, ceremony**: Bound multibase decoding of unauthenticated values (#1167)

* fix(proof, ts-proof, dart-proof): bound base58 decoding of proofValue and did:key

  The Rust, TypeScript and Dart proof verifiers decode a proof's proofValue
  and its did:key multikey with base58btc decoders whose cost grows with
  the square of the input length, and they do so before any signature is
  checked. A 1 MiB value took about 50 s per field in trust-tasks-proof
  (release build), and would take minutes in the Dart package and in the
  TypeScript package on @scure/base 2.0-2.3 (2.4 refuses over 4096
  characters itself).

  Each verifier now refuses a base58btc value longer than 175 characters
  before decoding it, or one that decodes to more than 128 bytes - the
  bounds the Go module adopts in #1166. The largest value any of them reads
  is a 96-byte P-384 signature; a did:key multikey is at most 51 bytes;
  175 = ceil(128 * log 256 / log 58).

  Nothing that could verify before is refused: a value over the bounds
  always decoded to more bytes than any supported key or signature.

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
