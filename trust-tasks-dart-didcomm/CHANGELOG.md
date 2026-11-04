# Changelog — `trust_tasks_didcomm`

All notable changes to the Dart DIDComm binding package.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against.
Below 1.0 a breaking change bumps the leading non-zero component.

Publishing is triggered by the `trust-tasks-dart-didcomm-v<version>` tag,
because pub.dev only accepts an automated publish from a tag-triggered
workflow. See `RELEASING.md`.

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
