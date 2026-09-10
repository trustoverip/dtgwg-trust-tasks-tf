---
slug: rooms/owner/anchor
version: "0.1"
title: "Rooms Owner — Anchor"
summary: "A room's owner writes the room's current epoch authenticator, version watermark and data commitment into its witnessed log."
status: draft
targetFrameworkVersion: "0.5.0"
category: access-control
keywords:
  - room
  - anchor
  - witness
  - equivocation
  - commitment
authors:
  - Glenn Gore (https://github.com/stormer78)
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
  rationale: >-
    The request makes the recipient publish a statement in the room's name and rotate the room DID's update key. One whose origin depended on the transport would let a compromised channel move a room's identity.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    An anchor is a statement about a room at a moment, and a request to make one that cannot itself be placed in time cannot be reasoned about against the log entry it produced.
sideEffects:
  level: mutating
  rationale: >-
    Publishes a `did:webvh` update, which is witnessed and permanent, and which rotates the room DID's update key as a parallel consequence. Nothing here is reversible: an anchor can be superseded and not withdrawn.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: true
  rationale: >-
    `actsAsSubject` twice over: the recipient presents its principal's room credentials to the named host to read the head, and then acts as the room's DID controller to publish. What is disclosed is public by design — an anchor is written to be read by anyone, which is what makes it singular.
retention:
  class: durable
  rationale: >-
    The anchor is a log entry and lives for the life of the room's DID. That is the point: a statement that could be withdrawn would be one a host could wait out.
errorCodes:
  - code: rooms/owner/anchor:notTheOwner
    meaning: "The recipient does not control this room's DID, so it cannot publish in the room's name."
    retryable: false
  - code: rooms/owner/anchor:hostUnreachable
    meaning: "The named host could not be resolved, advertises no transport this recipient speaks, or did not answer — so there is no head to anchor."
    retryable: true
  - code: rooms/owner/anchor:hostRefused
    meaning: "The host answered and declined to serve the room's head. Its own code and reason are carried in `details`."
    retryable: false
  - code: rooms/owner/anchor:notWitnessed
    meaning: "The room's DID is configured with no witnesses, so an entry would be the controller's own word. `details.roomId` names it; the repair is a witness configuration, not a retry."
    retryable: false
related:
  - rooms/keys/browse
  - rooms/epoch/mint
  - rooms/owner/register
---

## Abstract

A room's **owner** asks their key holder to write the room's current state into the room's own witnessed log: the MLS **epoch authenticator**, the **version watermark**, and — where the host offers one — the **data commitment** and the count that goes with it.

Everything else in this family produces values a host asserts. An anchor is the one statement a host does not make, cannot forge, and cannot show two members two versions of.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Why an anchor is worth anything

Three attacks die together, and none of them dies without it.

- **A rolled-back room.** A host that serves an older state of a room — fewer records, an earlier epoch — is indistinguishable from a room that simply has not moved, because every value it serves is its own. An anchor pins where the room *was*, witnessed, so going backwards past it is visible.
- **A forked group.** The epoch authenticator is derived independently by every member and by no host. A member whose own authenticator differs from the anchored one is in a different group than the room says it has — which is what a host splitting a room in two looks like from inside.
- **Equivocation about contents.** `DataCommitment` becomes evidence only when compared against a copy the host did not choose. An anchored root is that copy for **every member at once**, and it is the only one needing neither a gossip channel rooms deliberately lack nor durable state in a member's agent.

**Witnessed is the whole of it.** Witnesses co-sign a `did:webvh` log entry, so an anchor that rides one is singular. An anchor that did not would be a value the host could equally have made up.

## The owner holds one of the three values it publishes

This is the part that decides the shape of the task, and it was got wrong in an earlier design note before it was got right.

`epochAuthenticator` the recipient has: it holds the room's group state. **`headVersion` and `dataCommitment` it does not.** [`rooms/epoch/mint`](../../../epoch/mint/0.1/spec.md) answers `{roomId, epoch}` — no watermark, and no reason there should be one, because minting acts on the *epoch* while the watermark and the commitment are facts about the room's **records**, which live at the host.

So an anchor is assembled from a read. The recipient **MUST** read the room's head from `host`, presenting its own credentials as any member does, and **MUST** take `headVersion`, `dataCommitment` and `recordCount` from **one** response — a root and a version from two reads can straddle a write, and the pair is then individually correct and jointly false.

### The owner is anchoring a value the host gave it, and that is sound

It looks circular and it is not. The owner is **not vouching for the root** — it did not compute the tree and cannot. What the anchor does is make the root **singular and witnessed**: a host that has claimed root `R` at version `V` in a witnessed log cannot claim `R'` at `V` to anybody else.

That is exactly Certificate Transparency's arrangement, where the log operator's own signed tree head is what gets published and gossip is what makes equivocation fatal.

A recipient **SHOULD** reconcile before publishing and **MUST** report the outcome in `reconciled`. It **MUST NOT** refuse to anchor on a failed reconciliation: an owner that withheld an anchor from a suspect room would leave it with **no witnessed statement at all**, which is the position a misbehaving host benefits from. Publish what the host claimed, and record that it had already been caught claiming something inconsistent.

## What publishing costs

`vta/webvh/dids/update/1.0` already exists, so nothing new is needed to write an anchor. What its own description says, and what a room operator has to plan around:

> Supplying this **ROTATES the DID's update key** and refreshes its pre-rotation commitments, as a parallel consequence of the change.

So each anchor is a witnessed update **and** a rotation of the room DID's update key, with witness co-signing and a fresh pre-rotation commitment each time. **Cadence is therefore an operational decision rather than only a freshness one**, and a room anchoring on every write would rotate its own DID's update key on every write.

It also brushes succession: a transfer landing between an anchor and its successor is aiming at a key that has moved. Not a new problem — every `did:webvh` update has it — but worth knowing before scheduling anchors densely.

## A room with no witnesses cannot anchor

A recipient **MUST** refuse with `notWitnessed` where the room's DID is configured with no witness set. An entry nobody co-signed is the controller's own word, and publishing one would produce something that *looks* like an anchor and carries none of the property the anchor exists for — which is worse than the absence, because a member checking it would believe they had checked something.

This also constrains a configuration that is otherwise legal: a room whose DID is **host-controlled** cannot anchor honestly, because the party being checked would be writing the check.

## Definitions

**`roomId`** — the room to anchor.

**`host`** — where to read the head from, as a DID. Named by the caller because nothing maps a room to its host, and a room may have more than one.

**`signingKeyId`** — the room's signing key, named rather than looked up: nothing maps a DID to its key, and a wrong one fails against the room's own document at first use.

**`versionId`** (response) — the log entry the anchor rode. Naming it is what lets anyone fetch that entry and check the witnesses' signature; an anchor whose entry could not be named would be unverifiable in exactly the way this task exists to prevent.

**`reconciled`** (response) — see above. `false` is a finding, not a failure.

## Request

An **Owner** sends this to their **KeyHolder**.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/owner/anchor/0.1#request",
  "issuer": "did:example:owner",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "host": "did:webvh:example.com:northwind-community",
    "signingKeyId": "room-northwind-signing"
  }
}
```

## Response

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/owner/anchor/0.1#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:owner",
  "issuedAt": "2026-01-01T00:00:05Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "anchored": {
      "epoch": 7,
      "epochAuthenticator": "zQmZ4tDuvesekSs4qM5ZBKpXiZGun7S2CYtEZRB3DYXkjGx",
      "headVersion": 412,
      "dataCommitment": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
      "recordCount": 118
    },
    "versionId": "42-QmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n",
    "reconciled": true
  }
}
```

