---
slug: rooms/keys/list
version: "0.1"
title: "Rooms Keys — List"
summary: "An agent asks which rooms its principal's key holder can open, and how far back each one reads."
status: draft
targetFrameworkVersion: "0.5.0"
category: ai-agents
# keywords and authors are OPTIONAL, and omitted here on purpose: the build derives
# keywords from the slug segments + category, and authors from CODEOWNERS (falling
# back to this folder's git history). Declare them only where the derivation would
# be wrong — a term a searcher would use that appears nowhere in the slug, or an
# editor who is not this slug's CODEOWNER.
#   keywords: [rooms, keys, list, a-term-a-searcher-would-use]
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
  rationale: "The response names every room this key holder can open, which is its principal's room membership as far as key custody reveals it. A request whose origin depended on the transport would let a compromised channel enumerate it."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "The answer changes as commits arrive and chains are delivered, so an undated request cannot be placed against the state it described."
sideEffects:
  level: none
  rationale: >-
    A read of what this key holder already holds. Opens nothing and changes nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses which rooms the principal holds keys for, and how far back each reads. No record content and no key material — but the room identifiers are the principal's membership as key custody sees it, which is more than any single room operation reveals.
  # ingests: metadata     # framework 0.5.0, OPTIONAL: what the REQUEST carries INTO the recipient
                          # (none | metadata | personal | secret). Note the enum differs from
                          # discloses — `personal` exists here because personal-but-not-secret
                          # data is exactly what changes a recipient's minimisation obligations.
                          # `personal` or `secret` makes exposure.rationale REQUIRED.
retention:
  class: transient
  rationale: >-
    A snapshot of state the recipient already holds. Worth nothing once acted on, and stale the moment a commit arrives.
errorCodes: []
related:
  - rooms/keys/open
  - rooms/keys/chain
  - rooms/records/list
---

## Abstract

An agent asks its principal's **key holder** which rooms it can open, and how far back each one reads.

This is what a surface needs before it can show anything at all. Every other room task takes a `roomId` the caller already knows; nothing said where that identifier comes from, so a client could act in a room it had been told about out of band and could not present its principal with a list of their own rooms.

**It is key custody, not membership.** The two differ, and the difference is the useful part — see below.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Definitions

The request has no members beyond `ext`: the recipient answers for its own principal, and there is nothing to ask it about.

**`rooms[].roomId`** — the room's identifier.

**`rooms[].epoch`** — the epoch this key holder is at.

**`rooms[].earliestReadableEpoch`** — the earliest epoch it can derive a key for, walking the epoch key chain it holds.

### What the two epochs tell a surface, and why both are here

Neither number alone says what a member can read.

- `epoch` **behind the room's own** means a commit has not been delivered. Records written since will not open, and the repair is delivery, not anything the member does.
- `earliestReadableEpoch` **equal to `epoch`** means this key holder reads only from where it joined. The repair is [`rooms/keys/chain`](../../chain/0.1/spec.md).
- `earliestReadableEpoch` of **`1`** means the room's whole retained history is reachable.
- Anything **between** is a chain delivered in part, or one deliberately severed below that point — and a surface cannot tell those apart, because they are the same fact about reachability.

Presenting a room without them invites the failure the room family keeps trying to avoid: a member opens a room, sees less than they expect, and reads it as loss rather than as a delivery that has not happened.

### Membership is not custody

This lists rooms whose **group state** the recipient holds. A principal may hold a perfectly good membership credential for a room whose Welcome never arrived — that room is absent here, and correctly so: the key holder cannot open it.

The converse also holds and matters more. A key holder that has not been told about a removal still holds keys for a room its principal has been removed from. It cannot write there — the host checks credentials, and theirs no longer verify — and it can still open what it already has, which is exactly what removal has always meant. A consumer **MUST NOT** present this list as an authority to act, only as what can be opened.

## Request

An **Agent** sends this to its principal's **Oracle**. The payload carries nothing but an optional `ext`.

### The whole request

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/keys/list/0.1#request",
  "issuer": "did:example:agent",
  "recipient": "did:example:oracle",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {}
}
```

## Response

The **Oracle** answers with one entry per room, using the sub-schema reachable via `$anchor: "response"`.

Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §6.4](/SPEC.md#64-error-documents).

### Three rooms in three different states

The first reads its whole history; the second reads only from where its holder joined, and wants [`rooms/keys/chain`](../../chain/0.1/spec.md); the third is missing a commit, and wants delivery instead.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/keys/list/0.1#response",
  "issuer": "did:example:oracle",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "rooms": [
      { "roomId": "did:webvh:example.com:rooms:northwind", "epoch": 4, "earliestReadableEpoch": 1 },
      { "roomId": "did:webvh:example.com:rooms:pricing", "epoch": 7, "earliestReadableEpoch": 7 },
      { "roomId": "did:webvh:example.com:rooms:archive", "epoch": 2, "earliestReadableEpoch": 1 }
    ]
  }
}
```

## Security & Privacy

### Data carried

Room identifiers and two epoch numbers each. No record content, no key material.

**The identifiers are the sensitive part**, and they are more than any single room operation discloses: together they are the principal's room membership as key custody sees it. A recipient answers only for its own principal and **MUST NOT** offer a form of this that answers for anyone else.

A producer sends nothing but `ext`, and **MUST NOT** put a filter there. A filter would be a way to ask "is this principal in *that* room", which is a different question with a different answer and no authorization story.

### Correlation

The recipient learns nothing new — it is answering from its own storage.

The epochs are a weak history signal to anyone who sees several responses over time: a room whose `epoch` advances quickly has an active membership, and `earliestReadableEpoch` moving to `1` says a chain has just been delivered, which usually means its principal has just joined or restored. A surface that polls this on a timer publishes that rhythm to whatever it polls over; refreshing on use rather than on a clock says less.

### Retention

Transient on both sides. It is a snapshot of state the recipient already holds, worth nothing once acted on and stale the moment a commit arrives. A caller that caches it **SHOULD** treat a failure to open as the authority, not the cached list.

### Consent/purpose

The list is read so that a principal's own surfaces can show them their own rooms. It confers no authority: a room appearing here says the key holder can decrypt its records, never that its principal may still act in it — the host decides that, from credentials the room issued.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
