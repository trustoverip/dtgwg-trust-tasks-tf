# Changelog — `@openvtc/trust-tasks-gen`

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The
package versions over its own API. Below 1.0 a breaking change bumps the leading
non-zero component. Publishing is triggered by the
`trust-tasks-ts-gen-v<version>` tag; see `RELEASING.md`.

## 0.1.2 — 2026-10-28


### Fixed

- **ts-gen**: Qualify a bare component's nested branch types when their titles collide (#971)

A lock that reaches one shape of `ConsumerKind` (device/list 0.3,
  device/register 0.2) leaves it bare, so its `oneOf` branches keep their
  titles, `Companion` and `Service`. A lock that also pins auth/oob/claim 0.2
  or prove 0.3 has a shared `Service` of its own. json-schema-to-typescript
  then numbers the second `Service1`, and generation refuses the lock
  ("the components module declares counter-suffixed duplicates: Service1").

  The prebuilt package never meets this: over the whole registry
  `ConsumerKind` has two shapes and is qualified, so its branches are tagged
  `_DeviceV0_2`. Tag a bare component's nested titles the same way when any of
  them collides with a component name or with another bare component's
  nested title. Modules still bind the plain names (`Service_DeviceV0_2 as
  Service`), and `npm run equivalence` is unchanged.

## 0.1.1 — 2026-10-27


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



### Build & CI

- **release**: Give @openvtc/trust-tasks-gen the CHANGELOG.md its release bump writes into (#956)

release-bump-ts.sh prepends each release's section to <pkg>/CHANGELOG.md.
  trust-tasks-ts-gen was published by hand at 0.1.0 without one, so the first
  merge to touch it since (#954) made the Release PR job fail with ENOENT and
  left the Release PR (#949) stale.

## 0.1.0

- Initial release: `trust-tasks-gen-ts` generates TypeScript modules for the
  versions pinned in `trust-tasks.lock`, importing the runtime from
  `@openvtc/trust-tasks/runtime` (0.27.8 or later). Published by hand.
