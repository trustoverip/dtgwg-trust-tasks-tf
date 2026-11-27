---
slug: hcp/entries/query
version: "0.1"
title: "HCP — Query Context"
summary: A Context Client asks a Human Context Protocol provider for the context relevant to a query, for a named purpose, under a grant; carries the HCP 2.0-draft-3 Query operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Query
category: data-exchange
keywords: [hcp, human-context-protocol, personal-context, memory, query, disclosure]
parties:
  - role: HCP Client (a Context Client, or a Management Client)
    requirement: REQUIRED
    member: issuer
  - role: HCP Provider
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The response discloses a person's context, so it carries the provider's signature: the client can show which
    provider released which context, when, and with which use limits, which is what the person's audit trail and
    any later dispute turn on. The request proof is REQUIRED too. The grant is evidenced by the access token, not
    the proof, but the proof is what lets the provider refuse a token sent from a VID other than the client's, so
    that a token lifted from a stored or forwarded query cannot be replayed in an unsigned one (a carriage rule pending HCP confirmation, question 53).
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task discloses secret material and so is consequential; a re-delivered query must be placeable in the
    consumer's duplicate-execution window, so that it is absorbed instead of disclosing and auditing twice.
sideEffects:
  level: none
  rationale: >-
    A read. The provider appends a `context.read` audit event and mints grant-scoped `ref`s (HCP 8.1, 7.4), but
    both are recordkeeping HCP requires of every read, not a change the client asked for.
exposure:
  discloses: secret
  actsAsSubject: false
  ingests: secret
  rationale: >-
    The response discloses a person's personal context, which may include health and other High-sensitivity
    categories. The request carries the HCP access token, a bearer secret, and the query text, which can itself say
    something about the person.
retention:
  class: durable
  rationale: >-
    The provider keeps the `context.read` audit event HCP requires for every read (HCP 8.1), so the person can see
    who read what. HCP's audit event records the `purpose` and `purposeDescription` but has no member for the query
    text (HCP 8.2). The client stops using each disclosed copy at its `useUntil` (HCP 7.6).
    The access token is not retained: a kept request holds its digest instead (see Security & Privacy, The access token).
maxDocumentBytes:
  response: 12582912
  rationale: >-
    Up to 20 disclosures (HCP 7.2), each of which may carry a verifiable credential as evidence: 20 times the 512 KiB
    response bound of hcp/entries/get, so that 20 disclosures each of a size get can return always fit, plus 2 MiB for
    the directives, the minimization report, envelope and proof. This is a framework bound, not an HCP requirement;
    HCP sets no size limit.
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence", here the access token
      the request presents. A token issued under a revoked grant, a relinquished one included (HCP 5.2.7), fails here
      too: HCP requires that no request presenting evidence issued under a revoked grant succeeds, but names no error
      (HCP 6.2.2), and this family reads that evidence as invalid, pending HCP confirmation (question 2). Such a
      failure is itself a notice of revocation to the client (HCP 6.2.2). Defined once for the family in
      [the hcp family conventions](/specs/hcp/_shared/0.1/CONVENTIONS.md#1-family-error-codes).
    retryable: false
related:
  - hcp/grants/revoke
  - hcp/grants/revoked
---

## Abstract

The **HCP — Query Context** Trust Task carries the Query operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 5.2.1). A *Context Client* asks an *HCP Provider* for the context about a person that is relevant to a query, for a named purpose, under the grant its access token evidences. The provider answers with at most 20 disclosed entries, each with a use limit, together with the directives the person set and the provider's account of how it minimized the result.

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)): it carries one HCP operation, unchanged, so that a Context Client can use it over any transport binding, such as TSP, as well as over HCP's own HTTP and MCP Bindings.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage. [SPEC §9.4](/SPEC.md#94-carried-protocols) says a protocol's owner should publish or endorse its family, and [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications) lets them do so under their own authority. This family is a proposal to them. When HCP changes the Query operation, this specification moves to a new version.

HCP's text is © its authors under CC BY 4.0. This specification and the shared `hcp/_shared/0.1` schemas were written from that text, with attribution, and do not copy HCP's own Apache-2.0 schema files. Where they differ from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on the Query operation; this specification restates the ones that bear on the document and does not relax any. Conformance to HCP is HCP's: serving this task does not make a provider conform to an HCP binding, and HCP defines no conformance identifier for Trust Task carriage, so a provider **MUST NOT** claim this carriage in its HCP `conformsTo`, and advertises the task through [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)).