## Security & Privacy

### Data carried

The request carries three identifiers. What is *published* is the interesting half, and it is **public by design**: an anchor is written to be read by anyone, which is what makes it singular.

An anchor discloses that a room exists, roughly how much it holds, and how often it moves. On a `private` room that is a real disclosure and it is the one the tier can afford — it says nothing about **who** is in the room or **what** any record says. A room for which even the shape is sensitive should anchor rarely or not at all, and should decide that knowingly.

A producer **MUST NOT** put member identifiers or record content in `ext`.

### Correlation

The anchor is a public timeline of a room's activity. Anchoring on every write publishes a write log; anchoring per renewal publishes a membership-change log, at whatever granularity the cadence sets. Neither names anyone, and both are more than nothing.

To the host, this is a member reading the room's head — indistinguishable from any other read, which is worth keeping true: a host that could tell an anchor was imminent could serve a flattering answer to that one read.

### Retention

Permanent, by construction. A `did:webvh` log is append-only and witnessed, so an anchor can be **superseded and not withdrawn** — which is the property, not a side effect. A statement that could be withdrawn would be one a host could wait out.

### Consent/purpose

The room's own state is published so that members can check the room they are in against the room its owner says it is. It confers nothing: an anchor grants no authority and admits nobody.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required — though an implementation whose surface makes anchoring an operator action **SHOULD** show what will be published, since it cannot be taken back.
