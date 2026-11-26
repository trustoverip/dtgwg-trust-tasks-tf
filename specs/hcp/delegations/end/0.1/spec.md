---
slug: hcp/delegations/end
version: "0.1"
title: "HCP — Relinquish or End Delegation"
summary: A Management Client, acting for a delegate or the person, relinquishes or ends a Human Context Protocol delegation, ending every grant made under it; carries the HCP 2.0-draft-3 Relinquish or end delegation operation of the Delegated Authorization Profile.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Relinquish or end delegation
category: access-control
keywords: [hcp, human-context-protocol, delegation, guardianship, governance]
parties:
  - role: Management Client (HCP), acting for the delegate, the Context Subject, or the delegate who appointed a sub-delegation
    requirement: REQUIRED
    member: issuer
  - role: HCP Provider
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    Ending a delegation ends every grant made under it and changes who may act for the person, and exercises the
    caller's own authority, so the request is signed: the provider can show which Management Client ended the
    delegation, and when, independently of the transport. The response proof is RECOMMENDED so the Management Client
    can show the provider acknowledged it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task is destructive and acts for the person or the delegate, so it is consequential; a re-delivered ending
    must be placeable in the provider's duplicate-execution window.
sideEffects:
  level: destructive
  rationale: >-
    The delegation ends and cannot be resumed: a change is a new delegation, which only the Authorization Service
    establishes (HCP 17.2). Every grant authorized under it ends at that moment, and every sub-delegation made under
    it ends too; no grant carries over (HCP 17.6).
exposure:
  discloses: none
  actsAsSubject: true
  ingests: secret
  rationale: >-
    The Management Client exercises the authority of the delegate over their own delegation, or of the Context
    Subject over an appointed one, and its effect is attributed to them in the audit log. The request carries the
    Management Client's access token, a bearer secret.
retention:
  class: durable
  rationale: >-
    The provider sets the delegation's `ended` and appends `delegation.ended`, and `grant.expired` for each grant
    that ends (HCP 17.6, 8.1), which are the person's record of who ended the delegation and what it ended.
    The access token is not retained: a kept request holds its digest instead (see Security & Privacy, The access token).
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence", here the Management
      Client's access token.
    retryable: false
related:
  - hcp/delegations/list
  - hcp/grants/revoked
  - hcp/grants/revoke
  - hcp/audit/read
---

## Abstract

The **HCP — Relinquish or End Delegation** Trust Task carries the Relinquish or end delegation operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) Delegated Authorization Profile (HCP 17.6, 17.5 and 17.10; `DELETE /v2/delegations/{delegationId}` in HCP 9.2 and 9.4.13). A *Management Client* tells an *HCP Provider* to end a delegation: acting for the delegate, it relinquishes it; acting for the *Context Subject*, it ends an appointed one; acting for a delegate who appointed a sub-delegation, it ends that sub-delegation. Every grant made under the delegation ends with it, and each client holding one is told, as for a revocation. Delegations are read with [`hcp/delegations/list`](../../list/0.1/spec.md).

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** conform to the Delegated Authorization Profile (HCP 17.1, 9.4.13) and meet every requirement it places on ending a delegation; the ones that bear on the document are restated below, unchanged. Conformance to HCP is HCP's ([SPEC §9.4](/SPEC.md#94-carried-protocols)): a provider **MUST NOT** list a conformance identifier for this carriage in its HCP `conformsTo`, and advertises the task through `trust-task-discovery`, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)). HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1); this specification takes it that a carriage other than that binding may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and carries this one as a Trust Task on that reading, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

