---
slug: rooms/revocations/get
version: "0.1"
title: "Rooms Revocations — Get"
summary: "An administrator of a data room reads the revocation list its host holds and enforces — to confirm a withdrawal reached the host, or to recover the room's list after losing it."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - room
  - revocation
  - audit
  - recovery
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "The response names the parties a room withdrew. Who asked must be established independently of the transport, as on every room read that a presentation authorizes."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A read authorized by a time-bounded presentation must be placeable in time, as every rooms/* request is."
sideEffects:
  level: none
  rationale: "Reads the list the host holds and changes nothing."
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns the room's own signed list to one of its administrators: on `open` and `attributed` rooms the withdrawn credential identifiers and parties, on `private` only times."
retention:
  class: transient
  rationale: "Nothing is recorded by the read."
errorCodes: []
related:
  - rooms/revocations/put
  - rooms/owner/revoke
---

## Abstract

The **Rooms Revocations — Get** Trust Task returns the [`RevocationList`](../../../_shared/0.1/room.schema.json) a host holds for a room, exactly as the room signed it, and when the host began enforcing it.

It answers two questions an administrator has and no other task answers. **Is this host enforcing what the room withdrew?** — the list's head against the one the room's key holder signed. And, for a key holder that has lost its copy of the room's list — restored from a backup older than its last withdrawal — **what did the room sign?** A list is room-signed, so a host serving it cannot have altered it; at worst it serves an older one, which the reader recognises by its sequence.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the host):

1. **MUST** verify the presentation under the chain rules of [`rooms/records/put`](../../../records/put/0.2/spec.md) and against the list it holds, and refuse one that does not confer `admin` with the framework's standard `permissionDenied`.
2. **MUST** return the list it holds unaltered — every member, `proof` included — with the `inForceAt` it answered when accepting it, and **MUST** omit both when it holds none.

A conforming **reader** of the response **MUST** verify the list as a host does before relying on it — `roomId` is this room, and the proof verifies with a key the room's DID document lists for `assertionMethod` — and **MUST NOT** adopt it in place of a list it already holds at a higher sequence. A key holder recovering its list adopts the one at the highest sequence among the hosts it asks.

## Authorization

The authority this task presupposes is **`admin` at the room's scope**, conferred by the room and presented as an authority chain. A member who is not an administrator does not read the list: on `open` and `attributed` rooms it names the parties the room withdrew, which is the room's administration.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome.

## Definitions

- **Revocation list** — as `RevocationList` in [`rooms/_shared/0.1/room.schema.json`](../../../_shared/0.1/room.schema.json).
- **`inForceAt`** (response) — when the host began enforcing the list.

## Request

```json
{
  "id": "urn:uuid:5b1f0c34-2a7e-4c8b-9d61-3e0f8a1b2c11",
  "type": "https://trusttasks.org/spec/rooms/revocations/get/0.1#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:rooms.northwind.example",
  "issuedAt": "2026-10-16T10:00:00Z",
  "threadId": "urn:uuid:5b1f0c34-2a7e-4c8b-9d61-3e0f8a1b2c1f",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "presentation": {
      "membership": "eyJhbGciOiJFZERTQSJ9.membership.sig",
      "authority": [
        "eyJhbGciOiJFZERTQSJ9.authority-admin.sig"
      ]
    }
  }
}
```

## Response

### The list in force since the last withdrawal

```json
{
  "id": "urn:uuid:5b1f0c34-2a7e-4c8b-9d61-3e0f8a1b2c12",
  "type": "https://trusttasks.org/spec/rooms/revocations/get/0.1#response",
  "issuer": "did:web:rooms.northwind.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-16T10:00:00Z",
  "threadId": "urn:uuid:5b1f0c34-2a7e-4c8b-9d61-3e0f8a1b2c1f",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "list": {
      "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
      "sequence": 3,
      "issuedAt": "2026-10-16T09:29:58Z",
      "entries": [
        {
          "kind": "subject",
          "subject": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
          "revokedAt": "2026-10-16T09:29:58Z"
        }
      ],
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:webvh:QmRoom:rooms.northwind.example:deal#key-0",
        "proofPurpose": "assertionMethod",
        "created": "2026-10-16T09:29:58Z",
        "proofValue": "z3FXQjecWufY46yg5abdVZsXqLhxhueuSoZgNSARiKBk9czhSePTFehP8c3PGfb6a22gkfUKcuiBNPvFaEb5PNoq9"
      }
    },
    "inForceAt": "2026-10-16T09:30:00Z"
  }
}
```

## Security & Privacy

**A host cannot alter what it serves, only age it.** The list is the room's signed statement, so a host serving a modified one serves one that does not verify. What a host can do is serve an older list than it holds — which a reader holding a later one ignores, and which a key holder recovering from nothing guards against by asking every host serving the room and taking the highest sequence.

### Data carried

A presentation in; the room's list out.

### Correlation

The list goes only to an administrator of the room, who could have had it from the room's key holder.

### Retention

The read records nothing.

### Consent/purpose

The list is returned so an administrator can confirm enforcement or recover the room's state, and for no other purpose.
