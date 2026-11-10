# Changelog — `trust_tasks_proof`

All notable changes to the Dart Data Integrity proof package.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against.
Below 1.0 a breaking change bumps the leading non-zero component.

Publishing is triggered by the `trust-tasks-dart-proof-v<version>` tag, because
pub.dev only accepts an automated publish from a tag-triggered workflow. See
`RELEASING.md`.

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
