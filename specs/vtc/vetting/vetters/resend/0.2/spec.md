---
slug: vtc/vetting/vetters/resend
version: "0.2"
title: VTC Vetting — Resend Vetter Grant
summary: A vetter asks the community to deliver their live vetter role credential again, or an administrator does the same on a named member's behalf. Nothing new is issued.
status: draft
targetFrameworkVersion: "0.6.0"
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
  - role: community administrator
    requirement: OPTIONAL
    identifierScope: pairwise
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "The community finds the grant to resend from the requester's identity, or from memberDid when an administrator sends the request, and rate-limits resends per requester, so it must know from the document itself who is asking and, on an administrator's path, that the sender holds that authority."
sideEffects:
  level: none
  rationale: "Delivers an existing credential again over credential-exchange/issue. It issues nothing, and changes no grant, status-list entry or member record."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "The response returns the credential's id and expiry. The credential itself goes to the member it names, over credential-exchange/issue, and says only that the member holds the vetter role in this community. The request carries at most one member DID, when an administrator sends it."
retention:
  class: transient
  rationale: "Nothing about the grant changes, so there is nothing new to keep. A community may count resends for rate limiting."
errorCodes:
  - code: vtc/vetting/vetters/resend:notGranted
    meaning: "The named member — the sender, or memberDid when an administrator sent the request — holds no live vetter grant in this community: it was never granted, has expired or been revoked, or the member is not an active member."
    retryable: false
related:
  - vtc/vetting/vetters/grant
  - vtc/vetting/vetters/profile
  - credential-exchange/issue
  - vetting/request
---

## Abstract

A vetter proves eligibility to an applicant by presenting the vetter role credential their community issued through [`vtc/vetting/vetters/grant`](../../grant/0.1/spec.md). A vetter who has lost it — a new device, a wallet restored from an old backup, or a first delivery that never arrived — cannot vet, although the grant is still live. `0.1` let only the vetter ask for their own resend; an administrator helping a vetter who could not reach their own agent had no route through this task, and used a REST-only door instead.

This task asks the community to deliver that credential again — to the vetter, or, when an administrator names them in `memberDid`, on the vetter's behalf. The community sends the same credential over [`credential-exchange/issue`](../../../../../credential-exchange/issue/0.1/spec.md) and returns its `id` and `validUntil`. Nothing new is issued: the grant, its revocation status entry and its expiry are unchanged.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change in place while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.1

`0.1` accepted only an empty payload, and only from the vetter whose grant it was — "a resend always concerns the sender's own grant" was itself one of `0.1`'s invalid-example notes. `0.2` adds an optional `memberDid`: present, the sender **MUST** hold the community-administrator capability and the resend concerns the named member's grant instead of the sender's own; absent, behaviour is exactly `0.1`'s.

This is purely additive: every `0.1` document (an empty payload) is a conforming `0.2` document. `0.1` is not retired.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **vetter** (`issuer`) resending their own credential sends an empty payload from the member DID its grant names, exactly as `0.1`.

A conforming **administrator** (`issuer`) resending on a vetter's behalf sets `memberDid` to that vetter's DID.

A conforming **community** (`recipient`):

