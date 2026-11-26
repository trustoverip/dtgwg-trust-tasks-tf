---
slug: hcp/rules/list
version: "0.1"
title: "HCP — List Standing Rules"
summary: A Management Client, acting for the person or a delegate, reads the Human Context Protocol standing rules a provider holds; carries the HCP 2.0-draft-3 List standing rules operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: List standing rules
category: access-control
keywords: [hcp, human-context-protocol, standing-rule, governance]
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
    A read of metadata whose authority is the Management Client's access token, which the provider checks whatever
    the binding, and which changes nothing. The request proof is REQUIRED, as on every hcp/* request that carries a
    token, because it is what lets the provider refuse a token sent from a VID other than the client's (a carriage rule pending HCP confirmation, question 53). The response proof is RECOMMENDED so the Management Client
    can show which rules the provider said it held, and when.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task is not consequential, but its answer describes the rules in force now, which the person relies on to
    decide what to change; requiring `issuedAt` lets the provider refuse a stale or replayed request by its
    acceptance window.
sideEffects:
  level: none
  rationale: >-
    A read. The provider changes nothing, and HCP requires no audit event for it (HCP 8.1).
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: secret
  rationale: >-
    The response lists the rules, which name operators, client claims and categories but carry no context. A read
    produces no attributable effect in the person's name, so `actsAsSubject` is false (SPEC 7.3 item 14), as for every
    Management Client read. The request carries the Management Client's access token, a bearer secret.
retention:
  class: transient
  rationale: >-
    HCP requires no audit event for reading the standing rules (HCP 8.1 lists none), and the provider keeps nothing
    because of this task. The rules themselves are kept because they were set, not because they were read.
maxDocumentBytes:
  response: 4194304
  rationale: >-
    The list is not paginated (HCP 9.4.14), and HCP bounds neither the number of rules nor their size. A typical rule
    is a few hundred bytes, so 4 MiB holds thousands of rules with envelope and proof. This is a framework bound, not
    an HCP requirement.
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence", here the Management
      Client's access token.
    retryable: false
related:
  - hcp/rules/set
  - hcp/rules/remove
  - hcp/directives/list
---

## Abstract

The **HCP — List Standing Rules** Trust Task carries the List standing rules operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 6.3 and 5.4; `GET /v2/rules` in HCP 9.2 and 9.4.14). A *Management Client*, acting for the *Context Subject*, or for a delegate under the Delegated Authorization Profile, asks an *HCP Provider* for the standing rules it holds, and the provider answers with them. A rule is set with [`hcp/rules/set`](../../set/0.1/spec.md) and removed with [`hcp/rules/remove`](../../remove/0.1/spec.md).

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on standing rules; the ones that bear on the document are restated below, unchanged. Conformance to HCP is HCP's ([SPEC §9.4](/SPEC.md#94-carried-protocols)): a provider **MUST NOT** list a conformance identifier for this carriage in its HCP `conformsTo`, and advertises the task through `trust-task-discovery`, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)). HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1); this specification takes it that a carriage other than that binding may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and carries this one as a Trust Task on that reading, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

This specification adds one check HCP does not make: the token binding of [The access token](#the-access-token), a carriage rule of the `hcp/*` family, not HCP's. [SPEC §9.4](/SPEC.md#94-carried-protocols) item 6 places a check the carried protocol does not make in a separate specification that does not declare `carries`. This family keeps this one in the carriage, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)), because it protects the token that the carriage itself moves from the HTTP `Authorization` header into the document, where it can be stored or forwarded with the document; a separate task could not protect the document this one carries.

## Authorization

The authority is **the Context Subject's own authority over their standing rules**, which a Management Client exercises for them (HCP 6.3, 6.1.9). It is evidenced by the Management Client's access token in `accessToken`, which the provider validates exactly as over HCP's HTTP binding. Listing standing rules is a governance operation: any caller other than a Management Client acting for the Context Subject, or for a delegate, receives not authorized (HCP 6.1.9, 9.2), carried as `permissionDenied`. A grant held by a Context Client never authorizes it, whatever its scopes (HCP 6.1.9). HCP names only its HTTP Binding as the carriage for a governance operation (HCP 5.4, 9.1); this specification accepts the operation as a Trust Task from a Management Client acting for the Authorizing Principal, and from no other caller (HCP 6.1.9), *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

Under the Delegated Authorization Profile, HCP lets a delegate call this operation "within the delegation's scope" (HCP 9.4.14), and a delegate's governance operations reach only categories within scope (HCP 17.4), but HCP does not say which rules a delegate sees. This specification takes the narrowest reading, *pending HCP confirmation* ([question 28](#pending-hcp-confirmation)): a delegate's list carries each rule whose `categories` are absent, or are all within the delegation's scope; a rule that names any category outside the scope is left out whole, never trimmed, so the list never discloses an out-of-scope category. The Context Subject's list carries every rule.

The `proof` establishes which Management Client VID sent the document and that it is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). It is not the authorization: the token is. The request always carries a proof, *pending HCP confirmation* ([question 53](#pending-hcp-confirmation)), so a token issued to a client other than the one the `issuer` VID identifies is always refused with `permissionDenied` (see [The access token](#the-access-token)).

## Definitions

- **Management Client**, **Context Client**, **HCP Provider**, **Context Subject**, **Authorizing Principal**, **delegate** and **standing rule** have their HCP 2.0-draft-3 meanings.
- HCP defines no request or result schema for this operation. The request has no member but the token (HCP 9.4.14); each rule in the result is the standing rule of HCP 6.3, the shared [`StandingRule`](../../../_shared/0.1/standing-rule.schema.json).

### HCP field mapping

| HCP field | Member | Notes |
|---|---|---|
| `GET /v2/rules` (HCP 9.2, 9.4.14) | the Type URI | No query parameters in HCP, so no request member beyond the token. |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result `200`: a `rules` array (HCP 9.4.14) | `rules` in the `#response` payload | Unchanged. Not paginated: HCP gives this operation no cursor ([question 3](#pending-hcp-confirmation)). |

### Parties

| HCP role | Trust Task member |
|---|---|
| Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../../entries/query/0.1/spec.md#parties), a provider **MUST** make its VID verifiably bound to its HCP identity (its discovery document, HCP 19.1), and a Management Client **MUST** check that binding before it sends a token to a VID.

## Request

The Management Client asks for the rules. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json): `accessToken`, the Management Client's token, REQUIRED and the only member.

### Reading the person's standing rules

```json
{
  "id": "urn:uuid:e4b8d2a6-7c1f-4e3a-9d5b-6a0c2f4e8b17",
  "type": "https://trusttasks.org/spec/hcp/rules/list/0.1#request",
  "issuer": "did:web:wallet.person.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T11:00:00Z",
  "threadId": "urn:uuid:1f5c9a3e-2b8d-4c6f-a0e7-9d3b5f7a1c29",
  "payload": {
    "accessToken": "hcp_mt_8f3a1c9e7b2d4f60"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T11:00:00Z",
    "verificationMethod": "did:web:wallet.person.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z2Lm7pQ4rXcN9vB3hT8kW1sY6dF5gJ0aRe2uZ3oKnC8iM4tD7wH1yS9xE6bV5qA2jU8lG3fO7pN0vX4zK1cT6mY9"
  }
}
```

## Response

The provider answers with the rules. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): `rules`, an array of standing rules, empty when there is none. HCP sets no order. Failures use `trust-task-error`, not a `#response` document (see [Errors](#errors)).

### Two rules, one set by a delegate

```json
{
  "id": "urn:uuid:7a9c1e3f-5b2d-4f8a-8c6e-0d4b2a9f7e53",
  "type": "https://trusttasks.org/spec/hcp/rules/list/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:wallet.person.example",
  "issuedAt": "2026-11-25T11:00:01Z",
  "threadId": "urn:uuid:1f5c9a3e-2b8d-4c6f-a0e7-9d3b5f7a1c29",
  "payload": {
    "rules": [
      {
        "ruleId": "urn:uuid:3e9b1c7d-5a2f-4d8e-9b6c-0f1a2e3d4c5b",
        "effect": "deny",
        "operators": ["ads.example"],
        "categories": ["health_context"],
        "setBy": "context-subject",
        "createdAt": "2026-11-25T10:00:01Z"
      },
      {
        "ruleId": "urn:uuid:b2d4f6a8-0c1e-4a3b-9d5f-8e7c6b5a4d39",
        "effect": "deny",
        "requires": ["https://hcp.example/spec/clients/directives/2.0"],
        "categories": ["education_learner_profile"],
        "setBy": "urn:uuid:6c8e0a2b-4d1f-4b3c-8e5a-7f9d1b3c5e70",
        "createdAt": "2026-11-20T16:30:00Z"
      }
    ]
  }
}
```

The second rule, set by a delegate, denies every client that does not claim to follow directives (HCP 6.3, 18.1) any of the person's `education_learner_profile`.

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows.

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request` | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: the caller is not a Management Client acting for the Context Subject or a delegate (HCP 6.1.9, 9.4.14) | `permissionDenied`, with one fixed `message` whatever the reason (HCP 5.5, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| conflict | `idempotency-conflict` | Does not arise: HCP defines it for a retried request with an idempotency key (HCP 9.6), and a read carries none. |
| version conflict | `version-conflict` | Does not arise: a standing rule has no version (HCP 6.3). |
| credential invalid | `credential-invalid` | Does not arise: it concerns a credential written as an entry's content (HCP 14.6). |
| subject unavailable | `subject-unavailable` | `unavailable` |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion` |

## Rules carried from HCP

- **What a rule is.** Each listed rule is a deny rule the Context Subject (or a delegate) set across every grant, present and future, with the `ruleId`, `effect`, `setBy` and `createdAt` the provider set (HCP 6.3).
- **No audit event.** HCP 8.1 lists no event for reading the standing rules, and this task adds none.
- **Not paginated.** HCP 9.4.14 answers with the whole `rules` array; this task does the same ([question 3](#pending-hcp-confirmation)).
- **Removed rules.** A removed rule is no longer held, so it is not listed; its `rule.removed` audit event stays readable through the audit log (HCP 8.1, 8.3).

## Repeated requests

Reading has no effect, so a re-delivered document changes nothing. A consumer still applies [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements) to a re-delivered document with the same `id` and content, and answers it with the original response; a Management Client that wants the current rules sends a new document.

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 1 — conformance claims.** HCP defines no conformance identifier for Trust Task carriage, so a provider **MUST NOT** claim this carriage in its HCP `conformsTo`; it advertises the task with [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead. The binding identifier for the carriage is the same question as [question 37](#pending-hcp-confirmation): whether a carriage other than the HTTP Binding may carry governance operations at all.
- **Question 3 — pagination.** The rules list is not paginated, as in HCP 9.4.14.
- **Question 22 — the client's identity.** The `issuer` VID, authenticated by the request's `proof`, identifies the HCP client the token was issued to; a token issued to any other client **MUST** be refused with `permissionDenied`. This token binding is a carriage rule of this family, a check HCP does not itself make (see [The access token](#the-access-token) and [Conformance](#conformance)).
- **Question 28 — what a delegate's list carries.** Each rule whose `categories` are absent or all within the delegation's scope; a rule naming any other category is left out whole (see [Authorization](#authorization)).
- **Question 37 — governance over a carriage other than the HTTP Binding.** HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1). This specification takes it that another carriage, here a Trust Task, may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and applies every rule HCP places on the operation unchanged (see [Conformance](#conformance)).
- **Question 53 — a request proof on every request that carries a token.** A request `proof` is **REQUIRED** on every `hcp/*` request that carries `accessToken`, so that the token binding of question 22 always applies and a token lifted from a stored or forwarded document cannot be replayed in an unsigned request. This is a carriage rule of this family, not HCP's, and is stricter than HCP's bearer model, under which presenting a valid token is enough (HCP 9.3); see [The access token](#the-access-token).

## What HCP requires that a document cannot express

- The provider-held standing rules, and their enforcement on every request and every grant (HCP 6.3).
- Which rules a delegate may see, which turns on the delegation the provider holds (HCP 17.4).
- Carriage by the HTTP Binding: HCP carries governance operations only over that binding (HCP 1.6, 5.4, 9.1), and a Trust Task is not it. This specification carries the operation for a Management Client acting for the Authorizing Principal (HCP 6.1.9) instead, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

## Security & Privacy

### Data carried

The request carries only the Management Client's access token, a bearer secret that **MUST** travel only to the provider. The response carries the standing rules, which carry no context but can say something about the person: which categories they hold and want kept from whom, and which organizations they distrust. A producer **MUST NOT** put the token or any context in `ext`.

### The access token

`accessToken` is a bearer secret: whoever presents it exercises the authority it evidences. It **MUST** travel only to the provider. A recipient **MUST NOT** log `accessToken` or retain it beyond the exchange.

**Token binding, a carriage rule of this family.** Every request that carries `accessToken` carries a `proof`, *pending HCP confirmation* ([question 53](#pending-hcp-confirmation)), and the provider **MUST** refuse with `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies, so that a token lifted from a stored or forwarded document is of no use from another VID. HCP validates the token alone and does not bind it to a VID (HCP 9.3): this rule is the `hcp/*` family's, not HCP's, and rests on the `issuer` VID identifying the HCP client, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)). It is a check HCP does not make (see [Conformance](#conformance)).

**Retained copies.** The provider verifies the request's `proof` over the document as received, at receipt, before it removes anything ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). A provider that retains the request document, as a duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) or as evidence of the request, **MUST** remove `accessToken` from the copy it keeps and record in its place `accessTokenDigest`: the SHA-256 multihash of the token's UTF-8 bytes, multibase-encoded as base64url without padding (prefix `u`), the framework's `DigestMultibase` form ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)). The kept copy's `proof` no longer verifies over it, so the provider records with the copy that the proof was verified at receipt: when, under which verification method, and with what result. It relies on that record, and **MUST NOT** verify the proof again over the kept copy. A record that keeps only a digest of the document, such as its task digest ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)), keeps no token either.

### Correlation

The provider learns that a Management Client read the rules, and when; HCP does not require it to record that. A `setBy` that is a `delegationId` tells the reader which delegate set a rule; the reader is the Context Subject or a delegate of the same person, so this discloses nothing outside the person's own governance. `threadId` **SHOULD** be fresh per request.

### Retention

The provider keeps nothing because of this task. A Management Client **SHOULD NOT** keep the list longer than it needs to show it to the person, because a kept copy goes stale as soon as a rule is set or removed.

### Consent/purpose

The rules are the person's own settings, read for them so they can see and change them. Only the person's own Management Clients, or their delegates', receive the list; a Context Client never does (HCP 6.1.9).
