# Changelog — `trust_tasks_https`

All notable changes to the Dart HTTPS binding package.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against.
Below 1.0 a breaking change bumps the leading non-zero component.

Publishing is triggered by the `trust-tasks-dart-https-v<version>` tag, because
pub.dev only accepts an automated publish from a tag-triggered workflow. See
`RELEASING.md`.

## 0.1.7 — 2026-10-27


### Added

- **ts, go**: Trust-task-error/0.6 and the unsupportedExtension and indeterminate standard codes (#961)

* feat(ts)!: trust-task-error/0.6 and the unsupportedExtension and indeterminate standard codes

  Framework 0.7.0 added unsupportedExtension to the standard codes, and the
  indeterminate ruling (SPEC §8.3) adds indeterminate: the recipient could
  not decide because something it needed and does not control could not
  be obtained or checked, which is neither a refusal nor a failed
  verification. Both live in the closed code list of the trust-task-error
  payload schema, so they need a new version.

  - specs/trust-task-error/0.6: both codes appended to the enum, and
    extends: "0.5", which the build checks. References to 0.5 across the
    registry therefore stay current, and no other spec changes.
  - All four libraries recognize both codes and emit 0.6:
    trust_task_error_type_uri(), TRUST_TASK_ERROR_TYPE_URI,
    TrustTaskErrorTypeURI and trustTaskErrorTypeUri move together.
    Rust's StandardCode is hand-written (trust-task-error is in
    SKIP_SLUGS) and gains UnsupportedExtension and Indeterminate, with
    indeterminate retryable by default as §8.3 gives it.
  - The HTTPS binding's status table maps unsupportedExtension to 422 and
    indeterminate to 503, in the binding prose, trust-tasks-https and
    trust_tasks_https. The Dart package spells the new codes out so it
    still builds against a core that predates them.

  Breaking for @openvtc/trust-tasks only: StandardCode is a closed union
  there, so an exhaustive switch over it stops compiling. Rust's
  StandardCode is #[non_exhaustive] and Go's and Dart's are open string
  types, so the change is additive for them.

  Part of trustoverip/dtgwg-trust-tasks-spec#42.

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

## 0.1.2 — 2026-10-23

## 0.1.1 — 2026-09-21

## 0.1.0

### Added

- Initial release: `HttpsClient` and a framework-agnostic `HttpsServer` for the
  Trust Tasks HTTPS binding (`bindings/https/0.2`), with a `dart:io` adapter in
  `package:trust_tasks_https/io.dart`.
