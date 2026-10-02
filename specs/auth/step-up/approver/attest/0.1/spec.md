---
slug: auth/step-up/approver/attest
version: "0.1"
title: "Auth — Step-up Approver Attest"
summary: A step-up approver's signed statement — for one subject, at one relying party, over one challenge, bound to one operation or enrolment — carried embedded in the document it backs, never sent on its own.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - step-up
  - approver
  - re-authentication
  - second-factor
  - attestation
parties:
  - role: Step-up approver
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Relying party
    requirement: REQUIRED
    member: recipient
    identifierScope: public
subjectPath: /subject
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The proof is the whole of what this document contributes. It is the approver key's signature over one challenge for one subject, and it is the step-up factor itself: a relying party that accepted a proofless statement would accept a factor nobody demonstrated holding. It is made for `authentication`, by the approver's own `did:key`, and it is verified over the document exactly as received.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The statement is accepted only inside the lifetime of the pending step-up or enrolment it answers, and that comparison needs an issue time. A statement that could not be placed in time could be held back and spent whenever its challenge was still parked.
sideEffects:
  level: none
  rationale: >-
    The document does nothing on its own. It is evidence carried inside another task — auth/step-up/approve-response/0.6, task-consent/decision/0.2, auth/step-up/approver/redeem/finish/0.1, auth/step-up/approver/enroll/0.1, vtc/install/claim/finish/0.3 — and the effect, if any, is that task's.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The payload names the subject, the relying party, a challenge the relying party issued and the opaque value the statement is bound to. Nothing secret moves; the approver's private key signs and stays where it is.
retention:
  class: durable
  rationale: >-
    The relying party keeps the statement with the record it satisfied — the step-up mark, or the approver binding — as the evidence of which factor answered, and keeps its `id` for as long as its acceptance window lasts so it cannot be spent twice.
errorCodes: []
related:
  - auth/step-up/approve-request
  - auth/step-up/approve-response
  - auth/step-up/approver/invite
  - auth/step-up/approver/redeem/start
  - auth/step-up/approver/redeem/finish
  - auth/step-up/approver/enroll
  - auth/step-up/approver/list
  - auth/step-up/approver/revoke
  - vtc/install/claim/finish
  - task-consent/decision
---

## Abstract

A **step-up approver** is a `did:key` (Ed25519) bound at a relying party to exactly one subject DID as that subject's step-up factor — for example the approver identity a browser plugin holds, whose seed is unlocked only by a user gesture. It confers nothing. It is distinct from the subject's DID keys and from every key the subject signs operations with, and its signature over a challenge the relying party issued for one operation is the subject's own additional factor.

This specification defines that signature. An `attest` document is issued and signed by the approver, addressed to the relying party, and states exactly one thing: *for this subject, at this relying party, over this challenge, I attest to this purpose, bound to this value*. It is never sent on its own. It travels embedded, as an open object, inside the document it backs:

| Purpose | Carried in | `challenge` | `boundTo` |
|---|---|---|---|
| `stepUp` | [`auth/step-up/approve-response/0.6`](../../../approve-response/0.6/spec.md) `evidence.statement` | the approve-request's `challenge` | the approve-request's `boundTo` — the relying party's salted wire digest of the operation |
| `decision` | [`task-consent/decision/0.2`](../../../../../task-consent/decision/0.2/spec.md) `evidence.statement` (`kind: approverSigned`) | the decision's `challenge` | the decision's `payloadDigest` |
| `enrol` | [`redeem/finish/0.1`](../../redeem/finish/0.1/spec.md), [`enroll/0.1`](../../enroll/0.1/spec.md), [`vtc/install/claim/finish/0.3`](../../../../../vtc/install/claim/finish/0.3/spec.md) | the challenge the enrolment's start issued | the enrolment id — `enrollmentId`, `claimId`, or the terms digest [`enroll/0.1`](../../enroll/0.1/spec.md) defines |

The purpose is signed so that a statement made for one use is refused at another: an enrolment statement is never a step-up, and a step-up statement never enrols.

For `decision`, `subject` is the DID that makes the decision document's proof (the approver of the consent request, not the requester), and `audience` and `recipient` are the executor the decision is addressed to. The executor verifies the statement as [`task-consent/decision/0.2`](../../../../../task-consent/decision/0.2/spec.md) requires, and reports a failure as `task-consent/decision:evidenceInvalid`.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the approver):

1. **MUST** set `issuer` to its own `did:key`, `recipient` and `payload.audience` to the relying party's DID, and sign the document with that key, `proofPurpose: authentication`.
2. **MUST** copy `subject`, `challenge` and `boundTo` from the pending step-up or enrolment it is answering, verbatim, and set `purpose` to the one the carrying task requires.
3. **MUST** sign only after the user-verification gesture that makes its key usable. That gesture is what makes the signature an additional factor rather than a second signature by the same party; an approver whose key signs without one is a signing key, not a factor.
4. For `stepUp`, **SHOULD** recompute the relying party's operation digest from the operation it is shown and refuse to sign when it differs from `boundTo`, so the human approves the bytes the relying party will execute.
5. **MUST NOT** send the document on its own. It is embedded in the document it backs.

A conforming **consumer** (the relying party, processing the carrying document):

