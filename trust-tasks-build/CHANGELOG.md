# Changelog

Notable changes to the published packages. Generated from conventional commits
by [git-cliff](https://git-cliff.org) when a release is cut — do not edit by
hand.
## [0.1.3](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-build-v0.1.2...trust-tasks-build-v0.1.3) — 2026-10-29


### Added

- **build**: An opt-in Type URI index for generated tasks ([#974](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/974))


## [0.1.2](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-build-v0.1.1...trust-tasks-build-v0.1.2) — 2026-10-26


### Added

- **build**: Bundles record the $defs order and each shared definition's origin ([#938](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/938))

* feat(build): bundles record the $defs order and each shared definition's origin

  A bundle's schemas are key-sorted, so the TypeScript, Go and Dart generators
  could not declare a schema's definitions in the order their in-repo
  counterparts do (1,100 of 1,127 Go files and 662 Dart files differed only in
  that order), and the TypeScript generator could not tell one shared component
  copied into many bundles from unrelated definitions that share a name.

  Two members, both additive within bundleVersion 1 (readers ignore them, and
  treat them as {} when absent):

  - defsOrder: /schema and /responseSchema -> the root $defs names in the
    order the TS/Go/Dart inliners leave them: the payload schema's own as
    authored, then each spliced definition in the order a last-in-first-out
    $ref frontier, collected in document order, reaches it. The builder
    replays that walk over the authored documents and refuses a bundle whose
    replay does not name exactly its $defs.
  - defOrigins: each spliced definition -> <its document's $id>#/$defs/<name>,
    or, for a document with no $id, its path under the specifications root
    below the Type URI prefix.

  check-bundles verifies both without the Rust builder: defsOrder against the
  $defs order of the TypeScript module's schemas (all 1,127 agree, as they do
  with Go's embedded schema text), and every origin against the $ids and $defs
  of the shared schemas in specs/.

  Every bundle's digest changes once when this deploys. The by-digest store is
  append-only, so existing locks still build, check and fetch; `trust-tasks
  update` takes the new bytes and says so.



## [0.1.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-build-v0.1.0...trust-tasks-build-v0.1.1) — 2026-10-26


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


