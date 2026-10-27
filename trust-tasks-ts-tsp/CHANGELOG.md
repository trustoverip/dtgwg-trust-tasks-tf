# Changelog — `@openvtc/trust-tasks-tsp`

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The
package versions over its own API. Below 1.0 a breaking change bumps the leading
non-zero component. Publishing is triggered by the `trust-tasks-ts-tsp-v<version>`
tag; see `RELEASING.md`.

## 0.1.1 — 2026-10-27


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

## 0.1.0

### Added

- Initial release: `packTrustTask` / `unpackTrustTask` for HPKE-sealed Direct TSP
  messages (`bindings/tsp/0.1`) on `@openvtc/vti-tsp-js`, and `TspConsumer`, the
  guarded inbound path running SPEC §7.2 with the item-11 duplicate-execution
  record on by default. The sealed `{type, document}` envelope matches the Rust
  crate, the Dart package and the Go module.