A provider that does not conform to the profile does not expose this operation (HCP 9.4.13): it **MUST NOT** advertise this task, and answers a request for it with `unsupportedType`, as [`hcp/delegations/list`](../../list/0.1/spec.md#conformance) explains.

This specification adds one check HCP does not make: the token binding of [The access token](#the-access-token), a carriage rule of the `hcp/*` family, not HCP's. [SPEC §9.4](/SPEC.md#94-carried-protocols) item 6 places a check the carried protocol does not make in a separate specification that does not declare `carries`. This family keeps this one in the carriage, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)), because it protects the token that the carriage itself moves from the HTTP `Authorization` header into the document, where it can be stored or forwarded with the document; a separate task could not protect the document this one carries.

## Authorization

The authority, and the reason the delegation ends with, are fixed by **who the Management Client acts for**, which the provider establishes from the access token in `accessToken`, validated exactly as over HCP's HTTP binding. The request carries no reason; the provider records the one the caller's authority gives (HCP 17.6). HCP names only its HTTP Binding as the carriage for a governance operation (HCP 5.4, 9.1); this specification accepts the operation as a Trust Task from a Management Client acting for the Authorizing Principal, and from no other caller (HCP 6.1.9), *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

| Management Client acting for | May end | Reason recorded |
|---|---|---|
| The delegate the delegation names | That delegation, whatever its basis (HCP 9.4.13, 17.6) | `relinquished` |
| The Context Subject | A delegation whose `basis.method` is `appointed` (HCP 9.4.13, 17.5, 17.6) | `ended-by-subject` |
| The delegate who appointed a sub-delegation | That sub-delegation (HCP 17.6, 17.10), *pending HCP confirmation* ([question 32](#pending-hcp-confirmation)) | `ended-by-delegate` |
| The Context Subject | A sub-delegation whose parent's `basis.method` is `appointed`, since they may end it "wherever they may end the parent" (HCP 17.10), *pending HCP confirmation* ([question 32](#pending-hcp-confirmation)) | `ended-by-subject` |

Any other caller receives not authorized (HCP 9.4.13), carried as `permissionDenied`: among them the Context Subject for a delegation resting on any basis other than `appointed` (a court order, a parent's authority, an attested relationship), which ends only as HCP 17.6 sets out, though its grants can always be revoked with [`hcp/grants/revoke`](../../../grants/revoke/0.1/spec.md) (HCP 17.5). A grant held by a Context Client never authorizes this task, whatever its scopes (HCP 6.1.9).

The REQUIRED `proof` establishes which Management Client VID sent the document and that it is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). It is not the authorization: the token is. A token issued to a client other than the one the `issuer` VID identifies is refused with `permissionDenied` (see [The access token](#the-access-token)).

## Definitions

- **Management Client**, **Context Client**, **HCP Provider**, **Authorization Service**, **Context Subject**, **Authorizing Principal**, **delegate**, **delegation**, **sub-delegation** and **grant** have their HCP 2.0-draft-3 meanings.
- HCP defines no request or result schema for this operation. The members come from HCP 9.2 (the path) and 9.4.13.

### HCP field mapping

| HCP field | Member | Notes |
|---|---|---|
| path parameter `{delegationId}` (`DELETE /v2/delegations/{delegationId}`, HCP 9.2) | `delegationId` | Moved from the path into the payload, unchanged. |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result `204 No Content` | the empty `#response` payload | No body in HCP, so none here. |

**Where this payload is stricter than HCP's shapes.** Each payload is closed at its top level (`additionalProperties: false`): a top-level member outside this mapping is refused, where HCP's own request and result shapes leave the top level open, so an extension member HCP would allow there is not carried. The HCP shapes nested within stay open, as HCP's are. Every size bound, and every non-empty requirement (a `minLength` or `minItems` of 1) on a member that HCP's own schema allows to be empty, is a Trust Tasks framework bound on what an `hcp/*` party writes, not an HCP requirement, as the schema descriptions say.

### Parties

| HCP role | Trust Task member |
|---|---|
| Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../../entries/query/0.1/spec.md#parties), a provider **MUST** make its VID verifiably bound to its HCP identity (its discovery document, HCP 19.1), and a Management Client **MUST** check that binding before it sends a token to a VID.

## Request

The Management Client sends the ending to the provider. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json): `delegationId`, the delegation to end, and `accessToken`, the Management Client's token. Both are REQUIRED. There is no `reason` member: the reason follows from who the caller acts for ([Authorization](#authorization)).

Before it sends the request, a Management Client **SHOULD** tell the person it acts for that every grant made under the delegation, and every sub-delegation made under it, will end at once, and that no grant carries over (HCP 17.6).

### A caregiver relinquishes a delegation

```json
{
  "id": "urn:uuid:4a6c8e0f-2b3d-4f5a-97c9-1e3a5c7e9b20",
  "type": "https://trusttasks.org/spec/hcp/delegations/end/0.1#request",
  "issuer": "did:web:wallet.caregiver.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T13:00:00Z",
  "threadId": "urn:uuid:6c8e0a2b-4d5f-4b7c-a9e1-3c5e7a9c1d42",
  "payload": {
    "delegationId": "urn:uuid:5b7d9f1a-3c4e-4a6b-88d0-2e4a6c8e0f31",
    "accessToken": "hcp_mt_c41e7a2d9b6f3058"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T13:00:00Z",
    "verificationMethod": "did:web:wallet.caregiver.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z5Nc8vX1zL4kJ7hG0fD3sA6pO9iU2yT5rE8wQ1mN4bV7cX0zL3kJ6hG9fD2sA5pO8iU1yT4rE7wQ0mN3bV6c"
  }
}
```

## Response

The provider answers once the delegation has ended and its consequences are recorded. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): the empty object, which carries HCP's `204 No Content`. The response is declared, although it carries nothing, because the ending is consequential and its completion is relied upon ([SPEC §4.4.2](/SPEC.md#442-acknowledging-a-fire-and-forget-task)). Failures use `trust-task-error`, not a `#response` document (see [Errors](#errors)).

### The delegation has ended

```json
{
  "id": "urn:uuid:8e0a2c4d-6f7b-4d9e-b1a3-5e7c9e1b3d53",
  "type": "https://trusttasks.org/spec/hcp/delegations/end/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:wallet.caregiver.example",
  "issuedAt": "2026-11-25T13:00:01Z",
  "threadId": "urn:uuid:6c8e0a2b-4d5f-4b7c-a9e1-3c5e7a9c1d42",
  "payload": {}
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows.

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request` | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: the caller is not one of those in [Authorization](#authorization) for this delegation (HCP 9.4.13, 17.5, 17.10), or the `delegationId` is unknown or not visible to the caller (see [Repeated requests](#repeated-requests)) | `permissionDenied`, with one fixed `message` whatever the reason (HCP 5.5, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| conflict | `idempotency-conflict` | Does not arise: HCP defines it for a retried request with an idempotency key (HCP 9.6), and this operation carries none. A re-delivered document is handled under [Repeated requests](#repeated-requests). |
| version conflict | `version-conflict` | Does not arise: a delegation has no version (HCP 17.2). |
| credential invalid | `credential-invalid` | Does not arise: it concerns a credential written as an entry's content (HCP 14.6). |
| subject unavailable | `subject-unavailable` | `unavailable` |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion` |

A provider that does not conform to the Delegated Authorization Profile answers `unsupportedType` (see [Conformance](#conformance)).

## Rules carried from HCP

When a delegation ends (HCP 17.6):

- **Every grant under it ends at that moment.** Its `status` becomes `expired`, and the provider appends `grant.expired` with `data.reason` `delegation-ended` (HCP 8.2). Invalidation and notification follow HCP 6.2.2–6.2.4, as for a revocation: no request presenting evidence issued under such a grant succeeds, and each client holding one is sent the grant-revoked notice. Over Trust Tasks that notice is [`hcp/grants/revoked`](../../../grants/revoked/0.1/spec.md) with `reason: "delegation-ended"` and `revokedAt` the instant the delegation ended, *pending HCP confirmation* ([question 17](#pending-hcp-confirmation)). The delegation's own reason (`relinquished`, `ended-by-subject`, ...) is never sent to a client.
- **Every sub-delegation under it ends**, with reason `parent-ended` (HCP 17.6, 17.10), and with it every grant made under the sub-delegation, as above.
- **The provider appends `delegation.ended`**, recording `delegationId` and `reason` (HCP 8.1, 8.2), and sets the delegation's `ended` to the instant and the reason. Like every event other than `context.read`, it is durable before this task's response is sent (HCP 8.4).
- **Entries written under the delegation's grants are unaffected.** They are the Context Subject's, and their provenance shows the grant under which each was written (HCP 17.6, 3.5).
- **No grant carries over.** The Context Subject authorizes any access they want to continue (HCP 17.6).
- **No change, only an end.** Neither the Context Subject nor the delegate may change a delegation; a change is a new delegation, which only the Authorization Service establishes (HCP 17.2).
- **Accountability survives.** A former delegate keeps reading the events of the grants they authorized, for as long as the log is kept (HCP 17.7; [`hcp/audit/read`](../../../audit/read/0.1/spec.md)).

## Repeated requests

HCP 2.0-draft-3 does not say what ending an already-ended delegation yields (HCP 9.4.13). This specification reads HCP as follows, *pending HCP confirmation* ([question 27](#pending-hcp-confirmation)):

- A caller who could end the delegation while it held, and asks to end it after it has ended, for any reason, succeeds idempotently with the empty response. The provider appends no second `delegation.ended` event, changes no `ended` member, and sends no second notice. The authorization check comes first: any other caller receives `permissionDenied`.
- A `delegationId` the provider does not hold, or one not visible to the caller, is refused with `permissionDenied`, with the same fixed `message` as every other not-authorized refusal, so that a caller cannot tell an unknown delegation from one it may not end.

A re-delivered document with the same `id` and content is absorbed by [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements): it is not executed twice, and is answered with the original empty response.

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 1 — conformance claims.** HCP defines no conformance identifier for Trust Task carriage, so a provider **MUST NOT** claim this carriage in its HCP `conformsTo`; it advertises the task with [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead. The binding identifier for the carriage is the same question as [question 37](#pending-hcp-confirmation): whether a carriage other than the HTTP Binding may carry governance operations at all.
- **Question 17 — notice to clients.** Each grant that ends with the delegation is announced with [`hcp/grants/revoked`](../../../grants/revoked/0.1/spec.md#pending-hcp-confirmation), carrying `reason: "delegation-ended"` and `revokedAt` the instant the delegation ended.
- **Question 22 — the client's identity.** The `issuer` VID, authenticated by the request's `proof`, identifies the HCP client the token was issued to; a token issued to any other client **MUST** be refused with `permissionDenied`. This token binding is a carriage rule of this family, a check HCP does not itself make (see [The access token](#the-access-token) and [Conformance](#conformance)).
- **Question 27 — an already-ended or unknown delegation.** Ending an ended delegation succeeds idempotently, with no second `delegation.ended` event; an unknown one is refused with `permissionDenied` (see [Repeated requests](#repeated-requests)).
- **Question 32 — ending a sub-delegation.** HCP 17.10 governs where HCP 9.4.13 is narrower: the delegate who appointed a sub-delegation may end it (`ended-by-delegate`), and so may the Context Subject where the parent's basis is `appointed` (`ended-by-subject`) (see [Authorization](#authorization)).
- **Question 37 — governance over a carriage other than the HTTP Binding.** HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1). This specification takes it that another carriage, here a Trust Task, may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and applies every rule HCP places on the operation unchanged (see [Conformance](#conformance)).
- **Question 53 — a request proof on every request that carries a token.** A request `proof` is **REQUIRED** on every `hcp/*` request that carries `accessToken`, so that the token binding of question 22 always applies and a token lifted from a stored or forwarded document cannot be replayed in an unsigned request. This is a carriage rule of this family, not HCP's, and is stricter than HCP's bearer model, under which presenting a valid token is enough (HCP 9.3); see [The access token](#the-access-token).

## What HCP requires that a document cannot express

- The provider-held delegation and its grants, and the check of who may end it (HCP 9.4.13, 17.5, 17.10).
- Ending every grant and sub-delegation under it, invalidating their evidence, and notifying each client (HCP 17.6, 6.2.2–6.2.4).
- The `delegation.ended` and `grant.expired` audit events and their durability (HCP 8.1, 8.4).
- Carriage by the HTTP Binding: HCP carries governance operations only over that binding (HCP 1.6, 5.4, 9.1), and a Trust Task is not it. This specification carries the operation for a Management Client acting for the Authorizing Principal (HCP 6.1.9) instead, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

## Security & Privacy

### Data carried

The request carries a delegation identifier and the Management Client's access token, a bearer secret that **MUST** travel only to the provider. It carries no context and no reason. The response carries nothing. A producer **MUST NOT** put the token or any detail of the delegation in `ext`.

### The access token

`accessToken` is a bearer secret: whoever presents it exercises the authority it evidences. It **MUST** travel only to the provider. A recipient **MUST NOT** log `accessToken` or retain it beyond the exchange.

**Token binding, a carriage rule of this family.** Every request that carries `accessToken` carries a `proof`, *pending HCP confirmation* ([question 53](#pending-hcp-confirmation)), and the provider **MUST** refuse with `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies, so that a token lifted from a stored or forwarded document is of no use from another VID. HCP validates the token alone and does not bind it to a VID (HCP 9.3): this rule is the `hcp/*` family's, not HCP's, and rests on the `issuer` VID identifying the HCP client, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)). It is a check HCP does not make (see [Conformance](#conformance)).

**Retained copies.** The provider verifies the request's `proof` over the document as received, at receipt, before it removes anything ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). A provider that retains the request document, as a duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) or as evidence of the request, **MUST** remove `accessToken` from the copy it keeps and record in its place `accessTokenDigest`: the SHA-256 multihash of the token's UTF-8 bytes, multibase-encoded as base64url without padding (prefix `u`), the framework's `DigestMultibase` form ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)). The kept copy's `proof` no longer verifies over it, so the provider records with the copy that the proof was verified at receipt: when, under which verification method, and with what result. It relies on that record, and **MUST NOT** verify the proof again over the kept copy. A record that keeps only a digest of the document, such as its task digest ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)), keeps no token either.

### Correlation

The provider learns which Management Client ended which delegation and when; that is the record the person needs. A `delegationId` is the same in every grant made under the delegation, which is why HCP never shows it to a Context Client (HCP 17.4); it is known only to the provider and the Management Clients of the Context Subject and the delegate. The notice each client receives names its own grant and `delegation-ended`, never the delegation or its reason. `threadId` **SHOULD** be fresh per request.

### Retention

The provider keeps the delegation record with its `ended` member, the `delegation.ended` event, and a `grant.expired` event for each grant that ended (HCP 17.6, 8.1), readable by the Context Subject and, for the grants they authorized, by the former delegate (HCP 17.7). The signed request is evidence of who ended the delegation and when, and a provider **SHOULD** keep it with that event, with `accessToken` replaced by its digest and a record that its proof was verified at receipt (see [The access token](#the-access-token)).

### Consent/purpose

Ending a delegation is the delegate's right to stop acting for the person, the Context Subject's right over a delegate they appointed themselves (HCP 17.5), and an appointing delegate's right over their sub-delegate (HCP 17.10). It cuts off every client holding a grant under the delegation, which may interrupt services the person relies on; that is why the Management Client **SHOULD** say so before sending ([Request](#request)), and why the Context Subject is free to authorize new grants afterwards. Whether a Management Client asks for confirmation is its own policy ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).
