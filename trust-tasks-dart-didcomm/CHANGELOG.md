# Changelog — `trust_tasks_didcomm`

All notable changes to the Dart DIDComm binding package.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against.
Below 1.0 a breaking change bumps the leading non-zero component.

Publishing is triggered by the `trust-tasks-dart-didcomm-v<version>` tag,
because pub.dev only accepts an automated publish from a tag-triggered
workflow. See `RELEASING.md`.

## 0.1.0

### Added

- Initial release: `packTrustTask` / `unpackTrustTask` for authcrypt DIDComm
  v2.1 envelopes (`bindings/didcomm/0.2`), and `DidcommConsumer`, the guarded
  inbound path with the §7.2 item 11 duplicate-execution record on by default.
