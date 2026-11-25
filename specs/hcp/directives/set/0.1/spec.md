---
slug: hcp/directives/set
version: "0.1"
title: "HCP — Set Directive"
summary: A Management Client, acting for the person or a delegate, sets a Human Context Protocol directive telling clients what they may do with context they receive, or on the person's behalf; carries the HCP 2.0-draft-3 Set directive operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Set directive
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
    Setting a directive exercises the person's own authority and changes what every client it applies to is told it
    may do, so the request is signed: the provider can show which Management Client set the directive, in which
    words, and when, independently of the transport. The response proof is RECOMMENDED so the Management Client can
    show the provider acknowledged the directive as it holds it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task is mutating and acts for the person, so it is consequential; a re-delivered request must be placeable in
    the provider's duplicate-execution window, so that it is absorbed instead of setting a second directive.
sideEffects:
  level: mutating
  rationale: >-
    The provider stores a new directive and delivers it with every result it applies to (HCP 6.4). A directive only
    narrows, and is undone by removing it with hcp/directives/remove.
exposure:
  discloses: metadata
  actsAsSubject: true
  ingests: secret
  rationale: >-
    The Management Client exercises the Context Subject's (or a delegate's) authority over their own directives. The
    request carries the Management Client's access token, a bearer secret, and the directive's text in the person's
    own words. The response returns the directive as the provider holds it, which carries no context.
retention:
  class: durable
  rationale: >-
    The provider keeps the directive until it is removed, and appends a `directive.set` audit event (HCP 6.4, 8.1),
    which is the person's record of the change.
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence", here the Management
      Client's access token.
    retryable: false
related:
  - hcp/directives/list
  - hcp/directives/remove
  - hcp/rules/set
  - hcp/entries/query
---

## Abstract

The **HCP — Set Directive** Trust Task carries the Set directive operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 6.4 and 5.4; `POST /v2/directives` in HCP 9.2 and 9.4.15). A *Management Client*, acting for the *Context Subject*, or for a delegate under the Delegated Authorization Profile, tells an *HCP Provider* to set a directive: what a client may do with context it has received, or on the person's behalf, such as "never pass my health context to another agent" or "ask me before you sign anything". The provider stores it, delivers it with every disclosure it applies to, and records a `directive.set` audit event. A directive is read with [`hcp/directives/list`](../../list/0.1/spec.md) and removed with [`hcp/directives/remove`](../../remove/0.1/spec.md).

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on directives; the ones that bear on the document are restated below, unchanged. Conformance to HCP is HCP's ([SPEC §9.4](/SPEC.md#94-carried-protocols)): a provider **MUST NOT** list a conformance identifier for this carriage in its HCP `conformsTo`, and advertises the task through `trust-task-discovery`, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)). HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1); this specification takes it that a carriage other than that binding may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and carries this one as a Trust Task on that reading, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

## Authorization

The authority is **the Context Subject's own authority over their directives**, which a Management Client exercises for them (HCP 6.4, 6.1.9). It is evidenced by the Management Client's access token in `accessToken`, which the provider validates exactly as over HCP's HTTP binding. Setting a directive is a governance operation: any caller other than a Management Client acting for the Context Subject, or for a delegate, receives not authorized (HCP 6.1.9, 9.2, 9.4.15), carried as `permissionDenied`. A grant held by a Context Client never authorizes it, whatever its scopes (HCP 6.1.9). HCP names only its HTTP Binding as the carriage for a governance operation (HCP 5.4, 9.1); this specification accepts the operation as a Trust Task from a Management Client acting for the Authorizing Principal, and from no other caller (HCP 6.1.9), *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

Under the Delegated Authorization Profile, a delegate **MAY** set directives within the categories of the delegation's scope (HCP 6.4, 17.4). A delegation's scope names each category and has no wildcard (HCP 17.4), and a directive without `categories` applies to every category (HCP 6.4), so a delegate's directive names `categories`, each within scope. A delegate's request for any other directive is refused with `permissionDenied`. The provider records who set the directive in `setBy`: `context-subject`, or the delegate's `delegationId` (HCP 6.4, 6.3).

The REQUIRED `proof` establishes which Management Client VID sent the document and that it is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). It is not the authorization: the token is. A provider that knows which VID it registered the Management Client under **SHOULD** check that the `issuer` is that client.

## Definitions

