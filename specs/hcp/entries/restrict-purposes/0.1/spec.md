---
slug: hcp/entries/restrict-purposes
version: "0.1"
title: "HCP — Restrict Entry Purposes"
summary: A Management Client, acting for the Authorizing Principal, narrows the purposes a Human Context Protocol entry may be disclosed for, or removes that narrowing, and receives the entry back; carries the HCP 2.0-draft-3 Restrict purposes operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Restrict purposes
category: data-exchange
keywords: [hcp, human-context-protocol, personal-context, purpose, purpose-limitation, governance]
parties:
  - role: Management Client (HCP), acting for the Authorizing Principal
    requirement: REQUIRED
    member: issuer
  - role: HCP Provider
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The Authorizing Principal's purpose list decides which requests may receive the entry (HCP 4.2.6), and setting it
    exercises the person's own authority: the request is signed, so the provider can show which Management Client
    changed it, and when, independently of the transport. The response returns the whole entry, the person's personal
    context, so it is signed too.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task changes which purposes an entry may be disclosed for, acts for the person and discloses their context,
    so it is consequential; a re-delivered change must be placeable in the provider's duplicate-execution window.
sideEffects:
  level: mutating
  rationale: >-
    The entry's `classification.permittedPurposes` is set or removed, so that a request naming a purpose outside it no
    longer receives the entry, or, when the narrowing is removed, receives it again where the grant and the asserter's
    list allow (HCP 4.2.6). No version is created. A later change restores any earlier list, so the change is
    recoverable.
exposure:
  discloses: secret
  actsAsSubject: true
  ingests: secret
  rationale: >-
    The Management Client exercises the Authorizing Principal's authority over their own context. The request carries
    the Management Client's access token, a bearer secret. The response returns the whole entry, which is personal
    context and may be in a High-sensitivity category such as `health_context`.
retention:
  class: durable
  rationale: >-
    The provider appends a `context.restricted` audit event naming the prior and new lists (HCP 4.2.6, 8.1, 8.2), which
    is the person's record of what the entry could be used for, and when.
maxDocumentBytes:
  response: 262144
  rationale: >-
    One whole entry, which may carry a verifiable credential as evidence (HCP 4.2.2), plus its provenance,
    relationships, envelope and proof. This is a framework bound, not an HCP requirement; HCP sets no size limit.
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence", here the Management
      Client's access token.
    retryable: false
  - code: hcp:versionConflict
    meaning: >-
      HCP `version-conflict` (HCP 5.5): `expectedVersion` is not the entry's current `record.version` (HCP 5.4). Nothing
      changes. The Management Client shows the person the current version before asking again.
    retryable: false
related:
  - hcp/entries/set-categories
  - hcp/entries/confirm
  - hcp/entries/query
---

## Abstract

