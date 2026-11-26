---
slug: hcp/grants/relinquish
version: "0.1"
title: "HCP — Relinquish Own Grant"
summary: A Context Client gives up the Human Context Protocol grant under which it acts, which revokes it; carries the HCP 2.0-draft-3 Relinquish own grant operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Relinquish own grant
category: access-control
keywords: [hcp, human-context-protocol, grant, relinquish, revocation]
parties:
  - role: Context Client (HCP), the client the grant was issued to
    requirement: REQUIRED
    member: issuer
  - role: HCP Provider
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    Relinquishing is irreversible and is recorded as a revocation naming the client as initiator (HCP 5.2.7), so the
    request is signed: the provider can show which client gave the grant up, and when. The response proof is
    RECOMMENDED so the client can show the provider accepted it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task is destructive and so consequential; a re-delivered request must be placeable in the provider's
    duplicate-execution window.
sideEffects:
  level: destructive
  rationale: >-
    Relinquishing is a revocation of the grant (HCP 5.2.7): the grant ends at once (HCP 6.2.2), and restoring access
    needs a new grant, given by the person again.
exposure:
  discloses: none
  actsAsSubject: false
  ingests: secret
  rationale: >-
    The client gives up its own authority, not the person's. The request carries the HCP access token, a bearer secret.
retention:
  class: durable
  rationale: >-
    The provider records a `grant.revoked` audit event naming the client as initiator (HCP 3.9, 5.2.7), which is the
    person's record that access ended and why.
    The access token is not retained: a kept request holds its digest instead (see Security & Privacy, The access token).
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence". A token issued under a
      grant already revoked or relinquished also fails here, pending HCP confirmation (question 2 in the spec's
      "Pending HCP confirmation" section).
    retryable: false
related:
  - hcp/grants/revoke
  - hcp/grants/revoked
  - hcp/grants/read-own
---

## Abstract

The **HCP — Relinquish Own Grant** Trust Task carries the Relinquish own grant operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 5.2.7; `DELETE /v2/grants/{grantId}` with a Context Client's token in HCP 9.2). A *Context Client* gives up the grant under which it acts. Relinquishing is a revocation of that grant: it takes effect at once, and is recorded as a revocation the client initiated. No revocation notice is sent to the client that relinquished (HCP 5.2.7).

HCP's MCP binding does not carry this operation in 2.0-draft-3 (HCP 5.2.7); this task carries it over any Trust Task binding. It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on relinquishment and revocation (HCP 5.2.7, 6.2.2). A provider **MUST NOT** claim this carriage in its HCP `conformsTo`, and advertises the task through [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)).

