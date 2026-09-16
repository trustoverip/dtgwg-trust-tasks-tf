# DIDComm authcrypt interop fixtures

DIDComm v2 authcrypt is not byte-deterministic (random ephemeral key, IV and
CEK per message), so cross-library interop is asserted by **one library packing
a JWE and another unpacking it**, not by reproducing bytes. The Go binding rolls
its own ECDH-1PU authcrypt on the standard library; these fixtures pin it against
`affinidi-messaging-didcomm` — the library the Rust and Dart Trust Tasks DIDComm
bindings are built on.

- **`authcrypt-interop.fixture.json`** — a JWE packed by **affinidi**, with fixed
  X25519 keys. `interop_test.go` unpacks it (affinidi → Go).
- The Go → affinidi direction is asserted on the Rust side:
  `trust-tasks-didcomm/tests/interop.rs` unpacks a Go-packed JWE
  (`trust-tasks-didcomm/tests/fixtures/go-authcrypt-interop.json`) with affinidi.

Both fixtures use the same conventions the DIDComm ecosystem uses: `apu =
base64url(skid)`, `apv = base64url(SHA-256(sorted recipient kids joined by "."))`,
and the ECDH-1PU **tag-in-KDF** key derivation (the content-encryption tag as a
length-prefixed `SuppPrivInfo`).

## Regenerating the affinidi-packed fixture

A throwaway Rust binary drives affinidi's public `jwe::encrypt::authcrypt` with
fixed keys:

```toml
# Cargo.toml
[dependencies]
affinidi-messaging-didcomm = "=0.15.8"
affinidi-crypto = "=0.2.6"
serde_json = "1"
```

```rust
use affinidi_crypto::jose::key_agreement::{Curve, PrivateKeyAgreement};
use affinidi_messaging_didcomm::jwe::encrypt::authcrypt;

fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{:02x}", x)).collect() }

fn main() {
    let sp = PrivateKeyAgreement::from_raw_bytes(Curve::X25519, &[7u8; 32]).unwrap();
    let rp = PrivateKeyAgreement::from_raw_bytes(Curve::X25519, &[9u8; 32]).unwrap();
    let (sk, rk) = ("did:example:alice#key-x25519-1", "did:example:bob#key-x25519-1");
    let pt = serde_json::to_vec(&serde_json::json!({
        "id": "urn:uuid:2b1a7e0c-0000-4000-8000-00000000abcd",
        "type": "https://trusttasks.org/binding/didcomm/0.1/envelope",
        "from": "did:example:alice", "to": ["did:example:bob"],
        "thid": "urn:uuid:00000000-0000-4000-8000-000000000001",
        "body": {"id": "urn:uuid:00000000-0000-4000-8000-000000000001",
                 "type": "https://trusttasks.org/spec/example/echo/0.1",
                 "issuedAt": "2026-06-01T12:00:00Z",
                 "payload": {"text": "hello from affinidi"}}
    })).unwrap();
    let jwe = authcrypt(&pt, sk, &sp, &[(rk, &rp.public_key())]).unwrap();
    println!("{}", serde_json::json!({
        "senderKid": sk, "recipientKid": rk,
        "senderPubX25519": hex(&sp.public_key().to_public_bytes()),
        "recipientPrivX25519": hex(&[9u8; 32]),
        "plaintext": String::from_utf8(pt).unwrap(), "jwe": jwe,
    }));
}
```

## Regenerating the Go-packed fixture

A `go run` program using this module's public `PackTrustTask` with fixed keys
(sender scalar `0x11`, recipient scalar `0x13`), writing the JWE plus
`senderPubX25519` / `recipientPrivX25519` (hex), `senderKid`, `recipientKid` —
the fields `trust-tasks-didcomm/tests/interop.rs` reads.
