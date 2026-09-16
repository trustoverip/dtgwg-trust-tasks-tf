# Changelog — `trust-tasks-go/tsp`

All notable changes to the Go TSP transport binding module.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). A Go
module's version is its git tag; this module is published by pushing a
`trust-tasks-go/tsp/vX.Y.Z` tag, after which `proxy.golang.org` serves it.

## 0.1.0

### Added

- Initial release: `PackTrustTask` / `UnpackTrustTask` for HPKE-sealed Direct TSP
  messages (`bindings/tsp/0.1`), and `Consumer` with `Receive`, the guarded
  inbound path running SPEC §7.2 with the item-11 duplicate-execution record on
  by default. A separate module from `trust-tasks-go`, so the core stays
  dependency-free.

  Requires `github.com/affinidi/affinidi-tsp-go`, at a pseudo-version until it
  is tagged.
