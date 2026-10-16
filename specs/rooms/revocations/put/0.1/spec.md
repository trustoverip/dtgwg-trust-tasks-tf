---
slug: rooms/revocations/put
version: "0.1"
title: "Rooms Revocations — Put"
summary: "An administrator of a data room delivers the room's signed revocation list to its host, which refuses every withdrawn credential — and everything delegated from one — from the moment it accepts the list."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - room
  - revocation
  - revoke
  - removal
  - membership
  - credential
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
  rationale: "The request changes which credentials a host honours for every member of a room. Its delivery must be attributable to an administrator independently of the transport, beside the room's own signature on the list it carries."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. Replay of the document cannot roll anything back — a host accepts only a higher sequence — but the delivery must still be placeable in time for the room's audit."
sideEffects:
  level: mutating
  rationale: "Replaces the revocation list the host holds for the room, and with it the set of presentations the host refuses."
subjectPath: /roomId
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: "On `open` and `attributed` rooms the list names withdrawn credentials and parties, which the host has already seen presented. On a `private` room it carries only room-wide entries, which name nobody. The presentation discloses the delivering administrator on `open` and `attributed`."
retention:
  class: durable
  rationale: "The host keeps the latest list for as long as it serves the room: it is enforced on every later request, and a room moving hosts relies on each host holding it."
errorCodes:
  - code: rooms/revocations/put:listInvalid
    meaning: "The list is not one the room signed for this room: its `roomId` differs from the request's, its proof does not verify with a key the room's DID document lists for `assertionMethod`, or an entry names the room itself as a subject."
    retryable: false
  - code: rooms/revocations/put:stale
    meaning: "The list's sequence is below that of the list the host holds. Nothing changed. The held sequence is carried in `details.heldSequence`, so the key holder learns the host is ahead of what it delivered."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [heldSequence]
      properties:
        heldSequence:
          type: integer
          minimum: 1
  - code: rooms/revocations/put:sequenceConflict
    meaning: "The host holds a different list at the same sequence. The room has signed two lists at one position — its key holder has forked, or its key is in more than one party's hands. Nothing changed: the host keeps the list it holds."
    retryable: false
  - code: rooms/revocations/put:notAppendOnly
    meaning: "The list omits an entry of the held list whose `until` has not passed. A withdrawal is never undone by omission; a room re-admits a party by issuing fresh credentials, which a `subject` or `room` entry does not reach."
    retryable: false
  - code: rooms/revocations/put:entryNotPermitted
    meaning: "The room is `private` and the list carries a `credential` or `subject` entry, which a host cannot apply there and which would disclose who was in the room."
    retryable: false
related:
  - rooms/owner/revoke
  - rooms/revocations/get
  - rooms/owner/group/remove
  - rooms/owner/issue-authority
  - rooms/owner/issue-membership
  - rooms/epoch/mint
  - rooms/records/put
---

## Abstract

The **Rooms Revocations — Put** Trust Task makes a room's withdrawals take effect at its host **at once**. An administrator delivers the room's **revocation list** — a cumulative, numbered list the room signs with its own key ([`RevocationList`](../../../_shared/0.1/room.schema.json)) — and from the moment the host accepts it, every presentation carrying a withdrawn credential is refused, under every task the host serves for the room.

Without it, a room has no way to take back a grant before it expires. A host authorizes from credentials the room issued and keeps no roster, so removing a member from the room's group ([`rooms/owner/group/remove`](../../../owner/group/remove/0.1/spec.md)) denies them what is sealed next but leaves the host honouring their membership and authority credentials until their `validUntil` — reading what the host still serves, writing into the room, fetching its blobs and its commit relay. Short expiries narrow that window; they do not close it.

**The list is the room's, not the host's.** It is signed by the room, carries every withdrawal still in force, and is numbered, so it means the same thing at every host it reaches. A host holds the latest one it accepted and consults it on every request — the one piece of state a host consults beyond the presentation, and the reason it is room-signed state rather than a host's own: a room that moves hosts carries its withdrawals with it, and a host cannot add to or subtract from what the room withdrew.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (an administrator of the room — normally the owner's key holder, through [`rooms/owner/revoke`](../../../owner/revoke/0.1/spec.md)):

1. **MUST** present a chain conferring `admin` at the room's scope.
2. **MUST** deliver the list unaltered, as the room signed it. A producer that is not the room's key holder relays; it does not edit.

A conforming **consumer** (the host):

1. **MUST** verify the presentation, under the chain rules of [`rooms/records/put`](../../../records/put/0.2/spec.md) and against the list it **already** holds, before reading the list, and **MUST** refuse a presentation that does not confer `admin` with the framework's standard `permissionDenied`. Checking the deliverer against the list it delivers would let the order of two deliveries decide which one counts; the list's own authority is the room's signature.
2. **MUST** refuse with `listInvalid` a list whose `roomId` is not the request's, whose `proof` does not verify with a key the room's DID document lists for `assertionMethod`, or which carries a `subject` entry naming the room. The list is the room's statement or it is nothing: a proof by any other key is refused however well it verifies ([`AuthorityPresentation`](../../../_shared/0.1/room.schema.json), *every credential is its issuer's attestation*).
3. **MUST**, on a `private` room, refuse with `entryNotPermitted` a list carrying a `credential` or `subject` entry.
4. **MUST** accept the list only under the sequence and append-only rules of `RevocationList`, refusing with `stale`, `sequenceConflict` or `notAppendOnly`, and changing nothing when it refuses. A repeat — the same sequence and an identical list — is answered as accepted, with the `inForceAt` of its first acceptance.
5. **MUST** store an accepted list durably **before** answering, and **MUST** answer with the head of the list it then holds.
6. **MUST**, from the moment it accepts a list, refuse every presentation that list revokes (*Enforcement*, below).

