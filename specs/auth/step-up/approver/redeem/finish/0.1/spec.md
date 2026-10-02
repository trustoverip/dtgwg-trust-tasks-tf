---
slug: auth/step-up/approver/redeem/finish
version: "0.1"
title: "Auth — Step-up Approver Redeem (finish)"
summary: The invited subject, signing as their own DID, names the approver to bind and carries its signed enrolment statement over the ceremony's challenge; the relying party binds the approver to the subject as their step-up factor and consumes the invite.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - step-up
  - approver
  - invite
  - redeem
  - enrollment
  - proof-of-possession
parties:
  - role: Invited subject
    requirement: REQUIRED
    member: issuer
  - role: Relying party
    requirement: REQUIRED
    member: recipient
  - role: Step-up approver
    requirement: REQUIRED
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    Two signatures are on this request and both are needed. The document's proof is the invited subject's own, so nobody holding an intercepted invite can bind their device to someone else's DID. The embedded statement's proof is the approver's, over the ceremony's challenge, which is the proof of possession: a factor nobody demonstrated holding would be a factor anyone could claim.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed finish would try to bind a factor after the invite was meant to be spent; the enrollmentId is consumed on success and placing the request in time bounds the rest.
sideEffects:
  level: mutating
  rationale: >-
    Binds a step-up approver to the invite's subject and consumes the invite. The binding confers no role, scope or session; it is reversed by auth/step-up/approver/revoke.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries an approver DID, an optional label and the approver's signed statement; the response returns the stored binding. No key material moves either way.
retention:
  class: durable
  rationale: >-
    The binding is kept until revoked, and the redemption — which invite, which administrator, which approver — is the audit record of the anchor the factor was bound on.
errorCodes:
  - code: auth/step-up/approver/redeem/finish:enrollmentNotFound
    meaning: "`enrollmentId` names no open ceremony, or its invite was consumed, voided or expired meanwhile."
    retryable: false
  - code: auth/step-up/approver/redeem/finish:enrollmentExpired
    meaning: "The ceremony's expiry has elapsed. Start again with redeem/start while the invite is still valid."
    retryable: true
  - code: auth/step-up/approver/redeem/finish:notInvitedSubject
    meaning: "The document is not signed, by a verification method of its own DID, by the subject the ceremony was opened for."
    retryable: false
  - code: auth/step-up/approver/redeem/finish:statementInvalid
    meaning: "`statement` is not a valid auth/step-up/approver/attest/0.1 document for this ceremony: its proof does not verify, its issuer is not `approverDid`, its `recipient` or `audience` is not this relying party, its `purpose` is not `enrol`, its `subject` is not the invited subject, its `challenge` is not the ceremony's, its `boundTo` is not `enrollmentId`, it was issued outside the ceremony's lifetime, or its `id` was already spent."
    retryable: false
  - code: auth/step-up/approver/redeem/finish:approverNotDistinct
    meaning: "`approverDid` is the subject's own DID, appears as a verification method in the subject's DID document, is a key the subject signs operations with at the relying party (such as a console signing key), or holds standing of its own there. A step-up factor must be a key the caller does not already hold for signing."
    retryable: false
  - code: auth/step-up/approver/redeem/finish:approverAlreadyBound
    meaning: "`approverDid` is already bound — to this subject or another — or was bound and revoked. An approver DID is bound to at most one subject, once; generate a new one."
    retryable: false
  - code: auth/step-up/approver/redeem/finish:tooManyApprovers
    meaning: "The subject already holds five live approvers. Revoke one (auth/step-up/approver/revoke) first, or replace one with auth/step-up/approver/enroll."
    retryable: false
related:
  - auth/step-up/approver/invite
  - auth/step-up/approver/redeem/start
  - auth/step-up/approver/attest
  - auth/step-up/approver/list
  - auth/step-up/approver/revoke
  - auth/step-up/approver/enroll
---

## Abstract

