---
slug: confirm/request
version: "0.1"
title: Confirm — Request
summary: A relying party asks a wallet (or any consent agent) to obtain user confirmation for a specific action; the wallet returns a signed confirm/response that is the cryptographic record of the user's decision.
status: draft
targetFrameworkVersion: "0.1"
category: identity
keywords:
  - confirm
  - consent
  - wallet
  - approval
  - transaction
  - rp
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Relying party
    requirement: REQUIRED
  - role: Approver
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED
  rationale: The `reason` is displayed to the user as the basis of consent. A proof binds the request — including the reason — to the relying party's key, so a man-in-the-middle cannot substitute a different reason while leaving the signed envelope intact.
errorCodes:
  - code: confirm/request:subject_unknown
    meaning: The approver does not speak for the named subject.
    retryable: false
  - code: confirm/request:rate_limited
    meaning: The relying party has exceeded the approver's confirm-request budget.
    retryable: true
related:
  - confirm/response
  - auth/step-up/approve-request
  - auth/step-up/approve-response
---

## Abstract

The **Confirm — Request** Trust Task is the generic "ask the user to approve something" wire form. A *relying party* sends this to a *wallet* (or any approval agent acting for a subject) asking for explicit user consent; the wallet surfaces the request to the user verbatim and returns a signed [`confirm/response/0.1`](../../response/0.1/spec.md) carrying the user's decision.

This pair (`confirm/request` + `confirm/response`) is intentionally less coupled than the [`auth/step-up/approve-*`](../../auth/step-up/approve-request/0.1/spec.md) pair: step-up is specifically about elevating an authenticated session's AAL; confirm is the broader "the user must consent before we take action X" pattern that applies to transactions, data shares, ACL edits, irreversible operations, anything where capturing explicit consent matters for compliance or audit.

When the action being confirmed IS an AAL elevation, use the step-up pair. For everything else, use confirm.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the relying party) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/confirm/request/0.1`, with itself as `issuer` and the approver as `recipient`.
2. Populate `payload.subject` with the VID whose consent is sought.
3. Generate `payload.challenge` with ≥128 bits of entropy and bind it server-side to `(subject, action, expiresAt)`.
4. Populate `payload.reason` with a user-meaningful description of the action.
5. **MAY** populate `payload.actionType` (machine-readable category) and `payload.actionDetails` (structured data the wallet MAY surface).
6. Include a verified `proof` so the approver can rely on the `recipient` as authoritative.

A conforming **consumer** (the approver) **MUST**:

1. Verify the document's `proof`.
2. Determine whether it speaks for `payload.subject`. If not → `subject_unknown`.
3. Surface the request to the user (or apply policy-bound delegation per the approver's own rules). When a human reviews, the wallet MUST display `payload.reason` verbatim. The relying party's `issuer` SHOULD be displayed (e.g. "bank.example wants to:").
4. Return a `#response` document carrying `status: accepted` (will return a confirm/response asynchronously) or `status: refused` (with a `reason`).

The confirm/response arrives out-of-band via the approver's preferred transport.

## Payload

`payload.subject`, `payload.challenge`, `payload.reason` — REQUIRED.

`payload.actionType`, `payload.actionDetails`, `payload.ttl` — optional.

`payload.ext` — extension slot.

## Examples

### RP requests transfer confirmation

```json
{
  "id": "confirm-req-1234-5678-90ab-cdef12345678",
  "type": "https://trusttasks.org/spec/confirm/request/0.1",
  "issuer": "did:web:bank.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T18:00:00Z",
  "payload": {
    "subject": "did:web:alice.example",
    "challenge": "VHJhbnNmZXJDb25maXJtTm9uY2VYWQ",
    "reason": "Confirm transfer of $1,000 to did:web:bob.example",
    "actionType": "payment.transfer",
    "actionDetails": {
      "amount": "1000",
      "currency": "USD",
      "beneficiary": "did:web:bob.example"
    },
    "ttl": 180
  },
  "proof": { "…": "…" }
}
```

### RP requests data-share confirmation

```json
{
  "id": "confirm-req-2345-6789-01bc-def234567890",
  "type": "https://trusttasks.org/spec/confirm/request/0.1",
  "issuer": "did:web:partner.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T18:05:00Z",
  "payload": {
    "subject": "did:web:alice.example",
    "challenge": "RGF0YVNoYXJlTm9uY2VYWVo",
    "reason": "Share your verified employment status with partner.example for 30 days",
    "actionType": "data.share",
    "actionDetails": {
      "claims": ["employment.status"],
      "durationDays": 30
    }
  },
  "proof": { "…": "…" }
}
```

## Response

The `#response` is a synchronous acknowledgement. The actual signed user decision is the separate [`confirm/response/0.1`](../../response/0.1/spec.md) document that arrives out-of-band.

### Approver accepts to surface

```json
{
  "id": "confirm-req-ack-3456-7890-12cd-ef3456789012",
  "type": "https://trusttasks.org/spec/confirm/request/0.1#response",
  "threadId": "confirm-req-1234-5678-90ab-cdef12345678",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:bank.example",
  "issuedAt": "2026-05-23T18:00:01Z",
  "payload": {
    "status": "accepted"
  }
}
```

## Security & Privacy

**Reason integrity.** The `reason` is the user's basis for consent. The framework `proof` covers it; consumers MUST verify the proof BEFORE surfacing the reason — never display unverified content as if the RP had asserted it.

**actionDetails handling.** The wallet's surfacing of `actionDetails` is consumer-defined. A simple wallet might just display the JSON; a richer one applies type-specific UI (e.g. recognize `actionType: "payment.transfer"` and render a transaction card). Both are valid; the framework imposes no UI requirement.

**Challenge entropy.** ≥128 bits is the floor. The confirm/response signs over the challenge, so it must be unguessable.

**TTL.** The RP's server-side expiry is authoritative — `payload.ttl` is advisory. A late confirm/response is rejected by the RP regardless of the wallet's TTL view.

**Privacy.** `actionDetails` may carry sensitive information. Transport-level confidentiality is REQUIRED.

The optional `ext` extension is part of the signed surface.
