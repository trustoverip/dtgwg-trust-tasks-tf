---
slug: rooms/keys/seal
version: "0.1"
title: "Rooms Keys — Seal"
summary: "An agent asks the party holding its principal's room keys to seal one record; the ciphertext comes back and the key never leaves."
status: draft
targetFrameworkVersion: "0.5.0"
category: ai-agents
# keywords and authors are OPTIONAL, and omitted here on purpose: the build derives
# keywords from the slug segments + category, and authors from CODEOWNERS (falling
# back to this folder's git history). Declare them only where the derivation would
# be wrong — a term a searcher would use that appears nowhere in the slug, or an
# editor who is not this slug's CODEOWNER.
#   keywords: [rooms, keys, seal, a-term-a-searcher-would-use]
#   authors:
#     - Your Name (https://github.com/your-handle)
parties:
  - role: Agent
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Oracle
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "The oracle seals with its principal's key material. A request whose integrity depended on the transport would let a compromised channel choose what gets written into a room under its principal's name."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A replayed request re-seals the same bytes and is harmless, but an undated one cannot be placed in the window its principal authorized the agent for."
sideEffects:
  level: none
  rationale: >-
    Returns ciphertext for one record. Stores nothing, and does not write to the room — the caller takes the result to a host with `rooms/records/put`.
exposure:
  discloses: metadata
  ingests: secret
  actsAsSubject: false
  rationale: >-
    The request carries the record's PLAINTEXT into the oracle, which is why `ingests` is `secret`; what comes back is ciphertext, so the response discloses only that a record of roughly that size was sealed. The oracle is the principal's own infrastructure, which is why it may see the plaintext at all.
  # ingests: metadata     # framework 0.5.0, OPTIONAL: what the REQUEST carries INTO the recipient
                          # (none | metadata | personal | secret). Note the enum differs from
                          # discloses — `personal` exists here because personal-but-not-secret
                          # data is exactly what changes a recipient's minimisation obligations.
                          # `personal` or `secret` makes exposure.rationale REQUIRED.
retention:
  class: transient
  rationale: >-
    The oracle seals and returns. It retains neither the plaintext nor the ciphertext — the record belongs at the host, and a copy here would be a second place the room's material lives.
errorCodes:
  - code: rooms/keys/seal:notAMember
    meaning: "The recipient holds no group state for this room, so there is no key to seal with."
    retryable: false
related:
  - rooms/keys/open
  - rooms/records/put
  - rooms/keys/chain
---

## Abstract

An agent hands its principal's **key holder** a record body and gets back the sealed form, ready to store with [`rooms/records/put`](../../../records/put/0.1/spec.md). It is the exact mirror of [`rooms/keys/open`](../../open/0.1/spec.md): plaintext in, ciphertext out, and the key never crosses in either direction.

Without it a client can **read** a sealed room and cannot **write** to one. That asymmetry is not a design choice; it is a gap. Sealing needs the epoch's storage key, the key lives in the key holder and by design never leaves it, and until now nothing asked the key holder to do the sealing on the caller's behalf. The practical effect was that every writing surface for an `attributed` or `private` room had to be the key holder itself.

The version and the record key are bound into the ciphertext, so this task cannot be used to produce a record that will open somewhere else — see the binding below, which is the whole reason a caller must decide where a record is going *before* asking for it to be sealed.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Definitions

**`roomId`** — the room to seal for. The recipient **MUST** hold group state for it; there is no key otherwise.

**`key`** — the record key this will be stored under. On a sealed tier it **MUST** be opaque: a descriptive key is readable by the host and defeats the encryption sitting beside it, so structured naming belongs *inside* the sealed body.

**`version`** — the version the writer intends the record to take.

**`plaintext`** — the record body, base64url. Sealed whole rather than field by field: splitting it would let a host learn the shape of the material from ciphertext lengths, for no benefit, since a reader decrypts the whole record either way.

