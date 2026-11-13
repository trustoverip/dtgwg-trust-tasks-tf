# Runtime parity matrix

The four client libraries — `trust-tasks-rs`, `@openvtc/trust-tasks`
(`trust-tasks-ts`), `trust-tasks-go` and `trust_tasks` (`trust-tasks-dart`) —
implement the same §7.2 pipeline independently. This page says, for every
place they are known to differ, whether the difference is a decision or a gap,
so a reader can tell the two apart (#1130).

- **yes** — the runtime has it.
- **deliberately no** — it does not, on purpose; the reason is given.
- **todo** — it does not, and nobody has decided whether it should. *Needs
  decision* marks a row where the runtimes disagree and one behaviour has to
  be picked before anything is changed.

Every cell was checked against the source at the time of writing; the file and
line are cited so it can be checked again. When you change a row's behaviour
in one library, update this page in the same PR.

## Mechanically checked

These agree today and `npm run check-bindings`
(`scripts/check-bindings-conformance.mjs`) fails if one library moves without
the others.

| Item | Rust | TypeScript | Go | Dart |
|---|---|---|---|---|
| Emitted error Type URI | `TRUST_TASK_ERROR_TYPE_URI` | `TRUST_TASK_ERROR_TYPE_URI` | `TrustTaskErrorTypeURI` | `trustTaskErrorTypeUri` |
| Default clock skew — 60 s | `DEFAULT_SKEW` | `DEFAULT_SKEW_MS` | `DefaultSkew` | `defaultSkew` |
| Consequential max age over `issuedAt` — 5 min | `DEFAULT_MAX_AGE` | `DEFAULT_MAX_AGE_MS` | `DefaultMaxAge` | `defaultMaxAge` |
| Expiry horizon (how far past `issuedAt` an `expiresAt` is honoured) — 30 days | `DEFAULT_EXPIRY_HORIZON` | `DEFAULT_EXPIRY_HORIZON_MS` | `DefaultExpiryHorizon` | `defaultExpiryHorizon` |
| In-memory replay guard default capacity — 100,000 | `DEFAULT_REPLAY_CAPACITY` | `DEFAULT_REPLAY_CAPACITY` | `DefaultReplayCapacity` | `defaultReplayCapacity` |

Also in agreement, guarded by per-library tests rather than by
`check-bindings`: every `issuedAt` a library stamps is UTC in whole seconds
(`tests/issued_at_precision.rs`, `test/issued-at.test.ts`,
`trusttasks/issued_at_test.go`, `test/issued_at_test.dart` pin the same
vectors — see CLAUDE.md "Instants the libraries write").

## Differences

| Item | Rust | TypeScript | Go | Dart | Status |
|---|---|---|---|---|---|
| Type-erased dispatcher (`Dispatcher`, `AsyncDispatcher`, `consume_dispatched`) | yes (`src/dispatcher.rs`, `src/async_dispatch.rs`, `src/consume.rs`) | deliberately no | deliberately no | deliberately no | Decided: the consume functions take the spec's policy as a value, and the caller looks the spec up by `type` (CLAUDE.md "Consumers that route on the Type URI"). |
| Declared error codes (`ERROR_CODES`, `error_codes::…`) | yes (generated; compared with front matter by `check-bindings`) | todo | todo | todo | Rust only *so far* (CLAUDE.md "Declared error codes are generated"); when the others emit them, add them to the same comparison. |
| Adding a §8.3 standard error code is non-breaking | yes (`StandardCode` is `#[non_exhaustive]`, `src/error.rs`) | deliberately no (`StandardCode` is a closed union, `_runtime/codes.ts`) | yes (named `string` type, `trusttasks/codes.go`) | yes (`extension type` over `String`, `runtime/codes.dart`) | Decided: TypeScript's closed union makes a new code a breaking change there (CLAUDE.md, the `StandardCode` note). |
| HTTPS server routes a later version along `extends` and answers `unsupportedVersion` | yes (`trust-tasks-https` runs `consume_dispatched_answer`) | n/a (no HTTPS server) | n/a (no HTTPS server) | yes (`trust-tasks-dart-https`, `readForVersion`; #1242) | Decided (#1117): every server and consumer routes forward. |
| Name of "read a later version as the one implemented" | `extension::read_as` | `readForSpec` | `ReadForSpec` | `readForVersion` | Naming only; behaviour is the same rule. todo: needs decision on whether to align the names. |
| Per-type document size bound (`maxDocumentBytes` front matter) | generated (`Payload::MAX_DOCUMENT_BYTES`, `schema_index::max_document_bytes_for`); enforced by `consume_inbound` / `consume_dispatched` when the binding passes `ConsumeChecks::received_bytes`, which `trust-tasks-https`'s server does (its body limit is the largest registered bound, 256 KiB floor); the message bindings do not yet | todo (not generated) | todo (not generated) | generated (`SpecPolicy.maxDocumentBytes`); enforced by `trust-tasks-dart-https` (body limit the largest registered bound, 256 KiB floor) | todo for TypeScript and Go |
| `trust-task-discovery` responder | yes (`src/discovery.rs`, `DiscoveryRegistry`) | todo | todo | todo | todo |
| Bundled JSON Schema validator | yes, behind the `validate` feature (`jsonschema`) | no — the schema is exported (`PAYLOAD_SCHEMA`) for a caller's validator | deliberately no — the core module has zero dependencies; `PayloadSchemaJSON` is exported for a caller's validator | no — `SpecPolicy.payloadSchema` for a caller's validator | Go decided; TypeScript and Dart todo: needs decision. |
| `InMemoryReplayGuard` with capacity 0 | panics (`src/replay.rs:476-479`) | throws `RangeError` (`_runtime/replay.ts:258-260`) | silently uses 100,000 (`trusttasks/replay.go:283-285`) | silently uses 100,000 (`runtime/replay.dart:275-277`) | **todo: needs decision** — refuse or default, then make all four do it. |
| `extends` chain-walk bound | 256 links (`MAX_CHAIN`, `src/extension.rs:55`) | none; the walk stops when the MINOR does not strictly decrease (`_runtime/extends.ts:139-146`) | 64 links (`maxExtendsChain`, `trusttasks/extends.go:63`) | seen-set, stops on a repeat (`runtime/extension.dart:65-76`) | **todo: needs decision.** All terminate; the generator only emits links to a strictly earlier MINOR, so real chains are short. |
| Prune recursion depth (beyond it, the value is passed through unpruned) | 128 (`MAX_DEPTH`, `src/extension.rs:59`) | 64 (`MAX_DEPTH`, `_runtime/extends.ts:312`) | 256 (`maxPruneDepth`, `trusttasks/extends.go:277`) | 128 (`_maxDepth`, `runtime/extension.dart:331`) | **todo: needs decision.** A payload nested 65–128 levels deep is pruned by Rust and Dart and passed through by TypeScript, so the typed payload a handler sees differs between libraries. |

Once a *needs decision* row is settled and the four agree, add its constant to
the runtime-defaults comparison in `scripts/check-bindings-conformance.mjs` and
move the row to the checked table above.
