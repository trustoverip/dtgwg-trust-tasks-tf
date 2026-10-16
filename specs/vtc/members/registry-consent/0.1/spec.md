---
slug: vtc/members/registry-consent
version: "0.1"
title: "VTC Members — Registry Consent"
summary: "A member grants or withdraws their own consent to the community publishing their membership in its trust registry, without leaving and rejoining."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - members
  - consent
  - trust-registry
  - privacy
parties:
  - role: member
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "The signed document is the community's evidence that the member consented to publication, or withdrew that consent. Publication discloses the membership beyond the community, so the decision must stay attributable to the member after the transport has closed — to an auditor, to the registry's operator, and to the member asking later why they were listed."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed grant re-publishes a member who has since withdrawn, against the later of their two decisions; dating the document is what lets the community apply only the newer one."
sideEffects:
  level: mutating
  rationale: "Sets `publishConsent` on the member record and queues the registry sync that follows from it. Recoverable by the member sending the opposite value."
consequences:
  - "With `publishConsent: true`, the community may publish the member's membership in its trust registry at its next sync."
  - "With `publishConsent: false`, the community removes the member's published registry record at its next sync and stops publishing them."
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: true
  rationale: "The member acts on their own record; the subject is the signer and there is no target field. The request carries one boolean. The response tells the member about their own record and nothing about anyone else's. The disclosure this task permits — the membership, in the trust registry — happens at the community's sync, not in this exchange."
retention:
  class: durable
  rationale: "The consent record is what justifies publishing the member, and its withdrawal what obliges removal. A community keeps the signed decision now in force for as long as it keeps the member record, and SHOULD keep a withdrawn grant's record long enough to answer for the publication it permitted."
errorCodes:
  - code: vtc/members/registry-consent:notMember
    meaning: "The signer holds no member record in this community, or holds one that has been removed. There is no consent of theirs to record."
    retryable: false
  - code: vtc/members/registry-consent:superseded
    meaning: "The community has recorded a consent change issued after this document — by the member, or a withdrawal by an administrator — so applying it would undo a later decision. Nothing was changed. A member who still wants this value sends a new document."
    retryable: false
related:
  - vtc/members/update
  - vtc/join-requests/submit
  - vtc/members/admit
  - vtc/members/make-membership
  - vtc/registry/sync-jobs/list
  - vtc/registry/records/list
---

## Abstract

A member's consent to be published in the community's trust registry is the member's own privacy decision. Until this task, the only place to give it was `registryConsent` on [`vtc/join-requests/submit`](../../../join-requests/submit/0.3/spec.md), and [`vtc/members/update`](../../update/0.1/spec.md) lets an administrator withdraw it but never grant it. So a member admitted before consent existed, one who answered "no" at join and has changed their mind, and one admitted by an administrator ([`vtc/members/admit`](../../admit/0.1/spec.md), [`vtc/members/make-membership`](../../make-membership/0.1/spec.md)) — whose record starts without consent — could become listed only by leaving and rejoining, at the cost of their credentials, their role and any vetter grants. And a member who wanted to be unlisted had to ask an administrator.

**VTC Members — Registry Consent** is the member's own way to do both: they sign a document setting their `publishConsent` to `true` or `false`, and the community records it and syncs its registry accordingly.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (`issuer`) is the member, signing with their member DID. It supplies `publishConsent` and carries a proof. A client **MUST NOT** send this document on the member's behalf without the member's decision: it is the evidence that they made one.

A conforming **consumer** (`recipient`):

1. **MUST** resolve the signer to their member record, and refuse with `notMember` when there is none or it has been removed.
2. **MUST** refuse with `superseded`, changing nothing, when the consent change recorded on the member record was issued later than this document's `issuedAt` — whether that change was the member's own or an administrator's withdrawal through `vtc/members/update`. Two of the member's documents arriving out of order otherwise leave in force the decision they made first.
3. When `publishConsent` equals the recorded value, **MUST** answer with `changed: false` and `registrySync: "unchanged"`, and change nothing. Restating a decision is not an error.
4. Otherwise **MUST** record the new value on the member record, together with this document's `id` and `issuedAt` as the consent change now in force, and audit it. Then:
   - on a **grant**, where the community publishes to a trust registry and publishes this member, it queues the sync job that publishes them (`registrySync: "queued"`);
   - on a **withdrawal**, where the member is published, it **MUST** queue the job that removes their published record (`registrySync: "queued"`), and **MUST NOT** publish them again until they consent anew;
   - where the community publishes to no trust registry at present, it records the consent all the same and answers `registrySync: "notConfigured"`; the recorded consent governs any registry it configures later.
5. **MUST** answer with the member's `did`, the `publishConsent` now recorded, `changed`, `registrySync` and `effectiveAt`.