**`sealed`** (response) — a [`SealedRecord`](../../../_shared/0.1/room.schema.json): ciphertext, nonce, and the epoch it was sealed under.

### The binding is why `version` is an input

A record's associated data commits to `roomId`, `key`, `version` and `epoch`. A host that relocates a sealed record — to another key, another version, another epoch, or another room — produces an authentication failure rather than a readable record. It holds every byte and still cannot move one, which is the property that makes an untrusted host tolerable.

The cost lands here: the host assigns the version, so a writer does not know it at sealing time. This task therefore takes the version the writer **intends**, and a caller that lets the host assign a different one will find the record does not open. That is the correct failure — silently accepting whatever came back would mean the binding commits to nothing.

The shapes that work are a create-only write (`expectedVersion: 0`, which makes the host assign exactly the version the caller predicted) or a read of the current version before a rewrite. Both are what [`rooms/records/put`](../../../records/put/0.1/spec.md) already supports.

### What this deliberately does not do

It does not write. The caller takes the result to a host, presenting its own authority there. Sealing and being allowed to store are different questions asked of different parties — the key holder knows the key and nothing about the room's ACL; the host knows the credentials and cannot read a byte.

## Request

An **Agent** sends this to its principal's **Oracle**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A create-only write, which is the shape that works

`version: 1` because the caller will store this with `expectedVersion: 0` — the host then assigns exactly the version predicted here, and the binding holds.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/keys/seal/0.1#request",
  "issuer": "did:example:agent",
  "recipient": "did:example:oracle",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "key": "giXFLTGBdnnQJRoIsktuIg",
    "version": 1,
    "plaintext": "IyBOb3J0aHdpbmQKCkRvIG5vdCByZXByaWNlIGJlZm9yZSByZW5ld2FsLg"
  }
}
```

## Response

The **Oracle** responds with the sealed record, using the sub-schema reachable via `$anchor: "response"`. `epoch` is the one it sealed under — the caller does not choose it, and a host will reject a record whose epoch is not the room's current one.

Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §6.4](/SPEC.md#64-error-documents).

### Ready to hand to a host

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/keys/seal/0.1#response",
  "issuer": "did:example:oracle",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "sealed": {
      "ciphertext": "3QhV1sVvR0m5xAqZ7Aw2mQnR4vBk8QLp3wXc9TgYw",
      "nonce": "b0Zt8Qm2Yq1sVvR0",
      "epoch": 4
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries the record's **plaintext**. This is the one direction in the room family where cleartext room material travels to the oracle rather than from it, and it is why `ingests` is `secret`.

That is not a weakening: the oracle is the principal's own key holder, and it is already the party that decrypts every record the principal reads. A surface that would not send it plaintext should not be asking it to seal.

What comes back is ciphertext, so the response discloses only that a record of roughly that size was sealed. A producer **MUST NOT** put record content in `ext`; the body has a member of its own, and material placed beside it is material that does not get sealed.

### Correlation

The oracle learns what its principal writes, which it would learn anyway from `open` when the principal reads it back. It also learns the record **key** and the intended **version**, which the host learns too.

To an observer, the informative signal is timing: a seal is followed by a `rooms/records/put` to the room's host, so the pair links a member to a write even where the host cannot read it. On a `private` room that pairing is worth avoiding — a producer **SHOULD** not make the two requests distinguishable as a pair by timing alone, for the same reason the tier exists.

Ciphertext length tracks plaintext length. A room whose records are short and formulaic discloses more by size than one whose records are prose, and no part of this task can fix that.

### Retention

**The oracle retains neither the plaintext nor the ciphertext.** It seals and returns. The record belongs at the host, and a copy here would be a second place the room's material lives — one the room's own lifecycle, curation and deletion verbs know nothing about.

### Consent/purpose

The plaintext is sent so that it can be written to the room the principal is a member of. It confers nothing the principal did not already hold: the authority to store the result is presented separately, to the host, and a sealed record the host refuses is a record that goes nowhere.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
