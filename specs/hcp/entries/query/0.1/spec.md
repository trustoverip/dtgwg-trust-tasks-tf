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
  - role: Context Client (HCP)
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
    provider released which context, when, and with which use limits, which is what the person's audit trail and
    any later dispute turn on. A request proof is RECOMMENDED, not REQUIRED, because the grant is evidenced by the
    access token, which the provider checks on every request whatever the binding.
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
    categories. The request carries the HCP access token, a bearer secret, and query text that HCP treats as being
    as sensitive as content (HCP 8.2).
retention:
  class: durable
  rationale: >-
    The provider keeps the `context.read` audit event HCP requires for every read (HCP 8.1), so the person can see
    who read what. The query text itself is not logged (HCP 8.2), and the client keeps disclosed copies only until
    each one's `useUntil`.
maxDocumentBytes:
  response: 2097152
  rationale: >-
    Up to 20 disclosures, each of which may carry a verifiable credential as evidence, plus directives, envelope and
    proof. This is a framework bound, not an HCP requirement; HCP sets no size limit.
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      The access token is missing from the provider's records, expired, revoked, or otherwise not valid for this
      provider (HCP 5.5 `invalid-token`). A grant that was revoked surfaces here, and that failure is itself a
      notice of revocation to the client (HCP 6.2.2).
    retryable: false
related:
  - hcp/grants/revoke
  - hcp/grants/revoked
---

## Abstract

The **HCP — Query Context** Trust Task carries the Query operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 5.2.1). A *Context Client* asks an *HCP Provider* for the context about a person that is relevant to a query, for a named purpose, under the grant its access token stands for. The provider answers with at most 20 disclosed entries, each with a use limit, together with the directives the person set and the provider's account of how it minimized the result.

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)): it carries one HCP operation, unchanged, so that a Context Client can use it over any Trust Task binding, such as TSP, as well as over HCP's own HTTP and MCP bindings.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage. [SPEC §9.4](/SPEC.md#94-carried-protocols) says a protocol's owner should publish or endorse its family, and [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications) lets them do so under their own authority. This family is a proposal to them. When HCP changes the Query operation, this specification moves to a new version.

HCP's text is © its authors under CC BY 4.0. This specification and the shared `hcp/_shared/0.1` schemas were written from that text, with attribution, and do not copy HCP's own Apache-2.0 schema files. Where they differ from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on the Query operation; this specification restates the ones that bear on the document and does not relax any. Conformance to HCP is HCP's: serving this task does not make a provider conform to an HCP binding, and HCP defines no conformance identifier for Trust Task carriage, so a provider **MUST NOT** list one in its HCP `conformsTo`.

## Authorization

The authority is **the grant** the person gave the client: a category-scoped, purpose-bound and revocable HCP grant (HCP 6.1), held by the provider and evidenced by the access token in `accessToken`. The provider **MUST** check the grant on every request (HCP 6.2.2), exactly as it does over HCP's HTTP binding: the token must be valid, and the requested `categories` and `purpose` must be within the grant (HCP 5.2.1).

Possession of a valid token is the evidence; neither the document's `proof` nor the `issuer` VID is. Verifying the proof or the VID establishes who sent the document and that it is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); it does not establish the grant. A provider that knows which VID it registered the client under **SHOULD** check that the `issuer` is that client, so a stolen token sent from another VID is refused with `permissionDenied`; HCP itself does not bind the token to a VID.

A Management Client token does not authorize this task: querying context is a Context Client operation (HCP 5.1).

## Definitions

- **Context Client**, **Management Client**, **HCP Provider**, **Authorizing Principal**, **Context Subject**, **grant**, **entry**, **disclosure**, **directive** and **category** have their HCP 2.0-draft-3 meanings.
- The shared shapes are in [`hcp/_shared/0.1`](../../../_shared/0.1/common.schema.json): [`Disclosure`](../../../_shared/0.1/disclosure.schema.json), [`Entry`](../../../_shared/0.1/context-entry.schema.json) and [`Directive`](../../../_shared/0.1/directive.schema.json). They are open, as HCP's objects are (HCP Appendix A.3): a member HCP defines that those schemas do not declare is preserved, never rejected.

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

