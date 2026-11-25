---
slug: hcp/shares/report
version: "0.1"
title: "HCP — Report Onward Sharing"
summary: An HCP Client tells a Human Context Protocol provider what context it shared with another organization, so the person can see it; carries the HCP 2.0-draft-3 Report onward sharing operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Report onward sharing
category: data-exchange
keywords: [hcp, human-context-protocol, onward-sharing, disclosure, audit]
parties:
  - role: HCP Client (the client that shared)
    requirement: REQUIRED
    member: issuer
  - role: HCP Provider
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The report is the client's own account of a disclosure it made, which the provider records for the person (HCP
    5.2.8, 8.1), so it is signed: the person's record can show which client reported which share. The response proof is
    RECOMMENDED so the client can show it reported.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task records an event for the person and is consequential; a re-delivered report must be placeable in the
    provider's duplicate-execution window, so that it is recorded once.
sideEffects:
  level: mutating
  rationale: >-
    The provider appends a `context.shared` audit event for every report (HCP 5.2.8, 8.1). Nothing else changes.
exposure:
  discloses: none
  actsAsSubject: false
  ingests: secret
  rationale: >-
    The request carries the HCP access token, a bearer secret, and says which of the person's entries went to which
    organization, for which purpose. The response carries nothing.
retention:
  class: durable
  rationale: >-
    The provider keeps the `context.shared` audit event, naming the entries by their `id`, whether or not the share
    was allowed, because the Context Subject is entitled to see it (HCP 5.2.8, 8.1).
    The access token is not retained: a kept request holds its digest instead (see Security & Privacy, The access token).
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence". A request under a
      revoked grant also fails here, pending HCP confirmation (question 2 in the spec's "Pending HCP confirmation"
      section).
    retryable: false
related:
  - hcp/entries/query
  - hcp/entries/get
  - hcp/grants/read-own
---

## Abstract

The **HCP — Report Onward Sharing** Trust Task carries the Report onward sharing operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 5.2.8; `POST /v2/shares` in HCP 9.2). An *HCP Client* that shared context with another organization, where the person allowed it (HCP 3.12), tells the *HCP Provider* which entries it drew on, whom it told, for which purpose, and when. The provider records the share for the person, whether or not the grant allowed it.

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and does not copy HCP's own schema files. Where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A client that shares context with another organization under HCP 3.12 **MUST** report each share to the provider when it next reaches it (HCP 5.2.8); this task is one way to do so. A provider **MUST NOT** claim this carriage in its HCP `conformsTo`, and advertises the task through [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)).

