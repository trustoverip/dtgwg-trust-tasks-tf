---
slug: rooms/owner/group/remove
version: "0.2"
title: "Rooms Owner — Group — Remove"
summary: "A room's owner asks their own agent to remove a member: withdraw every credential the room gave them, in force at its host at once, then drop their leaf from the room's MLS group and advance the room's epoch."
status: draft
targetFrameworkVersion: "0.6.0"
category: ai-agents
keywords: [rooms, owner, group, mls, sealed, remove, revoke, revocation, epoch]
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
  rationale: "The recipient ends a party's access to the room — at its host at once, and to everything the room seals from now on — and signs as the room to do it. A request whose origin depended on the transport would let a compromised channel evict members from a room nobody else can audit."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A removal withdraws credentials by time and advances the room's epoch. A replay must be placeable in time so it is taken neither for a second advance nor for a withdrawal after a re-admission."
sideEffects:
  level: mutating
  rationale: "The recipient signs and keeps the room's next revocation list and advances its group, and makes outbound requests that put the list in force at each host and advance the room's epoch."
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: "The recipient speaks to each host as its principal, presenting the owner's `admin` authority. It discloses the room's revocation list, which names `member` — a party an `attributed` room's host has seen act — and to the room's host a new epoch number, an epoch link and a commit, all opaque to a party without the group."
retention:
  class: durable
  rationale: "The advanced group state and the room's revocation list persist, as they must for the owner to keep sealing the room and to sign the next list."
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
  - code: rooms/owner/group/remove:noSigningKey
    meaning: "The recipient holds no key by `signingKeyId`, or the key is not one its caller may name."
    retryable: false
  - code: rooms/owner/group/remove:noAuthority
    meaning: "The recipient holds no authority credential conferring `admin` on this room for its caller, so it can neither deliver the withdrawal nor advance the room's epoch at the host."
    retryable: false
  - code: rooms/owner/group/remove:privateRoom
    meaning: "The room is `private`. Its host cannot be told which party to refuse without being told who was a member, so a member cannot be withdrawn one at a time there. Nothing changed. Remove them with rooms/owner/group/remove 0.1, which withdraws nothing at the host, and reset every credential with rooms/owner/revoke if the room needs that too."
    retryable: false
  - code: rooms/owner/group/remove:epochConflict
    meaning: "The host is at a different epoch than the recipient's group — another admin advanced the room. The withdrawal, if it was accepted, stays in force. Catch up (rooms/epoch/commits) and try again. The host's epoch is carried in `details.hostEpoch`."
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
    meaning: "The named host could not be resolved, advertises no transport this recipient speaks, or did not answer. The outcome is unknown, so the recipient keeps the pending removal and the next request to it settles it (item 8). `details.stage` says which step went unanswered."
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      required: [stage]
      properties:
        stage:
          type: string
          enum: [revocation, advance]
  - code: rooms/owner/group/remove:hostRefused
    meaning: "The host answered and declined. `details.stage` says which step: at `revocation` nothing changed; at `advance` the withdrawal is in force at the host and the group is unchanged. The host's own code is in `details.code`."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [stage]
      properties:
        stage:
          type: string
          enum: [revocation, advance]
        code:
          type: string
          maxLength: 128
related:
  - rooms/owner/revoke
  - rooms/revocations/put
  - rooms/owner/group/create
  - rooms/owner/group/add
  - rooms/owner/group/renew
  - rooms/owner/transfer
  - rooms/keys/commit
  - rooms/epoch/mint
  - rooms/epoch/commits
---

## Abstract

A room's owner asks their **key holder** to remove a member — and at `0.2`, removal takes effect at the room's host **immediately**.

Removal is two acts, and `0.1` performed one. Taking the member's leaf out of the room's MLS group and advancing the epoch stops them opening what is written **next**. It does not stop the host honouring the membership and authority credentials the room gave them, which a host authorizes from and which last until their `validUntil`: until then they could still read every record and blob the host serves, fetch the commit relay and the key chain, and write into the room. So `0.2` first **withdraws** their credentials — the room's next [`RevocationList`](../../../../_shared/0.1/room.schema.json), signed as the room and delivered with [`rooms/revocations/put`](../../../../revocations/put/0.1/spec.md) — and only then advances the group.

