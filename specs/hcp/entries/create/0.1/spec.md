---
slug: hcp/entries/create
version: "0.1"
title: "HCP — Create Entry"
summary: An HCP Client writes a new entry about a person to a Human Context Protocol provider, under a grant that lets it write the entry's categories; carries the HCP 2.0-draft-3 Create operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Create
category: data-exchange
keywords: [hcp, human-context-protocol, personal-context, memory, entry, write, provenance]
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
    The request is the client's attributable statement about a person, which the provider records as the entry's
    provenance (HCP 3.5): the signature lets the provider, and later the person, show which client submitted which
    assertion. The response discloses the entry as stored, with its new `ref`, so it carries the provider's signature.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task changes the person's context and is consequential; a re-delivered create must be placeable in the
    provider's duplicate-execution window, so that it is absorbed instead of creating a second entry.
sideEffects:
  level: mutating
  rationale: >-
    A new entry is stored, and the provider appends a `context.created` audit event (HCP 8.1). The entry can later be
    revised or withdrawn (HCP 5.2.4, 5.2.5).
exposure:
  discloses: secret
  actsAsSubject: false
  ingests: secret
  rationale: >-
    The request carries an assertion about a person, which may be in a High-sensitivity category or be a credential,
    and the HCP access token, a bearer secret. The response discloses the stored entry back to the client.
retention:
  class: durable
  rationale: >-
    The provider keeps the entry and its provenance until it is hard-deleted, and the `context.created` audit event
    (HCP 4.2, 8.1).
    The access token is not retained (see Security & Privacy, The access token).
maxDocumentBytes:
  request: 524288
  response: 524288
  rationale: >-
    One entry, which may carry a verifiable credential and supporting credentials as evidence, plus envelope and proof.
    This is a framework bound, not an HCP requirement; HCP sets no size limit.
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence". A request under a
      revoked grant also fails here, pending HCP confirmation (question 2 in the spec's "Pending HCP confirmation"
      section).
    retryable: false
  - code: hcp:credentialInvalid
    meaning: >-
      HCP `credential-invalid` (HCP 5.5): a credential that is the entry's content fails verification, in its proof,
      its securing mechanism, its verification method or its status (HCP 14.6). The `message` says which, and reveals
      no status-list internals.
    retryable: false
related:
  - hcp/entries/revise
  - hcp/entries/withdraw
  - hcp/entries/get
---

## Abstract

The **HCP — Create Entry** Trust Task carries the Create operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 5.2.3; `POST /v2/entries` in HCP 9.2). An *HCP Client* writes a new entry about the *Context Subject* to an *HCP Provider*, under a grant that lets it write the entry's categories. The provider records the entry's provenance itself (HCP 3.5) and returns the entry as stored; to a *Context Client*, as a disclosure carrying the entry's new `ref`.

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and does not copy HCP's own schema files. Where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on the Create operation, including the VC Evidence Profile's ingest rules where the content is a credential (HCP 14.6); this specification restates the ones that bear on the document and does not relax any. A provider **MUST NOT** claim this carriage in its HCP `conformsTo`, and advertises the task through [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)).

## Authorization

The authority is **the grant** (HCP 6.1), evidenced by the access token in `accessToken`, which the provider checks on every request (HCP 6.2.2). Each category in `classification.categories` **MUST** be one the grant authorizes the caller to write; a write naming any other fails with not authorized (HCP 5.2.3), carried as `permissionDenied`. An `issuer-attested` write is accepted only from a client authenticated as the issuer, or authorized by it, and a write whose content is a credential is `issuer-attested`; any other is refused with `permissionDenied` (HCP 3.7, 5.2.3).

