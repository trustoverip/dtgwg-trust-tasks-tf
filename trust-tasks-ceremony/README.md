# trust-tasks-ceremony

Verification for **Trust Ceremonies** — the flow layer of [SPEC.md §4.11].

A ceremony composes several Trust Tasks into one flow. A
[`trust-ceremony-receipt`] attests that one *enactment* completed; this crate
checks that attestation.

```rust,ignore
use trust_tasks_ceremony::{verify, JcsSha256Digester, Outcome};

match verify(&receipt, &definition, definition_digest, recorder, &held, &JcsSha256Digester)? {
    Outcome::Complete            => { /* the rule is satisfied and a terminal step is present */ }
    Outcome::Incomplete { .. }   => { /* a prefix, or the rule is unsatisfied */ }
    Outcome::Invalid { .. }      => { /* the receipt contradicts itself or a document it names */ }
    Outcome::Unverifiable { .. } => { /* nothing was learned — not a failure */ }
}
```

## What it establishes, and what it cannot

A recorder attests **completeness and ordering, never step content**. This crate
checks the shape of an enactment against its pinned definition. It does not tell
you a step's payload meant what you hope — that is attested by the step's own
issuer through its own `proof`, verified separately with `trust-tasks-proof`.

Three things are deliberate:

- **`Unverifiable` is not `Invalid`.** A verifier that cannot resolve the
  definition has learned nothing.
- **The recorder's `complete` flag is never consulted.** Completion is decided by
  evaluating the definition's rule, and a receipt with no terminal step is a
  *prefix* however that flag is set. That is the truncation defence.
- **Holding no step documents is a supported case**, not a degraded one.

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
