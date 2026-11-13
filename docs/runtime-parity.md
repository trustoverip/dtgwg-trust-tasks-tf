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
| `extends` chain-walk bound — 64 links. Every link must also name a strictly earlier MINOR of the same MAJOR, so the walk ends anyway; the bound is a backstop on the work of one lookup | `MAX_CHAIN` (private, `src/extension.rs`) | `MAX_EXTENDS_CHAIN` (private, `_runtime/extends.ts`) | `maxExtendsChain` (`trusttasks/extends.go`) | `_maxExtendsChain` (`runtime/extension.dart`) |
| Prune recursion depth — 128, through the payload and the schema's `$ref`s and branches together; beyond it the rest of the value is passed through unpruned | `MAX_DEPTH` (private, `src/extension.rs`) | `MAX_DEPTH` (private, `_runtime/extends.ts`) | `maxPruneDepth` (`trusttasks/extends.go`) | `_maxDepth` (`runtime/extension.dart`) |
| Per-type document size bound (`maxDocumentBytes` front matter), generated per variant only where declared | `Payload::MAX_DOCUMENT_BYTES` (and `schema_index::max_document_bytes_for`) | `SPEC.maxDocumentBytes` / `RESPONSE_SPEC.maxDocumentBytes` | `Spec.MaxDocumentBytes` / `ResponseSpec.MaxDocumentBytes` (0: none) | `spec.maxDocumentBytes` / `responseSpec.maxDocumentBytes` |

The two `extends` bounds are private constants; `check-bindings` reads them
from the source. The size bounds are compared with each specification's front
matter, independently for each language.

Also in agreement, guarded by per-library tests rather than by
`check-bindings`:

- Every `issuedAt` a library stamps is UTC in whole seconds
  (`tests/issued_at_precision.rs`, `test/issued-at.test.ts`,
  `trusttasks/issued_at_test.go`, `test/issued_at_test.dart` pin the same
  vectors — see CLAUDE.md "Instants the libraries write").
- An `InMemoryReplayGuard` built with a capacity below one takes the default
  capacity rather than retaining nothing — a guard that retains nothing
  answers "fresh" to everything, a silent total defeat of §7.2 item 11. It is
  not refused either: Go and Dart already defaulted, and moving Rust (which
  panicked) and TypeScript (which threw `RangeError`) to the default only
  relaxes them (#1130). Pinned by `replay::tests::a_zero_capacity_takes_the_default`,
  `test/replay.test.ts`, `TestInMemoryGuardNeverRetainsNothing` and
  `replay_test.dart`'s "never retains nothing".
- Every core `Proof` type treats `created` as present, and every signer writes
  it, although VC Data Integrity makes it optional (#1119). Rust, TypeScript
  and Dart require it when parsing; Go's `Proof.Created` is a plain `string`
  and reads an absent one as `""`. The proof verifiers read it as "when
  present". Each proof package's README says so.

## Differences

| Item | Rust | TypeScript | Go | Dart | Status |
|---|---|---|---|---|---|
| Type-erased dispatcher (`Dispatcher`, `AsyncDispatcher`, `consume_dispatched`) | yes (`src/dispatcher.rs`, `src/async_dispatch.rs`, `src/consume.rs`) | deliberately no | deliberately no | deliberately no | Decided: the consume functions take the spec's policy as a value, and the caller looks the spec up by `type` (CLAUDE.md "Consumers that route on the Type URI"). |
| Declared error codes (`ERROR_CODES`, `error_codes::…`) | yes (generated; compared with front matter by `check-bindings`) | todo | todo | todo | Rust only *so far* (CLAUDE.md "Declared error codes are generated"); when the others emit them, add them to the same comparison. |
| Adding a §8.3 standard error code is non-breaking | yes (`StandardCode` is `#[non_exhaustive]`, `src/error.rs`) | deliberately no (`StandardCode` is a closed union, `_runtime/codes.ts`) | yes (named `string` type, `trusttasks/codes.go`) | yes (`extension type` over `String`, `runtime/codes.dart`) | Decided: TypeScript's closed union makes a new code a breaking change there (CLAUDE.md, the `StandardCode` note). |
| Name of "read a later version as the one implemented" | `extension::read_as` | `readForSpec` | `ReadForSpec` | `readForVersion` | Decided: deliberately different, naming only — the behaviour is the same rule, and aligning the names would cost three breaking releases for no change in behaviour (#1130). |
| Enforcing `maxDocumentBytes` (the generated bound itself is in the checked table above) | not enforced yet: the HTTPS server has a flat 256 KiB body cap (#1121's Rust half rides with #1089) | not by the runtime: `consumeInbound` is handed a parsed document, not its bytes, so the transport that reads the body enforces `SPEC.maxDocumentBytes` | `ConsumeInbound` refuses a document of `Spec`'s own version larger than `Spec.MaxDocumentBytes` with `malformedRequest` (`DocumentTooLargeWireMessage`) when given the bytes as `ConsumeOptions.Received` | the HTTPS server: body limit is the largest registered bound with a 256 KiB floor, then each document is held to its route's bound | Rust todo (#1121); TypeScript has no server package to enforce it in. A document read forward along `extends` is never held to the implemented version's bound — a later version may declare a larger one. |
| `trust-task-discovery` responder | yes (`src/discovery.rs`, `DiscoveryRegistry`) | todo | todo | todo | todo, low priority: a responder mainly matters to a server, and TypeScript and Go have no HTTPS server package. Revisit when one exists (#1130). |
| Bundled JSON Schema validator | yes, behind the `validate` feature (`jsonschema`) | deliberately no — the schema is exported (`PAYLOAD_SCHEMA`) for a caller's validator (ajv, for example) | deliberately no — the core module has zero dependencies; `PayloadSchemaJSON` is exported for a caller's validator | deliberately no — `SpecPolicy.payloadSchema` for a caller's validator (`json_schema`, for example) | Decided: the TypeScript, Go and Dart cores stay dependency-light, and the caller brings the validator (#1130). |

When a *needs decision* row is settled and the four agree, add its constant to
the runtime-defaults comparison in `scripts/check-bindings-conformance.mjs` and
move the row to the checked table above, as the replay-capacity, `extends`
chain and prune-depth rows were (#1130).
