---
slug: vtc/vetting/revocations/list
version: "0.1"
title: "VTC Vetting — Revocations — List"
summary: An administrator pages through the identity-vetting statements vetters have withdrawn, and sees which current members were admitted on one.
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
    Reads withdrawal notices and the join requests that counted each statement. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses, to an administrator, which vetter withdrew which statement and why, and which members' admissions rested on it — personal data about vetters and members, disclosed to the party that decides what to do about it.
retention:
  class: transient
  rationale: >-
    A read; the community keeps nothing new. The notices themselves are durable records of vtc/vetting/revoke-statement.
errorCodes: []
related:
  - vtc/vetting/revoke-statement
  - vtc/join-requests/vetting/show
---

## Abstract

When a vetter withdraws an identity-vetting statement with [`vtc/vetting/revoke-statement`](../../../revoke-statement/0.1/spec.md), the community records a notice. The statement may already have counted toward an admission — the community decided on evidence its author has since taken back.

The **VTC Vetting — Revocations — List** Trust Task pages through those notices, newest first, and matches each to the approved join requests whose recorded vetting facts counted it, reporting which of their applicants are current members. `needsReview` is the administrator's cue to look; the task changes nothing.

**Why there is no `show` sibling.** A notice is keyed by its vetter and statement id, which an administrator does not hold before reading this list, and the question asked of the collection is "which need review" — answered by the `reviewState` filter. The per-request view is [`vtc/join-requests/vetting/show`](../../../../join-requests/vetting/show/0.1/spec.md), which reports `withdrawnNow` for each statement a request counted.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`.
2. **MUST** return notices newest `recordedAt` first, restricted to `reviewState` when present, at most `limit` per page, with `nextCursor` present exactly when more remain.
3. **MUST** match a notice to an approved join request only when the request's recorded vetting facts counted a statement with the notice's issuer and statement id, and **MUST** report `needsReview` exactly when at least one such request's applicant is a current member.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — read at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

## Definitions

- **Notice** — the community's record of a withdrawn statement.
- **`affectedJoinRequests`** — approved requests that counted the statement.
- **`affectedMembers`** — their applicants who are current members.

## Request

A community administrator (`issuer`) asks the community (`recipient`) for a page.

### Only the notices that need review

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/vetting/revocations/list/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "reviewState": "needsReview"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### One notice touching a current member

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/vetting/revocations/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "items": [
      {
        "issuer": "did:example:vetter",
        "statementId": "urn:uuid:5e4d3c2b-1a09-4f8e-9d7c-6b5a4f3e2d1c",
        "statementDigestMultibase": "zQmYtUc4iTCbbfVSDNKvtQqrfyezPPnFvE33wFmutw9PBBk",
        "reason": "newInformation",
        "recordedAt": "2026-09-27T16:20:00Z",
        "reviewState": "needsReview",
        "affectedJoinRequests": [
          "0f8e7d6c-5b4a-4938-8271-605f4e3d2c1b"
        ],
        "affectedMembers": [
          "did:example:member"
        ]
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Vetter DIDs, statement identifiers and digests, the vetter's coded reason, and the DIDs of members admitted on a withdrawn statement. No statement content and no free text.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The response joins a vetter to the members they vetted — a relationship the community already holds, disclosed only to its administrators.

### Retention

A read. A caller acting on a `needsReview` notice **SHOULD** keep its own record of what it decided; this task records nothing.

### Consent/purpose

The purpose is to let administrators revisit admissions whose evidence was withdrawn. Using the list to judge the members concerned without that review is outside it: a withdrawn statement says the vetter no longer vouches, not that the member did anything.
