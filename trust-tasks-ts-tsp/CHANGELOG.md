# Changelog — `@openvtc/trust-tasks-tsp`

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The
package versions over its own API. Below 1.0 a breaking change bumps the leading
non-zero component. Publishing is triggered by the `trust-tasks-ts-tsp-v<version>`
tag; see `RELEASING.md`.

## 0.1.4 — 2026-11-13


### Fixed

- **rs,ts,ts-tsp,proof,didcomm,didcomm-v1,tsp,dart-didcomm,dart-tsp,go-didcomm,go-tsp**: Message bindings enforce document size bounds; shared extends routing vector (#1263)

Size bounds (#1121). GuardedConsumer::consume_body and
  consume_body_dispatched, the inbound half of the DIDComm, DIDComm v1 and
  TSP consumers, now hold a body to its specification's declared
  maxDocumentBytes (SPEC §12.4) and refuse a larger one malformedRequest. The
  refusal comes after the envelope has authenticated the sender, goes to that
  sender, and is made before the item-11 record is written. The size is the
  body's compact JSON text, because those stacks hand over a decoded body. A
  specification that declares no bound gets no default bound.
  consume_inbound and consume_dispatched used to hold a document read forward
  along extends to the implemented version's bound. They no longer do, because
  a later version may declare a larger bound. The Dart DIDComm and TSP
  consumers and the TypeScript TSP consumer now apply the same rule. The
  TypeScript consumer reads SPEC.maxDocumentBytes defensively, since core
  0.29.0 does not carry it yet.

  Extends (#1117). trust-tasks-go/didcomm and trust-tasks-go/tsp now require
  the published core v0.11.0, use ReadForSpecWith, and take an optional
  Consumer.Extends lookup that is passed through to ConsumeOptions.Extends.
  They already pass the received bytes as ConsumeOptions.Received, so size
  bounds take effect there once they require a core that carries #1251's
  check, which v0.11.0 predates.
  A new shared vector, fixtures/consume/extends-routing.json, is run by all
  four runtimes. It found that TypeScript's readForSpec refused the
  implemented version's own #request Type URI as unsupportedType; that is now
  fixed.

  Proof (#1092). verify_raw no longer clones the whole document (or the
  proof) to drop `proof`. It serialises a borrowed view without that member.
  The byte-for-byte fixtures still pass.

- **rs,ts,go,dart,go-didcomm,go-tsp,ts-tsp**: Settle the open runtime parity rows (#1251)

* fix(rs,ts,go,dart): settle the open runtime parity rows

  Replay guard: a capacity below one takes the default capacity in all four
  libraries. Rust's InMemoryReplayGuard::new(0) no longer panics and the
  TypeScript constructor no longer throws RangeError; Go and Dart already
  defaulted. Moving the two to the default only relaxes them.

- **tsp,ts-tsp,go-tsp**: Open a TSP message only when it names this consumer's VID (#1247)

The receiver VID in a TSP envelope is written by the sender; the HPKE seal only shows the message was encrypted to the recipient's key. The Rust and TypeScript bindings took that VID as the transport recipient, so a message sealed to our key but naming another VID passed the recipient check and was answered as that VID. Both now refuse a message whose receiver VID is set and differs from the configured one, and take the configured VID as the transport recipient. The Go binding's upstream library already refuses it; Go now takes the configured VID as the transport recipient too and keeps its own check. Dart's upstream library already refuses it.

- **ts-proof,ts-tsp,ts-capability-client**: Require @openvtc/trust-tasks 0.29 (#1237)

## 0.1.3 — 2026-11-12


### Fixed

- **ts,go,dart,rs**: Whole-second capability-client issuedAt, tested Dart core floors, TS default export condition, Go schema caching (#1214)

* fix(dart-capability-client,go-capabilityclient,go-didcomm): stamp issuedAt in whole seconds and fail on entropy failure

  The Dart capability client stamped issuedAt with toIso8601String (full
  precision); it now uses the core's systemClock. The Go port now stamps
  through trusttasks.SystemClock. The TS port already stamped whole seconds.

  The Go capabilityclient and didcomm UUIDv4 generators ignored crypto/rand
  errors and would mint an all-zero id; they now panic, as tsp's does.

## 0.1.2 — 2026-10-28


### Fixed

- **ts-proof, ts-tsp, ts-capability-client**: Admit @openvtc/trust-tasks 0.28 (#965)

The core reached 0.28.0 in #963. The published siblings declare
  @openvtc/trust-tasks ^0.27.8 as a peer dependency, which npm reads as
  <0.28.0, so an application on the current core cannot install them.

  Raise the peer and dev ranges to ^0.28.0 and refresh the lockfiles
  against the published core. All three build, typecheck and pass their
  tests against it. Releasing this releases the three siblings.

## 0.1.1 — 2026-10-27


### Fixed

- The TypeScript sibling packages share the application's core and load only its runtime (#947)

@openvtc/trust-tasks-proof, -tsp and -capability-client declared
  "@openvtc/trust-tasks": "^0.19.0" as a regular dependency. npm reads that as
  >=0.19.0 <0.20.0, so an application on core 0.27 that added any of them got a
  second core, 0.19.x, installed beside its own, with every prebuilt module in
  it; their own lockfiles still built and tested against 0.19.8. They also
  imported from the package root, which loads every prebuilt module, where only
  runtime types and functions were used.

  Each now takes the core as a peer dependency at ^0.27.8 (and a dev
  dependency at the same range, to build and test against) and imports from
  @openvtc/trust-tasks/runtime. All three build and pass their tests against
  0.27.8.

  scripts/check-sibling-core-ranges.mjs (npm run check-sibling-core-ranges, in
  ts.yml's `bindings match specs`) fails when a TypeScript sibling lists the
  core anywhere but peerDependencies, or when a TypeScript or Dart sibling's
  caret range does not admit the core version in the checkout. Both ecosystems
  had drifted that way (Dart until #895); on a Release PR that moves the core
  across a 0.x minor it fails until the siblings' floors are raised.

  This is milestone 4 for TypeScript: no hand-written package uses a generated
  specification module outside its tests, so depending on the runtime alone is
  the whole change. The Rust crates did it in #939; the Go and Dart siblings
  already import only the runtime (docs/design/consumer-side-generation.md).

## 0.1.0

### Added

- Initial release: `packTrustTask` / `unpackTrustTask` for HPKE-sealed Direct TSP
  messages (`bindings/tsp/0.1`) on `@openvtc/vti-tsp-js`, and `TspConsumer`, the
  guarded inbound path running SPEC §7.2 with the item-11 duplicate-execution
  record on by default. The sealed `{type, document}` envelope matches the Rust
  crate, the Dart package and the Go module.
