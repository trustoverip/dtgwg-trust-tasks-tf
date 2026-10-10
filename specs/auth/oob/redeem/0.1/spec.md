---
slug: auth/oob/redeem
version: "0.1"
title: "Auth — OOB Redeem"
summary: "The starter long-polls an out-of-band request with its own key and, once the member has approved, turns it into a session bound to that key; before approval it learns the request's state and, once claimed, the number to show."
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords: [auth, oob, out-of-band, sign-in, login, session, session-key, long-poll]
parties:
  - role: Starter
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
    Only the holder of the starter key may redeem, and the key is non-extractable in the browser; the proof is what turns a stolen grant or a leaked handle into nothing. It is also the only thing that entitles a caller to the match number.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task creates a session, so it is consequential and SPEC §7.3 item 17 sets REQUIRED as the floor. Each poll is a freshly signed document, and an issue time lets the service refuse a replayed one.
sideEffects:
  level: mutating
  rationale: >-
    On an approved request, moves it to consumed in one compare-and-set and creates a member session whose subject is the approving DID and whose session key is the starter key.
exposure:
  discloses: secret
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    Creates a session in the approving member's name for the starter key, exercising the authority the grant conferred. The session credential is delivered as HttpOnly cookies, never in the body; the body carries the session record and a display name. Before approval the response is an error carrying the request state and, once claimed, the match number.
retention:
  class: durable
  rationale: >-
    The session and its binding to the starter key are kept for the session's life, and its creation is audited with the grant that armed it.
errorCodes:
  - code: auth/oob/redeem:pending
    meaning: The request is not yet approved. Returned after the service has held the call for up to 25 s. `details.state` gives the state, and `details.matchNumber` the number to show once the request is claimed. The starter retries with a freshly signed document.
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      required: [state]
      properties:
        state:
          type: string
          enum: [pending, claimed, identified]
        matchNumber:
          type: string
          pattern: "^[0-9]{2}$"
  - code: auth/oob/redeem:declined
    meaning: The request was declined or cancelled and will not be approved.
    retryable: false
  - code: auth/oob:notStarter
    meaning: The redeem is not signed by the request's starter key.
    retryable: false
  - code: auth/oob:requestExpired
    meaning: The request has expired or was already consumed.
    retryable: false
  - code: auth/oob:requestNotFound
    meaning: No request has this `requestId`.
    retryable: false
  - code: auth/oob:rateLimited
    meaning: Another poll is already open for this request, or the starter's network has hit the poll cap. `details.retryAfter` MAY give seconds until a retry.
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        retryAfter: { type: integer, minimum: 0 }
related:
  - auth/oob/request
  - auth/oob/respond
  - auth/oob/cancel
  - auth/whoami
---

## Abstract

The starter opened the request with a key only it holds ([`auth/oob/request`](../../request/0.1/spec.md)). It polls this task, signing each poll with that key. Before approval the service holds the call and then answers `auth/oob/redeem:pending`, with the request's state and — once a wallet has claimed it — the two-digit number the starter shows the member. Once the member has approved ([`auth/oob/respond`](../../respond/0.1/spec.md)), the service consumes the request and creates the session the purpose defines. For `login` that is a member session whose subject is the approving DID and whose session key is the starter key; the starter then asks the person "Continue as …?" before using it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **starter**:

1. **MUST** sign each poll with the starter key, as a new document with a fresh `id`, and keep at most one poll open per request.
2. **MUST** show the person the identity named in the response and let them refuse it before using the session; refusing signs out at once.
3. **MUST** delete the starter key at sign-out.

A conforming **service**:

