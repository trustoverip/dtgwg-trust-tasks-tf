---
slug: hcp/rules/set
version: "0.1"
title: "HCP — Set Standing Rule"
summary: A Management Client, acting for the person or a delegate, sets a Human Context Protocol standing rule that denies context to named Client Operators across every grant; carries the HCP 2.0-draft-3 Set standing rule operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Set standing rule
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
    Setting a standing rule exercises the person's own authority and changes which clients receive their context on
    every grant, so the request is signed: the provider can show which Management Client set the rule, and when,
    independently of the transport. The response proof is RECOMMENDED so the Management Client can show the provider
    acknowledged the rule as it holds it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task is mutating and acts for the person, so it is consequential; a re-delivered request must be placeable in
    the provider's duplicate-execution window, so that it is absorbed instead of setting a second rule.
sideEffects:
  level: mutating
  rationale: >-
    The provider stores a new standing rule and enforces it on every grant, present and future (HCP 6.3). It only
    narrows access, and is undone by removing it with hcp/rules/remove; a grant the provider revokes because the rule
    covers every category it reaches stays revoked (HCP 6.3, 6.2.2).
exposure:
  discloses: metadata
  actsAsSubject: true
  ingests: secret
  rationale: >-
    The Management Client exercises the Context Subject's (or a delegate's) authority over their own standing rules.
    The request carries the Management Client's access token, a bearer secret. The response returns the rule as the
    provider holds it, which names operators and categories but carries no context.
retention:
  class: durable
  rationale: >-
    The provider keeps the rule until it is removed, and appends a `rule.set` audit event (HCP 6.3, 8.1), which is the
    person's record of the change.
    The access token is not retained: a kept request holds its digest instead (see Security & Privacy, The access token).
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence", here the Management
      Client's access token.
    retryable: false
related:
  - hcp/rules/list
  - hcp/rules/remove
  - hcp/directives/set
  - hcp/grants/revoke
---

## Abstract

The **HCP — Set Standing Rule** Trust Task carries the Set standing rule operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 6.3 and 5.4; `POST /v2/rules` in HCP 9.2 and 9.4.14). A *Management Client*, acting for the *Context Subject*, or for a delegate under the Delegated Authorization Profile, tells an *HCP Provider* to set a standing rule: "never give my context to anyone in this namespace", for everything or only for some categories. The provider stores the rule, enforces it on every grant from that moment, and records a `rule.set` audit event. A rule is read with [`hcp/rules/list`](../../list/0.1/spec.md) and removed with [`hcp/rules/remove`](../../remove/0.1/spec.md).

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on standing rules; the ones that bear on the document are restated below, unchanged. Conformance to HCP is HCP's ([SPEC §9.4](/SPEC.md#94-carried-protocols)): a provider **MUST NOT** list a conformance identifier for this carriage in its HCP `conformsTo`, and advertises the task through `trust-task-discovery`, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)). HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1); this specification takes it that a carriage other than that binding may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and carries this one as a Trust Task on that reading, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