The REQUIRED `proof` establishes which client VID sent the assertion and that it is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). It is not the authorization, and it does not replace the provenance the provider records (HCP 3.5): `submittedByClient` and `authorizedUnder` are still taken from the grant. A token issued to a client other than the one the `issuer` VID identifies is refused with `permissionDenied` (see [The access token](#the-access-token)).

## Definitions

- **Context Client**, **Management Client**, **HCP Provider**, **Context Subject**, **Context Asserter**, **grant**, **entry**, **evidence** and **category** have their HCP 2.0-draft-3 meanings.
- The shared shapes are in [`hcp/_shared/0.1`](../../../_shared/0.1/common.schema.json): the write members, `WriteSubject` among them, in [`entry-write.schema.json`](../../../_shared/0.1/entry-write.schema.json), and `Content`, `EvidenceItem`, `Assertion` and `Entry` in [`context-entry.schema.json`](../../../_shared/0.1/context-entry.schema.json). They are open, as HCP's own schemas are.

### HCP field mapping

| HCP Create field | Member | Notes |
|---|---|---|
| request `schema`, `subject.id`, `content`, `evidence` | `schema`, `subject`, `content`, `evidence` | Unchanged. |
| request `classification.categories`, `classification.dimension` | `classification` | Unchanged. |
| request `assertion.*` | `assertion` | Unchanged. |
| request `record.provenance.*`, `record.relationships` | `record` | Unchanged. |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `Idempotency-Key` header (HCP 9.6) | the document `id` | See [Repeated requests](#repeated-requests). |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result: the created entry (`201`, HCP 9.4.3) | `entry` | **Wrapped**: HCP returns the entry or disclosure itself; this task puts it in `entry`, as HCP's own Get entry result does (HCP 5.2.2). |

### Parties

| HCP role | Trust Task member |
|---|---|
| Context Client or Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../query/0.1/spec.md#parties), a provider **MUST** make its VID verifiably bound to its HCP identity (HCP 19.1), and a client **MUST** check that binding before it sends a token or an assertion to a VID.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json). The members are HCP's (HCP 5.2.3):

- `schema`, `subject`, `content`, `classification`, `assertion` and `record` are REQUIRED; `evidence` is OPTIONAL. `record.provenance.source` is REQUIRED.
- `assertion.confidence` is REQUIRED and non-null when `assertion.declarationType` is `model-inferred`, and null or absent otherwise. A write that breaks this is refused with `malformedRequest` (HCP 5.2.3). The schema does not check it.
- `record.provenance.assertedBy.id` is REQUIRED unless `declarationType` is `person-declared`, when the provider records the Context Subject, or the content is a credential, when the provider takes the credential's issuer (HCP 5.2.3, 14.6). The schema does not check it.
- `record.relationships[].target`, and a proposal's `entry`, name an entry by its `ref` for a Context Client and its `id` for a Management Client (HCP 5.2.3). Only a `contradicts` relationship carries a `proposal` (HCP 4.7.4).
- `classification.permittedPurposes` is not a create member: only the Authorizing Principal sets it, by a governance operation (HCP 4.2.1, 4.2.6). A create carrying it is refused with `malformedRequest`, *pending HCP confirmation* ([question 7](#pending-hcp-confirmation)).
- A `schema` the provider does not accept, or a category it does not hold, is refused with `malformedRequest` (HCP 5.2.3).
- The provider assigns `id`, `record.status`, `record.version`, the rest of `record.provenance` and each relationship's `assertedBy` and `recordedAt`, and never takes them from a request (HCP 5.2.3).
- `accessToken` (REQUIRED) is the HCP access token for the grant.

### A booking agent records an inferred preference

```json
{
  "id": "urn:uuid:7a2d5f81-3b9c-4e6a-8d1f-0c4b2e9a7d13",
  "type": "https://trusttasks.org/spec/hcp/entries/create/0.1#request",
  "issuer": "did:web:booking-agent.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T10:00:00Z",
  "threadId": "urn:uuid:1e9c4a73-6d2b-4f8e-a5c0-3b7d9f1e2a24",
  "payload": {
    "schema": "https://hcp.example/spec/entry/2.0",
    "subject": { "id": "urn:uuid:3f8a1c2e-5b7d-4e9f-a0c6-8d2b4e1f7a35" },
    "content": {
      "profile": "https://hcp.example/spec/profiles/general-context/2.0",
      "value": "Prefers a window table.",
      "language": "en"
    },
    "classification": { "categories": ["general_preferences"], "dimension": "seating" },
    "assertion": {
      "declarationType": "model-inferred",
      "confidence": "medium",
      "basis": { "task": "Restaurant booking", "rationale": "Asked for a window seat at three bookings." }
    },
    "record": {
      "provenance": {
        "assertedBy": { "id": "https://booking-agent.example/model/v4" },
        "source": "integration"
      }
    },
    "accessToken": "hcp_at_2b7f0c6e9d1a4e8b"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T10:00:00Z",
    "verificationMethod": "did:web:booking-agent.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z2HdT9qLw5RnX3vYb8KcF1mJp7sAe4uGz6oNiQ2xV9tB5kW3rM8yD1gS6hL4jE7aP2cU9nO5fZ3wC8vT1bR6mK4q"
  }
}
```

## Response

The provider answers with the created entry. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): `entry`, a `Disclosure` carrying the entry's new `ref` for a Context Client (HCP 7.4), otherwise the whole `Entry` with the provider-assigned fields populated (HCP 5.2.3). Failures use `trust-task-error` (see [Errors](#errors)).

### The entry, created

```json
{
  "id": "urn:uuid:9b4e2c17-8a3d-4f6b-b1e9-5d0c7a2f3e46",
  "type": "https://trusttasks.org/spec/hcp/entries/create/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:booking-agent.example",
  "issuedAt": "2026-11-25T10:00:01Z",
  "threadId": "urn:uuid:1e9c4a73-6d2b-4f8e-a5c0-3b7d9f1e2a24",
  "payload": {
    "entry": {
      "ref": "hcp-ref-c73e19",
      "version": 1,
      "content": {
        "profile": "https://hcp.example/spec/profiles/general-context/2.0",
        "value": "Prefers a window table.",
        "language": "en"
      },
      "assertion": {
        "declarationType": "model-inferred",
        "confidence": "medium",
        "basis": { "task": "Restaurant booking", "rationale": "Asked for a window seat at three bookings." }
      },
      "classification": { "categories": ["general_preferences"], "dimension": "seating" },
      "standing": { "assurance": "authenticated", "confirmed": false },
      "useUntil": "2026-11-26T10:00:01Z"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T10:00:01Z",
    "verificationMethod": "did:web:context.provider.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z5JmR8tXq2WvL6nB3dKcY9pHf1sZa7eU4oGiN2xT8wV5bM3rE6kP9gS1jA4hD7yC2uF8lO5zQ3nX6mB9vT1kR4w"
  }
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows. The provider never sends HCP's Core error object or the HTTP binding's problem details (HCP 9.7).

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request`: schema validation, a `confidence` that does not match `declarationType`, a `schema` the provider does not accept, a category it does not hold (HCP 5.2.3) | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: a category the grant does not let the caller write, an `issuer-attested` write from a client that is not the issuer or authorized by it (HCP 5.2.3, 3.7) | `permissionDenied`, with one fixed `message` whatever the reason (HCP 5.5, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| conflict | `idempotency-conflict`: a retried request that does not match the original (HCP 9.6) | `idConflict`: the document `id` reused with different content ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)). See [Repeated requests](#repeated-requests). |
| version conflict | `version-conflict` | Does not arise: a create names no existing version (HCP 5.2.4). |
| credential invalid | `credential-invalid`: the content is a credential that fails verification (HCP 14.6) | `hcp:credentialInvalid`, `retryable: false`. The `message` says which check failed (proof, securing mechanism, verification method or status) and reveals no status-list internals (HCP 5.5). |
| subject unavailable | `subject-unavailable`: under end-to-end encrypted storage, the content checks need the Context Subject's trust domain (HCP 4.8, 5.2.3) | `unavailable`. The client may retry later (HCP 5.5). |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion` |

A client that does not know `hcp:invalidToken` or `hcp:credentialInvalid` reads it as `taskFailed` ([SPEC §8.5](/SPEC.md#85-extension-by-individual-trust-task-specifications)) and still honours `retryable: false`.

## Repeated requests

HCP's HTTP binding makes a create idempotent under an `Idempotency-Key` header (HCP 9.6). This task uses the document `id` as that key: it is minted fresh per create, and [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements) applies.

- A re-delivered document with the same `id` and content is not executed again. The provider **MUST** answer it with the original response, including the original `ref` (or `id`) and `version`, and **MUST NOT** append a second `context.created` event (HCP 9.6).
- A document whose `id` matches an accepted one but whose content differs is refused with `idConflict`, HCP's conflict (HCP 9.6).
- The provider **MUST** retain its duplicate-execution record, with the response, for at least 24 hours (HCP 9.6), and for at least the acceptance window it applies to `issuedAt`.

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 1 — conformance claims.** A provider **MUST NOT** claim this carriage in its HCP `conformsTo` (see [Conformance](#conformance)).
- **Question 2 — a request under a revoked grant.** Refused with `hcp:invalidToken` (see [Errors](#errors)).
- **Question 7 — `classification.permittedPurposes` on a create.** Refused with `malformedRequest`, since only a governance operation sets it (see [Request](#request)).
- **Question 22 — the client's identity.** The `issuer` VID, authenticated by the request's `proof`, identifies the HCP client the token was issued to; where the request carries a proof, a token issued to any other client is refused with `permissionDenied`, a strengthening HCP does not itself require (see [The access token](#the-access-token)).

## What HCP requires that a document cannot express

- **The grant and write-scope checks**, and the issuer check on an `issuer-attested` write (HCP 3.7, 5.2.3).
- **Provenance** (HCP 3.5): the provider records `assurance`, `submittedByClient`, `authorizedUnder`, `recordedByProvider` and `recordedAt` from what it knows, never from the request.
- **Credential ingest** (HCP 14.6): verification of a credential that is the content, which the provider **MUST NOT** modify (HCP 4.2.2).
- **Conflict detection** (HCP 4.7) and the `conflict.proposed` event for a proposal the client makes (HCP 4.7.4).
- **Encrypted storage** (HCP 4.8): content checks made within the Context Subject's trust domain.

## Security & Privacy

### Data carried

The request carries an assertion about the person, which may be in a High-sensitivity category, a credential with its proof, and the access token, a bearer secret. The response discloses the stored entry back to the client. A producer **MUST NOT** put content or the token in `ext` or in any `message`, and an `hcp:credentialInvalid` `message` names the failed check without status-list internals (HCP 5.5).

### The access token

`accessToken` is a bearer secret: whoever presents it exercises the authority it evidences. It **MUST** travel only to the provider. A recipient **MUST NOT** log `accessToken` or retain it beyond the exchange: a duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) or an evidentiary record of the request keeps the document with `accessToken` removed, or a digest of the document, such as its task digest ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)), and never the token. Where the request carries a `proof`, the provider **MUST** refuse with `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies, so that a token lifted from a stored or forwarded document is of no use from another VID. HCP does not itself bind a token to a VID: this is a strengthening of the carriage, resting on the `issuer` VID identifying the HCP client, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)).

### Correlation

The provider learns which client asserted what, when; that is the provenance HCP records. A Context Client receives the new entry only by its grant-scoped `ref` (HCP 7.4). The `subject.id` the client supplies is the per-client pseudonym from its grant (HCP 3.4), so it does not join clients. `threadId` **SHOULD** be fresh per create.

### Retention

The provider keeps the entry until it is hard-deleted (HCP 5.4.2), with the `context.created` audit event (HCP 8.1). The signed request is evidence of what the client asserted, and a provider **MAY** keep it with the entry's provenance, with `accessToken` removed (see [The access token](#the-access-token)). The client's disclosed copy of the entry is subject to its `useUntil` like any other (HCP 7.6).

### Consent/purpose

The grant's write scope is the person's permission to have context written about them in those categories. The person can see, confirm or reject every entry through a Management Client (HCP 3.8). Whether a given write needs any further approval is the provider's policy under HCP, not a requirement of this specification ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).

The acceptance window for `issuedAt` is consumer policy.