**Consent permits; it does not oblige.** A grant makes publication permissible; whether the community publishes a given member, or runs a registry at all, remains its own policy. A community that does not publish this member on other grounds answers `registrySync: "unchanged"` to a grant. A withdrawal, by contrast, always binds: a community **MUST** honour it and **MUST NOT** place any step a grant does not have in front of it.

## Authorization

The authority is **being the member**: the `issuer` of this document is the member whose record changes, and the task has no target field, so no party can use it for anyone else. That is the whole entitlement. It is not a capability an administrator holds, and an administrator **MUST NOT** be able to exercise it: an administrator's only lever on consent is withdrawal, through [`vtc/members/update`](../../update/0.1/spec.md), which refuses a grant with `consentGrantForbidden`.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the `issuer`, the transport identity or the `proof` establishes who sent the document. Here who sent it *is* the authorization question, answered by matching the proven signer to the member record — and nothing else is needed.

## Definitions

**`publishConsent`** — the member's decision, **REQUIRED**. `true` consents to the community publishing their membership in its trust registry; `false` withdraws that consent. It is a single decision about the community's one trust registry, the same one `registryConsent` on a join submission records. A consent per registry is not modelled here.

**`did` (response)** — the member, which is the signer.

**`publishConsent` (response)** — the value now recorded.

**`changed` (response)** — whether this document changed it.

**`registrySync` (response)** — what the community did about its registry: `queued`, `notConfigured` or `unchanged`. `queued` means a sync job is queued, not that the registry holds the change; a member or operator who wants to know that reads [`vtc/registry/records/list`](../../../registry/records/list/0.1/spec.md) or the sync queue.

**`effectiveAt` (response)** — when the consent now in force was recorded.

## Request

The member issues the request to the community maintainer; the payload is the top-level object in [`payload.schema.json`](payload.schema.json).

### A member admitted by an administrator consents to being listed

```json
{
  "id": "urn:uuid:7c2e9f4a-3b1d-4e8a-9c5f-1a2b3c4d5e6f",
  "type": "https://trusttasks.org/spec/vtc/members/registry-consent/0.1",
  "issuer": "did:web:carol.example",
  "recipient": "did:webvh:QmCommunity:community.example",
  "issuedAt": "2026-10-16T09:00:00Z",
  "threadId": "urn:uuid:7c2e9f4a-3b1d-4e8a-9c5f-0000000000ff",
  "payload": {
    "publishConsent": true
  }
}
```

## Response

The community maintainer answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`, never a `#response` document.

### Recorded, and publication queued

```json
{
  "id": "urn:uuid:7c2e9f4a-3b1d-4e8a-9c5f-1a2b3c4d5e70",
  "type": "https://trusttasks.org/spec/vtc/members/registry-consent/0.1#response",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-16T09:00:01Z",
  "threadId": "urn:uuid:7c2e9f4a-3b1d-4e8a-9c5f-0000000000ff",
  "payload": {
    "did": "did:web:carol.example",
    "publishConsent": true,
    "changed": true,
    "registrySync": "queued",
    "effectiveAt": "2026-10-16T09:00:01Z"
  }
}
```

### Refused: a later decision is already in force

```json
{
  "id": "urn:uuid:7c2e9f4a-3b1d-4e8a-9c5f-1a2b3c4d5e71",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-16T09:05:01Z",
  "threadId": "urn:uuid:7c2e9f4a-3b1d-4e8a-9c5f-0000000001ff",
  "payload": {
    "code": "vtc/members/registry-consent:superseded",
    "message": "A consent change issued after this document is already recorded; send a new document if you still want this value.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

One boolean in the request, and the member's own consent state in the response. Nothing about any other member travels either way. A producer **MUST NOT** put anything in `ext` that the member has not chosen to give the community.

### Correlation

The community already knows who its members are; this task tells it only what each member decided about the registry, and when. The registry is where the correlation this task permits happens: a published membership is visible to whoever reads the registry, and links the member's DID to the community there. That is exactly what the member is deciding about, which is why the decision is theirs and why the document is signed.

### Retention

The decision now in force is kept with the member record, as its `id` and `issuedAt` and the signed document, for as long as the record is kept. It is the community's answer to "why is this member listed?", and the ordering in Conformance item 2 depends on it. A community **SHOULD** keep the record of a grant it acted on for as long as it may be asked to account for the publication that grant permitted, even after a withdrawal.

### Consent/purpose

This task carries consent to one use: publication of the membership in the community's trust registry. A consumer **MUST NOT** read a `true` here as agreement to anything else — a directory, a mailing list, a disclosure to another community — just as it may not read `registryConsent` on a join submission that way.

Consent runs both ways for the member, and one way for anyone else. The member may grant and withdraw as often as they like; an administrator may only withdraw, and an administrator who wants a member unlisted for the community's own reasons does so through the community's publication policy (consent permits, it does not oblige) rather than by holding the member's consent at `false`. A member may therefore grant again after an administrator's withdrawal, and Conformance item 2 orders the two by when each was issued.
