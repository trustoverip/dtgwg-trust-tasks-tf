---
slug: auth/oob/redeem
version: "0.3"
title: "Auth — OOB Redeem"
summary: "The starter long-polls an out-of-band request with its own key and, once approved, gets a session bound to that key or, for deviceEnrol, an ACL entry at the person's VTA whose subject is that key; before approval it learns the request's state and, once claimed, the number."
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
    On an approved request, moves it to consumed in one compare-and-set and creates a member session whose subject is the approving DID and whose session key is the starter key — or, for deviceEnrol, writes an ACL entry at the VTA whose subject is the starter key, with the scope the grant chose.
exposure:
  discloses: secret
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    Creates a session in the approving member's name for the starter key, exercising the authority the grant conferred — or, for deviceEnrol, an ACL entry delegated by the approving administrator. The enrolment response carries the entry, which names the client's scope and no secret. The session credential is delivered as HttpOnly cookies, never in the body; the body carries the session record and a display name. Before approval the response is an error carrying the request state and, once claimed, the match number.
retention:
  class: durable
  rationale: >-
    The session and its binding to the starter key are kept for the session's life, and its creation is audited with the grant that armed it. A deviceEnrol entry is kept until it expires or is revoked, with the grant as its provenance.
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
          pattern: "^([0-9]{2}|[0-9]{4})$"
  - code: auth/oob/redeem:declined
    meaning: The request was declined or cancelled and will not be approved. `details.state` says which.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [state]
      properties:
        state:
          type: string
          enum: [declined, cancelled]
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

The starter opened the request with a key only it holds ([`auth/oob/request`](../../request/0.3/spec.md)). It polls this task, signing each poll with that key. Before approval the service holds the call and then answers `auth/oob/redeem:pending`, with the request's state and — once a wallet has claimed it — the number the starter shows the member — two digits for `login`, four for `deviceEnrol`. Once the member has approved ([`auth/oob/respond`](../../respond/0.4/spec.md)), the service consumes the request and creates the session the purpose defines. For `login` that is a member session whose subject is the approving DID and whose session key is the starter key; the starter then asks the person "Continue as …?" before using it.

For `deviceEnrol` the service is the person's VTA, and on approval it writes an ACL entry itself: subject the starter key, scope the grant's `aclScope`, delegated by the approving administrator. The response carries the VTA's DID, that entry, and `swapClosesAt`, by which the client must replace the starter key with its own DID through [`acl/swap-key`](../../../../acl/swap-key/0.1/spec.md). The client then registers itself with [`device/register`](../../../../device/register/0.2/spec.md). Neither is part of this family, and neither is done for the client: both are its own signed requests.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.2

- The response is one of two shapes. For `login` it is 0.2's response,
  unchanged: `subject`, `displayName`, `sessionEndsAt` and `amr`. For
  `deviceEnrol` it is the enrolment: **`vtaDid`**, **`aclEntry`** (an
  [`acl/_shared/0.3`](../../../../acl/_shared/0.3/acl-entry.schema.json)
  `AclEntry`) and **`swapClosesAt`**. Exactly one shape is present; the schema
  says so.
- The request payload is unchanged. A `login` redeem is handled exactly as in
  0.2.

