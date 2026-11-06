---
slug: vtc/badges/retire
version: '0.1'
title: VTC Badges — Retire
summary: 'Retire a badge: it is given to nobody again, its open offers are withdrawn, and for a community badge its status-bearing credentials are revoked and its badge group deleted; every award is kept for its holders'' own history.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- gamification
- group
parties:
- role: badge administrator or maker
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: Revokes credentials the community stands behind and deletes an identity it answers for; must stay attributable after the transport has closed, and is what the requester's step-up binds to.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. Placing the document in time bounds the duplicate-execution window.
sideEffects:
  level: destructive
  rationale: Marks the badge retired, withdraws every open offer and, for a community badge, revokes every badge credential that carries a status and deletes the badge group — which revokes the community's authorization credential to it, so every credential the group issued stops verifying. None of it is restored by creating a badge of the same name again. The awards are kept.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: Names a badge and a reason; returns the badge record.
retention:
  class: durable
  rationale: A retired badge and its awards are kept for its holders' own history (VTI-BDG-005); the community keeps the record of the retirement and why.
errorCodes:
- code: vtc/badges/retire:badgesOff
  meaning: The community has badges turned off (VTI-BDG-004). Nothing was changed; turning badges off keeps every record, and retiring one waits until they are on.
  retryable: false
- code: vtc/badges/retire:alreadyRetired
  meaning: The badge is retired and nothing of it remains to withdraw.
  retryable: false
- code: vtc/badges/retire:keyHolderUnavailable
  meaning: The community's VTA did not answer the revocation or the badge group's deletion. The badge is marked retired and its offers withdrawn; the community's reconciler completes the revocation and the deletion, and the request may be sent again until it has.
  retryable: true
- code: vtc/badges/retire:keyHolderRefused
  meaning: The community's VTA refused the revocation or the deletion. Its own code and message are carried in `details`. The badge is marked retired and its offers withdrawn; the revocation and the deletion wait for an operator.
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
- vtc/badges/show
- vtc/badges/update
- vtc/badges/create
- vtc/badges/awards/withdraw
- vtc/groups/delete
---

## Abstract

The **VTC Badges — Retire** Trust Task retires a badge (VTI-BDG-005). A retired badge is given to nobody again, is shown to nobody who does not hold it, and is kept — with every award of it — for its holders' own history. A community badge's retirement also ends what the badge stood behind: every badge credential that carries a status is revoked in the community's status set (VTI-BDG-063), and the badge group is deleted as [`vtc/groups/delete`](../../../groups/delete/0.1/spec.md) deletes one, which revokes the community's authorization credential to the group (VTI-DIR-083) and so voids every credential the group issued. A kudos badge has no group and no credential: retiring it withdraws its open offers and stops its giving.

## The steps, and their order

The community, on a valid request:

1. Audits the retirement by reference — the badge, the signer and the reason — before anything changes, so that what follows is attributable even if it is interrupted (VTI-BDG-091).
2. Marks the badge `retired` with `retiredAt`. From here the badge is given to nobody, shown to nobody who does not hold it, and left out of every listing but an administrator's and its holders' own.
3. Withdraws every open offer of it (`withdrawn`), as [`vtc/badges/awards/withdraw`](../../awards/withdraw/0.1/spec.md) withdraws one.
4. For a community badge, revokes every badge credential of it that carries a status — those issued under the outside regime — in the community's own status set (VTI-BDG-062, VTI-BDG-063), then deletes the badge group exactly as [`vtc/groups/delete`](../../../groups/delete/0.1/spec.md) does: stops admitting anyone, stops renewing every holder's credential, revokes the community's authorization credential to the group through the community's own status list (VTI-DIR-083), and deletes the group and its membership record. Where the community publishes to a trust registry, removes the badge group from it (VTI-BDG-080), through the projector that is the one retry owner.
5. Keeps every award record, accepted and closed alike, for its holders' own history.

The holders are told: their portal shows the badge retired, and a community **MAY** notify them.

Steps 2 and 3 are the community's own and complete before it answers. Step 4 needs its VTA: where the VTA does not answer, the badge is already retired and the community's reconciler, the one owner of those retries, completes the revocation and the deletion.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the administrator's console) **SHOULD** show the administrator, before the request is signed and before any step-up gesture bound to it, how many holders the badge has and that every credential it stands behind stops verifying.

A conforming **consumer** (the community) **MUST**:

