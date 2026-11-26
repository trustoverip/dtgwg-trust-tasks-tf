---
slug: auth/oob/request
version: "0.3"
title: "Auth — OOB Request"
summary: "A starter, such as a browser on a portal page or a client asking to be enrolled at a person's VTA, asks a service to open an out-of-band request that an approver will decide on another device, and gets back the request's handle and the instant its claim window closes."
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords: [auth, oob, out-of-band, sign-in, login, qr, trigger-link, wallet, enrolment, device-enrolment, ai-agent]
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
    The proof by the starter key is what makes that key the request's starter key. Every later redeem and cancel by the starter is checked against it, so a request opened without one could be redeemed by anyone.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task creates server state, so it is consequential and SPEC §7.3 item 17 sets REQUIRED as the floor. An issue time lets the service refuse a replayed request before it opens a second one.
sideEffects:
  level: mutating
  rationale: >-
    Creates a pending request record holding the starter key, the purpose, the mode, the origin and the starter's coarse connection details, with the instant its claim window closes — and, for `deviceEnrol`, the client's self-asserted kind and name and the scope ceiling it asks for.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries a purpose and a mode — for `deviceEnrol`, also a client kind and name and a scope ceiling — signed by a throwaway key; the response returns a random handle and a deadline. The service also records the starter's egress address for the life of the request, to compute the same-network signal, and never returns it.
retention:
  class: exchange
  rationale: >-
    The request record lives until the request is consumed, declined, cancelled or expired. The egress address is dropped when the request ends; an audit record keeps city and country only.
errorCodes:
  - code: auth/oob/request:purposeUnsupported
    meaning: The `purpose` is not one this service implements. This version defines `login` and `deviceEnrol`.
    retryable: false
  - code: auth/oob/request:clientKindUnsupported
    meaning: For deviceEnrol, the VTA does not enrol clients of the `consumerKind` the request names.
    retryable: false
  - code: auth/oob/request:scopeUnsupported
    meaning: For deviceEnrol, the `requestedScope` names a capability, purpose or key scope the VTA never grants an enrolled client, or a `maxLifetimeDays` beyond its own limit. A client may retry with a narrower ceiling.
    retryable: false
  - code: auth/oob/request:modeUnsupported
    meaning: The `mode` is not one this service implements. This version defines `scan` only.
    retryable: false
  - code: auth/oob:keyUnsupported
    meaning: The document's issuer is not an Ed25519 `did:key`.
    retryable: false
  - code: auth/oob:rateLimited
    meaning: The starter's network has hit the service's limit on requests or on pending requests. `details.retryAfter` MAY give seconds until a retry.
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        retryAfter: { type: integer, minimum: 0 }
related:
  - auth/oob/claim
  - auth/oob/redeem
  - auth/oob/cancel
---

## Abstract

The `auth/oob/*` family lets a **starter** on one device (for `login`, a browser on a community's portal page) get a request decided by an **approver** on another (a member's wallet and their VTA). This task opens the request. The starter generates a fresh Ed25519 key, signs this document with it, and gets back the request's handle and the instant its claim window closes, from which it builds the `sign-in` trigger link shown as a QR code ([`TRIGGER-LINK.md`](../../_shared/0.2/TRIGGER-LINK.md)).

For `deviceEnrol` the starter is a client that holds its own keys — an AI agent, a command-line tool, an app or a device — asking for scoped, long-lived access to a person's VTA, and the service is that VTA. The request then also names the client (`client`) and the most it asks to be allowed (`requestedScope`). The person's wallet, an administrator of the VTA, decides it with the same QR code, number match and user verification as `login`, and chooses the scope within that ceiling.

The family's roles, state machine and shared rules are in [`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.2

- **`purpose` may be `deviceEnrol`**, defined in
  [`oob.schema.json` 0.2](../../_shared/0.2/oob.schema.json) and its
  [`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §3 row.
- Two new error codes for `deviceEnrol`, `clientKindUnsupported` and
  `scopeUnsupported`.
- Two optional members, **`client`** (`EnrolClient`: the client's
  `consumerKind`, in [`device/_shared/0.2`](../../../../device/_shared/0.2/device-binding.schema.json)'s
  terms, and its `displayName`) and **`requestedScope`** (`RequestedScope`:
  the most capabilities, purposes and keys it asks for, and `maxLifetimeDays`).
  Both are REQUIRED for `deviceEnrol` and MUST be absent for `login`; the
  schema says so.

A `login` request is the same document as in 0.2, apart from its `type`, and is
handled the same way. The response is unchanged. Every reference to the shared
components moves to `_shared/0.2`.

This version cannot declare `extends: "0.2"`
([`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §7 A10). A `deviceEnrol`
request is handled only by the family versions that define it (CONVENTIONS §5
rule 3c).
`0.2` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **starter**:

