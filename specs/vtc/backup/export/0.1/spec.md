---
slug: vtc/backup/export
version: "0.1"
title: VTC Backup — Export
summary: Export a Verifiable Trust Community's full state as a single password-encrypted envelope (Argon2id + AES-256-GCM), optionally including the audit log.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - backup
  - export
  - disaster-recovery
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: super administrator
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Exports every backed-up keyspace and the community signing key bundle. The caller must be attributable.
sideEffects:
  level: none
  rationale: "Reads state and returns an envelope; the community is not mutated. The export is audited."
exposure:
  discloses: secret
  actsAsSubject: false
  rationale: The envelope carries the community signing key bundle alongside every backed-up keyspace. Its only protection is the caller-supplied passphrase.
errorCodes:
  - code: vtc/backup/export:passwordTooShort
    meaning: The supplied password is under the 12-character minimum.
    retryable: false
  - code: vtc/backup/export:permissionDenied
    meaning: The consumer lacks the community super-admin capability.
    retryable: false
---

## Abstract

The **VTC Backup — Export** Trust Task returns the community's full state as one encrypted [`BackupEnvelope`](#response). The envelope is self-describing: it carries its KDF and cipher parameters in the clear so a future reader can decrypt it without out-of-band knowledge, and everything of substance sits in `ciphertext`.

`includeAudit` is opt-in and defaults to `false`. When it is set, the audit log travels **with its signed checkpoints** — the two are meaningless apart, because a log restored without the checkpoints that attest to it reads as mass truncation.

It is the counterpart of [`vtc/backup/import`](../../import/0.1/).

## Conformance

Producer: supply `password` (≥ 12 characters). Set `includeAudit` only when the audit log is wanted — it is large and carries plaintext DIDs.

Consumer: verify the super-admin capability. Return the envelope under a named `envelope` member rather than as the bare response body — the registry response convention requires `additionalProperties: false` and an `ext` extension point, and neither can be attached to a bare `$ref`. Reject a password under 12 characters with `passwordTooShort` **before** doing any work. Derive the key with Argon2id, encrypt the payload with AES-256-GCM, and return the envelope with `kdf` and `encryption` populated so it is decryptable without reference to this specification's defaults. Record an audit event naming the keyspace count and the community DID.

## Security & Privacy

**The envelope contains the community's signing key bundle**, so `exposure.discloses` is `secret` — this is the one VTC read task that hands over key material. Its protection is entirely the password: an envelope that leaks is only as strong as the passphrase chosen for it, which is why the 12-character floor is a hard consumer-side check rather than producer advice.

`includeAudit` is off by default because the audit log carries plaintext actor and target DIDs. An operator taking a routine disaster-recovery snapshot should not silently widen the blast radius of that file to include the community's membership activity.
