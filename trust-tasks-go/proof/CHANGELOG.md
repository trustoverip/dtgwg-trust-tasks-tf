# Changelog

All notable changes to `trust-tasks-go/proof` are documented here. Entries are
generated from conventional commits by the release automation (see RELEASING.md).

## 0.2.0 — 2026-11-12


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

- **proof,ts-proof,go-proof,dart-proof**: Follow the JCS cryptosuites' @context rule in all four proof backends (#1233)

Signers copy a document's @context into the proof (eddsa-jcs-2022 /
  ecdsa-jcs-2019 section 3.3.1 step 2). Verifiers hash the proof
  configuration as received, and refuse a proof whose @context is not
  exactly the document's (section 3.3.2 step 4, held to equality because a
  ProofVerifier returns no verified document). The Dart verifier no longer
  lets ssi copy the document's @context into the configuration, which made
  it refuse what the other three signers emit. Documents without @context
  get byte-identical proofs. Six shared vectors in fixtures/proof pin the
  behaviour in every library.

- **ts,go,dart,rs**: Whole-second capability-client issuedAt, tested Dart core floors, TS default export condition, Go schema caching (#1214)

* fix(dart-capability-client,go-capabilityclient,go-didcomm): stamp issuedAt in whole seconds and fail on entropy failure

  The Dart capability client stamped issuedAt with toIso8601String (full
  precision); it now uses the core's systemClock. The Go port now stamps
  through trusttasks.SystemClock. The TS port already stamped whole seconds.

  The Go capabilityclient and didcomm UUIDv4 generators ignored crypto/rand
  errors and would mint an all-zero id; they now panic, as tsp's does.

- **rs,ts,go,dart,proof,ts-proof,go-proof,dart-proof**: Refuse out-of-range timestamp fields instead of rolling them over (#1210)

issuedAt and expiresAt, and a proof's created, are now read with every
  field range-checked after the shared pattern: month 1-12, a day the month
  has, hour 0-23, minute and second 0-59 (no leap second), and an offset of
  at most 23:59 (14:00 for a proof's dateTimeStamp, as for expires).

  Before, the four runtimes disagreed on the same bytes. TypeScript's
  Date.parse read 2026-02-30 as 2 March and 24:00 as the next day; Dart's
  DateTime.tryParse rolled 2026-13-45T25:61:61Z over into 2027 and took any
  offset; Rust's chrono read :60 as a leap second and accepted a space for
  the T; Go's time.Parse accepted +24:00 and +23:60. In the TypeScript proof
  verifier a regex-valid but unparseable created (month 13) gave NaN, which
  never compares later than the clock-skew horizon, so it was accepted.

  New shared vectors: fixtures/consume/timestamps.json, run by all four
  runtimes, and five correctly signed created-* proof vectors.

- **go-proof**: Decode documents as I-JSON and refuse out-of-range ECDSA private keys (#1208)

The JCS decoder now refuses input that encoding/json would accept only by
  rewriting or dropping part of it: invalid UTF-8, an unpaired surrogate
  escape, two members with the same name in one object, and data after the
  first JSON value. Such input canonicalized to the same bytes as a different
  document, so one proof covered both. The verifier reports it as
  malformedProof; Sign returns an error.

  SignerFromPrivateKey now requires an ECDSA scalar of the curve's exact byte
  length in [1, n-1], and derives the public key through crypto/ecdh instead
  of elliptic.Curve.ScalarBaseMult. A zero, short, long or out-of-range key
  is refused at construction instead of producing a signer.

- **go-proof, ts-proof, dart-proof**: Check proofPurpose and expires, and sign with authentication by default (#1168)

* fix(go-proof, ts-proof, dart-proof)!: check proofPurpose and expires, and sign with authentication by default

  The Go, TypeScript and Dart proof verifiers accepted any non-empty
  proofPurpose, including keyAgreement, and none of the four read a proof's
  expires. They now apply the rule trust-tasks-proof (Rust) already applies
  (SPEC §4.7.3):

  - proofPurpose must be authentication, assertionMethod,
    capabilityInvocation or capabilityDelegation; keyAgreement and any other
    value are malformedProof.
  - Dart's withResolver path checks that the issuer's DID document lists the
    exact verificationMethod under the relationship proofPurpose names (by
    absolute reference, relative fragment, or embedded), defines it under
    verificationMethod, and names the issuer as its controller. The document
    is resolved once per verification and shared with ssi.
  - expires, when present, must be a dateTimeStamp (Z or an explicit offset)
    that has not passed; no clock-skew allowance. All four runtimes.
  - The Go, TypeScript and Dart signers default proofPurpose to
    authentication, as the Rust signer does, and refuse a purpose naming no
    signing relationship.
  - TypeScript verifies Ed25519 with RFC 8032 decoding ({ zip215: false }),
    as Go, Rust and Dart do.

  Three shared vectors (fixtures/proof) are correctly signed and must fail in
  every runtime: proofPurpose keyAgreement, an unrecognised proofPurpose, and
  a passed expires.

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

## 0.1.4 — 2026-10-27

## 0.1.3 — 2026-10-22


### Fixed

- **trust-tasks-ts-proof**: Ecdsa-jcs-2019 hashes once and accepts high-S, so P-256/P-384 proofs interoperate (#874)

signTrustTask passed the suite digest of hashData to @noble/curves, whose
  `prehash` defaults to true, so it hashed again: TS ECDSA proofs were
  signatures over SHA(SHA(hashData)), which verified only against this
  package. The verifier made the same mistake, so it rejected every standard
  ecdsa-jcs-2019 proof — including the Go backend's — and its default `lowS`
  would also have refused the high-S signatures Go's ecdsa.Sign emits about
  half the time.

  Sign and verify now pass `prehash: false` (the digest is the ECDSA message
  hash), and verify passes `lowS: false`. Ed25519 is unaffected.

## 0.1.2 — 2026-10-01

## 0.1.1 — 2026-09-21