This specification adds one check HCP does not make: the token binding of [The access token](#the-access-token), a carriage rule of the `hcp/*` family, not HCP's. [SPEC §9.4](/SPEC.md#94-carried-protocols) item 6 places a check the carried protocol does not make in a separate specification that does not declare `carries`. This family keeps this one in the carriage, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)), because it protects the token that the carriage itself moves from the HTTP `Authorization` header into the document, where it can be stored or forwarded with the document; a separate task could not protect the document this one carries.

## Authorization

The authority is **the Context Subject's own authority over their standing rules**, which a Management Client exercises for them (HCP 6.3, 6.1.9). It is evidenced by the Management Client's access token in `accessToken`, which the provider validates exactly as over HCP's HTTP binding. Setting a standing rule is a governance operation: any caller other than a Management Client acting for the Context Subject, or for a delegate, receives not authorized (HCP 6.1.9, 9.2), carried as `permissionDenied`. A grant held by a Context Client never authorizes it, whatever its scopes (HCP 6.1.9). HCP names only its HTTP Binding as the carriage for a governance operation (HCP 5.4, 9.1); this specification accepts the operation as a Trust Task from a Management Client acting for the Authorizing Principal, and from no other caller (HCP 6.1.9), *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

Under the Delegated Authorization Profile, a delegate **MAY** set standing rules within the categories of the delegation's scope (HCP 6.3, 17.4). A delegation's scope names each category and has no wildcard (HCP 17.4), and a rule without `categories` applies to every category (HCP 6.3), so a delegate's rule names `categories`, each within scope. A delegate's request for any other rule is refused with `permissionDenied`. The provider records who set the rule in `setBy`: `context-subject`, or the delegate's `delegationId` (HCP 6.3, 17.2).

The REQUIRED `proof` establishes which Management Client VID sent the document and that it is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). It is not the authorization: the token is. A token issued to a client other than the one the `issuer` VID identifies is refused with `permissionDenied` (see [The access token](#the-access-token)).

## Definitions

- **Management Client**, **Context Client**, **HCP Provider**, **Context Subject**, **Authorizing Principal**, **Client Operator**, **delegate** and **standing rule** have their HCP 2.0-draft-3 meanings.
- HCP defines no request or result schema for this operation. The request members come from HCP 9.4.14; the result is the standing rule of HCP 6.3, the shared [`StandingRule`](../../../_shared/0.1/standing-rule.schema.json).

### HCP field mapping

| HCP field | Member | Notes |
|---|---|---|
| request body `operators`, `requires`, `categories` (`POST /v2/rules`, HCP 9.4.14) | `operators`, `requires`, `categories` | Unchanged. HCP requires `operators`, `requires`, or both; the schema checks it. |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result `201`: the rule (HCP 9.4.14) | `rule` in the `#response` payload | HCP's body is the rule itself; this task wraps it in one member, `rule`, so the response payload stays a closed object. The rule's members are unchanged. |
| `ruleId`, `effect`, `setBy`, `createdAt` | set by the provider, in `rule` | Never taken from the request (HCP 6.3, 9.4.14); the request schema has no such member. `effect` is always `deny`. |

### Parties

| HCP role | Trust Task member |
|---|---|
| Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../../entries/query/0.1/spec.md#parties), a provider **MUST** make its VID verifiably bound to its HCP identity (its discovery document, HCP 19.1), and a Management Client **MUST** check that binding before it sends a token to a VID.

## Request

The Management Client sends the rule to the provider. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json): `operators`, `requires` and `categories`, each OPTIONAL, and `accessToken`, REQUIRED. HCP 9.4.14 requires `operators`, `requires`, or both, and the schema requires at least one of them; a request with neither fails schema validation and is refused with `malformedRequest`.

### Keeping health context from an advertising network

```json
{
  "id": "urn:uuid:5d1e7a3c-9b2f-4c6d-8e0a-1f2b3c4d5e61",
  "type": "https://trusttasks.org/spec/hcp/rules/set/0.1#request",
  "issuer": "did:web:wallet.person.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T10:00:00Z",
  "threadId": "urn:uuid:8a3f6c2e-4d7b-4e1a-9c5d-2b6e8f0a1c73",
  "payload": {
    "operators": ["ads.example"],
    "categories": ["health_context"],
    "accessToken": "hcp_mt_8f3a1c9e7b2d4f60"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T10:00:00Z",
    "verificationMethod": "did:web:wallet.person.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z5Ht2nW8qLcV3xB7mK1pR4sY9dJ6fG0aTe5uN2oZvC8iQ3kD7wH1yE9rS4bX6jA2lU8gF3tO7pM0vZ4nK1cT6qY2"
  }
}
```

## Response

The provider answers when the rule is set. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): `rule`, the standing rule as the provider holds it, with the `ruleId`, `effect`, `setBy` and `createdAt` it set. Failures use `trust-task-error`, not a `#response` document (see [Errors](#errors)).

### The rule is set

```json
{
  "id": "urn:uuid:c7e2a9f4-1b3d-4a5e-8f6c-3d9b0e2a4f85",
  "type": "https://trusttasks.org/spec/hcp/rules/set/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:wallet.person.example",
  "issuedAt": "2026-11-25T10:00:01Z",
  "threadId": "urn:uuid:8a3f6c2e-4d7b-4e1a-9c5d-2b6e8f0a1c73",
  "payload": {
    "rule": {
      "ruleId": "urn:uuid:3e9b1c7d-5a2f-4d8e-9b6c-0f1a2e3d4c5b",
      "effect": "deny",
      "operators": ["ads.example"],
      "categories": ["health_context"],
      "setBy": "context-subject",
      "createdAt": "2026-11-25T10:00:01Z"
    }
  }
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows.

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request`: for example neither `operators` nor `requires` is present (HCP 9.4.14) | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: the caller is not a Management Client acting for the Context Subject or a delegate (HCP 6.1.9), or a delegate's rule reaches a category outside the delegation's scope (HCP 6.3, 17.4) | `permissionDenied`, with one fixed `message` whatever the reason (HCP 5.5, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| conflict | `idempotency-conflict` | Does not arise: HCP defines it for a retried request with an idempotency key (HCP 9.6), and this operation carries none. A re-delivered document is handled under [Repeated requests](#repeated-requests). |
| version conflict | `version-conflict` | Does not arise: a standing rule has no version (HCP 6.3). |
| credential invalid | `credential-invalid` | Does not arise: it concerns a credential written as an entry's content (HCP 14.6). |
| subject unavailable | `subject-unavailable` | `unavailable` |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion` |

## Rules carried from HCP

- **A standing rule only denies.** `effect` is `deny`, the only effect in HCP 2.0-draft-3, and a standing rule never widens access (HCP 6.3).
- **Existing grants.** The rule applies to existing grants from the moment it is set, through the status check the provider makes on every request (HCP 6.3, 6.2.2). A request it covers fails with not authorized. Where it covers every category a grant reaches, the provider revokes the grant, and HCP 6.2.2–6.2.4 apply, including the revocation notice ([`hcp/grants/revoked`](../../../grants/revoked/0.1/spec.md)).
- **New grants.** The provider **MUST NOT** issue a grant whose `access` reaches a category a standing rule denies to that grant's Client Operator; before the approving act, the Authorizing Principal is told which requested categories a rule excludes (HCP 6.3, 6.1.6). The rule can be overridden only by removing it, which is a separate act. A delegate cannot authorize a grant a standing rule denies (HCP 17.4).
- **Entries.** A standing rule reaches an entry through any of its categories: where it denies a Client Operator any category of an entry, the entry is withheld from that operator's clients, whichever category a request names (HCP 6.3, 4.5, 7.1).
- **`notTo`.** An operator that the rule denies any category of a detail carrying `onwardSharing` is listed in that detail's `notTo`, on every grant, from the moment the rule is set, and kept current as rules change (HCP 6.3, 6.1.5). A client cannot share with it under HCP 3.12.
- **Categories.** A rule without `categories` applies across every category, held now or later (HCP 6.3). HCP says that an entry naming a category the provider does not support is a malformed request (HCP 4.5); it says nothing of a standing rule's categories, and this specification adds nothing.
- **Audit.** The provider appends `rule.set`, recording `ruleId`, `operators`, `requires`, `categories` and `setBy` (HCP 8.1, 8.2), durable before this task's response is sent (HCP 8.4).
- **Export and import.** Standing rules are part of the Context Subject's export, and an import brings them with it (HCP 6.3, 5.4.1, 5.4.3).

## Repeated requests

HCP gives setting a rule no idempotency key (HCP 9.6), and two identical requests set two rules. A re-delivered document with the same `id` and content is absorbed by [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements): it is not executed twice, sets no second rule and causes no second `rule.set` event, and is answered with the original response. A Management Client that wants a second, identical rule sends a new document.

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 1 — conformance claims.** HCP defines no conformance identifier for Trust Task carriage, so a provider **MUST NOT** claim this carriage in its HCP `conformsTo`; it advertises the task with [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead. The binding identifier for the carriage is the same question as [question 37](#pending-hcp-confirmation): whether a carriage other than the HTTP Binding may carry governance operations at all.
- **Question 22 — the client's identity.** The `issuer` VID, authenticated by the request's `proof`, identifies the HCP client the token was issued to; a token issued to any other client **MUST** be refused with `permissionDenied`. This token binding is a carriage rule of this family, a check HCP does not itself make (see [The access token](#the-access-token) and [Conformance](#conformance)).
- **Question 37 — governance over a carriage other than the HTTP Binding.** HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1). This specification takes it that another carriage, here a Trust Task, may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and applies every rule HCP places on the operation unchanged (see [Conformance](#conformance)).
- **Question 53 — a request proof on every request that carries a token.** A request `proof` is **REQUIRED** on every `hcp/*` request that carries `accessToken`, so that the token binding of question 22 always applies and a token lifted from a stored or forwarded document cannot be replayed in an unsigned request. This is a carriage rule of this family, not HCP's, and is stricter than HCP's bearer model, under which presenting a valid token is enough (HCP 9.3); see [The access token](#the-access-token).

## What HCP requires that a document cannot express

- The provider-held standing rules, and their enforcement on every request and every grant (HCP 6.3).
- Revoking a grant the rule wholly covers, and the notice that follows (HCP 6.2.2–6.2.4).
- Keeping each affected grant's `notTo` current (HCP 6.1.5).
- Telling the Authorizing Principal, before a later approval, which requested categories a rule excludes (HCP 6.1.6).
- The `rule.set` audit event and its durability (HCP 8.1, 8.4).
- Carriage by the HTTP Binding: HCP carries governance operations only over that binding (HCP 1.6, 5.4, 9.1), and a Trust Task is not it. This specification carries the operation for a Management Client acting for the Authorizing Principal (HCP 6.1.9) instead, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

## Security & Privacy

### Data carried

The request carries the operators, claims and categories of the rule, and the Management Client's access token, a bearer secret that **MUST** travel only to the provider. It carries no context. The response carries the rule as the provider holds it. A producer **MUST NOT** put the token or any context in `ext`.

### The access token

`accessToken` is a bearer secret: whoever presents it exercises the authority it evidences. It **MUST** travel only to the provider. A recipient **MUST NOT** log `accessToken` or retain it beyond the exchange.

**Token binding, a carriage rule of this family.** Every request that carries `accessToken` carries a `proof`, *pending HCP confirmation* ([question 53](#pending-hcp-confirmation)), and the provider **MUST** refuse with `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies, so that a token lifted from a stored or forwarded document is of no use from another VID. HCP validates the token alone and does not bind it to a VID (HCP 9.3): this rule is the `hcp/*` family's, not HCP's, and rests on the `issuer` VID identifying the HCP client, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)). It is a check HCP does not make (see [Conformance](#conformance)).

**Retained copies.** The provider verifies the request's `proof` over the document as received, at receipt, before it removes anything ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). A provider that retains the request document, as a duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) or as evidence of the request, **MUST** remove `accessToken` from the copy it keeps and record in its place `accessTokenDigest`: the SHA-256 multihash of the token's UTF-8 bytes, multibase-encoded as base64url without padding (prefix `u`), the framework's `DigestMultibase` form ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)). The kept copy's `proof` no longer verifies over it, so the provider records with the copy that the proof was verified at receipt: when, under which verification method, and with what result. It relies on that record, and **MUST NOT** verify the proof again over the kept copy. A record that keeps only a digest of the document, such as its task digest ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)), keeps no token either.

### Correlation

The provider learns which Management Client set which rule and when; that is the record the person needs. A rule's categories can themselves say something about the person (that they hold health context and want it kept from someone), and its operators which organizations they distrust; the rule is known only to the provider and the person's Management Clients, and is never disclosed to a Context Client, although its effect, a refusal or a `notTo` entry, is visible to the clients it reaches. `threadId` **SHOULD** be fresh per request.

### Retention

The provider keeps the rule until it is removed, and keeps the `rule.set` audit event (HCP 8.1), readable by the Context Subject (HCP 8.3). The signed request is evidence of who set the rule and when, and a provider **SHOULD** keep it with that event, with `accessToken` replaced by its digest and a record that its proof was verified at receipt (see [The access token](#the-access-token)).

### Consent/purpose

Setting a standing rule withholds context from the operators it names on every grant, and can end a grant outright. A Management Client **SHOULD** show the person which operators and categories the rule covers, and which existing grants it would end, before sending it; whether it asks for confirmation is its own policy ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).
