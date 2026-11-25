---
slug: hcp/directives/remove
version: "0.1"
title: "HCP — Remove Directive"
summary: A Management Client, acting for the person or a delegate, removes a Human Context Protocol directive; carries the HCP 2.0-draft-3 Remove directive operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Remove directive
category: access-control
keywords: [hcp, human-context-protocol, directive, governance]
parties:
  - role: Management Client (HCP), acting for the Context Subject or a delegate
    requirement: REQUIRED
    member: issuer
  - role: HCP Provider
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    Removing a directive lifts a limit the person set on what clients may do, and exercises the person's own
    authority, so the request is signed: the provider can show which Management Client removed the directive, and
    when, independently of the transport. The response proof is RECOMMENDED so the Management Client can show the
    provider acknowledged it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task is destructive and acts for the person, so it is consequential; a re-delivered removal must be placeable
    in the provider's duplicate-execution window.
sideEffects:
  level: destructive
  rationale: >-
    The directive is removed and cannot be restored: setting it again makes a new directive with a new `directiveId`
    (HCP 6.4). From the moment it is removed, the provider no longer delivers it.
exposure:
  discloses: none
  actsAsSubject: true
  ingests: secret
  rationale: >-
    The Management Client exercises the Context Subject's (or a delegate's) authority over their own directives. The
    request carries the Management Client's access token, a bearer secret.
retention:
  class: durable
  rationale: >-
    The provider appends a `directive.removed` audit event (HCP 6.4, 8.1), which is the person's record of the change.
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence", here the Management
      Client's access token.
    retryable: false
related:
  - hcp/directives/set
  - hcp/directives/list
---

## Abstract

The **HCP — Remove Directive** Trust Task carries the Remove directive operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 6.4 and 5.4; `DELETE /v2/directives/{directiveId}` in HCP 9.2 and 9.4.15). A *Management Client*, acting for the *Context Subject*, or for a delegate under the Delegated Authorization Profile, tells an *HCP Provider* to remove a directive. The provider removes it and records a `directive.removed` audit event. A directive is set with [`hcp/directives/set`](../../set/0.1/spec.md) and read with [`hcp/directives/list`](../../list/0.1/spec.md).

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on directives; the ones that bear on the document are restated below, unchanged. Conformance to HCP is HCP's ([SPEC §9.4](/SPEC.md#94-carried-protocols)): a provider **MUST NOT** list a conformance identifier for this carriage in its HCP `conformsTo`, and advertises the task through `trust-task-discovery`, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)). HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1); this specification takes it that a carriage other than that binding may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and carries this one as a Trust Task on that reading, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

## Authorization

The authority is **the Context Subject's own authority over their directives**, which a Management Client exercises for them (HCP 6.4, 6.1.9). It is evidenced by the Management Client's access token in `accessToken`, which the provider validates exactly as over HCP's HTTP binding. Removing a directive is a governance operation: any caller other than a Management Client acting for the Context Subject, or for a delegate, receives not authorized (HCP 6.1.9, 9.2, 9.4.15), carried as `permissionDenied`. A grant held by a Context Client never authorizes it, whatever its scopes (HCP 6.1.9). HCP names only its HTTP Binding as the carriage for a governance operation (HCP 5.4, 9.1); this specification accepts the operation as a Trust Task from a Management Client acting for the Authorizing Principal, and from no other caller (HCP 6.1.9), *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

Under the Delegated Authorization Profile, a delegate removes directives as for standing rules (HCP 6.4, 9.4.15): the Context Subject can remove any directive, and a delegate can remove only the directives they set (HCP 6.3, 9.4.14); the provider checks the directive's `setBy` against the caller. A delegate's attempt to remove any other directive is refused with `permissionDenied`.

The REQUIRED `proof` establishes which Management Client VID sent the document and that it is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). It is not the authorization: the token is. A provider that knows which VID it registered the Management Client under **SHOULD** check that the `issuer` is that client.

## Definitions

- **Management Client**, **Context Client**, **HCP Provider**, **Context Subject**, **Authorizing Principal**, **delegate** and **directive** have their HCP 2.0-draft-3 meanings.
- HCP defines no request or result schema for this operation. The members come from HCP 9.2 (the path) and 9.4.15.

### HCP field mapping

| HCP field | Member | Notes |
|---|---|---|
| path parameter `{directiveId}` (`DELETE /v2/directives/{directiveId}`, HCP 9.2) | `directiveId` | Moved from the path into the payload, unchanged. |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result `204 No Content` | the empty `#response` payload | No body in HCP, so none here. |

### Parties

| HCP role | Trust Task member |
|---|---|
| Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../../entries/query/0.1/spec.md#parties), a provider **MUST** make its VID verifiably bound to its HCP identity (its discovery document, HCP 19.1), and a Management Client **MUST** check that binding before it sends a token to a VID.

## Request

The Management Client sends the removal to the provider. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json): `directiveId`, the directive to remove, and `accessToken`, the Management Client's token. Both are REQUIRED.

### Removing a directive that asked before every signature

```json
{
  "id": "urn:uuid:f2a4c6e8-0b1d-4c3f-9e5a-7b9d1f3a5c84",
  "type": "https://trusttasks.org/spec/hcp/directives/remove/0.1#request",
  "issuer": "did:web:wallet.person.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T14:00:00Z",
  "threadId": "urn:uuid:3c5e7a9b-1d2f-4e6a-8c0b-5d7f9b1d3e66",
  "payload": {
    "directiveId": "urn:uuid:e6a8c0d2-4f3b-4e7a-9c1d-3b5f7a9c1e28",
    "accessToken": "hcp_mt_8f3a1c9e7b2d4f60"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T14:00:00Z",
    "verificationMethod": "did:web:wallet.person.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z7Wq4nT8rLcB2xV9mH3pK1sY5dF6gJ0aNe2uR3oZvC8iX4kD7tM1yQ9wE6bS5hA2jU8lG3fO7pL0vZ4zK1cN6mY5"
  }
}
```

