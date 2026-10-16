---
slug: vtc/rooms/community/members/remove
version: "0.2"
title: "VTC Rooms — Community — Members — Remove"
summary: "An administrator removes a member from a data room the community owns: the room's host refuses the member's credentials from that moment, on a sealed tier the room advances to an epoch sealed to everyone else, and any open offer is withdrawn."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Ends a party's access to a room at its host at once, and to its future content. It must be attributable to its requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed removal could remove someone re-admitted since.
sideEffects:
  level: mutating
  rationale: >-
    Has the community's VTA withdraw the member's credentials at the room's host and, on a sealed tier, remove them from the room's group and advance its epoch; withdraws any offer; records the removal.
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request names a member; the response is their record, the revocation list in force, and on a sealed tier the new epoch.
retention:
  class: durable
  rationale: >-
    The removal is kept with the member's record as the owner's history of the room.
errorCodes:
  - code: vtc/rooms/community/members/remove:notFound
    meaning: "The community owns no room with this identifier."
    retryable: false
  - code: vtc/rooms/community/members/remove:notInRoom
    meaning: "`member` holds neither an admission to the room nor an open offer."
    retryable: false
  - code: vtc/rooms/community/members/remove:keyHolderUnavailable
    meaning: "The community's VTA, or the room's host behind it, did not answer. The outcome is unknown: a removal the VTA may have completed is settled by retrying, which it answers from its own record."
    retryable: true
  - code: vtc/rooms/community/members/remove:keyHolderRefused
    meaning: "The community's VTA refused the removal. Its own code and message are carried in `details`. Nothing changed."
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
  - vtc/rooms/community/members/add
  - rooms/owner/group/add
  - rooms/owner/group/remove
  - rooms/owner/revoke
  - rooms/revocations/put
  - rooms/epoch/mint
---

## Abstract

**VTC Rooms — Community — Members — Remove** ends a member's access to a room the community owns.

**What removal means, and does not.** On an `open` or `attributed` room, the community's VTA **withdraws** every credential the room gave the member: it signs the room's next revocation list and delivers it to the room's host ([`rooms/owner/revoke`](../../../../../../rooms/owner/revoke/0.1/spec.md), [`rooms/revocations/put`](../../../../../../rooms/revocations/put/0.1/spec.md)), which from that moment refuses their membership and authority credentials — and anything they delegated — on every task. On an `attributed` room it does so as the first step of removing them from the room's group ([`rooms/owner/group/remove`](../../../../../../rooms/owner/group/remove/0.2/spec.md) at `0.2`), which then advances the room to a new epoch sealed to everyone else, so they cannot open anything written from then on either. Nothing they already read or saved can be recalled, and nothing written before is re-sealed.

On a `private` room a member cannot be withdrawn one at a time: its host verifies membership in zero knowledge and does not know who the member is. The VTA removes them from the group ([`rooms/owner/group/remove`](../../../../../../rooms/owner/group/remove/0.1/spec.md) at `0.1`) and the room advances, but their credentials are honoured at the host until their `validUntil`; what they admit them to is ciphertext they cannot open from the new epoch on. A community that needs more on a private room resets every credential with [`rooms/owner/revoke`](../../../../../../rooms/owner/revoke/0.1/spec.md) and re-admits those who remain.

An open offer is withdrawn, and on an `open` or `attributed` room its credentials are withdrawn at the host as well: a member collects them with [`vtc/rooms/offers/list`](../../../../../../vtc/rooms/offers/list/0.1/spec.md) before accepting, so an offer may already have put usable credentials in their hands.

### What changed from 0.1

`0.1` withdrew nothing at the host: a removed member's credentials stayed valid there until they expired, and on an `open` room removal withdrew nothing at all. `0.2` withdraws them, at once, on `open` and `attributed` rooms, and answers with the revocation list in force. A breaking change to a `draft` specification, so a `MINOR` increment per [SPEC §5.2](/SPEC.md#52-compatibility-rules). Requires a VTA serving [`rooms/owner/revoke`](../../../../../../rooms/owner/revoke/0.1/spec.md) and [`rooms/owner/group/remove`](../../../../../../rooms/owner/group/remove/0.2/spec.md) `0.2`, and a host serving [`rooms/revocations/put`](../../../../../../rooms/revocations/put/0.1/spec.md) — which the community is, for the rooms it owns.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.manage` with `permissionDenied`, and **MUST** answer `notFound` for a room it does not own and `notInRoom` for a member holding neither an admission nor an open offer.
2. For an admitted member: on an `open` room **MUST** have its VTA withdraw them ([`rooms/owner/revoke`](../../../../../../rooms/owner/revoke/0.1/spec.md), one `subject` withdrawal naming `member`, delivered to every host serving the room); on an `attributed` room **MUST** have its VTA remove them with [`rooms/owner/group/remove`](../../../../../../rooms/owner/group/remove/0.2/spec.md) at `0.2`, which withdraws them first and then advances the room; on a `private` room **MUST** have its VTA remove them from the group with `0.1`. In every case it **MUST** record the removal only once the VTA has answered that it did — and on `open` and `attributed` rooms only once the room's host has accepted the withdrawal. A withdrawal the host did not accept is `keyHolderUnavailable` or `keyHolderRefused`, never a removal.
3. **MUST** withdraw an open offer, so it is never listed or accepted again, and on an `open` or `attributed` room **MUST** have its VTA withdraw the offer's credentials at the host as for an admitted member.
4. **MUST NOT** represent a removal, in its record of it or in its answer, as recalling what the member already read or re-sealing what was written before it, and **MUST NOT** represent it as withdrawing the member's credentials on a `private` room. The answer says what the removal did in its structured members, and nothing more is required of it: `member.state` is `removed`; `revocation` and `revokedAt`, present on `open` and `attributed` rooms, say that the host refuses the member's credentials from `revokedAt`; `member.validUntil`, where `revocation` is absent, is when a host stops honouring them; and `epoch`, present only on a sealed tier for an admitted member, is the first epoch sealed without them.

