---
slug: rooms/keys/seal
version: "0.2"
title: "Rooms Keys — Seal"
summary: "A writer asks the party holding its principal's room keys to seal one record — and, when the writer is the principal's own client, to sign the record's body as the principal first; the ciphertext comes back and no key leaves."
status: draft
targetFrameworkVersion: "0.6.0"
category: ai-agents
keywords:
  - room
  - keys
  - seal
  - persona
  - attribution
parties:
  - role: Writer
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: KeyHolder
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The key holder seals with its principal's key material and, when asked, signs the record as its principal. A request whose integrity depended on the transport would let a compromised channel choose what gets written into a room under the principal's name.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A request that asks for a signature exercises the principal's own authority, so it is consequential and SPEC §7.3 item 17 sets REQUIRED as the floor. An undated request also cannot be placed in the window its principal authorized the writer for.
sideEffects:
  level: none
  rationale: >-
    Returns ciphertext for one record. Stores nothing, and does not write to the room — the caller takes the result to a host with `rooms/records/put`.
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: secret
  actsAsSubject: true
  rationale: >-
    The request carries the record's PLAINTEXT into the key holder, which is why `ingests` is `secret`; what comes back is ciphertext, so the response discloses only that a record of roughly that size was sealed. `actsAsSubject` because, under `signAsPrincipal`, the key holder signs the record's body as its principal — the signature every reader attributes the record by.
retention:
  class: transient
  rationale: >-
    The key holder seals and returns. It retains neither the plaintext nor the ciphertext — the record belongs at the host, and a copy here would be a second place the room's material lives.
errorCodes:
  - code: rooms/keys/seal:notAMember
    meaning: "The recipient holds no group state for this room, so there is no key to seal with."
    retryable: false
  - code: rooms/keys/seal:bodyInvalid
    meaning: "Under `signAsPrincipal`, `plaintext` did not decode to a `SealedRecordBody` without `proof`: it is not JSON, carries an unknown member, already carries a `proof`, or lacks a required member."
    retryable: false
  - code: rooms/keys/seal:authorNotPrincipal
    meaning: "Under `signAsPrincipal`, the body's `author` is not a member of this room the caller may act as. `details.author` repeats it."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [author]
      properties:
        author:
          type: string
          pattern: "^did:"
          maxLength: 2048
  - code: rooms/keys/seal:locationMismatch
    meaning: "Under `signAsPrincipal`, the body's `roomId`, `key` or `version` differs from the payload's. A body signed for one location and sealed for another opens nowhere and attributes nothing."
    retryable: false
related:
  - rooms/keys/open
  - rooms/keys/act
  - rooms/records/put
  - rooms/keys/chain
---

## Abstract

A writer hands its principal's **key holder** a record body and gets back the sealed form, ready to store with [`rooms/records/put`](../../../records/put/0.2/spec.md). It is the mirror of [`rooms/keys/open`](../../open/0.1/spec.md): plaintext in, ciphertext out, and the key never crosses in either direction.

The body is a [`SealedRecordBody`](../../../_shared/0.1/room.schema.json), and its signature is what attributes the record: every reader verifies it, and on an `attributed` room checks it against the author the host recorded. Who signs depends on who is writing.

- **An agent** acting under an attenuated chain ([`rooms/keys/present`](../../present/0.2/spec.md)) signs its own body, as its own author, and asks only for the seal. That is `0.1`'s behaviour, unchanged.
- **The principal's own client** — a CLI, a browser extension, a phone, acting as the member rather than for them — holds no key of the member's. Its writes must be the member's, signed by the member's DID, and the only party holding that key is the key holder. So it asks for both: `signAsPrincipal: true`, and the key holder signs the body as the principal before sealing it.

