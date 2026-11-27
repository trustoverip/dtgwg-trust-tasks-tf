---
slug: hcp/audit/read
version: "0.1"
title: "HCP — Read Audit Log"
summary: A Management Client, acting for the person or a delegate, reads one page of the Human Context Protocol audit log, filtered as asked; carries the HCP 2.0-draft-3 Read audit log operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Read audit log
category: access-control
keywords: [hcp, human-context-protocol, audit, audit-log, governance]
parties:
  - role: Management Client (HCP), acting for the Context Subject, a delegate or a former delegate
    requirement: REQUIRED
    member: issuer
  - role: HCP Provider
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The response discloses the person's own record of who read and wrote their context, for which purposes and in
    which categories, so it carries the provider's signature: the person can show which provider gave which account
    of their log, and when, which is what a later dispute with a client or the provider turns on. The request proof is
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
    A read. HCP 8.1 lists no audit event for reading the log, so the provider records nothing and changes nothing.
exposure:
  discloses: secret
  actsAsSubject: false
  ingests: secret
  rationale: >-
    The response discloses the person's audit log: which clients and operators read or wrote their context, the
    categories (possibly High-sensitivity ones such as `health_context`), the purposes stated, and the stable entry
    identifiers, which together describe the person's life. The request carries the Management Client's access
    token, a bearer secret.
retention:
  class: transient
  rationale: >-
    Reading the log appends no audit event (HCP 8.1 lists none), and the provider keeps nothing of the request beyond
    its duplicate-execution record. The cursor it issues stays valid for at least 300 seconds (HCP 9.5), and is not a
    record.
maxDocumentBytes:
  response: 2097152
  rationale: >-
    A page of up to 1000 audit events, each with its participants, categories and up to 20 disclosed entries, plus
    envelope and proof. This is a framework bound, not an HCP requirement; HCP sets no size limit. A provider chooses
    a page size that keeps each page under it, ending the page early and returning `nextCursor` where it must.
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
  - hcp/export
  - hcp/disclosures/summary
  - hcp/delegations/list
---

## Abstract

The **HCP — Read Audit Log** Trust Task carries the Read audit log operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 8.3 and 5.4; `GET /v2/audit` in HCP 9.2 and 9.4.7, paginated per HCP 9.5). A *Management Client*, acting for the *Context Subject*, or for a delegate or former delegate under the Delegated Authorization Profile, asks an *HCP Provider* for one page of the audit log, filtered by client, category, event type and date range. The provider answers with the events the caller may read, and a cursor for the next page. The log in JSON-LD form, whole, comes with [`hcp/export`](../../../export/0.1/spec.md) with `includeAudit` (HCP 8.3, 5.4.1); a summary per client comes with [`hcp/disclosures/summary`](../../../disclosures/summary/0.1/spec.md).

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on the audit log; the ones that bear on the document are restated below, unchanged. Conformance to HCP is HCP's ([SPEC §9.4](/SPEC.md#94-carried-protocols)): a provider **MUST NOT** list a conformance identifier for this carriage in its HCP `conformsTo`, and advertises the task through `trust-task-discovery`, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)). HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1); this specification takes it that a carriage other than that binding may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and carries this one as a Trust Task on that reading, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

