---
slug: vtc/members/step-up-passkey-notice
version: "0.1"
title: "VTC Members — Step-up Passkey Notice"
summary: "The VTC tells a member, unprompted, that a step-up passkey was enrolled for them or revoked from them — what happened, which credential, who acted and when — so a change made by someone else, such as an administrator, is never silent."
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - auth
  - passkey
  - step-up
  - notice
  - account-security
  - webauthn
parties:
  - role: VTC
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: member
    requirement: REQUIRED
    member: recipient
    identifierScope: pairwise
proofRequirement:
  requirement: REQUIRED
  rationale: "The notice is what a member acts on — reviewing their account, revoking a passkey they don't recognise, contacting an administrator — and what they may later point to in a dispute. Without a proof it evidences nothing beyond the transport that carried it."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A notice replayed long after the member already reviewed the change would raise an alarm over something already settled. The member places it in time, and against `at`, to tell whether it is current."
sideEffects:
  level: none
  rationale: "Reports an event already carried out at the VTC; the notice changes nothing at the recipient."
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: "Carries the affected credential's non-secret identifier and, when an administrator gave one, their free-text reason."
retention:
  class: durable
  rationale: "A member keeps the notice as their record of being told; the VTC's audit history is the authoritative record either way."
errorCodes: []
related:
  - auth/passkey/enroll/invite
  - auth/passkey/enroll/finish
  - auth/passkey/revoke/finish
  - auth/passkey/admin-list
  - vtc/members/removal-notice
---

## Abstract

A step-up passkey is the gesture a VTC asks for before it lets a signed document confer elevated authority — the user-verification evidence behind [`auth/step-up/approve-response`](../../../../auth/step-up/approve-response/0.5/spec.md). Enrolling or revoking one is ordinarily self-service, but [`auth/passkey/enroll/invite`](../../../../auth/passkey/enroll/invite/0.2/spec.md) and [`auth/passkey/revoke/start`](../../../../auth/passkey/revoke/start/0.2/spec.md) both let an *administrator* act for a member who cannot act for themselves. That is necessary, and also exactly the shape of a silent account takeover: an attacker with administrator standing — or an administrator fooled into using it — can hand a member's identity a new approver, or strip the one they trust, without the member ever being asked.

This task is how the VTC closes that gap outside its own consoles: it pushes a signed notice to the member the moment a step-up passkey of theirs is enrolled or revoked, naming the event, the credential, who acted and when.

Like [`vtc/members/removal-notice`](../../removal-notice/0.1/spec.md) and [`git-ns/right/break-glass-notice`](../../../../git-ns/right/break-glass-notice/0.1/spec.md), it answers nothing: the recipient did not ask, is not waiting, and may be offline.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

Not consequential for the recipient, and declared for clarity. The notice presupposes that its `issuer` is the VTC governing the member's credential; a recipient **MUST** verify the `proof` against that VTC's DID and **MUST NOT** act on a notice from anyone else. A notice is information, never authority: a member who wants to act on it — list their step-up passkeys, revoke one, ask an administrator what happened — sends the task for that act, which the VTC checks on its own terms.

## Producer requirements

A conforming VTC:

