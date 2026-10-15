---
slug: rooms/owner/group/remove
version: "0.1"
title: "Rooms Owner — Group — Remove"
summary: "A room's owner asks their own agent to remove a member from the room's MLS group: drop their leaf and advance the room's epoch at its host, so nothing written from then on is readable to them."
status: draft
targetFrameworkVersion: "0.5.0"
category: ai-agents
keywords: [rooms, owner, group, mls, sealed, remove, revoke, epoch]
parties:
  - role: Owner
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: KeyHolder
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "The recipient ends a party's access to everything the room seals from now on, and advances the room at its host in the owner's name. A request whose origin depended on the transport would let a compromised channel evict members from a room nobody else can audit."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A removal advances the room's epoch. A replay must be placeable in time so it is not taken for a second advance."
sideEffects:
  level: mutating
  rationale: "The recipient advances its group, and makes an outbound request that advances the room's epoch at its host."
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: "The recipient speaks to the host as its principal, presenting the owner's `admin` authority, and discloses to it a new epoch number, an epoch link and a commit — all opaque to a party without the group. Which member left is inside the commit and is not disclosed to the host."
retention:
  class: durable
  rationale: "The advanced group state persists, as it must for the owner to keep sealing and opening the room."
errorCodes:
  - code: rooms/owner/group/remove:noGroup
    meaning: "The recipient holds no group state for this room."
    retryable: false
  - code: rooms/owner/group/remove:notMember
    meaning: "`member` has no leaf in the group."
    retryable: false
  - code: rooms/owner/group/remove:ownLeaf
    meaning: "`member` is the recipient's own leaf. A key holder does not remove itself; an owner leaving a room hands it on with rooms/owner/transfer."
    retryable: false
  - code: rooms/owner/group/remove:noAuthority
    meaning: "The recipient holds no authority credential conferring `admin` on this room for its caller, so it cannot advance the room's epoch at the host."
    retryable: false
  - code: rooms/owner/group/remove:epochConflict
    meaning: "The host is at a different epoch than the recipient's group — another admin advanced the room. Catch up (rooms/epoch/commits) and try again. The host's epoch is carried in `details.hostEpoch`."
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      required: [hostEpoch]
      properties:
        hostEpoch:
          type: integer
          minimum: 1
  - code: rooms/owner/group/remove:hostUnreachable
    meaning: "The named host could not be resolved, advertises no transport this recipient speaks, or did not answer. The outcome is unknown — the host may have accepted the advance — so the recipient keeps the pending removal and the next request to it settles it (item 6)."
    retryable: true
  - code: rooms/owner/group/remove:hostRefused
    meaning: "The host answered and declined the epoch advance. Its reason is carried in `details`. Nothing was changed."
    retryable: false
related:
  - rooms/owner/group/create
  - rooms/owner/group/add
  - rooms/owner/group/renew
  - rooms/owner/transfer
  - rooms/keys/commit
  - rooms/epoch/mint
  - rooms/epoch/commits
---

## Abstract

A room's owner asks their **key holder** to remove a member from the room's MLS group.

Removal is an epoch advance, and nothing else is. A membership credential that lapses stops a member presenting it, but they still hold the keys to every epoch they were in; what stops them reading what is written next is a new epoch whose key they do not have. So removal is the mirror of [`rooms/owner/group/add`](../../add/0.1/spec.md): the group loses a leaf, the room's epoch advances at its host with the link and commit, and every remaining member's key holder applies the commit — directly, or from the host's relay ([`rooms/epoch/commits`](../../../../epoch/commits/0.1/spec.md)).

**What removal does not do, and a surface offering it must say so:** it does not recall anything the member already read, and on a `chained` room it does not stop them reading the epochs they were in. It ends their access to what is written from now on.

The member's authority credential is a separate thing and is not touched here. A surface removing someone also lets their authority lapse, or issues none further; until it does, they can still present it to the host for operations their old epochs' keys suffice for — reading what they could already read.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming recipient:

1. **MUST** refuse with `noGroup` when it holds no group for `roomId`, with `notMember` when `member` has no leaf in it, and with `ownLeaf` when `member` is the recipient's own leaf.
2. **MUST** build the commit removing `member`'s leaf, and the epoch link that seals the outgoing epoch's storage key under the incoming one, **without** applying the commit to the group it holds. (A committer cannot process its own commit later, so what it keeps is the advanced group state alongside the one it holds.)
3. **MUST** retain that pending removal — the advanced state, the link and the commit — durably **before** sending anything to the host. A recipient holds at most one pending advance per room, whether an admission, a removal or a renewal, and settles any it finds (item 6) before building another.
4. **MUST** advance the room at `host` with [`rooms/epoch/mint`](../../../../epoch/mint/0.1/spec.md), carrying the new epoch, the link, the commit and a presentation of the caller's `admin` authority — minted as [`rooms/keys/present`](../../../../keys/present/0.2/spec.md) would mint it — in a document addressed to `host` whose proof covers its `recipient` ([SPEC §4.8.2](/SPEC.md#482-audience-binding)). The presentation is bound to the room, not to a host; the document's `recipient` is what binds the advance to `host`.
5. **MUST** apply the pending removal only once the host has accepted the advance, and **MUST** discard it, leaving the group unchanged, when the host **refuses** it. When the outcome is unknown — the host could not be reached, or did not answer — it **MUST** keep the pending removal for item 6.
6. **MUST**, when it finds a pending advance at a later request, ask the host it was sent to for the room's epoch — with [`rooms/info`](../../../../info/0.1/spec.md), under the caller's `read` authority — and apply it if the host is at its epoch, or discard it if the host is still at the epoch before it. A host anywhere else has moved on without it: discard it and refuse with `epochConflict`.

## Authorization

**The entitlement is membership of the group and the room's `admin` authority**, both of which the recipient holds for its caller — as for [`rooms/owner/group/add`](../../add/0.1/spec.md), and for the same reasons. The host verifies the `admin` chain on the epoch advance; the recipient does not decide that its caller is an admin.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome.

## Definitions

**`roomId`** — the room whose group loses a member.

**`host`** — the room's host, as a DID: where its epoch advances.

**`member`** — the DID whose leaf is removed.

**`reason`** — why, carried to the host's epoch advance.

**`epoch`** (response) — the room epoch after the removal.

## Request

An **Owner** sends this to their **KeyHolder**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Removing a member who has left the deal

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/owner/group/remove/0.1#request",
  "issuer": "did:example:owner",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "host": "did:webvh:example.com:northwind-community",
    "member": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "reason": "Left the deal team"
  }
}
```

## Response

The **KeyHolder** returns the room's new epoch, using the sub-schema reachable via `$anchor: "response"`. Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification).

### The room advanced to epoch 4 without them

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/owner/group/remove/0.1#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:owner",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "member": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "epoch": 4
  }
}
```

## Security & Privacy

### Data carried

A room, a host and a DID in; an epoch out. The commit and link go to the host, opaque to it.

### Correlation

On a `private` room the host must not learn who left. It sees an epoch advance, a link and a commit, none of which it can read.

### Retention

The advanced group state is durable. The pending removal of item 3 is retained only until the host's answer is known — by its reply, or by item 6.

### Consent/purpose

Removal is the owner's act alone; the member is not asked. That is the point of it, and why it needs the room's `admin` authority.
