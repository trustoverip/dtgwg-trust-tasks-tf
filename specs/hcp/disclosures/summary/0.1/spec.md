---
slug: hcp/disclosures/summary
version: "0.1"
title: "HCP — Disclosure Summary"
summary: A Management Client, acting for the person or a delegate, asks a Human Context Protocol provider what each client has received; carries the HCP 2.0-draft-3 Disclosure summary operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Disclosure summary
category: access-control
keywords: [hcp, human-context-protocol, disclosure, audit, governance]
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
    The response tells the person which clients received which categories of their context, for which purposes,
    and until when the last copy stays usable, so it carries the provider's signature: the person can show what the
    provider reported, which is what a complaint against a client turns on. The request proof is REQUIRED, as for every
    Management Client read whose response is secret: the access token is the authority, but a signed request lets the
    provider refuse a token lifted from a stored or forwarded document and sent from another VID, and shows which
    Management Client asked for the disclosure.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task discloses secret material and so is consequential; a re-delivered read must be placeable in the
    consumer's duplicate-execution window.
sideEffects:
  level: none
  rationale: >-
    A read computed from the audit log (HCP 8.6). HCP 8.1 lists no audit event for it, so the provider records
    nothing and changes nothing.
exposure:
  discloses: secret
  actsAsSubject: false
  ingests: secret
  rationale: >-
    The response is aggregate, but it names each client and Client Operator that received the person's context, the
    categories (possibly High-sensitivity ones such as `health_context`), the purposes, and any record a client keeps
    under a legal obligation: that a named clinic received health context is itself confidential. The request
    carries the Management Client's access token, a bearer secret.
retention:
  class: transient
  rationale: >-
    The summary is computed from the audit log on request (HCP 8.6); reading it appends no audit event (HCP 8.1 lists
    none), and the provider keeps nothing of the request beyond its duplicate-execution record.
maxDocumentBytes:
  response: 2097152
  rationale: >-
    A page of up to 1000 client summaries, each with its client description, grants, categories, purposes and any
    legal-retention declaration, plus envelope and proof. This is a framework bound, not an HCP requirement; HCP sets
    no size limit. A provider chooses a page size that keeps each page under it, returning `nextCursor` for the rest.
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence", here the Management
      Client's access token.
    retryable: false
related:
  - hcp/audit/read
  - hcp/grants/list
  - hcp/grants/revoke
  - hcp/grants/revoked
---

## Abstract

The **HCP — Disclosure Summary** Trust Task carries the Disclosure summary operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 8.6 and 5.4; `GET /v2/disclosures` in HCP 9.2 and 9.4.12). A *Management Client*, acting for the *Authorizing Principal*, asks an *HCP Provider* what each HCP Client has received: under which grants, in which categories, for which purposes, how many entries and retrievals, over what period, until when the last copy stays usable, whether the last revocation notice reached the client, and whether the client keeps copies under a legal obligation. The summary is computed from the audit log, which [`hcp/audit/read`](../../../audit/read/0.1/spec.md) reads event by event.

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on the disclosure summary; the ones that bear on the document are restated below, unchanged. Conformance to HCP is HCP's ([SPEC §9.4](/SPEC.md#94-carried-protocols)): a provider **MUST NOT** list a conformance identifier for this carriage in its HCP `conformsTo`, and advertises the task through `trust-task-discovery`, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)). HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1); this specification takes it that a carriage other than that binding may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and carries this one as a Trust Task on that reading, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

## Authorization

The authority is **the Authorizing Principal's right to know what each client has received** (HCP 8.6, 18.4), which a Management Client exercises for them. It is evidenced by the Management Client's access token in `accessToken`, which the provider validates exactly as over HCP's HTTP binding. The disclosure summary is a governance operation (HCP 8.6, 6.1.9): any caller other than a Management Client acting for the Authorizing Principal receives not authorized (HCP 6.1.9, 9.2), carried as `permissionDenied`. A grant held by a Context Client never authorizes it, whatever its scopes (HCP 6.1.9). HCP names only its HTTP Binding as the carriage for a governance operation (HCP 5.4, 9.1); this specification accepts the operation as a Trust Task from a Management Client acting for the Authorizing Principal, and from no other caller (HCP 6.1.9), *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

