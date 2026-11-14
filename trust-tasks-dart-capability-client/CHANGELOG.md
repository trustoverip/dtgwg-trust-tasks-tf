# Changelog

## 0.2.1 — 2026-11-14

### Dependencies

- Requires `trust_tasks` ^0.12.0 (was ^0.11.0), the core released beside it.

## 0.2.0 — 2026-11-13


### Added

- **dart,dart-https,dart-didcomm**: JSON priorResponse, observability hook, forward routing and per-type size bounds (#1242)

* feat(dart,dart-https,dart-didcomm)!: JSON priorResponse, observability hook, forward routing and per-type size bounds



### Fixed

- **capability-client,ts-capability-client,go-capabilityclient,dart-capability-client**: Bind a capability reply to the request it answers (#1248)

* fix(capability-client,ts-capability-client,go-capabilityclient,dart-capability-client): bind a capability reply to the request it answers

  The capability clients matched a reply to a request by thread id and slug alone. All four ports gain request-bound classifiers (classify_git_trust_reply_to / parse_capability_reply_to and their TS, Go and Dart counterparts) that also require the reply's issuer to be the request's recipient, its recipient to be the request's issuer, and its type to be the request's own #response or the framework error type, as the HTTPS clients do. The thread-only classifiers now read trust-task-error and family responses only under the registry's authority, and a toggle or listing reply missing a required member is no longer read as reporting defaults. TypeScript parses a body as a document only when its payload is an object. Rust adds try_build_document for a caller-supplied Type URI.

### Dependencies

- Requires `trust_tasks` ^0.11.0 (was ^0.10.0), the core released beside it.

## 0.1.2 — 2026-11-12


### Fixed

- **ts,go,dart,rs**: Whole-second capability-client issuedAt, tested Dart core floors, TS default export condition, Go schema caching (#1214)

* fix(dart-capability-client,go-capabilityclient,go-didcomm): stamp issuedAt in whole seconds and fail on entropy failure

  The Dart capability client stamped issuedAt with toIso8601String (full
  precision); it now uses the core's systemClock. The Go port now stamps
  through trusttasks.SystemClock. The TS port already stamped whole seconds.

  The Go capabilityclient and didcomm UUIDv4 generators ignored crypto/rand
  errors and would mint an all-zero id; they now panic, as tsp's does.

### Dependencies

- Requires `trust_tasks` ^0.10.0 (was ^0.9.3), the core released beside it.

## 0.1.1 — 2026-10-23


### Fixed

- **dart**: The sibling packages require the current trust_tasks core (#895)

Every published sibling (trust_tasks_https, _proof, _didcomm, and the
  unreleased _tsp and _capability_client) declared `trust_tasks: ^0.1.0`,
  which pub reads as `>=0.1.0 <0.2.0`. A consumer therefore got core 0.1.19,
  missing every fix since, and an app already on trust_tasks 0.9.x could not
  add a sibling at all: the constraints conflict. tag-dart's release check
  did not notice, because it verifies against the newest published core the
  constraint admits. The floor is now ^0.9.3.

  That core releases the replay claim itself when a handler throws (#876),
  so the HTTPS server and the DIDComm and TSP consumers no longer release it
  a second time. Their release also fired when the guard failed to record a
  response after the handler had run, freeing a document whose effect had
  already happened; the core keeps the claim in that case, and so do they now.

  They also pass `responseToJson` (#878), so a persistent ReplayGuard caches
  the success response as JSON and a resend is answered with it.

## 0.1.0

- Initial release: document builders and reply classification for the capability
  Trust Task families (`governance/capability/*` and `git-trust/*`). No crypto —
  sign with `package:trust_tasks_proof`.
