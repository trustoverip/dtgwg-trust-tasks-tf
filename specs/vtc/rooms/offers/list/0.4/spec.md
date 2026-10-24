---
slug: vtc/rooms/offers/list
version: "0.4"
title: "VTC Rooms — Offers — List"
summary: "A member lists the admissions to community-owned data rooms the community has offered them — each room's label and what it is for — and the renewals of admissions they hold, with the credentials to store in their own key holder."
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
    The offers are the signer's own, and they carry credentials naming the signer. The community must bind the read to the member it answers for.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A request that returns credentials is worth replaying; `issuedAt` bounds how old a request the community will answer.
sideEffects:
  level: none
  rationale: >-
    Reads the caller's own offers. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: true
  rationale: >-
    The caller's own offers and renewals: room identifiers, labels, descriptions, tiers, actions, expiries and the credentials issued to the caller. Nobody else's.
retention:
  class: transient
  rationale: >-
    A read; the community keeps nothing.
errorCodes:
  - code: vtc/rooms/offers/list:notAMember
    meaning: "The caller is not an active member of this community."
    retryable: false
related:
  - vtc/rooms/offers/accept
  - vtc/rooms/community/update
  - vtc/rooms/community/members/add
  - rooms/keys/key-package
  - vtc/rooms/memberships/list
  - vtc/groups/bindings/add
  - rooms/keys/act
---

## Abstract

**VTC Rooms — Offers — List** is how a member collects what a room the community owns has issued them. It returns:

- **offers** — the caller's open offers of admission ([`vtc/rooms/community/members/add`](../../../../../vtc/rooms/community/members/add/0.1/spec.md), or a group's room binding, [`vtc/groups/bindings/add`](../../../../../vtc/groups/bindings/add/0.1/spec.md)), and accepted ones while a repeated accept can still be answered, each with the three credentials the room issued them;
- **renewals** — fresh membership and authority credentials for an admission the caller already holds, issued before the ones they hold expire.

The member's surface — the community's member portal, or their wallet — stores the credentials with the member's own key holder. For an offer it then, on a sealed tier, asks the key holder for a KeyPackage against the invitation ([`rooms/keys/key-package`](../../../../../rooms/keys/key-package/0.2/spec.md)) and accepts ([`vtc/rooms/offers/accept`](../../../../../vtc/rooms/offers/accept/0.1/spec.md)). A renewal needs nothing more: the member is already in the room, and storing its credentials in place of the ones they renew is the whole of taking it up.

A community renews an admission it made through a group's room binding for as long as the member stays in the group, as that binding requires ([`vtc/groups/bindings/add`](../../../../../vtc/groups/bindings/add/0.1/spec.md)). It **MAY** renew a direct admission the same way.

An offer leaves this list once a repeated accept can no longer be answered, and a renewal once the credentials it replaces expire. Where the member stands in each room the community owns, for as long as the community keeps the record, is [`vtc/rooms/memberships/list`](../../../../../vtc/rooms/memberships/list/0.1/spec.md), which every surface reads so that each shows the member the same rooms.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.3

- `renewals[].renews` → **`renewsAt`** (type unchanged: an RFC 3339 date-time, still REQUIRED).

The rename follows the registry's naming consistency: an instant that records an event is named `<event>At`. The response's `Renewal` now resolves to [`vtc/_shared/0.4/community-room.schema.json`](../../../../../vtc/_shared/0.4/community-room.schema.json), and `items[]`'s `OfferV2` moves to the same file, where it is unchanged; nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.3` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### Community

A conforming **community** (`recipient`):

1. **MUST** answer only the offers and renewals addressed to the proven signer — never another member's, whatever the request carries.
2. **MUST** refuse a signer who is not an active member of the community with `notAMember`.
3. **MUST NOT** list an offer that has been withdrawn.
4. **MUST** fill `label` and `description` from the room's current details, not from a copy taken when the offer or renewal was made, and **MUST NOT** include the room's tags or anything else of its administrators' tracking.
5. **MUST** issue a renewal through the room's key holder before recording it, granting exactly the actions of the admission it renews — never more — and **MUST NOT** issue an invitation with it.
6. **MUST** list a renewal, on the page answered without a `cursor`, from when it is issued until the credentials it renews expire; **MUST** list only the latest renewal of an admission; and **MUST** stop listing it once the member is removed from the room or the renewal is superseded by a later one.
7. **MUST** record the renewed `validUntil` as the admission's own once it lists the renewal, so that [`vtc/rooms/memberships/list`](../../../../../vtc/rooms/memberships/list/0.1/spec.md) and [`vtc/rooms/community/members/list`](../../../../../vtc/rooms/community/members/list/0.1/spec.md) show it.

### Surface

A conforming **surface** (the member's own client):

1. **MUST** store each renewal's credentials with the member's key holder **in place of** the membership and authority credentials it renews for that room, and **MUST NOT** store them beside those: a key holder that holds two authorities from one room for one member cannot choose between them.
2. **MUST** treat storing a renewal it has already stored as success: every surface the member uses may see the same renewal, and whichever stores it first takes it up for all of them.
3. **MUST NOT** call [`vtc/rooms/offers/accept`](../../../../../vtc/rooms/offers/accept/0.1/spec.md) for a renewal.

## Authorization

The entitlement is **being the member the offers are addressed to**, proven by the document's proof and checked against the community's own membership record at execution time. No administrative standing is involved, and none is accepted in its place: an administrator reading another member's offers would be reading credentials that name somebody else. A community answering members separately from administrators answers this on its member audience.

**Signing as the member.** The DID an offer is addressed to is often one whose keys are held by the member's key holder, so the surface the member acts from has this request completed and signed as that DID with [`rooms/keys/act`](../../../../../rooms/keys/act/0.6/spec.md) at a version whose table lists this version of this type, which carries no presentation for it. `0.6`'s table lists `0.1`, `0.2` and `0.3` of this type, not `0.4`.

## Request

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/offers/list/0.4#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-14T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {}
}
```

## Response

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/rooms/offers/list/0.4#response",
  "issuer": "did:example:community",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-14T10:00:01Z",
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
        "offeredAt": "2026-10-14T10:00:00Z",
        "state": "open",
        "credentials": {
          "invitation": "eyJhbGciOi…",
          "membership": "eyJhbGciOi…",
          "authority": "eyJhbGciOi…"
        }
      }
    ],
    "renewals": [
      {
        "roomId": "did:webvh:example.com:rooms:wg-identity",
        "host": "did:example:community",
        "label": "Identity working group",
        "visibility": "attributed",
        "actions": [
          "read",
          "write"
        ],
        "validUntil": "2027-11-10T00:00:00Z",
        "renewsAt": "2026-11-10T00:00:00Z",
        "issuedAt": "2026-10-14T09:00:00Z",
        "credentials": {
          "membership": "eyJhbGciOi…",
          "authority": "eyJhbGciOi…"
        }
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The caller's own offers and renewals — each room's label and description as its administrators wrote them — and the credentials issued to them. None is a secret: each names the caller, and a presentation of it must be signed by the caller.

### Correlation

The community maintainer's identifier scope is public: the community is addressed by its public DID because it is the party administrators, members and the rooms it owns already know it by: it is the room's owner of record and the issuer members' admissions trace back to, and a pairwise identifier per caller would make a room's owner differ by who asks. The answer reveals to the caller which of the community's rooms they have been offered, and nothing about anyone else's.

### Retention

None; a read.

### Consent/purpose

The member collecting what was offered to them, and keeping what they hold.
