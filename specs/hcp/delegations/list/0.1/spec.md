---
slug: hcp/delegations/list
version: "0.1"
title: "HCP — List Delegations"
summary: A Management Client, acting for the person or a delegate, lists the Human Context Protocol delegations it may see; carries the HCP 2.0-draft-3 List delegations operation of the Delegated Authorization Profile.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: List delegations
category: access-control
keywords: [hcp, human-context-protocol, delegation, guardianship, governance]
parties:
  - role: Management Client (HCP), acting for the Context Subject or a delegate
    requirement: REQUIRED
    member: issuer
  - role: HCP Provider
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The response states who may act for the person, on what basis and within what scope, and may carry the evidence
    of that basis, such as a credential of guardianship or a court order. It carries the provider's signature, so
    the person or the delegate can show what the provider held as the delegation, and when. The request proof is
    REQUIRED, as for every Management Client read whose response is secret: the access token is the authority, but
    a signed request lets the provider refuse a token lifted from a stored or forwarded document and sent from
    another VID, and shows which Management Client asked for the disclosure.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task discloses secret material and so is consequential; a re-delivered read must be placeable in the
    consumer's duplicate-execution window.
sideEffects:
  level: none
  rationale: >-
    A read of the delegations the provider holds. HCP 8.1 lists no audit event for it, so the provider records
    nothing and changes nothing.
exposure:
  discloses: secret
  actsAsSubject: false
  ingests: secret
  rationale: >-
    The response discloses who may act for the person and why: the delegate's name, the relationship (a guardian, an
    attorney-in-fact), the basis, which may hold the evidence presented whole (a credential of guardianship, a court
    order, a power of attorney, HCP 17.3), and the High-sensitivity categories it covers. The request carries the
    Management Client's access token, a bearer secret.
retention:
  class: transient
  rationale: >-
    A read: HCP 8.1 lists no audit event for it, and the provider keeps nothing of the request beyond its
    duplicate-execution record.
maxDocumentBytes:
  response: 2097152
  rationale: >-
    The delegations for one person are few, but each may carry up to 16 evidence items, each of which may be a
    Verifiable Credential held whole, plus envelope and proof. This is a framework bound, not an HCP requirement;
    HCP sets no size limit and does not paginate this list.
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence", here the Management
      Client's access token.
    retryable: false
related:
  - hcp/delegations/end
  - hcp/audit/read
  - hcp/grants/list
---

## Abstract

The **HCP — List Delegations** Trust Task carries the List delegations operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) Delegated Authorization Profile (HCP 17.2; `GET /v2/delegations` in HCP 9.2 and 9.4.13). A *Management Client*, acting for the *Context Subject* or for a delegate, asks an *HCP Provider* for the delegations it may see: who may act as Authorizing Principal for the person, in which relationship, on what basis, within what scope and until when. A delegation is ended with [`hcp/delegations/end`](../../end/0.1/spec.md); it is established by the Authorization Service, never through this family (HCP 17.2, 9.4.13).

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** conform to the Delegated Authorization Profile (HCP 17.1, 9.4.13) and meet every requirement it places on delegations; the ones that bear on the document are restated below, unchanged. Conformance to HCP is HCP's ([SPEC §9.4](/SPEC.md#94-carried-protocols)): a provider **MUST NOT** list a conformance identifier for this carriage in its HCP `conformsTo`, and advertises the task through `trust-task-discovery`, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)). HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1); this specification takes it that a carriage other than that binding may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and carries this one as a Trust Task on that reading, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