## Response

The provider answers when the directive is removed. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): the empty object, which carries HCP's `204 No Content`. The response is declared, although it carries nothing, because the removal is consequential and its completion is relied upon ([SPEC §4.4.2](/SPEC.md#442-acknowledging-a-fire-and-forget-task)). Failures use `trust-task-error`, not a `#response` document (see [Errors](#errors)).

### The directive is removed

```json
{
  "id": "urn:uuid:6d8f0a2c-4e1b-4d3a-a7c9-0f2e4a6c8b91",
  "type": "https://trusttasks.org/spec/hcp/directives/remove/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:wallet.person.example",
  "issuedAt": "2026-11-25T14:00:01Z",
  "threadId": "urn:uuid:3c5e7a9b-1d2f-4e6a-8c0b-5d7f9b1d3e66",
  "payload": {}
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows.

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request` | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: the caller is not a Management Client acting for the Context Subject or a delegate (HCP 6.1.9), a delegate names a directive they did not set (HCP 6.4, 9.4.14, 9.4.15), or the `directiveId` is unknown or already removed (see [Repeated requests](#repeated-requests)) | `permissionDenied`, with one fixed `message` whatever the reason (HCP 5.5, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| conflict | `idempotency-conflict` | Does not arise: HCP defines it for a retried request with an idempotency key (HCP 9.6), and this operation carries none. A re-delivered document is handled under [Repeated requests](#repeated-requests). |
| version conflict | `version-conflict` | Does not arise: a directive has no version (HCP 6.4). |
| credential invalid | `credential-invalid` | Does not arise: it concerns a credential written as an entry's content (HCP 14.6). |
| subject unavailable | `subject-unavailable` | `unavailable` |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion` |

## Rules carried from HCP

- **Effect.** From the moment the directive is removed, the provider no longer holds it, so it no longer delivers it: every query and get-entry result carries only the directives the provider holds that apply (HCP 6.4, 5.2.1, 5.2.2), and so does a client's read of its own grant (HCP 5.2.6).
- **Clients that already received it.** An HCP Client that claims to follow directives **MUST** follow every directive delivered to it, the most recent it has received, for as long as it holds context disclosed under the grant (HCP 6.4). HCP says nothing more about how a removal reaches a client that already received the directive, and this specification adds nothing.
- **Audit.** The provider appends `directive.removed`, recording `directiveId`, `action`, `effect`, `categories`, `operators` and `setBy` (HCP 8.1, 8.2), durable before this task's response is sent (HCP 8.4).

## Repeated requests

HCP 2.0-draft-3 does not say what removing an unknown or already-removed directive yields (HCP 9.4.15). This specification reads HCP as follows, *pending HCP confirmation* ([question 27](#pending-hcp-confirmation)): a `directiveId` the provider does not hold, including one already removed, is refused with `permissionDenied`, with the same fixed `message` as every other not-authorized refusal, so that a caller cannot tell an unknown directive from one it may not remove.

A re-delivered document with the same `id` and content is absorbed by [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements): it is not executed twice, and is answered with the original empty response.

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 1 — conformance claims.** HCP defines no conformance identifier for Trust Task carriage, so a provider **MUST NOT** claim this carriage in its HCP `conformsTo`; it advertises the task with [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead. The binding identifier for the carriage is the same question as [question 37](#pending-hcp-confirmation): whether a carriage other than the HTTP Binding may carry governance operations at all.
- **Question 27 — unknown and already-removed directives.** Refused with `permissionDenied`, with the same text as every other not-authorized refusal (see [Repeated requests](#repeated-requests)).
- **Question 37 — governance over a carriage other than the HTTP Binding.** HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1). This specification takes it that another carriage, here a Trust Task, may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and applies every rule HCP places on the operation unchanged (see [Conformance](#conformance)).

## What HCP requires that a document cannot express

- The provider-held directives, and their delivery with every result they apply to (HCP 6.4).
- What a client does with a directive it already received: the provider cannot observe it (HCP 6.4).
- The `directive.removed` audit event and its durability (HCP 8.1, 8.4).
- Carriage by the HTTP Binding: HCP carries governance operations only over that binding (HCP 1.6, 5.4, 9.1), and a Trust Task is not it. This specification carries the operation for a Management Client acting for the Authorizing Principal (HCP 6.1.9) instead, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

## Security & Privacy

### Data carried

The request carries a directive identifier and the Management Client's access token, a bearer secret that **MUST** travel only to the provider. It carries no context. The response carries nothing. A producer **MUST NOT** put the token or any context in `ext`.

### Correlation

The provider learns which Management Client removed which directive and when; that is the record the person needs. The `directiveId` is delivered with the directive to every client it applied to (HCP 6.4), so those clients know it, but none of them learns of its removal from this task; they see only that later results no longer carry it. `threadId` **SHOULD** be fresh per removal.

### Retention

The provider keeps the `directive.removed` audit event (HCP 8.1), readable by the Context Subject (HCP 8.3). The signed request is evidence of who removed the directive and when, and a provider **SHOULD** keep it with that event.

### Consent/purpose

Removing a directive lifts a limit the person set on what clients may do with their context, or on their behalf. A Management Client **SHOULD** show the person the directive's `text` and effect before sending the removal; whether it asks for confirmation is its own policy ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).