This specification adds one check HCP does not make: the token binding of [The access token](#the-access-token), a carriage rule of the `hcp/*` family, not HCP's. [SPEC §9.4](/SPEC.md#94-carried-protocols) item 6 places a check the carried protocol does not make in a separate specification that does not declare `carries`. This family keeps this one in the carriage, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)), because it protects the token that the carriage itself moves from the HTTP `Authorization` header into the document, where it can be stored or forwarded with the document; a separate task could not protect the document this one carries.

## Authorization

The authority is **the grant** the person gave the client: a category-scoped, purpose-bound and revocable HCP grant (HCP 6.1), held by the provider and evidenced by the access token in `accessToken`. The provider **MUST** check the grant on every request (HCP 6.2.2), exactly as it does over HCP's HTTP Binding: the token must be valid, and the requested `categories` and `purpose` must be within the grant (HCP 5.2.1).

Possession of a valid token is the evidence; neither the document's `proof` nor the `issuer` VID is. Verifying the proof or the VID establishes who sent the document and that it is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); it does not establish the grant. The request always carries a proof, *pending HCP confirmation* ([question 53](#pending-hcp-confirmation)), so a token issued to a client other than the one the `issuer` VID identifies is always refused with `permissionDenied` (see [The access token](#the-access-token)). A grant identifies the client it was issued to (HCP 3.11, invariant 3), so a client that changes its identity, to a new VID or to a key it cannot prove continuity with, is a new client and needs a new grant from the person; a grant is never moved to another client identity.

Query is a context operation, for HCP Clients acting under a grant (HCP 5.1). This specification is written for a Context Client: its result carries each entry as a `Disclosure` and always carries `directives` (HCP 6.4, 7.4). HCP's result also foresees a requester that is not a Context Client, which receives entries whole (HCP 5.2.1, 7.4); a provider that serves such a requester returns `Entry` objects as HCP allows it, and `directives` null. This specification reads HCP as letting a Management Client query and receive whole `Entry` objects with no directives, *pending HCP confirmation* ([question 5](#pending-hcp-confirmation)).

## Definitions

- **Context Client**, **Management Client**, **HCP Provider**, **Authorizing Principal**, **Context Subject**, **grant**, **entry**, **disclosure**, **directive** and **category** have their HCP 2.0-draft-3 meanings.
- The shared shapes are in [`hcp/_shared/0.1`](../../../_shared/0.1/common.schema.json): [`Disclosure`](../../../_shared/0.1/disclosure.schema.json), [`Entry`](../../../_shared/0.1/context-entry.schema.json) and [`Directive`](../../../_shared/0.1/directive.schema.json). They are open, as HCP's own schemas are: a member those schemas do not declare is accepted, never rejected, and a provider preserves an entry member it does not define (HCP 4.2, A.3).

### HCP field mapping

| HCP Query field | Member | Notes |
|---|---|---|
| request `query` | `query` | Unchanged. |
| request `categories` | `categories` | Unchanged. At most 256 items, a framework bound HCP does not set. |
| request `purpose` | `purpose` | Unchanged: a purpose-code absolute URI. |
| request `purposeDescription` | `purposeDescription` | Unchanged. |
| request `maxEntries` | `maxEntries` | Unchanged. Absent means HCP's default of 5. |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result `entries` | `entries` | Unchanged. |
| result `retrievalId` | `retrievalId` | Unchanged. |
| result `minimization` | `minimization` | Unchanged. |
| result `categoriesSearched` | `categoriesSearched` | Unchanged. |
| result `directives` | `directives` | Unchanged. |
| result `timestamp` | `respondedAt` | **Renamed**, following the registry's `<event>At` convention for instants. Same type and meaning. |

**Where this payload is stricter than HCP's shapes.** Each payload is closed at its top level (`additionalProperties: false`): a top-level member outside this mapping is refused, where HCP's own request and result shapes leave the top level open, so an extension member HCP would allow there is not carried. The HCP shapes nested within stay open, as HCP's are. Every size bound, and every non-empty requirement (a `minLength` or `minItems` of 1) on a member that HCP's own schema allows to be empty, is a Trust Tasks framework bound on what an `hcp/*` party writes, not an HCP requirement, as the schema descriptions say. These bounds, and each `maxDocumentBytes` the family declares, bound what a party writes and accepts; they are not delivery guarantees. A transport or mediator may cap a message lower, for example a mediator that accepts messages of at most 1 MiB, so a party **MUST NOT** assume that a document within the bound will be delivered.

### Parties

| HCP role | Trust Task member |
|---|---|
| Context Client or Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

HCP identifies a provider by the `provider` URL and `jwks_uri` in its discovery document (`/.well-known/hcp-configuration`, HCP 19.1), and a client by its registration. Neither is a VID. A provider that serves the family only over a transport binding may publish no discovery document: HCP requires one of an implementation that claims HCP conformance (HCP 19.1), whose schema lists at least one of HCP's own bindings, and the provider may not claim the carriage in its `conformsTo` ([question 1](#pending-hcp-confirmation)). This rule, which every `hcp/*` specification applies, covers both cases:

- **The anchor.** A client's trust anchor for a provider is the provider VID the client obtained when the person connected it to that provider: from the pairing or QR code the person used, or from [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) asked of a VID the client already trusts for that purpose. A client **MUST** send a token only to that VID.
- **The discovery binding.** Where the provider publishes an HCP discovery document, it **MUST** also make its VID verifiably bound to its HCP identity: its DID document lists the discovery document as a service, or its discovery document names the VID, or both. A client **MUST** check that binding before it sends a token to the VID.

## Request

The Context Client sends the query to the provider. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

- `query` (REQUIRED) is what the client wants context about.
- `categories` (OPTIONAL) narrows the search. Absent means every category the grant covers. It is never empty: HCP gives an empty list no meaning, and an empty list and an absent member would mean different queries, so the schema carries presence explicitly and refuses an empty list (`minItems: 1`), *pending HCP confirmation* ([question 42](#pending-hcp-confirmation)). That is the carriage's rule, not a library limitation; a separate Rust generator fix restores absent, empty and null fidelity in generated types for every family, and neither depends on the other.
- `purpose` (REQUIRED) is the purpose code the context is for; `purposeDescription` (OPTIONAL) says it in words.
- `maxEntries` (OPTIONAL) is the most entries the client wants. HCP's default is 5 and its ceiling is 20 (HCP 7.2); those are HCP's values, restated.
- `accessToken` (REQUIRED) is the HCP access token for the grant.

### A query for dietary preferences, for a booking

```json
{
  "id": "urn:uuid:0d4f7a52-3c1e-4b8e-9a6f-2e1b7c9d0a11",
  "type": "https://trusttasks.org/spec/hcp/entries/query/0.1#request",
  "issuer": "did:web:booking-agent.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-24T10:00:00Z",
  "threadId": "urn:uuid:7c2a9e41-5b3d-4f6a-8e1c-0a9b8d7c6e51",
  "payload": {
    "query": "dietary preferences and allergies for a restaurant booking",
    "categories": ["general_preferences", "health_context"],
    "purpose": "https://hcp.example/purposes/reservation",
    "purposeDescription": "Book a table that can serve the guest's diet.",
    "maxEntries": 3,
    "accessToken": "hcp_at_2b7f0c6e9d1a4e8b"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-24T10:00:00Z",
    "verificationMethod": "did:web:booking-agent.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "zR7FjY6BFx2GpqpM3AbnuVz15bVdYyYHy1hswAuaQS1JFK4ZonnmHLommsePUWev2gMnBT96kmcf8T64trLapb1"
  }
}
```

## Response

The provider answers with the disclosed context. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json).

- `entries` holds at most the lesser of `maxEntries` and 20 entries (HCP 7.2), in relevance order (HCP 7.3). `maxEntries` is a ceiling, not a target: the provider never pads the result to reach it (HCP 7.2). Each entry is a `Disclosure`. It carries a grant-scoped `ref` instead of the entry's stable `id`, the `version` disclosed, and the entry's `standing`. It withholds the entry's `id`, `subject` and `record` (HCP 7.4). Its `useUntil` is the time after which the client **MUST NOT** use the copy or pass it on (HCP 7.6). An `Entry` appears only where the requester is not a Context Client (HCP 5.2.1, 7.4). An empty `entries` is a successful answer: nothing matched, or nothing could be disclosed.
- `retrievalId` identifies this retrieval in the provider's audit log.
- `minimization` is the provider's minimization report (HCP 7.5). Its `method` is the relevance method that determined the final ordering, which the provider declares in its discovery document (HCP 7.3, 19.3): `lexical`, `semantic` or `llm` among the registered values. It never carries a count of what was considered or withheld (HCP 7.5).
- `categoriesSearched` lists the categories searched: those the request named or, where it named none, every category the grant can read for the purpose (HCP 5.2.1).
- `directives` are the person's directives that apply to the client and to the disclosed context (HCP 6.4). Every result to a Context Client carries it, empty when no directive applies (HCP 6.4); a result to a requester that is not a Context Client carries it as null ([question 5](#pending-hcp-confirmation)). HCP's own shape leaves the member optional; the carriage requires it, a list or null, so that "none applies" is never confused with "not a Context Client", *pending HCP confirmation* ([question 42](#pending-hcp-confirmation)). A client that claims to follow directives **MUST** follow every directive delivered to it, and **MUST NOT** treat one whose `action` it does not recognize as a permission (HCP 6.4).
- `respondedAt` is HCP's `timestamp`, renamed.

Failures use `trust-task-error`, not a `#response` document (see [Errors](#errors)).

### Two entries disclosed under the grant

```json
{
  "id": "urn:uuid:3e8d1b62-9f4a-4c7e-b2d0-6a5c4e3f2b12",
  "type": "https://trusttasks.org/spec/hcp/entries/query/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:booking-agent.example",
  "issuedAt": "2026-11-24T10:00:01Z",
  "threadId": "urn:uuid:7c2a9e41-5b3d-4f6a-8e1c-0a9b8d7c6e51",
  "payload": {
    "entries": [
      {
        "ref": "hcp-ref-a91c4f",
        "version": 2,
        "content": {
          "profile": "https://hcp.example/spec/profiles/general-context/2.0",
          "value": "Vegetarian; eats fish occasionally.",
          "language": "en"
        },
        "assertion": { "declarationType": "person-declared" },
        "classification": { "categories": ["general_preferences"] },
        "standing": { "assurance": "authenticated", "confirmed": true },
        "useUntil": "2026-11-25T10:00:01Z"
      },
      {
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
        "useUntil": "2026-11-24T22:00:01Z"
      }
    ],
    "retrievalId": "urn:uuid:9a1e5c73-2d4b-4e8f-a6c1-7b0d3e2f1a99",
    "minimization": { "method": "lexical" },
    "categoriesSearched": ["general_preferences", "health_context"],
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
    "respondedAt": "2026-11-24T10:00:01Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-24T10:00:01Z",
    "verificationMethod": "did:web:context.provider.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z3FXQjecWufY46yg5abdVZsXqLhxhueuSoZgNSARiKBk9czhSePTFehP8c3PGfb6a22gkfUKKPWtsuNYYHNXS5d2"
  }
}
```

### A revoked grant

The token's grant was revoked, so the provider refuses the query with `hcp:invalidToken`, *pending HCP confirmation* ([question 2](#pending-hcp-confirmation)). The client treats this as notice of the revocation (HCP 6.2.2).

```json
{
  "id": "urn:uuid:4f9e2c73-0a5b-4d8f-93e1-7b6d5f4e3c23",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:booking-agent.example",
  "issuedAt": "2026-11-24T10:05:00Z",
  "threadId": "urn:uuid:7c2a9e41-5b3d-4f6a-8e1c-0a9b8d7c6e51",
  "payload": {
    "code": "hcp:invalidToken",
    "message": "The access token is not valid.",
    "retryable": false
  }
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows. The provider uses the code in the right-hand column. It never sends HCP's Core error object or the HTTP Binding's problem details (HCP 9.7).

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request` | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: the request falls outside the grant (a category or purpose the grant does not cover, HCP 5.2.1), or the entry does not exist | `permissionDenied`, with one fixed `message` whatever the reason. HCP requires that the response not disclose which (HCP 5.5, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)). |
| conflict | `idempotency-conflict` | Does not arise: HCP defines it for a retried request with an idempotency key (HCP 9.6), and Query carries none. A re-delivered document is handled under [Repeated requests](#repeated-requests). |
| version conflict | `version-conflict` | Does not arise: Query names no entry version (HCP 5.2.4, 5.4). |
| credential invalid | `credential-invalid` | Does not arise: HCP raises it when a credential written as an entry's content fails verification (HCP 14.6), which a read does not do. |
| subject unavailable | `subject-unavailable` | `unavailable`. The client may retry later (HCP 5.5). |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion`. The provider never serves a different HCP version silently (HCP 5.5). |

A client that does not know `hcp:invalidToken` reads it as `taskFailed` ([SPEC §8.5](/SPEC.md#85-extension-by-individual-trust-task-specifications)) and still honours `retryable: false`.

## Repeated requests

HCP requires a `context.read` audit event for every query (HCP 8.1). Because this task is consequential, a consumer applies [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements): a re-delivered document with the same `id` and content is not executed twice, so it produces no second disclosure and no second `context.read` event. This is how carriage reads HCP 8.1. HCP's own idempotency rule (HCP 9.6) is a MUST for create and revise and a SHOULD for withdraw, and says nothing of Query. That a re-delivered query causes no second `context.read` event is *pending HCP confirmation* ([question 4](#pending-hcp-confirmation)). A provider **MUST** retain its duplicate-execution record for at least the acceptance window it applies to `issuedAt`.

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of [the list of questions put to the editors](../../../_shared/0.1/QUESTIONS.md).

- **Question 1 — conformance claims.** HCP defines no conformance identifier for Trust Task carriage, so a provider **MUST NOT** claim this carriage in its HCP `conformsTo`; it advertises the task with `trust-task-discovery` instead. A conformance identifier for the carriage is to be requested from the HCP editors (see [Conformance](#conformance)).
- **Question 2 — a request under a revoked grant.** Refused with `hcp:invalidToken`: HCP 5.5 defines `invalid-token` as invalid authorization evidence, and HCP 6.2.2 names no error (see [Errors](#errors)).
- **Question 4 — a re-delivered query.** It causes no second `context.read` audit event (HCP 8.1, 9.6; see [Repeated requests](#repeated-requests)).
- **Question 5 — a Management Client as requester.** A Management Client may query and receive whole `Entry` objects, with `directives` null (HCP 5.1, 5.2.1, 7.4; see [Authorization](#authorization)).
- **Question 22 — the client's identity.** The `issuer` VID, authenticated by the request's `proof`, identifies the HCP Client the token was issued to; a token issued to any other client **MUST** be refused with `permissionDenied`. This token binding is a carriage rule of this family, a check HCP does not itself make (see [The access token](#the-access-token) and [Conformance](#conformance)).
- **Question 42 — presence signals the carriage adds.** The result's `directives` is required: a list, empty when none applies, for a Context Client, and null for any other requester; and a request's `categories`, where present, is never empty. HCP's own shapes leave `directives` optional and accept an empty `categories` (HCP 5.2.1, 6.4).
- **Question 53 — a request proof on every request that carries a token.** A request `proof` is **REQUIRED** on every `hcp/*` request that carries `accessToken`, so that the token binding of question 22 always applies and a token lifted from a stored or forwarded document cannot be replayed in an unsigned request. This is a carriage rule of this family, not HCP's, and is stricter than HCP's bearer model, under which presenting a valid token is enough (HCP 9.3); see [The access token](#the-access-token).

## What HCP requires that a document cannot express

- **The grant check on every request** (HCP 6.2.2), and the scope checks of HCP 5.2.1: they are made against provider-held state.
- **Minimization** (HCP 7.1–7.3): the provider decides which entries to disclose and in what order; the document only carries the result.
- **Use limits** (HCP 7.6): `useUntil` is carried, but the client's duty to stop using a copy after it, and not to pass copies on (HCP 3.12), is on the client.
- **Directives** (HCP 6.4): carried with every result to a Context Client; following them is on a client that claims to.
- **Health context** (HCP 16.2). A copy disclosed from `health_context`, a High-sensitivity category, is usable for at most 24 hours (HCP 7.6). A provider conforming to the Health Context Profile (HCP 16.1, 18.1) does not disclose a `model-inferred` entry in `health_context` to a Context Client unless the most recent confirmation record on its current version is `confirmed`: until then a query does not return it.
- **Grant-scoped `ref`s** (HCP 7.4): minted by the provider. A disclosure carries no subject identifier at all (HCP 7.4); the per-client pseudonym is in the grant (HCP 3.4).
- **Cryptosuite.** An HCP implementation **MUST** verify `eddsa-rdfc-2022` and **SHOULD** verify `ecdsa-sd-2023` for credentials that are an entry's content (HCP 14.5). That concerns credentials inside entries, not this document's `proof`, which follows [SPEC §4.7.1](/SPEC.md#471-cryptosuites).

## Security & Privacy

### Data carried

The request carries the access token, a bearer secret, and the query text, which can itself reveal something about the person. The response carries the person's context: up to 20 disclosures that may include High-sensitivity categories such as `health_context`, and credentials as evidence. A producer **MUST NOT** put context, query text or the token in `ext` or in any `message`. The smallest answer is an empty `entries`; the provider, not the client, decides how much to disclose, and a client asks for no more than its purpose needs, through `categories` and `maxEntries`.

Because the response proof is REQUIRED ([SPEC §7.3 item 8](/SPEC.md#73-specification-requirements)), each disclosure reaches the client as a statement signed by the provider, which anyone holding the provider's public key can check came from it. The proof is made for `authentication`, not as an attestation ([SPEC §4.7.3](/SPEC.md#473-proof-purpose-and-verification-relationship)). That makes the copy more valuable to leak or resell. HCP bars onward passing (HCP 3.12, 6.2.2), and a client **MUST NOT** pass the signed response, or any part of it, to anyone the grant does not cover.

### The access token

`accessToken` is a bearer secret: whoever presents it exercises the authority it evidences. It **MUST** travel only to the provider. A recipient **MUST NOT** log `accessToken` or retain it beyond the exchange.

**In an agent runtime** (HCP 10.5). HCP states for its MCP Binding that the grant is held by the component that presents credentials, never by the model; that the access token **MUST NOT** enter the model's context and **MUST NOT** be returned as tool content (HCP 3.11, invariant 14); and that where the client claims to follow directives, the host applies them, since placing a directive in the model's context is not following it. The same holds wherever an agent runtime carries `hcp/*`, whatever its binding. The token, whether the client presents it or receives it in a response, is held by the component that builds and sends the document, and **MUST NOT** enter a model's context or be returned as tool content or in a tool's result. Where the client claims to follow directives, the host applies them (HCP 6.4, 10.5). Every `hcp/*` specification applies this rule.

**Token binding, a carriage rule of this family.** Every request that carries `accessToken` carries a `proof`, *pending HCP confirmation* ([question 53](#pending-hcp-confirmation)), and the provider **MUST** refuse with `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies, so that a token lifted from a stored or forwarded document is of no use from another VID. HCP validates the token alone and does not bind it to a VID (HCP 9.3): this rule is the `hcp/*` family's, not HCP's, and rests on the `issuer` VID identifying the HCP Client, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)). It is a check HCP does not make (see [Conformance](#conformance)).

**Retained copies.** The provider verifies the request's `proof` over the document as received, at receipt, before it removes anything ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). A provider that retains the request document, as a duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) or as evidence of the request, **MUST** remove `accessToken` from the copy it keeps and record in its place `accessTokenDigest`: the SHA-256 multihash of the token's UTF-8 bytes, multibase-encoded as base64url without padding (prefix `u`), the framework's `DigestMultibase` form ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)). The kept copy's `proof` no longer verifies over it, so the provider records with the copy that the proof was verified at receipt: when, under which verification method, and with what result. It relies on that record, and **MUST NOT** verify the proof again over the kept copy. A record that keeps only a digest of the document, such as its task digest ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)), keeps no token either.

### Correlation

The provider sees which client queried, when, for which purpose and categories; that is inherent in a grant check and is what the audit log records. A disclosure carries no subject identifier, and names each entry only by a grant-scoped `ref` (HCP 7.4); the subject identifier the client does learn, in its grant, is a pseudonym specific to that client (HCP 3.4). So two clients cannot join their copies on an identifier HCP mints. A credential disclosed whole as evidence may still carry identifiers of its own (HCP 7.4). The access token is stable for the life of the grant and joins every query made under it, which is one more reason it **MUST** travel only to the provider (see [The access token](#the-access-token)). `threadId` joins a request to its response and **SHOULD** be fresh per query. Response size can reveal how much context matched; a provider **MAY** pad the response.

### Retention

The provider keeps the `context.read` audit event (HCP 8.1) for as long as HCP and its own policy require, readable only by the Context Subject (HCP 8.3). HCP's audit event records the `purpose` and `purposeDescription` but has no member for the query text (HCP 8.2). The client stops using each disclosed copy at its `useUntil` (HCP 7.6). When the grant is revoked, it stops using every copy disclosed under the grant at once, whatever its `useUntil`, and passes none of them on (HCP 6.2.2). The one exception is a copy kept under a legal obligation the client declared at registration (HCP 6.2.5). HCP does not require deletion, and the provider cannot enforce it (HCP 6.2.2). The signed response is evidence of what the provider released; a client that keeps it as evidence keeps it apart from the context it carries, which still expires.

### Consent/purpose

The context is disclosed for the `purpose` named in the request, which must be one the grant allows, and for no other. The person sets the grant and the directives; this task only carries a use of them. Whether a given query needs any further approval is the provider's policy under HCP, not a requirement of this specification ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).

The **acceptance window** for `issuedAt` is consumer policy; a provider **SHOULD** apply a short one, minutes rather than hours, since a query answers the present.
