---
slug: vtc/vetting/vetters/resend
version: "0.1"
title: VTC Vetting — Resend Vetter Grant
summary: A vetter asks the community to deliver their live vetter role credential again, after losing a device or when the first delivery never arrived. Nothing new is issued.
status: draft
targetFrameworkVersion: "0.5"
category: governance
keywords:
  - vtc
  - vetting
  - vetter
  - role
  - credential-delivery
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: vetter
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: The community delivers the credential only to the DID its grant names, so a forged request can at most cause a credential to be delivered again to its own holder. An authenticated transport is enough to find the sender's grant; a proof is recommended so a community that rate-limits resends can attribute them on every transport.
sideEffects:
  level: none
  rationale: "Delivers an existing credential again over credential-exchange/issue. It issues nothing, and changes no grant, status-list entry or member record."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "The response returns the credential's id and expiry. The credential itself goes to the member it names, over credential-exchange/issue, and says only that the member holds the vetter role in this community. The request carries nothing."
retention:
  class: transient
  rationale: "Nothing about the grant changes, so there is nothing new to keep. A community may count resends for rate limiting."
errorCodes:
  - code: vtc/vetting/vetters/resend:notGranted
    meaning: "The sender holds no live vetter grant in this community: it was never granted, has expired or been revoked, or the sender is not an active member."
    retryable: false
related:
  - vtc/vetting/vetters/grant
  - vtc/vetting/vetters/profile
  - credential-exchange/issue
  - vetting/request
---

## Abstract

A vetter proves eligibility to an applicant by presenting the vetter role credential their community issued through [`vtc/vetting/vetters/grant`](../../grant/0.1/spec.md). A vetter who has lost it — a new device, a wallet restored from an old backup, or a first delivery that never arrived — cannot vet, although the grant is still live.

This task asks the community to deliver that credential again. The community sends the same credential over [`credential-exchange/issue`](../../../../../credential-exchange/issue/0.1/spec.md) and returns its `id` and `validUntil`. Nothing new is issued: the grant, its revocation status entry and its expiry are unchanged.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5 and may change in place while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **vetter** (`issuer`) sends an empty payload from the member DID its grant names.

A conforming **community** (`recipient`):

1. Applies the [SPEC §7.2](/SPEC.md#72-consumer-requirements) pipeline.
2. **MUST** refuse with `vtc/vetting/vetters/resend:notGranted` a sender that holds no **live vetter grant** — a vetter role credential this community issued to the sender, neither expired nor revoked, as defined in [`vtc/vetting/vetters/grant`](../../grant/0.1/spec.md). A sender that is not an active member holds none, because the community revokes a member's grants when they leave.
3. Otherwise **MUST** deliver that grant's credential, exactly as issued, over [`credential-exchange/issue/0.1`](../../../../../credential-exchange/issue/0.1/spec.md) to the DID the credential names. It returns the credential's `id` as `credentialId`, and its `validUntil`. A community that cannot hand the delivery to its transport refuses with `unavailable`, not with a response.
4. **MUST NOT** issue a new credential, change the credential's validity, or touch its revocation status entry. A vetter whose grant is about to expire needs an administrator to grant the role again once it has.
5. **MAY** limit how often it delivers to one vetter under its own policy.

Repeating a request is safe: each one delivers the same credential again and changes nothing at the community.

## Authorization

The authority this task presupposes is **being the subject of a live vetter grant**. The credential is the vetter's already. Delivering it again gives the vetter nothing they were not given at the grant, and gives nobody else anything, because it goes only to the DID it names.

This task is not consequential ([SPEC §2](/SPEC.md#2-terminology)): it changes no state at the community and discloses nothing secret. It is declared here anyway, so that nobody reads it as a way to obtain a credential. Per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10, identifying the sender establishes whose grant to look up, never that one exists.

An administrator's own tools may deliver a vetter's credential again as well. That is out of scope for this task, and delivers the same credential.

## Definitions

**Live vetter grant** — as defined in [`vtc/vetting/vetters/grant`](../../grant/0.1/spec.md): a vetter role credential issued by this community to the member, not expired and not revoked.

## Request

The vetter sends the request to the community; the payload is empty. See the top-level schema in [`payload.schema.json`](payload.schema.json).

### Carol has a new phone

```json
{
  "id": "urn:uuid:7d9f1b3d-5e7a-4c9b-8d1f-3a5c7e9b1d01",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/resend/0.1",
  "threadId": "urn:uuid:7d9f1b3d-5e7a-4c9b-8d1f-3a5c7e9b1d01",
  "issuer": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
  "recipient": "did:webvh:QmVtcScid:kernel-vtc.example",
  "issuedAt": "2026-11-02T18:00:00Z",
  "payload": {}
}
```

## Response

The community, now responding, names the credential it delivered, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). The credential itself arrives separately, over `credential-exchange/issue/0.1`. Refusals use `trust-task-error` with a framework code or this specification's code.

### Delivered again

The same credential the [`vtc/vetting/vetters/grant`](../../grant/0.1/spec.md) example issued.

```json
{
  "id": "urn:uuid:7d9f1b3d-5e7a-4c9b-8d1f-3a5c7e9b1d02",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/resend/0.1#response",
  "threadId": "urn:uuid:7d9f1b3d-5e7a-4c9b-8d1f-3a5c7e9b1d01",
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "recipient": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
  "issuedAt": "2026-11-02T18:00:01Z",
  "payload": {
    "credentialId": "urn:uuid:5c7e9a1b-3d5f-4b7c-9e1a-2c4e6a8b0d01",
    "validUntil": "2027-09-13T10:00:01Z"
  }
}
```

### Refused: no live grant

```json
{
  "id": "urn:uuid:7d9f1b3d-5e7a-4c9b-8d1f-3a5c7e9b1d04",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:7d9f1b3d-5e7a-4c9b-8d1f-3a5c7e9b1d03",
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "recipient": "did:webvh:QmDavidScid1:kernel-vtc.example:david",
  "issuedAt": "2026-11-02T18:10:01Z",
  "payload": {
    "code": "vtc/vetting/vetters/resend:notGranted",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request carries nothing. The response carries the credential's identifier and expiry, which the vetter was given at the grant. The credential goes to the DID it names and nowhere else, so a resend copies role material to nobody new. It says only that the member holds the `vetter` role in this community.

### Correlation

The vetter declares `identifierScope: public`, for the reason [`vtc/vetting/vetters/grant`](../../grant/0.1/spec.md) gives: the credential names the member DID, and a vetter must present it under that DID. A resend adds no new handle. It delivers the same credential, with the same `id` and the same status-list entry. Issuing a fresh credential instead would give every applicant who saw both a second identifier to link, and would spend a status-list slot for nothing.

The community declares `identifierScope: public` for the same reason the grant does.

### Retention

Transient. Nothing about the grant changes. A community **MAY** count resends per vetter to rate-limit them, and has no reason to keep more.

### Consent/purpose

The purpose is to restore a vetter's ability to present a grant they already hold. A resend often follows a lost or replaced device. A community **MUST NOT** read one as evidence that a vetter's keys were compromised, or act against the vetter on that basis. A vetter who believes a key was compromised needs key rotation, and possibly revocation of the grant, not this task. Whether an agent asks its user before requesting a resend is the agent's policy; per [SPEC §7.3](/SPEC.md#73-specification-requirements) item 13 this specification takes no position on it.
