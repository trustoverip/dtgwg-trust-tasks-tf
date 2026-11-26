---
slug: hcp/entries/revise
version: "0.1"
title: "HCP — Revise Entry"
summary: An HCP Client writes a new version of an entry at a Human Context Protocol provider, provided the version it names is still current; carries the HCP 2.0-draft-3 Revise operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Revise
category: data-exchange
keywords: [hcp, human-context-protocol, personal-context, memory, entry, revision, version-conflict]
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
    The request is the client's attributable statement about a person, which the provider records as the new
    version's provenance (HCP 3.5, 5.2.4), so it is signed. The response discloses the revised entry, so it carries the
    provider's signature.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task changes the person's context and is consequential; a re-delivered revision must be placeable in the
    provider's duplicate-execution window, so that it is absorbed instead of failing with a version conflict.
sideEffects:
  level: mutating
  rationale: >-
    A new version of the entry is stored and the provider appends a `context.revised` audit event (HCP 8.1). The prior
    version is never deleted; it is versioned (HCP 5.1).
exposure:
  discloses: secret
  actsAsSubject: false
  ingests: secret
  rationale: >-
    The request carries a revised assertion about a person, which may be in a High-sensitivity category or be a
    credential, and the HCP access token, a bearer secret. The response discloses the revised entry back to the client.
retention:
  class: durable
  rationale: >-
    The provider keeps every version until the entry is hard-deleted, and the `context.revised` audit event with the
    `reason` (HCP 5.2.4, 8.1).
    The access token is not retained: a kept request holds its digest instead (see Security & Privacy, The access token).