This is the write half of acting as the principal. The other half — the host-bound document that carries the sealed record, its presentation and its proof, all as the principal — is [`rooms/keys/act`](../../act/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

### What changed from 0.1

`0.1` said a key holder "seals what it is given and does not sign on the writer's behalf: the signature attributes the record, and that is the writer's act". That holds for an agent, which has a key of its own. It left the member's own client with no way to write as the member: the client holds no key of the member's, so it signed with its own, and every record it wrote was attributed to the client's identifier rather than to the member — on an `attributed` room, to an identity the room never admitted.

`0.2` adds one optional member, `signAsPrincipal`. Absent or `false`, the task is `0.1`. The change is a new member on a closed payload, so a `0.1` consumer refuses it, and it is a `MINOR` increment per [SPEC §5.2](/SPEC.md#52-compatibility-rules).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer**:

1. **MUST** set `signAsPrincipal: true` only when it acts as the principal — the member's own client — and **MUST** then send a `SealedRecordBody` without `proof` whose `author` is the member it acts as and whose `roomId`, `key` and `version` equal the payload's.
2. **MUST NOT** set `signAsPrincipal` when it acts under a chain attenuated to itself. An agent signs its own body as its own author; asking its principal's key holder to sign as the principal would let it write as somebody it was only delegated to act for.

A conforming **recipient** (the key holder):

1. **MUST** refuse with `notAMember` when it holds no group state for `roomId` for the member the request concerns — the body's `author` under `signAsPrincipal`, otherwise the member its caller acts for.
2. Under `signAsPrincipal`, **MUST** honour the request only from a caller it accepts as acting **as** its principal, and **MUST** refuse a caller to which the principal has delegated narrower standing with the standard `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)).
3. Under `signAsPrincipal`, **MUST** decode `plaintext` and refuse with `bodyInvalid` unless it is a `SealedRecordBody` with no `proof` and no unknown member; **MUST** refuse with `authorNotPrincipal` unless `author` is a member of this room the caller may act as, and with `locationMismatch` unless `roomId`, `key` and `version` equal the payload's.
4. Under `signAsPrincipal`, **MUST** sign the body with a W3C Data Integrity proof made with a key of `author`'s DID document listed for `assertionMethod`, with `proofPurpose: assertionMethod`, as `SealedRecordBody` requires, and **MUST** seal the signed body's UTF-8 JSON — never the unsigned one, and never one signed by any other identifier.
5. **MUST NOT** sign a body without `signAsPrincipal`, and **MUST NOT** alter a body it was not asked to sign: without the flag it seals exactly the bytes it was given, as `0.1` did.
6. **MUST** seal under its current epoch for the member concerned, with associated data binding `roomId`, `key`, `version` and that epoch, and **MUST** return the epoch it sealed under.
7. **MUST NOT** retain the plaintext, the signed body or the ciphertext.

A key holder serving several members — several principals, or several personas of one — keeps one group state per member per room, and seals with the state of the member the request concerns. State held for another member **MUST NOT** be used.

## Authorization

**The entitlement is being — or acting as — the recipient's own principal**, as with every custody task in this family. A key holder holds a member's room keys and their signing keys; this asks it to use them.

The flag does not widen what a caller may obtain. A caller that may act as the principal already reaches everything the principal holds through this key holder; the flag lets it reach the one thing it could not do for itself, which is sign as the member. A caller that may not act as the principal — an agent holding an attenuated chain — is refused the flag (item 2), and its writes stay its own.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome.

## Definitions

**`roomId`** — the room to seal for. The recipient **MUST** hold group state for it.

**`key`** — the record key this will be stored under. On a sealed tier it **MUST** be opaque: a descriptive key is readable by the host and defeats the encryption beside it.

**`version`** — the version the writer intends the record to take, bound into the associated data. See `0.1`, *The binding is why `version` is an input*, which applies unchanged.

**`plaintext`** — the record body, base64url. Without `signAsPrincipal`, a `SealedRecordBody` the writer has already signed. With it, the same object without `proof`.

**`signAsPrincipal`** — `true` asks the recipient to sign the body as its `author`, a member the caller acts as, before sealing it. Absent means `false`.

**`sealed`** (response) — a [`SealedRecord`](../../../_shared/0.1/room.schema.json): ciphertext, nonce, and the epoch it was sealed under.

## Request

A **Writer** sends this to its principal's **KeyHolder**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### The member's own client writes, as the member

`plaintext` decodes to `{"roomId":"did:webvh:example.com:rooms:northwind","key":"giXFLTGBdnnQJRoIsktuIg","version":1,"author":"did:webvh:example.com:people:alice","body":"Do not reprice before renewal."}` — the body, unsigned, naming the member as its author.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/keys/seal/0.2#request",
  "issuer": "did:example:client",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "key": "giXFLTGBdnnQJRoIsktuIg",
    "version": 1,
    "plaintext": "eyJyb29tSWQiOiJkaWQ6d2Vidmg6ZXhhbXBsZS5jb206cm9vbXM6bm9ydGh3aW5kIiwia2V5IjoiZ2lYRkxUR0Jkbm5RSlJvSXNrdHVJZyIsInZlcnNpb24iOjEsImF1dGhvciI6ImRpZDp3ZWJ2aDpleGFtcGxlLmNvbTpwZW9wbGU6YWxpY2UiLCJib2R5IjoiRG8gbm90IHJlcHJpY2UgYmVmb3JlIHJlbmV3YWwuIn0",
    "signAsPrincipal": true
  }
}
```

### An agent writes, as itself

The body is already signed by the agent, which is its `author`; the key holder only seals.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000011",
  "type": "https://trusttasks.org/spec/rooms/keys/seal/0.2#request",
  "issuer": "did:example:agent",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000011ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "key": "kq0R7mZp2Yd8cN1vXwT4sA",
    "version": 2,
    "plaintext": "eyJyb29tSWQiOiIuLi4iLCJwcm9vZiI6ey4uLn19"
  }
}
```

## Response

The **KeyHolder** responds with the sealed record, using the sub-schema reachable via `$anchor: "response"`. `epoch` is the one it sealed under — the caller does not choose it, and a host will reject a record whose epoch is not the room's current one.

Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification).

### Ready to hand to a host

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/keys/seal/0.2#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:client",
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

### Refused: the body names somebody the caller does not act as

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:client",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "code": "rooms/keys/seal:authorNotPrincipal",
    "message": "The body's author is not a member of this room you may act as.",
    "retryable": false,
    "details": {
      "author": "did:webvh:example.com:people:bob"
    }
  }
}
```

