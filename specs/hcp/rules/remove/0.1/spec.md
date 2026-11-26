---
slug: hcp/rules/remove
version: "0.1"
title: "HCP — Remove Standing Rule"
summary: A Management Client, acting for the person or a delegate, removes a Human Context Protocol standing rule; carries the HCP 2.0-draft-3 Remove standing rule operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Remove standing rule
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
    Removing a standing rule widens what clients may receive, and exercises the person's own authority, so the
    request is signed: the provider can show which Management Client removed the rule, and when, independently of the
    transport. The response proof is RECOMMENDED so the Management Client can show the provider acknowledged it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task is destructive and acts for the person, so it is consequential; a re-delivered removal must be placeable
    in the provider's duplicate-execution window.
sideEffects:
  level: destructive
  rationale: >-
    The rule is removed and cannot be restored: setting it again makes a new rule with a new `ruleId` (HCP 6.3). From
    the moment it is removed, the provider stops withholding context because of it.
exposure:
  discloses: none
  actsAsSubject: true
  ingests: secret
  rationale: >-
    The Management Client exercises the Context Subject's (or a delegate's) authority over their own standing rules.
    The request carries the Management Client's access token, a bearer secret.
retention:
  class: durable
  rationale: >-
    The provider appends a `rule.removed` audit event (HCP 6.3, 8.1), which is the person's record of the change.
    The access token is not retained: a kept request holds its digest instead (see Security & Privacy, The access token).
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence", here the Management
      Client's access token.
    retryable: false
related:
  - hcp/rules/set
  - hcp/rules/list
---

## Abstract

The **HCP — Remove Standing Rule** Trust Task carries the Remove standing rule operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 6.3 and 5.4; `DELETE /v2/rules/{ruleId}` in HCP 9.2 and 9.4.14). A *Management Client*, acting for the *Context Subject*, or for a delegate under the Delegated Authorization Profile, tells an *HCP Provider* to remove a standing rule. The provider removes it and records a `rule.removed` audit event. A rule is set with [`hcp/rules/set`](../../set/0.1/spec.md) and read with [`hcp/rules/list`](../../list/0.1/spec.md).

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on standing rules; the ones that bear on the document are restated below, unchanged. Conformance to HCP is HCP's ([SPEC §9.4](/SPEC.md#94-carried-protocols)): a provider **MUST NOT** list a conformance identifier for this carriage in its HCP `conformsTo`, and advertises the task through `trust-task-discovery`, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)). HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1); this specification takes it that a carriage other than that binding may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and carries this one as a Trust Task on that reading, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

This specification adds one check HCP does not make: the token binding of [The access token](#the-access-token), a carriage rule of the `hcp/*` family, not HCP's. [SPEC §9.4](/SPEC.md#94-carried-protocols) item 6 places a check the carried protocol does not make in a separate specification that does not declare `carries`. This family keeps this one in the carriage, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)), because it protects the token that the carriage itself moves from the HTTP `Authorization` header into the document, where it can be stored or forwarded with the document; a separate task could not protect the document this one carries.

## Authorization

The authority is **the Context Subject's own authority over their standing rules**, which a Management Client exercises for them (HCP 6.3, 6.1.9). It is evidenced by the Management Client's access token in `accessToken`, which the provider validates exactly as over HCP's HTTP binding. `accessToken` is OPTIONAL. Where the request carries none, the provider authenticates the request's `issuer` VID, by the request's REQUIRED `proof`, as a Management Client the Authorizing Principal enrolled by authenticating for that purpose (HCP 3.2.3), acting for the Authorizing Principal who enrolled it, and refuses any other issuer with `permissionDenied`, with the same fixed `message` as every other not-authorized refusal. Where it carries one, the token rules apply unchanged, including the token binding of [The access token](#the-access-token). Both are *pending HCP confirmation* ([question 56](#pending-hcp-confirmation)). Removing a standing rule is a governance operation: any caller other than a Management Client acting for the Context Subject, or for a delegate, receives not authorized (HCP 6.1.9, 9.2), carried as `permissionDenied`. A grant held by a Context Client never authorizes it, whatever its scopes (HCP 6.1.9). HCP names only its HTTP Binding as the carriage for a governance operation (HCP 5.4, 9.1); this specification accepts the operation as a Trust Task from a Management Client acting for the Authorizing Principal, and from no other caller (HCP 6.1.9), *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

Under the Delegated Authorization Profile, the Context Subject can remove any standing rule, and a delegate can remove only the rules they set (HCP 6.3, 9.4.14); the provider checks the rule's `setBy` against the caller. A delegate's attempt to remove any other rule is refused with `permissionDenied`.

The REQUIRED `proof` establishes which Management Client VID sent the document and that it is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). Where the request carries `accessToken`, the proof is not the authorization: the token is. Where it carries none, the proof is the Management Client's authentication (see above). A token issued to a client other than the one the `issuer` VID identifies is refused with `permissionDenied` (see [The access token](#the-access-token)).

