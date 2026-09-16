// Data Integrity proofs for Trust Tasks are a SEPARATE module from
// trust-tasks-go, the way trust-tasks-proof is a separate crate from
// trust-tasks-rs and trust_tasks_proof a separate package from trust_tasks.
// Unlike the tsp module it pulls in no third-party code — it rolls the
// cryptosuites on the Go standard library (crypto/ed25519, crypto/ecdsa,
// crypto/sha256, crypto/sha512) plus a JCS canonicaliser and a base58btc codec
// written here — so its only requirement is the core module whose ProofVerifier
// seam it plugs into.
module github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/proof

go 1.22

require github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go v0.1.2
