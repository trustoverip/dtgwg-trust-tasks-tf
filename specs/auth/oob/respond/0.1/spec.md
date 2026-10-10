---
slug: auth/oob/respond
version: "0.1"
title: "Auth — OOB Respond"
summary: "The lock holder of an out-of-band request delivers the member's signed decision by carrying an auth/oob/grant, and the service records the request as approved or declined; no session material ever comes back to the approver."
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords: [auth, oob, out-of-band, sign-in, login, grant, wallet]
parties:
  - role: Approver (lock key)
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
    The proof by the lock key ties the carried grant to the device that held the lock and saw step 2; the grant names that key, so no other device can deliver it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task records a decision that lets a key act as the member, so it is consequential and SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: mutating
  rationale: >-
    Moves the request from identified to approved, storing the grant, or to declined. Each request allows one decision.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The carried grant authorizes the starter's key to act as the member's DID for one session; recording it exercises the member's own authority. The response is a bare status and discloses nothing.
retention:
  class: durable
  rationale: >-
    The stored grant is the audit record of the sign-in decision and is kept with the session it armed.
errorCodes:
  - code: auth/oob/respond:contextMismatch
    meaning: The grant's sessionKey, origin or contextDigest does not match what the service stored for the request. The request is declined.
    retryable: false
  - code: auth/oob:notClaimant
    meaning: The respond is not signed by the request's lock key.
    retryable: false
  - code: auth/oob:requestExpired
    meaning: The decision window has passed, or the request has otherwise ended.
    retryable: false
  - code: auth/oob:alreadyDecided
    meaning: The request has already been approved, declined or cancelled.
    retryable: false
  - code: auth/oob:notAuthorized
    meaning: The carried grant failed verification, is not from the identified DID or does not name the lock, or the member is no longer active. Deliberately generic. The request is declined.
    retryable: false
related:
  - auth/oob/prove
  - auth/oob/grant
  - auth/oob/redeem
---

## Abstract

After step 2 the member approves (with user verification) or declines, and the member's VTA signs an [`auth/oob/grant`](../../grant/0.1/spec.md). The wallet delivers it in this task, signed by the lock key. The service checks the grant against everything it stored for the request and records the decision. The response is only a status: the authority the grant confers goes to the starter's key, and only through [`auth/oob/redeem`](../../redeem/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **approver**:

1. **MUST** sign with the lock key and **SHOULD** set `parentThreadId` to the `requestId` ([SPEC §4.9.2](/SPEC.md#492-the-parentthreadid-member)).
2. **MUST** check that the grant its VTA returned is the one it sent, apart from the added proof, before carrying it.
3. **MUST** discard the lock key after this task, and **MUST NOT** resend once the request has ended.

A conforming **service**, in this order:

1. **MUST** require the outer issuer to equal the lock, else `auth/oob:notClaimant`; the request to be `identified`, else `auth/oob:alreadyDecided` or `auth/oob:requestExpired`; and the decision window to be open.
2. **MUST** verify the grant as [`auth/oob/grant/0.1`](../../grant/0.1/spec.md) requires, against the issuer's **`assertionMethod`** relationship, and require its issuer to be the identified DID, else `auth/oob:notAuthorized`.
3. **MUST** require the grant's `approverKey` to equal the lock, its `sessionKey` the starter key, its `origin` the request's origin and its `contextDigest` the stored digest of the signed step 2, else `auth/oob/respond:contextMismatch`.
4. **MUST** check that the member is still active, else `auth/oob:notAuthorized`.
5. On success, **MUST** move the request to `approved` (or `declined` for a decline) in one compare-and-set, store the grant, and answer with the status. On any failure from step 2 on, **MUST** move the request to `declined`.
6. **MUST NOT** put session material in the response.

## Authorization

The authority is the member's own, carried in the grant: the identified DID's attestation that the starter key may act as it ([`auth/oob/grant`](../../grant/0.1/spec.md) Authorization). The lock key's proof entitles the device to deliver it, and nothing more. Active membership, read at execution time, is the standing the session needs.

## Definitions

- **`grant`** — a complete signed `auth/oob/grant/0.1` document, verified over its members as received.
- **`status`** (response) — `approved` or `declined`.

## Request

The approver (`issuer`, the lock key) delivers the grant to the service (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A wallet delivers an approval

```json
{
  "id": "urn:uuid:9e5f6071-8293-4dae-8f10-3b4c5d6e7f01",
  "type": "https://trusttasks.org/spec/auth/oob/respond/0.1",
  "parentThreadId": "q3Vt7mXo9LwA2bR5cJ8kNg",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-10T10:01:31Z",
  "payload": {
    "grant": {
      "id": "urn:uuid:8d4e5f60-7182-4c9d-8e0f-2a3b4c5d6e01",
      "type": "https://trusttasks.org/spec/auth/oob/grant/0.1",
      "issuer": "did:web:alice.example",
      "recipient": "did:web:community.example",
      "issuedAt": "2026-10-10T10:01:30Z",
      "payload": {
        "requestId": "q3Vt7mXo9LwA2bR5cJ8kNg",
        "decision": "approve",
        "sessionKey": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
        "approverKey": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "origin": "https://members.community.example",
        "contextDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
        "notAfter": 1791655200
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:web:alice.example#key-1",
        "created": "2026-10-10T10:01:30Z",
        "proofPurpose": "assertionMethod",
        "proofValue": "z3kg…"
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "created": "2026-10-10T10:01:31Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## Response

The service answers with the recorded status. The payload is the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Approved

```json
{
  "id": "urn:uuid:9e5f6071-8293-4dae-8f10-3b4c5d6e7f02",
  "type": "https://trusttasks.org/spec/auth/oob/respond/0.1#response",
  "threadId": "urn:uuid:9e5f6071-8293-4dae-8f10-3b4c5d6e7f01",
  "parentThreadId": "q3Vt7mXo9LwA2bR5cJ8kNg",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-10T10:01:31Z",
  "payload": {
    "status": "approved"
  }
}
```

## Security & Privacy

### Data carried

The signed grant: a handle, two throwaway keys, an origin, a digest, a time and a decision. The response is a status only.

### Correlation

The service joins the grant to the request, the identified DID and the starter key, which is the purpose of the task. The lock key is pairwise and discarded after this task. The service has identifier scope `public` because the approver addressed the community's published DID throughout.

### Retention

The service keeps the signed grant as the durable audit record of the sign-in decision, with the session it arms.

### Consent/purpose

The grant is delivered to record one decision for one request. A service **MUST NOT** treat it as consent to anything beyond the session the grant names.