### Enforcement

A host that has accepted a list for a room **MUST** apply it to every presentation it verifies for that room, and a presentation it revokes is refused with the framework's standard `permissionDenied`:

1. **Every link, and the membership credential.** A presentation is revoked when its membership credential, or any link of its `authority` chain, is matched by an entry. A chain from which any link is revoked confers nothing — what an agent was delegated by a withdrawn member is withdrawn with them, at whatever depth.
2. **Every task, every version.** The list governs the room, not a task: [`rooms/records/*`](../../../records/put/0.2/spec.md), [`rooms/blobs/*`](../../../blobs/get/0.2/spec.md) including the issuance of a storage target, [`rooms/epoch/*`](../../../epoch/mint/0.1/spec.md) including the commit relay and the key chain, [`rooms/info`](../../../info/0.1/spec.md), [`rooms/usage`](../../../usage/0.1/spec.md), succession, and this task's own presentation — at every version of each the host serves, including versions published before this one. A host does not get to serve an older version as a way round a withdrawal.
3. **From the moment of acceptance, with nothing cached.** A mutation that takes effect after the host accepts a list **MUST** have been authorized against it. A host that verified a presentation before accepting a list and commits its effect afterwards has let a withdrawn party act after the withdrawal; acceptance and authorization **MUST** be serialized per room so that cannot happen. A multi-step operation begun before acceptance — an upload — is authorized again at each step that presents, and the step that commits it is refused.
4. **What is already out of the host's hands.** A storage target issued before acceptance ([`rooms/blobs/get`](../../../blobs/get/0.2/spec.md)) is a bearer URL at a store the host may not control, and remains usable for the rest of its lifetime — minutes. No new one is issued. A host that serves targets from storage it controls **SHOULD** refuse those too.

The list is applied whether or not the room's epoch has moved. A withdrawal and an epoch advance are independent: a host that has accepted a list refuses what it revokes even before the advance arrives, and an advance that arrives first changes nothing about which credentials verify. A host **MUST NOT** wait for one before applying the other — on a `private` room it could not tell which leaf the advance removed even if it tried.

### Matching

An entry matches a credential as `RevocationEntry` states: a `credential` entry by `id`; a `subject` entry by subject **or issuer**, for a credential whose `validFrom` is earlier than the entry's `revokedAt`; a `room` entry for any credential whose `validFrom` is earlier than its `revokedAt`. A credential without `validFrom` is treated as issued before every entry. On a `private` room, where the presentation withholds the credentials, a host **MUST** refuse a presentation for which it cannot establish that no credential in it has a `validFrom` earlier than the latest `room` entry's `revokedAt` — a profile that cannot prove it cannot be served while a `room` entry is in force.

Matching is by time, not by identity, so a party can come back. A room that re-admits someone it withdrew with a `subject` entry issues them fresh credentials, and those verify; nothing in the list has to be removed for that to work, and nothing may be.

## Authorization

The authority this task presupposes is **`admin` at the room's scope**, conferred by the room and presented as an authority chain verified link by link to a credential the room issued — the same authority that mints an epoch. The host consults no access-control list of its own.

The presentation authorizes the **delivery**. What is withdrawn is decided by the list, and the list's authority is the room's signature on it: an administrator who could deliver an arbitrary list but not sign one cannot withdraw anything the room did not. That split is deliberate. Signing as the room needs the room's key, which the owner's key holder holds ([`rooms/owner/revoke`](../../../owner/revoke/0.1/spec.md)); delivering needs only standing in the room, so any administrator — or the key holder speaking for the owner — can bring a host up to date.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome.

## Definitions

- **Revocation list**, **entry**, **head** — as `RevocationList`, `RevocationEntry` and `RevocationHead` in [`rooms/_shared/0.1/room.schema.json`](../../../_shared/0.1/room.schema.json).
- **Held list** — the list the host last accepted for the room. A host that has accepted none holds none, and accepts any valid list whatever its sequence.
- **`inForceAt`** (response) — when the host began refusing what the list withdraws.

## Request

An **Administrator** (`issuer`) sends this to the room's **Host** (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Withdrawing a member who left, and an agent credential that leaked