1. Applies the [SPEC §7.2](/SPEC.md#72-consumer-requirements) pipeline.
2. Where `memberDid` is present, **MUST** refuse a sender without the community-administrator capability with the framework's `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)).
3. Resolves the **subject** of the request: `memberDid` when present, otherwise the sender.
4. **MUST** refuse with `vtc/vetting/vetters/resend:notGranted` a subject that holds no **live vetter grant** — a vetter role credential this community issued to the subject, neither expired nor revoked, as defined in [`vtc/vetting/vetters/grant`](../../grant/0.1/spec.md). A subject that is not an active member holds none, because the community revokes a member's grants when they leave.
5. Otherwise **MUST** deliver that grant's credential, exactly as issued, over [`credential-exchange/issue/0.1`](../../../../../credential-exchange/issue/0.1/spec.md) to the DID the credential names. It returns the credential's `id` as `credentialId`, and its `validUntil`. A community that cannot hand the delivery to its transport refuses with `unavailable`, not with a response.
6. **MUST NOT** issue a new credential, change the credential's validity, or touch its revocation status entry. A vetter whose grant is about to expire needs an administrator to grant the role again once it has.
7. **MAY** limit how often it delivers to one subject under its own policy.

Repeating a request is safe: each one delivers the same credential again and changes nothing at the community.

## Authorization

The authority this task presupposes is one of two: being the subject of a live vetter grant, or holding the community-administrator capability and naming that grant's subject in `memberDid`. The credential is the subject's already, whichever route asked for it. Delivering it again gives the subject nothing they were not given at the grant, and gives nobody else anything, because it goes only to the DID it names.

The vetter's own route is not consequential ([SPEC §3](/SPEC.md#3-terminology)): it changes no state at the community and discloses nothing secret. It is declared here anyway, so that nobody reads it as a way to obtain a credential. Per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10, identifying the sender establishes whose grant to look up, never that one exists — and, on the administrator's route, that the sender holds the community-administrator capability establishes only that they may ask this task to act on `memberDid`'s behalf, never that `memberDid` holds a grant.

## Definitions

**Live vetter grant** — as defined in [`vtc/vetting/vetters/grant`](../../grant/0.1/spec.md): a vetter role credential issued by this community to the member, not expired and not revoked.

## Request

The vetter sends the request to the community; the payload is empty. An administrator sends the same request naming the vetter in `memberDid`. See the top-level schema in [`payload.schema.json`](payload.schema.json).

### Carol has a new phone

```json
{
  "id": "urn:uuid:7d9f1b3d-5e7a-4c9b-8d1f-3a5c7e9b1d01",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/resend/0.2",
  "threadId": "urn:uuid:7d9f1b3d-5e7a-4c9b-8d1f-3a5c7e9b1d01",
  "issuer": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
  "recipient": "did:webvh:QmVtcScid:kernel-vtc.example",
  "issuedAt": "2026-11-02T18:00:00Z",
  "payload": {}
}
```

### Dana, an administrator, resends Carol's grant for her

Carol's device is locked out of its wallet backup; Dana resends on her behalf instead of waiting for her to regain access.

```json
{
  "id": "urn:uuid:8d1f3a5c-7e9b-4c2d-a4f6-1b3d5e7f9a03",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/resend/0.2",
  "threadId": "urn:uuid:8d1f3a5c-7e9b-4c2d-a4f6-1b3d5e7f9a03",
  "issuer": "did:webvh:QmDanaScid1:kernel-vtc.example:dana",
  "recipient": "did:webvh:QmVtcScid:kernel-vtc.example",
  "issuedAt": "2026-11-02T18:05:00Z",
  "payload": {
    "memberDid": "did:webvh:QmCarolScid1:kernel-vtc.example:carol"
  }
}
```

## Response

The community, now responding, names the credential it delivered, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). The credential itself arrives separately, over `credential-exchange/issue/0.1`, to the DID it names — Carol's, in both requests above, whichever party asked. Refusals use `trust-task-error` with a framework code or this specification's code.

### Delivered again

The same credential the [`vtc/vetting/vetters/grant`](../../grant/0.1/spec.md) example issued.

```json
{
  "id": "urn:uuid:7d9f1b3d-5e7a-4c9b-8d1f-3a5c7e9b1d02",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/resend/0.2#response",
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

### Refused: not an administrator

Erin names a member other than herself without holding the community-administrator capability.

```json
{
  "id": "urn:uuid:8d1f3a5c-7e9b-4c2d-a4f6-1b3d5e7f9a05",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:8d1f3a5c-7e9b-4c2d-a4f6-1b3d5e7f9a04",
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "recipient": "did:webvh:QmErinScid1:kernel-vtc.example:erin",
  "issuedAt": "2026-11-02T18:12:00Z",
  "payload": {
    "code": "permissionDenied",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The vetter's own request carries nothing; an administrator's names one member DID, one the community already holds in its own membership and grant records. The response carries the credential's identifier and expiry, which the subject was given at the grant. The credential goes to the DID it names and nowhere else, so a resend copies role material to nobody new — not even the requesting administrator, who never sees it. It says only that the member holds the `vetter` role in this community.

### Correlation

The vetter declares `identifierScope: public`, for the reason [`vtc/vetting/vetters/grant`](../../grant/0.1/spec.md) gives: the credential names the member DID, and a vetter must present it under that DID. A resend adds no new handle, whichever party requested it — it delivers the same credential, with the same `id` and the same status-list entry. Issuing a fresh credential instead would give every applicant who saw both a second identifier to link, and would spend a status-list slot for nothing.

The community declares `identifierScope: public` for the same reason the grant does. The administrator declares `identifierScope: pairwise`, because only this community needs to recognise it, against its own access control — the same declaration [`vtc/vetting/vetters/grant`](../../grant/0.1/spec.md) makes for the same role.

### Retention

Transient. Nothing about the grant changes. A community **MAY** count resends per subject to rate-limit them — on either route — and has no reason to keep more.

### Consent/purpose

The purpose is to restore a vetter's ability to present a grant they already hold, whether the vetter asks directly or an administrator asks on their behalf because the vetter could not. A resend often follows a lost or replaced device. A community **MUST NOT** read one as evidence that a vetter's keys were compromised, or act against the vetter on that basis, whichever party requested it. A vetter who believes a key was compromised needs key rotation, and possibly revocation of the grant, not this task. Whether a community requires the vetter's own consent before an administrator resends on their behalf, and whether an agent asks its user before requesting a resend, are the community's policy and the agent's; per [SPEC §7.3](/SPEC.md#73-specification-requirements) item 13 this specification takes no position on either.
