---
slug: rooms/epoch/chain
version: "0.1"
title: "Rooms Epoch Chain"
summary: "A member fetches a room's epoch key chain from its host, so records sealed before they joined can still be opened."
status: draft
targetFrameworkVersion: "0.5.0"
category: ai-agents
# keywords and authors are OPTIONAL, and omitted here on purpose: the build derives
# keywords from the slug segments + category, and authors from CODEOWNERS (falling
# back to this folder's git history). Declare them only where the derivation would
# be wrong — a term a searcher would use that appears nowhere in the slug, or an
# editor who is not this slug's CODEOWNER.
#   keywords: [rooms, epoch, chain, a-term-a-searcher-would-use]
#   authors:
#     - Your Name (https://github.com/your-handle)
parties:
  - role: Member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The request carries an authority presentation, and a presentation names what may be done rather than who is doing it. Without a proof binding the request to the party that signed it, an observed presentation is a bearer token and whoever captures one inherits the room's history.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A read presentation is time-bounded, and accepting a stale one would hand a room's whole history to a party whose grant has since lapsed — the one read where that mistake is not recoverable by revoking anything, because the keys have already travelled."
sideEffects:
  level: none
  rationale: >-
    A read. The chain is written by `rooms/epoch/mint`, one rung at a time, and nothing here changes what a host holds.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    The response is ciphertext under keys no host holds, so it discloses the number of epochs a room has had and nothing else — and the room's epoch number already disclosed that. What the request discloses is the ordinary cost of any room read: that a member acted, and on a disclosing tier which one.
  # ingests: metadata     # framework 0.5.0, OPTIONAL: what the REQUEST carries INTO the recipient
                          # (none | metadata | personal | secret). Note the enum differs from
                          # discloses — `personal` exists here because personal-but-not-secret
                          # data is exactly what changes a recipient's minimisation obligations.
                          # `personal` or `secret` makes exposure.rationale REQUIRED.
retention:
  class: durable
  rationale: >-
    The requester keeps the links: they are what makes the room's history openable, and a member who discards them has to ask again on every restart. The host already holds them — it served them from storage.
errorCodes:
  - code: rooms/epoch/chain:notFound
    meaning: "No room with this identifier is served here."
    retryable: false
  - code: rooms/epoch/chain:forbidden
    meaning: "The presentation does not confer `read` at this room's scope, does not verify, or was not made to the party that signed the request."
    retryable: false
related:
  - rooms/epoch/mint
  - rooms/keys/open
  - rooms/keys/welcome
---

## Abstract

A member asks a room's host for the room's **epoch key chain**, so that records sealed before they joined — or before the last membership change — can still be opened.

Records in a sealed room are encrypted under a storage key derived per epoch, and a group key schedule deliberately offers no way to derive an earlier epoch's key from a later one. That property is what makes removing a member mean something. It also means that, left alone, the first membership change makes every record already in the room unopenable by everyone, the writer included. A room is a library rather than a message stream: what was written is supposed to stay readable to whoever is in the room.

The chain is that one-way street run deliberately the other way. Each advance seals the outgoing epoch's key under the incoming one (`rooms/epoch/mint`), and this task hands those rungs to a member so they can walk back. Backwards only — a member holding an earlier key still derives nothing later, so removal remains forward-only.

It is a Trust Task rather than an API call because what authorises it is a credential the **room** issued, verified against the room's own identifier, and never an access list the host maintains. That is what lets a room move hosts without reissuing anything, and it is the same reason every other `rooms/*` task is one.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Definitions

**Epoch key chain.** The set of [`EpochLink`](../../../_shared/0.1/room.schema.json) rungs for a room. Rung `n` holds epoch `n − 1`'s storage key sealed under epoch `n`'s, so a holder of epoch `n`'s key can recover every retained key below it by walking down. A room's first epoch has no rung, because it has no predecessor.

**`roomId`** — the room whose chain is wanted. Chosen by the requester; a host serves only rooms it holds.

**`presentation`** — the requester's authority, which **MUST** confer `read` at this room's scope. Reading the room and reading the parts written earlier are the same act, so they take the same grant.

**`fromEpoch`** — optional. Return only rungs at or below this epoch. A member who already holds the top of the chain uses it to ask for the rest; absent, a host serves from the room's current epoch downwards.

**`limit`** — optional. The most rungs to return, highest epoch first. A host **MAY** return fewer.

**`links`** (response) — the rungs, highest epoch first, contiguous within the range returned.

### Walking, and knowing when to stop

A requester continues by re-asking with `fromEpoch` one below the lowest rung it received. It has the whole retained chain when a response returns a rung for epoch 2, or returns fewer rungs than it asked for.

A response that stops higher than epoch 2 means the chain does not go further — either because the room's history below that point was deliberately severed, or because the host is not serving it. **A requester cannot tell those apart, and a specification should not pretend otherwise.** What it can do is establish what it can actually open, which is done by walking the rungs it holds and not by trusting a claim about them. A host that withholds a rung it holds is an availability failure, which is the one thing a host is trusted for and the one failure no cryptography here defends against.

