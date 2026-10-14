---
slug: vtc/rooms/community/members/add
version: "0.1"
title: "VTC Rooms — Community — Members — Add"
summary: "An administrator admits a community member to a data room the community owns: the community's VTA issues the room's credentials to them, and the community keeps them as an offer the member collects."
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
    Grants a party access to a room's content from now on, through credentials signed in the room's name. It must be attributable to its requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed admission would re-issue credentials to someone since removed.
sideEffects:
  level: mutating
  rationale: >-
    Has the community's VTA sign three credentials and records an offer to the member.
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request names a member, actions and an expiry. The response is the member's record. The credentials are kept for the member to collect, never returned to the administrator.
retention:
  class: durable
  rationale: >-
    The offer, and after acceptance the admission, are the owner's record of whom it admitted, kept until the member is removed and their authority has lapsed.
errorCodes:
  - code: vtc/rooms/community/members/add:notFound
    meaning: "The community owns no room with this identifier."
    retryable: false
  - code: vtc/rooms/community/members/add:notAMember
    meaning: "`member` is not an active member of this community."
    retryable: false
  - code: vtc/rooms/community/members/add:alreadyMember
    meaning: "`member` is already admitted to the room or holds an open offer to it."
    retryable: false
  - code: vtc/rooms/community/members/add:keyHolderUnavailable
    meaning: "The community's VTA did not answer. Nothing was offered."
    retryable: true
  - code: vtc/rooms/community/members/add:keyHolderRefused
    meaning: "The community's VTA refused to issue. Its own code and message are carried in `details`. Nothing was offered."
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
  - vtc/rooms/offers/accept
  - vtc/rooms/community/members/remove
  - rooms/owner/invite
  - rooms/owner/issue-membership
  - rooms/owner/issue-authority
---

## Abstract

**VTC Rooms — Community — Members — Add** admits one member of the community to a room the community owns, with the actions they may take there and until when.

**Admission is pull-shaped.** The member's key holder is their own VTA, which no community can reach: it has no inbound address, and a member's browser is not an inbox. So the community does not deliver anything. It has its VTA issue the room's three credentials to the member — the invitation ([`rooms/owner/invite`](../../../../../../rooms/owner/invite/0.1/spec.md)), membership ([`rooms/owner/issue-membership`](../../../../../../rooms/owner/issue-membership/0.1/spec.md)) and authority ([`rooms/owner/issue-authority`](../../../../../../rooms/owner/issue-authority/0.2/spec.md)) — and keeps them as an **offer**. The member lists their offers ([`vtc/rooms/offers/list`](../../../../../../vtc/rooms/offers/list/0.1/spec.md)), stores the credentials with their own key holder and accepts ([`vtc/rooms/offers/accept`](../../../../../../vtc/rooms/offers/accept/0.1/spec.md)); on a sealed tier, accepting is what puts them in the room's group.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without `vtc.rooms.manage` with `permissionDenied`, and **MUST** answer `notFound` for a room it does not own.
2. **MUST** refuse with `notAMember` a `member` who is not an active member of the community. A room the community owns is a community resource; a party from outside is admitted to a room a member owns, which does not need the community's say.
3. **MUST** refuse with `alreadyMember` a member already admitted or holding an open offer.
4. **MUST** have its VTA issue all three credentials, the authority credential granting exactly `actions` until `validUntil`, and **MUST NOT** grant `admin`.
5. **MUST** record the offer, with its credentials, only after all three are issued, and **MUST NOT** return the credentials to the caller.
6. **SHOULD** remove the member from every room it owns when they leave the community.

## Authorization

The authority this task presupposes is the **`vtc.rooms.manage` capability at the community**: the standing to run the rooms the community owns — create them, admit and remove their members. It is distinct from `vtc.rooms.admin`, the community's authority as a **host** (storage, limits, usage, over every room it stores), because a community may let someone run its board room without letting them reassign every room's storage, and the reverse; [`vtc/roles/define`](../../../../../../vtc/roles/define/0.1/spec.md) can place either in a role's ceiling. It is read from the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

**This is the owner's authority, exercised through the owner's key holder.** The community is the room's owner, and its own VTA holds the room's signing key and, on a sealed tier, the community's place in the room's group. The community acts on the room by sending its VTA the same `rooms/owner/*` and `rooms/keys/*` tasks any owner's key holder serves, as itself; the VTA applies its own gates to each. The community holds no room key and no group secret.

**The model this task is designed for.** A community implementing administrator step-up binds this operation to a step-up of the requester's (a gesture bound to this one document), records it in its audit trail with its `reason`, and executes it without a second administrator's consent: a room the community owns is the community's own, and what this changes is reversible. A community whose approvals rules require consent for it parks it as it parks any consented act, answering with `trust-task-next-step` naming the action. Whether a step-up or consent is required is the community's policy, which this specification does not declare ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).

## Definitions

- `RoomMember`, `MemberActions` — see [`vtc/_shared/0.1/community-room.schema.json`](../../../../../../vtc/_shared/0.1/community-room.schema.json).
- **Offer** — the credentials issued to a member and kept for them to collect.

## Request

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/members/add/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-14T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:board",
    "member": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "actions": [
      "read",
      "write"
    ],
    "validUntil": "2027-10-01T00:00:00Z",
    "reason": "Joins the board."
  }
}
```

## Response

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/rooms/community/members/add/0.1#response",
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
      "state": "offered",
      "offeredAt": "2026-10-14T10:00:00Z"
    }
  }
}
```

## Security & Privacy

### Data carried

A member DID, actions and an expiry in; the member's record out. The credentials stay with the community until the member collects them; none is a secret, since a presentation of each must be signed by its subject.

### Correlation

The community maintainer's identifier scope is public: the community is addressed by its public DID because it is the party administrators, members and the rooms it owns already know it by: it is the room's owner of record and the issuer members' admissions trace back to, and a pairwise identifier per caller would make a room's owner differ by who asks. The community, as the room's owner, knows whom it admitted — it signed their credentials. That record lives in the owner's own store and is **never read on the host path**: the room is served, to its members and to any host it moves to, on the strength of its credential chains alone, so the room's portability and the host's ignorance of its membership are unchanged. On a `private` room this is what the owner knows anyway; the host half still learns nothing new.

### Retention

The offer is kept until accepted or withdrawn; the admission until the member is removed and their authority has lapsed.

### Consent/purpose

Admission is two-party: the community offers here, and the member consents by accepting.
