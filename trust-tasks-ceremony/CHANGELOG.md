# Changelog

All notable changes to `trust-tasks-ceremony` are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

**Versioning follows [Semantic Versioning](https://semver.org/) over this
library's own API — not the framework's.**

## [0.2.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/compare/trust-tasks-ceremony-v0.2.0...trust-tasks-ceremony-v0.2.1) — 2026-11-12


### Documentation

- Fix drifted counts, release runbook, licences and stale examples across the four libraries ([#1224](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/1224))

* docs(getting-started): move the worked example to acl/grant/0.3 and stamp issuedAt with issued_at_now()

  The example pinned acl/grant/0.2, superseded by 0.3, and stamped issuedAt
  with chrono::Utc::now(), the full-precision form #970 removed from the
  libraries. The TS consumer example wrote toISOString() milliseconds into
  document fields the same way. The dependency snippet now uses cargo add,
  so it cannot name a stale version, and §4.4 describes the shared version
  group instead of versions from 0.14.



### Fixed

- **ceremony**: Decode the salt as multibase in every build, and compare digests by their decoded bytes ([#1232](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/1232))

A default-features = false build hashed the salt's string bytes, so its digests changed whenever anything else in the graph enabled affinidi. trust-ceremony-receipt/0.1 defines the salt as decoded from multibase; multibase is now an unconditional dependency and the salt is decoded under every feature set.

  Step, prev and definition digests are compared as decoded multihash bytes (length-bounded, constant-time over equal lengths), so the same digest in another multibase base matches, and a malformed digest never matches a well-formed one. The fixture suite now runs without the affinidi backend, and CI runs it that way.

- **proof, ts-proof, dart-proof, dart-didcomm, ceremony**: Bound multibase decoding of unauthenticated values ([#1167](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/1167))

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



## [0.2.0] - 2026-08-26

**Breaking.** Receipts that verified as `Outcome::Complete` under 0.1.x now
return `Outcome::Unverifiable` unless the caller holds the terminal step's
document and supplies the enactment's role bindings. This is deliberate: both
were being credited as checked when neither was.

### Security

- **A recorder can no longer mint the terminal marker.** `verify()` read
  `ReceiptStep.terminal` — a boolean on the *receipt*, written by the recorder —
  and `DefinitionStep.terminal` was parsed and never consulted. The
  `trust-ceremony-receipt/0.1` specification states that a recorder "cannot mint
  that marker without the terminal step issuer's key"; the implementation let it
  do exactly that, so a truncated prefix could be presented as a completed
  enactment. `terminal` is now accepted only on a step the **definition**
  declares terminal, and is confirmed against the **signed** `ceremony.terminal`
  of a held step document. Where no terminal step document is held, the outcome
  is `Unverifiable` naming what could not be checked — `Complete` now means the
  verifier saw the marker.
- **Thresholds are no longer inflatable by repetition.** Instances were counted
  with `*instances.entry(..) += 1` over every receipt entry, so listing one
  approver's step twice satisfied `threshold.ofStep` with `n = 2`. Duplicate
  document `id`s are rejected, as are two entries for one step at one round —
  and, for a `perRole` step, two entries at one round from one issuer. Counting
  happens only after de-duplication.
- **A step issuer is now bound to the role the definition names.** De-duplication
  alone does not close a threshold: a recorder that cannot list one approver
  twice can invent a second. Verification against the enactment's role bindings
  is what makes a distinct issuer a distinct *authorised party*.

### Added

- `Bindings` — the enactment's role → VID map, and a new `verify()` argument.
  `evidence.recorders` and a step's `issuer`/`recipient` are **role names**, so
  neither the recorder check nor the issuer check was answerable without them;
  the recorder check previously compared a VID against a role name. Pass
  `Bindings::unbound()` where you do not have them and the affected rules are
  reported as unchecked rather than assumed to pass.
- The `prev` chain walk (design note §7.9 step 4), over held documents: a
  document's signed `ceremony.prev` must name a predecessor the receipt
  enumerates, with the digest the receipt gives it, and that predecessor's step
  must be an ancestor of this one in the definition's `prev` graph (transitively,
  so an enactment that skips an optional step still verifies). This is what
  carries the specification's claim that an omitted intermediate step is caught
  because "its successor committed to its digest".
- Envelope agreement for held documents (§7.9 step 1): `ceremony.enactment`,
  `ceremony.definitionDigest`, `ceremony.step`, `ceremony.round` and the
  document's `issuer` and `recipient` must agree with what the receipt says.
- `Definition.roles`, and `DefinitionStep`'s `kind`, `issuer`, `recipient`,
  `multiplicity`, `prev` and `optional`, which were not parsed and so could not
  have been checked however `verify()` was written. `StepKind`, `Multiplicity`
  and `DefinitionRole` are new public types.
- Recorder conformance rule 5 is enforced: a receipt whose steps carry `prev`
  must carry the enactment salt.

### Changed

- `verify()` takes `bindings: &Bindings` between `recorder` and `held`.
- `Outcome::Unverifiable` now also reports a *partial* inability to verify — an
  unheld terminal document, unsupplied role bindings, or a nested ceremony step —
  rather than only an unresolvable definition. A receipt that would otherwise be
  `Complete` is downgraded to `Unverifiable`; one that is `Invalid` or
  `Incomplete` still reports that, since those are findings rather than gaps.

### Known limitations

Stated rather than silently skipped, and now disclaimed in the specification:

- A **nested ceremony** step (`kind: ceremony`) is not recursed into. The child's
  evidence is its own receipt, verified on its own terms; a receipt containing
  one is reported `Unverifiable` rather than passed as `Complete`.
- **`maxDuration`** is not evaluated. The receipt payload carries no `issuedAt`,
  so the issuance window is not derivable from a receipt.
- The chain walk and the terminal marker are checkable only over the documents
  the caller holds. A verifier holding none checks the recorder's enumeration and
  the shape of the flow, which is what that position is worth.

## [0.1.1] - 2026-08-10

### Changed

- Dropped the `trust-tasks-rs` dependency, which was never used. `verify()`
  takes envelope facts (the recorder) as parameters rather than a `TrustTask`,
  deliberately — the party that signed a receipt is an envelope fact and a
  payload could claim anything — so the crate never needed the envelope type.
  It is therefore unaffected by envelope version changes.

## [0.1.0] - 2026-08-10

### Added

- Initial release. Verification for Trust Ceremonies: the `Digester` trait and
  its default JCS/SHA-256 backend, the completion-predicate evaluator, and
  `verify()` implementing the verifier conformance rules of
  `trust-ceremony-receipt/0.1`.
- `Outcome::Unverifiable` is distinct from a failure: a verifier that cannot
  resolve or match the definition has learned nothing, which is not the same as
  having learned the receipt is bad.
- The recorder's own `complete` flag is never consulted — completion is decided
  by evaluating the definition's rule, and a receipt with no terminal step is
  reported as a prefix however that flag is set.