A conforming **producer** (the administrator's surface):

1. **SHOULD** show the requester, before the request is signed — and before any step-up gesture bound to it — what removal does and does not do on the room's tier (*What removal means, and does not*, above), and on a `private` room **SHOULD** show `validUntil` from the member's record.
2. **SHOULD** report the answer from its structured members — `revocation` and `revokedAt` where present, `validUntil` where they are absent, and `epoch` where present — and **MUST NOT** report any removal as ending access to what the member already read.

### Why the answer carries no notice

What a removal does not do is the same for every removal on a tier, so a sentence saying it in every answer carries nothing the caller cannot render from this specification, from the tier, and from `revocation`, `validUntil` and `epoch` — and it would be server-chosen prose shown to an administrator as guidance, in one language, after the decision was made. The moment it matters is **before** the requester commits, when a step-up gesture binds them to this one document, which is why the producer requirements put it there. An implementation **MAY** still carry such a statement under `ext`; a caller **MUST NOT** depend on one.

## Authorization

The authority this task presupposes is the **`vtc.rooms.manage` capability at the community**: the standing to run the rooms the community owns — create them, admit and remove their members. It is distinct from `vtc.rooms.admin`, the community's authority as a **host** (storage, limits, usage, over every room it stores), because a community may let someone run its board room without letting them reassign every room's storage, and the reverse; [`vtc/roles/define`](../../../../../../vtc/roles/define/0.1/spec.md) can place either in a role's ceiling. It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

**This is the owner's authority, exercised through the owner's key holder.** The community is the room's owner, and its own VTA holds the room's signing key and, on a sealed tier, the community's place in the room's group. The community acts on the room by sending its VTA the same `rooms/owner/*` and `rooms/keys/*` tasks any owner's key holder serves, as itself; the VTA applies its own gates to each. The community holds no room key and no group secret.

**The model this task is designed for.** A community implementing administrator step-up binds this operation to a step-up of the requester's (a gesture bound to this one document), records it in its audit trail with its `reason`, and executes it without a second administrator's consent: a room the community owns is the community's own, and what this changes is reversible. A community whose approvals rules require consent for it parks it as it parks any consented act, answering with `trust-task-next-step` naming the action. Whether a step-up or consent is required is the community's policy, which this specification does not declare ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).

## Request

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/members/remove/0.2#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-14T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:board",
    "member": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "reason": "Left the board."
  }
}
```

## Response

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/members/remove/0.2#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-14T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "member": {
      "member": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "actions": [
        "read",
        "write"
      ],
      "validUntil": "2027-10-01T00:00:00Z",
      "state": "removed",
      "offeredAt": "2026-10-14T10:00:00Z",
      "admittedAt": "2026-10-14T11:00:00Z",
      "removedAt": "2026-11-02T09:00:00Z"
    },
    "revocation": {
      "sequence": 2,
      "digest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR"
    },
    "revokedAt": "2026-11-02T08:59:59Z",
    "epoch": 3
  }
}
```

## Security & Privacy

### Data carried

A member DID in; their record, the revocation list's head and an epoch out. The list the VTA delivers names the member to the room's host — the community itself, for the rooms it owns — which has seen them act on an `open` or `attributed` room.

### Correlation

The community maintainer's identifier scope is public: the community is addressed by its public DID because it is the party administrators, members and the rooms it owns already know it by: it is the room's owner of record and the issuer members' admissions trace back to, and a pairwise identifier per caller would make a room's owner differ by who asks. On a `private` room the room's host sees an epoch advance and a commit it cannot read, never who left, and no list naming them is delivered.

### Retention

The removal stays with the member's record.

### Consent/purpose

Removal is the owner's, and needs no consent from the member removed.