maxDocumentBytes:
  request: 524288
  response: 524288
  rationale: >-
    One entry version, which may carry a verifiable credential and supporting credentials as evidence, plus envelope
    and proof. This is a framework bound, not an HCP requirement; HCP sets no size limit.
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
  - code: hcp:versionConflict
    meaning: >-
      HCP `version-conflict` (HCP 5.5): `expectedVersion` is not the entry's current `record.version` (HCP 5.2.4,
      5.4). Nothing changes, and the same document never succeeds: the client reads the entry again, and a Management
      Client shows the person the current version, before deciding whether to ask again. Defined once for the family
      in [the hcp family conventions](/specs/hcp/_shared/0.1/CONVENTIONS.md#1-family-error-codes).
    retryable: false
  - code: hcp:credentialInvalid
    meaning: >-
      HCP `credential-invalid` (HCP 5.5): a credential that is an entry's content, as created or revised, fails
      verification, in its proof, its securing mechanism, its verification method or its status (HCP 14.6); or, for
      an `issuer-attested` entry in `health_context`, no credential the request carries, content or supporting, passes
      verification (HCP 16.2). The `message` says which check failed, and reveals no status-list internals. Defined
      once for the family in [the hcp family conventions](/specs/hcp/_shared/0.1/CONVENTIONS.md#1-family-error-codes).
    retryable: false
uncheckedRules:
  - member: "/properties/ref"
    rule: "Exactly one of `ref` and `entryId` is present. A request with both or neither is refused with `malformedRequest` (pending HCP confirmation, question 6)."
    source: "HCP 5.2.4"
  - member: "/properties/content"
    rule: "Under the General Context Profile, `content` carries `value`."
    source: "HCP 4.2, 11.2"
  - member: "/properties/content"
    rule: "Under the VC Evidence Profile, `content` carries `evidence` and no `value`."
    source: "HCP 4.2.2, 14"
  - member: "/properties/evidence"
    rule: "At most one evidence item has `role` `content`."
    source: "HCP 4.2.2, 5.2.4"
  - member: "/properties/evidence/items/properties/credential"
    rule: "An evidence item whose `type` is `verifiable-credential` carries `credential`, an object with `@context`, `type`, `issuer` and `credentialSubject`."
    source: "HCP 4.2.2"
  - member: "/properties/assertion/properties/confidence"
    rule: "`confidence` is non-null when the revised `declarationType` is `model-inferred`, and null or absent otherwise."
    source: "HCP 4.2, 5.2.3, 5.2.4"
  - member: "/properties/record/properties/provenance/properties/assertedBy"
    rule: "`assertedBy.id` is present unless the revised declaration type is `person-declared` or the content is a credential, as for a create."
    source: "HCP 5.2.3, 5.2.4"
  - member: "/properties/record/properties/relationships/items/properties/proposal"
    rule: "Only a `contradicts` relationship carries a `proposal`."
    source: "HCP 4.7.4"
  - member: "/$defs/Response/properties/entry/anyOf/0/properties/content"
    rule: "Under the General Context Profile, `content` carries `value`."
    source: "HCP 4.2, 11.2"
  - member: "/$defs/Response/properties/entry/anyOf/0/properties/content"
    rule: "Under the VC Evidence Profile, `content` carries `evidence` and no `value`."
    source: "HCP 4.2.2, 14"
  - member: "/$defs/Response/properties/entry/anyOf/0/properties/evidence"
    rule: "At most one disclosed evidence item has `role` `content`."
    source: "HCP 4.2.2, 7.4"
  - member: "/$defs/Response/properties/entry/anyOf/0/properties/evidence/items/allOf/0/properties/credential"
    rule: "An evidence item whose `type` is `verifiable-credential` carries `credential`, an object with `@context`, `type`, `issuer` and `credentialSubject`."
    source: "HCP 4.2.2"
  - member: "/$defs/Response/properties/entry/anyOf/0/properties/assertion/properties/confidence"
    rule: "`confidence` is non-null when `declarationType` is `model-inferred`, and null or absent otherwise."
    source: "HCP 4.2, 4.4"
  - member: "/$defs/Response/properties/entry/anyOf/1/properties/type"
    rule: "An entry's `type` array includes `ContextEntry`."
    source: "HCP 4.2"
  - member: "/$defs/Response/properties/entry/anyOf/1/properties/content"
    rule: "Under the General Context Profile, `content` carries `value`."
    source: "HCP 4.2, 11.2"
  - member: "/$defs/Response/properties/entry/anyOf/1/properties/content"
    rule: "Under the VC Evidence Profile, `content` carries `evidence` and no `value`."
    source: "HCP 4.2.2, 14"
  - member: "/$defs/Response/properties/entry/anyOf/1/properties/assertion/properties/confidence"
    rule: "`confidence` is non-null when `declarationType` is `model-inferred`, and null or absent otherwise."
    source: "HCP 4.2, 4.4"
  - member: "/$defs/Response/properties/entry/anyOf/1/properties/evidence"
    rule: "At most one evidence item has `role` `content`."
    source: "HCP 4.2.2"
  - member: "/$defs/Response/properties/entry/anyOf/1/properties/evidence/items/properties/credential"
    rule: "An evidence item whose `type` is `verifiable-credential` carries `credential`, an object with `@context`, `type`, `issuer` and `credentialSubject`."
    source: "HCP 4.2.2"
  - member: "/$defs/Response/properties/entry/anyOf/1/properties/record/properties/relationships/items/properties/proposal"
    rule: "Only a `contradicts` relationship carries a `proposal`."
    source: "HCP 4.7.4"
related:
  - hcp/entries/create
  - hcp/entries/get
  - hcp/entries/withdraw
---

## Abstract

The **HCP — Revise Entry** Trust Task carries the Revise operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 5.2.4; `POST /v2/entries/{entryId}/revisions` in HCP 9.2). An *HCP Client* writes a new version of an entry, replacing its content, assertion and evidence whole. The revision names the version it replaces; where that is no longer the current version, it fails and changes nothing, so a revision is never lost to a concurrent one (HCP 5.2.4).

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and does not copy HCP's own schema files. Where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on the Revise operation; this specification restates the ones that bear on the document and does not relax any. A provider **MUST NOT** claim this carriage in its HCP `conformsTo`, and advertises the task through [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)).

This specification adds one check HCP does not make: the token binding of [The access token](#the-access-token), a carriage rule of the `hcp/*` family, not HCP's. [SPEC §9.4](/SPEC.md#94-carried-protocols) item 6 places a check the carried protocol does not make in a separate specification that does not declare `carries`. This family keeps this one in the carriage, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)), because it protects the token that the carriage itself moves from the HTTP `Authorization` header into the document, where it can be stored or forwarded with the document; a separate task could not protect the document this one carries.

## Authorization

The authority is **the grant** (HCP 6.1), evidenced by the access token in `accessToken`, which the provider checks on every request (HCP 6.2.2). The grant **MUST** let the caller write the entry's categories, and a Context Client names only an entry disclosed to it under the same grant (HCP 5.2.2, 5.2.4). Anything else is refused with `permissionDenied`, with one fixed `message`, so that entry existence is not disclosed (HCP 5.5). A Context Client that names the entry by `entryId` is refused the same way, *pending HCP confirmation* ([question 6](#pending-hcp-confirmation)).

Only an issuer revises an `issuer-attested` entry (HCP 3.7, 4.2.6). A Context Client **MUST NOT** revise an entry whose current version the Context Subject has rejected (HCP 3.8, 5.2.4); a provider refuses such a revision with `permissionDenied`, *pending HCP confirmation* ([question 9](#pending-hcp-confirmation)).

As for [`hcp/entries/create`](../../create/0.1/spec.md#authorization), the REQUIRED `proof` is not the authorization, and does not replace the provenance the provider records for the new version (HCP 3.5).

A token issued to a client other than the one the `issuer` VID identifies is refused with `permissionDenied` (see [The access token](#the-access-token)).

## Definitions

- **Context Client**, **Management Client**, **HCP Provider**, **Context Subject**, **Context Asserter**, **grant**, **entry** and **version** have their HCP 2.0-draft-3 meanings.
- The shared shapes are in [`hcp/_shared/0.1`](../../../_shared/0.1/common.schema.json): `RevisionAssertion` and `WriteRecord` in [`entry-write.schema.json`](../../../_shared/0.1/entry-write.schema.json); `Content`, `EvidenceItem`, `Disclosure` and `Entry` as for create.

### HCP field mapping

| HCP Revise field | Member | Notes |
|---|---|---|
| request `ref` or `entryId` (the `{entryId}` path segment, HCP 9.4.4) | `ref` or `entryId` | Moved from the path into the payload. Exactly one is present. |
| request `expectedVersion`, `content`, `evidence`, `reason` | the same | Unchanged. |
| request `assertion.*` | `assertion` | Unchanged. Includes `basis` and `permittedPurposes`, which HCP's text gives a revision ("as for create") but its published request schema omits (HCP 5.2.4); see [question 8](#pending-hcp-confirmation). |
| request `record.provenance.*`, `record.relationships` | `record` | Unchanged. A relationship may carry a `proposal`, as for create (HCP 4.7.4, 5.2.4). |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `Idempotency-Key` header (HCP 9.6) | the document `id` | See [Repeated requests](#repeated-requests). |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result: the revised entry (`201`, HCP 9.4.4) | `entry` | **Wrapped**, as in `hcp/entries/create`. |

The request payload's root is closed, as in `hcp/entries/create`: a top-level member that HCP does not define cannot be sent through this task, although HCP's revise request schema admits one (HCP A.2). The nested `content`, `assertion` and `record` stay open. Every size bound, and every non-empty requirement (a `minLength` or `minItems` of 1) on a member that HCP's own schema allows to be empty, is a Trust Tasks framework bound on what an `hcp/*` party writes, not an HCP requirement, as the schema descriptions say.

### Parties

| HCP role | Trust Task member |
|---|---|
| Context Client or Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../query/0.1/spec.md#parties), a client **MUST** send a token only to the provider VID it obtained when the person connected it, and where the provider publishes an HCP discovery document (HCP 19.1), the provider **MUST** make its VID verifiably bound to it and the client **MUST** check that binding before it sends a token to the VID.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

- `ref` (a Context Client) or `entryId` (a Management Client) names the entry. Exactly one is present; both or neither is refused with `malformedRequest`, *pending HCP confirmation* ([question 6](#pending-hcp-confirmation)). The schema does not check this.
- `expectedVersion` (REQUIRED) is the version the client read, which the revision replaces.
- `content` (REQUIRED) is the revised content, whole; `evidence` (OPTIONAL) the revision's evidence, whole, absent meaning none.
- `reason` (REQUIRED) is recorded in the audit log.
- `assertion` (OPTIONAL): `declarationType` omitted means unchanged; `confidence`, `validUntil`, `basis` and `permittedPurposes` are as for create, and the `confidence` rule of HCP 5.2.3 applies to the revised declaration type. The schema does not check it.
- `record` (OPTIONAL): provenance for the revised assertion, attributed to its own Context Asserter, and relationships, as for create (HCP 5.2.4).
- `subject` and `classification` are not revision members: a revision never changes them (HCP 4.2.1, 5.2.4), and the schema refuses them.
- `accessToken` (REQUIRED) is the HCP access token for the grant.

### The booking agent corrects the preference

```json
{
  "id": "urn:uuid:4c8e1a25-9d3b-4f7a-a6e2-1b5d0c9f8e57",
  "type": "https://trusttasks.org/spec/hcp/entries/revise/0.1#request",
  "issuer": "did:web:booking-agent.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T11:00:00Z",
  "threadId": "urn:uuid:6f2b9d14-3a8c-4e5f-b7d0-9c1e4a2b3f68",
  "payload": {
    "ref": "hcp-ref-c73e19",
    "expectedVersion": 1,
    "content": {
      "profile": "https://hcp.example/spec/profiles/general-context/2.0",
      "value": "Prefers a window table, away from the kitchen.",
      "language": "en"
    },
    "reason": "The person added a detail when confirming a booking.",
    "assertion": { "declarationType": "person-declared" },
    "record": { "provenance": { "source": "person-input" } },
    "accessToken": "hcp_at_2b7f0c6e9d1a4e8b"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T11:00:00Z",
    "verificationMethod": "did:web:booking-agent.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z3KpS7wYr1XnM5vTc9LbG2qHj8dAf6uEz4oRiN3xW7tC1kV9mB5yP2gD8hJ6lO4aU1cF3nQ7sZ5eX9vR2bT8wK6m"
  }
}
```

## Response

The provider answers with the revised entry. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): `entry`, with its version incremented by 1, as a `Disclosure` for a Context Client (HCP 7.4), otherwise the whole `Entry`. The entry's `id`, and its `ref` under the grant, are unchanged (HCP 5.2.4). The new version starts with no confirmations (HCP 3.8). Failures use `trust-task-error` (see [Errors](#errors)).

### Version 2

```json
{
  "id": "urn:uuid:8e3a6c49-1f7d-4b2e-9a5c-0d8b3f6e1a79",
  "type": "https://trusttasks.org/spec/hcp/entries/revise/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:booking-agent.example",
  "issuedAt": "2026-11-25T11:00:01Z",
  "threadId": "urn:uuid:6f2b9d14-3a8c-4e5f-b7d0-9c1e4a2b3f68",
  "payload": {
    "entry": {
      "ref": "hcp-ref-c73e19",
      "version": 2,
      "content": {
        "profile": "https://hcp.example/spec/profiles/general-context/2.0",
        "value": "Prefers a window table, away from the kitchen.",
        "language": "en"
      },
      "assertion": { "declarationType": "person-declared" },
      "classification": { "categories": ["general_preferences"], "dimension": "seating" },
      "standing": { "assurance": "authenticated", "confirmed": false },
      "useUntil": "2026-11-26T11:00:01Z"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T11:00:01Z",
    "verificationMethod": "did:web:context.provider.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z6NqT2xWs8YmL4vRc1KbH9pGj3dZf7uAe5oSiM2xV6tD9kB1rN4yQ8gE3hJ7lP5aU2cW6nO1sF9eZ4vX8bR3mT7k"
  }
}
```

### Someone else revised it first

```json
{
  "id": "urn:uuid:2d7f4b38-6e1a-4c9d-8b3f-5a0e2c7d4b81",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:booking-agent.example",
  "issuedAt": "2026-11-25T11:00:01Z",
  "threadId": "urn:uuid:6f2b9d14-3a8c-4e5f-b7d0-9c1e4a2b3f68",
  "payload": {
    "code": "hcp:versionConflict",
    "message": "The entry has a newer version than the one named.",
    "retryable": false
  }
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows. The provider never sends HCP's Core error object or the HTTP Binding's problem details (HCP 9.7).

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request`: schema validation, or a `confidence` that does not match the declaration type (HCP 5.2.3) | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: an entry that does not exist, is outside the caller's write scope, whose `ref` belongs to another grant, or that the caller may not revise (HCP 3.7, 5.2.4, 5.5) | `permissionDenied`, with one fixed `message` whatever the reason (HCP 5.5, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| conflict | `idempotency-conflict` (HCP 9.6) | `idConflict` (see [Repeated requests](#repeated-requests)) |
| version conflict | `version-conflict`: `expectedVersion` is not the current version (HCP 5.2.4) | `hcp:versionConflict`, `retryable: false`. Nothing is changed. The provider checks authorization first, so a caller that may not see the entry gets `permissionDenied`, never this. |
| credential invalid | `credential-invalid`: the revised content is a credential that fails verification (HCP 14.6), or the entry is `issuer-attested` in `health_context` and no credential the revision carries, content or supporting, passes (HCP 16.2) | `hcp:credentialInvalid`, `retryable: false`, with a `message` that names the failed check and reveals no status-list internals (HCP 5.5) |
| subject unavailable | `subject-unavailable` (HCP 4.8) | `unavailable`. The client may retry later (HCP 5.5). |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion` |

A client that does not know an `hcp:` code reads it as `taskFailed` ([SPEC §8.5](/SPEC.md#85-extension-by-individual-trust-task-specifications)) and still honours `retryable: false`.

## Repeated requests

As for [`hcp/entries/create`](../../create/0.1/spec.md#repeated-requests), the document `id` is HCP's idempotency key (HCP 9.6). A re-delivered document with the same `id` and content is answered with the original response, including the original `version`, and appends no second `context.revised` event; a reused `id` with different content is refused with `idConflict`; and the provider retains the record, with the response, for at least 24 hours. This matters most for a revision: without it, a retried revision whose first attempt succeeded fails with a version conflict, and the client cannot tell its own success from someone else's revision (HCP 9.6).

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of [the list of questions put to the editors](../../../_shared/0.1/QUESTIONS.md).

- **Question 1 — conformance claims.** A provider **MUST NOT** claim this carriage in its HCP `conformsTo` (see [Conformance](#conformance)).
- **Question 2 — a request under a revoked grant.** Refused with `hcp:invalidToken` (see [Errors](#errors)).
- **Question 6 — naming the entry.** Exactly one of `ref` and `entryId`; both or neither is `malformedRequest`, and a Context Client that sends `entryId` gets `permissionDenied` (see [Request](#request)).
- **Question 8 — `basis`, `permittedPurposes` and `proposal` on a revision.** Carried, as HCP's text says ("as for create", HCP 5.2.4, 4.7.4), although HCP's published revise request schema omits them (see [HCP field mapping](#hcp-field-mapping)).
- **Question 9 — revising a rejected version.** A Context Client's revision of an entry whose current version the Context Subject rejected is refused with `permissionDenied` (see [Authorization](#authorization)).
- **Question 22 — the client's identity.** The `issuer` VID, authenticated by the request's `proof`, identifies the HCP Client the token was issued to; a token issued to any other client **MUST** be refused with `permissionDenied`. This token binding is a carriage rule of this family, a check HCP does not itself make (see [The access token](#the-access-token) and [Conformance](#conformance)).
- **Question 53 — a request proof on every request that carries a token.** A request `proof` is **REQUIRED** on every `hcp/*` request that carries `accessToken`, so that the token binding of question 22 always applies and a token lifted from a stored or forwarded document cannot be replayed in an unsigned request. This is a carriage rule of this family, not HCP's, and is stricter than HCP's bearer model, under which presenting a valid token is enough (HCP 9.3); see [The access token](#the-access-token).

## What HCP requires that a document cannot express

- **The current-version check** (HCP 5.2.4): made against provider-held state, atomically with the write.
- **The grant, write-scope and issuer checks** (HCP 3.7, 5.2.4), and the rejected-version rule (HCP 3.8).
- **Provenance of the new version** (HCP 3.5), attributed to its own Context Asserter, and its empty confirmations (HCP 3.8).
- **Credential ingest** (HCP 14.6) for a revised credential, and for a revised `issuer-attested` entry in `health_context` the Health Context Profile's rule that it carries a credential at least one of which passes verification, as for create (HCP 16.2).
- **Conflict detection on the revision** (HCP 15.2, 15.3). A provider conforming to the Memory Manager Profile (HCP 15.1, 18.1) evaluates detection on a revision as on every write; where the revision means its method no longer finds a conflict the provider asserted, it removes that `contradicts` relationship, and the resolution rule of HCP 4.7.3 applies.

## Security & Privacy

### Data carried

As for [`hcp/entries/create`](../../create/0.1/spec.md#data-carried): a revised assertion about the person, possibly a credential, and the access token. The `reason` is recorded in the audit log and the person reads it there; a client **SHOULD** write it for the person and put no more context in it than the revision needs. A `hcp:versionConflict` `message` says only that the version is not current, never what the current version is.

### The access token

`accessToken` is a bearer secret: whoever presents it exercises the authority it evidences. It **MUST** travel only to the provider. A recipient **MUST NOT** log `accessToken` or retain it beyond the exchange.

**In an agent runtime**, the token is held as [`hcp/entries/query`](../../query/0.1/spec.md#the-access-token) requires: by the component that builds and sends the document, never in a model's context or a tool's result (HCP 10.5).

**Token binding, a carriage rule of this family.** Every request that carries `accessToken` carries a `proof`, *pending HCP confirmation* ([question 53](#pending-hcp-confirmation)), and the provider **MUST** refuse with `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies, so that a token lifted from a stored or forwarded document is of no use from another VID. HCP validates the token alone and does not bind it to a VID (HCP 9.3): this rule is the `hcp/*` family's, not HCP's, and rests on the `issuer` VID identifying the HCP Client, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)). It is a check HCP does not make (see [Conformance](#conformance)).

**Retained copies.** The provider verifies the request's `proof` over the document as received, at receipt, before it removes anything ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). A provider that retains the request document, as a duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) or as evidence of the request, **MUST** remove `accessToken` from the copy it keeps and record in its place `accessTokenDigest`: the SHA-256 multihash of the token's UTF-8 bytes, multibase-encoded as base64url without padding (prefix `u`), the framework's `DigestMultibase` form ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)). The kept copy's `proof` no longer verifies over it, so the provider records with the copy that the proof was verified at receipt: when, under which verification method, and with what result. It relies on that record, and **MUST NOT** verify the proof again over the kept copy. A record that keeps only a digest of the document, such as its task digest ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)), keeps no token either.

### Correlation

As for create. The entry's `ref` is unchanged by a revision (HCP 5.2.4), so the client's copies of the two versions join on it by design. `threadId` **SHOULD** be fresh per revision.

### Retention

The provider keeps every version until the entry is hard-deleted, and the `context.revised` audit event with its `reason` (HCP 8.1). The client's earlier copy stays subject to its own `useUntil` (HCP 7.6).

### Consent/purpose

As for create: the grant's write scope is the person's permission, and the person can confirm or reject the new version through a Management Client (HCP 3.8). Whether a given revision needs any further approval is the provider's policy under HCP ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).

The acceptance window for `issuedAt` is consumer policy.