This version cannot declare `extends: "0.2"`: the 0.2 response members,
required there, are conditional here
([`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §7 A10). A `deviceEnrol`
request is never redeemed under 0.2 or before, and never answered with a `login`-shaped
response (CONVENTIONS §5 rule 3c). `0.2` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **starter**:

1. **MUST** sign each poll with the starter key, as a new document with a fresh `id`, and keep at most one poll open per request.
2. For `login`, **MUST** show the person the identity named in the response and let them refuse it before using the session; refusing signs out at once.
3. **MUST** delete the starter key at sign-out.
4. For `deviceEnrol`, needs no confirmation by a person at the client: there is no prompt to approve on the client's side, because only an administrator of the VTA can approve and the number match is the person's check ([`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §7, A13). It **MUST** check that the response's `vtaDid` is the VTA it opened the request at and that `aclEntry.subject` is its starter key; **MUST**, before `swapClosesAt`, send [`acl/swap-key`](../../../../acl/swap-key/0.1/spec.md) signed by the starter key, replacing it with a DID the client controls — a fresh one, not the subject of another ACL entry at the VTA, since [`acl/swap-key`](../../../../acl/swap-key/0.1/spec.md) refuses a subject already in use (`acl/swap-key:subjectAlreadyInUse`) and the entry would then be removed at `swapClosesAt`; to re-enrol a DID, an administrator first revokes its old entry — and **SHOULD** include `linkProof` in that swap; **MUST** then delete the starter key; and **SHOULD** then send [`device/register`](../../../../device/register/0.2/spec.md) with the new DID, naming the `consumerKind` it was enrolled as. It **MUST NOT** use the starter key for anything else ([`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §5 rule 1a). If the response is lost, it **MAY** poll again with the starter key before `swapClosesAt` and receive the same enrolment (service item 7); the swap needs only the starter key, not the response.

A conforming **service**:

1. **MUST** require the issuer to equal the starter key and the proof to verify, else `auth/oob:notStarter`.
2. Before approval, **SHOULD** hold the call for up to 25 s, with at most one open poll per request, and then answer `auth/oob/redeem:pending` with `details.state`, and `details.matchNumber` once the request is claimed. Only the starter key's holder ever receives the number.
3. For a declined or cancelled request, **MUST** answer `auth/oob/redeem:declined` with `details.state` `declined` or `cancelled`; for an expired or consumed one, `auth/oob:requestExpired` — except the re-delivery of a `deviceEnrol` enrolment in item 7.
3a. **MUST** handle a request opened as `deviceEnrol` only under this version, and answer a poll for one sent as an earlier version of this task — `auth/oob/redeem/0.2` or before — with `auth/oob:requestNotFound`; it **MUST NOT** answer a `deviceEnrol` request with a `login`-shaped response ([`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §5 rule 3c).
4. For an approved request, **MUST** move it to `consumed` in one compare-and-set, check membership again, and create the session: subject the grant's issuer, session key the starter key, `amr` `["did", "oob", "uv"]`, ending at the earlier of the grant's `sessionEndsAt` and the service's session limit.
5. **MUST** answer with `{ subject, displayName, sessionEndsAt, amr }` describing that session, `sessionEndsAt` as an RFC 3339 date-time. Over HTTPS, **MUST** deliver the session itself as `Secure; HttpOnly; SameSite=Lax` cookies and **MUST NOT** put tokens in the body.
6. For an approved `deviceEnrol` request, the service is the VTA and, instead of items 4 and 5, **MUST** move the request to `consumed` in one compare-and-set; check again that the grant's issuer may write the entry ([`auth/oob/respond/0.4`](../../respond/0.4/spec.md) service item 4), else answer `auth/oob/redeem:declined` with `details.state` `declined`; and create one ACL entry: `subject` the starter key; `role`, `capabilities`, `purposes`, `keys`, `expiresAt` and `label` from the grant's `aclScope` (`label` defaulting to the client's `displayName`); `act` `{"scope": "contexts", "contexts": aclScope.contexts}`; `approve` `{"scope": "none"}`; `approveCapabilities` `{"scope": "none"}`; `delegatedBy` and `createdBy` the grant's issuer. It **MUST** keep `aclScope.consumerKind` beside the entry — the ACL entry has no such member — and **SHOULD** refuse a later `device/register` for the entry that names another kind ([`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §7 A9). It **MUST** keep the user-verification `consent/decision` that the approving grant was signed on with the grant, as the entry's provenance, for as long as the entry exists: the grant's signature is its own vault's, so the decision is the only evidence of the approval made outside the VTA.
7. For `deviceEnrol`, **MUST** answer with `{ vtaDid, aclEntry, swapClosesAt }`: its own DID, the entry as [`acl/show`](../../../../acl/show/0.3/spec.md) would return it, and `swapClosesAt` no more than 300 s after this response (120 s RECOMMENDED). Until the swap it **MUST** accept the starter key for that `acl/swap-key` only, and **MUST** remove the entry if it has not been swapped by `swapClosesAt` (CONVENTIONS §7 A3, A5); the removal and a swap that arrives at the same moment are one compare-and-set on the entry, so exactly one of them takes effect. For that swap it **SHOULD** require `linkProof`, so that the entry lands on a key the client controls. A later poll signed by the same starter key before `swapClosesAt`, while the entry is unswapped, **MUST** receive the same enrolment again, with the same `swapClosesAt`; after that it gets `auth/oob:requestExpired`. It **MUST NOT** put any other identifier or credential in the response, `ext` included.

## Authorization

Two things together: the member's grant, which names the starter key ([`auth/oob/grant`](../../grant/0.4/spec.md) Authorization), and the starter key's own proof on this document, which shows the caller holds it. Neither alone suffices — a grant without the key is useless, and the key without a grant gets only `pending`. Active membership is checked again at execution time. For `deviceEnrol`, the grant is an administrator's, the administrator's authority to write the entry is checked again at execution time, and the entry is bounded by it afterwards through `delegatedBy`.

## Definitions

- **`requestId`** — the request being redeemed.
- **`subject`** (response) — the approving DID, now the session's subject. The session's key is the starter key.
- **`displayName`** (response) — a name for the signed-in identity, for the confirmation step.
- **`sessionEndsAt`** (response) — when the session ends, as an RFC 3339 date-time.
- **`amr`** (response) — the session's authentication methods, `["did", "oob", "uv"]`.
- **`vtaDid`** (response, `deviceEnrol`) — the DID of the VTA the client is enrolled at.
- **`aclEntry`** (response, `deviceEnrol`) — the ACL entry the VTA created for the starter key.
- **`swapClosesAt`** (response, `deviceEnrol`) — the instant by which the client must swap the starter key for its own DID, as an RFC 3339 date-time.

## Request

The starter (`issuer`, its key) polls the service (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A browser polls its request

```json
{
  "id": "urn:uuid:af607182-93a4-4ebf-9021-4c5d6e7f8a01",
  "type": "https://trusttasks.org/spec/auth/oob/redeem/0.3",
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
  "type": "https://trusttasks.org/spec/auth/oob/redeem/0.3#response",
  "threadId": "urn:uuid:af607182-93a4-4ebf-9021-4c5d6e7f8a01",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
  "issuedAt": "2026-10-10T10:01:32Z",
  "payload": {
    "subject": "did:web:alice.example",
    "displayName": "Alice",
    "sessionEndsAt": "2026-10-10T18:00:00Z",
    "amr": ["did", "oob", "uv"]
  }
}
```

### The member cancelled

```json
{
  "id": "urn:uuid:af607182-93a4-4ebf-9021-4c5d6e7f8a04",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:af607182-93a4-4ebf-9021-4c5d6e7f8a01",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
  "issuedAt": "2026-10-10T10:00:51Z",
  "payload": {
    "code": "auth/oob/redeem:declined",
    "message": "The sign-in was cancelled.",
    "retryable": false,
    "details": { "state": "cancelled" }
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

### Enrolled

Alice's VTA has written the agent's entry, as Alice's grant chose it.

```json
{
  "id": "urn:uuid:af607182-93a4-4ebf-9021-4c5d6e7f8a12",
  "type": "https://trusttasks.org/spec/auth/oob/redeem/0.3#response",
  "threadId": "urn:uuid:af607182-93a4-4ebf-9021-4c5d6e7f8a11",
  "issuer": "did:web:vta.alice.example",
  "recipient": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "issuedAt": "2026-10-10T10:01:32Z",
  "payload": {
    "vtaDid": "did:web:vta.alice.example",
    "aclEntry": {
      "subject": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
      "role": "application",
      "act": { "scope": "contexts", "contexts": ["personal"] },
      "approve": { "scope": "none" },
      "keys": { "scope": "none" },
      "capabilities": { "scope": "listed", "grants": [{ "capability": "memory.read" }] },
      "approveCapabilities": { "scope": "none" },
      "purposes": { "scope": "listed", "purposes": ["taskExecution"] },
      "delegatedBy": "did:web:alice.example",
      "label": "Research agent on laptop",
      "createdAt": "2026-10-10T10:01:32Z",
      "createdBy": "did:web:alice.example",
      "expiresAt": "2026-11-09T10:01:30Z"
    },
    "swapClosesAt": "2026-10-10T10:03:32Z"
  }
}
```


## Security & Privacy

### Data carried

The request carries a handle. The success response carries the subject, a display name, the end time and the authentication methods (no credential); the credential itself travels as HttpOnly cookies. The `pending` error carries the state and, once claimed, the match number — to the starter key's holder only. For `deviceEnrol`, the success response carries the VTA's DID and the client's own ACL entry, including the context it may act in and the approving administrator's DID as `delegatedBy` and `createdBy`, so after redeem the client learns that DID; no credential.

### Correlation

The starter key becomes the session key, so the session is joinable to this sign-in, which is its purpose. The service has identifier scope `public` because the starter addresses the community's published DID; the starter key is pairwise.

### Retention

The session and its binding to the starter key are kept for the session's life; the request record is consumed. The audit record keeps the grant that armed the session. A `deviceEnrol` entry lives until its `expiresAt` or until it is revoked, and the VTA keeps the grant, with the user-verification `consent/decision` it was signed on, as its provenance for as long (service item 6).

### Consent/purpose

The session lets the starter act as the member at the portal origin for one session. The starter's "Continue as …?" step exists so a person who did not intend this identity can refuse it before it is used.

For `deviceEnrol`, the entry gives the client the access the person chose and nothing else: no session, no signing as the person, no approve authority. The short swap window bounds how long a key that was shown in a QR exchange stays an ACL subject.

## Pending confirmation

This version takes the readings in [`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §7 and will follow the family authors' answer when they give one. Those that bear on this task: **A3**, **A5**, **A7**, **A9**, **A10**, **A12** and **A13**.
