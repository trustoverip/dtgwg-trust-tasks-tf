# trust-tasks-go/tsp

The [Trust Tasks](https://trusttasks.org) ToIP Trust Spanning Protocol (TSP)
transport binding ([`bindings/tsp/0.1`](https://trusttasks.org/binding/tsp/0.1))
for Go, built on
[`github.com/affinidi/affinidi-tsp-go`](https://github.com/affinidi/affinidi-tsp-go).
The Go counterpart of the Rust `trust-tasks-tsp` crate and the Dart
`trust_tasks_tsp` package: all three seal the same envelope object, so a document
sealed by one opens in the others.

```console
go get github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/tsp
```

## A separate module

This is its **own Go module**, not part of `trust-tasks-go`. The core module has
no dependencies — its selling point — and this one pulls in `affinidi-tsp-go` and
its cryptography. Keeping them apart means a consumer who only wants the types
and the §7.2 pipeline still gets a dependency-free core.

`affinidi-tsp-go` is not tagged yet, so `go.mod` requires it at a pseudo-version
from its public repository; the Go proxy resolves it. Bump to a real tag when one
is published.

## Sending

```go
wire, err := tsp.PackTrustTask(doc, alice, bob.Public())
```

`doc` is anything that marshals to a Trust Task document (a
`*trusttasks.Document[P]`, or `json.RawMessage`). The message is a **Direct** TSP
message, HPKE-sealed and signed; its payload is the envelope object
`{"type": "https://trusttasks.org/binding/tsp/0.1/envelope", "document": …}`. A
TSP VID **is** a framework VID, so nothing is transformed.

## Receiving

```go
// One per process: its replay guard is the duplicate-execution record.
consumer := tsp.NewConsumer(bob)

sender := lookUp(tsp.AdvertisedSender(wire)) // the Go TSP library resolves no VIDs

received, err := tsp.Receive(ctx, consumer, wire, sender, acl_grant.Spec,
	decodeGrant, handleGrant)
if err != nil {
	var envErr *tsp.EnvelopeError
	if errors.As(err, &envErr) { /* never reached the pipeline (§4) */ }
}
if received.Reply != nil {
	send(tsp.PackReply(bob, sender, received.Reply)) // sealed back to the sender
}
```

`Receive` opens the sealed message under the sender identity you supply (the Go
TSP library does no DID resolution — read `AdvertisedSender` and look the sender
up in your own store), then runs the SPEC §7.2 pipeline with the authenticated
sender as the transport identity (§3). `received.Outcome` is the core library's
`ConsumeOutcome`; `received.Reply` is the document to seal back, or nil when §8.1
or §7.1 says to stay silent.

### What is refused

Before the pipeline, with no reply possible — TSP has no anonymous sender, so
these leave nothing authenticated to answer (§4). They come back as an
`*EnvelopeError` whose `Failure` is one of `FailNotForThisReceiver` (bad framing,
wrong receiver, failed signature or HPKE authentication), `FailNotSealed` (a
signed-only message; §2 requires sealing), `FailNotFinalRecipient` (a
routed/nested layer this VID only relays, §5.2), `FailWrongEnvelopeType` or
`FailInvalidBody`.

Inside the pipeline, answered with a `trust-task-error` sealed back to the
authenticated sender: a body that does not deserialise is `malformedRequest`; an
in-band `issuer` that is not the authenticated sender is `identityMismatch`,
sealed to the sender that actually authenticated.

### Duplicates

**On by default.** §7 records that TSP data messages "do not inherently prevent
replay", and under routed/nested carriage an intermediary may re-send the sealed
inner message. The record is keyed on the document `id`, never the TSP envelope,
which is resealed with fresh material on every send: the same document again is a
duplicate replied to with the first response; a different document under the same
`id` is `idConflict`; a store that cannot be consulted fails closed with
`unavailable`. The default record is in-process — replicas must share a `Guard`.

### Proofs

TSP authenticates the sender end-to-end in Direct and nested modes, so a document
may omit `proof` over this binding (§5.3) — unless its specification requires one.
Set `Consumer.ProofVerifier` to accept proofs; without one, a proof-bearing
document is refused rather than having its proof ignored. Proofs are verified over
the document exactly as it arrived.

## Scope

This release carries **Direct** messages, the strongest case (§5.1). Routed and
nested relaying is the mediator's job on the wire; a layer this VID only relays
surfaces as `FailNotFinalRecipient`.

## Interoperability

The sealed envelope object is the same `{type, document}` shape the Rust
`trust-tasks-tsp` crate and the Dart `trust_tasks_tsp` package seal, pinned by a
test. Cross-implementation TSP wire interop rests on the shared TSP Rev 3
conformance vectors that `affinidi-tsp-go`, `affinidi_tsp` (Dart) and the
`affinidi-tsp` Rust crate each test against.

## License

Apache-2.0.
