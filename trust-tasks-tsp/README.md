# trust-tasks-tsp

The ToIP **Trust Spanning Protocol** (TSP) transport binding for the
[Trust Tasks](https://trusttasks.org/) framework, built on
[`affinidi-tsp`](https://crates.io/crates/affinidi-tsp). It implements
[`bindings/tsp/0.1`](https://trusttasks.org/bindings/tsp/0.1)
(binding URI `https://trusttasks.org/binding/tsp/0.1`).

A Trust Task document rides as the authenticated, encrypted payload of a TSP
message — sealed with HPKE authenticated encryption and signed from the
producer's *Verifiable Identifier* (VID) to the recipient's. On unwrap, the
authenticated sender VID becomes the framework's transport-authenticated sender
for [SPEC §4.8.1](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity)
precedence. TSP has no anonymous sender mode, so every envelope this binding
accepts carries a verified sender, surfaced verbatim (a TSP VID *is* the
framework VID).

```sh
cargo add trust-tasks-tsp
```

or, through the facade, `cargo add trust-tasks --features tsp`
and use `trust_tasks::tsp`.

## Wire shape

The TSP message payload is the JSON serialisation of the binding envelope:

```json
{
  "type": "https://trusttasks.org/binding/tsp/0.1/envelope",
  "document": { "…": "the full Trust Task document" }
}
```

The same `{type, document}` object is sealed by the TypeScript, Go and Dart TSP
bindings.

## API

| Item | What it does |
|---|---|
| `pack_trust_task(&doc, &sender, &recipient)` | Seal a document as a **Direct** TSP message from a `PrivateVid` to a `ResolvedVid`. `doc` may be any serializable document, including a `TrustTask<serde_json::Value>` or an `ErrorResponse`. |
| `pack_trust_task_nested(…)`, `pack_trust_task_routed(…)` | The same inner Direct message, wrapped for a single intermediary (**Nested**, metadata privacy) or relayed through hops (**Routed**). Relaying is the mediator's job; the consumer always opens the innermost Direct message. |
| `unpack_trust_task::<P>(&wire, &recipient, &sender)` | Open and verify the seal, check the envelope, and return `(TrustTask<P>, TspHandler)`. It stops there: the SPEC §7.2 consumer checks still have to run. |
| `TspConsumer` | The guarded inbound path: unseal, then run `consume_inbound` with §7.2 item 11's duplicate-execution record and the consequential freshness bound **on by default**. `receive` for one payload type, `receive_dispatched` with an `AsyncDispatcher` for a consumer that routes on `type`; `consume` / `consume_dispatched` when another TSP stack already opened the message. |
| `TspHandler` | The `TransportHandler` carrying the authenticated local and peer VIDs into the pipeline. |
| `TspError` | Envelope-layer failures (seal, signature, carriage, envelope type). These are errors rather than framework error responses: there is no authenticated party to answer. |

Use `TspConsumer` unless you have a reason not to. Binding §7 records that TSP
data messages do not inherently prevent replay, and under routed or nested
carriage an intermediary may re-send the sealed inner message — so an ordinary
re-forward executes a consequential task twice unless the consumer keeps the
record. The record is keyed on the document `id`, never on the envelope, which
is resealed with fresh material on every send. `TspConsumer::with_replay_guard`
takes a shared store for replicated consumers;
`TspConsumer::without_replay_record` exists for tasks where repeated execution
is safe and intended.

## Example

```sh
cargo run -p trust-tasks-tsp --example local_roundtrip
```

[`examples/local_roundtrip.rs`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/trust-tasks-tsp/examples/local_roundtrip.rs)
seals a document from one VID to another in-process and runs it through the
receiving pipeline.

## Versioning

This crate exposes `trust-tasks-rs` types in its public API, so it is released
in one version group with `trust-tasks-rs` and the other crates that do — they
always share a version. Pin them as a set, or depend on the `trust-tasks`
facade.

## License

Apache-2.0 — see
[`LICENSE`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/trust-tasks-tsp/LICENSE),
included in the published crate.