- **Management Client**, **Context Client**, **HCP Provider**, **Context Subject**, **Authorizing Principal**, **Client Operator**, **delegate**, **standing rule** and **directive** have their HCP 2.0-draft-3 meanings.
- HCP defines no request or result schema for this operation. The request members come from HCP 9.4.15; the result is the directive of HCP 6.4, the shared [`Directive`](../../../_shared/0.1/directive.schema.json).

### HCP field mapping

| HCP field | Member | Notes |
|---|---|---|
| request body `action`, `effect`, `text`, `categories`, `operators` (`POST /v2/directives`, HCP 9.4.15) | `action`, `effect`, `text`, `categories`, `operators` | Unchanged. The constraints are the directive's own (HCP 6.4); the array and `text` bounds are framework bounds. |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result `201`: the directive (HCP 9.4.15) | `directive` in the `#response` payload | HCP's body is the directive itself; this task wraps it in one member, `directive`, so the response payload stays a closed object. The directive's members are unchanged. |
| `directiveId`, `setBy`, `createdAt` | set by the provider, in `directive` | Never taken from the request (HCP 6.4, 9.4.15); the request schema has no such member. |

### Parties

| HCP role | Trust Task member |
|---|---|
| Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../../entries/query/0.1/spec.md#parties), a provider **MUST** make its VID verifiably bound to its HCP identity (its discovery document, HCP 19.1), and a Management Client **MUST** check that binding before it sends a token to a VID.

## Request

The Management Client sends the directive to the provider. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json): `action`, `effect`, `text` and `accessToken`, REQUIRED, and `categories` and `operators`, OPTIONAL.

### Never pass health context to another agent

```json
{
  "id": "urn:uuid:0b2d4f6a-8c1e-4a3b-9d5f-7e6c5b4a3d21",
  "type": "https://trusttasks.org/spec/hcp/directives/set/0.1#request",
  "issuer": "did:web:wallet.person.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T12:00:00Z",
  "threadId": "urn:uuid:9e1a3c5b-7d2f-4b6e-8a0c-4f2d6b8e0a35",
  "payload": {
    "action": "share",
    "effect": "deny",
    "text": "Never pass my health context to another agent.",
    "categories": ["health_context"],
    "accessToken": "hcp_mt_8f3a1c9e7b2d4f60"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T12:00:00Z",
    "verificationMethod": "did:web:wallet.person.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z4Np9sR3tYcM8wB2hV7kQ1xL6dF5gJ0aWe2uT3oZnC8iK4pD7mH1yS9vE6bX5rA2jU8lG3fO7qN0tZ4zK1cW6mY3"
  }
}
```

## Response

