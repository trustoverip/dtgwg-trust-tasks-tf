---
slug: hcp/entries/confirm
version: "0.1"
title: "HCP — Confirm or Reject Entry"
summary: A Management Client, acting for the Context Subject, confirms or rejects the current version of a Human Context Protocol entry, and receives the entry back; carries the HCP 2.0-draft-3 Confirm or reject operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Confirm or reject
category: data-exchange
keywords: [hcp, human-context-protocol, personal-context, confirmation, conflict, governance]
parties:
  - role: Management Client (HCP), acting for the Context Subject
    requirement: REQUIRED
    member: issuer
  - role: HCP Provider
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    A confirmation is the Context Subject's own endorsement or rejection of a statement about them, and it decides
    which entry is authoritative in a conflict (HCP 3.8, 4.7.3), so the request is signed: the provider can show which
    Management Client recorded it, and when, independently of the transport. The response returns the whole entry,
    the person's personal context, so it is signed too: the Management Client can show what the provider recorded.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task changes an entry's standing and status, acts for the person and discloses their context, so it is
    consequential; a re-delivered confirmation must be placeable in the provider's duplicate-execution window.
sideEffects:
  level: mutating
  rationale: >-
    A confirmation record is appended to the entry's current version, and the status transitions of HCP 4.7.3 apply:
    a rejected entry becomes `deleted`, an accepted one `active`, and another entry in the conflict may return to
    `active` or become `deprecated`. No version is created (HCP 3.8). A later confirmation or a revision changes the
    outcome, and a rejected entry is held until it is hard-deleted (HCP 4.2.4), so the change is recoverable.
exposure:
  discloses: secret
  actsAsSubject: true
  ingests: secret
  rationale: >-
    The Management Client exercises the Context Subject's authority over their own context. The request carries the
    Management Client's access token, a bearer secret. The response returns the whole entry, which is personal
    context and may be in a High-sensitivity category such as `health_context`.
retention:
  class: durable
  rationale: >-
    The confirmation record becomes part of the version's provenance (HCP 3.8, 4.2.1), and the provider appends a
    `context.confirmed` audit event, and `conflict.resolved` where the act resolves a conflict (HCP 8.1, 4.7.3). Both
    are the person's record of what they endorsed.
maxDocumentBytes:
  response: 262144
  rationale: >-
    One whole entry, which may carry a verifiable credential as evidence (HCP 4.2.2), plus its confirmation records,
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
  - hcp/entries/query
  - hcp/entries/set-categories
  - hcp/entries/restrict-purposes
  - hcp/entries/hard-delete
---

## Abstract

The **HCP — Confirm or Reject Entry** Trust Task carries the Confirm or reject operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 3.8, 4.7.3 and 5.4; `POST /v2/entries/{entryId}/confirmation` in HCP 9.2 and 9.4.9). A *Management Client*, acting for the *Context Subject*, tells an *HCP Provider* that the person confirms, or rejects, the current version of an entry. The provider appends a confirmation record to that version, applies the status transitions HCP defines for resolving a conflict, records a `context.confirmed` audit event, and returns the entry.

Confirmation is how a person endorses a statement someone else made about them, such as a model's inference, without making it their own: the entry's `declarationType` and `assertedBy` are unchanged (HCP 3.8). It is also how a person resolves a conflict between entries by accepting or rejecting one of them (HCP 4.7.3).

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on confirmation and conflict resolution; the ones that bear on the document are restated below, unchanged. Conformance to HCP is HCP's ([SPEC §9.4](/SPEC.md#94-carried-protocols)): a provider **MUST NOT** list a conformance identifier for this carriage in its HCP `conformsTo`, and advertises the task through `trust-task-discovery`, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)). HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1); this specification takes it that a carriage other than that binding may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and carries this one as a Trust Task on that reading, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

## Authorization

The authority is **the Context Subject's own authority over statements about them**, which a Management Client exercises for them (HCP 3.8, 6.1.9). It is evidenced by the Management Client's access token in `accessToken`, which the provider validates exactly as over HCP's HTTP binding. Confirming or rejecting an entry is a governance operation: any caller other than a Management Client acting for the Authorizing Principal receives not authorized (HCP 5.4, 6.1.9, 9.2), carried as `permissionDenied`. A grant held by a Context Client never authorizes it, whatever its scopes (HCP 6.1.9). HCP names only its HTTP Binding as the carriage for a governance operation (HCP 5.4, 9.1); this specification accepts the operation as a Trust Task from a Management Client acting for the Authorizing Principal, and from no other caller (HCP 6.1.9), *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

