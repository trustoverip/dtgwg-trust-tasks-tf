---
slug: hcp/entries/get
version: "0.1"
title: "HCP — Get Entry"
summary: An HCP Client asks a Human Context Protocol provider for one named entry, for a stated purpose, under a grant; carries the HCP 2.0-draft-3 Get entry operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Get entry
category: data-exchange
keywords: [hcp, human-context-protocol, personal-context, memory, entry, disclosure]
parties:
  - role: HCP Client (a Context Client, or a Management Client)
    requirement: REQUIRED
    member: issuer
  - role: HCP Provider
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: RECOMMENDED
  response: REQUIRED
  rationale: >-
    The response discloses a person's context, so it carries the provider's signature: the client can show which
    provider released which entry, when, and with which use limit. A request proof is RECOMMENDED, not REQUIRED,
    because the grant is evidenced by the access token, which the provider checks on every request.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task discloses secret material and so is consequential; a re-delivered request must be placeable in the
    consumer's duplicate-execution window, so that it is absorbed instead of disclosing and auditing twice.
sideEffects:
  level: none
  rationale: >-
    A read. The provider appends a `context.read` audit event (HCP 5.2.2, 8.1), which is recordkeeping HCP requires
    of every read, not a change the client asked for.
exposure:
  discloses: secret
  actsAsSubject: false
  ingests: secret
  rationale: >-
    The response discloses one entry of a person's context, which may be in a High-sensitivity category. The request
    carries the HCP access token, a bearer secret.
retention:
  class: durable
  rationale: >-
    The provider keeps the `context.read` audit event HCP requires for every read, with the stated purpose (HCP
    5.2.2, 8.1). The client stops using the disclosed copy at its `useUntil` (HCP 7.6).
    The access token is not retained (see Security & Privacy, The access token).
maxDocumentBytes:
  response: 524288
  rationale: >-
    One disclosure, which may carry a verifiable credential as evidence, plus directives, envelope and proof. This is
    a framework bound, not an HCP requirement; HCP sets no size limit.
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence". A request under a
      revoked grant also fails here, pending HCP confirmation (question 2 in the spec's "Pending HCP confirmation"
      section).
    retryable: false
related:
  - hcp/entries/query
  - hcp/entries/revise
  - hcp/entries/withdraw
---

## Abstract

The **HCP — Get Entry** Trust Task carries the Get entry operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 5.2.2; `GET /v2/entries/{entryId}` in HCP 9.2). An *HCP Client* asks an *HCP Provider* for one entry it names, for a stated purpose, under the grant its access token stands for. A *Context Client* names the entry by the grant-scoped `ref` it was disclosed with and receives a disclosure with a use limit; a *Management Client* names it by its `id` and receives the whole entry.

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)): it carries one HCP operation, unchanged, so that a client can use it over any Trust Task binding.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and does not copy HCP's own schema files. Where it differs from the HCP text, the HCP text governs. When HCP changes the operation, this specification moves to a new version.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on the Get entry operation; this specification restates the ones that bear on the document and does not relax any. A provider **MUST NOT** claim this carriage in its HCP `conformsTo`, and advertises the task through [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)).

## Authorization

The authority is **the grant** the person gave the client (HCP 6.1), evidenced by the access token in `accessToken`. The provider **MUST** check the grant on every request (HCP 6.2.2), exactly as over HCP's HTTP binding. At least one of the entry's categories **MUST** be within the grant with `purpose` among the purposes approved for it; no standing rule may deny the client's Client Operator any of the entry's categories (HCP 6.3); and the entry's permitted purposes, where present, **MUST** include `purpose` (HCP 4.2.6, 5.2.2). A narrowed grant reaches only the entries its narrowing allows (HCP 6.1.5.1, rule 4).

