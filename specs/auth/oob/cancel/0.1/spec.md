---
slug: auth/oob/cancel
version: "0.1"
title: "Auth — OOB Cancel"
summary: "The starter or the lock holder of an out-of-band request ends it; a cancelled request is final and can never be approved or redeemed."
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords: [auth, oob, out-of-band, sign-in, cancel, decline]
parties:
  - role: Starter or approver
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Service
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Only the starter key or the lock key may end a request. Anyone who has seen the code knows the handle, so without a proof any bystander could cancel any member's sign-in.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task changes request state, so it is consequential and SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: mutating
  rationale: >-
    Moves the request to cancelled, which is final. Nothing that had already happened is undone: a consumed request has a session, and cancel does not end it.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Carries a handle under a throwaway key; the response is a status.
retention:
  class: exchange
  rationale: >-
    The cancellation is the request's final state and is audited as an event; nothing else is kept.
errorCodes:
  - code: auth/oob:requestNotFound
    meaning: No request has this `requestId`.
    retryable: false
  - code: auth/oob:requestExpired
    meaning: The request has expired or was already consumed.
    retryable: false
  - code: auth/oob:alreadyDecided
    meaning: The request has already been approved or declined.
    retryable: false
  - code: auth/oob:notAuthorized
    meaning: The cancel is signed by neither the starter key nor the lock key.
    retryable: false
related:
  - auth/oob/request
  - auth/oob/claim
  - auth/oob/redeem
---

## Abstract

Either party with a key in a request can end it: the starter (a person clicks Cancel on the portal, for example after seeing that a stranger claimed the code) or the lock holder (a member taps Decline before proving). The request moves to `cancelled`, which is final; the starter's next `redeem` gets `auth/oob/redeem:declined`.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** **MUST** sign with the starter key or the lock key of the request. A lock holder **SHOULD** set `parentThreadId` to the `requestId` ([SPEC §4.9.2](/SPEC.md#492-the-parentthreadid-member)).

A conforming **service**:

1. **MUST** refuse a cancel signed by neither the starter key nor the lock key with `auth/oob:notAuthorized`.
2. **MUST** move a `pending`, `claimed` or `identified` request to `cancelled` in one compare-and-set and answer `status: cancelled`; **MUST** answer the same for a request already `cancelled`.
3. **MUST** refuse a request that is `approved` or `declined` with `auth/oob:alreadyDecided`, and one that is `expired` or `consumed` with `auth/oob:requestExpired`. A cancel never ends a session already created.

## Authorization

Holding one of the request's two keys — the starter key from `request`, or the lock from `claim` — is the entitlement. The proof shows which key signed; the service compares it with the keys it stored.

## Definitions

- **`requestId`** — the request to end.
- **`status`** (response) — always `cancelled`.

## Request

The starter or lock holder (`issuer`) asks the service (`recipient`) to end the request. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member declines before proving

```json
{
  "id": "urn:uuid:b0718293-a4b5-4fc0-8132-5d6e7f8a9b01",
  "type": "https://trusttasks.org/spec/auth/oob/cancel/0.1",
  "parentThreadId": "q3Vt7mXo9LwA2bR5cJ8kNg",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-10T10:00:50Z",
  "payload": {
    "requestId": "q3Vt7mXo9LwA2bR5cJ8kNg"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "created": "2026-10-10T10:00:50Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## Response

The service confirms. The payload is the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Cancelled

```json
{
  "id": "urn:uuid:b0718293-a4b5-4fc0-8132-5d6e7f8a9b02",
  "type": "https://trusttasks.org/spec/auth/oob/cancel/0.1#response",
  "threadId": "urn:uuid:b0718293-a4b5-4fc0-8132-5d6e7f8a9b01",
  "parentThreadId": "q3Vt7mXo9LwA2bR5cJ8kNg",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-10T10:00:50Z",
  "payload": {
    "status": "cancelled"
  }
}
```

## Security & Privacy

### Data carried

A handle under a throwaway key, and a status. Nothing personal.

### Correlation

The keys are fresh per request, so a cancel joins only to its own request. The service has identifier scope `public` because both parties address the community's published DID.

### Retention

The cancellation is the request's final state, kept as an audit event with the request; the request record is then swept like any other ended request.

### Consent/purpose

A cancel withdraws the request. It is not a decision about the member's identity and carries none.
