# Shared consumer-pipeline vectors

Documents that **every** runtime's §7.2 pipeline runs: `trust-tasks-rs` (`tests/received_digest.rs`), `@openvtc/trust-tasks` (`test/envelope.test.ts`), `trust-tasks-go` (`trusttasks/item11_vectors_test.go`) and `trust_tasks` (`test/input_robustness_test.dart`). A change that makes one runtime disagree fails that runtime's CI.

`item11-digest.json` pins [SPEC §7.2](../../SPEC.md) item 11's *Keying and comparison*: two documents under one `id` are the same document only when their serializations are identical under RFC 8785 canonicalization. Each case consumes `first`, then `second`, against one replay guard, with a payload type that models only `payload.subject`, and passes each document to the pipeline as received. `expect` is the second outcome: `duplicate` (absorbed, the handler does not run again) or `idConflict`. The cases that differ only in a member the payload type drops, or in the spelling of a timestamp, are the ones a digest over the typed reading gets wrong.

The documents are stored as JSON text, not objects, so member order and whitespace reach each runtime exactly as written.
