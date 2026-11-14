# Changelog — `trust_tasks_didcomm`

All notable changes to the Dart DIDComm binding package.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against.
Below 1.0 a breaking change bumps the leading non-zero component.

Publishing is triggered by the `trust-tasks-dart-didcomm-v<version>` tag,
because pub.dev only accepts an automated publish from a tag-triggered
workflow. See `RELEASING.md`.

## 0.2.1 — 2026-11-14

### Dependencies

- Requires `trust_tasks` ^0.12.0 (was ^0.11.0), the core released beside it.

## 0.2.0 — 2026-11-13


### Added

- **dart,dart-https,dart-didcomm**: JSON priorResponse, observability hook, forward routing and per-type size bounds (#1242)

* feat(dart,dart-https,dart-didcomm)!: JSON priorResponse, observability hook, forward routing and per-type size bounds



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

### Dependencies

- Requires `trust_tasks` ^0.11.0 (was ^0.10.0), the core released beside it.

## 0.1.4 — 2026-11-12


### Fixed

- **ts,go,dart,rs**: Whole-second capability-client issuedAt, tested Dart core floors, TS default export condition, Go schema caching (#1214)

* fix(dart-capability-client,go-capabilityclient,go-didcomm): stamp issuedAt in whole seconds and fail on entropy failure

  The Dart capability client stamped issuedAt with toIso8601String (full
  precision); it now uses the core's systemClock. The Go port now stamps
  through trusttasks.SystemClock. The TS port already stamped whole seconds.

  The Go capabilityclient and didcomm UUIDv4 generators ignored crypto/rand
  errors and would mint an all-zero id; they now panic, as tsp's does.

- **runtime**: Digest item 11 over the received document, and check envelope types and nesting depth (#1171)

* fix(runtime)!: digest item 11 over the received document, and check envelope types and nesting depth

  Item 11's duplicate-execution digest is now taken over the document as
  received wherever a pipeline holds it, so two documents under one id that
  differ only in a member the payload type does not model, or in the spelling
  of a timestamp, are an idConflict rather than a duplicate:

  - Go ConsumeInbound digests ConsumeOptions.Received when set.
  - Dart consumeInbound digests `received` whenever it is passed (it did only
    for a forward read), and the HTTPS server, DIDComm and TSP consumers pass it.
  - Rust consume_inbound / consume_dispatched digest the received JSON under
    ProofPolicy::VerifyReceived, with or without a proof; the HTTPS server
    digests the body. New received_document_digest.

  TypeScript consumeInbound now checks the envelope before anything reads it:
  member types (string id/type and string-valued optional members, object
  payload/proof/ceremony, @context string/array/object) and a nesting bound of
  128 levels, refused malformedRequest and addressed only from string members.
  validateEnvelope and MAX_NESTING_DEPTH are exported.

  Dart consumeInbound refuses a document nested beyond maxNestingDepth (128)
  with malformedRequest instead of overflowing the stack, and the new
  parseTrustTaskDocument decodes a received value without throwing, returning
  a malformedRequest reason for a mistyped member.

  Shared vectors in fixtures/consume/item11-digest.json run in all four
  runtimes.

- **proof, ts-proof, dart-proof, dart-didcomm, ceremony**: Bound multibase decoding of unauthenticated values (#1167)

* fix(proof, ts-proof, dart-proof): bound base58 decoding of proofValue and did:key

  The Rust, TypeScript and Dart proof verifiers decode a proof's proofValue
  and its did:key multikey with base58btc decoders whose cost grows with
  the square of the input length, and they do so before any signature is
  checked. A 1 MiB value took about 50 s per field in trust-tasks-proof
  (release build), and would take minutes in the Dart package and in the
  TypeScript package on @scure/base 2.0-2.3 (2.4 refuses over 4096
  characters itself).

  Each verifier now refuses a base58btc value longer than 175 characters
  before decoding it, or one that decodes to more than 128 bytes - the
  bounds the Go module adopts in #1166. The largest value any of them reads
  is a 96-byte P-384 signature; a did:key multikey is at most 51 bytes;
  175 = ceil(128 * log 256 / log 58).

  Nothing that could verify before is refused: a value over the bounds
  always decoded to more bytes than any supported key or signature.

### Dependencies

- Requires `trust_tasks` ^0.10.0 (was ^0.9.23), the core released beside it.

## 0.1.3 — 2026-11-04


### Added

- **trust-task-discovery**: Publish 0.4, a responder advertises the horizon up to which it honours a stated expiresAt (#1030)

SPEC §7.2 (Bounding the record) applies a consumer's maximum age only where a
  document states no expiresAt, and since trust-tasks-rs 0.37 (#1021)
  validate_freshness does so, honouring a stated expiry up to the policy's
  expiry_horizon. discovery 0.3 cannot describe that: its window "governs
  issuedAt only" and its responder refuses any document past
  issuedAt + maxAgeSeconds + clockSkewSeconds, whatever expiresAt it states. A
  responder that honours an expiry could only advertise a narrower window than
  it applies, and a producer could not learn that stating an expiry keeps a long
  delivery acceptable.

  - specs/trust-task-discovery/0.4: acceptanceWindow gains an optional
    expiryHorizonSeconds (whole seconds, at least 1), and the spec defines a
    document's acceptance end: issuedAt + maxAgeSeconds without an expiresAt;
    the earlier of expiresAt and issuedAt + expiryHorizonSeconds with one; and,
    where the window carries no horizon, the earlier of expiresAt and
    issuedAt + maxAgeSeconds, which is what a 0.3 window says.
    - Responder: MUST NOT advertise a horizon wider than it applies, nor one at
      all if it applies its maximum age whatever expiresAt states.
    - Discoverer: send bounds follow the acceptance end. MAY state an expiresAt
      within the horizon, no later than it will pursue delivery, instead of
      issuing a chain of new attempts; a new attempt carries it unchanged.
      Absent a horizon, it MUST assume the maximum age applies to every
      document.
    - AcceptanceWindow is closed in 0.3, so an added optional member is a MINOR
      increment (§5.2): a new version, not an edit.
    - Response-side invalid examples: a zero horizon, a duration string, and a
      fractional entry-level horizon.
  - trust-tasks-rs: DiscoveryRegistry::respond_to_v0_4 is the current answer.
    acceptance_window_from reads the policy's expiry_horizon, and
    expiry_horizon() sets one by hand. respond_to_v0_3 moves to
    discovery/superseded.rs and states the window without the horizon (narrower
    than applied, which 0.3 permits).
  - trust-tasks-https: enable_discovery answers 0.4, and 0.3 beside 0.2 and 0.1.
  - trust-tasks-dart-https: enableDiscovery answers 0.4. Dart's FreshnessPolicy
    has no horizon, so its window states none, which says the maximum age
    applies to every document. 0.3 moves to the superseded block.
  - Examples and tests that named the 0.3 types use 0.4.
  - Bindings regenerated (Rust, TS, Go, Dart). 0.3 is superseded.

  Follows from #1021 and trustoverip/dtgwg-vti-spec#96 (VTI-OPS-024,
  VTI-OPS-028, VTI-TRN-048), whose implementation in
  OpenVTC/verifiable-trust-infrastructure states delivery deadlines as expiresAt.

## 0.1.2 — 2026-10-26


### Added

- **codegen**: Superseded and retired spec versions are deprecated in every library, and withdrawn ones leave them in a quarterly batch (#921)

* feat(go): mark superseded and retired spec versions deprecated, and omit withdrawn ones

  Every generated request package now exports `const Deprecation` (empty for a
  current version, otherwise "superseded by <slug>/<M.N>", "retired; use
  <slug>/<M.N>" or "retired"), and a deprecated version's Payload and Response
  types carry a `// Deprecated:` paragraph. A version with `status: withdrawn`
  gets no package at all. The rule is derived independently from front matter,
  per SPEC "Superseded and Withdrawn Versions".

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

### Added

- Initial release: `packTrustTask` / `unpackTrustTask` for authcrypt DIDComm
  v2.1 envelopes (`bindings/didcomm/0.2`), and `DidcommConsumer`, the guarded
  inbound path with the §7.2 item 11 duplicate-execution record on by default.
