# Changelog

Notable changes to the published packages. Generated from conventional commits
by [git-cliff](https://git-cliff.org) when a release is cut — do not edit by
hand.
## [0.2.0](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-build-v0.1.4...trust-tasks-build-v0.2.0) — 2026-11-13


### Added

- **rs,build**: A generated file names the runtime contract it needs ([#1253](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/1253))

trust-tasks-rs gains a hidden __private::ABI_V1, and every file
  trust-tasks-build generates for an application opens with
  const _: () = ::trust_tasks_rs::__private::ABI_V1; under a comment naming
  the trust-tasks-build version. A runtime too old for the generator, or one
  that has moved to a newer contract, now fails on that line with an error
  naming the contract, instead of on whichever hidden door the generated code
  reaches first ([#1101](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/1101)). The in-repo prebuilt tree is unchanged.

- **rs,build,ts,ts-gen,go,dart**: Emit only the $defs each schema constant reaches ([#1243](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/1243))

* feat(rs,build,ts,ts-gen,go,dart): emit only the $defs each schema constant reaches

  Each generated request and response schema constant now keeps only the root $defs its root reaches: the request drops Response and what only the response uses, the response document drops what only the request uses. Registry bundles are unchanged, so no pinned digest moves. Types are still generated from the whole schema.

  trust-tasks-rs packages at 10,037,990 bytes (9.57 MiB) against 10,349,753 (9.87 MiB) before.

- **rs**: No prebuilt specification families by default ([#1240](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/1240))

`trust-tasks-rs` and the `trust-tasks` facade now have `default = []`. A
  plain dependency compiles the runtime and the framework's own
  `trust-task-*` modules, not every deprecated prebuilt family (a ~17 GB
  build). A consumer that still uses `specs::<family>` enables
  `features = ["all-specs"]`, or the one family it uses.



### Fixed

- **build,cli**: Require a spec slug and version for every locked task, and the canonical path for every index entry ([#1245](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/1245))

An official lockfile entry's slug and version were checked only against a Type URI built from the same strings, then used as path segments of the vendored bundle's location. Each entry's slug and version must now have their SPEC form, whichever registry it comes from. The CLI now also requires an index entry's path to be <slug>/<version>/bundle.json, the only form the registry publishes.



## [0.1.4](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-build-v0.1.3...trust-tasks-build-v0.1.4) — 2026-11-12


### Documentation

- RELEASING describes the Dart downgrade leg as it now behaves; trust-tasks-build README uses cargo add ([#1225](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/1225))
- Fix drifted counts, release runbook, licences and stale examples across the four libraries ([#1224](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/1224))

* docs(getting-started): move the worked example to acl/grant/0.3 and stamp issuedAt with issued_at_now()

  The example pinned acl/grant/0.2, superseded by 0.3, and stamped issuedAt
  with chrono::Utc::now(), the full-precision form #970 removed from the
  libraries. The TS consumer example wrote toISOString() milliseconds into
  document fields the same way. The dependency snippet now uses cargo add,
  so it cannot name a stale version, and §4.4 describes the shared version
  group instead of versions from 0.14.



### Fixed

- **rs,build,cli,codegen**: Review fixes — declared guard code, SchemaValidator, keyword slugs, generate --lang rs, registry hints, docs.rs ([#1215](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/1215))

* fix(rs): declared acl/revoke code in the guard, a caching SchemaValidator, docs.rs metadata

  - guards::acl::reject_last_authority returns acl/revoke:lastAuthorityProtected,
    the code every acl/revoke version declares (was snake_case); new
    LAST_AUTHORITY_PROTECTED constant, pinned to the generated declarations ([#1093](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/1093)).
  - validate::SchemaValidator: a PayloadValidator compiling each schema once,
    cached by the schema text's address and confirmed against its content;
    validate_value and check_details use a shared one; prune_payload reuses a
    parsed-schema cache; PayloadPolicy::accept_unvalidated() ([#1091](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/1091)).
  - docs.rs builds trust-tasks-rs with validate + acl and the facade with its
    non-spec features, and trust-tasks-rs / trust-tasks-https enable doc_cfg
    under docsrs, without which the docs.rs build cannot compile ([#1106](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/1106)).



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