A Context Client names only entries disclosed to it under the same grant, since it knows no other `ref` (HCP 5.2.2). A Context Client that names an entry by `entryId` is refused with `permissionDenied`, *pending HCP confirmation* ([question 6](#pending-hcp-confirmation)).

As for [`hcp/entries/query`](../../query/0.1/spec.md#authorization), neither the document's `proof` nor the `issuer` VID is the authorization; the token is. Where the request carries a proof, a token issued to a client other than the one the `issuer` VID identifies is refused with `permissionDenied` (see [The access token](#the-access-token)); where it carries none, a provider that knows which VID it registered the client under **SHOULD** still check that the `issuer` is that client.

## Definitions

- **Context Client**, **Management Client**, **HCP Provider**, **grant**, **entry**, **disclosure**, **directive** and **category** have their HCP 2.0-draft-3 meanings.
- The shared shapes are in [`hcp/_shared/0.1`](../../../_shared/0.1/common.schema.json): [`Disclosure`](../../../_shared/0.1/disclosure.schema.json), [`Entry`](../../../_shared/0.1/context-entry.schema.json) and [`Directive`](../../../_shared/0.1/directive.schema.json). They are open, as HCP's own schemas are.

### HCP field mapping

| HCP Get entry field | Member | Notes |
|---|---|---|
| request `ref` or `entryId` (the `{entryId}` path segment, HCP 9.4.2) | `ref` or `entryId` | Moved from the path into the payload. Exactly one is present. |
| request `purpose` | `purpose` | Unchanged. |
| request `purposeDescription` | `purposeDescription` | Unchanged. |
| request `version` | `version` | Unchanged. |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result `entry` | `entry` | Unchanged. |
| result `retrievalId` | `retrievalId` | Unchanged. |
| result `minimization` | `minimization` | Unchanged. |
| result `directives` | `directives` | Unchanged. |
| result `timestamp` | `respondedAt` | **Renamed**, as in `hcp/entries/query`. Same type and meaning. |

### Parties

| HCP role | Trust Task member |
|---|---|
| Context Client or Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../query/0.1/spec.md#parties), a provider **MUST** make its VID verifiably bound to its HCP identity (its discovery document, HCP 19.1), and a client **MUST** check that binding before it sends a token to a VID.

## Request

The client names the entry and the purpose. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

- `ref` (a Context Client) or `entryId` (a Management Client) names the entry. Exactly one is present; a request with both or neither is refused with `malformedRequest`, *pending HCP confirmation* ([question 6](#pending-hcp-confirmation)). The schema does not check this.
- `purpose` (REQUIRED) is the purpose code; `purposeDescription` (OPTIONAL) says it in words.
- `version` (OPTIONAL) asks for that version instead of the current one.
- `accessToken` (REQUIRED) is the HCP access token for the grant.

### A Context Client reads an entry it was disclosed

```json
{
  "id": "urn:uuid:2b6e9c41-7d3a-4f8e-a1c5-9e0d4b3a2f61",
  "type": "https://trusttasks.org/spec/hcp/entries/get/0.1#request",
  "issuer": "did:web:booking-agent.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T09:00:00Z",
  "threadId": "urn:uuid:8d3f1a72-4c6e-4b9a-b2d1-5e7f0a9c8b62",
  "payload": {
    "ref": "hcp-ref-5e02bd",
    "purpose": "https://hcp.example/purposes/reservation",
    "purposeDescription": "Confirm the allergy before ordering.",
    "accessToken": "hcp_at_2b7f0c6e9d1a4e8b"
  }
}
```

## Response

The provider answers with the entry. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json).

- `entry` is a `Disclosure` for a Context Client (HCP 7.4), with its `useUntil` (HCP 7.6), and the whole `Entry` for a Management Client.
- `retrievalId` identifies this retrieval in the provider's audit log.
- `minimization` is the minimization report (HCP 7.5). Its `method` is `none`: entry-selection minimization is inapplicable when the caller names a single entry, and its absence is not non-conformance (HCP 5.2.2).
- `directives` are the directives that apply, where the requester is a Context Client (HCP 6.4). The schema leaves the member optional, as HCP's does, but every result to a Context Client carries it, empty when none applies.
- `respondedAt` is HCP's `timestamp`, renamed.

Failures use `trust-task-error`, not a `#response` document (see [Errors](#errors)).

### The entry, disclosed

```json
{
  "id": "urn:uuid:5c1a8e93-2f4b-4d7c-9e6a-0b3d2c1f4e72",
  "type": "https://trusttasks.org/spec/hcp/entries/get/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:booking-agent.example",
  "issuedAt": "2026-11-25T09:00:01Z",
  "threadId": "urn:uuid:8d3f1a72-4c6e-4b9a-b2d1-5e7f0a9c8b62",
  "payload": {
    "entry": {
      "ref": "hcp-ref-5e02bd",
      "version": 1,
      "content": {
        "profile": "https://hcp.example/spec/profiles/general-context/2.0",
        "value": "Severe peanut allergy.",
        "language": "en"
      },
      "assertion": { "declarationType": "person-declared" },
      "classification": { "categories": ["health_context"] },
      "standing": { "assurance": "authenticated", "confirmed": true },
      "useUntil": "2026-11-25T21:00:01Z"
    },
    "retrievalId": "urn:uuid:0e7d4c2b-9a1f-4e3d-8b6c-2a5f1e0d9c83",
    "minimization": { "method": "none" },
    "directives": [
      {
        "directiveId": "urn:uuid:5b8c2e17-4a9d-4f3b-8e6a-1c0d9f7e2b44",
        "action": "share",
        "effect": "deny",
        "text": "Never pass my health information to another agent.",
        "categories": ["health_context"],
        "setBy": "context-subject",
        "createdAt": "2026-10-02T08:30:00Z"
      }
    ],
    "respondedAt": "2026-11-25T09:00:01Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T09:00:01Z",
    "verificationMethod": "did:web:context.provider.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z4Gk2pXwVqN7sLmT3dRcB9yHf6jZa1eU8oKnQ5vC2xWb7MtE4rP9gS3iJ6hA1yD8uF5lO2wX9zC7nB4mV6kT3qR"
  }
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows. The provider never sends HCP's Core error object or the HTTP binding's problem details (HCP 9.7).

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request` | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: an entry that does not exist, is outside the caller's scopes, whose `ref` belongs to another grant, that is not `active`, or for which `purpose` is not approved (HCP 5.2.2) | `permissionDenied`, with one fixed `message` whatever the reason, so that entry existence is not disclosed (HCP 5.2.2, 5.5, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| conflict | `idempotency-conflict` | Does not arise: a read carries no idempotency key (HCP 9.6). |
| version conflict | `version-conflict` | Does not arise: a `version` that does not exist is an entry that does not exist, refused as not authorized (HCP 5.2.2). |
| credential invalid | `credential-invalid` | Does not arise: a read verifies no newly written credential (HCP 14.6). |
| subject unavailable | `subject-unavailable` | `unavailable`. The client may retry later (HCP 5.5). |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion` |

A client that does not know `hcp:invalidToken` reads it as `taskFailed` ([SPEC §8.5](/SPEC.md#85-extension-by-individual-trust-task-specifications)) and still honours `retryable: false`.

## Repeated requests

HCP requires a `context.read` event for every get entry (HCP 5.2.2, 8.1). As for [`hcp/entries/query`](../../query/0.1/spec.md#repeated-requests), a re-delivered document with the same `id` and content is absorbed by [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements) and causes no second disclosure and no second event, *pending HCP confirmation* ([question 4](#pending-hcp-confirmation)).

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 1 — conformance claims.** A provider **MUST NOT** claim this carriage in its HCP `conformsTo` (see [Conformance](#conformance)).
- **Question 2 — a request under a revoked grant.** Refused with `hcp:invalidToken` (see [Errors](#errors)).
- **Question 4 — a re-delivered read.** It causes no second `context.read` audit event (see [Repeated requests](#repeated-requests)).
- **Question 6 — naming the entry.** Exactly one of `ref` and `entryId`; both or neither is `malformedRequest`, and a Context Client that sends `entryId` gets `permissionDenied` (see [Request](#request), [Authorization](#authorization)).
- **Question 22 — the client's identity.** The `issuer` VID, authenticated by the request's `proof`, identifies the HCP client the token was issued to; where the request carries a proof, a token issued to any other client is refused with `permissionDenied`, a strengthening HCP does not itself require (see [The access token](#the-access-token)).

## What HCP requires that a document cannot express

- **The grant, scope, standing-rule and purpose checks** (HCP 5.2.2, 6.2.2, 6.3, 4.2.6), and the narrowing of HCP 6.1.5.1: they are made against provider-held state.
- **Existence hiding** (HCP 5.2.2, 5.5): every refusal looks the same.
- **Use limits** (HCP 7.6) and **directives** (HCP 6.4): carried, but honouring them is on the client.

## Security & Privacy

### Data carried

The request carries the access token, a bearer secret, and names one entry and a purpose. The response carries one entry of the person's context, possibly in a High-sensitivity category, possibly with a credential as evidence. A producer **MUST NOT** put context or the token in `ext` or in any `message`. Because the response proof is REQUIRED ([SPEC §7.3 item 8](/SPEC.md#73-specification-requirements)), the disclosure reaches the client as a signed assertion; a client **MUST NOT** pass the signed response, or any part of it, to anyone the grant does not cover (HCP 3.12).

### The access token

`accessToken` is a bearer secret: whoever presents it exercises the authority it evidences. It **MUST** travel only to the provider. A recipient **MUST NOT** log `accessToken` or retain it beyond the exchange: a duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) or an evidentiary record of the request keeps the document with `accessToken` removed, or a digest of the document, such as its task digest ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)), and never the token. Where the request carries a `proof`, the provider **MUST** refuse with `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies, so that a token lifted from a stored or forwarded document is of no use from another VID. HCP does not itself bind a token to a VID: this is a strengthening of the carriage, resting on the `issuer` VID identifying the HCP client, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)).

### Correlation

The provider sees which client read which entry, when, and for which purpose; that is what the audit log records. A Context Client sees the entry only by its grant-scoped `ref`, which differs under every other grant (HCP 7.4), so two clients cannot join their copies on it. The token joins every request made under the grant. `threadId` **SHOULD** be fresh per request.

### Retention

The provider keeps the `context.read` audit event (HCP 8.1), readable by the Context Subject (HCP 8.3). The client stops using the copy at its `useUntil` (HCP 7.6), and at once when the grant is revoked (HCP 6.2.2), except a copy kept under a declared legal obligation (HCP 6.2.5).

### Consent/purpose

The entry is disclosed for the `purpose` named, which must be one the grant approved for a category of the entry, and for no other. Whether a given read needs any further approval is the provider's policy under HCP, not a requirement of this specification ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).

The acceptance window for `issuedAt` is consumer policy; a provider **SHOULD** apply a short one, minutes rather than hours.