## Definitions

- **Management Client**, **Context Client**, **HCP Provider**, **Context Subject**, **Authorizing Principal**, **delegate** and **standing rule** have their HCP 2.0-draft-3 meanings.
- HCP defines no request or result schema for this operation. The members come from HCP 9.2 (the path) and 9.4.14.

### HCP field mapping

| HCP field | Member | Notes |
|---|---|---|
| path parameter `{ruleId}` (`DELETE /v2/rules/{ruleId}`, HCP 9.2) | `ruleId` | Moved from the path into the payload, unchanged. |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result `204 No Content` | the empty `#response` payload | No body in HCP, so none here. |

**Where this payload is stricter than HCP's shapes.** Each payload is closed at its top level (`additionalProperties: false`): a top-level member outside this mapping is refused, where HCP's own request and result shapes leave the top level open, so an extension member HCP would allow there is not carried. The HCP shapes nested within stay open, as HCP's are. Every size bound, and every non-empty requirement (a `minLength` or `minItems` of 1) on a member that HCP's own schema allows to be empty, is a Trust Tasks framework bound on what an `hcp/*` party writes, not an HCP requirement, as the schema descriptions say.

### Parties

| HCP role | Trust Task member |
|---|---|
| Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../../entries/query/0.1/spec.md#parties), a Management Client **MUST** send a token only to the provider VID it obtained when the person connected it, and where the provider publishes an HCP discovery document (HCP 19.1), the provider **MUST** make its VID verifiably bound to it and the Management Client **MUST** check that binding before it sends a token to the VID.

## Request

