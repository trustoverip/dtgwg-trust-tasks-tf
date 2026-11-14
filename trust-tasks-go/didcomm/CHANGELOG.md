# Changelog

All notable changes to `trust-tasks-go/didcomm` are documented here. Entries are
generated from conventional commits by the release automation (see RELEASING.md).

## 0.3.3 — 2026-11-14


### Changed

- **go-tsp,go-didcomm,go-capabilityclient**: Share the core's ConsumeReceived and NewURNUUID (#1311)

The TSP and DIDComm consumers' Consume is now trusttasks.ConsumeReceived with the consumer's configuration; DIDComm's binding §3.1 thread cross-check is its BindingCheck. tsp, didcomm and capabilityclient mint ids with trusttasks.NewURNUUID; tsp.NewURNUUID and didcomm.NewMessageID remain as forwarders. Every module (and proof, and the getting-started example) requires core v0.12.0.

## 0.3.2 — 2026-11-14


### Fixed

- **go-didcomm,go-tsp**: Require core v0.11.1 so the message bindings enforce document size bounds (#1303)

The DIDComm and TSP nested modules already hand ConsumeInbound the exact received document bytes (#1263), but core v0.11.0 predates the per-type maxDocumentBytes check (#1251). Requiring v0.11.1 puts it in force. proof and capabilityclient move to v0.11.1 too.

  New tests in both modules: a document over its type's bound from an authenticated sender is answered malformedRequest to that sender, the handler does not run, and the replay record stays empty, so the same id within the bound is then handled.

## 0.3.1 — 2026-11-13


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

## 0.3.0 — 2026-11-12


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

- **go-didcomm**: Validate and bind the JWE header, and take the local DID from the opening identity (#1169)

* fix(go-didcomm)!: validate and bind the JWE header, and take the local DID from the opening identity

  UnpackTrustTask now checks the authcrypt JWE before opening it, following
  DIDComm v2 and RFC 7516:

  - apu must be base64url(skid) and apv base64url(SHA-256 of the sorted
    recipient kids). The per-recipient kids are unprotected; apv is what binds
    them to the protected header. A base64url error in either is refused
    instead of becoming an empty KDF input.
  - Every recipient entry must carry a kid, and only the entry whose kid is
    exactly the identity's Kid is opened. The single-unlabelled-entry fallback
    is gone.
  - Transport.Local is the DID of the identity the envelope was opened with,
    never the kid read from the envelope.
  - crit and zip are refused, as is a typ other than
    application/didcomm-encrypted+json, a shared unprotected header, aad, the
    flattened header/encrypted_key form, a per-recipient header repeating a
    protected parameter, and JSON member names differing only by case.
  - After decryption, a plaintext from that is not the authenticated sender is
    FailDecrypt, and a to that does not name the recipient is
    FailNotForThisRecipient.

  Envelopes packed by this module and by affinidi-messaging-didcomm (the
  interop fixture) still open.

- **go**: Bound base58 decoding and refuse a nil sender key in didcomm (#1166)

* fix(go): bound base58 decoding and refuse a nil sender key in didcomm

- **go**: Refuse member names that differ from a declared one only by case (#1161)

* fix(go): refuse member names that differ from a declared one only by case

  encoding/json matches object member names to struct fields without regard
  to case, so `{"issuer":"a","Issuer":"b"}` decoded with Issuer "b", while a
  proof verifier, a JSON Schema validator and the Rust, TypeScript and Dart
  runtimes all read the exact-case "issuer" and see "a". A document must mean
  one thing to every reader.

  - trusttasks: Document and Proof decoding now refuse a member whose name is
    not one the envelope, the proof or the payload type declares but which
    encoding/json would match to one (case, Unicode simple folding, and the
    '_'/'-' insensitivity of encoding/json/v2's matcher). The check follows
    the payload type into nested structs, slices, arrays and map values.
    CheckMemberNames[T] exposes the same check for a payload decoded outside
    a Document.
  - Because ConsumeInbound decodes Received when checking that Doc is a
    reading of it, a Received carrying such a member is now refused
    proofInvalid instead of vouching for a differently-decoded Doc.
  - proof: the Data Integrity verifier refuses a document or proof object
    carrying such a member (malformedProof), since the issuer binding reads
    the exact-case member.

  The didcomm and tsp modules pick the envelope check up when their core
  requirement moves to a release containing it.

## 0.2.1 — 2026-10-23


### Fixed

- **go**: The DIDComm and TSP consumers leave the replay claim to the core (#896)

Core v0.9.3 releases the claim itself when a handler returns an error that
  is not a Refusal (#876), so the didcomm and tsp modules now require it and
  no longer release a second time. TestConsumerHandlerErrorReleasesClaim
  still passes, now on the core's release.

## 0.2.0 — 2026-09-16


### Added

- **go-didcomm**: Make DIDComm authcrypt interoperate with the Rust/Dart bindings (#504)

The Go DIDComm binding rolled its own ECDH-1PU authcrypt but did not match the
  DIDComm ecosystem's key derivation, so a JWE it produced could not be opened by
  affinidi-messaging-didcomm (what the Rust and Dart bindings use), and vice
  versa. Two mismatches, both fixed:

  - ECDH-1PU is tag-in-KDF (draft-madden §2.3): encrypt the content first, then
    derive the key-wrapping KEK with the content-encryption tag as a
    length-prefixed SuppPrivInfo. The binding derived the KEK without the tag.
  - apv is base64url(SHA-256(sorted recipient kids joined by ".")), not the raw
    recipient kid. (apu = base64url(skid) already matched.)

  aries-askar, didcomm-python, didcomm-rust and affinidi all do the tag-in-KDF
  derivation; it is the de-facto DIDComm v2 definition. The reworked crypto now
  interoperates both directions, asserted against affinidi:

  - affinidi → Go: interop_test.go unpacks a JWE packed by affinidi
    (testdata/authcrypt-interop.fixture.json).
  - Go → affinidi: a companion Rust test in trust-tasks-didcomm unpacks a Go JWE.

  This changes the wire format, so it is a breaking change: v0.1.0 → v0.2.0. The
  existing round-trip and negative tests still pass; the RFC 3394 key-wrap vector
  is unchanged. Documented in CLAUDE.md and the module README/testdata.

