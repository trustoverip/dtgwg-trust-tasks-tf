# trust-tasks-go/didcomm

The [DIDComm v2.1](https://identity.foundation/didcomm-messaging/spec/) transport
binding for [Trust Tasks](../../README.md) (`bindings/didcomm/0.2`), on the Go
standard library.

- **DIDComm v2.1 authcrypt** — ECDH-1PU key agreement, `A256KW` key wrapping,
  `A256CBC-HS512` content encryption, over X25519 key-agreement keys. The
  verified `skid` becomes the framework's transport-authenticated sender
  (SPEC §4.8.1); anoncrypt and plaintext are rejected — there is no
  authenticated sender to route a reply to (binding §2, §4).
- **No third-party dependencies.** The authcrypt JWE is rolled on
  `crypto/ecdh`, `crypto/aes`, `crypto/hmac` and the SHA-2 family, with the
  RFC 3394 key-wrap and RFC 7518 content-encryption primitives written in the
  module (the key-wrap is pinned to the RFC's own test vector). Its only
  requirement is the core `trust-tasks-go` module. Same roll-our-own basis as
  `trust-tasks-go/proof`, and the place PQC lands later.
- **Key-based, like `trust-tasks-go/tsp`.** The binding does no DID resolution:
  the consumer holds the recipient's X25519 private key and takes a
  `ResolveSender(did) → *PeerKey` callback for the sender's public key, looked
  up in your own store.

This is a **separate module** from the core, so the core stays dependency-free.

## Install

```sh
go get github.com/trustoverip/dtgwg-trust-tasks-tf/trust-tasks-go/didcomm
```

## Pack / unpack

```go
alice, _ := didcomm.NewIdentity("did:example:alice", "did:example:alice#key-agreement-1", aliceX25519Priv)
bobKey, _ := didcomm.NewPeerKey("did:example:bob#key-agreement-1", bobX25519Pub)

wire, _ := didcomm.PackTrustTask(docJSON, alice, bobKey) // authcrypt to bob

unpacked, err := didcomm.UnpackTrustTask(wire, bob, func(senderDID string) (*didcomm.PeerKey, error) {
    return lookUp(senderDID) // your store
})
```

`PackTrustTask` sets the DIDComm `thid` from the document's `threadId` (or its
`id`) and `pthid` from `parentThreadId` (binding §3.1).

## Consume (§7.2 pipeline)

```go
c := didcomm.NewConsumer(bob) // duplicate-execution record + freshness on by default
r, err := didcomm.Receive(ctx, c, wire, resolveSender, spec, decodePayload, handler)
if r.Reply != nil {
    reply, _ := didcomm.PackReply(bob, senderKey, r.Reply) // sealed back to the sender
}
```

The consumer keeps the SPEC §7.2 item-11 duplicate-execution record on by
default: DIDComm is store-and-forward and provides no replay protection
(binding §6), so a mediator re-delivery would otherwise execute a consequential
task twice. Where both a DIDComm thread header and its document member are
present and disagree, it answers `malformedRequest`, routed to the authenticated
sender (binding §3.1).

## Interop

The authcrypt profile (ECDH-1PU + A256KW + A256CBC-HS512, X25519) matches the
DIDComm v2 ecosystem exactly: the **tag-in-KDF** ECDH-1PU key derivation (the
content-encryption tag as a length-prefixed `SuppPrivInfo`) and `apv =
base64url(SHA-256(sorted recipient kids))` that `aries-askar`, `didcomm-python`,
`didcomm-rust` and `affinidi` all use.

Interop is asserted **both directions** against `affinidi-messaging-didcomm` (the
library the Rust and Dart Trust Tasks DIDComm bindings are built on):

- **affinidi → Go**: `interop_test.go` unpacks a JWE packed by affinidi
  (`testdata/authcrypt-interop.fixture.json`).
- **Go → affinidi**: `trust-tasks-didcomm/tests/interop.rs` unpacks a JWE packed
  by this binding with affinidi.

See `testdata/README.md` for the fixtures and how to regenerate them.
