---
slug: hcp/grants/read-own
version: "0.1"
title: "HCP — Read Own Grant"
summary: A Context Client reads the Human Context Protocol grant under which it acts, with the directives that apply to it; carries the HCP 2.0-draft-3 Read own grant operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Read own grant
category: access-control
keywords: [hcp, human-context-protocol, grant, purposes, directives]
parties:
  - role: Context Client (HCP), the client the grant was issued to
    requirement: REQUIRED
    member: issuer
  - role: HCP Provider
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The grant is evidenced by the access token, which the provider checks on every request. The request proof is
    REQUIRED, as on every hcp/* request that carries a token, because it is what lets the provider refuse a token sent
    from a VID other than the client's (a carriage rule pending HCP confirmation, question 53). A response proof lets the
    client show which categories and purposes the provider said it was granted, which is what the client relies on
    when it states a purpose; it is RECOMMENDED because the read discloses nothing the client was not already told at
    authorization time (HCP 5.2.6).
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The answer reflects the grant's current state, including its status, so a request is placed in time and a stale
    one refused.
sideEffects:
  level: none
  rationale: >-
    A read. HCP requires no audit event for it (HCP 5.2.6, 6.1.5).
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: secret
  rationale: >-
    The response discloses the grant record: the person's per-client pseudonym, the categories and purposes approved,
    and any narrowing and onward-sharing election. It carries no context. The request carries the HCP access token, a
    bearer secret.
retention:
  class: transient
  rationale: >-
    The provider needs the request only to answer it: nothing is recorded by the read (HCP 5.2.6, 6.1.5), and
    beyond the duplicate-execution digest every task keeps (SPEC 7.2 item 11), it keeps nothing of the request, as
    for List grants. The grant record the client receives is the provider's, which the client may keep for the life
    of the grant.
maxDocumentBytes:
  response: 5242880
  rationale: >-
    One grant object plus the directives that apply to it (HCP 5.2.6). HCP bounds neither, and the result has no
    pages. The directives that apply to one grant are a subset of every directive, which hcp/directives/list
    bounds at 4 MiB; the extra 1 MiB holds the grant, envelope and proof. This is a framework bound, not an HCP
    requirement.
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence". A request under a
      revoked grant also fails here, pending HCP confirmation (question 2 in the spec's "Pending HCP confirmation"
      section).
    retryable: false
related:
  - hcp/grants/relinquish
  - hcp/grants/revoked
  - hcp/entries/query
---

## Abstract

The **HCP — Read Own Grant** Trust Task carries the Read own grant operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 5.2.6; `GET /v2/grants` with a Context Client's token in HCP 9.2). A *Context Client* asks an *HCP Provider* for the grant under which it acts, and receives that grant only (HCP 6.1.5), with the directives that apply to it (HCP 6.4). This is how a client learns which purposes the person approved (HCP 6.1.5.2).

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification and the shared [`grant.schema.json`](../../../_shared/0.1/grant.schema.json) were written from that text, with attribution, and do not copy HCP's own schema files. Where they differ from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on reading a grant (HCP 5.2.6, 6.1.5). A provider **MUST NOT** claim this carriage in its HCP `conformsTo`, and advertises the task through [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)).

This specification adds one check HCP does not make: the token binding of [The access token](#the-access-token), a carriage rule of the `hcp/*` family, not HCP's. [SPEC §9.4](/SPEC.md#94-carried-protocols) item 6 places a check the carried protocol does not make in a separate specification that does not declare `carries`. This family keeps this one in the carriage, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)), because it protects the token that the carriage itself moves from the HTTP `Authorization` header into the document, where it can be stored or forwarded with the document; a separate task could not protect the document this one carries.

## Authorization

The authority is **the grant itself**, evidenced by the access token in `accessToken`: a client may read the grant under which it acts, and no other (HCP 5.2.6). The token names the grant, so the request names none. An HCP Provider **MUST NOT** disclose a grant issued to one HCP Client to any other HCP Client, on every binding (HCP 6.1.5).

A Management Client's token does not stand for a grant it acts under; over HCP's HTTP binding the same `GET /v2/grants` with such a token is List grants, a governance operation (HCP 9.2, 9.4.6), which is not this task. A provider refuses a Management Client's token here with `permissionDenied`.

The request always carries a proof, *pending HCP confirmation* ([question 53](#pending-hcp-confirmation)), so a token issued to a client other than the one the `issuer` VID identifies is always refused with `permissionDenied` (see [The access token](#the-access-token)).

## Definitions

- **HCP Client**, **Context Client**, **Management Client**, **HCP Provider**, **grant**, **authorization detail**, **purpose** and **directive** have their HCP 2.0-draft-3 meanings.
- The grant shape is [`Grant`](../../../_shared/0.1/grant.schema.json) in `hcp/_shared/0.1`; the directive shape is [`Directive`](../../../_shared/0.1/directive.schema.json). Both are open, as HCP's own schemas are.

### HCP field mapping

| HCP field | Member | Notes |
|---|---|---|
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result: the grant (HCP 5.2.6, 6.1.5) | `grant` | **Wrapped**: HCP's published result schema is the grant object itself. |
| "with the directives that apply to it" (HCP 5.2.6) | `directives` | HCP's text gives the result directives but names no member for them, and its published result schema has none; see [question 18](#pending-hcp-confirmation). |

**Where this payload is stricter than HCP's shapes.** Each payload is closed at its top level (`additionalProperties: false`): a top-level member outside this mapping is refused, where HCP's own request and result shapes leave the top level open, so an extension member HCP would allow there is not carried. The HCP shapes nested within stay open, as HCP's are. Every size bound, and every non-empty requirement (a `minLength` or `minItems` of 1) on a member that HCP's own schema allows to be empty, is a Trust Tasks framework bound on what an `hcp/*` party writes, not an HCP requirement, as the schema descriptions say.

### Parties

| HCP role | Trust Task member |
|---|---|
| Context Client (HCP 5.2.6 says HCP Client; a Management Client's token is refused, see [Authorization](#authorization)) | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../../entries/query/0.1/spec.md#parties), a provider **MUST** make its VID verifiably bound to its HCP identity (HCP 19.1), and a client **MUST** check that binding before it sends a token to a VID.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json): `accessToken` (REQUIRED) only.

### The booking agent checks what it was granted

```json
{
  "id": "urn:uuid:0a6d3f92-4b1e-4c8a-9f7d-2e5b8c1a6d03",
  "type": "https://trusttasks.org/spec/hcp/grants/read-own/0.1#request",
  "issuer": "did:web:booking-agent.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T08:00:00Z",
  "threadId": "urn:uuid:7b1e5c84-2d9f-4a3b-8c6e-0f4a7d2b9e14",
  "payload": {
    "accessToken": "hcp_at_2b7f0c6e9d1a4e8b"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T08:00:00Z",
    "verificationMethod": "did:web:booking-agent.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "zNxikNAa867TFAdkjbHVBrv8JrfTEaTyHm1ZyPxsvTTaGviQjLaEcwGFdx74HMXhycZJZZPkgjhARgMRPGQDJP8"
  }
}
```

## Response

The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json):

- `grant` is the client's own grant (HCP 6.1.5). Where a detail is narrowed to particular entries, a Context Client receives their `ref`s in `narrowedTo`, never their `id`s (HCP 5.2.6). `expiresAt` is an instant for a temporary grant and null for a persistent one (HCP 6.1.4), which the schema does not check.
- `directives` are the directives that apply to the grant, empty when none applies (HCP 5.2.6, 6.4). A client that claims to follow directives **MUST** follow every one delivered to it (HCP 6.4).

Failures use `trust-task-error` (see [Errors](#errors)).

### The grant

```json
{
  "id": "urn:uuid:6e2c9a15-8f4b-4d1e-a7c3-5b0d2f8e4a25",
  "type": "https://trusttasks.org/spec/hcp/grants/read-own/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:booking-agent.example",
  "issuedAt": "2026-11-25T08:00:01Z",
  "threadId": "urn:uuid:7b1e5c84-2d9f-4a3b-8c6e-0f4a7d2b9e14",
  "payload": {
    "grant": {
      "grantId": "urn:uuid:c4d2e1f0-9a8b-4c7d-8e6f-5a4b3c2d1e0f",
      "subject": { "id": "urn:uuid:3f8a1c2e-5b7d-4e9f-a0c6-8d2b4e1f7a35" },
      "authorizingPrincipal": { "id": "urn:uuid:3f8a1c2e-5b7d-4e9f-a0c6-8d2b4e1f7a35" },
      "client": {
        "id": "booking-agent",
        "name": "Example Booking Agent",
        "operator": { "id": "https://booking-agent.example", "name": "Example Booking Ltd" }
      },
      "access": [
        {
          "type": "https://hcp.example/spec/authorization/context-access",
          "actions": ["read", "write"],
          "categories": ["general_preferences"],
          "purposes": [
            { "code": "https://hcp.example/purposes/reservation", "description": "Booking restaurants for you" }
          ],
          "onwardSharing": { "domains": ["restaurants.example"] }
        },
        {
          "type": "https://hcp.example/spec/authorization/context-access",
          "actions": ["read"],
          "categories": ["health_context"],
          "purposes": [
            { "code": "https://hcp.example/purposes/reservation", "description": "Booking restaurants for you" }
          ]
        }
      ],
      "scopes": ["hcp:general_preferences:read", "hcp:general_preferences:write", "hcp:health_context:read"],
      "grantType": "persistent",
      "issuedAt": "2026-10-01T09:00:00Z",
      "expiresAt": null,
      "status": "active"
    },
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
| not authorized | `scope-denied`: a token that stands for no grant the caller acts under, such as a Management Client's (see [Authorization](#authorization)) | `permissionDenied`, with one fixed `message` |
| conflict | `idempotency-conflict` | Does not arise: a read carries no idempotency key (HCP 9.6). |
| version conflict | `version-conflict` | Does not arise. |
| credential invalid | `credential-invalid` | Does not arise. |
| subject unavailable | `subject-unavailable` | `unavailable` |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion` |

A grant that is `revoked` or `expired` is not returned: a token issued under it is no longer valid evidence (HCP 6.2.2), and the request fails with `hcp:invalidToken`, *pending HCP confirmation* ([question 2](#pending-hcp-confirmation)).

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 1 — conformance claims.** A provider **MUST NOT** claim this carriage in its HCP `conformsTo` (see [Conformance](#conformance)).
- **Question 2 — a request under a revoked grant.** Refused with `hcp:invalidToken` (see [Errors](#errors)).
- **Question 18 — where the directives go.** The result is `{ "grant", "directives" }`, `directives` REQUIRED and empty when none applies; HCP's text gives the result directives, and its published result schema is the bare grant (see [HCP field mapping](#hcp-field-mapping)).
- **Question 22 — the client's identity.** The `issuer` VID, authenticated by the request's `proof`, identifies the HCP client the token was issued to; a token issued to any other client **MUST** be refused with `permissionDenied`. This token binding is a carriage rule of this family, a check HCP does not itself make (see [The access token](#the-access-token) and [Conformance](#conformance)).
- **Question 53 — a request proof on every request that carries a token.** A request `proof` is **REQUIRED** on every `hcp/*` request that carries `accessToken`, so that the token binding of question 22 always applies and a token lifted from a stored or forwarded document cannot be replayed in an unsigned request. This is a carriage rule of this family, not HCP's, and is stricter than HCP's bearer model, under which presenting a valid token is enough (HCP 9.3); see [The access token](#the-access-token).
- **Question 55 — HCP's grant and delegation schemas disagree.** HCP's grant schema requires a non-empty `client.name`, where its common `clientDescription`, which an entry's provenance uses, does not. The shared `Grant` follows HCP's grant schema and requires it, as does the shared `ClientDescription`.

## What HCP requires that a document cannot express

- **Grant isolation** (HCP 6.1.5): the provider returns the caller's own grant and never another client's.
- **`ref`s, not `id`s, in a narrowing** (HCP 5.2.6): the provider rewrites the narrowing for a Context Client.
- **Rule 3 of HCP 6.1.5.1**: a narrowed detail always carries its `narrowedTo`, so the record never overstates the grant.

## Security & Privacy

### Data carried

The request carries only the access token, a bearer secret. The response carries the grant record: the per-client pseudonym of the person, the client's own registration details, what the person approved, and the directives. It carries no entry content. A producer **MUST NOT** put the token in `ext` or any `message`.

### The access token

`accessToken` is a bearer secret: whoever presents it exercises the authority it evidences. It **MUST** travel only to the provider. A recipient **MUST NOT** log `accessToken` or retain it beyond the exchange.

**Token binding, a carriage rule of this family.** Every request that carries `accessToken` carries a `proof`, *pending HCP confirmation* ([question 53](#pending-hcp-confirmation)), and the provider **MUST** refuse with `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies, so that a token lifted from a stored or forwarded document is of no use from another VID. HCP validates the token alone and does not bind it to a VID (HCP 9.3): this rule is the `hcp/*` family's, not HCP's, and rests on the `issuer` VID identifying the HCP client, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)). It is a check HCP does not make (see [Conformance](#conformance)).

**Retained copies.** The provider verifies the request's `proof` over the document as received, at receipt, before it removes anything ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). A provider that retains the request document, as a duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) or as evidence of the request, **MUST** remove `accessToken` from the copy it keeps and record in its place `accessTokenDigest`: the SHA-256 multihash of the token's UTF-8 bytes, multibase-encoded as base64url without padding (prefix `u`), the framework's `DigestMultibase` form ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)). The kept copy's `proof` no longer verifies over it, so the provider records with the copy that the proof was verified at receipt: when, under which verification method, and with what result. It relies on that record, and **MUST NOT** verify the proof again over the kept copy. A record that keeps only a digest of the document, such as its task digest ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)), keeps no token either.

### Correlation

The grant's `subject` and `authorizingPrincipal` identifiers are pseudonyms specific to this client (HCP 3.4), so they do not join the person across clients. The `grantId` joins this read to the client's other requests and to any revocation notice, by design. `threadId` **SHOULD** be fresh per read.

### Retention

The provider records nothing for the read (HCP 5.2.6). The client may keep the grant record while the grant lasts, and discards it when the grant ends.

### Consent/purpose

The grant record is what the person approved; the client reads it to stay within it. Whether a client must read its grant before acting is not a requirement of this specification ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).

The acceptance window for `issuedAt` is consumer policy.
