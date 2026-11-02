# Changelog

Notable changes to the published packages. Generated from conventional commits
by [git-cliff](https://git-cliff.org) when a release is cut — do not edit by
hand.
## [0.1.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-cli-v0.1.2...trust-tasks-cli-v0.1.3) — 2026-10-30


## [0.1.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-cli-v0.1.1...trust-tasks-cli-v0.1.2) — 2026-10-26


### Added

- **cli**: Trust-tasks generate --lang runs a language's generator over a verified lock ([#944](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/944))

`trust-tasks generate --lang <rs|ts|go|dart> --out <dir> [-- <args>]` runs
  `check` and, only if the lock verifies, that language's generator from the
  project directory: `npx --no-install trust-tasks-gen-ts`, `go run
  …/trust-tasks-go/cmd/trust-tasks-gen` or `dart run trust_tasks:generate`, with
  absolute --lock and --out and anything after `--` passed on. It installs
  nothing; a missing tool is an error that says what to add. `--lang rs` checks
  and points at build.rs, where Rust generates.

  The TypeScript example had a hand-written lock and no trust-tasks.toml, and the
  Dart example's manifest did not name the third-party task its lock pins, so
  neither verified; both manifests now match their locks. Run through the new
  command, the Go and TypeScript examples regenerate byte-identically and the
  Dart one generates and runs.



## [0.1.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-cli-v0.1.0...trust-tasks-cli-v0.1.1) — 2026-10-26


### Added

- **cli, build, rs**: Pin and generate tasks from third-party sources beside the registry's ([#934](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/934))

* feat(cli, build, rs): pin and generate tasks from third-party sources beside the registry's

  SPEC §6.1/§6.5 already allow specifications under another authority, with
  a slug unique only within its authority. An application can now pin them
  from a third-party registry and generate them with the official ones.

  trust-tasks-cli
  - trust-tasks.toml gains [sources.<name>] (url = "https://…" or
    path = "<dir>"); tasks are named "<source>:<slug>", and on the command
    line `trust-tasks add acme:payments/settle/1.0`. add, update, check,
    fetch, list and remove all take sourced tasks; a source serves the
    trusttasks.org layout (index.json, bundle.json, by-digest).
  - The lock (still version 1) records [sources.*] and `source =`, sorted
    official first, then source, slug, version; bundles vendor to
    trust-tasks/@<source>/…. Every lock read or written is also held to
    trust-tasks-build's reading of it.
  - Refused: a sourced bundle under https://trusttasks.org/ (SPEC §6.5
    item 1) and an official one outside it; a framework slug from a source;
    two pins with one Type URI; a source named like a pinned official
    family; a malformed source name or an http url; a bundle whose Type
    URI, slug and version disagree with each other or the lock.
  - `trust-tasks registry build --specs --type-uri-prefix --out
    [--official-specs]` builds a third party's specs/ tree into the served
    layout, by-digest store included. TRUST_TASKS_CA_FILE trusts an
    internal CA.

  trust-tasks-build
  - Lock parsing accepts [sources.*] and `source`; a sourced task is
    generated under a top-level module named for its source
    (tasks::acme::payments::settle::v1_0) and read from its vendored path.
    Identity is the bundle's Type URI in any SPEC §6.1 form; EXTENDS and
    declare_extends() key by it.
  - New `publish` feature: the bundle builder (front matter, policy, $ref
    inlining incl. absolute https://trusttasks.org component refs,
    lifecycles, canonical JSON, the served layout), moved out of
    trust-tasks-codegen so the registry and third parties share one
    builder. The codegen's official bundles and trust-tasks-rs modules are
    byte-identical.

  trust-tasks-rs
  - TypeUri parses every SPEC §6.1 form — https and other path forms, DID
    URLs, and urn:<nid>:<prefix>:spec:<seg>:…:<M.N> — and prints each back
    exactly; http stays refused (its error text claimed otherwise).
  - The dispatchers and read_as decide unsupportedVersion vs
    unsupportedType by authority and slug, so another authority's acl/show
    is an unsupported type, not an unsupported version of the registry's.

  examples/consumer-app pins acme:vault/get/0.3 — the registry's slug and
  version under https://tasks.example.com — from a fixture registry, and
  dispatches both.


