# Trust Tasks — repo guide for Claude

This repo holds the Trust Tasks framework spec, the per-task spec/schema registry
(`specs/<slug>/<version>/`), the transport bindings (`bindings/`), the registry
website (`website/`), and the generated client libraries (`trust-tasks-rs`,
`trust-tasks-ts`, `trust-tasks-go`, `trust-tasks-dart`) with their hand-written
binding and proof packages.

The developer guide is for humans and lives in the docs — **read the section a
rule links to before acting on it**; this file is only the index of rules that
have been broken before:

- [`CONTRIBUTING.md`](CONTRIBUTING.md) — DCO, PR titles, what to run before a PR.
- [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md) — layout, build/regenerate
  commands, the CI-check table, hand-maintained lists, generator independence,
  lifecycle, bundles, `extends`, instants, routing consumers, error codes, size bounds.
- [`docs/packages/rust.md`](docs/packages/rust.md),
  [`typescript.md`](docs/packages/typescript.md),
  [`go.md`](docs/packages/go.md), [`dart.md`](docs/packages/dart.md) — per-library notes.
- [`CONTRIBUTING-SPECS.md`](CONTRIBUTING-SPECS.md) — writing a spec.
  [`RELEASING.md`](RELEASING.md) — releases.

When you change something one of those documents describes, update the
document in the same PR. Do not copy their content back into this file.

## ⚠️ Rules that must not be forgotten

- **`SPEC.md` is generated — never edit it here.** Framework changes go to the
  canonical repo; the canonical commit is pinned in
  `scripts/framework-spec-source.json` and moves only through the
  `framework-spec-bump.yml` PR; a `/SPEC.md#…` link that names no section fails
  the build.
  → [DEVELOPMENT.md](docs/DEVELOPMENT.md#specmd-is-generated--never-edit-it-here)
- **Every commit MUST be DCO signed** — `git commit -s` (`git merge --signoff`).
  → [CONTRIBUTING.md](CONTRIBUTING.md#every-commit-must-be-dco-signed)
- **Never bump a version or hand-write a `CHANGELOG.md` entry.** Merging is not
  releasing; the Release PR assigns versions and changelogs.
  → [DEVELOPMENT.md](docs/DEVELOPMENT.md#merging-is-not-releasing--read-releasingmd-before-touching-a-version), [RELEASING.md](RELEASING.md)
- **The PR title is load-bearing.** `feat(<slug>):` to add a spec family,
  `spec(<slug>):` to edit one; **a `!` only for a package's public API, scoped to
  the package (`feat(rs)!:`) — never for a spec change.**
  → [CONTRIBUTING.md](CONTRIBUTING.md#the-pull-request-title-is-the-changelog)
- **Changed anything under `specs/`? Regenerate all four libraries** (Rust, TS,
  Go, Dart) in the same PR; a released version only changes additively.
  → [DEVELOPMENT.md](docs/DEVELOPMENT.md#changing-a-spec-or-payload-schema--regenerate-the-libraries)
- **A spec version you add must target the current framework**
  (`CURRENT_FRAMEWORK_VERSION`, `0.6.0`), and a status you raise must be on the
  latest live version of its slug (`spec lifecycle rules`).
  → [DEVELOPMENT.md](docs/DEVELOPMENT.md#what-the-ci-checks-actually-assert)
- **The Rust toolchain is pinned** in `rust-toolchain.toml`; CI installs it via
  `.github/actions/rust-toolchain`, never `dtolnay/rust-toolchain@stable`; an MSRV
  job sets `RUSTUP_TOOLCHAIN`.
  → [DEVELOPMENT.md](docs/DEVELOPMENT.md#the-rust-toolchain-is-pinned--rust-toolchaintoml)
- **Judge a library bump by what a consumer observes** — a behavioural break
  counts, and a leading-component bump is a workspace event.
  → [DEVELOPMENT.md](docs/DEVELOPMENT.md#library-versions-are-semver-over-the-librarys-api-not-the-frameworks)
- **Hand-maintained lists:** a new category needs `spec.meta.schema.json` **and**
  `website/assets/data.js`; a new framework-reserved slug needs the
  `spec.meta.schema.json` allowlist; a new binding needs `website/assets/bindings.js`;
  a library change needs `website/assets/libraries.js`; binding prose must match
  its front matter and never pin a `trust-task-error` version.
  → [DEVELOPMENT.md](docs/DEVELOPMENT.md#hand-maintained-lists)
- **Do not factor the generators (or the checks) together.** The four
  generators, `check-bindings-conformance.mjs` and `check-bundles` are
  deliberately independent re-implementations.
  → [DEVELOPMENT.md](docs/DEVELOPMENT.md#keep-the-generators-and-checks-independent)
- **A green board is not proof** — know what each CI check cannot see.
  → [DEVELOPMENT.md](docs/DEVELOPMENT.md#what-the-ci-checks-actually-assert)
- **Withdrawn / superseded / prebuilt-deprecated modules** — Rust CI denies
  warnings; move hand-written uses before a withdrawal batch.
  → [DEVELOPMENT.md](docs/DEVELOPMENT.md#superseded-retired-and-withdrawn-versions-in-the-generators),
  [prebuilt modules](docs/DEVELOPMENT.md#every-prebuilt-specification-module-is-deprecated-milestone-5)
- **Proofs are verified over the document as received**, never a pruned or
  typed copy. → [DEVELOPMENT.md](docs/DEVELOPMENT.md#extends-routing-a-newer-version-to-the-version-a-consumer-implements)
- **Never write `Utc::now()` / `toISOString()` into a document** — stamp whole
  seconds through the library's clock.
  → [DEVELOPMENT.md](docs/DEVELOPMENT.md#instants-the-libraries-write-whole-seconds)
- **Changing a cross-runtime behaviour?** Update `docs/runtime-parity.md`.
  → [DEVELOPMENT.md](docs/DEVELOPMENT.md#consumers-that-route-on-the-type-uri)
- **Rust:** cross-crate construction uses `X::builder()`; unit tests see only
  the `acl` family; the per-module generator is `trust-tasks-build`.
  → [rust.md](docs/packages/rust.md)
- **Go:** no `_` directories, one package per directory, optional slices are
  pointers, a Go tag can never be retracted.
  → [go.md](docs/packages/go.md)
- **Dart:** the barrel exports the runtime only; closed value sets are
  extension types; `version:` and `packageVersion` move together.
  → [dart.md](docs/packages/dart.md)
- **TypeScript siblings:** the core is a peer dependency; raise its floor only
  after the core release publishes.
  → [typescript.md](docs/packages/typescript.md)
