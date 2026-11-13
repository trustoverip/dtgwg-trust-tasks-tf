# Shared proof vectors

One set of signed Trust Task documents that **every** library's proof verifier runs:
`trust-tasks-proof` (Rust), `@openvtc/trust-tasks-proof` (TypeScript), `trust_tasks_proof` (Dart) and `trust-tasks-go/proof` (Go). A document one implementation signs is then checked by all four, and a change that makes one verifier disagree fails that verifier's CI.

`manifest.json` lists each file with the result a conforming verifier must reach (`valid` or `invalid`) and why. Entries are verified with a `did:key` resolver and nothing else; the proof is checked against the document's `issuer`.

The timestamp pair exists for [SPEC §4.7](../../SPEC.md#47-proof), "What a consumer verifies". `eddsa-jcs-2022-timestamp-spellings.json` is signed over `issuedAt: 2026-01-01T00:00:00.000Z` and `expiresAt: 2027-01-01T00:00:00+00:00`. A verifier that re-serializes a typed parse rewrites both to `…Z` and refuses a correctly signed document. Its `-normalized` twin is that rewrite with the original proof, and must fail.

The `purpose-` and `expired` entries are correctly signed and must still fail: a `proofPurpose` that names no signing relationship (`keyAgreement`, or a value naming none) is refused before any key is used, and a proof past its `expires` is refused whatever its signature ([SPEC §4.7.3](../../SPEC.md#473-proof-purpose-and-verification-relationship)).

The `expires-` entries carry an `expires` whose fields are out of range (February 30, hour 24, a leap second, offsets beyond ±14:00 or with minute 60). Each runtime checks the ranges itself after the shared pattern, because each language's date parser accepts or rolls over a different subset. `expires-future` is valid, with a Rust exception: see `exceptions` in its manifest entry.

The `created-` entries carry a `created` out of range in the same ways (February 30, hour 24, month 13, a leap second, an offset of 24 hours). Every runtime applies the `expires` ranges to `created` too. Before, TypeScript accepted month 13, because `Date.parse` gives `NaN` and `NaN` is never later than the clock-skew horizon. Dart rolled it over, and Go accepted `+24:00`.

The `small-order-key`, `non-canonical-key` and `mixed-order-r` entries pin Ed25519 strictness. Every verifier refuses a public key of small order or a non-canonical encoding, and checks the cofactorless equation.

An entry's optional `exceptions` object names a runtime whose result differs today, with the reason. That runtime's test asserts the exception, so the test fails once the gap is fixed and the exception can be deleted.

All Ed25519 entries except `small-order-key` and `non-canonical-key` use the key of seed `[7; 32]` (`did:key:z6MkvDqGT54cXesYGvABpF1UapVNwjCqRcafi4Px6Thv5T3Z`). Ed25519 is deterministic, so the signers can reproduce each `proofValue`. To add an entry, sign it with any of the four signers, add it to `manifest.json`, and run every library's proof tests.
