---
slug: hcp/export
version: "0.1"
title: "HCP — Export"
summary: A Management Client, acting for the person, asks a Human Context Protocol provider for the person's own export of their context; carries the HCP 2.0-draft-3 Export operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Export
category: data-exchange
keywords: [hcp, human-context-protocol, export, portability, governance]
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
    The response discloses the person's whole context, so it carries the provider's signature: the person can show
    which provider released which context, when, and that the export is complete as the provider made it, which is
    what moving it to another provider relies on. The request proof is REQUIRED, as for every Management Client read
    whose response is secret: the access token is the authority, but a signed request lets the provider refuse a
    token lifted from a stored or forwarded document and sent from another VID, and shows which Management Client
    asked for the export.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task discloses secret material and so is consequential; a re-delivered export request must be placeable in
    the provider's duplicate-execution window, so that it is absorbed instead of disclosing and auditing twice.
sideEffects:
  level: none
  rationale: >-
    A read. The provider appends a `context.exported` audit event (HCP 5.4.1, 8.1), but that is recordkeeping HCP
    requires of every export, not a change the Management Client asked for.
exposure:
  discloses: secret
  actsAsSubject: false
  ingests: secret
  rationale: >-
    The response discloses the Context Subject's whole context, every status included, which may hold health and
    other High-sensitivity categories, and optionally their audit log. The request carries the Management Client's
    access token, a bearer secret.
retention:
  class: durable
  rationale: >-
    The provider keeps the `context.exported` audit event, which records the options used (HCP 5.4.1, 8.2), so the
    person can see when their context left the provider. The export itself is the person's own copy, which they keep
    to move or archive their context.
    The access token is not retained: a kept request holds its digest instead (see Security & Privacy, The access token).
maxDocumentBytes:
  response: 16777216
  rationale: >-
    The export of a person's context in one document: every entry, which may carry verifiable credentials as
    evidence, the standing rules, directives and delegations, and, where asked, every superseded version and the
    audit log. 16 MiB is the framework's ceiling for a declared bound; it is a framework bound, not an HCP
    requirement, since HCP sets no size limit. An export that would exceed it is refused with `hcp:exportTooLarge`,
    and is taken over HCP's own HTTP binding, or with narrower options, instead.
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence", here the Management
      Client's access token.
    retryable: false
  - code: hcp:exportTooLarge
    meaning: >-
      The export would exceed this task's declared maxDocumentBytes response bound, so it cannot be carried in one
      Trust Task document. Not an HCP error: HCP sets no size limit on an export (HCP 5.4.1, 9.4.8). Retrying the
      same request cannot succeed; the Management Client takes the export over HCP's HTTP binding (`GET /v2/export`,
      HCP 9.4.8), or asks again with narrower options (without versions or the audit log, or excluding deleted
      entries). Pending HCP confirmation (question 33 in the spec's "Pending HCP confirmation" section).
    retryable: false
related:
  - hcp/import
  - hcp/audit/read
  - hcp/entries/hard-delete
---

## Abstract

The **HCP — Export** Trust Task carries the Export operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 5.4.1 and 5.4; `GET /v2/export` in HCP 9.4.8). A *Management Client*, acting for the *Context Subject*, asks an *HCP Provider* for the Context Subject's own export: a document holding the current version of every entry the provider still holds, whatever its status, together with the person's standing rules, directives and delegations, and, where asked, superseded versions and the audit log. The provider records a `context.exported` audit event. An export is loaded into a provider with [`hcp/import`](../../import/0.1/spec.md).

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on export; the ones that bear on the document are restated below, unchanged. Conformance to HCP is HCP's ([SPEC §9.4](/SPEC.md#94-carried-protocols)): a provider **MUST NOT** list a conformance identifier for this carriage in its HCP `conformsTo`, and advertises the task through `trust-task-discovery`, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)). HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1); this specification takes it that a carriage other than that binding may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and carries this one as a Trust Task on that reading, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

