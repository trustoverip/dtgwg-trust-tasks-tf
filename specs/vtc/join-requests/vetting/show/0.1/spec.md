---
slug: vtc/join-requests/vetting/show
version: "0.1"
title: "VTC Join Requests — Vetting — Show"
summary: An administrator reads the identity-vetting facts one join request was decided on — every statement it carried, which counted and why not, and which have been withdrawn since.
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
    Reads the facts recorded at a decision and the current withdrawal notices. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses, to an administrator, which vetters vouched for an applicant, by which method, their declared relationship to the applicant, and whether each counted — personal data about the applicant and the vetters, disclosed to the party that decided the application.
retention:
  class: transient
  rationale: >-
    A read. The facts themselves were recorded at the decision and are kept with the join request.
errorCodes:
  - code: vtc/join-requests/vetting/show:notFound
    meaning: "No join request with this id exists."
    retryable: false
related:
  - vtc/join-requests/show
  - vtc/join-requests/decide
  - vtc/vetting/revocations/list
  - vtc/schemas/accepts/show
---

## Abstract

When a join request is decided under an Accepts criterion that requires peer identity vetting, the community records what it established about the vetting evidence: every statement the presentation carried, whether each verified, whether its issuer was an eligible vetter, whether it counted, the method floors and independence caps, and what was still missing.

The **VTC Join Requests — Vetting — Show** Trust Task returns those facts for one request, with one live addition — `withdrawnNow`, whether each statement's vetter has withdrawn it since the decision. It is what an administrator reads when [`vtc/vetting/revocations/list`](../../../../vetting/revocations/list/0.1/spec.md) flags an admission for review, and what explains a decision the join policy made.

**Why a `vetting/show` sub-slug rather than a member of [`vtc/join-requests/show`](../../../show/0.1/spec.md).** The request record and its vetting facts are different reads with different audiences and sizes: every administrator console lists and shows requests, while the facts are the evidence trail of one decision, name third parties (the vetters), and are read when something is being questioned. Adding them to the published `show` response would also be a breaking change to its schema. `show` under `vetting/` leaves room for a later verb on the same facts.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`, before looking the request up.
2. **MUST** answer `notFound` when no join request has this id.
3. **MUST** omit `vetting` — not refuse — for a request decided without vetting requirements, or recorded before the community kept the facts.
4. **MUST** return the facts as recorded at the decision, and **MUST** compute `withdrawnNow` from the withdrawal notices it holds at the time of the read.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — read at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

## Definitions

- **Vetting facts** — the join policy's vetting input for the decision, recorded with the request.
- **`counted`** — the statement counted toward the requirements; **`failures`** says why not.
- **`withdrawnNow`** — the statement's vetter has since withdrawn it (vtc/vetting/revoke-statement).

## Request

A community administrator (`issuer`) names one join request to the community (`recipient`).

### Why was this applicant referred?

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/join-requests/vetting/show/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "id": "0f8e7d6c-5b4a-4938-8271-605f4e3d2c1b"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### One statement counted — and has since been withdrawn — and one did not

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/join-requests/vetting/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "requestId": "0f8e7d6c-5b4a-4938-8271-605f4e3d2c1b",
    "vetting": {
      "criterionId": "membership",
      "requirementsDigest": "zQmNLei78zWmzUdbeRB3CiUfAizWUrbeeZh5K1rhAQKCh51",
      "applicantDigestMatches": true,
      "statements": [
        {
          "id": "urn:uuid:5e4d3c2b-1a09-4f8e-9d7c-6b5a4f3e2d1c",
          "issuer": "did:example:vetter",
          "verified": true,
          "eligible": true,
          "revoked": false,
          "withdrawnNow": true,
          "method": "inPerson",
          "declaredRelationship": "none",
          "counted": true,
          "failures": []
        },
        {
          "id": "urn:uuid:6f5e4d3c-2b1a-4098-8e7d-7c6b5a4f3e2d",
          "issuer": "did:example:vetter-2",
          "verified": true,
          "eligible": false,
          "revoked": false,
          "withdrawnNow": false,
          "method": "video",
          "counted": false,
          "failures": [
            "vetting:ineligibleVetter"
          ]
        }
      ],
      "distinctCountedVetters": 1,
      "byMethod": {
        "inPerson": 1
      },
      "commitmentsConsistent": true,
      "independenceOk": true,
      "invitationRequired": false,
      "satisfied": false,
      "needs": [
        "vetting:minStatements"
      ],
      "recordedAt": "2026-09-20T14:00:00Z"
    }
  }
}
```

### A request decided without vetting

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vtc/join-requests/vetting/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001fe",
  "payload": {
    "requestId": "0f8e7d6c-5b4a-4938-8271-605f4e3d2c1b"
  }
}
```

## Security & Privacy

### Data carried

Vetter DIDs, statement identifiers, vetting methods, each vetter's declared relationship to the applicant, and machine-coded reasons. No statement body, no identity commitment value, and no free text.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The response joins an applicant's request to the vetters who vouched for them; the community already holds that join, and discloses it only to administrators.

### Retention

A read. The facts are kept with the join request as the record of why it was decided as it was; a caller need not keep a copy.

### Consent/purpose

The purpose is to explain and, where evidence has been withdrawn, revisit one decision. A vetter's declared relationship is their own attributable declaration, disclosed so an administrator can apply the community's independence rules — not to be read as a finding about the relationship.
