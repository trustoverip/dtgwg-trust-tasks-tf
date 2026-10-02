---
slug: vtc/members/authority-reduced-notice
version: "0.1"
title: VTC Members — Authority-Reduced Notice
summary: A community tells a subject that an administrator reduced their administrative authority — on whose authority, when, why, and whether anyone else agreed.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - acl
  - notice
  - revocation
  - demotion
parties:
  - role: community maintainer
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: subject whose authority was reduced
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The subject may need to show a third party — another administrator, an auditor, a governance body — that their authority was reduced, by whom, and with or without anyone else's agreement. Without a proof the notice evidences nothing beyond the transport that carried it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: The notice is retained as evidence of a decision; a document that cannot be placed in time cannot be compared against the audit trail it summarises.
sideEffects:
  level: none
  rationale: Reports a reduction already carried out; the notice changes nothing at the recipient.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: Discloses the subject's own change of role, the deciding administrator's DID and the operator's stated reason to the subject only.
retention:
  class: durable
  rationale: The subject's evidence of a governance decision about them, kept for as long as they might contest it.
errorCodes: []
related:
  - vtc/members/removal-notice
  - acl/revoke
  - acl/change-role
  - acl/update
---

## Abstract

The **VTC Members — Authority-Reduced Notice** Trust Task is how a community
tells a subject that an administrator revoked their access control entry,
demoted their role, or narrowed what their entry allows. It names the deciding
administrator, when the change took effect, the reason given, and — the member
that makes it more than a courtesy — whether any other administrator agreed.

The specification of the Verifiable Trust Infrastructure requires a community
to obtain the consent of a party other than both the requester and the subject
before reducing another subject's unrestricted authority, and, where no such
party exists, to notify the subject (VTI-APV-019). This task is that
notification, and it is useful for every reduction, not only the unopposed one.

## Not a removal notice

[`vtc/members/removal-notice`](../../removal-notice/0.1/) reports that a member
was removed from the community. This task reports a reduction of
*administrative* authority, which usually leaves the subject a member. A
removal of a member who was also an administrator is reported by the removal
notice alone; a producer **MUST NOT** send both for one decision.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the community) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vtc/members/authority-reduced-notice/0.1`, with itself as `issuer` and the subject as `recipient`, carrying a `proof` and an `issuedAt`.
2. Send it **after** the reduction has taken effect, never on deciding it.
3. Set `agreement` to `consented` only where at least one administrator other than the decider and the subject consented to this reduction, and to `unopposed` otherwise.
4. Omit `resultingRole` exactly when `code` is `revoked`, and include it otherwise.
5. Populate `decidedAt` with when the reduction took effect.

A conforming **consumer** (the subject) **MUST** verify the `proof` and that the `issuer` is the community that held the entry before relying on the notice, and **MUST** treat the payload `did` as authoritative over transport addressing.

## Authorization

The notice is informational and confers nothing: it neither grants nor removes
authority, and receiving one is not evidence of standing at the community. The
community's authority to send it is the authority over its own access control;
the subject's entitlement to it is being the party the decision was about.

## Request

The notice is one-way: there is no response document, and the payload schema
carries no `$defs.Response`.

### An unopposed demotion in a two-administrator community

```json
{
  "id": "urn:uuid:5f1c2a7e-3b9d-4e60-a8c4-2d7f0b91e3a6",
  "type": "https://trusttasks.org/spec/vtc/members/authority-reduced-notice/0.1",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkFormerCoAdmin",
  "issuedAt": "2026-10-02T09:00:05Z",
  "payload": {
    "did": "did:key:z6MkFormerCoAdmin",
    "code": "demoted",
    "previousRole": "community-admin",
    "resultingRole": "member",
    "agreement": "unopposed",
    "reason": "Signing key reported lost; restoring after re-enrolment.",
    "decidedAt": "2026-10-02T09:00:01Z",
    "decidedBy": "did:key:z6MkFoundingAdmin"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-02T09:00:05Z",
    "verificationMethod": "did:web:community.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z3FXQjecWufY46yg5abdVZsXqLhxhueuSoZgNSARiKBk9czhSePTFehP8c3PGfb6a22gkfUKKiMU5gSwwFdcjtPar"
  }
}
```

### A revocation another administrator consented to

```json
{
  "id": "urn:uuid:7a2d3b8f-4c0e-4f71-b9d5-3e8a1c02f4b7",
  "type": "https://trusttasks.org/spec/vtc/members/authority-reduced-notice/0.1",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkRepoManager",
  "issuedAt": "2026-10-02T10:00:05Z",
  "payload": {
    "did": "did:key:z6MkRepoManager",
    "code": "revoked",
    "previousRole": "repo-manager",
    "agreement": "consented",
    "decidedAt": "2026-10-02T10:00:01Z",
    "decidedBy": "did:key:z6MkCommunityAdmin"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-02T10:00:05Z",
    "verificationMethod": "did:web:community.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z3FXQjecWufY46yg5abdVZsXqLhxhueuSoZgNSARiKBk9czhSePTFehP8c3PGfb6a22gkfUKKiMU5gSwwFdcjtPar"
  }
}
```

## Security & Privacy

### Data carried

The subject's own DID, the role they held and now hold, the deciding
administrator's DID, an operator-authored reason, and whether a third party
agreed. `reason` is free text that reaches the subject verbatim: a consumer
**MUST NOT** render it as markup, and a producer **MUST NOT** put anything in it,
or in `ext`, about any other subject.

### Correlation

The notice names the deciding administrator to the subject, which is the point:
a reduction whose maker is unnamed cannot be contested. It discloses nothing
about other subjects, the community's size or its other decisions. A subject
receiving several notices can correlate them by `issuer`, as they could by
their own membership.

The community's identifier is **public** on purpose (`identifierScope: public`):
the notice is evidence the subject may show others, and evidence is only
checkable against an issuer a third party can recognise and resolve. A pairwise
community identifier would make the notice verifiable to nobody but its
recipient, which defeats the reason it carries a proof.

### Retention

The subject is expected to keep the notice for as long as they might contest
the decision; it is their copy of a record the community also holds in its
audit trail. The community need not retain the sent notice beyond its own audit
row, which already records the decision.

### Consent/purpose

The notice exists to tell the subject about a decision concerning them and to
give them verifiable evidence of it. It is not for any other purpose, and
receiving it obliges the subject to nothing. Whether a reduction needed another
administrator's consent is decided by the community's own rules; this task only
reports which way it went.
