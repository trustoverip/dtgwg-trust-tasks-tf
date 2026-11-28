---
slug: vtc/events/invitations/revoke
version: '0.1'
title: VTC Events — Invitations — Revoke
summary: 'A host revokes an invitation: its notification is withdrawn and it no longer puts the member in the audience. A registration the member made stands.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
- invitation
parties:
- role: event host
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: 'Revoking an invitation is a host''s decision about who may see the event; it must be attributable.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor.'
sideEffects:
  level: mutating
  rationale: 'Marks the invitation revoked and withdraws its notification. Not recoverable: a host invites the member again.'
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: 'The invitation: who it names by entry, who invited, the answer. Never the event''s registrations.'
retention:
  class: durable
  rationale: 'The revoked invitation is kept until the event''s records go, so the member is not invited again by the audience rule against the host''s decision.'
errorCodes:
- code: vtc/events/invitations/revoke:invitationNotFound
  meaning: 'The event has no invitation with this identifier.'
  retryable: false
- code: vtc/events/invitations/revoke:alreadyRevoked
  meaning: 'The invitation was revoked already; nothing changed.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/invitations/send
- vtc/events/invitations/list
- vtc/events/invitations/links/revoke
---

## Abstract

The **VTC Events — Invitations — Revoke** Trust Task lets an event's host take back an invitation they or another host made, or one the community made under the event's invitation mode, or one a member or a link made (VTI-EVT-103). The invitation's notification is withdrawn (VTI-NTF-065), and where the invitation was what put the member in the event's audience it no longer does. A registration the member made stands: revoking an invitation is not removing a registrant, which is [`registrations/remove`](../../../registrations/remove/0.3/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), decision 5, and the requirements in the VTI Community Events chapter (VTI-EVT-100 – 135).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for an event that does not exist and one the caller does not host and may not see (VTI-EVT-023).
2. Refuse an invitation the event does not have with `invitationNotFound`, and one already revoked with `alreadyRevoked`.
3. Mark the invitation `revoked` and withdraw its notification: a revision marked withdrawn, sent to the invitee as every earlier revision was (VTI-NTF-024, VTI-NTF-065).
4. From then on, not count the invitation toward the invitee's place in the event's audience (VTI-EVT-103); the audience rule, a group, a role or another standing invitation may still admit them.
5. Leave any registration the invitee made as it stands (VTI-EVT-103), and not invite them again under the event's invitation mode while the revoked invitation is kept (VTI-EVT-101).
6. Audit the revocation by reference — event, invitation, entry — never with an answer or a note (VTI-EVT-091).

## Authorization

**Authority: `vtc.events.manage` covering the event** — a host's grant qualified `event:<eventId>`, or the authority over the event's calendar (VTI-EVT-013, VTI-EVT-070) — read from the signer's access-control entry at execution time. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A host's act: it **MUST NOT** be accepted from a key the portal holds for a member under `auth/oob` `signingScope` (VTI-EVT-095).

## Definitions

- `EventId`, `EventInvitation`, `EventInvitationId` — see [`event.schema.json`](../../../_shared/0.2/event.schema.json).

## Request

The event host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Revoke an invitation

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000561",
  "type": "https://trusttasks.org/spec/vtc/events/invitations/revoke/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-21T10:00:00Z",
  "payload": {
    "eventId": "evt_IntroSoldering2611",
    "invitationId": "inv_W5k2N8xRb3Qz7TpL"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:priya#key-1",
    "created": "2026-11-21T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/invitations/revoke/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Revoke an invitation — answered

```json
{
  "id": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000562",
  "type": "https://trusttasks.org/spec/vtc/events/invitations/revoke/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-21T10:00:01Z",
  "threadId": "urn:uuid:7d4e2b19-6a3c-4e8f-9b2d-000000000561",
  "payload": {
    "invitation": {
      "invitationId": "inv_W5k2N8xRb3Qz7TpL",
      "eventId": "evt_IntroSoldering2611",
      "entryId": "dir_AdaLovelace0000000",
      "invitedBy": "host",
      "inviterEntryId": "dir_PriyaNatarajan000",
      "replyExpected": false,
      "answer": "none",
      "revoked": true,
      "createdAt": "2026-11-20T10:00:01Z"
    }
  }
}
```

## Security & Privacy

### Data carried

An event and an invitation identifier in; the invitation out, its invitee named by community-scoped `entryId`.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID.

### Retention

**Durable**: the revoked invitation is kept with the event's records, so the audience rule does not invite the member again against the host's decision.

### Consent/purpose

Taking back an invitation that should not have been made. The invitee is told only that the notification was withdrawn, never why.
