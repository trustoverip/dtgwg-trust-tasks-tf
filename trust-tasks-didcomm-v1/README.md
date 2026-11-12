# trust-tasks-didcomm-v1

The **DIDComm v1** transport binding for the [Trust Tasks](https://trusttasks.org/)
framework, built on
[`affinidi-messaging-didcomm-v1`](https://crates.io/crates/affinidi-messaging-didcomm-v1).
It implements [`bindings/didcomm-v1/0.2`](https://trusttasks.org/bindings/didcomm-v1/0.2).

Credo — and so essentially every Aries-lineage wallet — speaks DIDComm v1 only.
This crate is how those agents carry a Trust Task. It is deliberately thin and
the same shape as `trust-tasks-didcomm` (DIDComm v2.1), so one implementation can
drive both.

```sh
cargo add trust-tasks-didcomm-v1
```

or, through the facade, `cargo add trust-tasks --features didcomm-v1`
and use `trust_tasks::didcomm_v1`.

## What differs from DIDComm v2.1

- **Attribution takes a step more.** A v1 envelope authenticates a bare base58
  Ed25519 verkey, not a DID; the verkey-to-DID binding is connection state. An
  envelope that is cryptographically sound but attributable to nobody this agent
  knows is `DidcommV1Error::UnknownSenderBinding`.
- **The document travels in an `~attach` decorator**, because v1 has no `body`.
- **Threads map explicitly**: `threadId` ↔ `~thread.thid`, `parentThreadId` ↔
  `~thread.pthid` (`THREAD_MAPPING`). Producers fill the decorator from the
  document, and omit — never rewrite — a correlator RFC 0008 cannot represent (a
  `urn:uuid:` id, most often). Consumers compare only where both are present; a
  disagreement is `malformedRequest`.

## API

| Item | What it does |
|---|---|
| `build_message(&doc)` | Place a document in a v1 message of the binding's dedicated type (`ENVELOPE_TYPE`), with the `~thread` decorator filled from the document. Returns the unpacked `MessageV1`; authcrypt-packing it is your agent's job, because it needs keys from connection state this crate does not model. |
| `unpack_trust_task::<P>(unpacked)` | From an unpacked v1 message: the authenticated-sender gate, the message-type check, the attachment lookup and the `~thread` cross-check. Returns `(TrustTask<P>, DidcommV1Handler)`. It stops there: the SPEC §7.2 consumer checks still have to run. |
| `DidcommV1Consumer` | The guarded inbound path: unpack, then run `consume_inbound` with §7.2 item 11's duplicate-execution record and the consequential freshness bound **on by default**. `receive` for one payload type, `receive_dispatched` with an `AsyncDispatcher` for a consumer that routes on `type`. An attachment that does not deserialise, or a `~thread` disagreement, is answered `malformedRequest` to the authenticated sender (binding §4) as `ConsumeOutcome::Rejected`. Dereferences to `trust_tasks_rs::GuardedConsumer`, the core it shares with the other message bindings, whose `consume` / `consume_dispatched` take a document already extracted. |
| `DidcommV1Handler` | The `TransportHandler` carrying the connection's `theirDid` into §4.8.1 precedence, and the `Carriage` the message arrived on. |

Use `DidcommV1Consumer` unless you have a reason not to. Binding §6 records v1's
freshness guarantee as **none**, so an ordinary store-and-forward redelivery
executes a consequential task twice unless the consumer keeps the record. The
record is keyed on the document `id`, never on the v1 `@id` or `~thread`, which a
redelivery may legitimately change.

## The `legacy-basic-message` feature

Binding §2.3 requires a `0.2` consumer to also accept the superseded `0.1`
carriage, an attachment on a `basic-message`. That is the Aries **chat** type, so
while the gate is open any chat message from any established connection carrying
a `trust-task` attachment is a framework input. The feature is **on by default**
because §2.3 makes accepting it a MUST; each message on that carriage is logged
at `warn` and reported as `Carriage::LegacyBasicMessage`, so an operator can see
which peers have not moved. A deployment whose peers have all migrated can close
it, as a deliberate departure from that MUST:

```sh
cargo add trust-tasks-didcomm-v1 --no-default-features
```

## Versioning

This crate exposes `trust-tasks-rs` types in its public API, so it is released
in one version group with `trust-tasks-rs` and the other crates that do — they
always share a version. Pin them as a set, or depend on the `trust-tasks`
facade.

## License

Apache-2.0 — see
[`LICENSE`](https://github.com/trustoverip/dtgwg-trust-tasks-tf/blob/main/trust-tasks-didcomm-v1/LICENSE),
included in the published crate.