The Management Client sends the removal to the provider. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json): `ruleId`, the rule to remove, and `accessToken`, the Management Client's token. `ruleId` is REQUIRED; `accessToken` is OPTIONAL; where it is absent, the request's `proof` authenticates an enrolled Management Client (see [Authorization](#authorization)).

### Removing a rule that kept health context from an advertising network

```json
{
  "id": "urn:uuid:9c2e4a6b-1d3f-4e5a-8b7c-6d5e4f3a2b10",
  "type": "https://trusttasks.org/spec/hcp/rules/remove/0.1#request",
  "issuer": "did:web:wallet.person.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T09:00:00Z",
  "threadId": "urn:uuid:4b7d9f1a-3c5e-4a2b-8d6f-0e9c8b7a6d52",
  "payload": {
    "ruleId": "urn:uuid:3e9b1c7d-5a2f-4d8e-9b6c-0f1a2e3d4c5b",
    "accessToken": "hcp_mt_8f3a1c9e7b2d4f60"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T09:00:00Z",
    "verificationMethod": "did:web:wallet.person.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z3Kq8vN2mXbT7cY4hR9pL1sW6dF5gJ0aZe2uQ3oVnB8iM4kC7tH1yD9xS6wE5rA2jU8fG3lO7pK0vX4zN1bT6cY9"
  }
}
```

## Response

The provider answers when the rule is removed. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): the empty object, which carries HCP's `204 No Content`. The response is declared, although it carries nothing, because the removal is consequential and its completion is relied upon ([SPEC §4.4.2](/SPEC.md#442-acknowledging-a-fire-and-forget-task)). Failures use `trust-task-error`, not a `#response` document (see [Errors](#errors)).

### The rule is removed

```json
{
  "id": "urn:uuid:2a4c6e8f-0b1d-4f3a-9c5e-7d6b5a4f3e21",
  "type": "https://trusttasks.org/spec/hcp/rules/remove/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:wallet.person.example",
  "issuedAt": "2026-11-25T09:00:01Z",
  "threadId": "urn:uuid:4b7d9f1a-3c5e-4a2b-8d6f-0e9c8b7a6d52",
  "payload": {}
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows.

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request` | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: the caller is not a Management Client acting for the Context Subject or a delegate (HCP 6.1.9), a delegate names a rule they did not set (HCP 9.4.14), or the `ruleId` is unknown or already removed (see [Repeated requests](#repeated-requests)) | `permissionDenied`, with one fixed `message` whatever the reason (HCP 5.5, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| conflict | `idempotency-conflict` | Does not arise: HCP defines it for a retried request with an idempotency key (HCP 9.6), and this operation carries none. A re-delivered document is handled under [Repeated requests](#repeated-requests). |
| version conflict | `version-conflict` | Does not arise: a standing rule has no version (HCP 6.3). |
| credential invalid | `credential-invalid` | Does not arise: it concerns a credential written as an entry's content (HCP 14.6). |
| subject unavailable | `subject-unavailable` | `unavailable` |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion` |

## Rules carried from HCP

- **Effect.** From the moment the rule is removed, the provider stops withholding context because of it, on every grant (HCP 6.3). A grant that was revoked because the rule covered every category it reached stays revoked (HCP 6.3, 6.2.2); removing the rule restores nothing already ended.
- **`notTo`.** An operator listed in a grant's `onwardSharing.notTo` only because of this rule is removed from it (HCP 6.1.5, 6.3: "kept current as rules change").
- **Audit.** The provider appends `rule.removed`, recording `ruleId`, `operators`, `requires`, `categories` and `setBy` (HCP 8.1, 8.2), durable before this task's response is sent (HCP 8.4).
- **Removal is the only override.** A standing rule is overridden only by removing it, which is a separate act from approving any grant (HCP 6.3).

## Repeated requests

HCP 2.0-draft-3 does not say what removing an unknown or already-removed rule yields (HCP 9.4.14). This specification reads HCP as follows, *pending HCP confirmation* ([question 27](#pending-hcp-confirmation)): a `ruleId` the provider does not hold, including one already removed, is refused with `permissionDenied`, with the same fixed `message` as every other not-authorized refusal, so that a caller cannot tell an unknown rule from one it may not remove.

A re-delivered document with the same `id` and content is absorbed by [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements): it is not executed twice, and is answered with the original empty response.

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of [the list of questions put to the editors](../../../_shared/0.1/QUESTIONS.md).

- **Question 1 — conformance claims.** HCP defines no conformance identifier for Trust Task carriage, so a provider **MUST NOT** claim this carriage in its HCP `conformsTo`; it advertises the task with [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead. The binding identifier for the carriage is the same question as [question 37](#pending-hcp-confirmation): whether a carriage other than the HTTP Binding may carry governance operations at all.
- **Question 22 — the client's identity.** The `issuer` VID, authenticated by the request's `proof`, identifies the HCP client the token was issued to; a token issued to any other client **MUST** be refused with `permissionDenied`. This token binding is a carriage rule of this family, a check HCP does not itself make (see [The access token](#the-access-token) and [Conformance](#conformance)).
- **Question 27 — unknown and already-removed rules.** Refused with `permissionDenied`, with the same text as every other not-authorized refusal (see [Repeated requests](#repeated-requests)).
- **Question 37 — governance over a carriage other than the HTTP Binding.** HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1). This specification takes it that another carriage, here a Trust Task, may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and applies every rule HCP places on the operation unchanged (see [Conformance](#conformance)).
- **Question 53 — a request proof on every request that carries a token.** A request `proof` is **REQUIRED** on every `hcp/*` request that carries `accessToken`, so that the token binding of question 22 always applies and a token lifted from a stored or forwarded document cannot be replayed in an unsigned request. This is a carriage rule of this family, not HCP's, and is stricter than HCP's bearer model, under which presenting a valid token is enough (HCP 9.3); see [The access token](#the-access-token).
- **Question 56 — a Management Client authenticated without a token.** `accessToken` is OPTIONAL. Where it is absent, the provider authenticates the request's `issuer` VID, by the request's REQUIRED `proof`, as a Management Client the Authorizing Principal enrolled by authenticating for that purpose, reading HCP 3.2.3's "credentials" as including an enrolled key proved by a signature on each request, and refuses any other issuer with `permissionDenied`, with the one fixed `message`. Where a token is present, the token rules apply unchanged, including the binding of question 22. The request `proof` stays REQUIRED whether or not a token is present (question 53), and a provider keeps a record of the Management Clients each person enrolled. This is not question 37, which asks whether governance may travel off the HTTP Binding at all; it asks how the caller is authenticated once it does (see [Authorization](#authorization)).

## What HCP requires that a document cannot express

- The provider-held standing rules, and their enforcement on every request and every grant (HCP 6.3).
- Keeping each affected grant's `notTo` current (HCP 6.1.5).
- The `rule.removed` audit event and its durability (HCP 8.1, 8.4).
- Carriage by the HTTP Binding: HCP carries governance operations only over that binding (HCP 1.6, 5.4, 9.1), and a Trust Task is not it. This specification carries the operation for a Management Client acting for the Authorizing Principal (HCP 6.1.9) instead, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

## Security & Privacy

### Data carried

The request carries a rule identifier and the Management Client's access token where it carries one, a bearer secret that **MUST** travel only to the provider. It carries no context. The response carries nothing. A producer **MUST NOT** put the token or any context in `ext`.

### The access token

`accessToken`, where a request carries it, is a bearer secret: whoever presents it exercises the authority it evidences. It **MUST** travel only to the provider. A recipient **MUST NOT** log `accessToken` or retain it beyond the exchange.

**In an agent runtime**, the token is held as [`hcp/entries/query`](../../../entries/query/0.1/spec.md#the-access-token) requires: by the component that builds and sends the document, never in a model's context or a tool's result (HCP 10.5).

**Token binding, a carriage rule of this family.** Every request that carries `accessToken` carries a `proof`, *pending HCP confirmation* ([question 53](#pending-hcp-confirmation)), and the provider **MUST** refuse with `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies, so that a token lifted from a stored or forwarded document is of no use from another VID. HCP validates the token alone and does not bind it to a VID (HCP 9.3): this rule is the `hcp/*` family's, not HCP's, and rests on the `issuer` VID identifying the HCP client, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)). It is a check HCP does not make (see [Conformance](#conformance)).

**Retained copies.** The provider verifies the request's `proof` over the document as received, at receipt, before it removes anything ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). A provider that retains the request document, as a duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) or as evidence of the request, **MUST** remove `accessToken` from the copy it keeps and record in its place `accessTokenDigest`: the SHA-256 multihash of the token's UTF-8 bytes, multibase-encoded as base64url without padding (prefix `u`), the framework's `DigestMultibase` form ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)). The kept copy's `proof` no longer verifies over it, so the provider records with the copy that the proof was verified at receipt: when, under which verification method, and with what result. It relies on that record, and **MUST NOT** verify the proof again over the kept copy. A record that keeps only a digest of the document, such as its task digest ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)), keeps no token either.

### Correlation

The provider learns which Management Client removed which rule and when; that is the record the person needs. The `ruleId` is known only to the provider and the person's Management Clients. `threadId` **SHOULD** be fresh per removal.

### Retention

The provider keeps the `rule.removed` audit event (HCP 8.1), readable by the Context Subject (HCP 8.3). The signed request is evidence of who removed the rule and when, and a provider **SHOULD** keep it with that event, with `accessToken` replaced by its digest and a record that its proof was verified at receipt (see [The access token](#the-access-token)).

### Consent/purpose

Removing a standing rule lets clients that it excluded receive context again under grants that reach it. A Management Client **SHOULD** tell the person which operators and categories the rule covered before sending the removal; whether it asks for confirmation is its own policy ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).
