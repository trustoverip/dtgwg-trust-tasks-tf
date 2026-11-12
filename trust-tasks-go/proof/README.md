# trust-tasks-go/proof

W3C Data Integrity proofs for [Trust Tasks](../../README.md), on the Go standard
library. A `ProofVerifier` for the core module's proof seam, and a signer that
produces what it verifies.

- **`eddsa-jcs-2022`** (Ed25519) and **`ecdsa-jcs-2019`** (P-256 / P-384), over
  the [JSON Canonicalization Scheme](https://www.rfc-editor.org/rfc/rfc8785)
  (RFC 8785).
- **`did:key` only** — the verification key travels in the identifier, so there
  is no network, no resolver, and no I/O. Good for self-issued documents and
  offline verification.
- **No third-party dependencies.** The cryptosuites are rolled on
  `crypto/ed25519`, `crypto/ecdsa` and the SHA-2 family; the JCS canonicaliser
  and the base58btc codec are written here. Its only requirement is the core
  `trust-tasks-go` module whose interface it satisfies. `@noble` (TypeScript),
  `ssi` (Dart) and `affinidi-data-integrity` (Rust) are the sibling backends;
  a test reproduces the shared `eddsa-jcs-2022` fixture's `proofValue` byte for
  byte, so a document signed by any of the four verifies with the others.

This is a **separate module** from the core (like `trust-tasks-go/tsp`), so the
core stays dependency-free.

## Install

```sh
go get github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/proof
```

## Verify

```go
import (
    "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/trusttasks"
    "github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/proof"
)

verifier := proof.NewVerifier() // did:key, both suites, 1-minute clock skew

// body is the document exactly as it arrived; doc is its decoded form.
outcome, err := trusttasks.ConsumeInbound(ctx, trusttasks.ConsumeOptions[P, R]{
    // ...
    Doc:         doc,
    Received:    body,
    ProofPolicy: trusttasks.ProofPolicy{Kind: trusttasks.ProofVerify, Verifier: verifier},
})
```

**Set `Received`.** Without it the proof is verified over a re-encoding of
`doc`, and a typed decode drops every payload member the Go type does not model,
so a validly signed document carrying such a member fails `proofInvalid`. With
it, the proof is verified over the bytes the producer signed.

`Verifier` binds the proof to the document's in-band `issuer`: a valid signature
proves only that *some* key signed, so that key's `did:key` must also be the
issuer (SPEC §4.7, §7.2 item 7). On failure it returns a typed `*proof.Error`
whose `Kind` (`malformedProof`, `issuerMismatch`, `unsupportedCryptosuite`,
`signatureInvalid`) is for **your logs only** — every failure reaches the wire as
the single `proofInvalid` code (SPEC §12.4).

The proof's `proofPurpose` must name a signing relationship — `authentication`,
`assertionMethod`, `capabilityInvocation` or `capabilityDelegation`, all of which
a `did:key` authorises; `keyAgreement` and any other value are refused
(SPEC §4.7.3). A proof carrying `expires` is refused once that instant has
passed, or when any of its fields is out of range (February 30, hour 24, a leap
second, an offset beyond ±14:00).

An Ed25519 key must be a canonically encoded point of large order.
`crypto/ed25519` accepts a non-canonical or small-order key, and a small-order
key satisfies the verification equation for any message with a chosen
signature, so the verifier checks the key itself (on `math/big`, no new
dependency) before verifying.

The document must be I-JSON (RFC 7493), as JCS (RFC 8785) requires. A document
that is not valid UTF-8, carries an unpaired surrogate escape (`"\ud800"`), has
two members with the same name in one object, or has anything but whitespace
after its closing brace fails as `malformedProof`. `encoding/json` would accept
each of these by rewriting or dropping part of the input, so the canonical form
the proof covers would differ from the bytes a handler reads.

## Sign

```go
signer, _ := proof.SignerFromPrivateKey(proof.Ed25519, seed) // 32-byte seed
signed, err := signer.Sign(docJSON, proof.SignOptions{})     // issuer must be signer.DID
```

`Sign` refuses a document whose `issuer` is absent or is not the signer's
`did:key` — such a document could never verify — and a `ProofPurpose` naming no
signing relationship. `ProofPurpose` defaults to `authentication`, the purpose
SPEC §4.7.3 recommends for a Trust Task document. `created` is written UTC,
`Z`-terminated, with no sub-second field (chrono's form), so the same key over
the same document and `created` reproduces the Rust crate's `proofValue`.

`SignerFromPrivateKey` takes an Ed25519 seed of 32 bytes, or an ECDSA scalar of
exactly 32 bytes (P-256) or 48 bytes (P-384) in the range [1, n-1]. Any other
key is refused with an error rather than producing a signer. `Sign` also
refuses a document the verifier would refuse as non-I-JSON.