### Parties

| HCP role | Trust Task member |
|---|---|
| Context Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

HCP identifies a provider by the `provider` URL and `jwks_uri` in its discovery document (`/.well-known/hcp-configuration`, HCP 19.1), and a client by its registration. Neither is a VID. A provider serving this task **MUST** make its VID verifiably bound to its HCP identity: its DID document lists the discovery document as a service, or its discovery document names the VID, or both. A client **MUST** check that binding before it sends a token to a VID.

## Request

The Context Client sends the query to the provider. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

- `query` (REQUIRED) is what the client wants context about.
- `categories` (OPTIONAL) narrows the search. Absent means every category the grant covers.
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
  }
}
```

## Response

The provider answers with the disclosed context. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json).

- `entries` holds at most the lesser of `maxEntries` and 20 entries, ordered by the provider's declared minimization method (HCP 7.2). Each is a `Disclosure`, carrying a grant-scoped `ref` instead of the entry's stable `id` (HCP 7.4) and a `useUntil` after which the client **MUST NOT** use the copy (HCP 7.6). An `Entry` appears only where HCP lets the reader see the entry itself. An empty `entries` is a successful answer: nothing matched, or nothing could be disclosed.
- `retrievalId` identifies this retrieval in the provider's audit log.
- `minimization` is the provider's account of how it reduced the result.
- `categoriesSearched` lists the categories searched.
- `directives` (OPTIONAL) are the person's directives that apply to the disclosed context (HCP 6.4). The client **MUST** honour them.
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
        "useUntil": "2026-11-25T10:00:01Z",
        "content": { "value": "Vegetarian; eats fish occasionally.", "language": "en" },
        "assertion": { "declarationType": "person-declared" },
        "classification": { "categories": ["general_preferences"] },
        "record": { "status": "active" }
      },
      {
        "ref": "hcp-ref-5e02bd",
        "useUntil": "2026-11-24T22:00:01Z",
        "content": { "value": "Severe peanut allergy.", "language": "en" },
        "assertion": { "declarationType": "person-declared" },
        "classification": { "categories": ["health_context"] },
        "record": { "status": "active" }
      }
    ],
    "retrievalId": "urn:uuid:9a1e5c73-2d4b-4e8f-a6c1-7b0d3e2f1a99",
    "minimization": { "method": "relevance" },
    "categoriesSearched": ["general_preferences", "health_context"],
    "directives": [
      { "action": "share", "effect": "deny" }
    ],
    "respondedAt": "2026-11-24T10:00:01Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-24T10:00:01Z",
    "verificationMethod": "did:web:context.provider.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3FXQjecWufY46yg5abdVZsXqLhxhueuSoZgNSARiKBk9czhSePTFehP8c3PGfb6a22gkfUKKPWtsuNYYHNXS5d2"
  }
}
```

### A revoked grant

The token's grant was revoked, so the provider refuses the query. The client treats this as notice of the revocation (HCP 6.2.2).

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

HCP's Core errors (HCP 5.5) map to Trust Task errors as follows. The provider uses the code in the right-hand column; it never sends HCP's `problem+json` body.