1. **MUST** generate a fresh Ed25519 key for each request, set `issuer` to it as a `did:key`, and sign the document with it for `authentication`. In a browser the key **SHOULD** be a non-extractable WebCrypto key.
2. **MUST** set `recipient` to the service's DID.
3. **MUST** use the same key for every `auth/oob/redeem` and `auth/oob/cancel` of this request, and for nothing else — except that, for `deviceEnrol`, the key then signs the one `acl/swap-key` that replaces it ([`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §5 rule 1a).
4. For `deviceEnrol`, **MUST** send `client` and `requestedScope`, and **SHOULD** ask for the least it needs: the fewest capabilities, listed purposes rather than `any`, keys `none` or `listed` — never `all`, which the schema refuses ([`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §7 A14) — and the shortest `maxLifetimeDays`. For `login` it **MUST NOT** send either.

A conforming **service**:

1. **MUST** refuse an issuer that is not an Ed25519 `did:key` with `auth/oob:keyUnsupported`, before parsing beyond the multicodec prefix.
2. **MUST** refuse a `purpose` or `mode` it does not implement with `auth/oob/request:purposeUnsupported` or `auth/oob/request:modeUnsupported`.
3. For `login`, **MUST** accept this task only from the portal's own origin, with no cross-origin access, where the transport can tell. For `deviceEnrol`, the service is a VTA and the request's origin is the origin of its `TrustTaskHTTPS` service endpoint ([`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §7, A6); a service that is not a VTA, or has no such endpoint, **MUST** refuse `deviceEnrol` with `auth/oob/request:purposeUnsupported`.
4. **MUST** generate `requestId` as 16 to 32 bytes from a cryptographically secure random source, written as unpadded base64url; 16 bytes is RECOMMENDED.
5. **MUST** store the request as `pending` with the starter key, purpose, mode, origin, for `deviceEnrol` the `client` and `requestedScope` exactly as received, the starter's coarse connection details (city and country, browser family, OS, creation time) and its egress address, and **MUST** set `claimClosesAt` no more than 180 s after accepting the request (120 s RECOMMENDED).
5a. For `deviceEnrol`, **MAY** refuse a client kind it does not enrol with `auth/oob/request:clientKindUnsupported`, and a ceiling it would never grant with `auth/oob/request:scopeUnsupported`, before opening the request. These and the family codes are the refusals a client sees for `deviceEnrol` ([`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §6).
6. **SHOULD** apply per-network limits on this task and on pending requests, and refuse with `auth/oob:rateLimited`. For `deviceEnrol` it **MUST**, and **MUST** also cap the unclaimed `deviceEnrol` requests pending at once across all networks; a claimed request no longer counts against that cap, so claimed requests, including an attacker's self-claimed ones, are bounded by the per-network limits on `request` and `claim` and by the decision window, and it **MAY** also cap them. It **SHOULD** let an administrator see and cancel pending `deviceEnrol` requests ([`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §5 rule 8, §7 A4).
7. **MUST** refuse a `deviceEnrol` request sent as an earlier version of this task — `auth/oob/request/0.2` or before, which cannot express it — with `auth/oob/request:purposeUnsupported`, or with the framework's payload-validation refusal, `malformedRequest`, which comes first because the earlier version's closed purpose list does not contain `deviceEnrol` ([`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §5 rule 3c).

## Authorization

This task is open to any caller: opening a request needs no standing at the service. For `deviceEnrol` that makes it the one unauthenticated task a VTA serves, which is why its rate limits are a **MUST**. That is safe because a pending request is inert. It confers nothing until a member claims it, proves membership, enters the number from the starter's screen and approves with user verification, and even then the authority goes only to the key that signed this document, through `auth/oob/redeem`. A `deviceEnrol` request confers nothing either: `requestedScope` is a ceiling the approver may narrow, never a grant, and it is shown to an administrator of the VTA only after that administrator has proved who they are and entered the number. The proof here establishes which key is the starter key; it does not authorize anything ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- **`purpose`** — what the request is for, from the closed `Purpose` list ([`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §3). This version defines `login` and `deviceEnrol`.
- **`mode`** — how the request reaches the approver, from the closed `Mode` list. This version defines `scan`.
- **`client`** — for `deviceEnrol`, who asks to be enrolled: a `consumerKind` (a `companion` with its form factor, or a `service` — `mediator`, `aiAgent` or `daemon`) and a `displayName`. Self-asserted.
- **`requestedScope`** — for `deviceEnrol`, the ceiling the client asks for: `capabilities` (a list of capability references; empty asks for none), `purposes`, `keys`, and `maxLifetimeDays`. Where the client may act is not part of it; the approver chooses the context.
- **`requestId`** (response) — the handle of the request; also its nonce and the trigger link's `_id`.
- **`claimClosesAt`** (response) — when the claim window closes, as an RFC 3339 date-time. The trigger link's `_exp` carries the same instant in the link's own integer form; the starter converts `claimClosesAt` into it when it builds the link.

## Request

The starter (`issuer`, its fresh key) asks the service (`recipient`) to open a request. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A browser opens a sign-in request

```json
{
  "id": "urn:uuid:3f2a1c4e-7b6d-4e8f-9a01-b2c3d4e5f601",
  "type": "https://trusttasks.org/spec/auth/oob/request/0.3",
  "issuer": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-10T10:00:00Z",
  "payload": {
    "purpose": "login",
    "mode": "scan"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp#z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
    "created": "2026-10-10T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

### An AI agent asks to be enrolled at its owner's VTA

```json
{
  "id": "urn:uuid:5e1d2c3b-4a59-4687-9a01-b2c3d4e5f611",
  "type": "https://trusttasks.org/spec/auth/oob/request/0.3",
  "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "recipient": "did:web:vta.alice.example",
  "issuedAt": "2026-10-10T10:00:00Z",
  "payload": {
    "purpose": "deviceEnrol",
    "mode": "scan",
    "client": {
      "consumerKind": { "kind": "service", "serviceKind": "aiAgent" },
      "displayName": "Research agent on laptop"
    },
    "requestedScope": {
      "capabilities": [
        { "capability": "memory.read" },
        { "capability": "memory.write" }
      ],
      "purposes": { "scope": "listed", "purposes": ["taskExecution"] },
      "keys": { "scope": "none" },
      "maxLifetimeDays": 90
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH#z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
    "created": "2026-10-10T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## Response

The service answers with the request's handle and the instant its claim window closes. The payload is the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document. The starter then shows the trigger link built from the service DID, `requestId` and `claimClosesAt` (converted to the link's `_exp`), and starts long-polling `auth/oob/redeem`.

### The request is open

```json
{
  "id": "urn:uuid:3f2a1c4e-7b6d-4e8f-9a01-b2c3d4e5f602",
  "type": "https://trusttasks.org/spec/auth/oob/request/0.3#response",
  "threadId": "urn:uuid:3f2a1c4e-7b6d-4e8f-9a01-b2c3d4e5f601",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
  "issuedAt": "2026-10-10T10:00:00Z",
  "payload": {
    "requestId": "q3Vt7mXo9LwA2bR5cJ8kNg",
    "claimClosesAt": "2026-10-10T10:02:00Z"
  }
}
```

### An unsupported purpose

```json
{
  "id": "urn:uuid:3f2a1c4e-7b6d-4e8f-9a01-b2c3d4e5f603",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:3f2a1c4e-7b6d-4e8f-9a01-b2c3d4e5f601",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
  "issuedAt": "2026-10-10T10:00:00Z",
  "payload": {
    "code": "auth/oob/request:purposeUnsupported",
    "message": "This service does not implement that purpose.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request carries a purpose and a mode, under a key that exists only for this request; for `deviceEnrol`, also the client's kind, a name it gives itself, and the scope it asks for. The response carries a random handle and a deadline. The service additionally observes the starter's egress address, from which it derives city and country, and the User-Agent, from which it derives browser family and OS; it keeps the address only until the request ends and never returns it to anyone.

### Correlation

The starter key is fresh for each request, so requests cannot be joined by key. The service is declared with identifier scope `public` because the starter must address the community's own well-known DID, the one its portal publishes and the approver already knows. The service can join a request to a network address while it is open; that is what the same-network signal needs, and it is why the address is discarded afterwards.

### Retention

The request record lives for the life of the request (`exchange`). An audit record kept afterwards holds city and country, never the address.

### Consent/purpose

The connection details are collected to show the approver, at step 2, where the request came from, and to compute the same-network signal. They serve no other purpose and are not shown to anyone before a member has proved membership and entered the number. The same holds for a `deviceEnrol` request's `client` and `requestedScope`: they are shown only to an administrator of the VTA who has proved who they are and entered the number, and kept only for the life of the request and the audit record of the enrolment.

## Pending confirmation

This version takes the readings in [`CONVENTIONS.md`](../../_shared/0.2/CONVENTIONS.md) §7 and will follow the family authors' answer when they give one. Those that bear on this task: **A3**, **A4**, **A6**, **A10** and **A14**.