This specification adds one check HCP does not make: the token binding of [The access token](#the-access-token), a carriage rule of the `hcp/*` family, not HCP's. [SPEC §9.4](/SPEC.md#94-carried-protocols) item 6 places a check the carried protocol does not make in a separate specification that does not declare `carries`. This family keeps this one in the carriage, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)), because it protects the token that the carriage itself moves from the HTTP `Authorization` header into the document, where it can be stored or forwarded with the document; a separate task could not protect the document this one carries.

## Authorization

The authority is **the grant itself**, evidenced by the access token in `accessToken`. A client can relinquish only the grant it presents: a request whose `grantId` is not the grant the token was issued under **MUST** fail with not authorized (HCP 5.2.7), carried as `permissionDenied`. Revoking any other grant is a governance operation, [`hcp/grants/revoke`](../../revoke/0.1/spec.md) (HCP 5.2.7, 6.2.1).

Over HCP's HTTP binding the same `DELETE /v2/grants/{grantId}` with a Management Client's token revokes any grant (HCP 9.4.6). This task carries relinquishment only: a Management Client's token is refused with `permissionDenied`, and a Management Client revokes with `hcp/grants/revoke`.

The REQUIRED `proof` establishes which client VID sent the document ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); it is not the authorization. A token issued to a client other than the one the `issuer` VID identifies is refused with `permissionDenied` (see [The access token](#the-access-token)).

## Definitions

- **HCP Client**, **Context Client**, **Management Client**, **HCP Provider** and **grant** have their HCP 2.0-draft-3 meanings.
- HCP defines no request or result schema for this operation. The members come from HCP 9.2 (the path), 9.4.6 and 5.2.7.

### HCP field mapping

| HCP field | Member | Notes |
|---|---|---|
| path parameter `{grantId}` (`DELETE /v2/grants/{grantId}`, HCP 9.2) | `grantId` | Moved from the path into the payload, unchanged. |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result `204 No Content` (HCP 9.4.6) | the empty `#response` payload | HCP defines no result, so none here. |

### Parties

| HCP role | Trust Task member |
|---|---|
| Context Client (HCP 5.2.7 says HCP Client; a Management Client's token is refused, see [Authorization](#authorization)) | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../../entries/query/0.1/spec.md#parties), a provider **MUST** make its VID verifiably bound to its HCP identity (HCP 19.1).

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json): `grantId`, the client's own grant, and `accessToken`, the token issued under it. Both are REQUIRED.

### The booking agent gives up its grant

```json
{
  "id": "urn:uuid:3b8f0d26-5c2a-4e9b-a1d7-6f3c9e0b2d36",
  "type": "https://trusttasks.org/spec/hcp/grants/relinquish/0.1#request",
  "issuer": "did:web:booking-agent.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T14:00:00Z",
  "threadId": "urn:uuid:9d4a2f17-6e3b-4c8d-b5a0-1e7c4b9f2a47",
  "payload": {
    "grantId": "urn:uuid:c4d2e1f0-9a8b-4c7d-8e6f-5a4b3c2d1e0f",
    "accessToken": "hcp_at_2b7f0c6e9d1a4e8b"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T14:00:00Z",
    "verificationMethod": "did:web:booking-agent.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z2LnV8yXu4ZpO7xTe3MdK2rLi8fCh9wGb6qUkP5zX3vF7nD4tY1oS2iH6jL9mR3cW8eB5gQ7uZ4fX2aT1wO6dM3y"
  }
}
```

## Response

The provider answers when the grant is relinquished. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): the empty object, which carries HCP's `204 No Content`. It is declared, although it carries nothing, because the relinquishment is consequential and its completion is relied upon ([SPEC §4.4.2](/SPEC.md#442-acknowledging-a-fire-and-forget-task)). Failures use `trust-task-error` (see [Errors](#errors)).

### Relinquished

```json
{
  "id": "urn:uuid:7c3e9b48-1a6d-4f2c-8e9b-4d0a5c7e3f58",
  "type": "https://trusttasks.org/spec/hcp/grants/relinquish/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:booking-agent.example",
  "issuedAt": "2026-11-25T14:00:01Z",
  "threadId": "urn:uuid:9d4a2f17-6e3b-4c8d-b5a0-1e7c4b9f2a47",
  "payload": {}
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows.

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request` | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: a `grantId` that is not the grant the token was issued under (HCP 5.2.7), or a Management Client's token (see [Authorization](#authorization)) | `permissionDenied`, with one fixed `message` whatever the reason (HCP 5.5) |
| conflict | `idempotency-conflict` | Does not arise: relinquishment carries no idempotency key (HCP 9.6). A re-delivered document is handled under [Repeated requests](#repeated-requests). |
| version conflict | `version-conflict` | Does not arise. |
| credential invalid | `credential-invalid` | Does not arise. |
| subject unavailable | `subject-unavailable` | `unavailable` |
| rate limited | `rate-limited` | Never: relinquishment is a revocation (HCP 5.2.7), and revocation is **never rate limited** (HCP 5.5, 6.2.1). A provider **MUST NOT** refuse it with `unavailable` for throttling. |
| unsupported version | `unsupported-version` | `unsupportedVersion` |

## Rules carried from HCP

- **A revocation.** The grant ends at once: no request presenting evidence issued under it succeeds afterwards (HCP 6.2.2), and the provider appends a `grant.revoked` audit event naming the client as initiator (HCP 3.9, 5.2.7) within 60 seconds (HCP 6.2.2).
- **No notice to the client that relinquished.** The provider sends no [`hcp/grants/revoked`](../../revoked/0.1/spec.md) notice to it (HCP 5.2.7).
- **Copies.** The client stops using every copy disclosed under the grant at once, except a copy kept under a legal obligation it declared at registration (HCP 6.2.2, 6.2.5).

## Repeated requests

A re-delivered document with the same `id` and content is absorbed by [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements) and answered with the original response; it appends no second `grant.revoked` event. A new document after the grant has ended presents a token that is no longer valid evidence and fails with `hcp:invalidToken`, *pending HCP confirmation* ([question 2](#pending-hcp-confirmation)).

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 1 — conformance claims.** A provider **MUST NOT** claim this carriage in its HCP `conformsTo` (see [Conformance](#conformance)).
- **Question 2 — a request under a revoked grant.** Refused with `hcp:invalidToken`, which covers a second relinquishment with a new document (see [Repeated requests](#repeated-requests)).
- **Question 22 — the client's identity.** The `issuer` VID, authenticated by the request's `proof`, identifies the HCP client the token was issued to; a token issued to any other client **MUST** be refused with `permissionDenied`. This token binding is a carriage rule of this family, a check HCP does not itself make (see [The access token](#the-access-token) and [Conformance](#conformance)).
- **Question 53 — a request proof on every request that carries a token.** A request `proof` is **REQUIRED** on every `hcp/*` request that carries `accessToken`, so that the token binding of question 22 always applies and a token lifted from a stored or forwarded document cannot be replayed in an unsigned request. This is a carriage rule of this family, not HCP's, and is stricter than HCP's bearer model, under which presenting a valid token is enough (HCP 9.3); see [The access token](#the-access-token).

## What HCP requires that a document cannot express

- The provider-held grant state, and the immediate invalidation of every token issued under the grant (HCP 6.2.2).
- The audit event and its 60-second deadline (HCP 6.2.2).
- The client's duty to stop using its copies (HCP 6.2.2).

## Security & Privacy

### Data carried

The request carries a grant identifier and the access token, a bearer secret that **MUST** travel only to the provider. It carries no context. The response carries nothing. A producer **MUST NOT** put the token in `ext`.

### The access token

`accessToken` is a bearer secret: whoever presents it exercises the authority it evidences. It **MUST** travel only to the provider. A recipient **MUST NOT** log `accessToken` or retain it beyond the exchange.

**Token binding, a carriage rule of this family.** Every request that carries `accessToken` carries a `proof`, *pending HCP confirmation* ([question 53](#pending-hcp-confirmation)), and the provider **MUST** refuse with `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies, so that a token lifted from a stored or forwarded document is of no use from another VID. HCP validates the token alone and does not bind it to a VID (HCP 9.3): this rule is the `hcp/*` family's, not HCP's, and rests on the `issuer` VID identifying the HCP client, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)). It is a check HCP does not make (see [Conformance](#conformance)).

**Retained copies.** The provider verifies the request's `proof` over the document as received, at receipt, before it removes anything ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). A provider that retains the request document, as a duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) or as evidence of the request, **MUST** remove `accessToken` from the copy it keeps and record in its place `accessTokenDigest`: the SHA-256 multihash of the token's UTF-8 bytes, multibase-encoded as base64url without padding (prefix `u`), the framework's `DigestMultibase` form ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)). The kept copy's `proof` no longer verifies over it, so the provider records with the copy that the proof was verified at receipt: when, under which verification method, and with what result. It relies on that record, and **MUST NOT** verify the proof again over the kept copy. A record that keeps only a digest of the document, such as its task digest ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)), keeps no token either.

### Correlation

The provider learns that the client gave the grant up, and when; that is the record the person needs. The `grantId` joins this document to the client's other requests under the grant, by design. `threadId` **SHOULD** be fresh per request.

### Retention

The provider keeps the `grant.revoked` audit event (HCP 5.2.7), readable by the Context Subject (HCP 8.3). The signed request is evidence of who ended the grant, and a provider **SHOULD** keep it with that event, with `accessToken` replaced by its digest and a record that its proof was verified at receipt (see [The access token](#the-access-token)).

### Consent/purpose

Relinquishing ends the client's use of the person's permission; it needs no consent from the person, who can always grant again. The data in the document is used only to carry out and record the relinquishment ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).

The acceptance window for `issuedAt` is consumer policy; a generous window suits this task, since refusing a late relinquishment leaves alive a grant the client meant to end.