1. **MUST NOT** act on an `attest` document received on its own, and **SHOULD** answer one with `unsupportedType`.
2. **MUST** verify an embedded `attest` document as a Trust Task document in its own right — the [SPEC §7.2](/SPEC.md#72-consumer-requirements) pipeline over the object exactly as received, with no re-serialisation: its `type`, its proof, the binding of the proof's `verificationMethod` DID to `issuer` ([SPEC §4.8](/SPEC.md#48-the-issuer-and-recipient-members)), `proofPurpose: authentication`, and its acceptance window on `issuedAt` and `expiresAt`.
3. **Audience.** **MUST** require `recipient` and `payload.audience` both to equal its own DID. A statement made for one relying party is refused at another, even if a challenge ever collided.
4. **Binding.** **MUST** require `payload.purpose` to be the purpose the carrying task names, and `payload.subject`, `payload.challenge` and `payload.boundTo` each to equal, bit for bit, the value bound in its own pending record — the step-up mark or the enrolment ceremony — never a value taken from the carrying document. **MUST** require `issuedAt` to fall within that record's lifetime.
5. **Approver.** **MUST** require `issuer` to be a live step-up approver bound to `payload.subject` — or, for `enrol`, the approver DID the carrying task is enrolling, which is not yet bound to anyone.
6. **Distinctness.** **MUST** require `issuer` to differ from `payload.subject`, to appear as no verification method in the subject's DID document, and to be no key the subject can sign operations with at the consumer (a signing-key delegation, a console key) — checked at enrolment and again at every use.
7. **MUST** record the document's `id` against reuse for at least its acceptance window ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 11).
8. **MUST** report any failure of items 2–7 as the carrying task's own refusal (`auth/step-up/approve-response:statementInvalid`, `auth/step-up/approver/redeem/finish:statementInvalid`, `task-consent/decision:evidenceInvalid`, and so on). This task declares no codes of its own, because it is never the request being answered.

## Definitions

- **Approver** — the `did:key` issuing the statement; the subject's step-up factor at this relying party.
- **`purpose`** — what the statement is for: `stepUp` (a re-authentication bound to one operation), `decision` (the factor backing a consent decision, carried in `task-consent/decision/0.2`), `enrol` (proof of possession at enrolment).
- **`subject`** — the one subject DID the approver acts for.
- **`audience`** — the relying party's DID. Restated in the payload, beside the envelope `recipient`, so the binding the approver signed is part of the payload a verifier compares and a typed binding exposes.
- **`challenge`** — the relying party's challenge for this ceremony, at least 128 bits of entropy.
- **`boundTo`** — the opaque value the statement is bound to, compared by equality (table above). For `stepUp` and `decision` it is a digest; for `enrol` an identifier or a digest, per the carrying task.

## Request

The approver (`issuer`) signs the document for the relying party (`recipient`); the subject's agent carries it inside the document it backs. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json). There is no response: this is an embedded attestation, answered — if at all — by the carrying task.

### A browser plugin's approver attests a step-up bound to one grant

```json
{
  "id": "urn:uuid:3a6c1e0b-9d2f-4b7a-8c15-2e4f6a8b0c01",
  "type": "https://trusttasks.org/spec/auth/step-up/approver/attest/0.1",
  "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-02T09:00:00Z",
  "payload": {
    "purpose": "stepUp",
    "subject": "did:webvh:QmAliceScid4:wallet.example:alice",
    "audience": "did:webvh:QmVtcScid7:acme-vtc.example",
    "challenge": "R3JhbnRBZG1pbk5vbmNlWFlaMTIzNDU2",
    "boundTo": "uEiD3n4bE1QbQ8nY2l0cGhYc2FsdGVkZGlnZXN0"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH#z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
    "created": "2026-10-02T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

### The same approver proves possession at enrolment

```json
{
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
```

## Security & Privacy

**Why a signature can be a factor.** A re-authentication must be the caller's *own additional* factor; a signature by a key the caller already holds for signing is not one. An approver's signature counts only because its key is not such a key: it is distinct from every key that can sign the subject's operations (Conformance item 6), held where using it takes user verification (producer item 3), and bound to the subject by an enrolment that rested on an anchor independent of the subject's signing key ([`invite`](../../invite/0.1/spec.md), [`enroll`](../../enroll/0.1/spec.md), [`vtc/install/claim/finish/0.3`](../../../../../vtc/install/claim/finish/0.3/spec.md)). Remove any of the three and the statement is a second signature by the same party.

**User verification is not provable here.** A WebAuthn assertion carries a UV flag; this statement does not. A relying party accepting it trusts, from enrolment, that the approver key is held behind a gesture. That trust is a policy decision the relying party makes about the approver implementations it enrols.

**Verified as received.** The proof covers the document's members exactly as the approver serialised them. A carrier that parsed the statement into a typed structure and re-serialised it could drop or reorder members and break a valid signature, which is why every carrying task holds it as an open object.

### Data carried

The subject's DID, the relying party's DID, a single-use challenge, an opaque binding value and the purpose — nothing else, and no free text. A digest `boundTo` reveals nothing about the operation to anyone without the operation and the challenge it is salted with.

### Correlation

The approver DID links every statement it makes to the one subject it is bound to, at the one relying party — which is the binding's purpose, and why the approver party is declared with identifier scope `pairwise`: a producer **SHOULD** use a distinct approver DID at each relying party, so communities cannot join a subject's activity by it. The relying party is declared with identifier scope `public` because the approver must recognise the relying party it binds the statement to, and a pairwise identifier for it would leave `audience` naming nothing a verifier could compare with itself — the statement is meaningful only at the relying party that bound the approver. The challenge is single-use and a salted `boundTo` is unlinkable across requests.

### Retention

The relying party keeps the statement with the record it satisfied, as the evidence of which factor answered, and keeps its `id` for at least its acceptance window so it cannot be replayed into a second ceremony.

### Consent/purpose

The statement attests one purpose, for one subject, over one challenge, and is exhausted by the ceremony it answers. Whether a relying party requires a step-up for a given operation, and which evidence it accepts, is its own policy; this specification describes the evidence and does not state when it is needed.
