---
slug: vtc/vetting/vetters/grants/list
version: "0.1"
title: "VTC Vetting — Vetter Grants — List"
summary: An administrator pages through every vetter grant the community has made — live, expired and revoked, automatic and manual — with each vetter's profile summary.
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
  requirement: RECOMMENDED
  rationale: >-
    Authority is the signer's administrator standing, which the community can establish from a verified proof or a transport-authenticated sender. A proof is recommended so the read is attributable on every transport, relayed ones included.
sideEffects:
  level: none
  rationale: >-
    Reads the community's vetter grants and profiles. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses, to an administrator, every member ever named a vetter — their DIDs, grant windows, revocations, and what they published in their vetter profile, including profiles they chose not to list. Personal data about members, disclosed to the party that administers their grants.
retention:
  class: transient
  rationale: >-
    A read; the community keeps nothing. The caller's copy is stale the moment a grant is revoked or the sweep runs.
errorCodes: []
related:
  - vtc/vetting/vetters/show
  - vtc/vetting/vetters/list
  - vtc/vetting/vetters/grant
  - vtc/vetting/auto-grant/show
---

## Abstract

The **VTC Vetting — Vetter Grants — List** Trust Task is the administrator's ledger of vetter grants: every grant the community has made through [`vtc/vetting/vetters/grant`](../../../grant/0.1/spec.md) or the automatic sweep ([`vtc/vetting/auto-grant/show`](../../../../auto-grant/show/0.1/spec.md)), newest first, whether live, expired or revoked, with who made it and the vetter's profile summary.

It is not [`vtc/vetting/vetters/list`](../../../list/0.1/spec.md), which is the public directory an applicant reads — live vetters who chose to be listed, and nothing about the rest. The slug sits under `vetters/grants/` because the thing enumerated is the grant, not the vetter: a member revoked and granted again appears twice.

**Why there is no `show` sibling in this family.** A single grant is looked up by vetter with [`vtc/vetting/vetters/show`](../../../show/0.1/spec.md), which answers a definite `live`, `revoked`, `expired` or `none`; the identifier an administrator holds is the member's DID, not the endorsement id, and withdrawing a grant goes through `vtc/endorsements/revoke` by `endorsementId` from this listing.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`.
2. **MUST** return every grant, newest `validFrom` first, at most `limit` per page, with `nextCursor` present exactly when more remain.
3. **MUST** compute `live` with the predicate vtc/vetting/vetters/show uses — unrevoked, unexpired, and held by a current member — so the two never disagree about one grant.
4. **MUST** include `revokedAt` exactly when `revoked` is true, and **MUST** include `profile` when the member has published one, whether or not it is listed.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — read at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

## Definitions

- **Grant** — a vetter role credential the community issued a member, and its record.
- **`origin`** — `auto` for the sweep's grants, `manual` for an administrator's.
- **`profile`** — a summary of the vetter's self-published profile.

## Request

A community administrator (`issuer`) asks the community (`recipient`) for a page.

### First page

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/grants/list/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "limit": 50
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### One live automatic grant

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/grants/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "items": [
      {
        "endorsementId": "urn:uuid:3c1f0e52-1a2b-4c3d-8e9f-0a1b2c3d4e5f",
        "memberDid": "did:example:vetter",
        "credentialId": "urn:uuid:7a8b9c0d-1e2f-4a3b-8c4d-5e6f7a8b9c0d",
        "validFrom": "2026-03-01T00:00:00Z",
        "validUntil": "2027-03-01T00:00:00Z",
        "revoked": false,
        "live": true,
        "origin": "auto",
        "profile": {
          "listed": true,
          "displayName": "Sam",
          "country": "DE",
          "languages": [
            "de",
            "en"
          ],
          "methods": [
            "inPerson",
            "video"
          ],
          "eventCount": 1,
          "updatedAt": "2026-09-01T12:00:00Z"
        }
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Member DIDs, grant windows and revocation times, and each vetter's profile summary — display name, country, languages, methods — which is self-asserted by the vetter and attributed to them wherever rendered. The revocation *reason* is deliberately not carried: it is the community's internal record.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The listing joins a member's DID to their vetting role and their published profile, including an unlisted one; that join is exactly what an administrator needs and what the public directory withholds.

### Retention

A read. A caller **SHOULD NOT** keep a copy beyond the task it was fetched for; it is stale at the next grant, revocation or sweep.

### Consent/purpose

The purpose is administration of the vetter role — seeing who holds it, how they came to, and which grants to withdraw. A vetter consented to being a vetter in this community; an unlisted profile is shown to administrators, never to applicants.
