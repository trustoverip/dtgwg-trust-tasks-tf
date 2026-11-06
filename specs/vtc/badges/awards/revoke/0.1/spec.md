---
slug: vtc/badges/awards/revoke
version: '0.1'
title: VTC Badges — Awards — Revoke
summary: 'Take an accepted award back: revoke or stop renewing the credential that evidences it, remove the holder from the badge group where it was their last award, and tell them.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- gamification
- award
- revocation
parties:
- role: badge owner or administrator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: Taking back an award revokes a credential the community's badge group issued and may end a membership; the act must be attributable to its requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed revoke is refused as not accepted, but bounding it keeps a captured document from taking back a later award of the same badge.
sideEffects:
  level: mutating
  rationale: Sets one award `revoked`; may end the holder's badge-group membership, flip a status bit in the community's status set or stop a credential's renewal, and have the badge group re-issue a credential. Not reversible — the badge is offered again instead.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: The request names an award and a reason; the response is the award record, revoked. The holder is told the award was taken back, never the reason.
retention:
  class: durable
  rationale: A revoked award is kept for the holder's own history and the badge's owners (VTI-BDG-005); the reason is kept with the audit of the change.
errorCodes:
- code: vtc/badges/awards/revoke:badgesOff
  meaning: The community has badges turned off (VTI-BDG-004). Nothing changed.
  retryable: false
- code: vtc/badges/awards/revoke:notFound
  meaning: No award the signer may take back has this identifier. Conflates an unknown award, an award of a badge the signer may not see, and an award of a badge the signer neither owns nor administers.
  retryable: false
- code: vtc/badges/awards/revoke:notAccepted
  meaning: The award is not `accepted` — it is still offered, was declined, withdrawn or expired, or is already revoked. Its state is in `details.state`. An open offer is closed with vtc/badges/awards/withdraw. Nothing changed.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      state:
        type: string
        maxLength: 32
- code: vtc/badges/awards/revoke:keyHolderUnavailable
  meaning: The community's VTA did not answer the re-issue of the holder's remaining evidence. The award stays `accepted`; the community's sweeper, the one retry owner, completes the take-back. The requester need not re-issue this task.
  retryable: true
- code: vtc/badges/awards/revoke:keyHolderRefused
  meaning: The community's VTA refused the re-issue. Its own code and message are carried in `details`. The award stays `accepted`; the community's sweeper retries it.
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
- vtc/badges/awards/withdraw
- vtc/badges/awards/list
- vtc/badges/awards/offer
- vtc/badges/retire
- vtc/badges/mine/list
- vtc/groups/members/remove
---

## Abstract

The **VTC Badges — Awards — Revoke** Trust Task takes back an award its recipient had accepted (VTI-BDG-023). A badge's owner, or a holder of `vtc.badges.admin`, names the award and a reason; the community records the award `revoked`, tells the holder, and undoes what acceptance did: where the award was the holder's last accepted award of a community badge, they leave the badge group as a manager's removal removes them ([`vtc/groups/members/remove`](../../../../groups/members/remove/0.1/spec.md)); where the badge credential carries a status, its bit in the community's status set is flipped, and where it does not, its renewal stops (VTI-BDG-063); where the badge is repeatable and the holder keeps other awards of it, the badge group re-issues the credential with the remaining evidence.

A kudos award is just marked `revoked`: a kudos badge has no group and no credential (VTI-BDG-041), so there is nothing else to undo. An offer not yet accepted is closed with [`vtc/badges/awards/withdraw`](../../withdraw/0.1/spec.md), not here.

## The steps, and their order

On a valid request the community:

1. **Audits** the take-back by reference — the award, the badge, the holder's entry, the signer and `reason` — before anything changes (VTI-BDG-091).
2. **Records** the award `revoked` with `decidedAt`, and tells the holder their award of the badge was taken back, without the reason.
3. For a community badge, where the holder has no other accepted award of it, **removes** them from the badge group exactly as [`vtc/groups/members/remove`](../../../../groups/members/remove/0.1/spec.md) removes a member: the membership ends at once; the badge leaves their wall.
4. For a community badge, **revokes or stops** the credential that evidences the award: where the badge is `shareableOutside`, flips its bit in the community's own status set (VTI-BDG-062, VTI-BDG-063); otherwise stops the credential's renewal and lets it lapse (VTI-BDG-061).
5. For a repeatable community badge where the holder keeps other accepted awards of it, **re-issues** the badge credential with the remaining awards as its evidence, through a signing request to the community's VTA as [`vtc/badges/mine/accept`](../../../mine/accept/0.1/spec.md) obtains one, and offers it to the holder's agent as it offers any credential it issues (VTI-BDG-060). The credential it replaces is revoked or stopped as in step 4.