## Security & Privacy

**A signature as the member is the member's act, and the key holder is the member's infrastructure.** It already signs as the member wherever the member's surfaces cannot — which is everywhere, since the member's keys never leave it. Signing a record body is the same act. What keeps it the member's is item 2: only a caller acting *as* the member may ask for it, and an agent never can.

**The body is signed, not composed.** The key holder signs exactly the body it was given, after checking that its `author` and location are right; it does not fill in, reorder or omit anything. A writer reading back its own record sees the body it wrote.

### Data carried

The request carries the record's **plaintext**, which is why `ingests` is `secret`. That is not a weakening: the key holder is the principal's own, and it is already the party that decrypts every record the principal reads. What comes back is ciphertext, so the response discloses only that a record of roughly that size was sealed. A producer **MUST NOT** put record content in `ext`.

### Correlation

The key holder learns what its principal writes, which it would learn anyway from `open`. To an observer the informative signal is timing: a seal is followed by a `rooms/records/put` to the room's host, so the pair links a member to a write even where the host cannot read it. On a `private` room a producer **SHOULD** not make the two requests distinguishable as a pair by timing alone.

Under `signAsPrincipal` the record carries the member's identifier and nothing of the client's: the client's identifier appears in no member of the body and in no proof over it.

### Retention

**The key holder retains neither the plaintext, the signed body nor the ciphertext.** The record belongs at the host.

### Consent/purpose

The plaintext is sent so that it can be written to a room the principal is a member of, under the principal's name. It confers nothing the principal did not already hold: the authority to store the result is presented separately, to the host.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
