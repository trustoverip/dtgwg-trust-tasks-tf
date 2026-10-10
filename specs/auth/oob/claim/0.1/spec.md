---
slug: auth/oob/claim
version: "0.1"
title: "Auth — OOB Claim"
summary: "An approver's wallet claims a pending out-of-band request with a throwaway key, locking the request to that key, and gets back the service-signed step 1: which community, at which portal origin, for what purpose, and until when."
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords: [auth, oob, out-of-band, sign-in, login, qr, trigger-link, wallet, lock]
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
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The request's proof makes its issuer the lock: every later prove, respond and cancel by the approver is checked against that key, so an unsigned claim would lock the request to nobody. The response's proof, by the service's assertionMethod key, is what lets the wallet treat the community, origin and purpose as the service's attestation rather than as text from whoever answered.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task changes request state, so it is consequential and SPEC §7.3 item 17 sets REQUIRED as the floor. The claim window is two minutes, and an issue time lets the service refuse a stale claim outright.
sideEffects:
  level: mutating
  rationale: >-
    Moves the request from pending to claimed in one compare-and-set, records the lock key, chooses the match number and starts the decision window. Only one claim can succeed.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The claim carries a handle under a throwaway key. Step 1 discloses the community's DID and name, the portal origin, the purpose and a deadline: facts the community publishes anyway, and nothing about the starter.
retention:
  class: exchange
  rationale: >-
    The lock key and match number live as long as the request. The claim is audited as an event; the throwaway key identifies nobody once the request ends.
errorCodes:
  - code: auth/oob/claim:alreadyClaimed
    meaning: The request has already been claimed by another key. Carries no details, so a second scanner learns nothing about the first.
    retryable: false
  - code: auth/oob:requestNotFound
    meaning: No request has this `requestId`.
    retryable: false
  - code: auth/oob:requestExpired
    meaning: The request's claim window has passed, or the request has otherwise ended.
    retryable: false
  - code: auth/oob:keyUnsupported
    meaning: The document's issuer is not an Ed25519 `did:key`.
    retryable: false
  - code: auth/oob:rateLimited
    meaning: The approver's network has hit the service's claim limit. `details.retryAfter` MAY give seconds until a retry.
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        retryAfter: { type: integer, minimum: 0 }
related:
  - auth/oob/request
  - auth/oob/prove
  - auth/oob/cancel
---

## Abstract

A member's wallet reads a `sign-in` trigger link ([`TRIGGER-LINK.md`](../../_shared/0.1/TRIGGER-LINK.md)), checks the community against its own records, and once the member chooses to continue, generates a throwaway Ed25519 key `K_a` and sends this claim. The first claim wins: the request is locked to `K_a`, the starter's page shows that it was claimed, and every later step by the approver must be signed by `K_a`.

The service answers with **step 1** of the disclosure ladder ([`CONVENTIONS.md`](../../_shared/0.1/CONVENTIONS.md) §5): the community, the portal origin, the purpose and the decision deadline. Nothing about the starter is revealed until the approver has proved membership and typed the number shown on the starter's screen (`auth/oob/prove`).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **approver** (the wallet):

1. **MUST** generate `K_a`, a fresh Ed25519 key, only after the person has chosen to continue, and **MUST NOT** use it for any other request.
2. **MUST** follow VTI-LNK-054: set `issuer` to `K_a` as a `did:key`, `recipient` to the service DID (the link's `_from`), a unique `id`, and `parentThreadId` to the handle (the link's `_id`), and sign with `K_a` for `authentication`.
3. **MUST** set `payload.requestId` to the same handle as `parentThreadId`.
4. **MUST** take the endpoint from the service's verified DID document, never from the link.
5. **MUST** verify the response's proof against a key the service DID lists under `assertionMethod`, and **MUST** check that `requestId` equals the handle, `service.did` equals the link's `_from`, `purpose` is one it implements, `origin` is the origin of the service's `SignInPortal` service, and `decisionDeadline` is in the future. Any failure ends the exchange on the approver's side.
6. **MUST** show the community's name from its own records and flag a difference from `service.name` (VTI-LNK-104).