The **HCP — Restrict Entry Purposes** Trust Task carries the Restrict purposes operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 4.2.6 and 5.4; `PUT /v2/entries/{entryId}/purposes` in HCP 9.2 and 9.4.10). A *Management Client*, acting for the *Authorizing Principal*, tells an *HCP Provider* to set the entry's `classification.permittedPurposes`: the purposes, among those the asserter permitted, for which the person lets the entry be disclosed. Sending none removes the person's narrowing. The provider records a `context.restricted` audit event and returns the entry.

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)). The other change to an entry's classification that only the person can make, changing its categories, is [`hcp/entries/set-categories`](../../set-categories/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on permitted purposes; the ones that bear on the document are restated below, unchanged. Conformance to HCP is HCP's ([SPEC §9.4](/SPEC.md#94-carried-protocols)): a provider **MUST NOT** list a conformance identifier for this carriage in its HCP `conformsTo`, and advertises the task through `trust-task-discovery`, *pending HCP confirmation* ([question 7](#pending-hcp-confirmation)).

## Authorization

The authority is **the Authorizing Principal's own authority over the uses of their context**, which a Management Client exercises for them (HCP 4.2.6, 6.1.9). It is evidenced by the Management Client's access token in `accessToken`, which the provider validates exactly as over HCP's HTTP binding. Restricting an entry's purposes is a governance operation (HCP 4.2.1, 4.2.6): any caller other than a Management Client acting for the Authorizing Principal receives not authorized (HCP 6.1.9, 9.2), carried as `permissionDenied`. A grant held by a Context Client never authorizes it, whatever its scopes (HCP 6.1.9).

Under the Delegated Authorization Profile, a Management Client may act for the Context Subject or for a delegate, within what HCP 17 allows each (HCP 6.1.9). A delegate's governance operations reach only entries in categories within the delegation's scope (HCP 17.4); an entry outside it is refused with `permissionDenied`, exactly as an entry that does not exist.

The REQUIRED `proof` establishes which Management Client VID sent the document and that it is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). It is not the authorization: the token is. A provider that knows which VID it registered the Management Client under **SHOULD** check that the `issuer` is that client.

## Definitions

- **Management Client**, **Context Client**, **HCP Provider**, **Context Subject**, **Authorizing Principal**, **delegate**, **entry**, **grant** and **purpose code** have their HCP 2.0-draft-3 meanings.
- The [`Entry`](../../../_shared/0.1/context-entry.schema.json) shape is shared in [`hcp/_shared/0.1`](../../../_shared/0.1/common.schema.json). It is open, as HCP's own schema is: a member it does not declare is accepted, never rejected, and a provider preserves an entry member it does not define (HCP 4.2, A.3).
- HCP defines no request or result schema for this operation. The members come from HCP 9.2 (the path) and 9.4.10 (the body and the result).

### HCP field mapping

| HCP field | Member | Notes |
|---|---|---|
| path parameter `{entryId}` (`PUT /v2/entries/{entryId}/purposes`, HCP 9.2) | `entryId` | Moved from the path into the payload, unchanged. A Management Client names the entry by its `id` (HCP 9.4.5). |
| request `permittedPurposes` | `permittedPurposes` | Unchanged, null included: null removes the narrowing (HCP 9.4.10). At least 1 and at most 256 items, framework bounds HCP does not set. |
| request `expectedVersion` | `expectedVersion` | Unchanged. |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result: the entry (`200`, HCP 9.4.10) | `entry` | **Wrapped**: HCP's result body is the entry itself; the carriage puts it in the member `entry`, unchanged, so the payload stays an object the carriage can extend. |

### Parties

| HCP role | Trust Task member |
|---|---|
| Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../query/0.1/spec.md#parties), a provider **MUST** make its VID verifiably bound to its HCP identity (its discovery document, HCP 19.1), and a Management Client **MUST** check that binding before it sends a token to a VID.

## Request

The Management Client sends the new list to the provider. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json). Every member is REQUIRED.

- `entryId` is the `id` of the entry.
- `permittedPurposes` is the new list of purpose codes, which replaces the old one whole, or null, which removes the person's narrowing (HCP 9.4.10). It is REQUIRED either way, so that removing the narrowing is always an explicit act. A list is never empty: the schema requires at least one purpose, a framework bound, since HCP defines no meaning for an empty list.
- `expectedVersion` is the `record.version` the Management Client showed the person (HCP 5.4).
- `accessToken` is the Management Client's token.

### Limiting a medication entry to care coordination

```json
{
  "id": "urn:uuid:b0c30bfa-da2d-478a-8396-d5ad53fc362a",
  "type": "https://trusttasks.org/spec/hcp/entries/restrict-purposes/0.1#request",
  "issuer": "did:web:wallet.person.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T09:00:00Z",
  "threadId": "urn:uuid:06ac83ac-1608-4a11-9c5f-62774a997a2f",
  "payload": {
    "entryId": "urn:uuid:16bc10cb-304a-4d25-802a-254d621e3cae",
    "permittedPurposes": ["https://purposes.example/care-coordination"],
    "expectedVersion": 3,
    "accessToken": "hcp_mt_7a2d9c4e1b6f3058"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T09:00:00Z",
    "verificationMethod": "did:web:wallet.person.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z5Qm3vN7kLcT8xB1hR4pW6sY2dF9gJ7aZe1uK5oVbC4iM3kD8tH2yP6xS9wE1rA3jU7fG4lO2pN6vX9zB5bT8cW1"
  }
}
```