1. Refuse every retire with `badgesOff` while its badge settings have `enabled` false (VTI-BDG-004).
2. Answer `notFound` for a badge the caller may not see, and `permissionDenied` for one they may see but may not retire: a community badge by anyone but a holder of `vtc.badges.admin`; a kudos badge by anyone but its maker (`createdBy`) or a holder of `vtc.badges.admin` (VTI-BDG-003). An owner of a community badge may not retire it: ownership is management of the badge group, and the badge is the community's.
3. For a community badge, require a step-up of the requester's bound to this one document. A kudos badge takes none.
4. Refuse a badge that is retired, and whose revocation and deletion are complete, with `alreadyRetired`; accept the request again for one whose step 4 is still owed, and answer it once that step completes.
5. Perform the steps above in order: audit first, then mark the badge retired and withdraw its open offers, then, for a community badge, revoke every status-bearing credential and delete the badge group as [`vtc/groups/delete`](../../../groups/delete/0.1/spec.md) does (VTI-BDG-005, VTI-BDG-063, VTI-DIR-083). Answer `keyHolderUnavailable` or `keyHolderRefused` where its VTA does not do its part, with the badge already retired, and complete that part with its reconciler.
6. Keep the badge record and every award of it for its holders' own history, and show the badge to its holders alone: on no card, in no listing and on no wall to anyone who does not hold it (VTI-BDG-005). A retired badge a holder had hidden stays hidden.
7. Where it publishes to a trust registry, remove the badge group from it (VTI-BDG-080).
8. Tell the holders: their portal shows the badge retired, and the community **MAY** notify them.

## Authorization

**Community badge: `vtc.badges.admin`** at the community, read from the signer's access-control entry at execution time, and a step-up bound to the request (VTI-BDG-003, VTI-BDG-005). An owner — a manager of the badge group — may give the badge and decide requests for it, but not retire it. **Kudos badge: its maker** (the badge's `createdBy`) **or `vtc.badges.admin`**, with no step-up: retiring a kudos badge revokes nothing. Retiring a badge confers nothing and removes no authority from anyone: a badge group never carried any (VTI-BDG-012). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

This is an administrator's or a maker's act, never a holder's: a key the member authorized for `vtc/badges` in the portal **MUST NOT** be accepted (VTI-BDG-095).

## Definitions

- `Badge`, `BadgeId`, `BadgeState` — see [`badge.schema.json`](../../_shared/0.1/badge.schema.json).
- **Reconciler** — the community's process that brings credentials, memberships and registry records into line with its own record; the one owner of retries for them, as [`vtc/groups/delete`](../../../groups/delete/0.1/spec.md) defines it.

## Request

The badge administrator or maker (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Retire a community badge

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000231",
  "type": "https://trusttasks.org/spec/vtc/badges/retire/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T09:30:00Z",
  "payload": {
    "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
    "reason": "The workshop programme ended with the season."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:administrator#key-1",
    "created": "2026-11-06T09:30:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/retire/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### The badge — retired

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000232",
  "type": "https://trusttasks.org/spec/vtc/badges/retire/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-06T09:30:03Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000231",
  "payload": {
    "badge": {
      "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
      "kind": "community",
      "name": "Workshop host",
      "description": "Ran a workshop for the community.",
      "criteria": "Hosted any workshop on the community calendar. A badge owner gives it after the session.",
      "image": { "sha256": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08", "mediaType": "image/png", "size": 61440 },
      "repeatable": true,
      "requestable": false,
      "listed": true,
      "shareableOutside": true,
      "groupId": "grp_Hs7Qm2Lk9Tn4Bv1X",
      "state": "retired",
      "holderCount": 12,
      "createdAt": "2026-11-06T09:00:02Z",
      "createdBy": "dir_AnnaReyes0000000",
      "retiredAt": "2026-11-06T09:30:03Z"
    }
  }
}
```

### Refused: the community's VTA did not answer

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000233",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-06T09:30:31Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000231",
  "payload": {
    "code": "vtc/badges/retire:keyHolderUnavailable",
    "message": "The badge is retired and its offers withdrawn; its credentials and group are being withdrawn. Send the request again to see it complete.",
    "retryable": true
  }
}
```

## Security & Privacy

### Data carried

A badge identifier and a reason in; the badge record out. The revocations and the group's deletion happen inside the community and its VTA; no credential, key or holder travels in either direction. A producer **MUST NOT** place personal data in `reason` or `ext`: the reason is kept with the audit record for the life of the community.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. Revoking the outside-regime credentials flips bits in the community's shared status set, whose crowd is the whole community's (VTI-BDG-062), so a verifier that fetches the list learns that *some* community credentials were revoked, not which badge or whose. Removing the badge group from the registry tells the registry the community no longer recognises that issuer; it never listed who held the badge (VTI-BDG-071).

### Retention

**Durable.** The badge record and every award of it are kept for its holders' own history (VTI-BDG-005); the membership record goes with the group, as [`vtc/groups/delete`](../../../groups/delete/0.1/spec.md) deletes it. The community keeps the record of the retirement and why.

### Consent/purpose

Ending a badge the community no longer gives. **A badge confers nothing**, so retiring one reduces nobody's authority: the community **MUST NOT** read a badge, an award or a badge credential in any authorization decision of its own (VTI-BDG-002), and a badge group was never bound to a capability or a role (VTI-BDG-012). The credentials it revokes were evidence for others; a verifier that relied on one finds it revoked, which is what the status is for (VTI-BDG-064).