1. **MUST** require the issuer to equal the starter key and the proof to verify, else `auth/oob:notStarter`.
2. Before approval, **SHOULD** hold the call for up to 25 s, with at most one open poll per request, and then answer `auth/oob/redeem:pending` with `details.state`, and `details.matchNumber` once the request is claimed. Only the starter key's holder ever receives the number.
3. For a declined or cancelled request, **MUST** answer `auth/oob/redeem:declined`; for an expired or consumed one, `auth/oob:requestExpired`.
4. For an approved request, **MUST** move it to `consumed` in one compare-and-set, check membership again, and create the session: subject the grant's issuer, `sessionKey` the starter key, `amr` `["did", "oob", "uv"]`, ending at the earlier of the grant's `notAfter` and the service's session limit.
5. Over HTTPS, **MUST** deliver the session credential as `Secure; HttpOnly; SameSite=Lax` cookies and **MUST NOT** put tokens in the body.

## Authorization

Two things together: the member's grant, which names the starter key ([`auth/oob/grant`](../../grant/0.1/spec.md) Authorization), and the starter key's own proof on this document, which shows the caller holds it. Neither alone suffices — a grant without the key is useless, and the key without a grant gets only `pending`. Active membership is checked again at execution time.

## Definitions

- **`requestId`** — the request being redeemed.
- **`session`** (response) — an [`auth/_shared/0.3`](../../../_shared/0.3/session.schema.json) `Session`: subject the approving DID, `sessionKey` the starter key.
- **`displayName`** (response) — a name for the signed-in identity, for the confirmation step.

## Request

The starter (`issuer`, its key) polls the service (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A browser polls its request

```json
{
  "id": "urn:uuid:af607182-93a4-4ebf-9021-4c5d6e7f8a01",
  "type": "https://trusttasks.org/spec/auth/oob/redeem/0.1",
  "issuer": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-10T10:01:32Z",
  "payload": {
    "requestId": "q3Vt7mXo9LwA2bR5cJ8kNg"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp#z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
    "created": "2026-10-10T10:01:32Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## Response

On an approved request the service answers with the session. The payload is the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Every other outcome, including "not yet", is a `trust-task-error`.

### Signed in

```json
{
  "id": "urn:uuid:af607182-93a4-4ebf-9021-4c5d6e7f8a02",
  "type": "https://trusttasks.org/spec/auth/oob/redeem/0.1#response",
  "threadId": "urn:uuid:af607182-93a4-4ebf-9021-4c5d6e7f8a01",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
  "issuedAt": "2026-10-10T10:01:32Z",
  "payload": {
    "session": {
      "id": "sess_7f3c1a",
      "subject": "did:web:alice.example",
      "issuedAt": "2026-10-10T10:01:32Z",
      "expiresAt": "2026-10-10T18:00:00Z",
      "amr": ["did", "oob", "uv"],
      "sessionKey": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp"
    },
    "displayName": "Alice"
  }
}
```

### Claimed, not yet approved

```json
{
  "id": "urn:uuid:af607182-93a4-4ebf-9021-4c5d6e7f8a03",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:af607182-93a4-4ebf-9021-4c5d6e7f8a01",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
  "issuedAt": "2026-10-10T10:00:45Z",
  "payload": {
    "code": "auth/oob/redeem:pending",
    "message": "Approve on your phone.",
    "retryable": true,
    "details": { "state": "claimed", "matchNumber": "47" }
  }
}
```

## Security & Privacy

### Data carried

The request carries a handle. The success response carries the session record (identifiers and times, no credential) and a display name; the credential itself travels as HttpOnly cookies. The `pending` error carries the state and, once claimed, the match number — to the starter key's holder only.

### Correlation

The starter key becomes the session key, so the session is joinable to this sign-in, which is its purpose. The service has identifier scope `public` because the starter addresses the community's published DID; the starter key is pairwise.

### Retention

The session and its binding to the starter key are kept for the session's life; the request record is consumed. The audit record keeps the grant that armed the session.

### Consent/purpose

The session lets the starter act as the member at the portal origin for one session. The starter's "Continue as …?" step exists so a person who did not intend this identity can refuse it before it is used.
