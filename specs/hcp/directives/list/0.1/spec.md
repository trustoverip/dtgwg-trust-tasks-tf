---
slug: hcp/directives/list
version: "0.1"
title: "HCP — List Directives"
summary: A Management Client, acting for the person or a delegate, reads the Human Context Protocol directives a provider holds; carries the HCP 2.0-draft-3 List directives operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: List directives
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
    A read of metadata whose authority is the Management Client's access token, which the provider checks whatever
    the binding, and which changes nothing. The request proof is REQUIRED, as on every hcp/* request that carries a
    token, because it is what lets the provider refuse a token sent from a VID other than the client's (a carriage rule pending HCP confirmation, question 53). The response proof is RECOMMENDED so the Management Client
    can show which directives the provider said it held, and when.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task is not consequential, but its answer describes the directives in force now, which the person relies on to
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
    The response lists the directives, in the person's own words, which carry no context. A read produces no
    attributable effect in the person's name, so `actsAsSubject` is false (SPEC 7.3 item 14), as for every Management
    Client read. The request carries the Management Client's access token, a bearer secret.
retention:
  class: transient
  rationale: >-
    HCP requires no audit event for reading the directives (HCP 8.1 lists none), and the provider keeps nothing
    because of this task. The directives themselves are kept because they were set, not because they were read.
maxDocumentBytes:
  response: 4194304
  rationale: >-
    The list is not paginated (HCP 9.4.15), and HCP bounds neither the number of directives nor their size. A
    directive's `text` is bounded at 4096 characters by the carriage, and a typical directive is well under 1 KiB, so
    4 MiB holds thousands of directives with envelope and proof. This is a framework bound, not an HCP requirement.
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence", here the Management
      Client's access token.
    retryable: false
related:
  - hcp/directives/set
  - hcp/directives/remove
  - hcp/rules/list
---

## Abstract

The **HCP — List Directives** Trust Task carries the List directives operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 6.4 and 5.4; `GET /v2/directives` in HCP 9.2 and 9.4.15). A *Management Client*, acting for the *Context Subject*, or for a delegate under the Delegated Authorization Profile, asks an *HCP Provider* for the directives it holds, and the provider answers with them. A directive is set with [`hcp/directives/set`](../../set/0.1/spec.md) and removed with [`hcp/directives/remove`](../../remove/0.1/spec.md).

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on directives; the ones that bear on the document are restated below, unchanged. Conformance to HCP is HCP's ([SPEC §9.4](/SPEC.md#94-carried-protocols)): a provider **MUST NOT** list a conformance identifier for this carriage in its HCP `conformsTo`, and advertises the task through `trust-task-discovery`, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)). HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1); this specification takes it that a carriage other than that binding may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and carries this one as a Trust Task on that reading, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

This specification adds one check HCP does not make: the token binding of [The access token](#the-access-token), a carriage rule of the `hcp/*` family, not HCP's. [SPEC §9.4](/SPEC.md#94-carried-protocols) item 6 places a check the carried protocol does not make in a separate specification that does not declare `carries`. This family keeps this one in the carriage, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)), because it protects the token that the carriage itself moves from the HTTP `Authorization` header into the document, where it can be stored or forwarded with the document; a separate task could not protect the document this one carries.

## Authorization

The authority is **the Context Subject's own authority over their directives**, which a Management Client exercises for them (HCP 6.4, 6.1.9). It is evidenced by the Management Client's access token in `accessToken`, which the provider validates exactly as over HCP's HTTP binding. `accessToken` is OPTIONAL. Where the request carries none, the provider authenticates the request's `issuer` VID, by the request's REQUIRED `proof`, as a Management Client the Authorizing Principal enrolled by authenticating for that purpose (HCP 3.2.3), acting for the Authorizing Principal who enrolled it, and refuses any other issuer with `permissionDenied`, with the same fixed `message` as every other not-authorized refusal. Where it carries one, the token rules apply unchanged, including the token binding of [The access token](#the-access-token). Both are *pending HCP confirmation* ([question 56](#pending-hcp-confirmation)). Listing directives is a governance operation: any caller other than a Management Client acting for the Context Subject, or for a delegate, receives not authorized (HCP 6.1.9, 9.2, 9.4.15), carried as `permissionDenied`. A grant held by a Context Client never authorizes it, whatever its scopes (HCP 6.1.9); a Context Client receives the directives that apply to it with its results and its own grant instead (HCP 6.4). HCP names only its HTTP Binding as the carriage for a governance operation (HCP 5.4, 9.1); this specification accepts the operation as a Trust Task from a Management Client acting for the Authorizing Principal, and from no other caller (HCP 6.1.9), *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

Under the Delegated Authorization Profile, HCP lets a delegate call this operation as for standing rules, "within the delegation's scope" (HCP 9.4.15, 9.4.14), and a delegate's governance operations reach only categories within scope (HCP 17.4), but HCP does not say which directives a delegate sees. This specification takes the narrowest reading, *pending HCP confirmation* ([question 28](#pending-hcp-confirmation)): a delegate's list carries each directive whose `categories` are absent, or are all within the delegation's scope; a directive that names any category outside the scope is left out whole, never trimmed, so the list never discloses an out-of-scope category. The Context Subject's list carries every directive.

The `proof` establishes which Management Client VID sent the document and that it is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). Where the request carries `accessToken`, the proof is not the authorization: the token is. Where it carries none, the proof is the Management Client's authentication (see above). The request always carries a proof, *pending HCP confirmation* ([question 53](#pending-hcp-confirmation)), so a token issued to a client other than the one the `issuer` VID identifies is always refused with `permissionDenied` (see [The access token](#the-access-token)).

## Definitions

- **Management Client**, **Context Client**, **HCP Provider**, **Context Subject**, **Authorizing Principal**, **delegate** and **directive** have their HCP 2.0-draft-3 meanings.
- HCP defines no request or result schema for this operation. The request has no member but the token (HCP 9.4.15); each directive in the result is the directive of HCP 6.4, the shared [`Directive`](../../../_shared/0.1/directive.schema.json).

### HCP field mapping

| HCP field | Member | Notes |
|---|---|---|
| `GET /v2/directives` (HCP 9.2, 9.4.15) | the Type URI | No query parameters in HCP, so no request member beyond the token. |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result `200`: a `directives` array (HCP 9.4.15) | `directives` in the `#response` payload | Unchanged. Not paginated: HCP gives this operation no cursor ([question 3](#pending-hcp-confirmation)). |

**Where this payload is stricter than HCP's shapes.** Each payload is closed at its top level (`additionalProperties: false`): a top-level member outside this mapping is refused, where HCP's own request and result shapes leave the top level open, so an extension member HCP would allow there is not carried. The HCP shapes nested within stay open, as HCP's are. Every size bound, and every non-empty requirement (a `minLength` or `minItems` of 1) on a member that HCP's own schema allows to be empty, is a Trust Tasks framework bound on what an `hcp/*` party writes, not an HCP requirement, as the schema descriptions say.

### Parties

| HCP role | Trust Task member |
|---|---|
| Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../../entries/query/0.1/spec.md#parties), a Management Client **MUST** send a token only to the provider VID it obtained when the person connected it, and where the provider publishes an HCP discovery document (HCP 19.1), the provider **MUST** make its VID verifiably bound to it and the Management Client **MUST** check that binding before it sends a token to the VID.

## Request

The Management Client asks for the directives. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json): `accessToken`, the Management Client's token, OPTIONAL and the only member: a request with no token has the empty payload, and its `proof` authenticates an enrolled Management Client (see [Authorization](#authorization)).

### Reading the person's directives

```json
{
  "id": "urn:uuid:a1c3e5f7-9b2d-4f4a-8c6e-0d2b4f6a8c13",
  "type": "https://trusttasks.org/spec/hcp/directives/list/0.1#request",
  "issuer": "did:web:wallet.person.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T13:00:00Z",
  "threadId": "urn:uuid:5e7a9c1b-3d2f-4a6e-9b8c-2f4d6a8e0c57",
  "payload": {
    "accessToken": "hcp_mt_8f3a1c9e7b2d4f60"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T13:00:00Z",
    "verificationMethod": "did:web:wallet.person.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z6Rt3mV9qNcX2wB8hL4kP1sY7dF5gJ0aQe2uK3oZnC8iT4pD7wH1yM9vE6bS5rA2jU8lG3fO7xN0tZ4zW1cK6mY8"
  }
}
```

## Response

The provider answers with the directives. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): `directives`, an array of directives, empty when there is none. HCP sets no order. Failures use `trust-task-error`, not a `#response` document (see [Errors](#errors)).

### Two directives

```json
{
  "id": "urn:uuid:8b0d2f4a-6c1e-4b3a-a5d7-9e1c3b5d7f62",
  "type": "https://trusttasks.org/spec/hcp/directives/list/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:wallet.person.example",
  "issuedAt": "2026-11-25T13:00:01Z",
  "threadId": "urn:uuid:5e7a9c1b-3d2f-4a6e-9b8c-2f4d6a8e0c57",
  "payload": {
    "directives": [
      {
        "directiveId": "urn:uuid:d5f7b9c1-3e2a-4d6f-8b0e-2c4a6e8f0b19",
        "action": "share",
        "effect": "deny",
        "text": "Never pass my health context to another agent.",
        "categories": ["health_context"],
        "setBy": "context-subject",
        "createdAt": "2026-11-25T12:00:01Z"
      },
      {
        "directiveId": "urn:uuid:e6a8c0d2-4f3b-4e7a-9c1d-3b5f7a9c1e28",
        "action": "sign",
        "effect": "ask-first",
        "text": "Ask me before you sign anything.",
        "setBy": "context-subject",
        "createdAt": "2026-11-18T08:15:00Z"
      }
    ]
  }
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows.

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request` | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: the caller is not a Management Client acting for the Context Subject or a delegate (HCP 6.1.9, 9.4.15) | `permissionDenied`, with one fixed `message` whatever the reason (HCP 5.5, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| conflict | `idempotency-conflict` | Does not arise: HCP defines it for a retried request with an idempotency key (HCP 9.6), and a read carries none. |
| version conflict | `version-conflict` | Does not arise: a directive has no version (HCP 6.4). |
| credential invalid | `credential-invalid` | Does not arise: it concerns a credential written as an entry's content (HCP 14.6). |
| subject unavailable | `subject-unavailable` | `unavailable` |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion` |

## Rules carried from HCP

- **What a directive is.** Each listed directive tells clients what they may do with context they received, or on the person's behalf, with the `directiveId`, `setBy` and `createdAt` the provider set (HCP 6.4). Directives only narrow, and where two that apply disagree, `deny` prevails over `ask-first`, and `ask-first` over `permit` (HCP 6.4).
- **No audit event.** HCP 8.1 lists no event for reading the directives, and this task adds none.
- **Not paginated.** HCP 9.4.15 answers with the whole `directives` array; this task does the same ([question 3](#pending-hcp-confirmation)).
- **Removed directives.** A removed directive is no longer held, so it is not listed; its `directive.removed` audit event stays readable through the audit log (HCP 8.1, 8.3).

## Repeated requests

Reading has no effect, so a re-delivered document changes nothing. A consumer still applies [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements) to a re-delivered document with the same `id` and content, and answers it with the original response; a Management Client that wants the current directives sends a new document.

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of [the list of questions put to the editors](../../../_shared/0.1/QUESTIONS.md).

- **Question 1 — conformance claims.** HCP defines no conformance identifier for Trust Task carriage, so a provider **MUST NOT** claim this carriage in its HCP `conformsTo`; it advertises the task with [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead. The binding identifier for the carriage is the same question as [question 37](#pending-hcp-confirmation): whether a carriage other than the HTTP Binding may carry governance operations at all.
- **Question 3 — pagination.** The directives list is not paginated, as in HCP 9.4.15.
- **Question 22 — the client's identity.** The `issuer` VID, authenticated by the request's `proof`, identifies the HCP client the token was issued to; a token issued to any other client **MUST** be refused with `permissionDenied`. This token binding is a carriage rule of this family, a check HCP does not itself make (see [The access token](#the-access-token) and [Conformance](#conformance)).
- **Question 28 — what a delegate's list carries.** Each directive whose `categories` are absent or all within the delegation's scope; a directive naming any other category is left out whole (see [Authorization](#authorization)).
- **Question 37 — governance over a carriage other than the HTTP Binding.** HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1). This specification takes it that another carriage, here a Trust Task, may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and applies every rule HCP places on the operation unchanged (see [Conformance](#conformance)).
- **Question 53 — a request proof on every request that carries a token.** A request `proof` is **REQUIRED** on every `hcp/*` request that carries `accessToken`, so that the token binding of question 22 always applies and a token lifted from a stored or forwarded document cannot be replayed in an unsigned request. This is a carriage rule of this family, not HCP's, and is stricter than HCP's bearer model, under which presenting a valid token is enough (HCP 9.3); see [The access token](#the-access-token).
- **Question 56 — a Management Client authenticated without a token.** `accessToken` is OPTIONAL. Where it is absent, the provider authenticates the request's `issuer` VID, by the request's REQUIRED `proof`, as a Management Client the Authorizing Principal enrolled by authenticating for that purpose, reading HCP 3.2.3's "credentials" as including an enrolled key proved by a signature on each request, and refuses any other issuer with `permissionDenied`, with the one fixed `message`. Where a token is present, the token rules apply unchanged, including the binding of question 22. The request `proof` stays REQUIRED whether or not a token is present (question 53), and a provider keeps a record of the Management Clients each person enrolled. This is not question 37, which asks whether governance may travel off the HTTP Binding at all; it asks how the caller is authenticated once it does (see [Authorization](#authorization)).

## What HCP requires that a document cannot express

- The provider-held directives, and their delivery with every result they apply to (HCP 6.4).
- Which directives a delegate may see, which turns on the delegation the provider holds (HCP 17.4).
- Carriage by the HTTP Binding: HCP carries governance operations only over that binding (HCP 1.6, 5.4, 9.1), and a Trust Task is not it. This specification carries the operation for a Management Client acting for the Authorizing Principal (HCP 6.1.9) instead, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

## Security & Privacy

### Data carried

The request carries only the Management Client's access token where it carries one, a bearer secret that **MUST** travel only to the provider. The response carries the directives, in the person's own words. They carry no context, but a directive's `text` and categories can say something about the person. A producer **MUST NOT** put the token or any context in `ext`.

### The access token

`accessToken`, where a request carries it, is a bearer secret: whoever presents it exercises the authority it evidences. It **MUST** travel only to the provider. A recipient **MUST NOT** log `accessToken` or retain it beyond the exchange.

**In an agent runtime**, the token is held as [`hcp/entries/query`](../../../entries/query/0.1/spec.md#the-access-token) requires: by the component that builds and sends the document, never in a model's context or a tool's result (HCP 10.5).

**Token binding, a carriage rule of this family.** Every request that carries `accessToken` carries a `proof`, *pending HCP confirmation* ([question 53](#pending-hcp-confirmation)), and the provider **MUST** refuse with `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies, so that a token lifted from a stored or forwarded document is of no use from another VID. HCP validates the token alone and does not bind it to a VID (HCP 9.3): this rule is the `hcp/*` family's, not HCP's, and rests on the `issuer` VID identifying the HCP client, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)). It is a check HCP does not make (see [Conformance](#conformance)).

**Retained copies.** The provider verifies the request's `proof` over the document as received, at receipt, before it removes anything ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). A provider that retains the request document, as a duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) or as evidence of the request, **MUST** remove `accessToken` from the copy it keeps and record in its place `accessTokenDigest`: the SHA-256 multihash of the token's UTF-8 bytes, multibase-encoded as base64url without padding (prefix `u`), the framework's `DigestMultibase` form ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)). The kept copy's `proof` no longer verifies over it, so the provider records with the copy that the proof was verified at receipt: when, under which verification method, and with what result. It relies on that record, and **MUST NOT** verify the proof again over the kept copy. A record that keeps only a digest of the document, such as its task digest ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)), keeps no token either.

### Correlation

The provider learns that a Management Client read the directives, and when; HCP does not require it to record that. A `setBy` that is a `delegationId` tells the reader which delegate set a directive; the reader is the Context Subject or a delegate of the same person, so this discloses nothing outside the person's own governance. `threadId` **SHOULD** be fresh per request.

### Retention

The provider keeps nothing because of this task. A Management Client **SHOULD NOT** keep the list longer than it needs to show it to the person, because a kept copy goes stale as soon as a directive is set or removed.

### Consent/purpose

The directives are the person's own settings, read for them so they can see and change them. Through this task only the person's own Management Clients, or their delegates', receive the whole list; a Context Client receives only the directives that apply to it, with its results (HCP 6.4).
