# Changelog

All notable changes to `trust-tasks-go/didcomm` are documented here. Entries are
generated from conventional commits by the release automation (see RELEASING.md).

## 0.2.0 — 2026-09-16


### Added

- **go-didcomm**: Make DIDComm authcrypt interoperate with the Rust/Dart bindings (#504)

The Go DIDComm binding rolled its own ECDH-1PU authcrypt but did not match the
  DIDComm ecosystem's key derivation, so a JWE it produced could not be opened by
  affinidi-messaging-didcomm (what the Rust and Dart bindings use), and vice
  versa. Two mismatches, both fixed:

  - ECDH-1PU is tag-in-KDF (draft-madden §2.3): encrypt the content first, then
    derive the key-wrapping KEK with the content-encryption tag as a
    length-prefixed SuppPrivInfo. The binding derived the KEK without the tag.
  - apv is base64url(SHA-256(sorted recipient kids joined by ".")), not the raw
    recipient kid. (apu = base64url(skid) already matched.)

  aries-askar, didcomm-python, didcomm-rust and affinidi all do the tag-in-KDF
  derivation; it is the de-facto DIDComm v2 definition. The reworked crypto now
  interoperates both directions, asserted against affinidi:

  - affinidi → Go: interop_test.go unpacks a JWE packed by affinidi
    (testdata/authcrypt-interop.fixture.json).
  - Go → affinidi: a companion Rust test in trust-tasks-didcomm unpacks a Go JWE.

  This changes the wire format, so it is a breaking change: v0.1.0 → v0.2.0. The
  existing round-trip and negative tests still pass; the RFC 3394 key-wrap vector
  is unchanged. Documented in CLAUDE.md and the module README/testdata.

