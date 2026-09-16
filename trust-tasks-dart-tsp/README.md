# trust_tasks_tsp

The [Trust Tasks](https://trusttasks.org) ToIP Trust Spanning Protocol (TSP)
transport binding ([`bindings/tsp/0.1`](https://trusttasks.org/binding/tsp/0.1))
for Dart, built on
[`affinidi_tsp`](https://github.com/affinidi/affinidi-tsp-dart). The Dart
counterpart of the Rust
[`trust-tasks-tsp`](https://crates.io/crates/trust-tasks-tsp) crate: it seals
the same envelope object, so a document sealed by either opens in the other.

> **Not yet on pub.dev.** This package depends on `affinidi_tsp`, which has not
> been published to pub.dev yet. Until it is, depend on this via a git
> reference, and note that this package cannot itself be published (pub.dev
> requires every dependency in the published closure to be on pub.dev).

Requires Dart 3.8 or later.

## Sending

```dart
final wire = await packTrustTask(
  document.toJson((p) => p.toJson()),
  sender: alicePrivateVid,   // a TSP PrivateVid (didManager.toTspPrivateVid())
  receiver: bobPublicVid,    // resolved with SsiVidResolver().resolve(bob.id)
);
```

The message is a **Direct** TSP message, HPKE-sealed and signed. Its payload is
the envelope object with `type` `https://trusttasks.org/binding/tsp/0.1/envelope`
and `document` set to the Trust Task document. A TSP VID **is** a framework VID,
so nothing is transformed on the way in or out.

TSP signing is Ed25519; a VID's HPKE key agreement is derived from it, so build
identities from Ed25519 keys.

## Receiving

```dart
// One per process: its replay guard is the duplicate-execution record.
final consumer = TspConsumer(
  recipient: bobPrivateVid,
  allowedSenders: {aliceVid},   // optional; checked before the sender is resolved
);

final received = await consumer.receive<acl_grant.Payload, acl_grant.Response>(
  wire,
  spec: acl_grant.spec,
  decode: acl_grant.Payload.fromJson,
  encode: (p) => p.toJson(),
  encodeResponse: (r) => r.toJson(),
  handler: (doc, parties) async => respondWith(doc, newUrnUuid(), await grant(doc)),
);

if (received.reply != null) {
  await send(await consumer.packReply(received)); // sealed back to the sender
}
```

`receive` reads the message's named sender, resolves it (an allowlist is checked
first), opens the sealed message, then runs the SPEC §7.2 pipeline with the
authenticated sender as the transport identity (§3). `received.outcome` is the
core library's sealed `ConsumeOutcome`; `received.reply` is the document to seal
back, or null when §8.1 or §7.1 says to stay silent.

### What is refused

Before the pipeline, with no reply possible — TSP has no anonymous sender, so
these leave nothing authenticated to answer (§4). They throw
`TspEnvelopeException`:

| `TspFailure` | Cause |
|---|---|
| `notForThisReceiver` | not a TSP message this VID can open — bad framing, wrong receiver, failed signature or HPKE authentication, or an unresolvable sender |
| `senderNotAllowed` | not in `allowedSenders` — refused **before** the sender VID is resolved |
| `notSealed` | a signed-only message; §2 requires the payload sealed as well as signed |
| `notFinalRecipient` | a routed/nested message this VID only relays (§5.2) |
| `wrongEnvelopeType`, `invalidBody` | not a Trust Task envelope object |

Inside the pipeline, answered with a `trust-task-error` sealed back to the
authenticated sender: a body that does not deserialise is `malformedRequest`
(§4); an in-band `issuer` that is not the authenticated sender is
`identityMismatch`, sealed to the sender that actually authenticated.

### Duplicates

**On by default.** §7 records that TSP data messages "do not inherently prevent
replay", and under routed/nested carriage an intermediary may re-send the sealed
inner message. The record is keyed on the document `id`, never the TSP envelope,
which is resealed with fresh material on every send:

- the same document again → `DuplicateOutcome`, replied to with the first
  response, handler not called;
- a different document under the same `id` → `idConflict`;
- a record that cannot be consulted → `unavailable`, handler not called.

The default record is in-process. **Replicas must share a `replayGuard`.**

### Proofs

TSP authenticates the sender end-to-end in Direct and nested modes, so a
document may omit `proof` over this binding (§5.3) — unless its specification
requires one. Pass a `proofVerifier` (for example `DataIntegrityProofVerifier`
from `trust_tasks_proof`) to accept proofs; without one, a proof-bearing
document is refused rather than having its proof ignored. Proofs are verified
over the document exactly as it arrived.

## Scope

This release carries **Direct** messages, the strongest and simplest case (§5.1)
and the one a `TspConsumer` is built to run end to end. TSP's routed and nested
carriage (§5.2) is the mediator's job on the wire; a nested inner message this
VID is the final recipient of opens through the same path, while a layer this VID
only relays is surfaced as `notFinalRecipient` rather than executed.

## Interoperability

The sealed envelope object is byte-for-byte the shape the Rust `trust-tasks-tsp`
crate seals (`{"type": …, "document": …}`, and nothing else), pinned by a test.
Cross-implementation TSP wire interop rests on the shared TSP Rev 3 conformance
vectors that `affinidi_tsp`, `affinidi-tsp-go` and the `affinidi-tsp` Rust crate
each test against.

## License

Apache-2.0.