## Response

The provider answers with the entry. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): `entry`, the whole entry with its new `classification.permittedPurposes`. Where the narrowing was removed, the member is absent from the entry, since an absent list sets no limit (HCP 4.2.6). Its `record.version` is unchanged, since a change of `classification` creates no version (HCP 4.2.1). HCP's result body is the entry itself; the carriage wraps it in `entry` (see [HCP field mapping](#hcp-field-mapping)). Failures use `trust-task-error`, not a `#response` document (see [Errors](#errors)).

### The entry, now limited to care coordination

```json
{
  "id": "urn:uuid:c152cb25-2302-4954-a91b-e51647c30c41",
  "type": "https://trusttasks.org/spec/hcp/entries/restrict-purposes/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:wallet.person.example",
  "issuedAt": "2026-11-25T09:00:01Z",
  "threadId": "urn:uuid:06ac83ac-1608-4a11-9c5f-62774a997a2f",
  "payload": {
    "entry": {
      "schema": "https://hcp.example/spec/entry/2.0",
      "id": "urn:uuid:16bc10cb-304a-4d25-802a-254d621e3cae",
      "type": ["ContextEntry"],
      "subject": { "id": "https://context.provider.example/subjects/p-7f3a9c", "relationship": null },
      "content": {
        "profile": "https://hcp.example/spec/profiles/general-context/2.0",
        "value": "Takes 10 mg of lisinopril each morning.",
        "language": "en"
      },
      "assertion": { "declarationType": "person-declared" },
      "classification": {
        "categories": ["health_context"],
        "permittedPurposes": ["https://purposes.example/care-coordination"]
      },
      "record": {
        "status": "active",
        "version": 3,
        "provenance": {
          "assertedBy": { "id": "https://context.provider.example/subjects/p-7f3a9c", "assurance": "authenticated" },
          "assertedAt": "2026-10-03T07:45:00Z",
          "source": "person-input",
          "submittedByClient": {
            "id": "https://wallet.person.example/clients/app",
            "name": "Person Wallet",
            "operator": { "id": "https://wallet.person.example", "name": "Person Wallet Co" }
          },
          "authorizedUnder": "urn:uuid:6d0e0734-8161-42bb-89a9-056debd5ed4e",
          "recordedByProvider": "https://context.provider.example",
          "recordedAt": "2026-10-03T07:45:01Z"
        }
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T09:00:01Z",
    "verificationMethod": "did:web:context.provider.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z2Xv9pT4mKcR7nB3hW1xL8sY5dF2gJ6aZe9uQ4oVnC7iN1kB5tH3yD8xS2wE6rA9jU4fG1lO7pK2vX5zM8bT6cQ3"
  }
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows.

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request` | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: the caller is not a Management Client acting for the Authorizing Principal (HCP 4.2.6, 6.1.9), a delegate names an entry outside the delegation's scope (HCP 17.4), or the entry does not exist, including one hard-deleted | `permissionDenied`, with one fixed `message` whatever the reason, which never discloses whether the entry exists (HCP 5.5, 9.7, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| conflict | `idempotency-conflict` | Does not arise: HCP defines it for a retried request with an idempotency key (HCP 9.6), and this operation carries none. A re-delivered document is handled under [Repeated requests](#repeated-requests). |
| version conflict | `version-conflict`: `expectedVersion` is not the entry's current version (HCP 5.4) | `hcp:versionConflict`, `retryable: false`. Nothing changes. |
| credential invalid | `credential-invalid` | Does not arise: it concerns a credential written as an entry's content (HCP 14.6), and a restriction writes no content; no credential's proof covers `classification` (HCP 4.2.1). |
| subject unavailable | `subject-unavailable` | `unavailable`. Under end-to-end encrypted storage, the request needs the Context Subject's trust domain, which cannot be reached (HCP 4.8); the Management Client may retry later. |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion` |

