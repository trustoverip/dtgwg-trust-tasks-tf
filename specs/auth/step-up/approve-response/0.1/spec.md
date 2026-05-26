---
slug: auth/step-up/approve-response
version: "0.1"
title: Auth — Step-up Approve Response
summary: An approver's signed answer to a step-up approve-request — the proof on this document is the cryptographic gate the relying party uses to elevate the subject's session.
status: draft
targetFrameworkVersion: "0.1"
category: authentication
keywords:
  - auth
  - step-up
  - aal
  - wallet
  - approval
  - consent
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Approver
    requirement: REQUIRED
  - role: Relying party
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED
  rationale: The proof IS the step-up. Without a verified signature from the subject's authoritative key, the relying party has no basis to elevate.
errorCodes:
  - code: auth/step-up/approve-response:challenge_unknown
    meaning: The relying party has no pending step-up matching the echoed challenge.
    retryable: false
  - code: auth/step-up/approve-response:challenge_expired
    meaning: The matching step-up has expired.
    retryable: false
  - code: auth/step-up/approve-response:subject_mismatch
    meaning: The document's issuer (or the proof's verificationMethod DID) does not equal the session's subject.
    retryable: false
  - code: auth/step-up/approve-response:acr_unsatisfied
    meaning: The grantedAcr is below the targetAcr the relying party originally requested.
    retryable: false
related:
  - auth/step-up/approve-request
  - auth/passkey/login/finish
  - auth/refresh
  - auth/whoami
---

## Abstract

The **Auth — Step-up Approve Response** Trust Task is the signed ratification of an earlier [`auth/step-up/approve-request/0.1`](../approve-request/0.1/spec.md). The approver echoes the request's `subject`, `sessionId`, and `challenge`, sets `decision` to `approved` or `denied`, and signs the document with the subject's key. The framework `proof` IS the step-up gate.

A relying party processing an `approved` response elevates the session's `amr`/`acr` per its own policy and replies with the elevated session snapshot. A `denied` response is signed too — it serves as audit evidence that the user explicitly refused.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the approver) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/step-up/approve-response/0.1`, with itself as `issuer` and the relying party as `recipient`.
2. Echo `payload.subject`, `payload.sessionId`, and `payload.challenge` verbatim from the matching approve-request.
3. Set `payload.decision` to `approved` or `denied`. When `denied`, populate `payload.deniedReason`.
4. **MAY** declare `payload.grantedAcr` to convey which AAL the approver believes it demonstrated. Relying parties MAY upgrade the session to ≤ this value; MUST NOT exceed it.
5. Include a `proof` whose `verificationMethod` resolves to a key the subject controls. The `proof.proofPurpose` MUST be `assertionMethod`.

A conforming **consumer** (the relying party) **MUST**:

1. Validate the document and verify the `proof`.
2. Locate the matching pending step-up via `payload.challenge`. Unknown → `challenge_unknown`. Expired → `challenge_expired`.
3. Verify `payload.subject` equals the session's subject AND equals the document's `issuer` AND equals the DID resolved from the proof's `verificationMethod`. Mismatch → `subject_mismatch`.
4. Verify `payload.challenge` equals the bound challenge bit-for-bit (constant-time comparator).
5. When `decision === "approved"`:
   - Apply the session elevation per the consumer's policy: update `session.amr` to include the new factor, raise `session.acr` to at most `payload.grantedAcr`.
   - If the session's `acr` cannot reach the originally-requested `targetAcr` → `acr_unsatisfied`.
   - Consume the step-up so the same approve-response cannot be replayed.
6. When `decision === "denied"`:
   - Consume the step-up.
   - Persist the denied response for audit. Take no further action on the session.

## Definitions

* **Approver.** The party authoritative for the subject; identified by `issuer`.
* **Relying party.** The party that initiated the step-up; identified by `recipient`.
* **Subject.** The VID whose session is being elevated.

## Payload

`payload.subject`, `payload.sessionId`, `payload.challenge`, `payload.decision` — REQUIRED.

`payload.deniedReason` — required when decision is `denied`.

`payload.grantedAcr` — optional approver-declared AAL.

`payload.ext` — extension slot.

## Examples

### Approver approves the transfer

```json
{
  "id": "approve-resp-7890-1234-5678-90abcdef1234",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:bank.example",
  "issuedAt": "2026-05-23T14:00:30Z",
  "payload": {
    "subject": "did:web:alice.example",
    "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
    "challenge": "VHJhbnNmZXJDb25maXJtTm9uY2VYWQ",
    "decision": "approved",
    "grantedAcr": "aal2"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:alice.example#key-1",
    "created": "2026-05-23T14:00:30Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg…"
  }
}
```

### Approver denies

```json
{
  "id": "approve-resp-8901-2345-6789-0abcdef12345",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:bank.example",
  "issuedAt": "2026-05-23T14:00:30Z",
  "payload": {
    "subject": "did:web:alice.example",
    "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
    "challenge": "VHJhbnNmZXJDb25maXJtTm9uY2VYWQ",
    "decision": "denied",
    "deniedReason": "User does not recognize this transfer."
  },
  "proof": { "…": "…" }
}
```

## Response

The relying party's `#response` confirms whether elevation succeeded.

### Successful elevation

```json
{
  "id": "approve-ack-9012-3456-7890-abcdef123456",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.1#response",
  "threadId": "approve-resp-7890-1234-5678-90abcdef1234",
  "issuer": "did:web:bank.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T14:00:31Z",
  "payload": {
    "status": "elevated",
    "session": {
      "id": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
      "subject": "did:web:alice.example",
      "issuedAt": "2026-05-23T10:00:31Z",
      "expiresAt": "2026-05-23T14:30:31Z",
      "amr": ["did", "vta"],
      "acr": "aal2"
    }
  }
}
```

### Elevation rejected

```json
{
  "id": "approve-ack-0123-4567-8901-bcdef1234567",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-response/0.1#response",
  "threadId": "approve-resp-7890-1234-5678-90abcdef1234",
  "issuer": "did:web:bank.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T14:00:31Z",
  "payload": {
    "status": "rejected",
    "reason": "challenge expired"
  }
}
```

## Security & Privacy

**Proof IS the gate.** The relying party MUST NOT take any field in this document as authoritative without a verified proof. A bearer-token-style step-up is not safe — the threat model includes a token-stealing attacker who would happily issue their own approve-response.

**Echo verification.** All three echo fields (`subject`, `sessionId`, `challenge`) MUST be compared bit-for-bit. An attacker who can re-target a captured approve-response to a different session (by mutating `sessionId`) MUST be defeated by the proof — but defense-in-depth: comparing all three fields blocks attacks against weak proof implementations.

**Replay.** Consuming the challenge on success-or-denial is mandatory. A second approve-response carrying the same challenge MUST fail with `challenge_unknown`.

**Denied responses as audit.** A signed `denied` response is valuable evidence — it proves the user actively refused, not that they were absent. Relying parties SHOULD preserve denied responses with the same retention policy as approvals.

**Wallet UX.** Approvers presenting approve-requests to humans MUST display the request's `reason` and the relying party identity verbatim. Substituting a friendlier summary for an unclear reason is a phishing vector.

The optional `ext` extension is part of the producer's signed surface.
