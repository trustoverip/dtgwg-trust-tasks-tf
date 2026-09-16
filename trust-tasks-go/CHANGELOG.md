# Changelog — `trust-tasks-go`

All notable changes to the Go bindings module.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The module versions over **its own API** — what a consumer compiles against —
not over `SPEC.md`. Below 1.0 a breaking change bumps the leading non-zero
component.

A Go module is published by tagging, so the released version of this module is
the `trust-tasks-go/vX.Y.Z` tag rather than anything in the tree; the `Version`
constant in `trusttasks/version.go` mirrors it. See `RELEASING.md`.

## 0.1.2 — 2026-09-16


### Added

- **go-tsp**: TSP transport binding for Go (trust-tasks-go/tsp) (#486)

A separate nested Go module so the core stays dependency-free: PackTrustTask / UnpackTrustTask for HPKE-sealed Direct TSP messages on affinidi-tsp-go, and Consumer.Receive, the guarded inbound path running SPEC §7.2 with the item-11 record on by default. The sealed {type, document} envelope matches the Rust crate and Dart package (pinned by a test).

  Not released: affinidi-tsp-go has no tag, so it is required at a pseudo-version and there is no trust-tasks-go/tsp/v* release wiring yet; go get resolves the pseudo-version from the public repo. go.yml gets a dedicated tsp job; the core ./... jobs exclude the nested module.

## 0.1.1 — 2026-09-16

## 0.1.0

### Added

- Initial release: generated payload types for every specification in the
  registry, and the hand-written SPEC.md §7.2 consumer pipeline in
  `trusttasks/`.
