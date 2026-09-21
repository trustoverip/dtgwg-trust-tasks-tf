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

outcome, err := trusttasks.ConsumeInbound(ctx, trusttasks.ConsumeOptions[P, R]{
    // ...
    ProofPolicy: trusttasks.ProofPolicy{Kind: trusttasks.ProofVerify, Verifier: verifier},
})
```

`Verifier` binds the proof to the document's in-band `issuer`: a valid signature
proves only that *some* key signed, so that key's `did:key` must also be the
issuer (SPEC §4.7, §7.2 item 7). On failure it returns a typed `*proof.Error`
whose `Kind` (`malformedProof`, `issuerMismatch`, `unsupportedCryptosuite`,
`signatureInvalid`) is for **your logs only** — every failure reaches the wire as
the single `proofInvalid` code (SPEC §12.4).

## Sign

```go
signer, _ := proof.SignerFromPrivateKey(proof.Ed25519, seed) // 32-byte seed
signed, err := signer.Sign(docJSON, proof.SignOptions{})     // issuer must be signer.DID
```

`Sign` refuses a document whose `issuer` is absent or is not the signer's
`did:key` — such a document could never verify. `created` is written UTC,
`Z`-terminated, with no sub-second field (chrono's form), so the same key over
the same document and `created` reproduces the Rust crate's `proofValue`.