A Management Client that does not know `hcp:versionConflict` reads it as `taskFailed` ([SPEC §8.5](/SPEC.md#85-extension-by-individual-trust-task-specifications)) and still honours `retryable: false`.

## Rules carried from HCP

- **Two lists.** `assertion.permittedPurposes` is set by the asserter and belongs to a version; only a revision changes it. `classification.permittedPurposes` is the Authorizing Principal's, set only by this governance operation (HCP 4.2.6, 4.2.1).
- **Only narrows.** A purpose in `classification.permittedPurposes` that `assertion.permittedPurposes` does not list has no effect. An entry **MAY** be disclosed for a purpose only where each list present includes it; an absent list sets no limit (HCP 4.2.6). A provider stores the list as sent; a listed purpose the asserter did not permit is not an error, it simply permits nothing.
- **Never widens a grant.** A request naming a purpose the grant did not approve for the entry's categories fails as before, whatever either list says (HCP 4.2.6).
- **Removing the narrowing.** A `permittedPurposes` of null removes the Authorizing Principal's narrowing (HCP 9.4.10); the entry is then limited only by the asserter's list, if any, and by the grant.
- **No new version.** `classification` may change without a new version (HCP 4.2.1).
- **Audit.** The provider appends `context.restricted`, recording `from` and `to`, each a list of purpose codes, or null where there was or is no narrowing (HCP 4.2.6, 8.1, 8.2), durable before this task's response is sent (HCP 8.4).

## Repeated requests

A re-delivered document with the same `id` and content is absorbed by [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements): it is not executed twice, produces no second `context.restricted` event, and is answered with the original response.

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 7 — conformance claims.** HCP defines no conformance identifier for Trust Task carriage, so a provider **MUST NOT** claim this carriage in its HCP `conformsTo`; it advertises the task with [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead.

## What HCP requires that a document cannot express

- Applying both purpose lists, and the grant's approved purposes, to every later request (HCP 4.2.6, 7.1).
- The `context.restricted` audit event and its durability (HCP 8.1, 8.4).

## Security & Privacy

### Data carried

The request carries an entry identifier, a list of purpose codes or null, a version number, and the Management Client's access token, a bearer secret that **MUST** travel only to the provider. The response carries the whole entry: the person's context, and its provenance, which names the clients and operators that wrote it; it may be in a High-sensitivity category and may carry a credential as evidence. A producer **MUST NOT** put the token or any context in `ext` or in any `message`.

Because the response proof is REQUIRED ([SPEC §7.3 item 8](/SPEC.md#73-specification-requirements)), the entry reaches the Management Client as a signed assertion any third party can verify. The Management Client **MUST NOT** pass the signed response, or any part of it, to anyone the person has not chosen to share it with.

### Correlation

The provider learns which Management Client restricted which entry to which purposes and when; that is the record the person needs. The purposes chosen say something about the entry and the person (limiting an entry to care coordination suggests what it concerns). The entry's `id` is never disclosed to a Context Client (HCP 7.4). `threadId` **SHOULD** be fresh per change.

### Retention

The provider keeps the `context.restricted` audit event (HCP 8.1), with the prior and new lists, readable by the Context Subject (HCP 8.3). The signed request is evidence of who changed the list and when, and a provider **SHOULD** keep it with that event. A Management Client keeps the returned entry only as long as it needs it to show the person. A narrowing stops later disclosures for other purposes; it does not recall copies already disclosed, which their `useUntil` governs (HCP 7.6).

### Consent/purpose

This task is itself a purpose limitation: the person decides which purposes the entry may serve. Removing the narrowing lets the entry be disclosed again for every purpose the grant and the asserter allow. A Management Client **SHOULD** show the person, before sending the request, the asserter's list and the purposes the person's list would add or remove, explaining that a purpose the asserter did not permit has no effect (HCP 4.2.6); whether it asks for a further confirmation is its own policy ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).
