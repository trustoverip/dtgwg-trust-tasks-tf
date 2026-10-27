# Shared proof vectors

One set of signed Trust Task documents that **every** library's proof verifier runs:
`trust-tasks-proof` (Rust), `@openvtc/trust-tasks-proof` (TypeScript), `trust_tasks_proof` (Dart) and `trust-tasks-go/proof` (Go). A document one implementation signs is then checked by all four, and a change that makes one verifier disagree fails that verifier's CI.

`manifest.json` lists each file with the result a conforming verifier must reach (`valid` or `invalid`) and why. Entries are verified with a `did:key` resolver and nothing else; the proof is checked against the document's `issuer`.

The timestamp pair exists for [SPEC §4.7](../../SPEC.md#47-proof), "What a consumer verifies". `eddsa-jcs-2022-timestamp-spellings.json` is signed over `issuedAt: 2026-01-01T00:00:00.000Z` and `expiresAt: 2027-01-01T00:00:00+00:00`. A verifier that re-serializes a typed parse rewrites both to `…Z` and refuses a correctly signed document. Its `-normalized` twin is that rewrite with the original proof, and must fail.

All Ed25519 entries use the key of seed `[7; 32]` (`did:key:z6MkvDqGT54cXesYGvABpF1UapVNwjCqRcafi4Px6Thv5T3Z`). Ed25519 is deterministic, so the signers can reproduce each `proofValue`. To add an entry, sign it with any of the four signers, add it to `manifest.json`, and run every library's proof tests.
