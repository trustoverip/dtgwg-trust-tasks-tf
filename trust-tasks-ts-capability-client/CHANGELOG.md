# Changelog

All notable changes to `@openvtc/trust-tasks-capability-client` are documented here.

## 0.1.2 — 2026-10-28


### Fixed

- **ts-proof, ts-tsp, ts-capability-client**: Admit @openvtc/trust-tasks 0.28 (#965)

The core reached 0.28.0 in #963. The published siblings declare
  @openvtc/trust-tasks ^0.27.8 as a peer dependency, which npm reads as
  <0.28.0, so an application on the current core cannot install them.

  Raise the peer and dev ranges to ^0.28.0 and refresh the lockfiles
  against the published core. All three build, typecheck and pass their
  tests against it. Releasing this releases the three siblings.

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

- Initial release: document builders and reply classification for the capability
  Trust Task families (`governance/capability/*` and `git-trust/*`). No crypto —
  sign with `@openvtc/trust-tasks-proof`.
