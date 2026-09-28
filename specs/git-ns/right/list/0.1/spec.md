---
slug: git-ns/right/list
version: "0.1"
title: "Git Namespaces — List Rights"
summary: "A community administrator lists every git right the VTC knows of — recorded rights and v0.1 role-derived grants alike — across every namespace, with the membership facts the console shows."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - rights
  - administration
  - console
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
  rationale: "The answer is the whole community's rights picture — who may sign, own or administer, and who granted it — disclosed to nobody but the community's own administrators. The VTC authorises the caller from its own records, so it must know from the document itself who the caller is; a proof binds the request to its `issuer` on every transport, and a bearer session proves nothing about the document."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no effect to replay, but an issue time dates the request for a VTC that audits administrative reads."
sideEffects:
  level: none
  rationale: "Reads the VTC's rights records and its role-derived configuration; changes nothing."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns every subject, right and resource the VTC records or derives, plus who granted each recorded right, when, and — restricted to community administrators, never to a resource's own owner — the granter's free-text reason; and whether the subject or the granter has since left the community."
retention:
  class: exchange
  rationale: "The request carries at most a resource and a subject filter, and is needed only to answer it."
errorCodes:
  - code: git-ns/right/list:notCommunityAdministrator
    meaning: "The caller does not hold the community-administrator capability. Unlike `git-ns/namespace/list` and `git-ns/repo/list`, holding `git.ns.admin` on some namespace is not enough: this task's answer spans every namespace and includes every granter's reason, which only the community-administrator capability entitles a caller to see."
    retryable: false
related:
  - git-ns/view
  - git-ns/right/grant
  - git-ns/right/revoke
  - git-ns/right/break-glass
  - git-ns/right/issued-by-departed
  - git-ns/namespace/list
  - git-ns/repo/list
---

## Abstract

[`git-ns/view`](../../../view/0.5/spec.md) shows a caller their own rights, and shows an administrator the rights within the namespaces they administer. Neither shows the whole community's rights picture in one answer, and neither carries the v0.1 `[hooks.git-trust] grant_on_role` grants that a hook relay publishes outside the rights model entirely — a right nobody granted through any `git-ns/*` task, and one the console must still show so an administrator sees the whole story of who may sign. This task is that whole-community list: every recorded right and every role-derived grant, each row also saying whether its subject is a current member and whether its granter has since departed. It is an admin console's *Rights* table, a command-line `rights list`.

It is a read. It changes nothing, grants nothing and publishes nothing.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

Not consequential, and declared for clarity, as the rest of the `git-ns/*` read family does. The caller is the DID the document's verified `proof` binds to its `issuer`; an unsigned document is refused with `proofRequired`.

The entitlement is **the community-administrator capability** in the VTC's own access control — the same capability [`git-ns/namespace/bind`](../../../namespace/bind/0.1/spec.md) requires, and the one behind `SPEC §7.2`'s "the VTC's own records" language throughout this family. It is deliberately narrower than [`git-ns/namespace/list`](../../../namespace/list/0.1/spec.md)'s and [`git-ns/repo/list`](../../../repo/list/0.1/spec.md)'s: holding `git.ns.admin` on one or more namespaces is not enough here, because this task's answer is not scoped to any namespace — it is every right the VTC knows of, everywhere, plus every granter's free-text reason, which those two tasks disclose only for the resources their caller administers. A caller who does not hold the community-administrator capability is refused with `git-ns/right/list:notCommunityAdministrator`.

The capability is read from the VTC's own records at execution time; the `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). Nothing in the payload names or widens whose rights are listed — `resource` and `subject` only narrow an answer the caller was already entitled to in full.

## Definitions

**Recorded right** — a live `git-ns/*` right record, as [`git-ns/view`](../../../view/0.5/spec.md) and the rights model define it. A break-glass record still waiting out a policy delay is recorded but not yet live; it is included here anyway, because it is exactly the one other administrators have a window to revoke.

**Role-derived grant** — a v0.1 `[hooks.git-trust] grant_on_role` entry: the hook relay grants `git.commit.sign` on a configured resource to every member holding a configured VTC access-control role. It is published from configuration, not recorded through any `git-ns/*` task, has no granter, grant time or expiry, and is withdrawn only by editing the configuration — never by [`git-ns/right/revoke`](../../revoke/0.3/spec.md).