## Request

A **Member** sends this to the room's **Host**. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member who has just joined asks for everything below their epoch

A Welcome carries the current epoch's key and nothing under it, so a member who joined at epoch 5 can read the room from epoch 5 forward and sees nothing older. This is the request that fixes that.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/epoch/chain/0.1#request",
  "issuer": "did:example:member",
  "recipient": "did:example:host",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "presentation": {
      "membership": "eyJhbGciOiJFZERTQSJ9.member-vmc",
      "authority": ["eyJhbGciOiJFZERTQSJ9.read-vac"]
    },
    "fromEpoch": 5
  }
}
```

## Response

The **Host** responds, using the sub-schema reachable via `$anchor: "response"`. `links` carries the rungs, highest `epoch` first and contiguous within the range returned: a host **MUST NOT** omit a rung it holds while returning a lower one, because a member cannot walk past a gap and would read the omission as history that had been severed.

Failures use a `trust-task-error` document rather than a `#response`, per [SPEC §6.4](/SPEC.md#64-error-documents).

### The rungs that take a member from epoch 5 back to the room's beginning

Four rungs: 5 unwraps 4, and so on down to 2, which unwraps the room's first epoch. Every one is ciphertext the host cannot read.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/epoch/chain/0.1#response",
  "issuer": "did:example:host",
  "recipient": "did:example:member",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:northwind",
    "links": [
      { "epoch": 5, "wrapped": "9jK2_QhV1sVvR0m5xAqZ7A", "nonce": "b0Zt8Qm2Yq1sVvR0" },
      { "epoch": 4, "wrapped": "Lp3wXc9TgYw2mQnR4vBk8Q", "nonce": "c1Au9Rn3Zr2tWwS1" },
      { "epoch": 3, "wrapped": "Tf6yNb2VhZx5pRoS7wCl9Q", "nonce": "d2Bv0So4As3uXxT2" },
      { "epoch": 2, "wrapped": "Wq8zMd4XjBy7rTqU9yEn1Q", "nonce": "e3Cw1Tp5Bt4vYyU3" }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries a room identifier, an authority presentation, and two integers. The response carries wrapped key material.

**The response is the sensitive part, and it is sensitive only to someone who already holds a key.** Each rung is one epoch's storage key sealed under the next epoch's. A party holding no epoch key learns nothing from the whole chain but how many epochs the room has had — which the room's epoch number told them already. This is why a host may hold the chain at all, and it is the property that makes this task a host-served read rather than something the room's owner must be online to answer.

A producer **MUST NOT** put room content, member identifiers, or anything descriptive into `ext`. The room's material is sealed precisely so that this exchange does not disclose it, and a free-form member is the obvious place to undo that by accident.

**What the chain gives up, and where that is chosen.** A chain means a member's current key reaches every retained epoch, so compromising one member's current key exposes the room's retained history rather than only what came after. That is the cost of a library, and it is a real cost. It is not chosen here: it is chosen when the room is created, and this task only moves rungs that a room already decided to produce. A room that made the other choice has no chain, and this task returns an empty `links` array for it.

### Correlation

On `open` and `attributed` rooms the presentation discloses the acting member, so a host can join these requests with that member's reads and writes. That is the tier's stated bargain and this task adds nothing to it.

On a `private` room the presentation is unlinkable, and this request **MUST NOT** become the exception. Note the shape of the risk: a member fetches the chain roughly once — on joining, or after restoring their state — so a chain request is a strong signal that *someone just joined or came back*, correlatable with a membership change the host also witnessed. A host learns that a member acted, which it already learns from every read; what it must not learn is which one. Requesters **SHOULD** avoid making the request adjacent to anything that identifies them, and **SHOULD** fetch the chain over the same transport as their ordinary reads rather than a distinguishable one.

`fromEpoch` discloses roughly how much of the room the requester already holds, which is a weak proxy for how long they have been a member. A requester that would rather not say **MAY** omit it and discard the rungs it already had.

### Retention

A requester keeps the rungs: they are what make the room's history openable, and discarding them means re-fetching on every restart. They are exactly as sensitive as the keys they wrap and belong wherever that member's other room key material lives — which, for an agent-facing deployment, is not the agent.

A host keeps the chain for the life of the room. Dropping a rung is not housekeeping: it is **cryptographic deletion** of everything sealed below it, irreversible for anyone who has not already walked past that point, and it is only correct as a deliberate act by a party authorised to make it.

### Consent/purpose

The chain is fetched so that a member can read the room they are a member of. Reusing it for anything else means reusing the room's content, which the room's own credentials already govern — this task confers no authority that `read` did not already confer, and a host that serves it to a party without `read` has not leaked a key, but has disclosed how many epochs a room it cannot read has had.

This specification describes purpose only. Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does **not** declare that consent, approval or a step-up is required.
