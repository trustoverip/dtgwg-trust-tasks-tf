# @openvtc/trust-tasks-tsp

The [Trust Tasks](https://trusttasks.org) ToIP Trust Spanning Protocol (TSP)
transport binding ([`bindings/tsp/0.1`](https://trusttasks.org/binding/tsp/0.1))
for TypeScript, on
[`@openvtc/vti-tsp-js`](https://www.npmjs.com/package/@openvtc/vti-tsp-js) — the
pure-TypeScript TSP the VTA browser plugin uses (browser, Node, React Native, no
WASM). The counterpart of the Rust `trust-tasks-tsp` crate, the Dart
`trust_tasks_tsp` package and the Go `trust-tasks-go/tsp` module: all seal the
same envelope object.

```console
npm install @openvtc/trust-tasks @openvtc/trust-tasks-tsp
```

## Sending

```ts
const wire = await packTrustTask(document, {
  senderVid, receiverVid,
  keys: { senderSigningKey, senderEncryptionKey, receiverEncryptionKey },
});
```

The message is a **Direct** TSP message, HPKE-Auth sealed and signed. Its payload
is the envelope object `{"type": "https://trusttasks.org/binding/tsp/0.1/envelope",
"document": …}`. A TSP VID **is** a framework VID, so nothing is transformed.

`@openvtc/vti-tsp-js` is key-based and does no DID resolution: you pass the raw
Ed25519 signing key and X25519 encryption keys for each party.

## Receiving

```ts
// One per process: its replay guard is the duplicate-execution record.
const consumer = new TspConsumer({
  recipientVid,
  decryptionKey,   // the recipient's X25519 private key
  signingKey,      // the recipient's Ed25519 private key, for sealed replies
  resolveSender: (vid) => lookUp(vid),   // -> { encryptionKey, signingKey } (public)
});

const received = await consumer.receive(wire, spec, async (doc, parties) =>
  respondWith(doc, newUrnUuid(), await grant(doc, parties)),
);

if (received.reply !== undefined) {
  await send(await consumer.packReply(received)); // sealed back to the sender
}
```

`receive` reads the message's named sender (`advertisedSender`), checks the
allowlist, resolves the sender's public keys, opens the sealed message, then runs
the SPEC §7.2 pipeline with the authenticated sender as the transport identity
(§3). `received.outcome` is the core `ConsumeOutcome`; `received.reply` is the
document to seal back, or `undefined` when §8.1 or §7.1 says to stay silent.

A handler **refuses** by returning an `ErrorResponse` (build it with the core's
`rejectWith`), not by throwing.

### What is refused

Before the pipeline, with no reply possible — TSP has no anonymous sender (§4).
These throw `TspEnvelopeError` whose `failure` is `notForThisReceiver` (bad
framing, wrong receiver, failed signature or HPKE authentication, an unresolvable
or disallowed sender), `notFinalRecipient` (a routed/nested layer this VID only
relays), `wrongEnvelopeType` or `invalidBody`.

Inside the pipeline, answered with a `trust-task-error` sealed back to the
authenticated sender: an in-band `issuer` that is not the authenticated sender is
`identityMismatch`, sealed to the sender that actually authenticated.

### Duplicates

**On by default.** The record is keyed on the document `id`, never the TSP
envelope (resealed with fresh material on every send). A re-forward of the same
document is a duplicate replied to with the first response; a different document
under the same `id` is `idConflict`; a store that cannot be consulted fails closed
with `unavailable`. Supply a shared `guard` behind a load balancer.

### Proofs

TSP authenticates the sender end-to-end (§5.3), so a document may omit `proof`
unless its specification requires one. Set `proofVerifier` (for example the
`DataIntegrityProofVerifier` from `@openvtc/trust-tasks-proof`) to accept proofs;
without one, a proof-bearing document is refused rather than ignored.

## Scope

This release carries **Direct** messages, the strongest case (§5.1). A routed or
nested layer this VID only relays surfaces as `notFinalRecipient`.

## License

Apache-2.0.