A conforming **service**:

1. **MUST** refuse an issuer that is not an Ed25519 `did:key` with `auth/oob:keyUnsupported`.
2. **MUST** refuse a document whose `parentThreadId` is absent or not equal to `payload.requestId` with `malformedRequest`. This compares two members of one document; it is not a check on `parentThreadId` alone.
3. **MUST** move the request from `pending` to `claimed` in one compare-and-set, only inside its claim window, recording `K_a` as the lock, choosing a two-digit match number uniformly at random and starting the decision window. A request that is not `pending` is refused with `auth/oob/claim:alreadyClaimed` (if claimed or later) or `auth/oob:requestExpired` (if ended), with no details.
4. **MUST** answer with step 1, signed by its `assertionMethod` key, and **MUST NOT** put anything about the starter in it.
5. **MUST** leave the request unchanged when a claim fails.

## Authorization

Possession of the handle entitles a party to *claim* the request and to nothing more. Whoever claims first becomes the lock; that confers no authority, because no grant can follow without a membership proof and the number from the starter's screen (`auth/oob/prove`), and the starter sees the claim at once. The proof establishes which key holds the lock ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- **Lock**, **`K_a`** — the approver's throwaway Ed25519 key. The request's `approverKey` from this claim on.
- **`requestId`** — the handle from the trigger link's `_id`, also carried as `parentThreadId`.
- **Step 1** (response) — `requestId`, `service{did, name}`, `origin` (the portal origin), `purpose` and `decisionDeadline`.

## Request

The approver (`issuer`, its lock key) claims the request at the service (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A wallet claims a scanned sign-in request

```json
{
  "id": "urn:uuid:5a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c01",
  "type": "https://trusttasks.org/spec/auth/oob/claim/0.1",
  "parentThreadId": "q3Vt7mXo9LwA2bR5cJ8kNg",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-10T10:00:40Z",
  "payload": {
    "requestId": "q3Vt7mXo9LwA2bR5cJ8kNg"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "created": "2026-10-10T10:00:40Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## Response

The service answers with step 1, signed by its `assertionMethod` key. The payload is the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Step 1

```json
{
  "id": "urn:uuid:5a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c02",
  "type": "https://trusttasks.org/spec/auth/oob/claim/0.1#response",
  "threadId": "urn:uuid:5a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c01",
  "parentThreadId": "q3Vt7mXo9LwA2bR5cJ8kNg",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-10T10:00:40Z",
  "payload": {
    "requestId": "q3Vt7mXo9LwA2bR5cJ8kNg",
    "service": { "did": "did:web:community.example", "name": "Example Community" },
    "origin": "https://members.community.example",
    "purpose": "login",
    "decisionDeadline": 1791626560
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:community.example#key-1",
    "created": "2026-10-10T10:00:40Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg…"
  }
}
```

### Someone else claimed it first

```json
{
  "id": "urn:uuid:5a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c03",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:5a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c01",
  "parentThreadId": "q3Vt7mXo9LwA2bR5cJ8kNg",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-10T10:00:41Z",
  "payload": {
    "code": "auth/oob/claim:alreadyClaimed",
    "message": "This code was already used by another device.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The claim carries the handle under a throwaway key. Step 1 carries the community's DID and its self-supplied name, the portal origin, the purpose and a deadline. It carries nothing about the starter: no location, no browser, no starter key and no match number.

### Correlation

`K_a` is fresh per request, so claims cannot be joined to each other or to the member; the member's DID is not revealed until `prove`. The handle appears both in the payload and as `parentThreadId`, so the service can join the claim to the request it opened, which is the point. The service has identifier scope `public` because the approver resolves the community's published DID and checks it against its own records before sending anything.

### Retention

The lock key and match number are kept for the life of the request. The service audits the claim as an event; nothing in it identifies the member.

### Consent/purpose

The handle is sent only to lock the request for the member who chose to continue. Step 1 exists so the member can confirm which community and portal they are signing in to before they reveal which identity they hold.