The summary discloses nothing the log does not hold (HCP 8.6), and so nothing the caller could not read in the log. Under the Delegated Authorization Profile, the Context Subject's summary is computed from the whole log. A delegate, while the delegation holds, is given a summary scoped as follows, *pending HCP confirmation* ([question 38](#pending-hcp-confirmation)): HCP 8.6 gives the summary to the Authorizing Principal, who may be a delegate (HCP 3.2.2), but neither 8.6 nor 17.7 says what a delegate's summary covers. Theirs is computed only from the events HCP 17.7 lets a delegate read ([`hcp/audit/read`](../../../audit/read/0.1/spec.md#authorization)): those of the grants they authorized under the delegation, and of the grants of sub-delegates they appointed (HCP 17.10). A delegate reads every member of those events, `disclosed` included, so `entries`, `retrievals` and `usableUntil` computed from them reveal nothing HCP 17.7 withholds from them; a client appears in their summary only for those grants, and `grants` lists only those grants. A former delegate is not given the summary: HCP 17.7 withholds `entryId` and `disclosed` from them, from which `entries` and `usableUntil` are computed, and they read the record of the grants they authorized through [`hcp/audit/read`](../../../audit/read/0.1/spec.md) instead. A Management Client acting for a former delegate, or for anyone else, receives `permissionDenied`.

The REQUIRED `proof` on the request establishes which Management Client VID sent the document and that it is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). It is not the authorization: the token is. A token issued to a client other than the one the `issuer` VID identifies is refused with `permissionDenied` (see [The access token](#the-access-token)).

## Definitions

- **Management Client**, **Context Client**, **HCP Provider**, **Context Subject**, **Authorizing Principal**, **delegate**, **grant**, **retrieval**, **use-until** and **Client Operator** have their HCP 2.0-draft-3 meanings.
- The client description is the shared [`ClientDescription`](../../../_shared/0.1/context-entry.schema.json) (HCP 3.5, 3.2.8), and the legal-retention declaration the shared [`LegalRetention`](../../../_shared/0.1/grant.schema.json) (HCP 6.2.5), as in the grant. `ClientSummary` and `GrantStatus` are defined in [`payload.schema.json`](payload.schema.json) from HCP 8.6, and are open, as HCP's own shapes are.

### HCP field mapping

| HCP field | Member | Notes |
|---|---|---|
| query parameters `client`, `category`, `from`, `to` (HCP 9.4.12) | `client`, `category`, `from`, `to` | Unchanged. `from` and `to` are both inclusive, *pending HCP confirmation* ([question 30](#pending-hcp-confirmation)). |
| query parameters `cursor`, `limit` (HCP 9.5, 9.4.12: "MAY be paginated") | `cursor`, `limit` | Unchanged; `limit` at most 1000, a framework bound ([question 3](#pending-hcp-confirmation)). |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result `clients` (HCP 9.4.12) | `clients` | Unchanged; each member carries the members of HCP 8.6, unchanged. |
| result `nextCursor` (HCP 9.5) | `nextCursor` | **Absent**, not null, on the last page, *pending HCP confirmation* ([question 3](#pending-hcp-confirmation)). |

### Parties

| HCP role | Trust Task member |
|---|---|
| Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../../entries/query/0.1/spec.md#parties), a provider **MUST** make its VID verifiably bound to its HCP identity (its discovery document, HCP 19.1), and a Management Client **MUST** check that binding before it sends a token to a VID.

## Request

The Management Client asks for the summary. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

- `accessToken` (REQUIRED) is the Management Client's token.
- `client`, `category`, `from` and `to` (each OPTIONAL) narrow the summary to one client, one category, and a date range (HCP 8.6, 9.4.12). Each summary is then computed only from the disclosures that match: `from` and `to` bound when each disclosure took place, both inclusive, *pending HCP confirmation* ([question 30](#pending-hcp-confirmation)).
- `cursor` and `limit` (OPTIONAL) page the result, which HCP lets a provider paginate (HCP 9.4.12, 9.5), *pending HCP confirmation* ([question 3](#pending-hcp-confirmation)). `limit` is the most client summaries to return, at most 1000; absent leaves the page size to the provider. Either way, a provider **MUST** paginate where the summaries within the filters would take the response past its `maxDocumentBytes`: it returns fewer, and `nextCursor` for the rest. A Management Client that sends a cursor **MUST** send the same filters it sent for the page that cursor came from, and a provider **MUST NOT** accept a cursor it did not issue.

### What has been disclosed in health context

```json
{
  "id": "urn:uuid:3a5c7e9b-1d2f-4b4a-8c6e-0f2a4c6e8a19",
  "type": "https://trusttasks.org/spec/hcp/disclosures/summary/0.1#request",
  "issuer": "did:web:wallet.person.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T11:00:00Z",
  "threadId": "urn:uuid:5c7e9a1b-3d4f-4a6c-9e8a-2b4d6f8a0c31",
  "payload": {
    "accessToken": "hcp_mt_8f3a1c9e7b2d4f60",
    "category": "health_context",
    "from": "2026-01-01T00:00:00Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T11:00:00Z",
    "verificationMethod": "did:web:wallet.person.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3Wq5eR8tY1uI4oP7aS0dF3gH6jK9lZ2xC5vB8nM1qW4eR7tY0uI3oP6aS9dF2gH5jK8lZ1xC4vB7nM0qW3e"
  }
}
```

## Response

The provider answers with one summary per client. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json).

- `clients` holds one `ClientSummary` per HCP Client that received context within the filters (HCP 8.6, 9.4.12), each with:
  - `client`: the client's `id` and `name`, and its `operator`, with `id` and `name` (HCP 3.2.8);
  - `grants`: each grant under which it received context, with `grantId` and `status` (`active`, `revoked` or `expired`);
  - `categories` and `purposes`: the categories disclosed to it and the purpose codes declared for those disclosures (HCP 6.1.5.2);
  - `entries` and `retrievals`: how many distinct entries were disclosed, and in how many retrievals;
  - `firstAt` and `lastAt`: the period over which the disclosures took place;
  - `usableUntil`: the latest `useUntil` among the copies disclosed (HCP 7.6) or, where a revocation notice was delivered, the time of delivery (HCP 6.2.4). It is REQUIRED and admits null, as in the result shape HCP publishes; HCP's text does not say when it is null, and a provider **SHOULD** send a date-time whenever the log records one;
  - `notification`: `delivered`, `abandoned` or `none`, for the most recent revocation notice (HCP 6.2.4);
  - `legalRetention`, only where the client declared at registration that the law requires it to keep what it receives: the `record`, `obligation` and `period`, as in the grant (HCP 6.1.5, 6.2.5).
- `nextCursor` is an opaque token for the next page, present only when more summaries remain; it is **absent**, never null, on the last page, and where every summary fits in one response, *pending HCP confirmation* ([question 3](#pending-hcp-confirmation)).

An empty `clients` is a successful answer: no client received context within the filters. Failures use `trust-task-error`, not a `#response` document (see [Errors](#errors)).

### Two clients have received health context

```json
{
  "id": "urn:uuid:7e9a1c3d-5f6b-4d8e-a0c2-4e6a8c0e2b53",
  "type": "https://trusttasks.org/spec/hcp/disclosures/summary/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:wallet.person.example",
  "issuedAt": "2026-11-25T11:00:01Z",
  "threadId": "urn:uuid:5c7e9a1b-3d4f-4a6c-9e8a-2b4d6f8a0c31",
  "payload": {
    "clients": [
      {
        "client": {
          "id": "https://booking-agent.example/client",
          "name": "Table Finder",
          "operator": { "id": "https://booking-agent.example", "name": "Table Finder Inc." }
        },
        "grants": [
          { "grantId": "urn:uuid:5e7a9c1b-3d4f-4a6b-8c0e-2f4a6c8e0b28", "status": "active" }
        ],
        "categories": ["health_context"],
        "purposes": ["https://hcp.example/purposes/reservation"],
        "entries": 1,
        "retrievals": 3,
        "firstAt": "2026-10-12T19:04:10Z",
        "lastAt": "2026-11-24T10:00:01Z",
        "usableUntil": "2026-11-24T22:00:01Z",
        "notification": "none"
      },
      {
        "client": {
          "id": "https://clinic.example/hcp-client",
          "name": "Riverside Clinic intake",
          "operator": { "id": "https://clinic.example", "name": "Riverside Clinic" }
        },
        "grants": [
          { "grantId": "urn:uuid:8a0c2e4f-6b7d-4f9a-b1c3-5d7f9b1d3e64", "status": "revoked" }
        ],
        "categories": ["health_context"],
        "purposes": ["https://hcp.example/purposes/treatment"],
        "entries": 4,
        "retrievals": 2,
        "firstAt": "2026-06-03T08:15:00Z",
        "lastAt": "2026-06-03T08:16:12Z",
        "usableUntil": "2026-09-01T14:02:37Z",
        "notification": "delivered",
        "legalRetention": {
          "record": "Patient medical record",
          "obligation": "State medical records retention law",
          "period": "10 years after the last visit"
        }
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T11:00:01Z",
    "verificationMethod": "did:web:context.provider.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z6Lm9nB2vC5xZ8aS1dF4gH7jK0lQ3wE6rT9yU2iO5pA8sD1fG4hJ7kL0zX3cV6bN9mQ2wE5rT8yU1iO4pA7s"
  }
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows.

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request`: including a cursor the provider did not issue, one past its validity, or one sent with different filters | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: the caller is not a Management Client acting for the Authorizing Principal (HCP 6.1.9, 8.6), or, under the Delegated Authorization Profile, for the Context Subject or a delegate while the delegation holds (HCP 17.7, [question 38](#pending-hcp-confirmation)), so including a former delegate | `permissionDenied`, with one fixed `message` whatever the reason (HCP 5.5, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| conflict | `idempotency-conflict` | Does not arise: HCP defines it for a retried request with an idempotency key (HCP 9.6), and this operation carries none. |
| version conflict | `version-conflict` | Does not arise: a read names no entry version (HCP 5.4). |
| credential invalid | `credential-invalid` | Does not arise: it concerns a credential written as an entry's content (HCP 14.6). |
| subject unavailable | `subject-unavailable` | `unavailable` |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion` |

## Rules carried from HCP

- **Computed from the log.** The summary is computed from the audit log (HCP 8.2, 8.6) and discloses nothing the log does not hold. It does not show what happens to context after it reaches a client, which the provider cannot see (HCP 6.2.2, 8.6). A Management Client that presents it **MUST NOT** present it as an account of what a client did with the context.
- **When the last copy stops being usable** (HCP 6.2.4). Where delivery of a revocation notice was abandoned, the provider tells the Authorizing Principal that the client was not notified, and when the last copy disclosed under the grant stops being usable; `notification: "abandoned"` and `usableUntil` show it here.
- **Copies kept under a legal obligation** (HCP 6.2.5). When a grant is revoked, the provider tells the Authorizing Principal which client keeps copies under a declared obligation, in which record, and for how long; `legalRetention` shows it here. Revoking the grant does not remove such copies.
- **At the age of majority** (HCP 17.6). Where a delegation ends for the reason `majority`, the provider shows the person, the first time they use a Management Client afterwards, a summary of the delegated period from the disclosure summary: the delegate, the grants made, and for each the client, its Client Operator, the categories, the purposes and the period. A provider **MAY** build that summary from this task, filtered with `from` and `to` to the delegated period, and the Management Client says that the former delegate can read the record of the grants they authorized (HCP 17.7).
- **No event is appended.** HCP 8.1 lists no event for the disclosure summary.
- **Cursors**, where the provider paginates, are opaque, encode no credential or entry content, stay valid for at least 300 seconds, and are never a bare offset (HCP 9.5).
- **Durability window** (HCP 8.4). The summary is computed from the log, so a provider that declares a durability window for `context.read` **SHOULD** disclose it wherever the summary is presented, as what the person may be missing.

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 1 — conformance claims.** HCP defines no conformance identifier for Trust Task carriage, so a provider **MUST NOT** claim this carriage in its HCP `conformsTo`; it advertises the task with [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead. The binding identifier for the carriage is the same question as [question 37](#pending-hcp-confirmation): whether a carriage other than the HTTP Binding may carry governance operations at all.
- **Question 3 — pagination.** `cursor` and `limit` as HCP 9.5 names them, `limit` at most 1000 (a framework bound), and `nextCursor` absent, not null, on the last page.
- **Question 22 — the client's identity.** The `issuer` VID, authenticated by the request's `proof`, identifies the HCP client the token was issued to; where the request carries a proof, a token issued to any other client is refused with `permissionDenied`, a strengthening HCP does not itself require (see [The access token](#the-access-token)).
- **Question 30 — the date range.** `from` and `to` are both inclusive, and bound when each disclosure took place.
- **Question 37 — governance over a carriage other than the HTTP Binding.** HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1). This specification takes it that another carriage, here a Trust Task, may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and applies every rule HCP places on the operation unchanged (see [Conformance](#conformance)).
- **Question 38 — a delegate's summary.** A delegate, while the delegation holds, is given the summary, computed only from the events of the grants they authorized under the delegation and of their sub-delegates' grants (HCP 17.7, 17.10); a former delegate is not given it, because `entries` and `usableUntil` are computed from `disclosed`, which HCP 17.7 withholds from them.

## What HCP requires that a document cannot express

- That the summary is computed from the audit log and discloses nothing it does not hold (HCP 8.6).
- The person-facing notices of HCP 6.2.4, 6.2.5 and 17.6, which a Management Client or the provider's own person-facing capabilities give (HCP 18.4).
- A cursor's opacity, and its validity for at least 300 seconds (HCP 9.5).
- Carriage by the HTTP Binding: HCP carries governance operations only over that binding (HCP 1.6, 5.4, 9.1), and a Trust Task is not it. This specification carries the operation for a Management Client acting for the Authorizing Principal (HCP 6.1.9) instead, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

## Security & Privacy

### Data carried

The request carries the Management Client's access token, a bearer secret that **MUST** travel only to the provider, and filters, which can say what the person is concerned about. The response is aggregate: it carries no entry, no entry identifier and no context. It does name each client and Client Operator that received the person's context, the categories and purposes, the grant identifiers, and any record a client keeps under a legal obligation; that a named clinic received `health_context` is itself confidential. A producer **MUST NOT** put the token, a filter, or any part of a summary in `ext` or in any `message`.

### The access token

`accessToken` is a bearer secret: whoever presents it exercises the authority it evidences. It **MUST** travel only to the provider. A recipient **MUST NOT** log `accessToken` or retain it beyond the exchange: a duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) or an evidentiary record of the request keeps the document with `accessToken` removed, or a digest of the document, such as its task digest ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)), and never the token. Where the request carries a `proof`, the provider **MUST** refuse with `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies, so that a token lifted from a stored or forwarded document is of no use from another VID. HCP does not itself bind a token to a VID: this is a strengthening of the carriage, resting on the `issuer` VID identifying the HCP client, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)).

### Correlation

The provider learns which Management Client asked about which clients and categories, and when. Grant identifiers in the response let anyone who holds it join it to the person's grants and audit log; a Management Client **MUST NOT** pass the summary to anyone but the person it acts for, unless that person asks it to. `threadId` **SHOULD** be fresh per request.

### Retention

The provider records nothing for this task. The Management Client keeps a summary only as long as it presents it, unless the person asks it to keep a copy; the signed summary is then evidence of what the provider reported, and stays as confidential as the log it was computed from.

### Consent/purpose

The summary exists so that the person knows who has received their context, and can act on it, by revoking a grant ([`hcp/grants/revoke`](../../../grants/revoke/0.1/spec.md)) or setting a standing rule ([`hcp/rules/set`](../../../rules/set/0.1/spec.md)). No further approval is needed to read it, and whether a Management Client asks the person again before each read is its own policy ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).