This specification adds one check HCP does not make: the token binding of [The access token](#the-access-token), a carriage rule of the `hcp/*` family, not HCP's. [SPEC §9.4](/SPEC.md#94-carried-protocols) item 6 places a check the carried protocol does not make in a separate specification that does not declare `carries`. This family keeps this one in the carriage, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)), because it protects the token that the carriage itself moves from the HTTP `Authorization` header into the document, where it can be stored or forwarded with the document; a separate task could not protect the document this one carries.

## Authorization

The authority is **the Authorizing Principal's right to inspect the audit log** (HCP 8, 8.3), which a Management Client exercises for them. It is evidenced by the Management Client's access token in `accessToken`, which the provider validates exactly as over HCP's HTTP Binding. `accessToken` is OPTIONAL. Where the request carries none, the provider authenticates the request's `issuer` VID, by the request's REQUIRED `proof`, as a Management Client the Authorizing Principal enrolled by authenticating for that purpose (HCP 3.2.3), acting for the Authorizing Principal who enrolled it, and refuses any other issuer with `permissionDenied`, with the same fixed `message` as every other not-authorized refusal. Where it carries one, the token rules apply unchanged, including the token binding of [The access token](#the-access-token). Both are *pending HCP confirmation* ([question 56](#pending-hcp-confirmation)). Reading the audit log is a governance operation: any caller other than a Management Client acting for the Context Subject, a delegate or a former delegate receives not authorized (HCP 8.3, 6.1.9, 9.2), carried as `permissionDenied`. Context Clients **MUST NOT** have read access to the audit log (HCP 8.3); a grant held by a Context Client never authorizes this task, whatever its scopes (HCP 6.1.9). HCP names only its HTTP Binding as the carriage for a governance operation (HCP 5.4, 9.1); this specification accepts the operation as a Trust Task from a Management Client acting for the Authorizing Principal, and from no other caller (HCP 6.1.9), *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

What the caller reads depends on whom the Management Client acts for (HCP 8.3, 17.7). The provider filters and redacts; the Management Client never receives an event, or a member of one, that the party it acts for may not read.

| Acting for | Reads |
|---|---|
| The Context Subject | The whole log, at any time (HCP 8.3, 17.5, 17.7). |
| A delegate, while the delegation holds | Every event of a grant they authorized and of a governance operation performed for them, and the same for any sub-delegate they appointed; not the events of grants the Context Subject authorized (HCP 17.7, 17.10). |
| A former delegate | The events of grants they authorized: issued, revoked and expired, and the reads and writes made under them, each showing event type, time, client, Client Operator, category and purpose. The provider **MUST** withhold `entryId` and `relatedEntryId`, carrying each as null, **MUST** leave out `disclosed` and the `data` of every `context.*` event, and **MUST NOT** return an event in a category outside the delegation's scope (HCP 17.7). |
| Anyone else | Nothing (HCP 17.7): `permissionDenied`. |

A redacted event carries `entryId` and `relatedEntryId` as null, whatever they hold in the log, because the shared `AuditEvent` requires them (see [Response](#response)); it leaves `disclosed` and `data` out. Either way a former delegate cannot tell an event with no entry from one whose entry is withheld. Carrying a withheld member as null, rather than leaving it out as HCP 17.7's "never" might be read, is *pending HCP confirmation* ([question 42](#pending-hcp-confirmation)). Filters apply within what the caller may read, and never widen it.

The REQUIRED `proof` on the request establishes which Management Client VID sent the document and that it is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). Where the request carries `accessToken`, the proof is not the authorization: the token is. Where it carries none, the proof is the Management Client's authentication (see above). A token issued to a client other than the one the `issuer` VID identifies is refused with `permissionDenied` (see [The access token](#the-access-token)).

## Definitions

- **Management Client**, **Context Client**, **HCP Provider**, **Context Subject**, **Authorizing Principal**, **delegate**, **audit log** and **audit event** have their HCP 2.0-draft-3 meanings.
- The event shape is the shared [`AuditEvent`](../../../_shared/0.1/audit-event.schema.json) (HCP 8.2). It is open, as HCP's own shapes are, and carries HCP's member names verbatim, including `timestamp`, so that an event read here is the event an export carries. It requires `entryId`, `relatedEntryId`, `purpose`, `purposeDescription` and `grantId`, each a value or null, which HCP's own shape leaves optional, *pending HCP confirmation* ([question 42](#pending-hcp-confirmation)).
- HCP defines no request or result schema for this operation. The filters come from HCP 8.3, the paging members from HCP 9.5.

### HCP field mapping

| HCP field | Member | Notes |
|---|---|---|
| filter by HCP Client (HCP 8.3) | `client` | Named as in HCP 9.4.12, *pending HCP confirmation* ([question 29](#pending-hcp-confirmation)). |
| filter by category (HCP 8.3) | `category` | As above. |
| filter by event type (HCP 8.3) | `eventType` | Named after the audit event's own member (HCP 8.2), *pending HCP confirmation* ([question 29](#pending-hcp-confirmation)). |
| filter by date range (HCP 8.3) | `from`, `to` | Named as in HCP 9.4.12; both inclusive, *pending HCP confirmation* ([questions 29 and 30](#pending-hcp-confirmation)). |
| query parameter `cursor` (HCP 9.5) | `cursor` | Unchanged. |
| query parameter `limit` (HCP 9.5) | `limit` | Unchanged; at most 1000, a framework bound ([question 3](#pending-hcp-confirmation)). |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| the audit log returned (HCP 9.4.7) | `events` | HCP names no member; *pending HCP confirmation* ([question 30](#pending-hcp-confirmation)). |
| result `nextCursor` (HCP 9.5) | `nextCursor` | **Absent**, not null, on the last page, *pending HCP confirmation* ([question 3](#pending-hcp-confirmation)). |

**Where this payload is stricter than HCP's shapes.** Each payload is closed at its top level (`additionalProperties: false`): a top-level member outside this mapping is refused, where HCP's own request and result shapes leave the top level open, so an extension member HCP would allow there is not carried. The HCP shapes nested within stay open, as HCP's are. Every size bound, and every non-empty requirement (a `minLength` or `minItems` of 1) on a member that HCP's own schema allows to be empty, is a Trust Tasks framework bound on what an `hcp/*` party writes, not an HCP requirement, as the schema descriptions say.

### Parties

| HCP role | Trust Task member |
|---|---|
| Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../../entries/query/0.1/spec.md#parties), a Management Client **MUST** send a token only to the provider VID it obtained when the person connected it, and where the provider publishes an HCP discovery document (HCP 19.1), the provider **MUST** make its VID verifiably bound to it and the Management Client **MUST** check that binding before it sends a token to the VID.

## Request

The Management Client asks for a page of the log. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

- `accessToken` (OPTIONAL) is the Management Client's token. Where it is absent, the request's `proof` authenticates an enrolled Management Client (see [Authorization](#authorization)).
- `client`, `category`, `eventType`, `from` and `to` (each OPTIONAL, single-valued) narrow the log as HCP 8.3 requires, *pending HCP confirmation* ([question 29](#pending-hcp-confirmation)). An event matches when it matches every filter given. `client` matches an event in which that client took part, as initiator or as the client of the grant concerned, *pending HCP confirmation* ([question 29](#pending-hcp-confirmation)): HCP 8.3 says only "filter by HCP Client"; `category` matches an event whose `categories` include it; `from` and `to` bound the event's `timestamp`, both inclusive, *pending HCP confirmation* ([question 30](#pending-hcp-confirmation)).
- `cursor` (OPTIONAL) is a previous response's `nextCursor`; absent asks for the first page. A Management Client that sends a cursor **MUST** send the same filters it sent for the page that cursor came from; a provider **MAY** refuse a cursor whose filters differ with `malformedRequest`. A provider **MUST NOT** accept a cursor it did not issue.
- `limit` (OPTIONAL) is the most events to return, at most 1000 (a framework bound, *pending HCP confirmation*, [question 3](#pending-hcp-confirmation)). Absent leaves the page size to the provider. Either way, the provider returns fewer events where more would take the response past its `maxDocumentBytes`, and returns `nextCursor` for the rest.

### The person reviews who read their health context in November

```json
{
  "id": "urn:uuid:6b1e3d9a-2f4c-4a7e-9d0b-5c8e1f2a3b41",
  "type": "https://trusttasks.org/spec/hcp/audit/read/0.1#request",
  "issuer": "did:web:wallet.person.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T09:30:00Z",
  "threadId": "urn:uuid:a3c5e7f9-1b2d-4e6f-8a0c-2d4f6b8e0a62",
  "payload": {
    "accessToken": "hcp_mt_8f3a1c9e7b2d4f60",
    "category": "health_context",
    "eventType": "context.read",
    "from": "2026-11-01T00:00:00Z",
    "to": "2026-11-25T23:59:59Z",
    "limit": 50
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T09:30:00Z",
    "verificationMethod": "did:web:wallet.person.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z4Rt7wY2nKcP9vB3xL6mQ1sD8fG5hJ2aZk4uE7oVnC9iN3bT6yH1rW8xS5eA2dU7gF4lO9pK3vX6zM1cT8bY2q"
  }
}
```

## Response

The provider answers with one page. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json).

- `events` holds the page's audit events, oldest first, each a shared [`AuditEvent`](../../../_shared/0.1/audit-event.schema.json) (HCP 8.2), as the caller may read it ([Authorization](#authorization)). An empty `events` is a successful answer: nothing matched. The member name is *pending HCP confirmation* ([question 30](#pending-hcp-confirmation)).
- Every event carries `entryId`, `relatedEntryId`, `purpose`, `purposeDescription` and `grantId`, each with its value or null, as HCP 8.2's template writes them: null says the event concerns no entry, no related entry, no purpose, no stated task or no grant. HCP's own shape leaves these members optional; the carriage requires them, so that a null always travels and a reader never has to tell a null from an absent member, *pending HCP confirmation* ([question 42](#pending-hcp-confirmation)).
- `nextCursor` is an opaque token for the next page, present only when more events remain. It is **absent**, never null, on the last page, a departure from HCP 9.5's null, *pending HCP confirmation* ([question 3](#pending-hcp-confirmation)).

Failures use `trust-task-error`, not a `#response` document (see [Errors](#errors)).

### A page of `context.read` events

```json
{
  "id": "urn:uuid:c8e0a2b4-6d7f-4b1c-9e3a-5f7b9d1e3c83",
  "type": "https://trusttasks.org/spec/hcp/audit/read/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:wallet.person.example",
  "issuedAt": "2026-11-25T09:30:01Z",
  "threadId": "urn:uuid:a3c5e7f9-1b2d-4e6f-8a0c-2d4f6b8e0a62",
  "payload": {
    "events": [
      {
        "eventId": "urn:uuid:1d3f5b7c-9e0a-4c2e-8b4d-6f8a0c2e4b17",
        "eventType": "context.read",
        "timestamp": "2026-11-24T10:00:01Z",
        "participants": {
          "initiator": { "role": "context-client", "id": "https://booking-agent.example/client" },
          "clientOperator": { "id": "https://booking-agent.example" },
          "authorizingPrincipal": { "id": "subject-7f2c" },
          "provider": { "id": "https://context.provider.example" },
          "retrievalId": "urn:uuid:9a1e5c73-2d4b-4e8f-a6c1-7b0d3e2f1a99"
        },
        "entryId": null,
        "relatedEntryId": null,
        "categories": ["general_preferences", "health_context"],
        "purpose": "https://hcp.example/purposes/reservation",
        "purposeDescription": "Book a table that can serve the guest's diet.",
        "grantId": "urn:uuid:5e7a9c1b-3d4f-4a6b-8c0e-2f4a6c8e0b28",
        "disclosed": [
          { "entryId": "urn:uuid:7b9d1f3a-5c6e-4b8a-9d0f-1a3c5e7b9d40", "version": 1, "useUntil": "2026-11-24T22:00:01Z" }
        ],
        "data": { "maxEntries": 3, "categoriesSearched": ["general_preferences", "health_context"], "directives": [] }
      }
    ],
    "nextCursor": "Y3VyLTRmOWEyYzdlLTEx"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T09:30:01Z",
    "verificationMethod": "did:web:context.provider.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z2Hn6tQ9wE4rY7uI1oP3aS5dF8gJ2kL4zX6cV9bN1mQ3wE5rT7yU9iO2pA4sD6fG8hJ1kL3zX5cV7bN9mQ2w"
  }
}
```

### A former delegate's last page

The Management Client acts for a former delegate. The provider returns only an event of a grant they authorized, with `entryId` and `relatedEntryId` withheld as null, without `disclosed` or the event's `data`, and with no `nextCursor`, because this is the last page.

```json
{
  "id": "urn:uuid:e2f4a6c8-0b1d-4e3f-a5b7-c9d1e3f5a794",
  "type": "https://trusttasks.org/spec/hcp/audit/read/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:wallet.guardian.example",
  "issuedAt": "2026-11-25T09:40:01Z",
  "threadId": "urn:uuid:b4d6f8a0-2c3e-4f5a-9b1d-3e5f7a9c1e05",
  "payload": {
    "events": [
      {
        "eventId": "urn:uuid:2e4a6c8e-0f1b-4d3a-8c5e-7a9c1e3b5d16",
        "eventType": "context.read",
        "timestamp": "2026-03-02T15:12:44Z",
        "participants": {
          "initiator": { "role": "context-client", "id": "https://tutor.example/client" },
          "clientOperator": { "id": "https://tutor.example" },
          "provider": { "id": "https://context.provider.example" }
        },
        "entryId": null,
        "relatedEntryId": null,
        "categories": ["learning_context"],
        "purpose": "https://hcp.example/purposes/tutoring",
        "purposeDescription": "Plan this week's maths practice.",
        "grantId": "urn:uuid:6f8b0d2e-4a5c-4e7b-9d1f-3b5d7f9a1c39"
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T09:40:01Z",
    "verificationMethod": "did:web:context.provider.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z5Kp2sW8eR3tY6uI9oA1sD4fG7hJ0kL2zX5cV8bN3mQ6wE9rT2yU5iO8pA1sD4fG7hJ0kL3zX6cV9bN2mQ5e"
  }
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows.

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request`: including a cursor the provider did not issue, one past its validity, or one sent with different filters | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: the caller is not a Management Client acting for the Context Subject, a delegate or a former delegate (HCP 6.1.9, 8.3, 17.7), including every Context Client | `permissionDenied`, with one fixed `message` whatever the reason (HCP 5.5, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| conflict | `idempotency-conflict` | Does not arise: HCP defines it for a retried request with an idempotency key (HCP 9.6), and this operation carries none. |
| version conflict | `version-conflict` | Does not arise: a read names no entry version (HCP 5.4). |
| credential invalid | `credential-invalid` | Does not arise: it concerns a credential written as an entry's content (HCP 14.6). |
| subject unavailable | `subject-unavailable` | `unavailable` |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion` |

A filter that matches nothing, or nothing the caller may read, is not an error: the answer is an empty `events`.

## Rules carried from HCP

- **Context Clients never read the log** (HCP 8.3), and a delegate or former delegate reads only what HCP 17.7 sets out ([Authorization](#authorization)).
- **No event is appended.** HCP 8.1 lists no event for reading the log, so this task records nothing; a re-delivered document is absorbed by [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements) like any other.
- **Entries by `id`.** The log names entries by their stable `id`, never by the `ref` a client received, because the log is the Context Subject's (HCP 8.2). The query text of a read is never recorded (HCP 8.2).
- **Cursors** (HCP 9.5) are opaque to the Management Client, encode no credential or entry content, and stay valid for at least 300 seconds. A bare sequence number or offset does not satisfy this: a provider **MUST NOT** issue a cursor from which the size of the log, or a position in it, can be read. An encrypted or signed offset, or a server-side handle, does.
- **Durability window** (HCP 8.4). A provider that declares a durability window for `context.read` **MUST** disclose it wherever the log is presented to the person, as what the person may be missing (for example, "reads in the last ten seconds may not appear yet"), not as storage behaviour, in whatever modality the log is presented. The provider declares the window in its discovery document; a Management Client that presents a page of this task to the person **MUST** make that disclosure, and a provider that presents the log itself makes it too.
- **Former delegates' access.** It lasts as long as the log is kept, and the Context Subject cannot end it; wherever the Context Subject's log is presented, they **MUST** be told that it exists (HCP 17.7).
- **Integrity.** The provider declares the log's integrity as `audit_integrity` in its discovery document (HCP 8.7). This task carries the events, not the integrity evidence; a hash-chained provider's events carry whatever members its scheme adds, which the open `AuditEvent` preserves.
- **Export.** The Authorizing Principal can export the log in JSON-LD (HCP 8.3) through [`hcp/export`](../../../export/0.1/spec.md) with `includeAudit`; this task returns JSON pages, not that export.

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of [the list of questions put to the editors](../../../_shared/0.1/QUESTIONS.md).

- **Question 1 — conformance claims.** HCP defines no conformance identifier for Trust Task carriage, so a provider **MUST NOT** claim this carriage in its HCP `conformsTo`; it advertises the task with [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead. The binding identifier for the carriage is the same question as [question 37](#pending-hcp-confirmation): whether a carriage other than the HTTP Binding may carry governance operations at all.
- **Question 3 — pagination.** `cursor` and `limit` as HCP 9.5 names them, `limit` at most 1000 (a framework bound), and `nextCursor` absent, not null, on the last page.
- **Question 22 — the client's identity.** The `issuer` VID, authenticated by the request's `proof`, identifies the HCP Client the token was issued to; a token issued to any other client **MUST** be refused with `permissionDenied`. This token binding is a carriage rule of this family, a check HCP does not itself make (see [The access token](#the-access-token) and [Conformance](#conformance)).
- **Question 29 — filter names, and what `client` matches.** `client`, `category`, `eventType`, `from` and `to`, each single-valued and optional; `client` matches an event whose initiator is that client, or whose grant was issued to it.
- **Question 30 — the result member and the date range.** The page's events are in `events`, oldest first, and `from` and `to` are both inclusive.
- **Question 37 — governance over a carriage other than the HTTP Binding.** HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1). This specification takes it that another carriage, here a Trust Task, may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and applies every rule HCP places on the operation unchanged (see [Conformance](#conformance)).
- **Question 42 — presence signals the carriage adds.** Every `AuditEvent` carries `entryId`, `relatedEntryId`, `purpose`, `purposeDescription` and `grantId`, each with its value or null, though HCP's own shape leaves them optional; an event redacted for a former delegate carries `entryId` and `relatedEntryId` as null rather than leaving them out (HCP 8.2, 17.7).
- **Question 53 — a request proof on every request that carries a token.** A request `proof` is **REQUIRED** on every `hcp/*` request that carries `accessToken`, so that the token binding of question 22 always applies and a token lifted from a stored or forwarded document cannot be replayed in an unsigned request. This is a carriage rule of this family, not HCP's, and is stricter than HCP's bearer model, under which presenting a valid token is enough (HCP 9.3); see [The access token](#the-access-token).
- **Question 56 — a Management Client authenticated without a token.** `accessToken` is OPTIONAL. Where it is absent, the provider authenticates the request's `issuer` VID, by the request's REQUIRED `proof`, as a Management Client the Authorizing Principal enrolled by authenticating for that purpose, reading HCP 3.2.3's "credentials" as including an enrolled key proved by a signature on each request, and refuses any other issuer with `permissionDenied`, with the one fixed `message`. Where a token is present, the token rules apply unchanged, including the binding of question 22. The request `proof` stays REQUIRED whether or not a token is present (question 53), and a provider keeps a record of the Management Clients each person enrolled. This is not question 37, which asks whether governance may travel off the HTTP Binding at all; it asks how the caller is authenticated once it does (see [Authorization](#authorization)).

## What HCP requires that a document cannot express

- The audit log itself: append-only, complete, and durable as declared (HCP 8, 8.1, 8.4, 8.7).
- The filtering and redaction of HCP 17.7 for delegates and former delegates, made against provider-held delegations and grants.
- The disclosure of a declared durability window wherever the log is presented (HCP 8.4).
- A cursor's opacity, and its validity for at least 300 seconds (HCP 9.5).
- Carriage by the HTTP Binding: HCP carries governance operations only over that binding (HCP 1.6, 5.4, 9.1), and a Trust Task is not it. This specification carries the operation for a Management Client acting for the Authorizing Principal (HCP 6.1.9) instead, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

## Security & Privacy

### Data carried

The request carries the Management Client's access token where it carries one, a bearer secret that **MUST** travel only to the provider, and the filters, which can themselves say what the person is concerned about (a category such as `health_context`). The response carries the person's audit log: clients, Client Operators, categories, purposes, stated tasks, grant identifiers and stable entry identifiers, which together describe the person's life. A producer **MUST NOT** put the token, a filter, or any part of an event in `ext` or in any `message`.

Because the response proof is REQUIRED ([SPEC §7.3 item 8](/SPEC.md#73-specification-requirements)), each page is a statement signed by the provider, which anyone holding the provider's public key can check came from it. The proof is made for `authentication`, not as an attestation ([SPEC §4.7.3](/SPEC.md#473-proof-purpose-and-verification-relationship)). That helps the person in a dispute, and makes the page more valuable to leak. A Management Client **MUST NOT** pass a page to anyone but the person it acts for, unless that person asks it to.

### The access token

`accessToken`, where a request carries it, is a bearer secret: whoever presents it exercises the authority it evidences. It **MUST** travel only to the provider. A recipient **MUST NOT** log `accessToken` or retain it beyond the exchange.

**In an agent runtime**, the token is held as [`hcp/entries/query`](../../../entries/query/0.1/spec.md#the-access-token) requires: by the component that builds and sends the document, never in a model's context or a tool's result (HCP 10.5).

**Token binding, a carriage rule of this family.** Every request that carries `accessToken` carries a `proof`, *pending HCP confirmation* ([question 53](#pending-hcp-confirmation)), and the provider **MUST** refuse with `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies, so that a token lifted from a stored or forwarded document is of no use from another VID. HCP validates the token alone and does not bind it to a VID (HCP 9.3): this rule is the `hcp/*` family's, not HCP's, and rests on the `issuer` VID identifying the HCP Client, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)). It is a check HCP does not make (see [Conformance](#conformance)).

**Retained copies.** The provider verifies the request's `proof` over the document as received, at receipt, before it removes anything ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). A provider that retains the request document, as a duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) or as evidence of the request, **MUST** remove `accessToken` from the copy it keeps and record in its place `accessTokenDigest`: the SHA-256 multihash of the token's UTF-8 bytes, multibase-encoded as base64url without padding (prefix `u`), the framework's `DigestMultibase` form ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)). The kept copy's `proof` no longer verifies over it, so the provider records with the copy that the proof was verified at receipt: when, under which verification method, and with what result. It relies on that record, and **MUST NOT** verify the proof again over the kept copy. A record that keeps only a digest of the document, such as its task digest ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)), keeps no token either.

### Correlation

The provider learns which Management Client read which part of the log, and when; that is inherent in the authorization check. Stable entry identifiers in the log would let anyone who holds two pages, or a page and an export, join them; that is why the log is the Context Subject's only, and why a former delegate never sees them (HCP 8.2, 17.7). Cursors are opaque so that they disclose nothing of the log's size (HCP 9.5). `threadId` **SHOULD** be fresh per read.

### Retention

The provider records nothing for this task (HCP 8.1 lists no event for reading the log). The Management Client keeps a page only as long as it presents it, unless the person asks it to keep a copy; the signed page is then evidence of what the provider reported. A cursor expires on the provider's terms, no sooner than 300 seconds after it is issued (HCP 9.5).

### Consent/purpose

The log is the person's record, and reading it is their right (HCP 8, 8.3, 17.5). A delegate's and a former delegate's reads exist because a person who consented on someone else's behalf stays accountable for what they approved (HCP 17.7); the Context Subject is told those reads are possible. No further approval is needed to read the log, and whether a Management Client asks the person again before each read is its own policy ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).
