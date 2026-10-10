---
slug: auth/oob/prove
version: "0.1"
title: "Auth — OOB Prove"
summary: "The lock holder of an out-of-band request proves membership and the number on the starter's screen by carrying a signed auth/oob/identify, and gets back the service-signed step 2: where the request came from and which key it would sign in."
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords: [auth, oob, out-of-band, sign-in, login, membership, wallet]
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
    The request's proof by the lock key is what ties the carried identify to the device holding the lock, so another device cannot re-wrap it. The response's proof, by the service's assertionMethod key, makes step 2 an attestation: the wallet shows its contents as the community's statement, and the grant binds its digest.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task changes request state, so it is consequential and SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: mutating
  rationale: >-
    Moves the request from claimed to identified and records the identified DID, or to declined on any failure. Each request allows one proof attempt.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: >-
    Ingests the member's chosen DID (inside identify). Discloses, to the proven lock holder only, the starter's coarse location (city and country), browser family, OS, creation time, a same-network signal and the starter key — never an IP address.
retention:
  class: durable
  rationale: >-
    The service audits every proof, including failed and peeking ones, with city and country, because step 2 is the moment the starter's location was shown to someone. It keeps the digest of the signed step 2 for the life of the request, to check the grant against.
errorCodes:
  - code: auth/oob/prove:numberMismatch
    meaning: The typed number does not match. The request is declined.
    retryable: false
  - code: auth/oob:notClaimant
    meaning: The prove is not signed by the request's lock key.
    retryable: false
  - code: auth/oob:requestExpired
    meaning: The decision window has passed, or the request has otherwise ended.
    retryable: false
  - code: auth/oob:notAuthorized
    meaning: The carried identify failed verification, does not match the request or the lock, or its issuer is not an active member. Deliberately generic. The request is declined.
    retryable: false
  - code: auth/oob:rateLimited
    meaning: The approver's network has hit the service's prove limit. `details.retryAfter` MAY give seconds until a retry.
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        retryAfter: { type: integer, minimum: 0 }
related:
  - auth/oob/claim
  - auth/oob/identify
  - auth/oob/respond
---

## Abstract

After the claim, the member picks an identity and types the number shown on the starter's screen. The wallet has the member's VTA sign an [`auth/oob/identify`](../../identify/0.1/spec.md) and sends it inside this task, signed by the lock key. If the member is a member and the number matches, the service answers with **step 2**: the step 1 members unchanged, plus the starter key (`sessionKey`), where the request came from (`requester`) and which DID was identified (`identifiedAs`). The wallet shows this before the member approves or declines.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **approver**:

1. **MUST** sign with the lock key `K_a` and **SHOULD** set `parentThreadId` to the `requestId` ([SPEC §4.9.2](/SPEC.md#492-the-parentthreadid-member)).
2. **MUST** check that the identify its VTA returned is the one it sent, apart from the added proof, before carrying it.
3. **MUST** verify the response's proof against a key the service DID lists under `assertionMethod`, and **MUST** check that it repeats the step 1 members unchanged and that `identifiedAs` is the DID it chose.

A conforming **service**, in this order:

1. **MUST** require the outer issuer to equal the lock, else `auth/oob:notClaimant`, and the request to be `claimed` and inside its decision window, else `auth/oob:requestExpired`.
2. **MUST** require the carried identify to name this request and the lock as `approverKey`.
3. **MUST** check that the identify issuer string is an active member **before** resolving any DID.
4. **MUST** verify the identify as [`auth/oob/identify/0.1`](../../identify/0.1/spec.md) requires, against the issuer's `authentication` relationship.
5. **MUST** require `enteredNumber` to equal the match number, else `auth/oob/prove:numberMismatch`.
6. On success, **MUST** move the request to `identified` in one compare-and-set, record the DID, set `sameNetwork` by comparing this call's egress network with the starter's, answer with step 2 signed by its `assertionMethod` key, and store the digest of that signed response.
7. On any failure from step 2 on, **MUST** move the request to `declined`. A non-member and a bad signature **MUST** both get `auth/oob:notAuthorized`.

## Authorization

The lock entitles its holder to attempt one proof. Membership, read from the service's own member list, and the matching number are what entitle the holder to see step 2. Neither the outer nor the inner proof is authorization by itself ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10): the outer one identifies the lock holder, the inner one the DID.

## Definitions

- **`identify`** — a complete signed `auth/oob/identify/0.1` document, verified over its members as received.
- **Step 2** (response) — step 1's `requestId`, `service`, `origin`, `purpose` and `decisionDeadline`, plus:
  - **`sessionKey`** — the starter key `K_b`;
  - **`requester`** — `location` (city and country, or `unknown`), `browser` (family), `os`, `createdAt`, and `sameNetwork` (`same`, `different` or `unknown`);
  - **`identifiedAs`** — the DID that issued the identify.

## Request

The approver (`issuer`, the lock key) sends the carried identify to the service (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A wallet proves membership and the number

```json
{
  "id": "urn:uuid:7c3d4e5f-6071-4b8c-9d0e-1f2a3b4c5d01",
  "type": "https://trusttasks.org/spec/auth/oob/prove/0.1",
  "parentThreadId": "q3Vt7mXo9LwA2bR5cJ8kNg",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-10T10:01:01Z",
  "payload": {
    "identify": {
      "id": "urn:uuid:6b2c3d4e-5f60-4a7b-8c9d-0e1f2a3b4c01",
      "type": "https://trusttasks.org/spec/auth/oob/identify/0.1",
      "issuer": "did:web:alice.example",
      "recipient": "did:web:community.example",
      "issuedAt": "2026-10-10T10:01:00Z",
      "payload": {
        "requestId": "q3Vt7mXo9LwA2bR5cJ8kNg",
        "approverKey": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "enteredNumber": "47"
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:web:alice.example#key-1",
        "created": "2026-10-10T10:01:00Z",
        "proofPurpose": "authentication",
        "proofValue": "z3kg…"
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
    "created": "2026-10-10T10:01:01Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## Response

The service answers with step 2, signed by its `assertionMethod` key. The payload is the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Step 2

```json
{
  "id": "urn:uuid:7c3d4e5f-6071-4b8c-9d0e-1f2a3b4c5d02",
  "type": "https://trusttasks.org/spec/auth/oob/prove/0.1#response",
  "threadId": "urn:uuid:7c3d4e5f-6071-4b8c-9d0e-1f2a3b4c5d01",
  "parentThreadId": "q3Vt7mXo9LwA2bR5cJ8kNg",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-10T10:01:01Z",
  "payload": {
    "requestId": "q3Vt7mXo9LwA2bR5cJ8kNg",
    "service": { "did": "did:web:community.example", "name": "Example Community" },
    "origin": "https://members.community.example",
    "purpose": "login",
    "decisionDeadline": "2026-10-10T10:02:40Z",
    "sessionKey": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
    "requester": {
      "location": "Sydney, Australia",
      "browser": "Chrome",
      "os": "macOS",
      "createdAt": "2026-10-10T10:00:00Z",
      "sameNetwork": "same"
    },
    "identifiedAs": "did:web:alice.example"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:community.example#key-1",
    "created": "2026-10-10T10:01:01Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg…"
  }
}
```

### The number did not match

```json
{
  "id": "urn:uuid:7c3d4e5f-6071-4b8c-9d0e-1f2a3b4c5d03",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:7c3d4e5f-6071-4b8c-9d0e-1f2a3b4c5d01",
  "parentThreadId": "q3Vt7mXo9LwA2bR5cJ8kNg",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-10T10:01:01Z",
  "payload": {
    "code": "auth/oob/prove:numberMismatch",
    "message": "The number does not match. Refresh the code on the website and scan again.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

In: the member's chosen DID, inside identify. Out: the starter's city and country (or `unknown`), browser family, OS, creation time, a same-network signal, and the starter key. Never the starter's IP address or full User-Agent. Step 2 is revealed only to the lock holder, after a membership proof and a number proof.

### Correlation

The service learns which DID approved which request, which is the purpose of the task. The lock key remains fresh per request. The service has identifier scope `public` because the approver must reach and verify the community's published DID; the approver's lock key is pairwise.

### Retention

The service keeps the identified DID and the digest of the signed step 2 for the life of the request, and audits each proof — including a failed one, and one that saw step 2 and went no further — with city and country, never the address.

### Consent/purpose

The starter's coarse location and browser are shown to the member so they can tell whether the request is theirs. They are not to be used for anything else. The location is a heuristic: a relaying attacker controls where the request appears to come from.
