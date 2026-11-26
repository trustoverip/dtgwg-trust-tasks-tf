---
slug: hcp/entries/withdraw
version: "0.1"
title: "HCP — Withdraw Entry"
summary: An HCP Client marks an entry deleted at a Human Context Protocol provider, hiding it from Context Clients at once; carries the HCP 2.0-draft-3 Withdraw operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Withdraw
category: data-exchange
keywords: [hcp, human-context-protocol, personal-context, memory, entry, soft-delete]
parties:
  - role: HCP Client (a Context Client, or a Management Client)
    requirement: REQUIRED
    member: issuer
  - role: HCP Provider
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    A withdrawal hides an entry from every Context Client and is recorded for the person, so the request is signed:
    the provider can show which client withdrew which entry, and when. The response proof is RECOMMENDED so the client
    can show the provider accepted the withdrawal.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task changes the person's context and is consequential; a re-delivered withdrawal must be placeable in the
    provider's duplicate-execution window, so that it appends no second audit event.
sideEffects:
  level: mutating
  rationale: >-
    The entry's status becomes `deleted` and the provider appends a `context.withdrawn` audit event (HCP 5.2.5, 8.1).
    The entry is still held, and the Context Subject still sees it, until it is hard-deleted (HCP 5.2.5); nothing is
    destroyed.
exposure:
  discloses: none
  actsAsSubject: false
  ingests: secret
  rationale: >-
    The request names an entry and carries the HCP access token, a bearer secret, and a reason. The response carries
    no context.
retention:
  class: durable
  rationale: >-
    The provider keeps the withdrawn entry until it is hard-deleted, and the `context.withdrawn` audit event with the
    `reason` (HCP 5.2.5, 8.1).
    The access token is not retained: a kept request holds its digest instead (see Security & Privacy, The access token).
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence". A request under a
      revoked grant also fails here, pending HCP confirmation (question 2 in the spec's "Pending HCP confirmation"
      section).
    retryable: false
related:
  - hcp/entries/create
  - hcp/entries/revise
  - hcp/entries/get
---

## Abstract

The **HCP — Withdraw Entry** Trust Task carries the Withdraw operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 5.2.5; `DELETE /v2/entries/{entryId}` in HCP 9.2). An *HCP Client* marks an entry deleted. The entry is hidden from *Context Clients* at once, but stays held, and visible to the *Context Subject*, until it is hard-deleted, which is a governance operation and not this task (HCP 5.4.2).

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and does not copy HCP's own schema files. Where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on the Withdraw operation. A provider **MUST NOT** claim this carriage in its HCP `conformsTo`, and advertises the task through [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)).

