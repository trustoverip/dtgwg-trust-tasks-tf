# Changelog — `trust_tasks_https`

All notable changes to the Dart HTTPS binding package.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
The package versions over **its own API** — what a consumer compiles against.
Below 1.0 a breaking change bumps the leading non-zero component.

Publishing is triggered by the `trust-tasks-dart-https-v<version>` tag, because
pub.dev only accepts an automated publish from a tag-triggered workflow. See
`RELEASING.md`.

## 0.3.1 — 2026-11-14

### Dependencies

- Requires `trust_tasks` ^0.12.0 (was ^0.11.0), the core released beside it.

## 0.3.0 — 2026-11-13


### Added

- **dart,dart-https,dart-didcomm**: JSON priorResponse, observability hook, forward routing and per-type size bounds (#1242)

* feat(dart,dart-https,dart-didcomm)!: JSON priorResponse, observability hook, forward routing and per-type size bounds



### Fixed

- **rs,https,dart-https**: Run the attribution gate before the route lookup (#1246)

Both HTTPS servers looked the route up before the attribution gate, so a sender with neither a bearer token nor a proof could tell a registered type (proofRequired) from an unregistered one (unsupportedType).

  The Rust server now runs on consume_dispatched_answer, so the reorder is made there: the checks that read only the envelope (expiry, recipient, freshness, identity) and the attribution gate move out of consume_core into envelope_checks, which consume_inbound and consume_dispatched_answer both run ahead of their item-3 type check. The route lookup still precedes the DID-method screen and proof verification, so an unregistered type never reaches a resolver.

  The Dart server runs the same checks before the route lookup and before decoding the payload.

### Dependencies

- Requires `trust_tasks` ^0.11.0 (was ^0.10.0), the core released beside it.

## 0.2.0 — 2026-11-12


### Fixed

- **ts,go,dart,rs**: Whole-second capability-client issuedAt, tested Dart core floors, TS default export condition, Go schema caching (#1214)

* fix(dart-capability-client,go-capabilityclient,go-didcomm): stamp issuedAt in whole seconds and fail on entropy failure

  The Dart capability client stamped issuedAt with toIso8601String (full
  precision); it now uses the core's systemClock. The Go port now stamps
  through trusttasks.SystemClock. The TS port already stamped whole seconds.

  The Go capabilityclient and didcomm UUIDv4 generators ignored crypto/rand
  errors and would mint an all-zero id; they now panic, as tsp's does.

- **dart-https**: Advertise the expiry horizon, and pin all four runtimes' freshness to shared vectors (#1212)

* test(runtime): shared freshness vectors run by all four runtimes

  fixtures/consume/freshness.json pins SPEC §7.2's acceptance window (items 4
  and 13, *Bounding the record*): now / policy / issuedAt / expiresAt to a
  verdict and, for an accepted document, the replay record's expiry. It covers
  a stated expiresAt replacing the maximum age, the 30-day expiry horizon and
  its clamp on the record, a zero horizon, the half-open edges to the
  millisecond, skew on both sides, a missing issuedAt with and without
  requireIssuedAt, offsets, fractions and calendar-invalid timestamps, and
  holds each runtime's default and consequential policies to the same values.

  Run by trust-tasks-rs (tests/freshness_vectors.rs), @openvtc/trust-tasks
  (test/freshness-vectors.test.ts), trust-tasks-go
  (trusttasks/freshness_vectors_test.go) and trust_tasks
  (test/freshness_vectors_test.dart). The Rust workflow now runs on a change
  under fixtures/, which its integration tests read.

- **dart-https**: Screen proof DID methods to did:key by default and hold error responses to the response verifier (#1207)

HttpsServer with a proofVerifier now accepts only did:key in
  proof.verificationMethod unless allowedDidMethods widens the set or
  allowAnyDidMethod: true lifts the screen (defaultAllowedDidMethods).
  A server with no verifier is unchanged.

  HttpsClient with a responseVerifier now requires a trust-task-error to carry
  a proof that verifies over the body as received, issued by serverVid to myVid.
  Otherwise it throws the new UnverifiedErrorResponseException, which keeps the
  status, the reason and the unverified document for diagnostics.

- **https,dart-https**: Cap HTTPS client responses, keep bearer tokens on https, stop following redirects, and keep response bodies out of errors (#1205)

* fix(https,dart-https)!: cap HTTPS client responses, keep bearer tokens on https, stop following redirects, and keep response bodies out of errors

  Rust HttpsClient and Dart HttpsClient:

  - The response body is read up to a cap (256 KiB by default,
    max_response_bytes / maxResponseBytes) and refused beyond it, with a new
    ResponseTooLarge error. A declared Content-Length over the cap is refused
    before reading.
  - server_url / base must be https. Plain http is accepted for a loopback
    host, or elsewhere with allow_insecure_http / allowInsecureHttp, but a
    bearer token is never sent over non-loopback http.
  - Redirects are not followed by default; a 3xx is returned as an HTTP status
    error so the signed document is never re-posted elsewhere. Rust can opt
    into same-origin redirects with follow_redirects(true); an https client
    never downgrades to http.
  - Error text no longer quotes the response body: HttpStatus keeps at most
    512 bytes in its field and leaves it out of Display; decode errors keep
    only the category and position.

  Dart only: every error the HTTP client raises (TLS included) is wrapped as
  TransportException; the timeout now covers reading the body and cancels the
  read when it fires; a responseVerifier requires a serverVid; a platform
  client that followed a redirect anyway is detected by its final URL.

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

### Dependencies

- Requires `trust_tasks` ^0.10.0 (was ^0.9.23), the core released beside it.

## 0.1.9 — 2026-11-10


### Added

- **acl**: Purpose of use on grants and on memory reads (AclEntry 0.3; vta/memory put/delete/list 0.3, get 0.2) (#1074)

* feat(acl): purpose of use on grants and on memory reads (AclEntry 0.3; vta/memory put/delete/list 0.3, get 0.2)

  An access-control entry says who may do what, where and until when, but not
  what for. An agent granted a read of its principal's memory can use the content
  for anything, and nothing records the use it intended. The VTI specification
  now requires the purpose (VTI-ACL-100 – 110, trustoverip/dtgwg-vti-spec#115);
  these are the wire shapes.

  acl/_shared/0.3 — AclEntry 0.3 adds one REQUIRED axis, `purposes`:
  `{"scope": "any"}` or `{"scope": "listed", "purposes": [...]}` with at least
  one `Purpose` — a lowerCamelCase name registered in the VTI purpose registry
  (personalisation, taskExecution, analytics, modelTraining) or an absolute IRI
  such as a DPV purpose. `any` never grants `modelTraining`. Delegated purposes
  are a subset of the granter's. CONVENTIONS.md §8a maps a 0.2 entry onto 0.3
  (`any`, the authority it already held, now stated) and §12 states the rules.

  acl tasks on AclEntry 0.3: grant/0.3, update/0.3, show/0.3, revoke/0.3,
  change-role/0.3, list/0.4. New errors on grant and update: `purposeOutsideDelegator`
  (purposes wider than the granter's) and `purposeRefused` (policy will not grant
  a purpose, refused by name, never dropped). update/0.3 replaces `purposes`
  wholesale; narrowing it is an audited privilege reduction. `purposes` is never
  redacted by show or list. No `extends`: a REQUIRED member on the entry cannot
  be added to a version a 0.2 consumer still reads.

  vta/memory put/0.3, delete/0.3, list/0.3, get/0.2 (each `extends` its
  predecessor) add an optional `purpose` (`MemoryPurpose`, new in
  vta/_shared/0.1/memory-record, a reference to acl's `Purpose`) and a
  `purposeNotGranted` error that discloses nothing and is distinct from
  `contextForbidden`. A recall, and a listing with values, is purpose-bound:
  refused for a purpose the entry does not grant, or for none under an entry
  listing several, and audited with its purpose either way. A write is not
  purpose-bound, but a conflict that returns `currentValue` is: the value is
  withheld unless the request's purpose is granted.

  Data rooms (a room authority credential's `purposes`, VTI-ACL-110) are a
  follow-up.

  Samples in other specs that named the superseded acl versions now name the new
  ones, with `purposes` on every embedded entry; prose links were moved by
  `npm run fix-stale-references`.

## 0.1.8 — 2026-11-04


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

## 0.1.7 — 2026-10-27


### Added

- **ts, go**: Trust-task-error/0.6 and the unsupportedExtension and indeterminate standard codes (#961)

* feat(ts)!: trust-task-error/0.6 and the unsupportedExtension and indeterminate standard codes

  Framework 0.7.0 added unsupportedExtension to the standard codes, and the
  indeterminate ruling (SPEC §8.3) adds indeterminate: the recipient could
  not decide because something it needed and does not control could not
  be obtained or checked, which is neither a refusal nor a failed
  verification. Both live in the closed code list of the trust-task-error
  payload schema, so they need a new version.

  - specs/trust-task-error/0.6: both codes appended to the enum, and
    extends: "0.5", which the build checks. References to 0.5 across the
    registry therefore stay current, and no other spec changes.
  - All four libraries recognize both codes and emit 0.6:
    trust_task_error_type_uri(), TRUST_TASK_ERROR_TYPE_URI,
    TrustTaskErrorTypeURI and trustTaskErrorTypeUri move together.
    Rust's StandardCode is hand-written (trust-task-error is in
    SKIP_SLUGS) and gains UnsupportedExtension and Indeterminate, with
    indeterminate retryable by default as §8.3 gives it.
  - The HTTPS binding's status table maps unsupportedExtension to 422 and
    indeterminate to 503, in the binding prose, trust-tasks-https and
    trust_tasks_https. The Dart package spells the new codes out so it
    still builds against a core that predates them.

  Breaking for @openvtc/trust-tasks only: StandardCode is a closed union
  there, so an exhaustive switch over it stops compiling. Rust's
  StandardCode is #[non_exhaustive] and Go's and Dart's are open string
  types, so the change is additive for them.

  Part of trustoverip/dtgwg-trust-tasks-spec#42.

## 0.1.6 — 2026-10-27


### Added

- The prebuilt specification modules are deprecated in all four libraries; generate the tasks you use (#954)

Milestone 5 of docs/design/consumer-side-generation.md. Every prebuilt module
  outside the framework's own specifications (trust-task-*, trust-ceremony-*,
  _framework/) is deprecated, so that an application is told to pin the tasks it
  uses with the trust-tasks CLI and generate them instead. Nothing is removed:
  the withdrawal batch that removes them is chosen later, at least 90 days after
  this ships.

  Each library marks the prebuilt entry point where its toolchain reports a
  deprecation, never the module body, which is also what the consumer-side
  generators emit:

  - Rust: #[deprecated] on each gated family module in trust-tasks-rs's
    specs/mod.rs (trust-tasks-codegen). It reaches every path and item under the
    module; framework modules and trust-tasks-build's output are unmarked.
  - TypeScript: the root index re-exports each module as
    `import * as _X …; /** @deprecated … */ export import X = _X;`, because
    TypeScript ignores @deprecated on `export * as`. API compatibility against
    0.27.8: nothing removed or changed. A deep import of a module file is not
    marked; TypeScript has no module-level deprecation.
  - Go: a `Deprecated:` paragraph in each prebuilt package's doc, which gopls and
    staticcheck (SA1019) report on import.
  - Dart: @Deprecated on each prebuilt library's `library` directive, which the
    analyzer reports on import from another package. This package's own tests
    use them, so analysis_options ignores the same-package diagnostic SDKs
    before 3.4 report.

  The TS, Go and Dart equivalence tests mask exactly the mark; every consumer
  generator is still held to the prebuilt tree as before. Hand-written Rust that
  uses prebuilt modules on purpose — the acl-specimen test modules, the
  integration tests and the examples — carries #[allow(deprecated)]; none of the
  runtime does.

  GETTING-STARTED, the four library READMEs, the design doc and CLAUDE.md say
  so, and point at each language's generator.

## 0.1.5 — 2026-10-26

## 0.1.4 — 2026-10-26


### Added

- **codegen**: Superseded and retired spec versions are deprecated in every library, and withdrawn ones leave them in a quarterly batch (#921)

* feat(go): mark superseded and retired spec versions deprecated, and omit withdrawn ones

  Every generated request package now exports `const Deprecation` (empty for a
  current version, otherwise "superseded by <slug>/<M.N>", "retired; use
  <slug>/<M.N>" or "retired"), and a deprecated version's Payload and Response
  types carry a `// Deprecated:` paragraph. A version with `status: withdrawn`
  gets no package at all. The rule is derived independently from front matter,
  per SPEC "Superseded and Withdrawn Versions".

## 0.1.3 — 2026-10-23


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

## 0.1.2 — 2026-10-23

## 0.1.1 — 2026-09-21

## 0.1.0

### Added

- Initial release: `HttpsClient` and a framework-agnostic `HttpsServer` for the
  Trust Tasks HTTPS binding (`bindings/https/0.2`), with a `dart:io` adapter in
  `package:trust_tasks_https/io.dart`.