**Withdraw first, then advance.** In that order there is no instant at which the member is outside the group and still credentialed at the host. The reverse order leaves one: after the advance and before the withdrawal, a removed member who no longer holds the room's key can still present to the host, and could write records labelled with the new epoch that the host cannot tell from real ones. If the withdrawal lands and the advance does not, the member is refused at the host and holds keys only to the epoch the room was already at — and anything written under it is refused to them by the host that stores it.

**What removal still does not do, and a surface offering it must say so:** it does not recall anything the member already read or saved. It ends their access to everything the host serves, from the moment the host accepts the withdrawal, and to everything written from now on.

### What changed from 0.1

`0.1` removed the leaf and left the member's credentials alone. `0.2` adds `signingKeyId` (REQUIRED, to sign as the room), `mirrors` and `until`; withdraws the member's credentials before the advance; answers with the revocation list's head and the withdrawal's time; and refuses a `private` room, where a withdrawal by party is not possible. A breaking change to a `draft` specification, so a `MINOR` increment per [SPEC §5.2](/SPEC.md#52-compatibility-rules). A recipient implementing `0.2` **SHOULD** continue to serve `0.1`, which remains the way to remove a member of a `private` room from its group.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming recipient:

1. **MUST** refuse with `noGroup` when it holds no group for `roomId`, with `notMember` when `member` has no leaf in it, with `ownLeaf` when `member` is the recipient's own leaf, with `noSigningKey` when it holds no key by `signingKeyId` its caller may name, and with `noAuthority` when it holds no `admin` authority for its caller — all before anything else.
2. **MUST** establish the room's tier — from [`rooms/info`](../../../../info/0.1/spec.md) at `host` under its caller's `read` authority, unless it already knows it — and refuse with `privateRoom` on a `private` room, changing nothing.
3. **MUST** build the commit removing `member`'s leaf, and the epoch link that seals the outgoing epoch's storage key under the incoming one, **without** applying the commit to the group it holds.
4. **MUST** build and sign the room's next revocation list as [`rooms/owner/revoke`](../../../revoke/0.1/spec.md) item 3 does, adding one `subject` entry for `member` with `revokedAt` the time it signs and `until` from the request.
5. **MUST** retain the pending removal — the advanced state, the link, the commit and the signed list — durably **before** sending anything to any host. A recipient holds at most one pending advance per room, whether an admission, a removal or a renewal, and settles any it finds (item 8) before building another.
6. **MUST** deliver the list to `host` with [`rooms/revocations/put`](../../../../revocations/put/0.1/spec.md) **before** advancing the room, and **MUST NOT** advance it until `host` has accepted the list. When `host` refuses the list it **MUST** discard the whole pending removal — list included, since no host has accepted it — and refuse with `hostRefused` (`stage: revocation`).
7. **MUST** then advance the room at `host` with [`rooms/epoch/mint`](../../../../epoch/mint/0.1/spec.md), carrying the new epoch, the link, the commit and a presentation of the caller's `admin` authority — minted as [`rooms/keys/present`](../../../../keys/present/0.2/spec.md) would mint it — in a document addressed to `host` whose proof covers its `recipient`. It **MUST** apply the pending advance only once the host has accepted it, and **MUST** discard the pending advance, leaving the group unchanged, when the host **refuses** it, refusing with `hostRefused` (`stage: advance`). The list is **not** discarded then: it is in force at the host and is the room's, and the member stays withdrawn there.
8. **MUST**, when it finds a pending removal at a later request, settle it in order: deliver its list to the host again if the host has not confirmed it (a repeat is answered as accepted), then ask the host for the room's epoch with [`rooms/info`](../../../../info/0.1/spec.md) and apply the pending advance if the host is at its epoch, or discard it if the host is still at the epoch before it; a host anywhere else has moved on without it: discard it and refuse with `epochConflict`. Discarding the advance never discards the list, which is the room's once any host has accepted it — the member stays withdrawn, and a repeated removal advances the group.
9. **MUST**, once `host` has accepted the list, deliver it to each of `mirrors` and name in `mirrorsPending` every mirror that did not answer with its head. The removal succeeds without them; a mirror not yet holding the list is reported, not waited for.

A host receives the withdrawal and the advance as two independent requests and treats them so ([`rooms/revocations/put`](../../../../revocations/put/0.1/spec.md), *Enforcement*): each takes effect when it is accepted, in whichever order they arrive. Item 6 fixes the order this recipient sends them in; nothing at the host depends on it.

## Authorization

**The entitlement is membership of the group, control of the room's signing key, and the room's `admin` authority**, all of which the recipient holds for its caller. Signing the withdrawal as the room is the entitlement of [`rooms/owner/revoke`](../../../revoke/0.1/spec.md) and of [`rooms/owner/issue-authority`](../../../issue-authority/0.2/spec.md); the host verifies the `admin` chain on both the delivery and the epoch advance. The recipient does not decide that its caller is an admin.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome.

## Definitions

**`roomId`** — the room whose group loses a member.

**`host`** — the room's host, as a DID: where the withdrawal is delivered first and the epoch advances.

**`mirrors`** — any other host serving the room. Each must hold the withdrawal too.

**`member`** — the DID whose leaf is removed and whose credentials are withdrawn — every credential naming them as subject or issuer that the room's chain gave them before `revokedAt`.

**`signingKeyId`** — the held key that signs the revocation list as the room.

**`until`** — when the withdrawal may be dropped from the room's list: the latest `validUntil` of anything the room issued `member`.

**`reason`** — why, carried to the host's epoch advance.

**`revocation`**, **`revokedAt`**, **`epoch`**, **`mirrorsPending`** (response) — the list in force at `host`, the withdrawal's time, the room epoch after the removal, and any mirror not yet holding the list.

### Re-admitting someone removed

The withdrawal reaches what the room issued `member` **before** `revokedAt`, and nothing after. Admitting them again is admitting them as anybody is admitted — fresh credentials ([`rooms/owner/issue-membership`](../../../issue-membership/0.1/spec.md), [`rooms/owner/issue-authority`](../../../issue-authority/0.2/spec.md)) and a new leaf ([`rooms/owner/group/add`](../../add/0.2/spec.md)) — and the withdrawal does not have to be undone. What they delegated before removal stays withdrawn.

## Request

An **Owner** sends this to their **KeyHolder**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Removing a member who has left the deal

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/owner/group/remove/0.2#request",
  "issuer": "did:example:owner",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-10-16T09:29:57Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "host": "did:webvh:example.com:northwind-community",
    "member": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "signingKeyId": "room-northwind-signing",
    "until": "2027-01-01T00:00:00Z",
    "reason": "Left the deal team"
  }
}
```

## Response

The **KeyHolder** returns the list in force and the room's new epoch, using the sub-schema reachable via `$anchor: "response"`. Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification).

### Withdrawn at the host, and the room advanced to epoch 4 without them

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/owner/group/remove/0.2#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:owner",
  "issuedAt": "2026-10-16T09:30:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "member": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "revocation": {
      "sequence": 2,
      "digest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR"
    },
    "revokedAt": "2026-10-16T09:29:58Z",
    "epoch": 4
  }
}
```