| HCP condition (5.5) | Trust Task error |
|---|---|
| `malformed-request` | `malformedRequest` |
| `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| `scope-denied`, including a category or purpose outside the grant (HCP 5.2.1) | `permissionDenied`, with one fixed `message` whatever the reason, so the error does not reveal whether out-of-scope context exists (HCP 5.5, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| `subject-unavailable` | `unavailable` |
| `rate-limited` | `unavailable`, with `retryAfter` |
| `unsupported-version` | `unsupportedVersion` |

A client that does not know `hcp:invalidToken` reads it as `taskFailed` ([SPEC §8.5](/SPEC.md#85-extension-by-individual-trust-task-specifications)) and still honours `retryable: false`.

## Repeated requests

HCP requires a `context.read` audit event for every query (HCP 8.1). Because this task is consequential, a consumer applies [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements): a re-delivered document with the same `id` and content is not executed twice, so it produces no second disclosure and no second `context.read` event. This is how carriage reads HCP 8.1. HCP's own no-duplicate rule (HCP 9.6) is written for create and revise only, and the HCP editors have not yet said whether a re-delivered query may skip a second event. A provider **MUST** retain its duplicate-execution record for at least the acceptance window it applies to `issuedAt`.

## What HCP requires that a document cannot express

- **The grant check on every request** (HCP 6.2.2), and the scope checks of HCP 5.2.1: they are made against provider-held state.
- **Minimization** (HCP 7.2): the provider decides which entries to disclose and in what order; the document only carries the result.
- **Use limits** (HCP 7.6): `useUntil` is carried, but the client's duty to stop using a copy after it, and not to pass copies on (HCP 3.12), is on the client.
- **Directives** (HCP 6.4): carried with every result; honouring them is on the client.
- **Pseudonymous subject identifiers** (HCP 3.4) and grant-scoped `ref`s (HCP 7.4): minted by the provider.
- **Cryptosuite.** HCP requires `eddsa-rdfc-2022` for credentials it verifies (HCP 14.5). That concerns credentials inside entries, not this document's `proof`, which follows [SPEC §4.7.1](/SPEC.md#471-cryptosuites).

## Security & Privacy

### Data carried

The request carries the access token, a bearer secret, and the query text, which HCP treats as being as sensitive as content (HCP 8.2). The response carries the person's context: up to 20 disclosures that may include High-sensitivity categories such as `health_context`, and credentials as evidence. A producer **MUST NOT** put context, query text or the token in `ext` or in any `message`. The smallest answer is an empty `entries`; the provider, not the client, decides how much to disclose, and a client asks for no more than its purpose needs, through `categories` and `maxEntries`.

Because the response proof is REQUIRED ([SPEC §7.3 item 8](/SPEC.md#73-specification-requirements)), each disclosure reaches the client as a signed assertion that any third party can verify. That makes the copy more valuable to leak or resell. HCP bars onward passing (HCP 3.12, 6.2.2), and a client **MUST NOT** pass the signed response, or any part of it, to anyone the grant does not cover.

### Correlation

The provider sees which client queried, when, for which purpose and categories; that is inherent in a grant check and is what the audit log records. The client sees the person only under a per-client pseudonym (HCP 3.4) and entries only by grant-scoped `ref`s (HCP 7.4), so two clients cannot join their copies on an identifier HCP mints. The access token is stable for the life of the grant and joins every query made under it; it must travel only to the provider. `threadId` joins a request to its response and **SHOULD** be fresh per query. Response size can reveal how much context matched; a provider **MAY** pad the response.

### Retention

The provider keeps the `context.read` audit event (HCP 8.1) for as long as HCP and its own policy require, readable only by the Context Subject (HCP 8.3). It does not log the query text (HCP 8.2). The client keeps each disclosed copy only until its `useUntil` (HCP 7.6), and deletes it, and every copy derived from it, on revocation (HCP 6.2.2). The signed response is evidence of what the provider released; a client that keeps it as evidence keeps it apart from the context it carries, which still expires.

### Consent/purpose

The context is disclosed for the `purpose` named in the request, which must be one the grant allows, and for no other. The person sets the grant and the directives; this task only carries a use of them. Whether a given query needs any further approval is the provider's policy under HCP, not a requirement of this specification ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).

The **acceptance window** for `issuedAt` is consumer policy; a provider **SHOULD** apply a short one, minutes rather than hours, since a query answers the present.
