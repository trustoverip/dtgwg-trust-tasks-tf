---
slug: git-ns/right/issued-by-departed
version: "0.1"
title: "Git Namespaces — Rights Issued By Departed Members"
summary: "A community administrator lists the recorded git rights whose granter has since left the community, grouped by granter, so they can review or revoke what an absent person authorised."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - rights
  - departure
  - administration
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTC
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "The answer names, for every departed granter, exactly what they authorised while a member — a review surface a VTC discloses to nobody but its community administrators. The VTC authorises the caller from its own records, so it must know from the document itself who the caller is; a proof binds the request to its `issuer` on every transport, and a bearer session proves nothing about the document."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no effect to replay, but an issue time dates the request for a VTC that audits administrative reads."
sideEffects:
  level: none
  rationale: "Reads the VTC's rights records and its departure history; changes nothing. It does not revoke — that is git-ns/right/revoke."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns, for each departed granter, every right they issued that is still recorded, the subject who holds it, and the granter's free-text reason — a targeted view of one governance risk (an absent person's authority still standing) rather than the whole community's rights."
retention:
  class: exchange
  rationale: "The request carries at most a paging cursor, and is needed only to answer it."
errorCodes:
  - code: git-ns/right/issued-by-departed:notCommunityAdministrator
    meaning: "The caller does not hold the community-administrator capability. As with git-ns/right/list, holding git.ns.admin on a namespace is not enough: a departed granter's rights can span namespaces nobody currently administers."
    retryable: false
related:
  - git-ns/right/list
  - git-ns/right/grant
  - git-ns/right/revoke
  - git-ns/view
  - git-ns/namespace/reseat
---

## Abstract

A right's grant does not lapse when its granter leaves — [`git-ns/right/grant`](../../grant/0.3/spec.md)'s Authorization section is explicit that a granter's authority is checked at grant time, not held in trust afterward. Left unreviewed, the community accumulates rights authorised by people no longer there to answer for them, some in namespaces whose last admin was that same departure ([`git-ns/namespace/reseat`](../../../namespace/reseat/0.3/spec.md) exists for exactly that case). This task is the administrator's worklist for that risk: every recorded right whose granter has since left, grouped by granter, alongside the community's `cascade_on_departure` policy setting — `true` means departures already revoke these automatically and the list is normally empty; `false` means each one is a live right an administrator must look at and decide whether to keep or revoke individually. It is an admin console's *Issued by departed members* panel.

It is a read. It changes nothing, grants nothing and publishes nothing.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

Not consequential, and declared for clarity. The caller is the DID the document's verified `proof` binds to its `issuer`; an unsigned document is refused with `proofRequired`.

The entitlement is **the community-administrator capability** in the VTC's own access control, exactly as in [`git-ns/right/list`](../../list/0.1/spec.md) and for the same reason: a departed granter's surviving rights are not scoped to any namespace a caller might administer — the granter could have administered every namespace they touched, and now administers none. Holding `git.ns.admin` on some namespace does not entitle a caller to this list. A caller who does not hold the community-administrator capability is refused with `git-ns/right/issued-by-departed:notCommunityAdministrator`.

The capability is read from the VTC's own records at execution time; the `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

**Departed granter** — a DID recorded as `grantedBy` on a still-recorded right, who was a member of the community at the time of the grant and is not a current member now. The VTC's own DID, used as `grantedBy` for a right the VTC derives from its own configuration, is never a departed granter.

**Cascade on departure** — the community's `cascade_on_departure` policy setting ([`git-ns/right/grant`](../../grant/0.3/spec.md)'s policy surface). When on, a member's departure revokes every right they granted as part of the same operation that revokes their own; this task's list is then normally empty, and a non-empty answer signals the policy has not yet run or was toggled on after these rights survived a departure under the old setting.

## Request

The community administrator sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Identifies the caller from the verified `proof` ([Authorization](#authorization)), refusing an unsigned document with `proofRequired`.
2. Confirms the caller holds the community-administrator capability, refusing with `git-ns/right/issued-by-departed:notCommunityAdministrator` otherwise.
3. Finds every still-recorded right whose `grantedBy` was a member at grant time and is not a current member, groups them by granter, and returns `granters` ordered by the granter's DID.
4. Returns at most `limit` granters (clamped to 1..=500, default 100) and, when more match, a `nextCursor` to continue.
5. Returns `cascadeOnDeparture`, the community's active policy setting.

A VTC **MUST NOT** change anything on this task, and **MUST NOT** include a granter with no surviving right.

### Dana checks who has left rights behind

```json
{
  "id": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6c01",
  "type": "https://trusttasks.org/spec/git-ns/right/issued-by-departed/0.1",
  "threadId": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6c01",
  "issuer": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-26T09:15:00Z",
  "payload": {}
}
```

## Response

The VTC answers with the departed granters and their surviving rights, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`: `proofRequired`, or `git-ns/right/issued-by-departed:notCommunityAdministrator`.

### What Dana sees

`cascade_on_departure` is off, so Grace's grant to Henry survived her own departure.

```json
{
  "id": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6c02",
  "type": "https://trusttasks.org/spec/git-ns/right/issued-by-departed/0.1#response",
  "threadId": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6c01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "issuedAt": "2026-09-26T09:15:01Z",
  "payload": {
    "cascadeOnDeparture": false,
    "granters": [
      {
        "granter": "did:webvh:QmGraceScid13:acme-vtc.example:grace",
        "rights": [
          {
            "subject": "did:webvh:QmHenryScid14:acme-vtc.example:henry",
            "right": "git.repo.maintain",
            "resource": "github.com/acme/widgets",
            "origin": "recorded",
            "grantedBy": "did:webvh:QmGraceScid13:acme-vtc.example:grace",
            "grantedAt": "2026-08-01T10:00:00Z",
            "reason": "Covering releases while Grace was on leave.",
            "subjectMember": true,
            "granterDeparted": true
          }
        ]
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries at most a paging cursor. The response carries, for every departed granter: their DID, and for each right they issued that is still recorded — the subject, the right, the resource, when it was granted, and the granter's free-text reason. It is a narrower slice of what [`git-ns/right/list`](../../list/0.1/spec.md) can return, filtered to one governance question, but the reason field carries the same weight here: it is often the only record of *why* a now-unreachable person authorised something, which is exactly what makes it worth disclosing to an administrator deciding whether to revoke.

### Correlation

Grouping by granter is itself the correlating operation this task performs: it answers "everything this one departed person authorised," which a filtered call to `git-ns/right/list` could also assemble, one granter at a time, for a caller who already knew to ask. What this task adds is the enumeration — it tells an administrator which departed granters have surviving rights at all, without the administrator first knowing their names.

The **VTC** declares `identifierScope: public`, for the same reason as [`git-ns/right/list`](../../list/0.1/spec.md): the recipient is the community's own VTC, and a community administrator must recognise it as the same VTC every time they run this review. The **community administrator** carries no such declaration.

### Retention

The request is kept only as long as the VTC keeps request logs. A VTC that audits administrative reads **MAY** record that the caller ran this review, and **SHOULD NOT** record the response, whose rows are the VTC's own durable rights records under their own retention.

### Consent/purpose

The purpose is a specific governance review: finding rights a departed person authorised, so an administrator can decide whether they still make sense. It is not a general profile of departed members — the response says only what they granted while a member, not anything else about their departure. A caller **MUST NOT** republish `reason` or any granter's DID outside the community's own administration.