A failure in step 5 is the one that cannot be hidden: the holder's membership and other awards stand, but their credential names evidence they no longer hold. The community answers `keyHolderUnavailable` or `keyHolderRefused`, leaves the award `accepted`, and its sweeper — the one retry owner — completes steps 2 to 5 when the VTA answers. The requester **SHOULD NOT** re-issue this task to retry.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse every revoke with `badgesOff` while its badge settings have `enabled` false (VTI-BDG-004).
2. Refuse with `notFound` an `awardId` that names no award, an award of a badge the signer may not see, and an award of a badge the signer neither owns — manages its badge group, by `vtc.groups.manage` unqualified or qualified `group:<groupId>` — nor administers with `vtc.badges.admin` (VTI-BDG-023). A kudos badge has no owners: only a holder of `vtc.badges.admin` takes a kudos award back. The cases answer alike, so a revoke does not tell a stranger that an award exists.
3. Refuse with `notAccepted` an award whose `state` is not `accepted`, naming its state in `details.state`.
4. Perform the steps above in that order. Where the award was the holder's last accepted award of a community badge, end their badge-group membership as [`vtc/groups/members/remove`](../../../../groups/members/remove/0.1/spec.md) ends one (VTI-BDG-023); revoke the credential that evidences the award where it carries a status, over the community's own status set and never a set of the badge's or the group's own (VTI-BDG-062), and stop its renewal where it does not (VTI-BDG-063); tell the holder the award was taken back, and **MUST NOT** tell them `reason`.
5. For a repeatable badge with accepted awards remaining, re-issue the credential with the remaining evidence and offer it to the holder's agent; answer `keyHolderUnavailable` or `keyHolderRefused` when the VTA does not answer or refuses, leave the award `accepted`, and complete the take-back through its sweeper, which is the one retry owner.
6. For a kudos award, record it `revoked` and nothing else; the holder's count of the badge falls by one.
7. Keep the revoked award for the holder's own history and the badge's owners (VTI-BDG-005), and accept a later offer of the same badge to the same member.
8. **MUST NOT** read the award, the membership or the credential in any authorization decision of its own, before or after (VTI-BDG-002).

## Authorization

**Authority: ownership of the badge, or `vtc.badges.admin`, read now** (VTI-BDG-023). An owner takes an award back because the badge's owners answer for who holds it, and ownership is the badge group's management (VTI-BDG-011), read from the signer's access-control entry at execution time; a holder of `vtc.badges.admin` because they run the community's badges. Taking an award back removes a membership that carried no authority and a credential the community never reads (VTI-BDG-012, VTI-BDG-002): it is not a reduction of the holder's authority at the community. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

A revoke **MUST NOT** be accepted from a key a member delegated to a portal (VTI-BDG-095): the signer is resolved as the member or administrator themselves.

## Definitions

- `AwardId`, `Award`, `AwardState`, `Badge` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).
- `GroupCredentialId` — see the groups family's [`group.schema.json`](../../../../groups/_shared/0.1/group.schema.json).

## Request

The badge owner or administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### An owner takes back a badge given by mistake

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000501",
  "type": "https://trusttasks.org/spec/vtc/badges/awards/revoke/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T12:00:00Z",
  "payload": {
    "awardId": "awd_K3mT9pQx2Lw8Rv5N",
    "reason": "The workshop was cancelled; nobody hosted it."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:priya#key-1",
    "created": "2026-11-06T12:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/awards/revoke/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Taken back

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000502",
  "type": "https://trusttasks.org/spec/vtc/badges/awards/revoke/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-06T12:00:02Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000501",
  "payload": {
    "award": {
      "awardId": "awd_K3mT9pQx2Lw8Rv5N",
      "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
      "entryId": "dir_SamNguyen0000000",
      "givenBy": "dir_Pr1yaNatarajan00",
      "note": "For the Saturday joinery workshop.",
      "state": "revoked",
      "offeredAt": "2026-11-05T18:00:00Z",
      "decidedAt": "2026-11-06T12:00:01Z",
      "credentialId": "urn:uuid:5a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c4d"
    }
  }
}
```

### Refused: the VTA did not answer the re-issue

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000503",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-06T12:00:30Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000501",
  "payload": {
    "code": "vtc/badges/awards/revoke:keyHolderUnavailable",
    "message": "The community's VTA did not answer the re-issue; the award stands as accepted and the take-back will complete when it does.",
    "retryable": true
  }
}
```

## Security & Privacy

### Data carried

An award identifier and a reason in; the award record out, which names the holder and the giver by entry and carries the giver's note. The reason is kept for audit and for the badge's owners and administrators; the holder is told the award was taken back, never why. A producer **MUST NOT** place personal data about the holder in `reason` or `ext`.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. Where the badge may be shown outside, the flip of a status bit is visible to anyone who fetches the community's status list; the bit was allocated at random in a set the whole community's credentials share (VTI-BDG-062), so the list says that some credential of the community's changed, not whose or which badge's.

### Retention

**Durable.** The award is kept as `revoked`, with its `decidedAt`, for the holder's own history and the badge's owners, as long as the badge's awards are kept (VTI-BDG-005). The reason is kept with the audit record. The giver's note is erased with its author (VTI-BDG-090).

### Consent/purpose

The badge's owners, or the community's badge administrators, correcting what the community said about a member. **A badge confers nothing**: ending the membership and revoking the credential change what the holder can show, never what they may do at the community, which **MUST NOT** read a badge, an award or a badge credential in any authorization decision of its own (VTI-BDG-002).
