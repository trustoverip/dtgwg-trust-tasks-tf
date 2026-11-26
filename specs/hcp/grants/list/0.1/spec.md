---
slug: hcp/grants/list
version: "0.1"
title: "HCP — List Grants"
summary: A Management Client, acting for the person or a delegate, lists the Human Context Protocol grants made for the person, filtered by status; carries the HCP 2.0-draft-3 List grants operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: List grants
category: access-control
keywords: [hcp, human-context-protocol, grant, consent, governance]
parties:
  - role: Management Client (HCP), acting for the Authorizing Principal
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
    the binding. The request proof is REQUIRED, as on every hcp/* request that carries a token, because it is what
    lets the provider refuse a token sent from a VID other than the client's (a carriage rule pending HCP confirmation, question 53). The response is RECOMMENDED to be signed so the person can show
    which grants the provider reported as active, and when, if a client later acts outside what was listed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task is not consequential, but its answer describes the present state of the person's grants, which the
    person relies on to decide what to revoke; requiring `issuedAt` lets the provider refuse a stale or replayed
    request by its acceptance window.
sideEffects:
  level: none
  rationale: >-
    A read. No audit event is required for reading a grant list (HCP 6.1.5), and the provider changes nothing.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: secret
  rationale: >-
    The response enumerates authorization records: which clients and operators the person has granted access to,
    for which categories and purposes, under which per-client pseudonyms. That is sensitive descriptive data about
    the person's relationships, but it holds no context and no credential that grants access to anything, and it is
    returned only to the person's own Management Client (or, to a Context Client, only its own grant), so it is metadata, not secret. The request carries the
    Management Client's access token, a bearer secret.
retention:
  class: transient
  rationale: >-
    The provider keeps nothing of the request: no audit event is required for reading a grant list (HCP 6.1.5). The
    Management Client shows the list to the person and need not keep it; the grants themselves stay with the provider.
maxDocumentBytes:
  response: 4194304
  rationale: >-
    A page of up to 1000 grant objects, each with its client, operator, authorization details, purposes and any
    narrowing and onward sharing; a typical grant is 1 to 2 KiB, so 4 MiB holds a full page of grants well above
    that size, with envelope and proof. This is a framework bound, not an HCP requirement; HCP sets no size limit. A
    provider ends the page early and returns `nextCursor` where a full page would not fit.
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence", here the Management
      Client's access token.
    retryable: false
uncheckedRules:
  - member: "/$defs/Response/properties/grants/items/properties/expiresAt"
    rule: "`expiresAt` is an instant for a `temporary` grant and null for a `persistent` one."
    source: "HCP 6.1.4, 6.1.5"
  - member: "/$defs/Response/properties/grants/items/properties/access/items/properties/onwardSharing"
    rule: "`onwardSharing` appears only in a detail that includes `read` and all of whose categories have a default sensitivity of Low."
    source: "HCP 6.1.5"
  - member: "/$defs/Response/properties/grants/items/properties/access/items/properties/actions"
    rule: "`provenance` is an action only alongside `read`."
    source: "HCP 6.1.5, 7.4.1"
related:
  - hcp/grants/revoke
  - hcp/audit/read
  - hcp/disclosures/summary
---

## Abstract

The **HCP — List Grants** Trust Task carries the List grants operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 5.4 and 6.1.5; `GET /v2/grants` with a Management Client's token in HCP 9.4.6). A *Management Client*, acting for the *Authorizing Principal*, asks an *HCP Provider* for the grants made for the person, filtered by status; the default is the active ones. The provider answers with grant objects, a page at a time where it paginates. It is how the person sees, in one place, every client that may reach their context, before revoking any with [`hcp/grants/revoke`](../../revoke/0.1/spec.md).

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on grant representation and grant access; the ones that bear on the document are restated below, unchanged. Conformance to HCP is HCP's ([SPEC §9.4](/SPEC.md#94-carried-protocols)): a provider **MUST NOT** list a conformance identifier for this carriage in its HCP `conformsTo`, and advertises the task through `trust-task-discovery`, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)). HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1); this specification takes it that a carriage other than that binding may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and carries this one as a Trust Task on that reading, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

This specification adds one check HCP does not make: the token binding of [The access token](#the-access-token), a carriage rule of the `hcp/*` family, not HCP's. [SPEC §9.4](/SPEC.md#94-carried-protocols) item 6 places a check the carried protocol does not make in a separate specification that does not declare `carries`. This family keeps this one in the carriage, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)), because it protects the token that the carriage itself moves from the HTTP `Authorization` header into the document, where it can be stored or forwarded with the document; a separate task could not protect the document this one carries.

## Authorization

The authority is **the Authorizing Principal's own authority over their grants**, which a Management Client exercises for them (HCP 5.4, 6.1.5, 6.1.9). It is evidenced by the Management Client's access token in `accessToken`, which the provider validates exactly as over HCP's HTTP binding. Listing grants is a governance operation: a grant held by a Context Client never authorizes it, whatever its scopes (HCP 6.1.9), and any caller that is neither a Management Client acting for the Context Subject or a delegate, nor a Context Client presenting its own token, receives not authorized, carried as `permissionDenied`. HCP names only its HTTP Binding as the carriage for a governance operation (HCP 5.4, 9.1); this specification accepts the operation as a Trust Task from a Management Client acting for the Authorizing Principal, and from no other caller (HCP 6.1.9), *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

Over HCP's HTTP binding, `GET /v2/grants` called with a Context Client's token returns the one grant under which that token was issued (HCP 9.4.6). This task does the same, *pending HCP confirmation* ([question 19](#pending-hcp-confirmation)): a request whose `accessToken` is a Context Client's token is answered with a `grants` list holding only that grant, where it has the `status` asked for, and otherwise an empty list; `nextCursor` is never present. The grant is returned as Read own grant returns it (HCP 5.2.6), with `authorizingPrincipal.delegation` withheld (HCP 17.4); a Context Client that also wants the directives that apply to its grant uses [`hcp/grants/read-own`](../../read-own/0.1/spec.md). The provider never returns a grant issued to one HCP Client to any other: an HCP Provider **MUST NOT** disclose a grant issued to one HCP Client to any other HCP Client (HCP 6.1.5).

Under the Delegated Authorization Profile, the Context Subject's Management Client is listed every grant made for them, whoever authorized it (HCP 6.1.5, 17.5). A delegate's Management Client is listed only the grants every one of whose `access` details' categories lies within the delegation's scope, which includes every grant the delegate authorized under it, *pending HCP confirmation* ([question 20](#pending-hcp-confirmation)); HCP 9.4.6 says "all grants", and HCP 17.4 limits a delegate's governance operations to the delegation's scope.

The `proof` establishes which Management Client VID sent the document and that it is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). It is not the authorization: the token is. The request always carries a proof, *pending HCP confirmation* ([question 53](#pending-hcp-confirmation)), so a token issued to a client other than the one the `issuer` VID identifies is always refused with `permissionDenied` (see [The access token](#the-access-token)).

## Definitions

- **Management Client**, **Context Client**, **HCP Client**, **HCP Provider**, **Context Subject**, **Authorizing Principal**, **Client Operator**, **delegate**, **delegation** and **grant** have their HCP 2.0-draft-3 meanings.
- The grant's shape is [`Grant`](../../../_shared/0.1/grant.schema.json) in `hcp/_shared/0.1`. It is open, as HCP's own shape is: a member it does not declare is accepted, never rejected.
- HCP defines no request or result schema for this operation. The members come from HCP 9.4.6 and 9.5.

### HCP field mapping

| HCP field | Member | Notes |
|---|---|---|
| query parameter `status` (HCP 9.4.6) | `status` | Unchanged. Absent means `active`, HCP's default. |
| query parameter `cursor` (HCP 9.5) | `cursor` | Unchanged. |
| query parameter `limit` (HCP 9.5) | `limit` | Unchanged. At most 1000, a framework bound HCP does not set, *pending HCP confirmation* ([question 3](#pending-hcp-confirmation)). |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| response body, the grant objects (HCP 9.4.6) | `grants` | HCP's binding names no member for the list; here it is `grants`. |
| response `nextCursor` (HCP 9.5) | `nextCursor` | **Absent, not null,** when the set is exhausted, *pending HCP confirmation* ([question 3](#pending-hcp-confirmation)). |

**Where this payload is stricter than HCP's shapes.** Each payload is closed at its top level (`additionalProperties: false`): a top-level member outside this mapping is refused, where HCP's own request and result shapes leave the top level open, so an extension member HCP would allow there is not carried. The HCP shapes nested within stay open, as HCP's are. Every size bound, and every non-empty requirement (a `minLength` or `minItems` of 1) on a member that HCP's own schema allows to be empty, is a Trust Tasks framework bound on what an `hcp/*` party writes, not an HCP requirement, as the schema descriptions say.

### Parties

| HCP role | Trust Task member |
|---|---|
| Management Client (or a Context Client, listed only its own grant; see [Authorization](#authorization)) | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../../entries/query/0.1/spec.md#parties), a provider **MUST** make its VID verifiably bound to its HCP identity (its discovery document, HCP 19.1), and a Management Client **MUST** check that binding before it sends a token to a VID.

## Request

The Management Client asks the provider for the grants. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

- `status` (OPTIONAL) lists only grants with that status: `active`, `revoked` or `expired`. Absent means `active` (HCP 9.4.6). HCP offers no value for every status; a Management Client that wants them all asks once for each.
- `cursor` (OPTIONAL) is the `nextCursor` of the previous page; absent asks for the first page. A Management Client **MUST** send the same `status` with a cursor as with the request that produced it.
- `limit` (OPTIONAL) is the most grants the Management Client wants in this page, from 1 to 1000. Absent leaves the page size to the provider.
- `accessToken` (REQUIRED) is the Management Client's token.

### Listing the person's active grants

```json
{
  "id": "urn:uuid:c17da410-d108-4db8-8956-6db1215eb41d",
  "type": "https://trusttasks.org/spec/hcp/grants/list/0.1#request",
  "issuer": "did:web:wallet.person.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T11:00:00Z",
  "threadId": "urn:uuid:49076a5c-39db-4fdb-af72-97577bf18f8b",
  "payload": {
    "status": "active",
    "limit": 50,
    "accessToken": "hcp_mt_8f3a1c9e7b2d4f60"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T11:00:00Z",
    "verificationMethod": "did:web:wallet.person.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "zMV4MVy1VnEV9srVQSbGxVHgwoZ3kEw1H4S2SyqRRKenkStDsYUnmMYAPCyWTWXgKgvBT7ANqdG2JMuxtWCwsPD"
  }
}
```

## Response

The provider answers with a page of grants. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json).

- `grants` holds the grant objects with the status asked for (HCP 6.1.5, 9.4.6), at most `limit` of them and never more than 1000. An empty `grants` is a successful answer: there are none. Where the grants that remain would take the response past its `maxDocumentBytes` (4 MiB, a framework bound), the provider **MUST** paginate: it returns fewer grants than `limit` asked for, or than its own page size, so that the response fits, and returns `nextCursor` for the rest. It **MUST NOT** drop a grant to fit, or refuse the request because the set is large.
- `nextCursor` is present where more grants remain, and absent when the set is exhausted, *pending HCP confirmation* ([question 3](#pending-hcp-confirmation)). It is opaque: the Management Client **MUST NOT** interpret it, and the provider **MUST NOT** make it a value from which the size or position of the set can be read, **MUST NOT** encode a credential or entry content in it, and **MUST** keep it valid for at least 300 seconds (HCP 9.5).

Failures use `trust-task-error`, not a `#response` document (see [Errors](#errors)).

### One active grant, to a booking agent

```json
{
  "id": "urn:uuid:cb15454f-4c25-4812-abec-fac450b495a3",
  "type": "https://trusttasks.org/spec/hcp/grants/list/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:wallet.person.example",
  "issuedAt": "2026-11-25T11:00:01Z",
  "threadId": "urn:uuid:49076a5c-39db-4fdb-af72-97577bf18f8b",
  "payload": {
    "grants": [
      {
        "grantId": "urn:uuid:6f3157c9-8481-423e-a0b7-ea9c4ecad060",
        "subject": { "id": "hcp-sub-4f2a91c7" },
        "authorizingPrincipal": { "id": "hcp-sub-4f2a91c7" },
        "client": {
          "id": "booking-agent",
          "name": "TableFinder booking assistant",
          "operator": { "id": "booking.example", "name": "TableFinder Ltd" }
        },
        "access": [
          {
            "type": "https://hcp.example/spec/authorization/context-access",
            "actions": ["read"],
            "categories": ["general_preferences", "health_context"],
            "purposes": [
              {
                "code": "https://hcp.example/purposes/reservation",
                "description": "Book restaurant tables that can serve your diet."
              }
            ]
          }
        ],
        "scopes": ["hcp:general_preferences:read", "hcp:health_context:read"],
        "grantType": "temporary",
        "issuedAt": "2026-11-24T09:30:00Z",
        "expiresAt": "2026-11-25T13:30:00Z",
        "status": "active"
      }
    ]
  }
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows. The provider uses the code in the right-hand column. It never sends HCP's Core error object or the HTTP binding's problem details (HCP 9.7).

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request`: the request fails schema validation, or `cursor` is not one the provider issued, has expired, or was issued for another `status` | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: the caller is neither a Management Client acting for the Context Subject or a delegate (HCP 6.1.9), nor a Context Client presenting its own token, which is answered with its own grant (see [Authorization](#authorization)) | `permissionDenied`, with one fixed `message` whatever the reason (HCP 5.5, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| conflict | `idempotency-conflict` | Does not arise: HCP defines it for a retried request with an idempotency key (HCP 9.6), and this operation carries none. A re-delivered document is answered under [Repeated requests](#repeated-requests). |
| version conflict | `version-conflict` | Does not arise: a grant has no version (HCP 6.1.5). |
| credential invalid | `credential-invalid` | Does not arise: it concerns a credential written as an entry's content (HCP 14.6). |
| subject unavailable | `subject-unavailable` | `unavailable`. The Management Client may retry later (HCP 5.5). |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion`. The provider never serves a different HCP version silently (HCP 5.5). |

A Management Client that does not know `hcp:invalidToken` reads it as `taskFailed` ([SPEC §8.5](/SPEC.md#85-extension-by-individual-trust-task-specifications)) and still honours `retryable: false`.

## Rules carried from HCP

- **A recognizable name.** Each grant's `client.name` **MUST** be a name the person can recognize; a grant list presenting only opaque client identifiers does not satisfy HCP 18 (HCP 6.1.5).
- **Everything the person approved is shown.** Each grant carries the `purposes` the person approved for each authorization detail (HCP 6.1.5.2), any narrowing below category grain in `narrowedTo` (HCP 6.1.5.1), and any `onwardSharing`, with the `notTo` operators standing rules deny, kept current as rules change (HCP 6.1.5, 6.3). A Management Client presenting the list **MUST** show narrowing and onward sharing (HCP 6.1.5, 18).
- **Delegations are shown to the person.** A grant a delegate authorized carries `authorizingPrincipal.delegation` (HCP 6.1.5, 17.4). It is withheld only from a Context Client reading its own grant (HCP 17.4), including through this task (see [Authorization](#authorization)), and never from a Management Client's list.
- **No cross-client disclosure.** An HCP Provider **MUST NOT** disclose a grant issued to one HCP Client to any other HCP Client (HCP 6.1.5).
- **No audit event.** No audit event is required for reading a grant list: a person reading their own grant list is not an access to entry content (HCP 6.1.5).
- **`expiresAt`.** A temporary grant carries its absolute expiry; a persistent grant carries `expiresAt: null`, as HCP writes it (HCP 6.1.4, 6.1.5). The `null` is HCP's own value inside the grant object, carried unchanged.

## Repeated requests

The task is a read with no side effect, so a re-delivered document may simply be answered again; the answer reflects the grants at the time it is made. A Management Client paging through the list uses a new document, with a new `id`, for each page.

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 1 — conformance claims.** HCP defines no conformance identifier for Trust Task carriage, so a provider **MUST NOT** claim this carriage in its HCP `conformsTo`; it advertises the task with [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead. The binding identifier for the carriage is the same question as [question 37](#pending-hcp-confirmation): whether a carriage other than the HTTP Binding may carry governance operations at all.
- **Question 3 — pagination.** The request takes `cursor` and `limit`, `limit` capped at 1000 by the carriage, and the response's `nextCursor` is absent, not null, when the set is exhausted; the cursor stays opaque and valid for at least 300 seconds (HCP 9.5).
- **Question 19 — a Context Client's token.** A Context Client presenting its own token is listed only the grant under which that token was issued, as `GET /v2/grants` answers it over HCP's HTTP binding (HCP 9.4.6), rather than refused as a caller of a governance operation (HCP 6.1.9) (see [Authorization](#authorization)).
- **Question 20 — a delegate's grant list.** A delegate's Management Client is listed only the grants every one of whose `access` details' categories lies within the delegation's scope (see [Authorization](#authorization)).
- **Question 22 — the client's identity.** The `issuer` VID, authenticated by the request's `proof`, identifies the HCP client the token was issued to; a token issued to any other client **MUST** be refused with `permissionDenied`. This token binding is a carriage rule of this family, a check HCP does not itself make (see [The access token](#the-access-token) and [Conformance](#conformance)).
- **Question 37 — governance over a carriage other than the HTTP Binding.** HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1). This specification takes it that another carriage, here a Trust Task, may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and applies every rule HCP places on the operation unchanged (see [Conformance](#conformance)).
- **Question 53 — a request proof on every request that carries a token.** A request `proof` is **REQUIRED** on every `hcp/*` request that carries `accessToken`, so that the token binding of question 22 always applies and a token lifted from a stored or forwarded document cannot be replayed in an unsigned request. This is a carriage rule of this family, not HCP's, and is stricter than HCP's bearer model, under which presenting a valid token is enough (HCP 9.3); see [The access token](#the-access-token).
- **Question 55 — HCP's grant and delegation schemas disagree.** HCP's grant schema requires a non-empty `client.name`, where its common `clientDescription`, which an entry's provenance uses, does not. The shared `Grant` follows HCP's grant schema and requires it, as does the shared `ClientDescription`.

## What HCP requires that a document cannot express

- The provider-held grant objects and their statuses (HCP 6.1.5), including `notTo`, kept current as standing rules change (HCP 6.3).
- The scope check that limits a delegate's list (HCP 17.4).
- That a person-facing presentation shows `client.name`, narrowing and onward sharing (HCP 6.1.5, 18): that is on the Management Client.
- Cursor opacity and lifetime (HCP 9.5).
- Carriage by the HTTP Binding: HCP carries governance operations only over that binding (HCP 1.6, 5.4, 9.1), and a Trust Task is not it. This specification carries the operation for a Management Client acting for the Authorizing Principal (HCP 6.1.9) instead, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

## Security & Privacy

### Data carried

The request carries a status filter, a cursor and the Management Client's access token, a bearer secret that **MUST** travel only to the provider. The response carries grant objects: the clients and operators the person has authorized, the categories and purposes each may reach, which may name High-sensitivity categories such as `health_context` and so reveal, for instance, that the person deals with a health service, any delegate who authorized a grant, and the per-client pseudonym under which each client knows the person. It carries no context and no token. A producer **MUST NOT** put grant details or the token in `ext` or in any `message`.

### The access token

`accessToken` is a bearer secret: whoever presents it exercises the authority it evidences. It **MUST** travel only to the provider. A recipient **MUST NOT** log `accessToken` or retain it beyond the exchange.

**Token binding, a carriage rule of this family.** Every request that carries `accessToken` carries a `proof`, *pending HCP confirmation* ([question 53](#pending-hcp-confirmation)), and the provider **MUST** refuse with `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies, so that a token lifted from a stored or forwarded document is of no use from another VID. HCP validates the token alone and does not bind it to a VID (HCP 9.3): this rule is the `hcp/*` family's, not HCP's, and rests on the `issuer` VID identifying the HCP client, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)). It is a check HCP does not make (see [Conformance](#conformance)).

**Retained copies.** The provider verifies the request's `proof` over the document as received, at receipt, before it removes anything ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). A provider that retains the request document, as a duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) or as evidence of the request, **MUST** remove `accessToken` from the copy it keeps and record in its place `accessTokenDigest`: the SHA-256 multihash of the token's UTF-8 bytes, multibase-encoded as base64url without padding (prefix `u`), the framework's `DigestMultibase` form ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)). The kept copy's `proof` no longer verifies over it, so the provider records with the copy that the proof was verified at receipt: when, under which verification method, and with what result. It relies on that record, and **MUST NOT** verify the proof again over the kept copy. A record that keeps only a digest of the document, such as its task digest ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)), keeps no token either.

### Correlation

The per-client pseudonyms in `subject` and `authorizingPrincipal` exist so that two clients cannot link the person (HCP 3.4); the list puts them side by side. That is safe only because the list goes to the person's own Management Client, and a Management Client **MUST NOT** pass the list, or any pseudonym in it, to any client. The `delegation` identifier is the same in every grant made under one delegation (HCP 17.4), and is shown here for the same reason. `threadId` **SHOULD** be fresh per listing; a cursor joins the pages of one listing and nothing else.

### Retention

The provider keeps nothing of the request (HCP 6.1.5). The Management Client shows the list to the person and **SHOULD NOT** keep it longer than that display needs; the grants remain at the provider, and the list goes stale as grants expire or are revoked.

### Consent/purpose

Listing grants is how the person exercises their right to view, and then revoke, every active grant (HCP 6.1.5, 18). The purposes shown are those the person approved; this task changes none of them. Whether the Management Client asks the person before listing is its own policy ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).
