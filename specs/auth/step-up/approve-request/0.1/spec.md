---
slug: auth/step-up/approve-request
version: "0.1"
title: Auth — Step-up Approve Request
summary: A relying party asks a wallet or verifiable-trust agent to ratify an authentication step-up — issuing a challenge that the approver will sign in the follow-up approve-response.
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
  - role: Relying party
    requirement: REQUIRED
  - role: Approver
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED
  rationale: The reason text is shown to the user as the basis of their consent decision. A proof binds the request to the relying party so a downstream attacker cannot intercept the channel and substitute a different reason.
errorCodes:
  - code: auth/step-up/approve-request:subject_unknown
    meaning: The approver does not speak for the named subject.
    retryable: false
  - code: auth/step-up/approve-request:method_unsupported
    meaning: The approver cannot deliver an approve-response (e.g. the wallet has no key for the subject, or doesn't support the requested AAL).
    retryable: false
  - code: auth/step-up/approve-request:user_declined
    meaning: The user reviewed the request and declined consent.
    retryable: false
  - code: auth/step-up/approve-request:rate_limited
    meaning: The relying party has exceeded the approver's request budget.
    retryable: true
related:
  - auth/step-up/approve-response
  - auth/passkey/login/finish
  - auth/refresh
  - auth/whoami
---

## Abstract

The **Auth — Step-up Approve Request** Trust Task is the first half of an out-of-band step-up flow. The relying party (typically an auth service holding a subject's session at AAL 1) sends this document to an *approver* — usually the subject's wallet or a Verifiable-Trust Agent acting for the subject — asking the approver to ratify an AAL elevation.

The approver SHOULD show the `reason` to a human and obtain consent. If consent is granted, the approver returns an [`auth/step-up/approve-response/0.1`](../approve-response/0.1/spec.md) signed by the subject's key. That signed response is the cryptographic gate the relying party uses to elevate the session.

This pair (`approve-request` + `approve-response`) is the canonical "trust task instead of a side channel" pattern. The same shape can support transaction-confirmation flows, high-value-operation gates, and admin-takes-over-from-subject ceremonies.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the relying party) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/step-up/approve-request/0.1`, with itself as `issuer` and the approver as `recipient`.
2. Populate `payload.subject` with the VID whose session is being elevated.
3. Populate `payload.sessionId` with the session id (opaque to the approver but echoed back in the approve-response so the relying party can correlate).
4. Generate `payload.challenge` with ≥128 bits of entropy and bind it server-side to `(subject, sessionId, expiresAt)`.
5. Populate `payload.reason` with a user-meaningful explanation. The approver MAY refuse with `user_declined` if the reason is empty or generic.
6. **MAY** declare `payload.targetAcr` — the AAL the relying party expects on completion.
7. Include a verified `proof` so the approver can rely on the request's `recipient` as authoritative.

A conforming **consumer** (the approver) **MUST**:

1. Verify the document's `proof`.
2. Determine whether it speaks for `payload.subject`. If not → `subject_unknown`.
3. Decide whether to surface the request to the user (subject of consent) or to ratify it programmatically (policy-bound delegation). The framework leaves this to the approver — but if a human is presented with the request, the `reason` MUST be shown verbatim.
4. Return a `#response` document carrying `status: accepted` (will return an approve-response asynchronously) or `status: refused` (with a `reason`).

The approve-response document arrives out-of-band — typically via the approver's preferred transport (DIDComm push to the relying party's mediator, or a push channel the relying party registered at request time).

## Definitions

* **Relying party.** The party requesting the elevation; identified by `issuer` and verified via `proof`.
* **Approver.** The party authoritative for `payload.subject`; identified by `recipient`. Wallets and VTAs are typical.
* **Subject.** The VID whose session is being elevated.
* **Session.** The session the relying party holds for the subject; the approver does not need to know its contents — only the opaque `sessionId`.

## Payload

`payload.subject`, `payload.sessionId`, `payload.challenge`, `payload.reason` — all REQUIRED.

`payload.targetAcr`, `payload.ttl` — optional hints.

`payload.ext` — extension slot per [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member).

## Examples

### Relying party asks the user's wallet to confirm a transfer

```json
{
  "id": "step-up-1234-5678-90ab-cdef12345678",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-request/0.1",
  "issuer": "did:web:bank.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-05-23T14:00:00Z",
  "payload": {
    "subject": "did:web:alice.example",
    "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
    "challenge": "VHJhbnNmZXJDb25maXJtTm9uY2VYWQ",
    "reason": "Confirm transfer of $1,000 to did:web:bob.example",
    "targetAcr": "aal2",
    "ttl": 120
  },
  "proof": { "…": "…" }
}
```

## Response

The `#response` is a synchronous acknowledgement that the approver received the request — NOT the approval itself. The approve-response (signed by the subject) follows out-of-band.

### Approver accepts

```json
{
  "id": "step-up-resp-3456-7890-1234-567890abcdef",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-request/0.1#response",
  "threadId": "step-up-1234-5678-90ab-cdef12345678",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:bank.example",
  "issuedAt": "2026-05-23T14:00:01Z",
  "payload": {
    "status": "accepted"
  }
}
```

### Approver refuses

```json
{
  "id": "step-up-resp-4567-8901-2345-67890abcdef0",
  "type": "https://trusttasks.org/spec/auth/step-up/approve-request/0.1#response",
  "threadId": "step-up-1234-5678-90ab-cdef12345678",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:bank.example",
  "issuedAt": "2026-05-23T14:00:01Z",
  "payload": {
    "status": "refused",
    "reason": "User declined"
  }
}
```

## Security & Privacy

**Reason integrity.** The `reason` is shown to the user as the basis of their consent. An attacker who can substitute the reason while leaving the rest of the request intact can elicit consent for an action the user did not intend. The framework `proof` binds the request — including the reason — to the relying party's key; consumers MUST verify the proof BEFORE surfacing the reason.

**Challenge entropy.** The approve-response signs over the challenge. ≥128 bits is the minimum; ≥192 bits is RECOMMENDED for high-value flows.

**Out-of-band binding.** The approve-response arrives over a channel the approver chose, not the request channel. Consumers correlating request↔response use the document's `threadId` (which equals the request id) AND verify the embedded `challenge` matches what they sent.

**TTL semantics.** `payload.ttl` is an advisory cap from the relying party. The relying party's server-side state is authoritative: when the relying party's expiry fires, any later approve-response is rejected regardless of the approver's view.

**Privacy of reason.** The reason may carry sensitive information (transfer amounts, account details, beneficiary identities). Consumers MUST require transport confidentiality.

The optional `ext` extension is part of the signed surface.