Under the Delegated Authorization Profile, a Management Client may act for the Context Subject or for a delegate, within what HCP 17 allows each (HCP 6.1.9). HCP 3.8 names the Context Subject as the one who confirms or rejects; this specification reads the delegate's governance authority (HCP 17.4) as including it, so a delegate's Management Client may confirm or reject an entry in categories within the delegation's scope, *pending HCP confirmation* ([question 12](#pending-hcp-confirmation)). A delegate's governance operations reach only entries in categories within the delegation's scope (HCP 17.4); an entry outside it is refused with `permissionDenied`, exactly as an entry that does not exist.

The REQUIRED `proof` establishes which Management Client VID sent the document and that it is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). It is not the authorization: the token is. A provider that knows which VID it registered the Management Client under **SHOULD** check that the `issuer` is that client.

## Definitions

- **Management Client**, **Context Client**, **HCP Provider**, **Context Subject**, **Authorizing Principal**, **delegate**, **entry**, **confirmation record**, **authoritative** and **conflict** have their HCP 2.0-draft-3 meanings.
- The [`Entry`](../../../_shared/0.1/context-entry.schema.json) shape is shared in [`hcp/_shared/0.1`](../../../_shared/0.1/common.schema.json). It is open, as HCP's own schema is: a member it does not declare is accepted, never rejected, and a provider preserves an entry member it does not define (HCP 4.2, A.3).
- HCP defines no request or result schema for this operation. The members come from HCP 9.2 (the path) and 9.4.9 (the body and the result).

### HCP field mapping

| HCP field | Member | Notes |
|---|---|---|
| path parameter `{entryId}` (`POST /v2/entries/{entryId}/confirmation`, HCP 9.2) | `entryId` | Moved from the path into the payload, unchanged. A Management Client names the entry by its `id` (HCP 9.4.5). |
| request `disposition` | `disposition` | Unchanged. |
| request `expectedVersion` | `expectedVersion` | Unchanged. |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result: the entry (`200`, HCP 9.4.9) | `entry` | **Wrapped**: HCP's result body is the entry itself; the carriage puts it in the member `entry`, unchanged, so the payload stays an object the carriage can extend. |

### Parties

| HCP role | Trust Task member |
|---|---|
| Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../query/0.1/spec.md#parties), a provider **MUST** make its VID verifiably bound to its HCP identity (its discovery document, HCP 19.1), and a Management Client **MUST** check that binding before it sends a token to a VID.

## Request

The Management Client sends the person's decision to the provider. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json). Every member is REQUIRED.

- `entryId` is the `id` of the entry.
- `disposition` is `confirmed` or `rejected`.
- `expectedVersion` is the `record.version` the Management Client showed the person (HCP 5.4).
- `accessToken` is the Management Client's token.

Before the person resolves a conflict, the Management Client **SHOULD** show them, for each entry, its basis, declaration type, standing and permitted purposes, and any proposal with its rationale (HCP 4.7.3).

### Accepting an inferred travel preference that was held in a conflict

```json
{
  "id": "urn:uuid:61929447-71df-4152-a0d8-49fdf48fcf7f",
  "type": "https://trusttasks.org/spec/hcp/entries/confirm/0.1#request",
  "issuer": "did:web:wallet.person.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T09:00:00Z",
  "threadId": "urn:uuid:c8ed75e7-8ce6-429e-8be8-92937351db7a",
  "payload": {
    "entryId": "urn:uuid:5affd201-734e-4217-a7b6-dc4dd177f561",
    "disposition": "confirmed",
    "expectedVersion": 2,
    "accessToken": "hcp_mt_9d4b2e7a1c3f5068"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T09:00:00Z",
    "verificationMethod": "did:web:wallet.person.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z5Bv2nT8kLqW3xC7hR1pM9sY6dF4gJ8aZe3uN6oVbK2iQ5kD7tH1yP4xS2wE9rA6jU3fG7lO1pK5vX8zN2bT4cW7"
  }
}
```

## Response