The second leg of redeeming an [`auth/step-up/approver/invite`](../../../invite/0.1/spec.md). The invited subject signs this document **as their own DID**, naming the approver to bind and carrying the approver's [`auth/step-up/approver/attest/0.1`](../../../attest/0.1/spec.md) statement with `purpose: enrol`, made over the `challenge` the [`start`](../../start/0.1/spec.md) issued and bound to its `enrollmentId`. The relying party verifies both signatures, checks the approver is distinct from every key the subject already holds, binds it to the subject with `enrolledVia: invite` (or `offline`, for an invite the operator minted from the host), and consumes the invite.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the invited subject):

1. **MUST** set `issuer` to its own DID and sign the document with a verification method of that DID.
2. **MUST** carry in `statement` the approver's signed `attest/0.1` document exactly as the approver produced it, with `purpose: enrol`, `subject` its own DID, `audience` and `recipient` the start's `audience`, `challenge` the start's `challenge`, and `boundTo` the `enrollmentId`.
3. **SHOULD** use an approver DID generated for this relying party alone.

A conforming **consumer** (the relying party), in this order:

1. **MUST** look up the ceremony by `enrollmentId` → `enrollmentNotFound` or `enrollmentExpired`.
2. **MUST** require the proof's verification method to belong to `issuer`'s own DID document, and `issuer` to be the ceremony's subject → `notInvitedSubject`. A delegated key acting for the subject does not satisfy this.
3. **MUST** verify `statement` as [`attest/0.1`](../../../attest/0.1/spec.md) requires of its consumer, over the object exactly as received, with its `issuer` equal to `approverDid`, `purpose` `enrol`, `subject` the ceremony's subject, `audience` and `recipient` its own DID, `challenge` the ceremony's and `boundTo` equal to `enrollmentId` → `statementInvalid`.
4. **MUST** refuse with `approverNotDistinct` when `approverDid` is the subject's DID, appears in the subject's DID document as a verification method, is a key the subject signs operations with at the relying party, or holds standing of its own there.
5. **MUST** refuse with `approverAlreadyBound` an approver DID that is bound to any subject or was bound and revoked.
6. **MUST** refuse with `tooManyApprovers` when the subject already holds five live approvers.
7. **MUST** write the binding — `approverDid`, `subject`, `label`, `enrolledAt`, `enrolledVia` — and consume the invite and the ceremony **atomically** with it, serialised with every other write of the subject's approvers, so two concurrent finishes can neither both pass the cap nor bind one approver twice.
8. **MUST** record an audit event naming the subject, the approver DID, the inviting administrator (or the host, for `offline`) and `enrolledVia`.
9. **SHOULD** tell the subject, over a channel it already has with them, that an approver was bound to their DID.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authority this task presupposes is **the invite**: the issuing administrator's authority over the subject, carried by the invite and the open ceremony named by `enrollmentId`. The subject's signature establishes that the invited subject — and no one holding an intercepted invite — is the one binding a factor; the statement establishes that the factor's key is held by whoever is binding it. Neither signature is the authority, and the subject's signing key is deliberately not enough: binding a second factor on the strength of the first would add nothing.

## Definitions

- **`enrollmentId`** — from redeem/start, echoed verbatim.
- **`approverDid`** — the step-up approver to bind, an Ed25519 `did:key`.
- **`label`** — a name for where the approver lives; overrides the inviter's suggestion.
- **`statement`** — the approver's signed `attest/0.1` document with `purpose: enrol`, its proof of possession.

## Request

