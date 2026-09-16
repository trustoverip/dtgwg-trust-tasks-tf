# trust_tasks_didcomm

The [Trust Tasks](https://trusttasks.org) DIDComm v2.1 transport binding
([`bindings/didcomm/0.2`](https://trusttasks.org/binding/didcomm/0.2)) for Dart,
built on Affinidi's [`package:didcomm`](https://pub.dev/packages/didcomm). The
Dart counterpart of the Rust
[`trust-tasks-didcomm`](https://crates.io/crates/trust-tasks-didcomm) crate,
and interoperable with it in both directions.

```console
dart pub add trust_tasks trust_tasks_didcomm
```

Requires Dart 3.6 or later.

## Sending

```dart
final envelope = await packTrustTask(
  document.toJson((p) => p.toJson()),
  sender: aliceDidManager,          // an ssi DidManager
  recipient: bobDidDocument,        // resolved however you resolve DIDs
);
```

The envelope is always **authcrypt** (ECDH-1PU): the binding requires it, and it
is what lets the recipient know who sent the document without trusting anything
in it. `thid` and `pthid` are set from the document's `threadId` and
`parentThreadId`, falling back to its `id` (§3.1).

## Receiving

```dart
// One per process: its replay guard is the duplicate-execution record.
final consumer = DidcommConsumer(
  recipient: bobDidManager,
  allowedSenders: {aliceDid},        // optional; checked before decryption
);

final received = await consumer.receive<acl_grant.Payload, acl_grant.Response>(
  envelope,
  spec: acl_grant.spec,
  decode: acl_grant.Payload.fromJson,
  encode: (p) => p.toJson(),
  encodeResponse: (r) => r.toJson(),
  handler: (doc, parties) async => respondWith(doc, newUrnUuid(), await grant(doc)),
);

if (received.reply != null) {
  await send(await consumer.packReply(received)); // authcrypt, back to the sender
}
```

`receive` opens the envelope, then runs the SPEC §7.2 pipeline with the
authenticated sender as the transport identity (§3). `received.outcome` is the
core library's sealed `ConsumeOutcome`; `received.reply` is the document to send
back, or null when §8.1 or §6.1 says to stay silent.

### What is refused, and how

Before the pipeline, with **no** reply possible — there is no authenticated
sender to address one to (§4). These throw `EnvelopeException`:

| `EnvelopeFailure` | Cause |
|---|---|
| `unauthenticatedSender` | anoncrypt, signed-only or plaintext |
| `senderNotAllowed` | not in `allowedSenders` — refused **before** decryption, so an unlisted `did:web` is never fetched |
| `undecryptable` | not for this recipient, tampered, or inconsistent headers |
| `unqualifiedSenderKid` | the `skid` names no key |
| `wrongEnvelopeType` | a DIDComm message that is not a Trust Task envelope |
| `invalidBody` | no Trust Task `id` and `type` in the body |

Inside the pipeline, answered with a `trust-task-error` to the authenticated
sender: a body that does not deserialise and a `thid`/`threadId` disagreement
are `malformedRequest` (§3.1, §4); an in-band `issuer` that is not the
authenticated sender is `identityMismatch`, addressed to the sender that
actually authenticated.

### Duplicates

**On by default, and it matters most on this transport.** DIDComm guarantees no
freshness and a mediator may redeliver (§6), so an ordinary queue retry would
execute a consequential task twice. The record is keyed on the *document* `id`,
never the DIDComm message `id` or `thid`, because a redelivery is a fresh DIDComm
message around the same document:

- the same document again → `DuplicateOutcome`, replied to with the first
  response, handler not called;
- a different document under the same `id` → `idConflict`;
- a record that cannot be consulted → `unavailable`, handler not called.

The default record is in-process. **Replicas must share a `replayGuard`.**

### Proofs

Authcrypt authenticates the sender, so a document may omit `proof` over this
binding (§5) — unless its specification requires one. Pass a `proofVerifier`
(for example `DataIntegrityProofVerifier` from `trust_tasks_proof`) to accept
proofs; without one, a proof-bearing document is refused rather than having its
proof silently ignored. Proofs are verified over the document exactly as it
arrived.

## Interoperability

Checked against the Rust `trust-tasks-didcomm` crate, with fixed did:key
identities on each side:

- a Dart-packed request is unpacked by Rust with the Dart sender authenticated,
  and Rust's reply opens in Dart with `thid` and `threadId` intact;
- a Rust-packed request goes through `DidcommConsumer` in Dart, a redelivery is
  absorbed as a duplicate, and the Dart reply opens in Rust.

## License

Apache-2.0.