**A provider that does not conform to the profile.** HCP exposes `GET /v2/delegations` only on an implementation conforming to the Delegated Authorization Profile (HCP 9.4.13). A provider that does not conform **MUST NOT** advertise this task, and answers a request for it with `unsupportedType`, as for any task it does not serve ([SPEC §8.3](/SPEC.md#83-standard-error-codes)). That is the carriage of an endpoint that is not exposed, not a refusal: whether a provider conforms to the profile is public in its discovery document (HCP 19), so `unsupportedType` reveals nothing about the person, and `permissionDenied` would wrongly suggest that some other caller could succeed.

This specification adds one check HCP does not make: the token binding of [The access token](#the-access-token), a carriage rule of the `hcp/*` family, not HCP's. [SPEC §9.4](/SPEC.md#94-carried-protocols) item 6 places a check the carried protocol does not make in a separate specification that does not declare `carries`. This family keeps this one in the carriage, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)), because it protects the token that the carriage itself moves from the HTTP `Authorization` header into the document, where it can be stored or forwarded with the document; a separate task could not protect the document this one carries.

## Authorization

The authority is **the right of the Context Subject and of a delegate to read a delegation** (HCP 17.2, 18.4), which a Management Client exercises for them. It is evidenced by the Management Client's access token in `accessToken`, which the provider validates exactly as over HCP's HTTP binding. Listing delegations is a governance operation (HCP 9.2, 6.1.9): any caller other than a Management Client acting for the Context Subject or for a delegate receives not authorized, carried as `permissionDenied`. A grant held by a Context Client never authorizes it, whatever its scopes (HCP 6.1.9). HCP names only its HTTP Binding as the carriage for a governance operation (HCP 5.4, 9.1); this specification accepts the operation as a Trust Task from a Management Client acting for the Authorizing Principal, and from no other caller (HCP 6.1.9), *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

- A Management Client acting for the **Context Subject** receives every delegation for them (HCP 9.4.13), including sub-delegations (HCP 17.10).
- A Management Client acting for a **delegate** receives the delegations naming that delegate (HCP 9.4.13), and also each sub-delegation that delegate appointed (one whose `basis.parent` is a delegation naming them), *pending HCP confirmation* ([question 32](#pending-hcp-confirmation)). HCP 9.4.13 gives a delegate only the delegations naming them, but HCP 17.6 and 17.10 let the delegate who appointed a sub-delegation end it, which [`hcp/delegations/end`](../../end/0.1/spec.md) does by its `delegationId`, and this list is where that identifier comes from. The delegate sees no other delegation, and reads the record of a sub-delegate's grants through [`hcp/audit/read`](../../../audit/read/0.1/spec.md) (HCP 17.7, 17.10).
- **Neither may change a delegation**; a change is a new delegation, which the Authorization Service establishes (HCP 17.2). This task only reads.

The REQUIRED `proof` on the request establishes which Management Client VID sent the document and that it is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). It is not the authorization: the token is. A token issued to a client other than the one the `issuer` VID identifies is refused with `permissionDenied` (see [The access token](#the-access-token)).

## Definitions

- **Management Client**, **Context Client**, **HCP Provider**, **Authorization Service**, **Context Subject**, **Authorizing Principal**, **delegate**, **delegation** and **sub-delegation** have their HCP 2.0-draft-3 meanings.
- The delegation shape is the shared [`Delegation`](../../../_shared/0.1/delegation.schema.json) (HCP 17.2). It is open, as HCP's own shapes are, and carries HCP's member names verbatim, including `ended.at`.

### HCP field mapping

| HCP field | Member | Notes |
|---|---|---|
| query parameter `status` (HCP 9.4.13) | `status` | Unchanged; its one value is `ended`, which adds ended delegations to those that hold, *pending HCP confirmation* ([question 31](#pending-hcp-confirmation)). |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result `delegations` (HCP 9.4.13) | `delegations` | Unchanged. Not paginated (HCP 9.4.13). |

**Where this payload is stricter than HCP's shapes.** Each payload is closed at its top level (`additionalProperties: false`): a top-level member outside this mapping is refused, where HCP's own request and result shapes leave the top level open, so an extension member HCP would allow there is not carried. The HCP shapes nested within stay open, as HCP's are. Every size bound, and every non-empty requirement (a `minLength` or `minItems` of 1) on a member that HCP's own schema allows to be empty, is a Trust Tasks framework bound on what an `hcp/*` party writes, not an HCP requirement, as the schema descriptions say.

### Parties

| HCP role | Trust Task member |
|---|---|
| Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../../entries/query/0.1/spec.md#parties), a provider **MUST** make its VID verifiably bound to its HCP identity (its discovery document, HCP 19.1), and a Management Client **MUST** check that binding before it sends a token to a VID.

## Request

The Management Client asks for the delegations. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

- `accessToken` (REQUIRED) is the Management Client's token.
- `status` (OPTIONAL) is HCP's `status` parameter, whose only defined value is `ended`. HCP 9.4.13 returns delegations "with ended ones where `status=ended` is asked". This specification reads "with" as additive, *pending HCP confirmation* ([question 31](#pending-hcp-confirmation)): without `status`, the result holds only the delegations that hold (those with no `ended` member); with `status: "ended"`, it holds those and the delegations that have ended. A Management Client that wants only ended delegations selects those that carry `ended`.

### The person asks for every delegation, ended ones included

```json
{
  "id": "urn:uuid:9b1d3f5a-7c8e-4a0b-b2d4-6e8a0c2e4f75",
  "type": "https://trusttasks.org/spec/hcp/delegations/list/0.1#request",
  "issuer": "did:web:wallet.person.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T12:00:00Z",
  "threadId": "urn:uuid:0c2e4a6b-8d9f-4b1c-a3e5-7f9b1d3f5a86",
  "payload": {
    "accessToken": "hcp_mt_8f3a1c9e7b2d4f60",
    "status": "ended"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T12:00:00Z",
    "verificationMethod": "did:web:wallet.person.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z4Tg8hY1jU4kI7oL0pZ3xC6vB9nM2qW5eR8tY1uI4oP7aS0dF3gH6jK9lZ2xC5vB8nM1qW4eR7tY0uI3oP6a"
  }
}
```

## Response

The provider answers with the delegations. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json).

- `delegations` holds each delegation the caller may see ([Authorization](#authorization)), each a shared [`Delegation`](../../../_shared/0.1/delegation.schema.json) (HCP 17.2): its `delegationId`, `subject`, `delegate` (a natural person, with a name the Context Subject can recognize), `relationship`, `basis`, `scope`, `term`, `mayDelegate` where true, and `ended` where it has ended, with when and why (HCP 17.6). An empty `delegations` is a successful answer.
- The list is not paginated (HCP 9.4.13), so there is no `cursor` or `nextCursor`.

A Management Client **MUST** show the person `relationship`, `basis.method` and `basis.description` (HCP 17.3), and **MUST NOT** present a basis whose `method` is `unspecified` as `attested` (HCP 17.3, 17.8). Failures use `trust-task-error`, not a `#response` document (see [Errors](#errors)).

### A parent's delegation, and a grandparent's sub-delegation that has ended

```json
{
  "id": "urn:uuid:1d3f5b7c-9e0a-4c2d-a4f6-8a0c2e4f6b97",
  "type": "https://trusttasks.org/spec/hcp/delegations/list/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:wallet.person.example",
  "issuedAt": "2026-11-25T12:00:01Z",
  "threadId": "urn:uuid:0c2e4a6b-8d9f-4b1c-a3e5-7f9b1d3f5a86",
  "payload": {
    "delegations": [
      {
        "delegationId": "urn:uuid:2e4a6c8d-0f1b-4d3e-b5a7-9c1e3a5c7e08",
        "subject": { "id": "subject-7f2c" },
        "delegate": { "id": "person-parent-31", "name": "Morgan Lee" },
        "relationship": "parent-or-guardian",
        "basis": {
          "method": "attested",
          "description": "Parent on the school enrollment record, checked by the Authorization Service.",
          "establishedBy": "https://auth.provider.example",
          "establishedAt": "2025-09-01T09:00:00Z"
        },
        "scope": { "categories": ["general_preferences", "learning_context"], "actions": ["read", "write"] },
        "term": { "endsAt": "2029-04-17T00:00:00Z", "atMajority": true },
        "mayDelegate": true
      },
      {
        "delegationId": "urn:uuid:3f5b7d9e-1a2c-4e4f-86b8-0d2f4b6d8f19",
        "subject": { "id": "subject-7f2c" },
        "delegate": { "id": "person-grandparent-12", "name": "Pat Lee" },
        "relationship": "caregiver",
        "basis": {
          "method": "delegated",
          "description": "Appointed by Morgan Lee to look after learning context during travel.",
          "parent": "urn:uuid:2e4a6c8d-0f1b-4d3e-b5a7-9c1e3a5c7e08",
          "establishedBy": "https://auth.provider.example",
          "establishedAt": "2026-07-01T08:00:00Z"
        },
        "scope": { "categories": ["learning_context"], "actions": ["read"] },
        "term": { "endsAt": "2026-08-15T00:00:00Z", "atMajority": true },
        "ended": { "at": "2026-08-10T17:30:00Z", "reason": "ended-by-delegate" }
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T12:00:01Z",
    "verificationMethod": "did:web:context.provider.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z2Pq7wE0rT3yU6iO9pA2sD5fG8hJ1kL4zX7cV0bN3mQ6wE9rT2yU5iO8pA1sD4fG7hJ0kL3zX6cV9bN2mQ5w"
  }
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows.

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request` | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: the caller is not a Management Client acting for the Context Subject or a delegate (HCP 6.1.9, 9.2, 9.4.13) | `permissionDenied`, with one fixed `message` whatever the reason (HCP 5.5, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| conflict | `idempotency-conflict` | Does not arise: HCP defines it for a retried request with an idempotency key (HCP 9.6), and this operation carries none. |
| version conflict | `version-conflict` | Does not arise: a delegation has no version (HCP 17.2). |
| credential invalid | `credential-invalid` | Does not arise: it concerns a credential written as an entry's content (HCP 14.6); evidence of a basis is verified when the Authorization Service establishes the delegation, not on a read (HCP 17.3). |
| subject unavailable | `subject-unavailable` | `unavailable` |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion` |

A provider that does not conform to the Delegated Authorization Profile answers `unsupportedType` (see [Conformance](#conformance)).

## Rules carried from HCP

- **Established elsewhere.** The Authorization Service establishes a delegation and records its `basis`; the provider **MUST NOT** take `basis`, `scope`, `term` or `mayDelegate` from a request by an HCP Client (HCP 17.2). No Trust Task in this family establishes or changes a delegation.
- **Nothing is changed by reading.** HCP 8.1 lists no audit event for listing delegations.
- **Evidence held whole.** Evidence that is a Verifiable Credential is held whole (HCP 17.3, 4.2.2) and is returned whole here, so that the person can see what the authority over them rests on.
- **What the basis is not.** The basis records how authority was established; it is not a judgment of the delegate's fitness (HCP 17.3). A Management Client **MUST NOT** present it as one.
- **Converted delegations.** A delegation converted from HCP v1 has `basis.method` `unspecified`, authorizes no new grant, and is shown to the Context Subject as unspecified (HCP 17.8).

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 1 — conformance claims.** HCP defines no conformance identifier for Trust Task carriage, so a provider **MUST NOT** claim this carriage in its HCP `conformsTo`; it advertises the task with [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead. The binding identifier for the carriage is the same question as [question 37](#pending-hcp-confirmation): whether a carriage other than the HTTP Binding may carry governance operations at all.
- **Question 22 — the client's identity.** The `issuer` VID, authenticated by the request's `proof`, identifies the HCP client the token was issued to; a token issued to any other client **MUST** be refused with `permissionDenied`. This token binding is a carriage rule of this family, a check HCP does not itself make (see [The access token](#the-access-token) and [Conformance](#conformance)).
- **Question 31 — `status=ended`.** Additive: without `status`, only the delegations that hold; with `status: "ended"`, those and the delegations that have ended (see [Request](#request)).
- **Question 32 — a delegate's sub-delegations.** HCP 17.10 governs where HCP 9.4.13 is narrower: a delegate's list also carries each sub-delegation they appointed, so that they can end it with [`hcp/delegations/end`](../../end/0.1/spec.md) (`ended-by-delegate`, HCP 17.6, 17.10).
- **Question 37 — governance over a carriage other than the HTTP Binding.** HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1). This specification takes it that another carriage, here a Trust Task, may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and applies every rule HCP places on the operation unchanged (see [Conformance](#conformance)).
- **Question 53 — a request proof on every request that carries a token.** A request `proof` is **REQUIRED** on every `hcp/*` request that carries `accessToken`, so that the token binding of question 22 always applies and a token lifted from a stored or forwarded document cannot be replayed in an unsigned request. This is a carriage rule of this family, not HCP's, and is stricter than HCP's bearer model, under which presenting a valid token is enough (HCP 9.3); see [The access token](#the-access-token).
- **Question 55 — HCP's grant and delegation schemas disagree.** HCP's delegation schema does not require a delegation's `scope.categories` to be unique, where its grant schema requires a grant's categories to be. The shared `Delegation` follows HCP's delegation schema and does not require it either.

## What HCP requires that a document cannot express

- The provider-held delegation records, and which of them the caller may see (HCP 9.4.13).
- Establishment by the Authorization Service, and the verification of any evidence (HCP 17.2, 17.3).
- What a Management Client must show the person about a basis (HCP 17.3, 17.8).
- Carriage by the HTTP Binding: HCP carries governance operations only over that binding (HCP 1.6, 5.4, 9.1), and a Trust Task is not it. This specification carries the operation for a Management Client acting for the Authorizing Principal (HCP 6.1.9) instead, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

## Security & Privacy

### Data carried

The request carries the Management Client's access token, a bearer secret that **MUST** travel only to the provider. The response carries who may act for the person and why: the delegate's identifier and name, the relationship, the basis, which may hold a credential of guardianship, a court order or a power of attorney whole, the High-sensitivity categories the basis covers, and when the delegation ends, which for a minor reveals when they come of age. A producer **MUST NOT** put the token or any part of a delegation in `ext` or in any `message`.

### The access token

`accessToken` is a bearer secret: whoever presents it exercises the authority it evidences. It **MUST** travel only to the provider. A recipient **MUST NOT** log `accessToken` or retain it beyond the exchange.

**Token binding, a carriage rule of this family.** Every request that carries `accessToken` carries a `proof`, *pending HCP confirmation* ([question 53](#pending-hcp-confirmation)), and the provider **MUST** refuse with `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies, so that a token lifted from a stored or forwarded document is of no use from another VID. HCP validates the token alone and does not bind it to a VID (HCP 9.3): this rule is the `hcp/*` family's, not HCP's, and rests on the `issuer` VID identifying the HCP client, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)). It is a check HCP does not make (see [Conformance](#conformance)).

**Retained copies.** The provider verifies the request's `proof` over the document as received, at receipt, before it removes anything ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). A provider that retains the request document, as a duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) or as evidence of the request, **MUST** remove `accessToken` from the copy it keeps and record in its place `accessTokenDigest`: the SHA-256 multihash of the token's UTF-8 bytes, multibase-encoded as base64url without padding (prefix `u`), the framework's `DigestMultibase` form ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)). The kept copy's `proof` no longer verifies over it, so the provider records with the copy that the proof was verified at receipt: when, under which verification method, and with what result. It relies on that record, and **MUST NOT** verify the proof again over the kept copy. A record that keeps only a digest of the document, such as its task digest ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)), keeps no token either.

### Correlation

A `delegationId` is the same in every grant made under the delegation, which is why HCP never shows it to a Context Client (HCP 17.4); the identifiers in this response let anyone who holds it link the person's grants. A Management Client **MUST NOT** pass the response to anyone but the person it acts for, unless that person asks it to. A delegate sees only their own delegations and the sub-delegations they appointed, whose delegates they chose, and so learns nothing of other delegates (HCP 9.4.13, 17.10). `threadId` **SHOULD** be fresh per request.

### Retention

The provider records nothing for this task. The Management Client keeps the list only as long as it presents it, unless the person asks it to keep a copy; the signed list is then evidence of what the provider held, and evidence it carries stays as sensitive as the originals.

### Consent/purpose

Reading a delegation is the right of the Context Subject and of the delegate (HCP 17.2, 18.4), so that each knows who may act for the person and within what limits. No further approval is needed, and whether a Management Client asks the person again before each read is its own policy ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).
