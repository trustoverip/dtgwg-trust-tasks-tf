---
slug: vtc/rooms/offers/accept
version: "0.1"
title: "VTC Rooms — Offers — Accept"
summary: "A member accepts an admission to a community-owned data room; on a sealed tier the community's VTA adds their KeyPackage to the room's group and the member receives the Welcome for their own key holder."
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
    Admits the signer to a room's group with a KeyPackage only they can open the Welcome to. The community must bind the acceptance to the member the offer names.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed acceptance must be placeable in time.
sideEffects:
  level: mutating
  rationale: >-
    On a sealed tier, has the community's VTA add the member to the room's group and advance its epoch; records the admission.
subjectPath: /roomId
exposure:
  discloses: secret
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The request carries a public KeyPackage. The response carries a Welcome holding the group's secrets, sealed to that KeyPackage: secret to the caller, and opening nothing for anyone else.
retention:
  class: durable
  rationale: >-
    The admission is the owner's record. The Welcome is kept only as long as a repeated accept can still be answered.
errorCodes:
  - code: vtc/rooms/offers/accept:notFound
    meaning: "No open or recently accepted offer to this room is addressed to the caller."
    retryable: false
  - code: vtc/rooms/offers/accept:notAMember
    meaning: "The caller is not an active member of this community."
    retryable: false
  - code: vtc/rooms/offers/accept:offerExpired
    meaning: "The offer's `validUntil` has passed."
    retryable: false
  - code: vtc/rooms/offers/accept:keyPackageRequired
    meaning: "The room is `attributed` or `private`, and the request carries no KeyPackage."
    retryable: false
  - code: vtc/rooms/offers/accept:keyPackageNotExpected
    meaning: "The room is `open`, which has no group, and the request carries a KeyPackage."
    retryable: false
  - code: vtc/rooms/offers/accept:keyPackageInvalid
    meaning: "The KeyPackage did not validate, or its leaf does not identify the caller."
    retryable: false
  - code: vtc/rooms/offers/accept:alreadyAccepted
    meaning: "The offer was accepted with a different KeyPackage, or so long ago that its Welcome is no longer kept. Ask the community to remove and re-admit."
    retryable: false
  - code: vtc/rooms/offers/accept:admissionPending
    meaning: "The community's VTA did not answer, or the room's epoch advance is not yet known. Retry the identical request: the community settles the admission before answering again."
    retryable: true
  - code: vtc/rooms/offers/accept:keyHolderRefused
    meaning: "The community's VTA refused the admission. Its own code and message are carried in `details`."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        code:
          type: string
          maxLength: 128
        message:
          type: string
          maxLength: 500
related:
  - vtc/rooms/offers/list
  - rooms/keys/key-package
  - rooms/keys/welcome
  - rooms/owner/group/add
---

## Abstract

**VTC Rooms — Offers — Accept** completes an admission the community offered ([`vtc/rooms/community/members/add`](../../../../../vtc/rooms/community/members/add/0.1/spec.md)).

On an `open` room there is no group, and accepting records that the member took up the offer. On an `attributed` or `private` room the member sends a KeyPackage their own key holder minted against the offer's invitation ([`rooms/keys/key-package`](../../../../../rooms/keys/key-package/0.1/spec.md)). The community has its VTA add it to the room's group ([`rooms/owner/group/add`](../../../../../rooms/owner/group/add/0.2/spec.md), with the community itself as host), and answers with the **Welcome**. The member's surface hands that to their own key holder ([`rooms/keys/welcome`](../../../../../rooms/keys/welcome/0.1/spec.md)), which checks the invitation again and joins.

The Welcome travels only between the community's VTA, the community and the member, over the member's own authenticated exchange with the community, and opens nothing for any of them but the member.

## Repeating an accept

A member's surface may lose the answer. So the community keeps the Welcome it received for an offer, for as long as it chooses to answer repeats, and:

- a repeated accept carrying the **same** KeyPackage is answered with the **same** Welcome, epoch and tree, changing nothing;
- a repeated accept carrying a **different** KeyPackage is `alreadyAccepted`: the group already holds the first, and a second leaf for the same member is not what was offered;
- once the Welcome is no longer kept, a repeat is `alreadyAccepted`, and the member asks to be removed and re-admitted.

When the community cannot tell whether its VTA added the member — no answer, or the room's epoch advance not yet confirmed — it answers `admissionPending` and records nothing as admitted. Retrying the identical request is safe: the community repeats its own request to its VTA, unchanged, and the VTA settles a pending admission against the room's epoch before acting again ([`rooms/owner/group/add`](../../../../../rooms/owner/group/add/0.2/spec.md) item 6), and answers a request that repeats the admission it made — the same member, a KeyPackage with the same `KeyPackageRef` — with that admission's Welcome rather than refusing it (item 8). That holds whether the VTA's answer was lost before or after it applied the admission. The community answers from what its VTA returns. Only when the VTA no longer keeps that Welcome does it refuse `alreadyMember`, and the community then answers `alreadyAccepted`.

A community's VTA therefore serves [`rooms/owner/group/add`](../../../../../rooms/owner/group/add/0.2/spec.md) at `0.2` or later: under `0.1` the repeat is refused and the Welcome is lost.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** act only on an offer addressed to the proven signer, and **MUST** refuse a signer who is not an active member with `notAMember`.
2. **MUST** require a KeyPackage on a sealed tier and refuse one on an `open` room.
3. **MUST** record the admission only after its VTA has answered that the member is in the group, on a sealed tier.
4. **MUST** answer a repeat as described above, and **MUST NOT** add a second leaf for the same member.
5. **MUST NOT** send the Welcome to anyone but the caller, and **MUST NOT** log or audit it or the KeyPackage.

## Authorization

The entitlement is **being the member the offer is addressed to**, proven by the document's proof and checked against the community's own membership record at execution time. No administrative standing is involved. The community's VTA then applies its own gates to the admission it is asked to make, as for any owner.

**Signing as the member.** The DID an offer is addressed to is often one whose keys are held by the member's key holder — a persona their agent holds for them in one community — so the surface the member acts from cannot sign this request itself. It builds the request and has its key holder complete and sign it as that DID with [`rooms/keys/act`](../../../../../rooms/keys/act/0.3/spec.md) at `0.3` or later, which carries no presentation for this type and completes an acceptance only with a KeyPackage it minted itself for the member and the room. The community sees the member's signature and nothing of the surface or the key holder.

## Request

### Accepting a private room

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/offers/accept/0.1#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-14T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:board",
    "keyPackage": "AAEAAgADAAQ"
  }
}
```

## Response

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/rooms/offers/accept/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-14T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:board",
    "state": "admitted",
    "epoch": 2,
    "welcome": "AAEAAgADAAQ",
    "ratchetTree": "AAEAAgADAAQ"
  }
}
```

## Security & Privacy

### Data carried

A public KeyPackage in; a Welcome sealed to it out.

### Correlation

The community maintainer's identifier scope is public: the community is addressed by its public DID because it is the party administrators, members and the rooms it owns already know it by: it is the room's owner of record and the issuer members' admissions trace back to, and a pairwise identifier per caller would make a room's owner differ by who asks. The room's host — this community — sees an epoch advance and a commit it cannot read. Its audit records that a member accepted an offer to a room, and never pairs them with anything the room holds.

### Retention

The admission is durable, as the owner's record. The Welcome is kept only while a repeated accept can be answered, and is secret only to the member it is sealed to.

### Consent/purpose

Joining is two-party, and this is the member's half: the community offered, and the member accepts.
