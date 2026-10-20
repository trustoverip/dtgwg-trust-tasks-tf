---
slug: vtc/rooms/memberships/list
version: "0.1"
title: "VTC Rooms — Memberships — List"
summary: "A member reads the community's own record of every room it has offered them — offered, admitted or removed — so that every surface they reach the community from shows the same rooms in the same state."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
parties:
  - role: member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The record is the signer's own and says which of the community's rooms they are in. The community must bind the read to the member it answers for.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A replayed request would let whoever captured it learn, later, which rooms the member is in; `issuedAt` bounds how old a request the community will answer.
sideEffects:
  level: none
  rationale: >-
    Reads the caller's own record. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: true
  rationale: >-
    The caller's own admissions: room identifiers, labels, descriptions, tiers, actions, expiries, states and when each changed. No credential, and nobody else's.
retention:
  class: transient
  rationale: >-
    A read; the community keeps nothing.
errorCodes:
  - code: vtc/rooms/memberships/list:notAMember
    meaning: "The caller is not an active member of this community."
    retryable: false
related:
  - vtc/rooms/offers/list
  - vtc/rooms/offers/accept
  - vtc/rooms/community/members/add
  - vtc/rooms/community/members/remove
  - rooms/keys/list
  - rooms/keys/key-package
  - rooms/keys/welcome
  - rooms/keys/act
---

## Abstract

**VTC Rooms — Memberships — List** returns the community's own record of the caller's admissions to the rooms it owns: every room it has offered them, and whether that offer is still `offered`, was accepted and is `admitted`, or ended `removed`.

A member reaches their community from more than one surface — its member portal through their wallet, a command-line or terminal client, a phone — and each one acts against the same community and the same key holder. What none of them can do is keep its own account of which rooms the member is in: a room accepted in one would be missing from the others, and an acceptance one began could be finished by no other. This task gives every surface the same answer, from the one party that records it, so that taking up a room in one surface is taken up in all of them.

It does not replace [`vtc/rooms/offers/list`](../../../offers/list/0.2/spec.md), which carries the credentials an open offer is collected with, and lists an accepted offer only while a repeat can be answered. This lists the record, for as long as the community keeps it, and carries no credentials.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### Community

A conforming **community** (`recipient`):

1. **MUST** answer only the record of the proven signer — never another member's, whatever the request carries.
2. **MUST** refuse a signer who is not an active member of the community with `notAMember`.
3. **MUST** list every room it owns for which it holds a record of an admission offered to the signer, in state `offered`, `admitted` or `removed`, and **MUST NOT** list an offer that was withdrawn before it was accepted.
4. **MUST** answer from the same record [`vtc/rooms/offers/accept`](../../../offers/accept/0.1/spec.md) writes, so that an acceptance it has recorded is `admitted` here as soon as it answers that acceptance, whichever surface sent it.
5. **MUST** include `repeatAcceptUntil` on an `attributed` or `private` room in state `admitted` exactly while it keeps the Welcome for that admission, and **MUST NOT** include it otherwise.
6. **MUST** fill `label` and `description` from the room's current details, and **MUST NOT** include the room's tags or anything else of its administrators' tracking.
7. **MUST** keep its order stable across the pages of one listing.

### Surface

A conforming **surface** (the member's own client, which builds the request and consumes the response — a member portal page through the member's wallet, a terminal client, or any other):

1. **MUST** treat this list, together with its key holder's [`rooms/keys/list`](../../../../../rooms/keys/list/0.2/spec.md), as the whole of what it shows of the member's rooms in this community, and **MUST NOT** keep a record of its own of which rooms the member has accepted. It **MAY** cache either list to draw a screen, and **MUST** read both again before it acts on what it shows.
2. **MUST** derive what it shows for each entry as *One record, every surface* sets out, and **MUST** act the same way for the same pair of answers.
3. **MUST**, before accepting an offer on an `attributed` or `private` room, ask its key holder for the room's KeyPackage with [`rooms/keys/key-package`](../../../../../rooms/keys/key-package/0.2/spec.md) at `0.2` or later, so that an acceptance another surface began is finished with the same KeyPackage and answered with the same Welcome.
4. **MUST** treat [`rooms/keys/welcome`](../../../../../rooms/keys/welcome/0.1/spec.md)'s `alreadyJoined` as success, for the member the invitation names: another surface finished the join first.
5. **MUST** treat [`vtc/rooms/offers/accept`](../../../offers/accept/0.1/spec.md)'s `alreadyAccepted` on a room in state `admitted` as an acceptance that can no longer be finished from here, and **MUST NOT** report it to the member as a failure of the room or as an acceptance that did not happen. Its repair is an administrator removing and re-admitting the member.

## One record, every surface

Two parties hold what a surface needs, and each holds a different half:

- **the community** holds the admission — whether the member was offered the room, accepted it, or was removed — and answers this task;
- **the member's key holder** holds the custody — whether it has the room's group state for the member, and so can open it — and answers [`rooms/keys/list`](../../../../../rooms/keys/list/0.2/spec.md).

Neither is the surface's, and every surface reads both, so every surface reaches the same result for the same room:

| This task's `state` | Tier | Key holder holds group state for the member | The surface shows | And offers |
|---|---|---|---|---|
| `offered` | any | — | an offer | to accept it: collect the credentials with [`vtc/rooms/offers/list`](../../../offers/list/0.2/spec.md), store them with the key holder, then accept |
| `admitted` | `open` | — (an `open` room has no group) | a room the member is in | to open it |
| `admitted` | any, with a renewal listed for it by [`vtc/rooms/offers/list`](../../../offers/list/0.3/spec.md) at `0.3` or later | — | a room the member is in | to keep it: store the renewal's credentials with the key holder in place of the ones it renews, before anything else the rows below offer |
| `admitted` | sealed | yes | a room the member is in | to open it |
| `admitted` | sealed | no, and `repeatAcceptUntil` present | a room whose acceptance is not finished | to finish it: Surface item 3, then a repeated accept, then the Welcome |
| `admitted` | sealed | no, and `repeatAcceptUntil` absent | a room the member was admitted to and cannot open from this key holder | nothing it can do itself: an administrator re-admits |
| `removed` | any | either | a room the member was removed from | to read what the key holder can already open, where it can; never to write |

*Finishing* is the same three steps as accepting, each of which answers a repeat with what it answered the first time: the key holder returns the KeyPackage it already minted for the room, the member and the invitation; the community answers an acceptance carrying that KeyPackage with the Welcome it kept; and the key holder answers a Welcome it has already processed with `alreadyJoined`. A surface finishing an acceptance another surface began, or one it began itself and lost the answer to, cannot tell the two cases apart and does not need to.

A surface with no channel to the member's key holder — a page that has not been given one — cannot read custody, and **SHOULD** show `admitted` rooms as rooms the member is in, without offering to open them.

### Why a record, and not each surface's own

A surface's own record of an acceptance is wrong twice: it is missing everything the member did in other surfaces, and what it does hold may since have been ended by an administrator. The community records the admission because it is the owner of the room; the key holder holds the group because it holds the member's keys. A surface holds neither, and a surface that kept its own copy would be the one place the three disagree.

## Authorization

The entitlement is **being the member the record is about**, proven by the document's proof and checked against the community's own membership record at execution time. No administrative standing is involved, and none is accepted in its place: administrators read who is in a room with [`vtc/rooms/community/members/list`](../../../community/members/list/0.1/spec.md). A community answering members separately from administrators answers this on its member audience.

**Signing as the member.** The DID the community addresses its offers to is often one whose keys are held by the member's key holder, so the surface the member acts from has this request completed and signed as that DID with [`rooms/keys/act`](../../../../../rooms/keys/act/0.5/spec.md) at `0.5` or later — the first version whose table lists this type — which carries no presentation for this type.

## Definitions

**`items[].state`** — `offered`, `admitted` or `removed`: `MemberState` in [`vtc/_shared/0.1/community-room.schema.json`](../../../../_shared/0.1/community-room.schema.json).

**`items[].repeatAcceptUntil`** — until when a repeated acceptance with the same KeyPackage is answered with its Welcome. See Community item 5.

The other members of an entry mean what they mean in an offer: `RoomMembership` in [`vtc/_shared/0.1/community-room.schema.json`](../../../../_shared/0.1/community-room.schema.json).

## Request

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/memberships/list/0.1#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-17T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {}
}
```

## Response

The member accepted the board pack in one surface, which has not yet handed the Welcome to their key holder; the community still keeps it. They have an open offer to the working group's room, and were removed from last year's audit room.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/rooms/memberships/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-17T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "items": [
      {
        "roomId": "did:webvh:example.com:rooms:board",
        "host": "did:example:community",
        "label": "Board pack",
        "description": "Papers for the next board meeting: agenda, minutes in draft, and the finance pack.",
        "visibility": "private",
        "actions": [
          "read",
          "write"
        ],
        "validUntil": "2027-10-01T00:00:00Z",
        "state": "admitted",
        "offeredAt": "2026-10-14T10:00:00Z",
        "admittedAt": "2026-10-17T09:58:00Z",
        "repeatAcceptUntil": "2026-10-24T09:58:00Z"
      },
      {
        "roomId": "did:webvh:example.com:rooms:standards-wg",
        "host": "did:example:community",
        "label": "Standards working group",
        "visibility": "attributed",
        "actions": [
          "read",
          "write",
          "curate"
        ],
        "validUntil": "2027-10-01T00:00:00Z",
        "state": "offered",
        "offeredAt": "2026-10-16T15:30:00Z"
      },
      {
        "roomId": "did:webvh:example.com:rooms:audit-2025",
        "host": "did:example:community",
        "label": "Audit 2025",
        "visibility": "open",
        "actions": [
          "read"
        ],
        "validUntil": "2026-12-31T00:00:00Z",
        "state": "removed",
        "offeredAt": "2025-11-02T09:00:00Z",
        "admittedAt": "2025-11-02T09:20:00Z",
        "removedAt": "2026-06-30T17:00:00Z"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The caller's own record: which of the community's rooms they were offered, accepted and were removed from, and when. No credential and no key material. A room's label and description are as its administrators wrote them.

### Correlation

The community maintainer's identifier scope is public, for the reason [`vtc/rooms/offers/list`](../../../offers/list/0.2/spec.md) gives. The answer reveals to the caller only their own admissions — nothing about who else was offered or admitted to any room. The community already holds this record as the rooms' owner; answering it adds nothing to what it knows, and it learns only that the member asked, which it learns of every surface that reads it.

### Retention

None; a read. How long the community keeps a `removed` record is its own retention of the admission, as the room's owner, and is unchanged by this task.

### Consent/purpose

The member reading their own standing in the community's rooms, so that every surface they use shows the same thing.