**Row** — one [`AdminRightRow`](../../../_shared/0.5/git-ns.schema.json#/$defs/AdminRightRow): a recorded right restated with `subjectMember` and `granterDeparted`, or a role-derived grant with `subjectMember` and `granterDeparted: false`.

## Request

The community administrator sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Identifies the caller from the verified `proof` ([Authorization](#authorization)), refusing an unsigned document with `proofRequired`.
2. Confirms the caller holds the community-administrator capability, refusing with `git-ns/right/list:notCommunityAdministrator` otherwise.
3. Enumerates every live recorded right (a break-glass record awaiting its delay included) and every role-derived grant, narrowed to `resource` and `subject` where given, and returns them as `rights`, ordered by `resource` then `subject`.
4. Returns at most `limit` rows (clamped to 1..=500, default 100) and, when more match, a `nextCursor` to continue.

A VTC **MUST NOT** change anything on this task. A `cursor` carries the filters it was minted under; a VTC **MUST** reject a request that supplies a `cursor` together with a different `resource` or `subject` with `malformedRequest`, and a consumer **MUST NOT** send one.

### An administrator lists every commit-signing right on one repository

```json
{
  "id": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6b01",
  "type": "https://trusttasks.org/spec/git-ns/right/list/0.1",
  "threadId": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6b01",
  "issuer": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-26T09:10:00Z",
  "payload": {
    "resource": "github.com/acme/widgets"
  }
}
```

## Response

The VTC answers with the matching rows, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`: `proofRequired`, `malformedRequest`, or `git-ns/right/list:notCommunityAdministrator`.

### What Dana sees

One recorded right, one break-glass record awaiting ratification, and one role-derived grant a `[hooks.git-trust]` entry publishes for every holder of the `maintainer` VTC role.

```json
{
  "id": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6b02",
  "type": "https://trusttasks.org/spec/git-ns/right/list/0.1#response",
  "threadId": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6b01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "issuedAt": "2026-09-26T09:10:01Z",
  "payload": {
    "rights": [
      {
        "subject": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
        "right": "git.repo.own",
        "resource": "github.com/acme/widgets",
        "origin": "recorded",
        "grantedBy": "did:webvh:QmBobScid2:acme-vtc.example:bob",
        "grantedAt": "2026-09-23T09:50:00Z",
        "reason": "Founding maintainer of the repository.",
        "subjectMember": true,
        "granterDeparted": false
      },
      {
        "subject": "did:webvh:QmEveScid11:acme-vtc.example:eve",
        "right": "git.repo.maintain",
        "resource": "github.com/acme/widgets",
        "origin": "recorded",
        "grantedBy": "did:webvh:QmEveScid11:acme-vtc.example:eve",
        "grantedAt": "2026-09-25T14:00:00Z",
        "reason": "On-call incident response; no maintainer was reachable.",
        "subjectMember": true,
        "granterDeparted": false,
        "breakGlass": {
          "by": "did:webvh:QmEveScid11:acme-vtc.example:eve",
          "at": "2026-09-25T14:00:00Z",
          "justification": "Production outage; every maintainer was unreachable and the fix could not wait for reseating."
        }
      },
      {
        "subject": "did:webvh:QmFrankScid12:acme-vtc.example:frank",
        "right": "git.commit.sign",
        "resource": "github.com/acme/widgets",
        "origin": "roleDerived",
        "subjectMember": true,
        "granterDeparted": false
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries at most a resource and a subject filter. The response carries, across the whole community: every subject, right and resource the VTC governs or derives; who granted each recorded right and when; the granter's free-text reason, which can name why a right was needed and is disclosed here to nobody but community administrators; and whether the subject or the granter has since left. `reason` and the departure flags are the members that make this list an administrator's tool rather than a member's — they say not just what is granted but why, and whether the people involved are still around to be asked about it. A producer **MUST NOT** put anything beyond a resource or a subject identifier into `ext`.

### Correlation

Every DID the response names is already published as a right holder or a granter in the Trust Registry, so this task discloses no identifier a public reader could not already find one right at a time; what it adds is the join — every right at once, with reasons and departure status attached, which the Trust Registry does not carry. `resource` and `subject` let a caller narrow a call to one part of that join, but the entitlement check does not depend on the narrowing: a community administrator who asks unfiltered receives everything a filtered call would have, a page at a time.

The **VTC** declares `identifierScope: public`: the recipient of this request is always the community's own VTC, the same one every right in the answer is recorded against, and a community administrator must be able to recognise it as the same VTC across every rights read they make. The **community administrator** carries no such declaration; this task takes no position on whether their identifier is reused across other administrative reads.

### Retention

The request is kept only as long as the VTC keeps request logs; nothing in it needs to outlive the exchange. A VTC that audits administrative reads **MAY** record that the caller listed rights, and **SHOULD NOT** record the response, whose rows are already the VTC's own durable records under their own retention.

### Consent/purpose

The purpose is oversight of who may act on the community's git namespaces and why — the console view a community administrator uses to notice an unexpected grant, an unratified break-glass record, or a right held by someone who has left. A caller **MUST NOT** republish `reason` or the departure flags outside the community's own administration.