The invited subject (`issuer`) sends the request to the relying party (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Alice binds her browser plugin's approver

```json
{
  "id": "urn:uuid:9c4e7a3b-2d5f-4b0c-8e1a-6f3a4b5c7d11",
  "type": "https://trusttasks.org/spec/auth/step-up/approver/redeem/finish/0.1",
  "issuer": "did:webvh:QmAliceScid4:wallet.example:alice",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-01T15:04:05Z",
  "threadId": "urn:uuid:9c4e7a3b-2d5f-4b0c-8e1a-6f3a4b5c7d1f",
  "payload": {
    "enrollmentId": "enr_5b0e3a2e6f4c4b8f9d2a1f0c2b7e4a01",
    "approverDid": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
    "label": "Browser plugin — work laptop",
    "statement": {
      "id": "urn:uuid:3a6c1e0b-9d2f-4b7a-8c15-2e4f6a8b0c02",
      "type": "https://trusttasks.org/spec/auth/step-up/approver/attest/0.1",
      "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
      "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
      "issuedAt": "2026-10-01T15:04:00Z",
      "payload": {
        "purpose": "enrol",
        "subject": "did:webvh:QmAliceScid4:wallet.example:alice",
        "audience": "did:webvh:QmVtcScid7:acme-vtc.example",
        "challenge": "RW5yb2xDaGFsbGVuZ2VOb25jZTAxMjM0NTY",
        "boundTo": "enr_5b0e3a2e6f4c4b8f9d2a1f0c2b7e4a01"
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH#z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
        "created": "2026-10-01T15:04:00Z",
        "proofPurpose": "authentication",
        "proofValue": "z4mD…"
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmAliceScid4:wallet.example:alice#key-1",
    "created": "2026-10-01T15:04:05Z",
    "proofPurpose": "authentication",
    "proofValue": "z5Lg…"
  }
}
```

## Response

The relying party answers with the sub-schema reachable via `$anchor: "response"`: the binding as stored. Refusals use `trust-task-error`.

### The approver is bound

```json
{
  "id": "urn:uuid:9c4e7a3b-2d5f-4b0c-8e1a-6f3a4b5c7d12",
  "type": "https://trusttasks.org/spec/auth/step-up/approver/redeem/finish/0.1#response",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmAliceScid4:wallet.example:alice",
  "issuedAt": "2026-10-01T15:04:06Z",
  "threadId": "urn:uuid:9c4e7a3b-2d5f-4b0c-8e1a-6f3a4b5c7d1f",
  "payload": {
    "approver": {
      "approverDid": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
      "subject": "did:webvh:QmAliceScid4:wallet.example:alice",
      "label": "Browser plugin — work laptop",
      "enrolledAt": "2026-10-01T15:04:06Z",
      "enrolledVia": "invite"
    }
  }
}
```

## Security & Privacy

**Consumption is atomic with binding.** An invite that stayed redeemable after an approver was bound would let a second approver follow the first on the same two messages.

**Distinctness is checked here and again at every use.** A subject's DID document can change after enrolment; an approver that later appears in it, or is later enrolled as a console key, stops counting (see [`attest/0.1`](../../../attest/0.1/spec.md) Conformance item 6).

**A burned approver stays burned.** Refusing to rebind a revoked approver DID means a key revoked as compromised cannot be re-bound by someone who later obtains an invite.

**Free text.** `label` is bounded at 64 characters, authored by the subject and untrusted; it is retained with the binding and shown to whoever may list or revoke it.

### Data carried

The request carries an approver DID, an optional label, and the approver's statement naming the subject, the relying party, the ceremony challenge and the `enrollmentId`. The response echoes the stored binding. No key material moves. A producer **SHOULD NOT** put a person's name or any identifier in `label` beyond what distinguishes one device from another.

### Correlation

The approver DID becomes a stable handle linking the subject's future step-ups at this relying party, which is its purpose. Used at one relying party only, it links nothing elsewhere.

### Retention

The binding is kept until revoked and then as a tombstone, so the approver DID cannot be bound again. The redemption's audit record is durable.

### Consent/purpose

The approver is bound for one purpose: to be the subject's step-up factor at this relying party. It **MUST NOT** be accepted to sign in, to open or elevate a session other than through a step-up asked of the subject, or as an approver for anyone else.