This specification adds one check HCP does not make: the token binding of [The access token](#the-access-token), a carriage rule of the `hcp/*` family, not HCP's. [SPEC §9.4](/SPEC.md#94-carried-protocols) item 6 places a check the carried protocol does not make in a separate specification that does not declare `carries`. This family keeps this one in the carriage, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)), because it protects the token that the carriage itself moves from the HTTP `Authorization` header into the document, where it can be stored or forwarded with the document; a separate task could not protect the document this one carries.

## Authorization

The report is made under **the grant under which the client shared**, evidenced by the access token in `accessToken` (HCP 9.4.16). A share is one the grant allowed only where the authorization detail covering the entries carries `onwardSharing`, every entry was eligible (HCP 7.4), `purpose` is one of the detail's purposes, and the recipient's `domain` is within the detail's `domains`, where present, and not in its `notTo` (HCP 5.2.8, 6.1.5). The provider checks this against its own state; the client's report is not taken as proof that the share was allowed.

A share the grant did not allow is still recorded: the `context.shared` event records `permitted: false`, and the task fails with `permissionDenied` (HCP 5.2.8). The event is recorded either way, because the share has already happened and the Context Subject is entitled to see it.

A token issued to a client other than the one the `issuer` VID identifies is refused with `permissionDenied` (see [The access token](#the-access-token)).

## Definitions

- **HCP Client**, **HCP Provider**, **Context Subject**, **grant**, **authorization detail**, **onward sharing** and **ref** have their HCP 2.0-draft-3 meanings.

### HCP field mapping

| HCP field | Member | Notes |
|---|---|---|
| request `recipient` (`domain`, `name`) | `recipient` | Unchanged. It is the organization told, not the document's own `recipient`, which is the provider. |
| request `refs` | `refs` | Unchanged. |
| request `purpose` | `purpose` | Unchanged. |
| request `sharedAt` | `sharedAt` | Unchanged. |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result `204 No Content` (HCP 9.4.16); MCP `hcp_reportSharing` (HCP 10.3.6) | the empty `#response` payload | HCP defines no result, so none here. |

### Parties

| HCP role | Trust Task member |
|---|---|
| HCP Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

The organization told is named in the payload, not as a party: it takes no part in the exchange. As for [`hcp/entries/query`](../../../entries/query/0.1/spec.md#parties), a provider **MUST** make its VID verifiably bound to its HCP identity (HCP 19.1).

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json): `recipient` (its `domain` and a `name` the person can recognize), `refs` (at least one), `purpose`, `sharedAt` and `accessToken`, all REQUIRED. A report covers one share; a client that shared with two organizations sends two reports.

### The booking agent told a restaurant about a preference

```json
{
  "id": "urn:uuid:1f5a8c37-9b2e-4d6a-b3c1-8e0d4a7f5b69",
  "type": "https://trusttasks.org/spec/hcp/shares/report/0.1#request",
  "issuer": "did:web:booking-agent.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T13:15:00Z",
  "threadId": "urn:uuid:4e8b1d59-0c3f-4a7e-9d2b-6a5c8f1e3d70",
  "payload": {
    "recipient": { "domain": "restaurants.example", "name": "Example Bistro" },
    "refs": ["hcp-ref-a91c4f"],
    "purpose": "https://hcp.example/purposes/reservation",
    "sharedAt": "2026-11-25T13:10:00Z",
    "accessToken": "hcp_at_2b7f0c6e9d1a4e8b"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T13:15:00Z",
    "verificationMethod": "did:web:booking-agent.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z5PtW3zYv9AqR2yUf8NeL4sMj1gDi7xHc5rVlQ6aY2wG9oE3uZ8pT4jI1kM7nS5dX2fC6hR9vA3gY8bU2xP7eN4w"
  }
}
```

## Response

The provider answers when the report is recorded and the share was one the grant allowed. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): the empty object, which carries HCP's `204 No Content`. It is declared because the client relies on knowing its duty to report was discharged ([SPEC §4.4.2](/SPEC.md#442-acknowledging-a-fire-and-forget-task)). A share the grant did not allow is answered with `permissionDenied` after it is recorded (see [Errors](#errors)).

### Recorded

```json
{
  "id": "urn:uuid:8a2d6f04-3e9c-4b1d-a7f5-0c4e9b3d6a81",
  "type": "https://trusttasks.org/spec/hcp/shares/report/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:booking-agent.example",
  "issuedAt": "2026-11-25T13:15:01Z",
  "threadId": "urn:uuid:4e8b1d59-0c3f-4a7e-9d2b-6a5c8f1e3d70",
  "payload": {}
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows.

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request` | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: a share the grant did not allow (HCP 5.2.8) | `permissionDenied`, with one fixed `message`. The share is recorded all the same, with `permitted: false` (HCP 5.2.8). |
| conflict | `idempotency-conflict` | Does not arise: HCP defines no idempotency key for a report (HCP 9.6). A re-delivered document is handled under [Repeated requests](#repeated-requests). |
| version conflict | `version-conflict` | Does not arise. |
| credential invalid | `credential-invalid` | Does not arise. |
| subject unavailable | `subject-unavailable` | `unavailable`. The client reports again later. |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter`. The client reports again later. |
| unsupported version | `unsupported-version` | `unsupportedVersion` |

## Repeated requests

HCP requires a `context.shared` event for every report (HCP 5.2.8, 8.1), and defines no idempotency for one (HCP 9.6). Because this task is consequential, [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements) applies: a re-delivered document with the same `id` and content is not executed again, so it appends no second `context.shared` event, *pending HCP confirmation* ([question 26](#pending-hcp-confirmation)). A client that retries a report it is unsure arrived resends the same document, never a new one, so that one share is recorded once.

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 1 — conformance claims.** A provider **MUST NOT** claim this carriage in its HCP `conformsTo` (see [Conformance](#conformance)).
- **Question 2 — a request under a revoked grant.** Refused with `hcp:invalidToken`. A client whose grant was revoked after it shared has no way to report under HCP 2.0-draft-3; that gap is HCP's, and this task does not fill it.
- **Question 22 — the client's identity.** The `issuer` VID, authenticated by the request's `proof`, identifies the HCP client the token was issued to; where the request carries a proof, a token issued to any other client **MUST** be refused with `permissionDenied`. This token binding is a carriage rule of this family, a check HCP does not itself make (see [The access token](#the-access-token) and [Conformance](#conformance)).
- **Question 26 — a re-delivered report.** It appends no second `context.shared` event (see [Repeated requests](#repeated-requests)).

## What HCP requires that a document cannot express

- **The client's duty to report** every share when it next reaches the provider (HCP 5.2.8): the provider cannot observe a share it was not told of.
- **The permission check** (HCP 5.2.8, 6.1.5): made against the grant and standing rules the provider holds.
- **Recording either way** (HCP 5.2.8): the event is appended before the refusal, and names the entries by `id`, which the client never sees.

## Security & Privacy

### Data carried

The request says which of the person's entries, by grant-scoped `ref`, went to which organization, for which purpose and when, and carries the access token, a bearer secret. It carries no entry content. The response carries nothing. A producer **MUST NOT** put context or the token in `ext` or any `message`.

### The access token

`accessToken` is a bearer secret: whoever presents it exercises the authority it evidences. It **MUST** travel only to the provider. A recipient **MUST NOT** log `accessToken` or retain it beyond the exchange.

**Token binding, a carriage rule of this family.** Where the request carries a `proof`, the provider **MUST** refuse with `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies, so that a token lifted from a stored or forwarded document is of no use from another VID. HCP validates the token alone and does not bind it to a VID (HCP 9.3): this rule is the `hcp/*` family's, not HCP's, and rests on the `issuer` VID identifying the HCP client, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)). It is a check HCP does not make (see [Conformance](#conformance)).

**Retained copies.** The provider verifies the `proof`, where the request carries one, over the document as received, at receipt, before it removes anything ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). A provider that retains the request document, as a duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) or as evidence of the request, **MUST** remove `accessToken` from the copy it keeps and record in its place `accessTokenDigest`: the SHA-256 multihash of the token's UTF-8 bytes, multibase-encoded as base64url without padding (prefix `u`), the framework's `DigestMultibase` form ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)). The kept copy's `proof` no longer verifies over it, so the provider records with the copy that the proof was verified at receipt: when, under which verification method, and with what result. It relies on that record, and **MUST NOT** verify the proof again over the kept copy. A record that keeps only a digest of the document, such as its task digest ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)), keeps no token either.

### Correlation

The report tells the provider which organization received the person's context; recording that for the person is the operation's purpose (HCP 5.2.8). The `refs` are grant-scoped and mean nothing to anyone but the provider (HCP 7.4). `threadId` **SHOULD** be fresh per report.

### Retention

The provider keeps the `context.shared` audit event, readable by the Context Subject (HCP 8.3), whether or not the share was allowed (HCP 5.2.8). The signed report is the client's own account of the share, and a provider **SHOULD** keep it with that event, with `accessToken` replaced by its digest and a record that its proof was verified at receipt (see [The access token](#the-access-token)).

### Consent/purpose

The person allowed onward sharing when they granted it, and the report lets them see each use of that permission. A report of a share the grant did not allow is not consent after the fact: it is refused, and recorded as not permitted ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).

The acceptance window for `issuedAt` is consumer policy; a generous window suits this task, since a client reports when it next reaches the provider and a refused report leaves the person's record incomplete.
