---
slug: rooms/owner/group/renew
version: "0.1"
title: "Rooms Owner — Group — Renew"
summary: "A room's owner asks their own agent to renew a sealed room: advance its MLS group with a commit that changes no membership, and advance the room's epoch at its host with that commit, so the room stays writable without anyone joining or leaving."
status: draft
targetFrameworkVersion: "0.5.0"
category: ai-agents
keywords: [rooms, owner, group, mls, sealed, renew, epoch, commit]
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
  rationale: "The recipient advances the group whose key schedule seals everything the room holds from now on, and advances the room at its host in the owner's name. A request whose origin depended on the transport would let a compromised channel move a room's epoch at will."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A renewal advances the room's epoch. A replay must be placeable in time so it is not taken for a second advance."
sideEffects:
  level: mutating
  rationale: "The recipient advances its group, and makes an outbound request that advances the room's epoch at its host."
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: "The recipient speaks to the host as its principal, presenting the owner's `admin` authority, and discloses to it a new epoch number, an epoch link and a commit — all opaque to a party without the group."
retention:
  class: durable
  rationale: "The advanced group state persists, as it must for the owner to keep sealing and opening the room."
errorCodes:
  - code: rooms/owner/group/renew:noGroup
    meaning: "The recipient holds no group state for this room. An `open` room has no group and renews with rooms/epoch/mint directly; a sealed room is renewed by the key holder that holds its owner's group."
    retryable: false
  - code: rooms/owner/group/renew:noAuthority
    meaning: "The recipient holds no authority credential conferring `admin` on this room for its caller, so it cannot advance the room's epoch at the host."
    retryable: false
  - code: rooms/owner/group/renew:epochConflict
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
  - code: rooms/owner/group/renew:hostUnreachable
    meaning: "The named host could not be resolved, advertises no transport this recipient speaks, or did not answer. The outcome is unknown — the host may have accepted the advance — so the recipient keeps the pending renewal and the next request to it settles it (item 6)."
    retryable: true
  - code: rooms/owner/group/renew:hostRefused
    meaning: "The host answered and declined the epoch advance. Its reason is carried in `details`. Nothing was changed."
    retryable: false
related:
  - rooms/owner/group/create
  - rooms/owner/group/add
  - rooms/owner/group/remove
  - rooms/owner/transfer
  - rooms/keys/act
  - rooms/keys/commit
  - rooms/epoch/mint
  - rooms/epoch/commits
  - rooms/info
---

## Abstract

A room's owner asks their **key holder** to renew a sealed room: to advance its epoch without changing who is in it.

A room's epoch lapses. A room nobody renews becomes read-only, and after its retention window its storage may be reclaimed ([`rooms/create`](../../../../create/0.1/spec.md)). On an `open` room renewal is a bare [`rooms/epoch/mint`](../../../../epoch/mint/0.1/spec.md): the host records the next number and nothing else depends on it.

**On a sealed room — `attributed` or `private` — the room's epoch is the MLS group's epoch plus one**, and every record is sealed under a key derived from the group at the room's current epoch. An epoch advance at the host is therefore only correct together with an advance of the group, carried to the host as the `commit` of the mint. A mint without one moves the host and no member: the host is at N+1, every member's key holder is still at N, the host refuses every record sealed at N (a superseded epoch), and the owner's next admission or removal fails `epochConflict`, because its group is behind the room. A renewal that bricks the room for writing is the failure mode, and it looks like success.

Until this task a sealed room had no correct way to renew. Admissions and removals ([`rooms/owner/group/add`](../../add/0.1/spec.md), [`rooms/owner/group/remove`](../../remove/0.1/spec.md)) advance its epoch correctly, but only as a side effect of a membership change; a sealed room whose membership is stable simply lapsed. This task is the third advance, beside those two: a commit that changes no membership — in MLS terms an update of the committer's own leaf, or an empty commit ([RFC 9420 §12.4](https://www.rfc-editor.org/rfc/rfc9420#section-12.4)) — carried to the host the same way.

A member's own client holds no group key, so it cannot make a commit; [`rooms/keys/act`](../../../../keys/act/0.1/spec.md) therefore refuses to complete a `rooms/epoch/mint` for a sealed room, and names this task.

*This paragraph is non-normative.* The advance moves the room's profile too: the profile is stored per epoch, so a key holder that also serves [`rooms/keys/profile/write`](../../../../keys/profile/write/0.1/spec.md) reads it before advancing and re-writes it under the new epoch afterwards, which is what lets a member who joins now — or reads only from where they joined — see what the room is. See that specification's *Keeping the profile readable across epochs*.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming recipient:

1. **MUST** refuse with `noGroup` when it holds no group for `roomId`, and with `noAuthority` when it holds no authority credential conferring `admin` on the room for its caller.
2. **MUST** build a commit that adds and removes no member — one carrying no proposal but an update of the recipient's own leaf, or none at all — and the epoch link that seals the outgoing epoch's storage key under the incoming one, **without** applying the commit to the group it holds. (A committer cannot process its own commit later, so what it keeps is the advanced group state alongside the one it holds.)
3. **MUST** retain that pending renewal — the advanced state, the link and the commit — durably **before** sending anything to the host. A recipient holds at most one pending advance per room, whether an admission, a removal or a renewal, and settles any it finds (item 6) before building another.
4. **MUST** advance the room at `host` with [`rooms/epoch/mint`](../../../../epoch/mint/0.1/spec.md), carrying the new epoch, the link, the commit and a presentation of the caller's `admin` authority, in a document addressed to `host` whose proof covers its `recipient` ([SPEC §4.8.2](/SPEC.md#482-audience-binding)). The presentation is bound to the room, not to a host ([`rooms/keys/present`](../../../../keys/present/0.2/spec.md)); the document's `recipient` is what binds the advance to `host`.
5. **MUST** apply the pending renewal only once the host has accepted the advance, and **MUST** discard it, leaving the group unchanged, when the host **refuses** it. When the outcome is unknown — the host could not be reached, or did not answer — it **MUST** keep the pending renewal for item 6, because the host may have accepted: discarding it then would leave the owner an epoch behind their own room.
6. **MUST**, when it finds a pending advance at a later request, ask the host it was sent to for the room's epoch — with [`rooms/info`](../../../../info/0.1/spec.md), under the caller's `read` authority — and apply it if the host is at its epoch, or discard it if the host is still at the epoch before it. A host anywhere else has moved on without it: discard it and refuse with `epochConflict`.
7. **MUST NOT** advance the room at the host without a commit, and **MUST NOT** carry in the commit a proposal that adds or removes a member. A membership change is an admission or a removal, each of which renews the room as it advances it.

## Authorization

**The entitlement is membership of the group and the room's `admin` authority**, both of which the recipient holds for its caller — as for [`rooms/owner/group/add`](../../add/0.1/spec.md) and [`rooms/owner/group/remove`](../../remove/0.1/spec.md), and for the same reasons. The host verifies the `admin` chain on the epoch advance; the recipient does not decide that its caller is an admin.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome.

## Definitions

**`roomId`** — the sealed room to renew.

**`host`** — the room's host, as a DID: where its epoch advances.

**`reason`** — why, carried to the host's epoch advance.

**`epoch`** (response) — the room epoch after the renewal.

### Remote first

The order in items 2 to 5 is that of [`rooms/owner/group/add`](../../add/0.1/spec.md), for its reason: a key holder that advanced its group and then failed to advance the host would seal to keys at an epoch the room is not at, and nothing would say so until somebody failed to open what it wrote. Advancing the host first, and applying only what the host accepted, means the worst a failure leaves is a renewal that did not happen.

## Request

An **Owner** sends this to their **KeyHolder**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Renewing a room whose membership has not changed

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/owner/group/renew/0.1#request",
  "issuer": "did:example:owner",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "host": "did:webvh:example.com:northwind-community",
    "reason": "Quarterly renewal"
  }
}
```

## Response

The **KeyHolder** returns the room's new epoch, using the sub-schema reachable via `$anchor: "response"`. Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification).

### The room advanced to epoch 5, with the same members

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/owner/group/renew/0.1#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:owner",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "epoch": 5
  }
}
```

## Security & Privacy

**This is the only correct way to keep a sealed room live.** A bare `rooms/epoch/mint` on a sealed room moves the host without the group and leaves every member unable to write; letting the room lapse leaves it read-only. A surface offering an owner "renew" on a sealed room **SHOULD** reach this task, and **SHOULD NOT** offer a mint of its own.

**Every remaining member's key holder must apply the commit.** A renewal is an epoch advance like any other: a member whose key holder has not applied the commit can open everything sealed before it and nothing sealed after, and the host refuses what it seals at the superseded epoch. The commit reaches them as for any advance — delivered with [`rooms/keys/commit`](../../../../keys/commit/0.1/spec.md), or fetched from the host's relay with [`rooms/epoch/commits`](../../../../epoch/commits/0.1/spec.md).

**A renewal changes nobody's access.** The commit adds and removes no one (item 7). A party that has been removed stays removed; a party not yet admitted is not admitted.

### Data carried

A room and a host in; an epoch out. The commit and link go to the host, opaque to it: the host learns an epoch number and two blobs it cannot open.

### Correlation

The host learns that the room advanced and when. It cannot read the commit, so it cannot tell from it that no member joined or left; a regular cadence of advances may suggest renewal, and an operator wanting renewals and membership changes indistinguishable at the host may renew at irregular times.

### Retention

The advanced group state is durable. The pending renewal of item 3 is retained only until the host's answer is known — by its reply, or by item 6.

### Consent/purpose

Renewal is the owner's act alone; no member is asked, and none needs to be, because nobody's access changes. It needs the room's `admin` authority because every epoch advance does.