This specification adds one check HCP does not make: the token binding of [The access token](#the-access-token), a carriage rule of the `hcp/*` family, not HCP's. [SPEC §9.4](/SPEC.md#94-carried-protocols) item 6 places a check the carried protocol does not make in a separate specification that does not declare `carries`. This family keeps this one in the carriage, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)), because it protects the token that the carriage itself moves from the HTTP `Authorization` header into the document, where it can be stored or forwarded with the document; a separate task could not protect the document this one carries.

## Authorization

The authority is **the grant** (HCP 6.1), evidenced by the access token in `accessToken`, which the provider checks on every request (HCP 6.2.2). The entry **MUST** be one the grant lets the caller write, and a Context Client names only an entry disclosed to it under the same grant (HCP 5.2.2, 5.2.5); anything else is refused with `permissionDenied`, with one fixed `message` (HCP 5.5). A Context Client that names the entry by `entryId` is refused the same way, *pending HCP confirmation* ([question 6](#pending-hcp-confirmation)).

This task never hard-deletes. HCP's HTTP binding hard-deletes on the same path with `hardDelete=true`, which only a Management Client acting for the Authorizing Principal may do (HCP 5.4.2, 9.4.5); that governance operation is not carried here.

A token issued to a client other than the one the `issuer` VID identifies is refused with `permissionDenied` (see [The access token](#the-access-token)).

## Definitions

- **Context Client**, **Management Client**, **HCP Provider**, **Context Subject**, **grant** and **entry** have their HCP 2.0-draft-3 meanings.

### HCP field mapping

| HCP Withdraw field | Member | Notes |
|---|---|---|
| request `ref` or `entryId` (the `{entryId}` path segment, HCP 9.4.5) | `ref` or `entryId` | Moved from the path into the payload. Exactly one is present. |
| request `reason` | `reason` | Unchanged. |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `Idempotency-Key` header (HCP 9.6) | the document `id` | See [Repeated requests](#repeated-requests). |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result `ref` or `entryId` | `ref` or `entryId` | Unchanged: the one the request used. |
| result `status` | `status` | Unchanged: always `deleted`. |
| result `hardDeleted` (HCP 9.4.5) | `hardDeleted` | OPTIONAL, and always `false` here. HCP's HTTP binding returns it; its Core text does not list it (HCP 5.2.5); see [question 10](#pending-hcp-confirmation). |
| result `timestamp` | `respondedAt` | **Renamed**, as in `hcp/entries/query`. |

**Where this payload is stricter than HCP's shapes.** Each payload is closed at its top level (`additionalProperties: false`): a top-level member outside this mapping is refused, where HCP's own request and result shapes leave the top level open, so an extension member HCP would allow there is not carried. The HCP shapes nested within stay open, as HCP's are. Every size bound, and every non-empty requirement (a `minLength` or `minItems` of 1) on a member that HCP's own schema allows to be empty, is a Trust Tasks framework bound on what an `hcp/*` party writes, not an HCP requirement, as the schema descriptions say.

### Parties

| HCP role | Trust Task member |
|---|---|
| Context Client or Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../query/0.1/spec.md#parties), a client's anchor for the provider is the provider VID the client obtained when the person connected it, and where the provider publishes an HCP discovery document (HCP 19.1), the provider **MUST** make its VID verifiably bound to it.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json): `ref` (a Context Client) or `entryId` (a Management Client), exactly one, the schema not checking it; `reason` (REQUIRED), recorded in the audit log; and `accessToken` (REQUIRED).

### The person asks the agent to forget a preference

```json
{
  "id": "urn:uuid:5e9b2d41-7c3a-4f8e-b0d6-2a4c1e9f7b83",
  "type": "https://trusttasks.org/spec/hcp/entries/withdraw/0.1#request",
  "issuer": "did:web:booking-agent.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T12:00:00Z",
  "threadId": "urn:uuid:3a7c0e52-8b4d-4f1a-9e6c-7d2b5f0a1c94",
  "payload": {
    "ref": "hcp-ref-c73e19",
    "reason": "The person asked the agent to forget it.",
    "accessToken": "hcp_at_2b7f0c6e9d1a4e8b"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T12:00:00Z",
    "verificationMethod": "did:web:booking-agent.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z4MrU9xZt3YoN6wSd2LcJ1qKh7eBg8vFa5pTjO4yW2uE6mC3sX9nR1hG5iK8lQ2bV7dA4fP6tY3eW1zS9vN5cL2x"
  }
}
```

## Response

The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): the entry as the caller named it (a Context Client gets its `ref` back, never the `id`), `status` `deleted`, and `respondedAt`. Failures use `trust-task-error` (see [Errors](#errors)).

### Withdrawn

```json
{
  "id": "urn:uuid:9c4f1a63-2e8b-4d7c-a5f0-1b6e3d9c2a05",
  "type": "https://trusttasks.org/spec/hcp/entries/withdraw/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:booking-agent.example",
  "issuedAt": "2026-11-25T12:00:01Z",
  "threadId": "urn:uuid:3a7c0e52-8b4d-4f1a-9e6c-7d2b5f0a1c94",
  "payload": {
    "ref": "hcp-ref-c73e19",
    "status": "deleted",
    "hardDeleted": false,
    "respondedAt": "2026-11-25T12:00:01Z"
  }
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows. The provider never sends HCP's Core error object or the HTTP binding's problem details (HCP 9.7).

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request` | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: an entry that does not exist, is outside the caller's write scope, or whose `ref` belongs to another grant (HCP 5.2.5, 5.5) | `permissionDenied`, with one fixed `message` whatever the reason (HCP 5.5, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| conflict | `idempotency-conflict` (HCP 9.6) | `idConflict` (see [Repeated requests](#repeated-requests)) |
| version conflict | `version-conflict` | Does not arise: a withdrawal names no version (HCP 5.2.5). |
| credential invalid | `credential-invalid` | Does not arise: nothing is ingested (HCP 14.6). |
| subject unavailable | `subject-unavailable` | `unavailable`. The client may retry later. |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion` |

## Repeated requests

HCP's HTTP binding asks a provider to accept an idempotency key on a withdrawal (a SHOULD, HCP 9.6). This task uses the document `id` as that key, under [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements): a re-delivered document with the same `id` and content is answered with the original response and appends no second `context.withdrawn` event; a reused `id` with different content is refused with `idConflict`. The provider **MUST** retain its duplicate-execution record, with the response, for at least 24 hours (HCP 9.6), and for at least the acceptance window it applies to `issuedAt`.

A new document withdrawing an entry that is already `deleted` succeeds with the same result, as HCP says repeated deletion is harmless to entry state, and appends a further `context.withdrawn` event, as HCP says a repeat without a key does (HCP 9.6). This reading is *pending HCP confirmation* ([question 10](#pending-hcp-confirmation)).

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 1 — conformance claims.** A provider **MUST NOT** claim this carriage in its HCP `conformsTo` (see [Conformance](#conformance)).
- **Question 2 — a request under a revoked grant.** Refused with `hcp:invalidToken` (see [Errors](#errors)).
- **Question 6 — naming the entry.** Exactly one of `ref` and `entryId`; both or neither is `malformedRequest`, and a Context Client that sends `entryId` gets `permissionDenied` (see [Authorization](#authorization)).
- **Question 10 — `hardDeleted`, and withdrawing a withdrawn entry.** `hardDeleted` is carried as an OPTIONAL member that is always `false`; a new withdrawal of a `deleted` entry succeeds and is audited again (see [HCP field mapping](#hcp-field-mapping), [Repeated requests](#repeated-requests)).
- **Question 22 — the client's identity.** The `issuer` VID, authenticated by the request's `proof`, identifies the HCP client the token was issued to; a token issued to any other client **MUST** be refused with `permissionDenied`. This token binding is a carriage rule of this family, a check HCP does not itself make (see [The access token](#the-access-token) and [Conformance](#conformance)).
- **Question 53 — a request proof on every request that carries a token.** A request `proof` is **REQUIRED** on every `hcp/*` request that carries `accessToken`, so that the token binding of question 22 always applies and a token lifted from a stored or forwarded document cannot be replayed in an unsigned request. This is a carriage rule of this family, not HCP's, and is stricter than HCP's bearer model, under which presenting a valid token is enough (HCP 9.3); see [The access token](#the-access-token).

## What HCP requires that a document cannot express

- **Immediate hiding** (HCP 5.2.5): once withdrawn, the entry is hidden from every Context Client, on every read path.
- **The grant and write-scope checks**, and existence hiding (HCP 5.5).
- **Retention until hard deletion** (HCP 5.2.5, 5.4.2), and the Context Subject's continued view of the entry.

## Security & Privacy

### Data carried

The request names one entry and carries a `reason` and the access token, a bearer secret. The response carries no context. The `reason` is recorded in the audit log for the person to read; a client **SHOULD** put no context in it beyond what the withdrawal needs, and **MUST NOT** put the token in `ext` or any `message`.

### The access token

`accessToken` is a bearer secret: whoever presents it exercises the authority it evidences. It **MUST** travel only to the provider. A recipient **MUST NOT** log `accessToken` or retain it beyond the exchange.

**In an agent runtime**, the token is held as [`hcp/entries/query`](../../query/0.1/spec.md#the-access-token) requires: by the component that builds and sends the document, never in a model's context or a tool's result (HCP 10.5).

**Token binding, a carriage rule of this family.** Every request that carries `accessToken` carries a `proof`, *pending HCP confirmation* ([question 53](#pending-hcp-confirmation)), and the provider **MUST** refuse with `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies, so that a token lifted from a stored or forwarded document is of no use from another VID. HCP validates the token alone and does not bind it to a VID (HCP 9.3): this rule is the `hcp/*` family's, not HCP's, and rests on the `issuer` VID identifying the HCP client, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)). It is a check HCP does not make (see [Conformance](#conformance)).

**Retained copies.** The provider verifies the request's `proof` over the document as received, at receipt, before it removes anything ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). A provider that retains the request document, as a duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) or as evidence of the request, **MUST** remove `accessToken` from the copy it keeps and record in its place `accessTokenDigest`: the SHA-256 multihash of the token's UTF-8 bytes, multibase-encoded as base64url without padding (prefix `u`), the framework's `DigestMultibase` form ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)). The kept copy's `proof` no longer verifies over it, so the provider records with the copy that the proof was verified at receipt: when, under which verification method, and with what result. It relies on that record, and **MUST NOT** verify the proof again over the kept copy. A record that keeps only a digest of the document, such as its task digest ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)), keeps no token either.

### Correlation

The provider learns which client withdrew which entry, and when; that is the record the person needs. A Context Client names the entry by its grant-scoped `ref` (HCP 7.4). `threadId` **SHOULD** be fresh per withdrawal.

### Retention

The provider keeps the entry, as `deleted`, until it is hard-deleted, and the `context.withdrawn` audit event (HCP 8.1). A withdrawal does not end a client's duty to stop using copies it holds at their `useUntil` (HCP 7.6). A person who wants the entry gone for good asks for a hard deletion through a Management Client (HCP 5.4.2).

### Consent/purpose

Withdrawal removes context from what clients can receive; the data in the document is used only to carry out and record that. Whether a withdrawal needs any further approval is the provider's policy under HCP ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).

The acceptance window for `issuedAt` is consumer policy; a generous window suits this task, since refusing a late withdrawal leaves visible an entry someone meant to hide.
