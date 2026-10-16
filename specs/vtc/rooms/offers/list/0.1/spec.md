---
slug: vtc/rooms/offers/list
version: "0.1"
title: "VTC Rooms — Offers — List"
summary: "A member lists the admissions to community-owned data rooms the community has offered them, with the credentials to store in their own key holder."
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
    The caller's own offers: room identifiers, labels, tiers, actions, expiries and the credentials issued to the caller. Nobody else's.
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
  - vtc/rooms/community/members/add
  - rooms/keys/key-package
---

## Abstract

**VTC Rooms — Offers — List** is how a member collects an admission the community offered them ([`vtc/rooms/community/members/add`](../../../../../vtc/rooms/community/members/add/0.1/spec.md)). It returns the caller's open offers, and accepted ones while a repeated accept can still be answered, each with the three credentials the room issued them.

The member's surface — the community's member portal, or their wallet — stores the credentials with the member's own key holder, and on a sealed tier asks it for a KeyPackage against the invitation ([`rooms/keys/key-package`](../../../../../rooms/keys/key-package/0.1/spec.md)), before accepting ([`vtc/rooms/offers/accept`](../../../../../vtc/rooms/offers/accept/0.1/spec.md)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** answer only the offers addressed to the proven signer — never another member's, whatever the request carries.
2. **MUST** refuse a signer who is not an active member of the community with `notAMember`.
3. **MUST NOT** list an offer that has been withdrawn.

## Authorization

The entitlement is **being the member the offers are addressed to**, proven by the document's proof and checked against the community's own membership record at execution time. No administrative standing is involved, and none is accepted in its place: an administrator reading another member's offers would be reading credentials that name somebody else. A community answering members separately from administrators answers this on its member audience.

**Signing as the member.** The DID an offer is addressed to is often one whose keys are held by the member's key holder, so the surface the member acts from has this request completed and signed as that DID with [`rooms/keys/act`](../../../../../rooms/keys/act/0.3/spec.md) at `0.3` or later, which carries no presentation for this type.

## Request

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/offers/list/0.1#request",
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
  "type": "https://trusttasks.org/spec/vtc/rooms/offers/list/0.1#response",
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
    ]
  }
}
```

## Security & Privacy

### Data carried

The caller's own offers and the credentials issued to them. None is a secret: each names the caller, and a presentation of it must be signed by the caller.

### Correlation

The community maintainer's identifier scope is public: the community is addressed by its public DID because it is the party administrators, members and the rooms it owns already know it by: it is the room's owner of record and the issuer members' admissions trace back to, and a pairwise identifier per caller would make a room's owner differ by who asks. The answer reveals to the caller which of the community's rooms they have been offered, and nothing about anyone else's.

### Retention

None; a read.

### Consent/purpose

The member collecting what was offered to them.