This specification adds one check HCP does not make: the token binding of [The access token](#the-access-token), a carriage rule of the `hcp/*` family, not HCP's. [SPEC §9.4](/SPEC.md#94-carried-protocols) item 6 places a check the carried protocol does not make in a separate specification that does not declare `carries`. This family keeps this one in the carriage, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)), because it protects the token that the carriage itself moves from the HTTP `Authorization` header into the document, where it can be stored or forwarded with the document; a separate task could not protect the document this one carries.

## Authorization

The authority is **the Context Subject's own authority over their data**, which a Management Client exercises for them (HCP 5.4, 6.1.9). It is evidenced by the Management Client's access token in `accessToken`, which the provider validates exactly as over HCP's HTTP binding. Export is a governance operation: the Context Subject's own export of their data is not a context operation, and is not available to a Context Client (HCP 5.1). Any caller other than a Management Client acting for the Context Subject, or for a delegate, receives not authorized (HCP 6.1.9), carried as `permissionDenied`. A grant held by a Context Client never authorizes it, whatever its scopes (HCP 6.1.9); a Context Client's access is always query-scoped and minimized, and bulk retrieval of a Context Subject's entries by a Context Client is not supported (HCP 5.1). HCP names only its HTTP Binding as the carriage for a governance operation (HCP 5.4, 9.1); this specification accepts the operation as a Trust Task from a Management Client acting for the Authorizing Principal, and from no other caller (HCP 6.1.9), *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

Under the Delegated Authorization Profile, a delegate's export carries no more than the delegate could read through the operation that reads each of its members, so that an export never shows a delegate what those operations withhold:

- `entries`: only entries in categories within the delegation's scope, since a governance operation performed by a delegate reaches only those (HCP 17.4).
- `versions`: only superseded versions of the entries returned, and of those only the versions whose own categories are all within the scope, since a version recorded before a change of categories may name one outside it, *pending HCP confirmation* ([question 40](#pending-hcp-confirmation)).
- `audit`: only the events HCP 17.7 lets a delegate read, as [`hcp/audit/read`](../../audit/read/0.1/spec.md#authorization) returns them: every event of a grant they authorized and of a governance operation performed for them, and the same for any sub-delegate they appointed, and never the events of grants the Context Subject authorized (HCP 17.7, 17.10). The audit option is "equivalent to the export already required by 8.3" (HCP 5.4.1), and HCP 8.3 gives a delegate only "the part 17.7 sets out".
- `delegations`: only those [`hcp/delegations/list`](../../delegations/list/0.1/spec.md#authorization) returns a delegate: the delegations naming them (HCP 9.4.13), ended ones included, and the sub-delegations they appointed, *pending HCP confirmation* ([question 32](#pending-hcp-confirmation)).
- `rules` and `directives`: only those [`hcp/rules/list`](../../rules/list/0.1/spec.md#authorization) and [`hcp/directives/list`](../../directives/list/0.1/spec.md#authorization) return a delegate: each whose `categories` are absent or all within the scope, a rule or directive naming any category outside it being left out whole, *pending HCP confirmation* ([question 28](#pending-hcp-confirmation)).

The Context Subject's export carries every member whole (HCP 5.4.1, 17.5).

The REQUIRED `proof` establishes which Management Client VID sent the document and that it is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). It is not the authorization: the token is. A token issued to a client other than the one the `issuer` VID identifies is refused with `permissionDenied` (see [The access token](#the-access-token)).

## Definitions

- **Management Client**, **Context Client**, **HCP Provider**, **Context Subject**, **Authorizing Principal**, **delegate**, **entry**, **standing rule**, **directive**, **delegation** and **export** have their HCP 2.0-draft-3 meanings.
- The export's shape is [`Export`](../../_shared/0.1/export.schema.json) in `hcp/_shared/0.1`. It is open, as HCP's own shape is: a member it does not declare is accepted, never rejected.
- HCP defines no request schema for this operation. The request members are HCP 5.4.1's three export options, under the names 5.4.1's `options` member gives them.

### HCP field mapping

| HCP field | Member | Notes |
|---|---|---|
| query parameter `exclude=deleted` (HCP 9.4.8) | `excludeDeleted: true` | The "excluding deleted entries" option of HCP 5.4.1, under the name HCP's export `options` member gives it. |
| query parameter `include=versions` (HCP 9.4.8) | `includeVersions: true` | The "including versions" option of HCP 5.4.1. |
| query parameter `include=audit` (HCP 9.4.8) | `includeAudit: true` | The "including the audit log" option of HCP 5.4.1. `include=audit,versions` sets both. |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| response body, the export (HCP 5.4.1) | `export` | **Wrapped**: HCP's body is the export itself; here it is the value of the one member `export`, unchanged. |

### Parties

| HCP role | Trust Task member |
|---|---|
| Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../entries/query/0.1/spec.md#parties), a provider **MUST** make its VID verifiably bound to its HCP identity (its discovery document, HCP 19.1), and a Management Client **MUST** check that binding before it sends a token to a VID.

## Request

The Management Client asks the provider for the export. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

- `excludeDeleted` (OPTIONAL) omits entries whose `record.status` is `deleted`. HCP intends it for a person moving their context to another provider, where entries they deleted should not travel with it (HCP 5.4.1).
- `includeVersions` (OPTIONAL) also returns the superseded versions of each entry returned.
- `includeAudit` (OPTIONAL) also returns the Context Subject's audit log. It is a convenience, equivalent to the audit log export HCP 8.3 already requires, not an additional capability (HCP 5.4.1).
- `accessToken` (REQUIRED) is the Management Client's token.

An absent option, or one set to `false`, takes HCP's default.

### Exporting to move to another provider

```json
{
  "id": "urn:uuid:6d55146a-7760-4fd1-b22f-8099f3bac805",
  "type": "https://trusttasks.org/spec/hcp/export/0.1#request",
  "issuer": "did:web:wallet.person.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-25T09:00:00Z",
  "threadId": "urn:uuid:534fe5bb-f155-4f4f-8b5d-daa9a354f432",
  "payload": {
    "excludeDeleted": true,
    "accessToken": "hcp_mt_8f3a1c9e7b2d4f60"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T09:00:00Z",
    "verificationMethod": "did:web:wallet.person.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3Kp8vR2mXcN6wB4hT9qL1sY7dF3gJ5aZe2uP8oVkC4iM6nD1tH9yW5xS3wE7rA2jU8fG4lO6pK1vX9zN5bR3cQ7"
  }
}
```

## Response

The provider answers with the export. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): one member, `export`, holding the HCP export document unchanged (see [HCP field mapping](#hcp-field-mapping)).

- `@context`, `type` (`ContextExport`) and `schema` (`https://hcp.example/spec/export/2.0`, compared as an exact string) are HCP's, carried verbatim.
- `subject`, `exportedBy` and `exportedAt` are the Context Subject, the exporting provider and when the export was made. `options` records the options used. It is REQUIRED, and carries `includeVersions` and `includeAudit`, each `true` where that option was asked and `false` otherwise, so that the export itself says what it was asked to hold: under a `true` option, an empty or absent `versions` or `audit` means asked for and none; under a `false` one, not asked for. HCP's own shape leaves `options` optional, though HCP 5.4.1 lists it among the export's members and its template writes every option; requiring it is *pending HCP confirmation* ([question 42](#pending-hcp-confirmation)). The requirement is on the export a provider produces: [`hcp/import`](../../import/0.1/spec.md#request) accepts an export written to HCP's schema alone, without `options`, and reads its absence as not knowing what was requested.
- `entries` is the current version of each entry exported, in the form of HCP 4.2, each carrying its status.
- `versions` is present only where `includeVersions` was asked, and `audit` only where `includeAudit` was asked. A reader tells "asked for, none" from "not asked for" by `options`, never by the presence of the array: the schema carries that presence explicitly.
- Each event in `audit` is a shared [`AuditEvent`](../../_shared/0.1/audit-event.schema.json), which requires `entryId`, `relatedEntryId`, `purpose`, `purposeDescription` and `grantId`, each a value or null, *pending HCP confirmation* ([question 42](#pending-hcp-confirmation)). [`hcp/import`](../../import/0.1/spec.md#request) accepts events written to HCP's audit-event schema alone, without them.
- `rules`, `directives` and `delegations` are the Context Subject's standing rules, directives, and the delegations made for them; a delegate's export carries only those [Authorization](#authorization) sets out.

Failures use `trust-task-error`, not a `#response` document (see [Errors](#errors)).

### The export, deleted entries excluded

```json
{
  "id": "urn:uuid:12207829-5d17-4e36-9e17-72b1dcce2523",
  "type": "https://trusttasks.org/spec/hcp/export/0.1#response",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:wallet.person.example",
  "issuedAt": "2026-11-25T09:00:02Z",
  "threadId": "urn:uuid:534fe5bb-f155-4f4f-8b5d-daa9a354f432",
  "payload": {
    "export": {
      "@context": ["https://hcp.example/contexts/v2"],
      "type": "ContextExport",
      "schema": "https://hcp.example/spec/export/2.0",
      "subject": { "id": "did:web:person.example" },
      "exportedBy": "https://context.provider.example",
      "exportedAt": "2026-11-25T09:00:02Z",
      "options": { "excludeDeleted": true, "includeVersions": false, "includeAudit": false },
      "entries": [
        {
          "@context": ["https://hcp.example/contexts/v2"],
          "schema": "https://hcp.example/spec/entry/2.0",
          "id": "urn:uuid:53deb29f-c7ae-450b-a129-82c40041336a",
          "type": ["ContextEntry"],
          "subject": { "id": "did:web:person.example" },
          "content": {
            "profile": "https://hcp.example/spec/profiles/general-context/2.0",
            "value": "Vegetarian; eats fish occasionally.",
            "language": "en"
          },
          "assertion": { "declarationType": "person-declared" },
          "classification": { "categories": ["general_preferences"] },
          "record": {
            "status": "active",
            "version": 2,
            "provenance": {
              "assertedBy": { "id": "did:web:person.example", "assurance": "authenticated" },
              "assertedAt": "2026-10-01T09:00:00Z",
              "source": "person-input",
              "submittedByClient": {
                "id": "wallet-app",
                "name": "Person Wallet",
                "operator": { "id": "wallet.person.example", "name": "Wallet Co" }
              },
              "authorizedUnder": "urn:uuid:245df6a2-f351-44c4-80e2-b836d09db543",
              "recordedByProvider": "https://context.provider.example",
              "recordedAt": "2026-10-01T09:00:01Z"
            }
          }
        }
      ],
      "rules": [
        {
          "ruleId": "urn:uuid:d312c611-65f3-488d-aaf5-b60a654e8784",
          "effect": "deny",
          "operators": ["ads.example"],
          "categories": ["health_context"],
          "setBy": "context-subject",
          "createdAt": "2026-09-14T18:20:00Z"
        }
      ]
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T09:00:02Z",
    "verificationMethod": "did:web:context.provider.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "zw3Poyx2XKHKGfru9rWYpAoPPXnhbCMuexjDmyC37VFxHbPhNBTbU9t2W7Hjwg4rrn1eopN2W6qxUdNw8MqPFzPv"
  }
}
```

### An export too large for one document

The person asked for every version and the audit log, and the export would exceed this task's 16 MiB bound, so the provider refuses with `hcp:exportTooLarge`, *pending HCP confirmation* ([question 33](#pending-hcp-confirmation)). The Management Client takes the export over HCP's HTTP binding, or asks again without the versions or the audit log.

```json
{
  "id": "urn:uuid:405593d5-e4e2-4be9-ae26-de3ee9b80a79",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:wallet.person.example",
  "issuedAt": "2026-11-25T09:10:01Z",
  "threadId": "urn:uuid:dab7261d-d7ad-4009-878f-897939119604",
  "payload": {
    "code": "hcp:exportTooLarge",
    "message": "The export exceeds the size this task can carry; use the HTTP binding or narrower options.",
    "retryable": false
  }
}
```

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows. The provider uses the code in the right-hand column. It never sends HCP's Core error object or the HTTP binding's problem details (HCP 9.7).

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request` | `malformedRequest` |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: the caller is not a Management Client acting for the Context Subject or a delegate, including any Context Client (HCP 5.1, 6.1.9) | `permissionDenied`, with one fixed `message` whatever the reason (HCP 5.5, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| conflict | `idempotency-conflict` | Does not arise: HCP defines it for a retried request with an idempotency key (HCP 9.6), and this operation carries none. A re-delivered document is handled under [Repeated requests](#repeated-requests). |
| version conflict | `version-conflict` | Does not arise: an export names no entry version (HCP 5.4.1). |
| credential invalid | `credential-invalid` | Does not arise: it concerns a credential written as an entry's content (HCP 14.6), which a read does not do. |
| subject unavailable | `subject-unavailable` | `unavailable`. Under end-to-end encrypted storage the export needs the Context Subject's trust domain (HCP 4.8); the Management Client may retry later (HCP 5.5). |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion`. The provider never serves a different HCP version silently (HCP 5.5). |

This task adds one code that is not an HCP error: `hcp:exportTooLarge`, `retryable: false`, where the export would exceed the declared `maxDocumentBytes` response bound (see [Export size](#export-size)). A Management Client that does not know `hcp:invalidToken` or `hcp:exportTooLarge` reads it as `taskFailed` ([SPEC §8.5](/SPEC.md#85-extension-by-individual-trust-task-specifications)) and still honours `retryable: false`.

## Rules carried from HCP

- **Default content.** The export holds the current version of every entry, across all categories, that has not been hard-deleted, whatever its `record.status`: `active`, `deprecated`, `conflicted`, `expired`, `revoked`, `suspended` or `deleted`. Every entry carries its status. The export also carries the Context Subject's standing rules (HCP 5.4.1).
- **The principle.** The export contains anything whose existence the Context Subject could not otherwise infer from it (HCP 5.4.1). A soft-deleted entry is still held and still processed, so it is exported by default: a person reading a filtered export could not discover that it exists. A superseded version is announced by the current entry's `record.version`, and the audit log is separately available (HCP 8.3), so both are opt-in.
- **Hard-deleted entries.** An entry removed by hard deletion (see `hcp/entries/hard-delete`) is unrecoverable and **MUST NOT** be returned under any option (HCP 5.4.1, 5.4.2).
- **Delegations.** The delegations made for the Context Subject, ended ones included, are exported as a record. An import does not establish them: a delegation is established by an Authorization Service (HCP 5.4.1, 17.2).
- **Audit.** The provider appends a `context.exported` event for every export, recording in its `options` detail the options used (HCP 5.4.1, 8.1, 8.2, 9.4.8), durable before this task's response is sent (HCP 8.4).

## Export size

HCP sets no size bound on an export, and does not page it (HCP 5.4.1, 9.4.8). This task carries the export in one document, and declares a `maxDocumentBytes` response bound of 16 MiB, the framework's ceiling for a declared bound: a framework bound, not an HCP requirement. A provider whose export would exceed it **MUST NOT** truncate the export, since a partial export would hide what HCP's principle requires it to show; it refuses with `hcp:exportTooLarge`, and the Management Client takes the export over HCP's HTTP binding (HCP 9.4.8) or asks again with narrower options, *pending HCP confirmation* ([question 33](#pending-hcp-confirmation)). The provider appends no `context.exported` event for a refused export, since nothing was exported.

## Repeated requests

HCP requires a `context.exported` event for every export (HCP 5.4.1). Because this task is consequential, a consumer applies [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements): a re-delivered document with the same `id` and content is not executed twice, so it produces no second export and no second `context.exported` event; the provider answers it with the original response. A new request, with a new `id`, is a new export.

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 1 — conformance claims.** HCP defines no conformance identifier for Trust Task carriage, so a provider **MUST NOT** claim this carriage in its HCP `conformsTo`; it advertises the task with [`trust-task-discovery`](../../../trust-task-discovery/0.4/spec.md) instead. The binding identifier for the carriage is the same question as [question 37](#pending-hcp-confirmation): whether a carriage other than the HTTP Binding may carry governance operations at all.
- **Question 22 — the client's identity.** The `issuer` VID, authenticated by the request's `proof`, identifies the HCP client the token was issued to; where the request carries a proof, a token issued to any other client **MUST** be refused with `permissionDenied`. This token binding is a carriage rule of this family, a check HCP does not itself make (see [The access token](#the-access-token) and [Conformance](#conformance)).
- **Question 28 — a delegate's rules and directives.** A delegate's export carries each standing rule and directive whose `categories` are absent or all within the delegation's scope, as [`hcp/rules/list`](../../rules/list/0.1/spec.md) and [`hcp/directives/list`](../../directives/list/0.1/spec.md) return them; one naming any category outside the scope is left out whole.
- **Question 32 — a delegate's sub-delegations.** A delegate's export carries the delegations naming them and the sub-delegations they appointed, as [`hcp/delegations/list`](../../delegations/list/0.1/spec.md) returns them.
- **Question 33 — export size.** The export travels in one document bounded at the framework's 16 MiB ceiling; a larger export is refused with `hcp:exportTooLarge`, and is taken over HCP's HTTP binding or with narrower options (see [Export size](#export-size)).
- **Question 37 — governance over a carriage other than the HTTP Binding.** HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1). This specification takes it that another carriage, here a Trust Task, may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and applies every rule HCP places on the operation unchanged (see [Conformance](#conformance)).
- **Question 40 — a delegate's export and import beyond entries.** A delegate's export carries only the superseded versions, of the entries it returns, whose own categories are all within the delegation's scope (see [Authorization](#authorization)).
- **Question 42 — presence signals the carriage adds.** The export's `options` is required, with `includeVersions` and `includeAudit` each `true` or `false`, so that the export says whether `versions` and `audit` were asked for; and every `AuditEvent` in `audit` carries `entryId`, `relatedEntryId`, `purpose`, `purposeDescription` and `grantId`, each with its value or null. HCP's own shapes leave these members optional (HCP 5.4.1, 8.2). [`hcp/import`](../../import/0.1/spec.md) still accepts an export without `options`, and audit events without those five members.

## What HCP requires that a document cannot express

- Which entries are held, their statuses, and whether an entry was hard-deleted: the export is assembled from provider-held state (HCP 5.4.1, 5.4.2).
- The scoping of a delegate's export: its entries and versions to the delegation's scope (HCP 17.4), its audit log to what HCP 17.7 lets a delegate read, and its delegations, rules and directives to what the list operations return a delegate (HCP 9.4.13).
- The `context.exported` audit event and its durability (HCP 8.1, 8.4).
- Carriage by the HTTP Binding: HCP carries governance operations only over that binding (HCP 1.6, 5.4, 9.1), and a Trust Task is not it. This specification carries the operation for a Management Client acting for the Authorizing Principal (HCP 6.1.9) instead, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

## Security & Privacy

### Data carried

The request carries the export options and the Management Client's access token, a bearer secret that **MUST** travel only to the provider. The response carries the person's whole context: every entry the provider holds, every status included, which may hold High-sensitivity categories such as `health_context` and credentials as evidence; the person's standing rules, directives and delegations, which reveal whom they distrust and who acts for them; and, where asked, every superseded version and the audit log, which records every client that read their context. This is the most a provider ever releases in one document. A producer **MUST NOT** put context or the token in `ext` or in any `message`.

Because the response proof is REQUIRED ([SPEC §7.3 item 8](/SPEC.md#73-specification-requirements)), the export reaches the Management Client as a signed assertion that any third party can verify, which makes a leaked copy more credible. A Management Client **MUST** hold the export only for the person, and **MUST NOT** pass it to anyone but a provider the person chose to import it into, or the person themselves.

### The access token

`accessToken` is a bearer secret: whoever presents it exercises the authority it evidences. It **MUST** travel only to the provider. A recipient **MUST NOT** log `accessToken` or retain it beyond the exchange.

**Token binding, a carriage rule of this family.** Where the request carries a `proof`, the provider **MUST** refuse with `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies, so that a token lifted from a stored or forwarded document is of no use from another VID. HCP validates the token alone and does not bind it to a VID (HCP 9.3): this rule is the `hcp/*` family's, not HCP's, and rests on the `issuer` VID identifying the HCP client, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)). It is a check HCP does not make (see [Conformance](#conformance)).

**Retained copies.** The provider verifies the `proof`, where the request carries one, over the document as received, at receipt, before it removes anything ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). A provider that retains the request document, as a duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) or as evidence of the request, **MUST** remove `accessToken` from the copy it keeps and record in its place `accessTokenDigest`: the SHA-256 multihash of the token's UTF-8 bytes, multibase-encoded as base64url without padding (prefix `u`), the framework's `DigestMultibase` form ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)). The kept copy's `proof` no longer verifies over it, so the provider records with the copy that the proof was verified at receipt: when, under which verification method, and with what result. It relies on that record, and **MUST NOT** verify the proof again over the kept copy. A record that keeps only a digest of the document, such as its task digest ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)), keeps no token either.

### Correlation

The provider learns which Management Client exported the person's context, and when; that is the record the person needs. The export carries the Context Subject's identifier and, in the audit log and delegations, the identifiers of clients and delegates: it is a complete map of the person's relationships, and **MUST** be protected as such. `threadId` **SHOULD** be fresh per export. Response size reveals how much context the person holds.

### Retention

The provider keeps the `context.exported` audit event (HCP 8.1), readable by the Context Subject (HCP 8.3). The export is the person's own copy; how long the Management Client keeps it is the person's choice, and a Management Client **SHOULD** delete it once it has been imported where the person asked, unless the person asked to keep it as an archive.

### Consent/purpose

The export is the person's exercise of their own access to their data; its purpose is theirs. A Management Client **SHOULD** make sure the person, not an automated process, asked for it, and **SHOULD** tell them that deleted entries travel with it unless they exclude them (HCP 5.4.1). Whether it asks for confirmation is its own policy ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).

The **acceptance window** for `issuedAt` is consumer policy; a provider **SHOULD** apply a short one, minutes rather than hours.