The provider answers with the entry. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): `entry`, the whole entry as it stands after the confirmation, with the new record last in its current version's `record.provenance.confirmations` and its `record.status` as the transitions of HCP 4.7.3 left it. HCP's result body is the entry itself; the carriage wraps it in `entry` (see [HCP field mapping](#hcp-field-mapping)). Failures use `trust-task-error`, not a `#response` document (see [Errors](#errors)).

### The entry, accepted and active again

The inference had been held `conflicted` against a model inference from another client. The person accepted it, so it is `active`; the other entry, not being authoritative, is unaffected by the acceptance.

```json
{
  "id": "urn:uuid:7770add8-2005-4d6b-98dc-6d74014c81bb",
  "type": "https://trusttasks.org/spec/hcp/entries/confirm/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:wallet.person.example",
  "issuedAt": "2026-11-25T09:00:01Z",
  "threadId": "urn:uuid:c8ed75e7-8ce6-429e-8be8-92937351db7a",
  "payload": {
    "entry": {
      "schema": "https://hcp.example/spec/entry/2.0",
      "id": "urn:uuid:5affd201-734e-4217-a7b6-dc4dd177f561",
      "type": ["ContextEntry"],
      "subject": { "id": "https://context.provider.example/subjects/p-7f3a9c", "relationship": null },
      "content": {
        "profile": "https://hcp.example/spec/profiles/general-context/2.0",
        "value": "Prefers an aisle seat on flights longer than three hours.",
        "language": "en"
      },
      "assertion": {
        "declarationType": "model-inferred",
        "confidence": "medium",
        "basis": { "rationale": "Chose an aisle seat on each of the last four long-haul bookings." }
      },
      "classification": { "categories": ["general_preferences"], "dimension": "flight-seat" },
      "record": {
        "status": "active",
        "version": 2,
        "provenance": {
          "assertedBy": { "id": "https://travel-assistant.example/clients/planner", "assurance": "authenticated" },
          "assertedAt": "2026-11-20T14:02:00Z",
          "source": "integration",
          "submittedByClient": {
            "id": "https://travel-assistant.example/clients/planner",
            "name": "Trip Planner",
            "operator": { "id": "https://travel-assistant.example", "name": "Travel Assistant Ltd" }
          },
          "authorizedUnder": "urn:uuid:eb54c220-b64f-40ad-a794-37a48095ce0b",
          "recordedByProvider": "https://context.provider.example",
          "recordedAt": "2026-11-20T14:02:01Z",
          "confirmations": [
            {
              "disposition": "confirmed",
              "confirmedBy": { "id": "https://context.provider.example/subjects/p-7f3a9c" },
              "confirmedAt": "2026-11-25T09:00:01Z"
            }
          ]
        },
        "relationships": [
          {
            "type": "contradicts",
            "target": "urn:uuid:b4f00822-4753-40a6-84cf-8fc51693b59e",
            "assertedBy": { "role": "hcp-provider", "id": "https://context.provider.example" },
            "recordedAt": "2026-11-22T08:15:00Z"
          }
        ]
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T09:00:01Z",
    "verificationMethod": "did:web:context.provider.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3Wd8pR5vMcT2nB9hK6xL1sY4dF7gJ3aZe8uQ5oVnC2iN7kB1tH4yD6xS3wE8rA5jU2fG9lO6pK1vX4zM7bT3cQ9"
  }
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows.

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request` | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: the caller is not a Management Client acting for the Authorizing Principal (HCP 6.1.9), a delegate names an entry outside the delegation's scope (HCP 17.4), or the entry does not exist, including one hard-deleted | `permissionDenied`, with one fixed `message` whatever the reason, which never discloses whether the entry exists (HCP 5.5, 9.7, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| conflict | `idempotency-conflict` | Does not arise: HCP defines it for a retried request with an idempotency key (HCP 9.6), and this operation carries none. A re-delivered document is handled under [Repeated requests](#repeated-requests). |
| version conflict | `version-conflict`: `expectedVersion` is not the entry's current version (HCP 5.4) | `hcp:versionConflict`, `retryable: false`. Nothing changes. |
| credential invalid | `credential-invalid` | Does not arise: it concerns a credential written as an entry's content (HCP 14.6), and a confirmation writes no content. |
| subject unavailable | `subject-unavailable` | `unavailable`. Under end-to-end encrypted storage, the request needs the Context Subject's trust domain, which cannot be reached (HCP 4.8); the Management Client may retry later. |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion` |

A Management Client that does not know `hcp:versionConflict` reads it as `taskFailed` ([SPEC §8.5](/SPEC.md#85-extension-by-individual-trust-task-specifications)) and still honours `retryable: false`.

## Rules carried from HCP

- **The record.** The provider appends a confirmation record, with the `disposition`, `confirmedBy` and `confirmedAt`, to the entry's current version (HCP 3.8, 9.4.9). It belongs to that version's provenance, which is otherwise fixed (HCP 4.2.1).
- **No new version.** A confirmation does not change the entry's `declarationType` or its `assertedBy`, and does not create a new version (HCP 3.8). A person who edits the entry while confirming it makes a new version, their own `person-declared` statement; that is a revision, not this task (HCP 3.8).
- **The version it was made on.** A confirmation applies to the version it was made on. A new version starts with no confirmations, so a revision never inherits the person's endorsement of earlier wording (HCP 3.8). `expectedVersion` guarantees that the version confirmed is the one the person saw (HCP 5.4).
- **Rejecting.** The provider appends a `rejected` record and sets the entry's `record.status` to `deleted`. Where another entry in the conflict is `conflicted` and has no other unresolved conflict, the provider **MUST** return it to `active`, appending no confirmation record to it. An authoritative entry is unaffected (HCP 4.7.3). A Context Client **MUST NOT** revise an entry whose current version the person has rejected (HCP 3.8).
- **Accepting.** The provider appends a `confirmed` record and sets the entry's `record.status` to `active`. Where another entry in the conflict was authoritative, the provider **MUST** set its `record.status` to `deprecated`, unless it is `issuer-attested`: an attestation is never deprecated by the person's acceptance of another entry, and both stay active (HCP 4.7.3, 4.4).
- **Standing.** An entry whose current version's most recent confirmation record is `confirmed` is authoritative in a conflict, whoever asserted it (HCP 3.8). Consumers determine standing from assurance and confirmation records, not from `declarationType` alone (HCP 3.8).
- **Audit.** The provider appends `context.confirmed`, recording the `disposition` and `version` (HCP 8.1, 8.2), and `conflict.resolved` where the act resolves a conflict (HCP 4.7.3), each durable before this task's response is sent (HCP 8.4).

## Repeated requests

A re-delivered document with the same `id` and content is absorbed by [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements): it is not executed twice, appends no second confirmation record and no second `context.confirmed` event, and is answered with the original response. A new document confirming the same version again is a new act and appends a new record; HCP does not forbid it (HCP 3.8).

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 1 — conformance claims.** HCP defines no conformance identifier for Trust Task carriage, so a provider **MUST NOT** claim this carriage in its HCP `conformsTo`; it advertises the task with [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md) instead. The binding identifier for the carriage is the same question as [question 37](#pending-hcp-confirmation): whether a carriage other than the HTTP Binding may carry governance operations at all.
- **Question 12 — a delegate confirming or rejecting.** HCP 3.8 says confirmation is performed by the Context Subject, and HCP 17.4 lets a delegate perform governance operations on entries within the delegation's scope. This specification reads the two together: a delegate's Management Client may confirm or reject an entry in categories within the delegation's scope, and no other (see [Authorization](#authorization)).
- **Question 37 — governance over a carriage other than the HTTP Binding.** HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1). This specification takes it that another carriage, here a Trust Task, may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and applies every rule HCP places on the operation unchanged (see [Conformance](#conformance)).

## What HCP requires that a document cannot express

- The status transitions on the other entries in a conflict (HCP 4.7.3): the response carries only the entry named.
- Refusing a Context Client's revision of a rejected version (HCP 3.8).
- The `context.confirmed` and `conflict.resolved` audit events and their durability (HCP 8.1, 8.4).
- Carriage by the HTTP Binding: HCP carries governance operations only over that binding (HCP 1.6, 5.4, 9.1), and a Trust Task is not it. This specification carries the operation for a Management Client acting for the Authorizing Principal (HCP 6.1.9) instead, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

## Security & Privacy

### Data carried

The request carries an entry identifier, the person's decision and a version number, and the Management Client's access token, a bearer secret that **MUST** travel only to the provider. The response carries the whole entry: the person's context, its provenance, which names the clients and operators that wrote it, and its relationships to other entries; it may be in a High-sensitivity category and may carry a credential as evidence. A producer **MUST NOT** put the token or any context in `ext` or in any `message`.

Because the response proof is REQUIRED ([SPEC §7.3 item 8](/SPEC.md#73-specification-requirements)), the entry reaches the Management Client as a signed assertion any third party can verify. The Management Client **MUST NOT** pass the signed response, or any part of it, to anyone the person has not chosen to share it with.

### Correlation

The provider learns which Management Client recorded which decision on which entry and when; that is the record the person needs. The entry's `id` is never disclosed to a Context Client (HCP 7.4), so it is known only to the provider and the person's Management Clients. `threadId` **SHOULD** be fresh per confirmation.

### Retention

The confirmation record stays in the version's provenance for as long as the entry is held, and is exported with it (HCP 5.4.1). The provider keeps the `context.confirmed` audit event (HCP 8.1), readable by the Context Subject (HCP 8.3). The signed request is evidence of who recorded the decision and when, and a provider **SHOULD** keep it with that event. A Management Client keeps the returned entry only as long as it needs it to show the person.

### Consent/purpose

Confirming endorses a statement about the person, and makes it authoritative in a conflict (HCP 3.8); rejecting removes it from what Context Clients can read. A Management Client **SHOULD** show the person the entry's content, basis, declaration type and standing, and the version it will confirm, before sending the request; whether it asks for a further confirmation is its own policy ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).
