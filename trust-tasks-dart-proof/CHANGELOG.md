# Changelog — `trust_tasks_proof`

All notable changes to the Dart Data Integrity proof package.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against.
Below 1.0 a breaking change bumps the leading non-zero component.

Publishing is triggered by the `trust-tasks-dart-proof-v<version>` tag, because
pub.dev only accepts an automated publish from a tag-triggered workflow. See
`RELEASING.md`.

## 0.2.2 — 2026-11-14

### Dependencies

- Requires `trust_tasks` ^0.12.0 (was ^0.11.0), the core released beside it.

## 0.2.1 — 2026-11-13


### Added

- **dart,dart-https,dart-didcomm**: JSON priorResponse, observability hook, forward routing and per-type size bounds (#1242)

* feat(dart,dart-https,dart-didcomm)!: JSON priorResponse, observability hook, forward routing and per-type size bounds



### Fixed

- **rs,ts,go,dart,go-didcomm,go-tsp,ts-tsp**: Settle the open runtime parity rows (#1251)

* fix(rs,ts,go,dart): settle the open runtime parity rows

  Replay guard: a capacity below one takes the default capacity in all four
  libraries. Rust's InMemoryReplayGuard::new(0) no longer panics and the
  TypeScript constructor no longer throws RangeError; Go and Dart already
  defaulted. Moving the two to the default only relaxes them.

### Dependencies

- Requires `trust_tasks` ^0.11.0 (was ^0.10.0), the core released beside it.

## 0.2.0 — 2026-11-12


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

### Dependencies

- Requires `trust_tasks` ^0.10.0 (was ^0.9.6), the core released beside it.

## 0.1.8 — 2026-11-10


### Added

- **acl**: Purpose of use on grants and on memory reads (AclEntry 0.3; vta/memory put/delete/list 0.3, get 0.2) (#1074)

* feat(acl): purpose of use on grants and on memory reads (AclEntry 0.3; vta/memory put/delete/list 0.3, get 0.2)

  An access-control entry says who may do what, where and until when, but not
  what for. An agent granted a read of its principal's memory can use the content
  for anything, and nothing records the use it intended. The VTI specification
  now requires the purpose (VTI-ACL-100 – 110, trustoverip/dtgwg-vti-spec#115);
  these are the wire shapes.

  acl/_shared/0.3 — AclEntry 0.3 adds one REQUIRED axis, `purposes`:
  `{"scope": "any"}` or `{"scope": "listed", "purposes": [...]}` with at least
  one `Purpose` — a lowerCamelCase name registered in the VTI purpose registry
  (personalisation, taskExecution, analytics, modelTraining) or an absolute IRI
  such as a DPV purpose. `any` never grants `modelTraining`. Delegated purposes
  are a subset of the granter's. CONVENTIONS.md §8a maps a 0.2 entry onto 0.3
  (`any`, the authority it already held, now stated) and §12 states the rules.

  acl tasks on AclEntry 0.3: grant/0.3, update/0.3, show/0.3, revoke/0.3,
  change-role/0.3, list/0.4. New errors on grant and update: `purposeOutsideDelegator`
  (purposes wider than the granter's) and `purposeRefused` (policy will not grant
  a purpose, refused by name, never dropped). update/0.3 replaces `purposes`
  wholesale; narrowing it is an audited privilege reduction. `purposes` is never
  redacted by show or list. No `extends`: a REQUIRED member on the entry cannot
  be added to a version a 0.2 consumer still reads.

  vta/memory put/0.3, delete/0.3, list/0.3, get/0.2 (each `extends` its
  predecessor) add an optional `purpose` (`MemoryPurpose`, new in
  vta/_shared/0.1/memory-record, a reference to acl's `Purpose`) and a
  `purposeNotGranted` error that discloses nothing and is distinct from
  `contextForbidden`. A recall, and a listing with values, is purpose-bound:
  refused for a purpose the entry does not grant, or for none under an entry
  listing several, and audited with its purpose either way. A write is not
  purpose-bound, but a conflict that returns `currentValue` is: the value is
  withheld unless the request's purpose is granted.

  Data rooms (a room authority credential's `purposes`, VTI-ACL-110) are a
  follow-up.

  Samples in other specs that named the superseded acl versions now name the new
  ones, with `purposes` on every embedded entry; prose links were moved by
  `npm run fix-stale-references`.

## 0.1.7 — 2026-10-27

## 0.1.6 — 2026-10-27


### Added

- The prebuilt specification modules are deprecated in all four libraries; generate the tasks you use (#954)

Milestone 5 of docs/design/consumer-side-generation.md. Every prebuilt module
  outside the framework's own specifications (trust-task-*, trust-ceremony-*,
  _framework/) is deprecated, so that an application is told to pin the tasks it
  uses with the trust-tasks CLI and generate them instead. Nothing is removed:
  the withdrawal batch that removes them is chosen later, at least 90 days after
  this ships.

  Each library marks the prebuilt entry point where its toolchain reports a
  deprecation, never the module body, which is also what the consumer-side
  generators emit:

  - Rust: #[deprecated] on each gated family module in trust-tasks-rs's
    specs/mod.rs (trust-tasks-codegen). It reaches every path and item under the
    module; framework modules and trust-tasks-build's output are unmarked.
  - TypeScript: the root index re-exports each module as
    `import * as _X …; /** @deprecated … */ export import X = _X;`, because
    TypeScript ignores @deprecated on `export * as`. API compatibility against
    0.27.8: nothing removed or changed. A deep import of a module file is not
    marked; TypeScript has no module-level deprecation.
  - Go: a `Deprecated:` paragraph in each prebuilt package's doc, which gopls and
    staticcheck (SA1019) report on import.
  - Dart: @Deprecated on each prebuilt library's `library` directive, which the
    analyzer reports on import from another package. This package's own tests
    use them, so analysis_options ignores the same-package diagnostic SDKs
    before 3.4 report.

  The TS, Go and Dart equivalence tests mask exactly the mark; every consumer
  generator is still held to the prebuilt tree as before. Hand-written Rust that
  uses prebuilt modules on purpose — the acl-specimen test modules, the
  integration tests and the examples — carries #[allow(deprecated)]; none of the
  runtime does.

  GETTING-STARTED, the four library READMEs, the design doc and CLAUDE.md say
  so, and point at each language's generator.

## 0.1.5 — 2026-10-26

## 0.1.4 — 2026-10-26


### Added

- **codegen**: Superseded and retired spec versions are deprecated in every library, and withdrawn ones leave them in a quarterly batch (#921)

* feat(go): mark superseded and retired spec versions deprecated, and omit withdrawn ones

  Every generated request package now exports `const Deprecation` (empty for a
  current version, otherwise "superseded by <slug>/<M.N>", "retired; use
  <slug>/<M.N>" or "retired"), and a deprecated version's Payload and Response
  types carry a `// Deprecated:` paragraph. A version with `status: withdrawn`
  gets no package at all. The rule is derived independently from front matter,
  per SPEC "Superseded and Withdrawn Versions".

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