## Security & Privacy

**Why not withdraw on a `private` room.** A host serving a `private` room verifies membership in zero knowledge and never learns which member acted, so an entry naming the member is one it could not apply — and one that would tell it who had been in the room. The only withdrawal a `private` host can apply is a reset of every credential issued until now (a `room` entry, [`rooms/owner/revoke`](../../../revoke/0.1/spec.md)), after which the room re-issues to those who remain. That is too large an effect to perform implicitly as part of removing one member, so this version refuses and says so; the owner chooses.

**Mirrors.** A mirror that has not received the withdrawal honours the member's credentials still. The removal is not held back for it — a mirror that is down would otherwise prevent anyone being removed — but it is reported, and a surface **SHOULD** show it as a host still serving the removed member.

### Data carried

A room, a host, its mirrors, a DID and a key name in; a list head, a time and an epoch out. The list names `member` to every host it reaches. The commit and link go to the room's host, opaque to it.

### Correlation

On an `attributed` room the host learns from the list that `member` was withdrawn and when; it sees an epoch advance besides. `0.2` refuses a `private` room, so no list naming a member reaches a host that is meant not to know who its members are.

### Retention

The advanced group state and the room's revocation list are durable. The pending removal of item 5 is retained only until each step's outcome is known — by the host's reply, or by item 8.

### Consent/purpose

Removal is the owner's act alone; the member is not asked. That is the point of it, and why it needs the room's key and its `admin` authority.
