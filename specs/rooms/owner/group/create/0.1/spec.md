---
slug: rooms/owner/group/create
version: "0.1"
title: "Rooms Owner — Group — Create"
summary: "A room's owner asks their own agent to form the room's MLS group, with the owner as its first member, so a sealed room can be written to before anyone else has joined."
status: draft
targetFrameworkVersion: "0.5.0"
category: ai-agents
keywords: [rooms, owner, group, mls, sealed, founding]
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
  rationale: "The recipient founds the group whose key schedule seals every record the room will hold. A request whose origin depended on the transport would let a compromised channel found a room's group under keys the owner never chose to hold."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Founding is done once per room. A replay must be placeable in time so a recipient can tell it from a deliberate second attempt after a failure."
sideEffects:
  level: mutating
  rationale: "The recipient creates and retains the room's group state: the owner's leaf, its signature key and the group's secrets."
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request names a room and a key; the response carries an epoch. The group's secrets are created inside the recipient and never leave it."
retention:
  class: durable
  rationale: "Group key material persists for as long as the owner's membership does. Without it the owner can neither seal nor open the room."
errorCodes:
  - code: rooms/owner/group/create:noSigningKey
    meaning: "The recipient holds no key by that identifier, or the key is not one its caller may name."
    retryable: false
  - code: rooms/owner/group/create:alreadyExists
    meaning: "The recipient already holds group state for this room. Its current epoch is carried in `details.epoch`."
    retryable: false
related:
  - rooms/create
  - rooms/owner/register
  - rooms/owner/group/add
  - rooms/keys/welcome
  - rooms/keys/seal
---

## Abstract

A room's owner asks their **key holder** to form the room's MLS group, with the owner as its only member.

On an `attributed` or `private` room every record and every file is sealed under a key derived from the group, so until a group exists nobody can write anything — including the owner. Every other member reaches the group by a Welcome ([`rooms/keys/welcome`](../../../../keys/welcome/0.1/spec.md)); the owner cannot, because there is nobody to welcome them. This is the step that makes the first member, and it is the only one in the family that creates a group rather than joining one.

**The group's room epoch is 1 when it is formed**, which is the epoch a host records when a room is registered ([`rooms/create`](../../../../create/0.1/spec.md)). MLS counts its epochs from 0 and a room from 1, so a group formed here and a room registered at its host agree without anything being minted.

An `open` room has no group: its records are stored in the clear. A recipient MAY refuse this task for a room it knows to be `open`, but is not required to know the tier.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming recipient:

1. **MUST** refuse with `noSigningKey` unless it holds the key `signingKeyId` names and the caller may name it — the same gate as [`rooms/owner/issue-membership`](../../../issue-membership/0.1/spec.md).
2. **MUST** refuse with `alreadyExists`, and change nothing, when it already holds group state for `roomId` — whether it formed that group or joined it by a Welcome. Replacing a group would discard the key schedule every sealed record of the room depends on.
3. **MUST** form the group with exactly one leaf, whose credential identifies `member` (absent: the caller), and **MUST** retain the group state durably before answering.
4. **MUST** answer with the room epoch, 1, and **MUST NOT** return any of the group's secrets.

## Authorization

**The entitlement is control of the room's signing key**, as it is for issuing the room's credentials. Founding the room's group is the founding act of its membership, and only the party that can issue in the room's name can make it.

Note what this does **not** check. Forming a group in one's own key holder for a room one does not own would admit nobody — no member of the real room would follow it — so the key gate is not there to protect other members. It is there so a key holder does not hold founding state for rooms its principal does not run, which a later, genuine Welcome to that room would then collide with.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient` or `proof` establishes *who sent this* and *that it is unaltered* — never that they were entitled to the outcome.

## Definitions

**`roomId`** — the room whose group is formed. Its identity exists already.

**`signingKeyId`** — the room's signing key, as named to [`rooms/owner/issue-membership`](../../../issue-membership/0.1/spec.md).

**`member`** — the DID the owner's leaf identifies: the subject of the owner's membership and authority credentials. Absent means the caller. It is the identity other members' key holders see when they process this owner's commits, so it SHOULD be the DID the owner presents to the room.

**`epoch`** (response) — the room epoch the group is at: 1.

### Before or after registration

Either order works, because neither step depends on the other: a host never sees the group, and the group never names a host. Forming the group first means a room is writable the moment it is registered; forming it after means a registration that a host refused leaves no group behind. A surface creating a room SHOULD register first and form the group second, so a refused registration costs nothing to retry.

## Request

An **Owner** sends this to their **KeyHolder**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Founding the group of a room the owner just registered

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/owner/group/create/0.1#request",
  "issuer": "did:example:owner",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "signingKeyId": "room-northwind-signing"
  }
}
```

## Response

The **KeyHolder** returns the epoch it formed the group at, using the sub-schema reachable via `$anchor: "response"`. Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §8.1](/SPEC.md#81-the-trust-task-error-specification).

### The group, at the epoch the host recorded at registration

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/owner/group/create/0.1#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:owner",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "epoch": 1
  }
}
```

## Security & Privacy

### Data carried

A room identifier, a key identifier and a DID in; an epoch out. The group's secrets are created inside the recipient and never cross the wire.

### Correlation

None beyond what the recipient already holds: it minted the room's identity and signs its credentials.

### Retention

The group state is durable for as long as the owner remains a member. Losing it loses the owner's ability to seal and open the room; a recipient SHOULD include it in whatever backup protects its other key material.

### Consent/purpose

The owner's own request, to their own key holder, about a room they run.
