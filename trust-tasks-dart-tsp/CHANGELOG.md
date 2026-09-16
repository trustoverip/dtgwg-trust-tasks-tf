# Changelog — `trust_tasks_tsp`

All notable changes to the Dart TSP binding package.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against.
Below 1.0 a breaking change bumps the leading non-zero component.

Publishing is triggered by the `trust-tasks-dart-tsp-v<version>` tag, because
pub.dev only accepts an automated publish from a tag-triggered workflow. See
`RELEASING.md`.

## 0.1.0

### Added

- Initial release: `packTrustTask` / `unpackTrustTask` for HPKE-sealed Direct
  TSP messages (`bindings/tsp/0.1`), and `TspConsumer`, the guarded inbound path
  with the §7.2 item 11 duplicate-execution record on by default.

  Not yet published to pub.dev: it depends on `affinidi_tsp`, which is not on
  pub.dev at the time of writing (see the README).