```json
{
  "id": "urn:uuid:5b1f0c34-2a7e-4c8b-9d61-3e0f8a1b2c01",
  "type": "https://trusttasks.org/spec/rooms/revocations/put/0.1#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:rooms.northwind.example",
  "issuedAt": "2026-10-16T09:30:00Z",
  "threadId": "urn:uuid:5b1f0c34-2a7e-4c8b-9d61-3e0f8a1b2cff",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "presentation": {
      "membership": "eyJhbGciOiJFZERTQSJ9.membership.sig",
      "authority": [
        "eyJhbGciOiJFZERTQSJ9.authority-admin.sig"
      ]
    },
    "list": {
      "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
      "sequence": 3,
      "issuedAt": "2026-10-16T09:29:58Z",
      "entries": [
        {
          "kind": "credential",
          "credentialId": "urn:uuid:44444444-4444-4444-8444-444444444444",
          "revokedAt": "2026-09-02T14:00:00Z",
          "until": "2026-12-01T00:00:00Z"
        },
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
    }
  }
}
```

## Response

The **Host** answers with the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`.

### Accepted, and in force

```json
{
  "id": "urn:uuid:5b1f0c34-2a7e-4c8b-9d61-3e0f8a1b2c02",
  "type": "https://trusttasks.org/spec/rooms/revocations/put/0.1#response",
  "issuer": "did:web:rooms.northwind.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-16T09:30:00Z",
  "threadId": "urn:uuid:5b1f0c34-2a7e-4c8b-9d61-3e0f8a1b2cff",
  "payload": {
    "roomId": "did:webvh:QmRoom:rooms.northwind.example:deal",
    "head": {
      "sequence": 3,
      "digest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR"
    },
    "inForceAt": "2026-10-16T09:30:00Z"
  }
}
```

### Refused: the host already holds a later list

```json
{
  "id": "urn:uuid:5b1f0c34-2a7e-4c8b-9d61-3e0f8a1b2c03",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:web:rooms.northwind.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-16T09:30:00Z",
  "threadId": "urn:uuid:5b1f0c34-2a7e-4c8b-9d61-3e0f8a1b2cff",
  "payload": {
    "code": "rooms/revocations/put:stale",
    "message": "This host holds a later revocation list for the room.",
    "retryable": false,
    "details": {
      "heldSequence": 4
    }
  }
}
```

## Security & Privacy

**Why a pushed, room-signed list.** A W3C Bitstring Status List referenced from each credential's `credentialStatus` was considered and not adopted. A host would have to fetch it, which this family forbids for the reasons it never dereferences a chain's `parent`: verification would depend on the availability of whoever serves the list, every check would signal credential use to them, and the location is one the issuer — or a holder relaying a credential — chooses. A cached list is stale by its cache lifetime, which is not *immediately*. And a status list buys its privacy from herd size, which a room's few dozen credentials do not have. A host-held list fed only by a push, without the room's signature, was rejected for the opposite reason: it would be the host's own state, unverifiable by the next host and unmovable with the room. The list here is pushed, so it needs no inbound surface at the room's owner and no fetch at the host, and room-signed, so it is portable.

**Rollback.** Nobody can roll a host back. A host accepts only a higher sequence, and only a list that keeps every live entry it holds; an old list the room once signed is `stale` wherever a later one has been delivered. Two different lists at one sequence are refused and the held one kept, and they are evidence that the room's key holder forked or its key is in more than one party's hands. What a host cannot know is that a list newer than the one it holds exists and has not reached it. The owner's key holder therefore delivers the list to every host serving the room ([`rooms/owner/revoke`](../../../owner/revoke/0.1/spec.md)), including one the room has just moved to; the head in each response lets it confirm that every host holds what it signed.

**A host that does not enforce.** A host that ignores the list honours withdrawn credentials. Nothing in this family can prevent that, any more than it can stop a host serving a record to anyone who asks: the host is the enforcement point. What the family can do is make enforcement checkable — a host that answered with a head has claimed to enforce that list from `inForceAt`, in a signed response.

**Opaque refusals.** A withdrawn credential is refused with the same `permissionDenied` as every other failed presentation, and the refusal does not say which rule failed. A distinct code would tell whoever holds a captured credential that its withdrawal has been noticed and propagated, and tell a removed party which host has caught up. The administrator delivering a list learns the specific outcome; the parties it withdraws learn only that they are not authorized.

### Data carried

A presentation and a list. On `open` and `attributed` rooms the list names withdrawn credential identifiers and withdrawn parties' DIDs — parties the host has seen act, since those tiers disclose the acting member. On a `private` room it carries only `room` entries: times, and nobody.

### Correlation

On `attributed`, a host can read from the list *that* a member was withdrawn and *when*. It could already infer most of that from their activity stopping and from an epoch advance; the list states it. On `private`, a `room` entry tells the host that every credential was reset, and nothing about whom. The list is never published: it reaches the hosts it is delivered to and nobody else, and the room's DID document carries no part of it.

### Retention

The host keeps the latest list it accepted for as long as it serves the room, and replaces it only with a later one. An entry whose `until` has passed may be discarded. A host that stops serving a room discards the list with the room.

### Consent/purpose

The list is held for one purpose: to refuse the credentials it withdraws. A host **MUST NOT** use it for anything else — not to build a history of a room's membership, and not to refuse a withdrawn party in another room on the strength of it. A withdrawal is the room's decision about the room.
