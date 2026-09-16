# Changelog — `@openvtc/trust-tasks-tsp`

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The
package versions over its own API. Below 1.0 a breaking change bumps the leading
non-zero component. Publishing is triggered by the `trust-tasks-ts-tsp-v<version>`
tag; see `RELEASING.md`.

## 0.1.0

### Added

- Initial release: `packTrustTask` / `unpackTrustTask` for HPKE-sealed Direct TSP
  messages (`bindings/tsp/0.1`) on `@openvtc/vti-tsp-js`, and `TspConsumer`, the
  guarded inbound path running SPEC §7.2 with the item-11 duplicate-execution
  record on by default. The sealed `{type, document}` envelope matches the Rust
  crate, the Dart package and the Go module.
