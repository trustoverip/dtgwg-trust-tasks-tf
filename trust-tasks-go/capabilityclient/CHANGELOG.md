# Changelog

All notable changes to `trust-tasks-go/capabilityclient` are documented here.
Entries are generated from conventional commits by the release automation (see
RELEASING.md).

## 0.2.0 — 2026-11-13


### Fixed

- **capability-client,ts-capability-client,go-capabilityclient,dart-capability-client**: Bind a capability reply to the request it answers (#1248)

* fix(capability-client,ts-capability-client,go-capabilityclient,dart-capability-client): bind a capability reply to the request it answers

  The capability clients matched a reply to a request by thread id and slug alone. All four ports gain request-bound classifiers (classify_git_trust_reply_to / parse_capability_reply_to and their TS, Go and Dart counterparts) that also require the reply's issuer to be the request's recipient, its recipient to be the request's issuer, and its type to be the request's own #response or the framework error type, as the HTTPS clients do. The thread-only classifiers now read trust-task-error and family responses only under the registry's authority, and a toggle or listing reply missing a required member is no longer read as reporting defaults. TypeScript parses a body as a document only when its payload is an object. Rust adds try_build_document for a caller-supplied Type URI.

## 0.1.1 — 2026-11-12


### Documentation

- Fix drifted counts, release runbook, licences and stale examples across the four libraries (#1224)

* docs(getting-started): move the worked example to acl/grant/0.3 and stamp issuedAt with issued_at_now()

  The example pinned acl/grant/0.2, superseded by 0.3, and stamped issuedAt
  with chrono::Utc::now(), the full-precision form #970 removed from the
  libraries. The TS consumer example wrote toISOString() milliseconds into
  document fields the same way. The dependency snippet now uses cargo add,
  so it cannot name a stale version, and §4.4 describes the shared version
  group instead of versions from 0.14.



### Fixed

- **ts,go,dart,rs**: Whole-second capability-client issuedAt, tested Dart core floors, TS default export condition, Go schema caching (#1214)

* fix(dart-capability-client,go-capabilityclient,go-didcomm): stamp issuedAt in whole seconds and fail on entropy failure

  The Dart capability client stamped issuedAt with toIso8601String (full
  precision); it now uses the core's systemClock. The Go port now stamps
  through trusttasks.SystemClock. The TS port already stamped whole seconds.

  The Go capabilityclient and didcomm UUIDv4 generators ignored crypto/rand
  errors and would mint an all-zero id; they now panic, as tsp's does.

