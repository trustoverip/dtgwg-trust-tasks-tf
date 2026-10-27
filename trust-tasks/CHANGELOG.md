# Changelog

## [0.35.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.35.3...trust-tasks-v0.35.4) — 2026-10-27


### Added

- The prebuilt specification modules are deprecated in all four libraries; generate the tasks you use ([#954](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/954))

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



### Documentation

- GETTING-STARTED pins and generates the task it uses, in all four languages ([#955](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/955))

The walkthroughs taught the prebuilt specification modules, which #954
  deprecated. They now start where an application does: `trust-tasks add
  acl/grant/0.2`, then each language's generator.

  - §1 is "pin the tasks you use, then one dependency line": the CLI, the
    facade with `default-features = false` plus a build.rs that generates
    through it (`runtime_path("::trust_tasks")`), and `trust-tasks generate
    --lang ts|go|dart` for the others. The facade line said version "0.1"; it is
    0.35. The advice to trim the 26 per-family features is gone with them.
  - §2's Rust is still extracted verbatim from a runnable example, now an
    application of its own, examples/getting-started (trust-tasks.lock pinning
    acl/grant/0.2, its vendored bundle, a build.rs), instead of an example of
    the facade crate, which cannot have a build script per example. The only
    code change is the `use` line: `tasks::acl::grant::v0_2`, generated, for
    `trust_tasks::specs::acl::grant::v0_2`. It runs end to end
    (`cargo run -p trust-tasks-example-getting-started`), and
    getting_started_snippets follows it.
  - §3–3b (TypeScript, Go, Dart) import the runtime and the generated module
    instead of the package's prebuilt one, and move from acl/grant 0.1, which is
    superseded, to 0.2, as the Rust uses. Every name in them was checked against
    generated 0.2 code and the consumer example apps.

  The facade's README and crate docs point at the new example, and lose the
  dev-dependencies only the old one needed.



## [0.35.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.35.2...trust-tasks-v0.35.3) — 2026-10-26


## [0.35.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.35.1...trust-tasks-v0.35.2) — 2026-10-26


## [0.35.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.35.0...trust-tasks-v0.35.1) — 2026-10-26


## [0.35.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.34.2...trust-tasks-v0.35.0) — 2026-10-26


### Added

- **codegen**: Superseded and retired spec versions are deprecated in every library, and withdrawn ones leave them in a quarterly batch ([#921](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/921))

* feat(go): mark superseded and retired spec versions deprecated, and omit withdrawn ones

  Every generated request package now exports `const Deprecation` (empty for a
  current version, otherwise "superseded by <slug>/<M.N>", "retired; use
  <slug>/<M.N>" or "retired"), and a deprecated version's Payload and Response
  types carry a `// Deprecated:` paragraph. A version with `status: withdrawn`
  gets no package at all. The rule is derived independently from front matter,
  per SPEC "Superseded and Withdrawn Versions".



## [0.34.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.34.1...trust-tasks-v0.34.2) — 2026-10-24


## [0.34.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.34.0...trust-tasks-v0.34.1) — 2026-10-23


## [0.34.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.33.2...trust-tasks-v0.34.0) — 2026-10-23


## [0.33.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.33.1...trust-tasks-v0.33.2) — 2026-10-22


## [0.33.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.33.0...trust-tasks-v0.33.1) — 2026-10-21


## [0.33.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.32.10...trust-tasks-v0.33.0) — 2026-10-21


## [0.32.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.32.9...trust-tasks-v0.32.10) — 2026-10-21


## [0.32.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.32.8...trust-tasks-v0.32.9) — 2026-10-21


## [0.32.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.32.7...trust-tasks-v0.32.8) — 2026-10-21


## [0.32.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.32.6...trust-tasks-v0.32.7) — 2026-10-20


## [0.32.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.32.5...trust-tasks-v0.32.6) — 2026-10-20


## [0.32.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.32.4...trust-tasks-v0.32.5) — 2026-10-20


## [0.32.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.32.3...trust-tasks-v0.32.4) — 2026-10-19


## [0.32.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.32.2...trust-tasks-v0.32.3) — 2026-10-19


## [0.32.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.32.1...trust-tasks-v0.32.2) — 2026-10-19


## [0.32.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.32.0...trust-tasks-v0.32.1) — 2026-10-19


## [0.32.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.31.16...trust-tasks-v0.32.0) — 2026-10-18


## [0.31.16](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.31.15...trust-tasks-v0.31.16) — 2026-10-18


## [0.31.15](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.31.14...trust-tasks-v0.31.15) — 2026-10-17


## [0.31.14](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.31.13...trust-tasks-v0.31.14) — 2026-10-17


## [0.31.13](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.31.12...trust-tasks-v0.31.13) — 2026-10-16


## [0.31.12](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.31.11...trust-tasks-v0.31.12) — 2026-10-16


## [0.31.11](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.31.10...trust-tasks-v0.31.11) — 2026-10-16


## [0.31.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.31.9...trust-tasks-v0.31.10) — 2026-10-16


## [0.31.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.31.8...trust-tasks-v0.31.9) — 2026-10-16


## [0.31.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.31.7...trust-tasks-v0.31.8) — 2026-10-15


## [0.31.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.31.6...trust-tasks-v0.31.7) — 2026-10-15


## [0.31.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.31.5...trust-tasks-v0.31.6) — 2026-10-15


## [0.31.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.31.4...trust-tasks-v0.31.5) — 2026-10-15


## [0.31.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.31.3...trust-tasks-v0.31.4) — 2026-10-15


## [0.31.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.31.2...trust-tasks-v0.31.3) — 2026-10-15


## [0.31.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.31.1...trust-tasks-v0.31.2) — 2026-10-14


## [0.31.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.31.0...trust-tasks-v0.31.1) — 2026-10-14


## [0.31.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.30.2...trust-tasks-v0.31.0) — 2026-10-13


## [0.30.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.30.1...trust-tasks-v0.30.2) — 2026-10-13


## [0.30.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.30.0...trust-tasks-v0.30.1) — 2026-10-12


## [0.30.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.29.0...trust-tasks-v0.30.0) — 2026-10-11


## [0.29.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.28.1...trust-tasks-v0.29.0) — 2026-10-11


## [0.28.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.28.0...trust-tasks-v0.28.1) — 2026-10-11


## [0.28.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.27.8...trust-tasks-v0.28.0) — 2026-10-11


## [0.27.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.27.7...trust-tasks-v0.27.8) — 2026-10-10


## [0.27.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.27.6...trust-tasks-v0.27.7) — 2026-10-05


## [0.27.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.27.5...trust-tasks-v0.27.6) — 2026-10-05


## [0.27.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.27.4...trust-tasks-v0.27.5) — 2026-10-05


## [0.27.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.27.3...trust-tasks-v0.27.4) — 2026-10-03


## [0.27.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.27.2...trust-tasks-v0.27.3) — 2026-10-03


## [0.27.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.27.1...trust-tasks-v0.27.2) — 2026-10-03


## [0.27.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.27.0...trust-tasks-v0.27.1) — 2026-10-03


## [0.27.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.26.3...trust-tasks-v0.27.0) — 2026-10-03


## [0.26.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.26.2...trust-tasks-v0.26.3) — 2026-10-02


## [0.26.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.26.1...trust-tasks-v0.26.2) — 2026-10-02


## [0.26.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.26.0...trust-tasks-v0.26.1) — 2026-10-02


## [0.26.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.25.4...trust-tasks-v0.26.0) — 2026-10-01


## [0.25.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.25.3...trust-tasks-v0.25.4) — 2026-10-01


## [0.25.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.25.2...trust-tasks-v0.25.3) — 2026-10-01


## [0.25.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.25.1...trust-tasks-v0.25.2) — 2026-10-01


## [0.25.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.25.0...trust-tasks-v0.25.1) — 2026-09-30


## [0.25.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.24.9...trust-tasks-v0.25.0) — 2026-09-30


## [0.24.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.24.8...trust-tasks-v0.24.9) — 2026-09-30


## [0.24.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.24.7...trust-tasks-v0.24.8) — 2026-09-29


## [0.24.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.24.6...trust-tasks-v0.24.7) — 2026-09-29


## [0.24.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.24.5...trust-tasks-v0.24.6) — 2026-09-28


## [0.24.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.24.4...trust-tasks-v0.24.5) — 2026-09-28


## [0.24.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.24.3...trust-tasks-v0.24.4) — 2026-09-28


## [0.24.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.24.2...trust-tasks-v0.24.3) — 2026-09-28


## [0.24.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.24.1...trust-tasks-v0.24.2) — 2026-09-27


## [0.24.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.24.0...trust-tasks-v0.24.1) — 2026-09-27


## [0.24.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.23.6...trust-tasks-v0.24.0) — 2026-09-27


## [0.23.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.23.5...trust-tasks-v0.23.6) — 2026-09-27


## [0.23.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.23.4...trust-tasks-v0.23.5) — 2026-09-27


## [0.23.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.23.3...trust-tasks-v0.23.4) — 2026-09-27


## [0.23.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.23.2...trust-tasks-v0.23.3) — 2026-09-26


## [0.23.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.23.1...trust-tasks-v0.23.2) — 2026-09-26


## [0.23.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.23.0...trust-tasks-v0.23.1) — 2026-09-26


## [0.23.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.22.7...trust-tasks-v0.23.0) — 2026-09-26


## [0.22.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.22.6...trust-tasks-v0.22.7) — 2026-09-24


## [0.22.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.22.5...trust-tasks-v0.22.6) — 2026-09-24


## [0.22.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.22.4...trust-tasks-v0.22.5) — 2026-09-24


## [0.22.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.22.3...trust-tasks-v0.22.4) — 2026-09-24


## [0.22.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.22.2...trust-tasks-v0.22.3) — 2026-09-23


## [0.22.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.22.1...trust-tasks-v0.22.2) — 2026-09-23


## [0.22.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.22.0...trust-tasks-v0.22.1) — 2026-09-23


## [0.22.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.21...trust-tasks-v0.22.0) — 2026-09-23


## [0.21.21](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.20...trust-tasks-v0.21.21) — 2026-09-22


## [0.21.20](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.19...trust-tasks-v0.21.20) — 2026-09-22


## [0.21.19](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.18...trust-tasks-v0.21.19) — 2026-09-22


## [0.21.18](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.17...trust-tasks-v0.21.18) — 2026-09-22


## [0.21.17](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.16...trust-tasks-v0.21.17) — 2026-09-22


## [0.21.16](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.15...trust-tasks-v0.21.16) — 2026-09-22


## [0.21.15](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.14...trust-tasks-v0.21.15) — 2026-09-22


## [0.21.14](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.12...trust-tasks-v0.21.14) — 2026-09-22


## [0.21.13](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.12...trust-tasks-v0.21.13) — 2026-09-22


## [0.21.12](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.11...trust-tasks-v0.21.12) — 2026-09-21


## [0.21.11](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.10...trust-tasks-v0.21.11) — 2026-09-21


## [0.21.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.9...trust-tasks-v0.21.10) — 2026-09-21


## [0.21.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.8...trust-tasks-v0.21.9) — 2026-09-21


## [0.21.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.7...trust-tasks-v0.21.8) — 2026-09-21


## [0.21.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.6...trust-tasks-v0.21.7) — 2026-09-21


## [0.21.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.5...trust-tasks-v0.21.6) — 2026-09-21


## [0.21.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.4...trust-tasks-v0.21.5) — 2026-09-20


## [0.21.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.3...trust-tasks-v0.21.4) — 2026-09-19


## [0.21.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.2...trust-tasks-v0.21.3) — 2026-09-17


## [0.21.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.1...trust-tasks-v0.21.2) — 2026-09-16


## [0.21.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.21.0...trust-tasks-v0.21.1) — 2026-09-16


## [0.21.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.20.7...trust-tasks-v0.21.0) — 2026-09-16


## [0.20.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.20.6...trust-tasks-v0.20.7) — 2026-09-15


## [0.20.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.20.5...trust-tasks-v0.20.6) — 2026-09-15


## [0.20.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.20.4...trust-tasks-v0.20.5) — 2026-09-14


## [0.20.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.20.3...trust-tasks-v0.20.4) — 2026-09-11


## [0.20.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.20.2...trust-tasks-v0.20.3) — 2026-09-10


## [0.20.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.20.1...trust-tasks-v0.20.2) — 2026-09-10


## [0.20.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.20.0...trust-tasks-v0.20.1) — 2026-09-10


## [0.20.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.19.4...trust-tasks-v0.20.0) — 2026-09-10


## [0.19.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.19.3...trust-tasks-v0.19.4) — 2026-09-10


## [0.19.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.19.2...trust-tasks-v0.19.3) — 2026-09-09


## [0.19.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.19.1...trust-tasks-v0.19.2) — 2026-09-09


## [0.19.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.19.0...trust-tasks-v0.19.1) — 2026-09-09


## [0.19.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.18.11...trust-tasks-v0.19.0) — 2026-09-09


## [0.18.11](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.18.10...trust-tasks-v0.18.11) — 2026-09-09


## [0.18.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.18.9...trust-tasks-v0.18.10) — 2026-09-09


## [0.18.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.18.8...trust-tasks-v0.18.9) — 2026-09-09


## [0.18.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.18.7...trust-tasks-v0.18.8) — 2026-09-08


## [0.18.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.18.6...trust-tasks-v0.18.7) — 2026-09-08


## [0.18.6](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.18.5...trust-tasks-v0.18.6) — 2026-09-08


## [0.18.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.18.4...trust-tasks-v0.18.5) — 2026-09-08


## [0.18.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.18.3...trust-tasks-v0.18.4) — 2026-09-07


## [0.18.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.18.2...trust-tasks-v0.18.3) — 2026-09-07


## [0.18.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.18.1...trust-tasks-v0.18.2) — 2026-09-07


## [0.18.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.18.0...trust-tasks-v0.18.1) — 2026-09-06


## [0.18.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.17.10...trust-tasks-v0.18.0) — 2026-09-06


## [0.17.10](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.17.9...trust-tasks-v0.17.10) — 2026-09-05


## [0.17.9](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.17.8...trust-tasks-v0.17.9) — 2026-09-05


## [0.17.8](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.17.7...trust-tasks-v0.17.8) — 2026-09-04


## [0.17.7](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.17.5...trust-tasks-v0.17.7) — 2026-09-02


## [0.17.5](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.17.4...trust-tasks-v0.17.5) — 2026-09-02


## [0.17.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.17.3...trust-tasks-v0.17.4) — 2026-09-01


## [0.17.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.17.2...trust-tasks-v0.17.3) — 2026-08-28


## [0.17.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.17.1...trust-tasks-v0.17.2) — 2026-08-28


## [0.17.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.17.0...trust-tasks-v0.17.1) — 2026-08-27


## [0.17.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-v0.2.0...trust-tasks-v0.17.0) — 2026-08-27


### Changed

- **versioning**: Release the trust-tasks-rs-exposing crates in lockstep ([#315](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/315))


### Specifications

- Bound every free-text payload member with a maxLength (§7.3) ([#296](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/296))

* spec: bound every free-text payload member with a maxLength

  SPEC.md §7.3 (framework 0.5.0) requires that any member holding free
  text declare a `maxLength`. 92 free-text string members across 83 draft
  schemas carried none, leaving the wire contract unbounded and every
  consumer to invent its own ceiling — or none, which is what §10.3
  (schema-validation DoS) exists to prevent.

  Bounds are chosen per member from the vocabulary the registry already
  uses rather than applied uniformly:

    256   `label`, `comment` — a display name or an OpenSSH key comment;
          matches the existing 256 on provision/integration `label` and
          the `name` members alongside it.
    500   requester-authored prose that a surface renders to a human who
          is deciding something; matches task-consent/request/0.1 `note`,
          the registry's considered consent-surface bound.
    1024  `reason`, `description`, `message` — operator or service prose
          recorded for audit or returned as a diagnostic; matches the six
          existing `reason: 1024` and the `description: 1024` in policy/
          and vtc/endorsement-type.
    16384 chat/message `text` — the task's actual content rather than
          metadata about it; matches the corpus's long-form bound on
          vault `secureNotes`.

  All amended specifications are `status: draft`, so the change is made in
  place per SPEC §5.2. Deliberately untouched:

    * 17 members in `retired` specifications, frozen by SPEC §6.4.
    * messaging/_shared/0.1 `AuditEntry.detail` and did-management/
      _shared/0.1 `DomainEntry.label` — shared $defs reachable from a
      retired specification, so bounding them would change a frozen
      specification's effective wire contract.
    * vault/_shared/{0.1,0.2,0.3} `TspMessageEnvelope.message` — opaque
      base64url TSP bytes, not free text.

  The `label` description in vault/_shared/*/vault-entry.schema.json said
  the wire spec enforced no maximum length. It now does, so the sentence
  is corrected rather than left contradicting the schema it annotates.

  `npm run validate` re-checks all 533 fenced example documents against
  the amended schemas; none is rejected by a new bound.



## [0.2.0] - 2026-08-26

### Changed

- **`trust-tasks-https` requirement moved to `0.16`.** That release makes
  `HttpsServer::on` require `RequestPayload` and adds `on_ack`; this crate
  re-exports the type, so the leading component moves with it. No change to
  this crate's own API.

