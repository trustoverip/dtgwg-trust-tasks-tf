# Changelog — `trust_tasks_https`

All notable changes to the Dart HTTPS binding package.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against.
Below 1.0 a breaking change bumps the leading non-zero component.

Publishing is triggered by the `trust-tasks-dart-https-v<version>` tag, because
pub.dev only accepts an automated publish from a tag-triggered workflow. See
`RELEASING.md`.

## 0.1.1 — 2026-09-21

## 0.1.0

### Added

- Initial release: `HttpsClient` and a framework-agnostic `HttpsServer` for the
  Trust Tasks HTTPS binding (`bindings/https/0.2`), with a `dart:io` adapter in
  `package:trust_tasks_https/io.dart`.