1. Sends a notice of event `enrolled` after a step-up passkey is durably bound to the member ([`auth/passkey/enroll/finish`](../../../../auth/passkey/enroll/finish/0.2/spec.md), including via an invite's redemption), and `revoked` after one is durably unbound ([`auth/passkey/revoke/finish`](../../../../auth/passkey/revoke/finish/0.2/spec.md)) — never on starting either ceremony.
2. Sends it to the member whose credential changed, as its own document with that member as `recipient`, over the VTC's own authenticated messaging channel, queued for guaranteed delivery with a window long enough for a member who is away.
3. Sets `at` to when the event took effect, `credentialId` to the affected credential's durable identifier, and `by` to who acted: the member's own DID when they enrolled or revoked it on their own session, the administrator's DID when they invited the enrollment or revoked for the member.
4. Includes `reason` whenever the acting administrator gave one. Omits it, rather than sending an empty string, when none was given — a community that chose not to explain is itself information.
5. **MUST NOT** carry the credential's public key, attestation, or any other secret or key material — `credentialId` is the opaque, already-non-secret identifier [`auth/passkey/admin-list`](../../../../auth/passkey/admin-list/0.1/spec.md) and `auth/passkey/revoke/start` already use, and nothing here raises its sensitivity.
6. Records in its audit history each notice it could not queue. A failure to notify **MUST NOT** undo the event.

## Consumer requirements

A recipient that shows members alerts **SHOULD** show a notice at once and prominently, with `reason` when present. A recipient **MUST NOT** render `reason` as markup.

A recipient **SHOULD** treat `by` differing from `did` as a prompt to review: the member did not act, so something else did. For `enrolled`, that means an approver now exists for this member's identity that they did not create — the recipient **SHOULD** surface a path to list the member's step-up passkeys ([`auth/passkey/admin-list`](../../../../auth/passkey/admin-list/0.1/spec.md) answers that for an administrator; a member reads their own) and to revoke it. For `revoked`, it means an approver the member relied on is gone — the recipient **SHOULD** surface a path to enroll a replacement before the member next needs to step up.

### An administrator enrolls a step-up passkey for Carol

```json
{
  "id": "urn:uuid:4e9a2c1b-7f3d-4a68-9e12-5b6d8c3a0f71",
  "type": "https://trusttasks.org/spec/vtc/members/step-up-passkey-notice/0.1",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
  "issuedAt": "2026-09-25T09:00:04Z",
  "payload": {
    "event": "enrolled",
    "did": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
    "credentialId": "c3RlcHVwLWNyZWQtY2Fyb2w",
    "by": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
    "at": "2026-09-25T09:00:01Z",
    "reason": "Replacing the device Carol reported lost at last week's standup."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-25T09:00:04Z",
    "proofPurpose": "authentication",
    "proofValue": "z6Rn3Qw9Kb2Tx8Lm4Hc1Np5Fz7Ya3Js6Vg9Ue1Ti8Ob2Qn4Mk6Cr9Pw1El7Gh3Sy5Ad2Bv6Xf9Lj3Tq1Rz8Kn4M"
  }
}
```

### Carol's step-up passkey is revoked without her — the case this task exists for

No `reason` was given: a member receiving this knows the credential is gone, who removed it, and that no reason was recorded, which is itself information.

```json
{
  "id": "urn:uuid:7b1d4f8a-2c90-4e6b-8a35-1f9c6d2b7e40",
  "type": "https://trusttasks.org/spec/vtc/members/step-up-passkey-notice/0.1",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
  "issuedAt": "2026-09-26T14:22:50Z",
  "payload": {
    "event": "revoked",
    "did": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
    "credentialId": "c3RlcHVwLWNyZWQtY2Fyb2w",
    "by": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
    "at": "2026-09-26T14:22:47Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-26T14:22:50Z",
    "proofPurpose": "authentication",
    "proofValue": "z3Jp7Qh1Nx9Lm2Vc8Tf0Wk5Bz4Ya6Rs1Ue9Ti2Ob7Qn0Mk3Cr8Pw5El1Gh6Sy9Ad4Bv2Xf7Lj0Tq3Rz1Kn8M"
  }
}
```

## Security & Privacy

### Data carried

The notice names one credential, the event, who acted and when, and — only when an administrator gave one — a free-text reason. It never carries the credential's public key, attestation object, or signature counter.

### Why `by` is the point

`credentialId` and `at` tell the member *what* happened; `by` tells them whether to be alarmed. A member who enrolled or revoked a step-up passkey themselves, moments earlier, recognises their own DID in `by` and moves on. A member who sees an administrator's DID there — especially for `enrolled` — is looking at the one signal that an approver now speaks for them that they did not create. Nothing in this specification can compel a VTC to send the notice; what it can do is make it verifiable when sent, so its absence is a visible choice rather than a technical limitation, exactly as [`removal-notice`](../../removal-notice/0.1/spec.md) argues for the same reason.

### Correlation

The VTC declares `identifierScope: public`, as the authority whose records these are. The recipient declares `pairwise`.

### Retention

A member keeps the notice for as long as it keeps other account-security records; the VTC's audit history is the authoritative record either way.

### Consent/purpose

The purpose is to make a change to a member's own step-up credential visible to the one person it can't be hidden from. The VTC and the recipient **MUST NOT** use a notice for anything else.

The optional `ext` member is part of the producer's signed surface; producers **MUST NOT** place data in `ext` they would not be comfortable signing.
