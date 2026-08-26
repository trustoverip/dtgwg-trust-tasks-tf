# trust-tasks-ceremony

Verification for **Trust Ceremonies** — the flow layer of [SPEC.md §4.11].

A ceremony composes several Trust Tasks into one flow. A
[`trust-ceremony-receipt`] attests that one *enactment* completed; this crate
checks that attestation.

```rust,ignore
use trust_tasks_ceremony::{verify, Bindings, JcsSha256Digester, Outcome};

// The enactment's role → VID bindings. `evidence.recorders` and a step's
// `issuer` are ROLE NAMES, so without these neither can be checked against a
// VID on the wire. `Bindings::unbound()` says you do not have them, and the
// affected rules are then reported rather than assumed.
let bindings = Bindings::unbound()
    .with("applicant", ["did:web:applicant.example"])
    .with("community", ["did:web:community.example"]);

match verify(&receipt, &definition, definition_digest, recorder, &bindings, &held, &JcsSha256Digester)? {
    Outcome::Complete            => { /* the rule is satisfied AND a held document carries the signed terminal marker */ }
    Outcome::Incomplete { .. }   => { /* a prefix, or the rule is unsatisfied */ }
    Outcome::Invalid { .. }      => { /* the receipt contradicts itself, the definition, or a document it names */ }
    Outcome::Unverifiable { .. } => { /* something could not be checked — not a failure */ }
}
```

## What it establishes, and what it cannot

A recorder attests **completeness and ordering, never step content**. This crate
checks the shape of an enactment against its pinned definition. It does not tell
you a step's payload meant what you hope — that is attested by the step's own
issuer through its own `proof`, verified separately with `trust-tasks-proof`.

Four things are deliberate:

- **`Unverifiable` is not `Invalid`.** A verifier that could not check something
  has learned nothing about it, which is not the same as having learned the
  receipt is bad.
- **The recorder's `complete` flag is never consulted.** Completion is decided by
  evaluating the definition's rule, and a receipt with no terminal step is a
  *prefix* however that flag is set. That is the truncation defence.
- **`terminal` is signed by the step's issuer, never asserted by the recorder.**
  The whole force of the truncation defence is that a recorder cannot mint the
  marker without the terminal step issuer's key, so `Complete` requires a held
  step document whose `ceremony.terminal` is true on a step the definition
  declares terminal. The `terminal` field on a receipt entry is the recorder's
  echo, checked against the signed value and never accepted in its place.
- **Holding no step documents is a supported case**, not a degraded one. It is
  also the case in which the terminal marker and the `prev` chain cannot be
  checked at all, so the outcome is `Unverifiable` naming what was missing rather
  than a completion claim resting on the party it exists to catch.

## What it checks, against §7.9 of the design note

| § | Check | Here |
|---|---|---|
| 1 | Group by `enactment`, reject disagreement on `definitionDigest` | over held documents |
| 2 | Resolve each `step` in the definition | yes |
| 3 | `round` within the repetition bound | yes |
| 4 | Walk `prev`, check the salted digests | over held documents |
| 5 | `issuer` / `recipient` against the definition's roles | yes, given `Bindings` |
| 6 | Recurse into a nested ceremony step | **no** — reported `Unverifiable` |
| 7 | Evaluate the completion predicate | yes, over de-duplicated instances |
| 8 | Confirm a terminal step | yes, from the *signed* marker |

`maxDuration` is not evaluated: the receipt payload carries no `issuedAt`, so the
issuance window is not derivable from a receipt.

## Digests

```
digestMultibase = multibase(multihash(H(JCS(document) ‖ salt)))
```

over the document **including its `proof`**, with the salt as a **suffix**.

The default backend canonicalizes with `serde_json_canonicalizer` — the same
RFC 8785 implementation `affinidi-data-integrity` uses for `eddsa-jcs-2022`, so
a ceremony digest and a Data Integrity proof over one document agree on what
that document is. Swap it by implementing `Digester`.

[SPEC.md §4.11]: https://trusttasks.org/SPEC#411-the-ceremony-member
[`trust-ceremony-receipt`]: https://trusttasks.org/spec/trust-ceremony-receipt/0.1