The provider answers when the directive is set. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): `directive`, the directive as the provider holds it, with the `directiveId`, `setBy` and `createdAt` it set. Failures use `trust-task-error`, not a `#response` document (see [Errors](#errors)).

### The directive is set

```json
{
  "id": "urn:uuid:4c6e8a0b-2d1f-4e3a-9b5c-1a7f3d9e5b47",
  "type": "https://trusttasks.org/spec/hcp/directives/set/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:wallet.person.example",
  "issuedAt": "2026-11-25T12:00:01Z",
  "threadId": "urn:uuid:9e1a3c5b-7d2f-4b6e-8a0c-4f2d6b8e0a35",
  "payload": {
    "directive": {
      "directiveId": "urn:uuid:d5f7b9c1-3e2a-4d6f-8b0e-2c4a6e8f0b19",
      "action": "share",
      "effect": "deny",
      "text": "Never pass my health context to another agent.",
      "categories": ["health_context"],
      "setBy": "context-subject",
      "createdAt": "2026-11-25T12:00:01Z"
    }
  }
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows.

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request` | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: the caller is not a Management Client acting for the Context Subject or a delegate (HCP 6.1.9), or a delegate's directive reaches a category outside the delegation's scope (HCP 6.4, 17.4) | `permissionDenied`, with one fixed `message` whatever the reason (HCP 5.5, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| conflict | `idempotency-conflict` | Does not arise: HCP defines it for a retried request with an idempotency key (HCP 9.6), and this operation carries none. A re-delivered document is handled under [Repeated requests](#repeated-requests). |
| version conflict | `version-conflict` | Does not arise: a directive has no version (HCP 6.4). |
| credential invalid | `credential-invalid` | Does not arise: it concerns a credential written as an entry's content (HCP 14.6). |
| subject unavailable | `subject-unavailable` | `unavailable` |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion` |

## Rules carried from HCP

- **Directives only narrow.** A directive never widens access to context: it does not authorize a disclosure the grant does not, or onward disclosure beyond HCP 3.12. `permit` applies only to actions HCP does not otherwise govern, such as signing. A `share` directive whose effect is `permit` relaxes nothing in HCP 3.12; one whose effect is `deny` or `ask-first` applies to onward sharing (HCP 6.4, 6.1.4).
- **Precedence.** Where two directives that apply disagree, `deny` prevails over `ask-first`, and `ask-first` over `permit` (HCP 6.4).
- **Delivery.** Every query and get-entry result to a Context Client carries each directive whose `operators` match the client's Client Operator, and whose `categories` include a category of an entry disclosed or are absent ([`hcp/entries/query`](../../../entries/query/0.1/spec.md); HCP 6.4, 5.2.1, 5.2.2). A client reading its own grant receives the directives that apply to it (HCP 5.2.6). Directives are delivered whether or not the client claims to follow them, and a directive set after a disclosure reaches the client with its next result (HCP 6.4).
- **Telling the person.** Because every client a directive applies to receives its `text`, a Management Client **SHOULD** tell the Context Subject, when they write one, that those clients will see it, and **SHOULD** scope a directive about sensitive context to its categories (HCP 6.4).
- **An unrecognized action.** A client that does not recognize `action` reads the directive from `text`, and **MUST NOT** treat it as a permission (HCP 6.4).
- **Audit.** The provider appends `directive.set`, recording `directiveId`, `action`, `effect`, `categories`, `operators` and `setBy` (HCP 8.1, 8.2), durable before this task's response is sent (HCP 8.4).
- **Export and import.** Directives are part of the Context Subject's export, and an import brings them with it (HCP 6.4, 5.4.1, 5.4.3).

## Repeated requests

HCP gives setting a directive no idempotency key (HCP 9.6), and two identical requests set two directives. A re-delivered document with the same `id` and content is absorbed by [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements): it is not executed twice, sets no second directive and causes no second `directive.set` event, and is answered with the original response. A Management Client that wants a second, identical directive sends a new document.

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 1 — conformance claims.** HCP defines no conformance identifier for Trust Task carriage, so a provider **MUST NOT** claim this carriage in its HCP `conformsTo`; it advertises the task with [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead. The binding identifier for the carriage is the same question as [question 37](#pending-hcp-confirmation): whether a carriage other than the HTTP Binding may carry governance operations at all.
- **Question 37 — governance over a carriage other than the HTTP Binding.** HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1). This specification takes it that another carriage, here a Trust Task, may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and applies every rule HCP places on the operation unchanged (see [Conformance](#conformance)).

## What HCP requires that a document cannot express

- The provider-held directives, and their delivery with every result they apply to (HCP 6.4).
- What a client does with a directive: the provider cannot observe it (HCP 6.4).
- The `directive.set` audit event and its durability (HCP 8.1, 8.4).
- Carriage by the HTTP Binding: HCP carries governance operations only over that binding (HCP 1.6, 5.4, 9.1), and a Trust Task is not it. This specification carries the operation for a Management Client acting for the Authorizing Principal (HCP 6.1.9) instead, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

## Security & Privacy

### Data carried

The request carries the directive, including its `text` in the person's own words, and the Management Client's access token, a bearer secret that **MUST** travel only to the provider. It carries no context. The response carries the directive as the provider holds it. A producer **MUST NOT** put the token or any context in `ext`.

### Correlation

The provider learns which Management Client set which directive and when; that is the record the person needs. A directive's `text` and categories can themselves say something about the person, and unlike a standing rule, a directive is delivered to every client it applies to (HCP 6.4): every such client learns its `text`. A directive scoped to its categories reaches only the clients that receive context in them. `threadId` **SHOULD** be fresh per request.

### Retention

The provider keeps the directive until it is removed, and keeps the `directive.set` audit event (HCP 8.1), readable by the Context Subject (HCP 8.3). The signed request is evidence of who set the directive, in which words, and when, and a provider **SHOULD** keep it with that event. A client that received the directive keeps following the most recent it has received for as long as it holds context disclosed under the grant (HCP 6.4).

### Consent/purpose

A directive narrows what clients may do with the person's context; it never authorizes a disclosure. A Management Client **SHOULD** show the person the directive's `text`, its effect and whom it reaches, and tell them those clients will see the text (HCP 6.4), before sending it; whether it asks for confirmation is its own policy ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).
