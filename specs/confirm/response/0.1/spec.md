---
slug: confirm/response
version: "0.1"
title: Confirm — Response
summary: An approver's signed answer to a confirm/request — the proof on this document is the cryptographic record of the user's decision.
status: draft
targetFrameworkVersion: "0.1"
category: identity
keywords:
  - confirm
  - consent
  - wallet
  - approval
  - decision
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Approver
    requirement: REQUIRED
  - role: Relying party
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED
  rationale: The proof IS the consent record. Without a verified signature from the subject's authoritative key, the relying party has no evidence the user actually approved.
errorCodes:
  - code: confirm/response:challenge_unknown
    meaning: The relying party has no pending confirm/request matching the echoed challenge.
    retryable: false
  - code: confirm/response:challenge_expired
    meaning: The matching confirm/request has expired.
    retryable: false
  - code: confirm/response:subject_mismatch
    meaning: The document's issuer (or proof verificationMethod DID) does not equal the requested subject.
    retryable: false
related:
  - confirm/request
  - auth/step-up/approve-response
---

## Abstract

The **Confirm — Response** Trust Task is the signed ratification of a [`confirm/request/0.1`](../../request/0.1/spec.md). The approver echoes the request's `subject` and `challenge`, sets `decision`, and signs the document with the subject's key.

A relying party processing an `approved` response proceeds with the gated action and acknowledges via `#response: { status: "recorded" }`. A `denied` response is signed too — it serves as audit evidence that the user actively refused.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the approver) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/confirm/response/0.1`, with itself as `issuer` and the relying party as `recipient`.
2. Echo `payload.subject` and `payload.challenge` verbatim from the matching confirm/request.
3. Set `payload.decision`. When `denied`, populate `payload.deniedReason`.
4. Include a `proof` whose `verificationMethod` resolves to a key the subject controls; `proofPurpose: assertionMethod`.

A conforming **consumer** (the relying party) **MUST**:

1. Validate the document and verify the `proof`.
2. Locate the matching pending confirm/request via `payload.challenge`. Unknown → `challenge_unknown`. Expired → `challenge_expired`.
3. Verify `payload.subject` equals the request's subject AND equals the document's `issuer` AND equals the DID resolved from the proof's `verificationMethod`. Mismatch → `subject_mismatch`.
4. Verify `payload.challenge` equals the bound challenge bit-for-bit.
5. Consume the request so the same response cannot be replayed.
6. Persist the response for audit regardless of decision.
7. Return `#response: { status: "recorded" }` on success.

## Payload

`payload.subject`, `payload.challenge`, `payload.decision` — REQUIRED.

`payload.deniedReason` — required when decision is `denied`.

`payload.ext` — extension slot.

## Examples

### Approver approves the transfer

```json
{
  "id": "confirm-resp-7890-1234-5678-90abcdef1234",
  "type": "https://trusttasks.org/spec/confirm/response/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:bank.example",
  "issuedAt": "2026-05-23T18:00:30Z",
  "payload": {
    "subject": "did:web:alice.example",
    "challenge": "VHJhbnNmZXJDb25maXJtTm9uY2VYWQ",
    "decision": "approved"
  },
  "proof": { "…": "…" }
}
```

### Approver denies

```json
{
  "id": "confirm-resp-8901-2345-6789-0abcdef12345",
  "type": "https://trusttasks.org/spec/confirm/response/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:bank.example",
  "issuedAt": "2026-05-23T18:00:30Z",
  "payload": {
    "subject": "did:web:alice.example",
    "challenge": "VHJhbnNmZXJDb25maXJtTm9uY2VYWQ",
    "decision": "denied",
    "deniedReason": "User does not recognize this transfer."
  },
  "proof": { "…": "…" }
}
```

## Response

```json
{
  "id": "confirm-ack-9012-3456-7890-abcdef123456",
  "type": "https://trusttasks.org/spec/confirm/response/0.1#response",
  "threadId": "confirm-resp-7890-1234-5678-90abcdef1234",
  "issuer": "did:web:bank.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T18:00:31Z",
  "payload": {
    "status": "recorded"
  }
}
```

## Security & Privacy

**Proof IS the consent record.** Treat every other field as advisory; only the verified proof binds the user's decision to their key. A denied response is just as cryptographically meaningful — it proves the user actively refused, not that they were absent.

**Replay.** Consuming the challenge on success-or-denial is mandatory.

**Audit retention.** Confirm responses (approved AND denied) MUST be retained per the relying party's compliance regime. A denied response showing the user actively refused is often more legally significant than an approval.

**Wallet UX.** Approvers presenting requests to humans MUST display the request's `reason` verbatim. Substituting a friendlier summary is a phishing vector.

The optional `ext` extension is part of the producer's signed surface.
