---
slug: hcp/import
version: "0.1"
title: "HCP — Import"
summary: A Management Client, at the person's direction, loads an HCP export into the person's Human Context Protocol instance and receives one outcome per entry; carries the HCP 2.0-draft-3 Import operation.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Import
category: data-exchange
keywords: [hcp, human-context-protocol, import, portability, governance]
parties:
  - role: Management Client (HCP), acting for the Context Subject or a delegate
    requirement: REQUIRED
    member: issuer
  - role: HCP Provider
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    Import writes the person's context into their instance and exercises their own authority, so the request is
    signed: the provider can show which Management Client loaded which export, and when, independently of the
    transport. The response proof is RECOMMENDED so the Management Client can show the person what the provider
    reported for each entry.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task changes the person's context and acts for them, so it is consequential; a re-delivered import must be
    placeable in the provider's duplicate-execution window, so that it is not carried out, or audited, twice.
sideEffects:
  level: mutating
  rationale: >-
    Entries are added to the Context Subject's instance, standing rules in the export take effect, and categories
    are replaced where `categoryMap` says so. Nothing held is overwritten or removed (HCP 5.4.3). A dry run changes
    nothing.
exposure:
  discloses: metadata
  actsAsSubject: true
  ingests: secret
  rationale: >-
    The Management Client exercises the Context Subject's (or a delegate's) authority to load their context. The
    request carries the person's exported context, which may hold High-sensitivity categories, and the Management
    Client's access token, a bearer secret. The response discloses only entry identifiers and an outcome for each.
retention:
  class: durable
  rationale: >-
    The imported entries are the person's context from then on, and the provider appends a `context.imported` audit
    event listing each entry and its outcome, and a `context.recategorized` event for each replacement (HCP 5.4.3,
    8.1), which are the person's record of what arrived.
    The access token is not retained (see Security & Privacy, The access token).
maxDocumentBytes:
  request: 16777216
  rationale: >-
    The request carries a whole HCP export: every entry, which may carry verifiable credentials as evidence, the
    standing rules, directives and delegations, and possibly superseded versions and the audit log. 16 MiB is the
    framework's ceiling for a declared bound; it is a framework bound, not an HCP requirement, since HCP sets no size
    limit. A larger export is loaded over HCP's own HTTP binding (`POST /v2/import`, HCP 9.4.11).
errorCodes:
  - code: hcp:invalidToken
    meaning: >-
      HCP `invalid-token` (HCP 5.5): "Missing, malformed, or expired authorization evidence", here the Management
      Client's access token.
    retryable: false
related:
  - hcp/export
  - hcp/entries/set-categories
  - hcp/rules/list
---

## Abstract

The **HCP — Import** Trust Task carries the Import operation of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 5.4.3 and 5.4; `POST /v2/import` in HCP 9.4.11). A *Management Client*, at the *Context Subject*'s direction, gives an *HCP Provider* an HCP export, made with [`hcp/export`](../../export/0.1/spec.md) or HCP's own HTTP binding, to load into the Context Subject's instance. The provider answers with one outcome for each entry: imported, converted from HCP v1, skipped, or rejected with the reason. A dry run reports the outcomes and changes nothing. Import is not federation: one person moves their own context (HCP 5.4.3).

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider that serves this task **MUST** also meet every requirement HCP 2.0-draft-3 places on import; the ones that bear on the document are restated below, unchanged. Conformance to HCP is HCP's ([SPEC §9.4](/SPEC.md#94-carried-protocols)): a provider **MUST NOT** list a conformance identifier for this carriage in its HCP `conformsTo`, and advertises the task through `trust-task-discovery`, *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)). HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1); this specification takes it that a carriage other than that binding may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and carries this one as a Trust Task on that reading, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

## Authorization

The authority is **the Context Subject's own authority over their context**, which a Management Client exercises at their direction (HCP 5.4.3, 6.1.9). It is evidenced by the Management Client's access token in `accessToken`, which the provider validates exactly as over HCP's HTTP binding. Import is a governance operation: any caller other than a Management Client acting for the Context Subject, or for a delegate, receives not authorized (HCP 6.1.9), carried as `permissionDenied`. A grant held by a Context Client never authorizes it, whatever its scopes (HCP 6.1.9). HCP names only its HTTP Binding as the carriage for a governance operation (HCP 5.4, 9.1); this specification accepts the operation as a Trust Task from a Management Client acting for the Authorizing Principal, and from no other caller (HCP 6.1.9), *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

Under the Delegated Authorization Profile, a governance operation performed by a delegate reaches only entries in categories within the delegation's scope (HCP 17.4). An entry of the export outside that scope is not imported; it is reported as a `rejected` outcome, *pending HCP confirmation* ([question 34](#pending-hcp-confirmation)).

The REQUIRED `proof` establishes which Management Client VID sent the document and that it is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). It is not the authorization: the token is. Nor does it vouch for the export it carries: the export's entries keep the provenance the exporting provider recorded, and the importing provider verifies their credentials again (see [Rules carried from HCP](#rules-carried-from-hcp)). A token issued to a client other than the one the `issuer` VID identifies is refused with `permissionDenied` (see [The access token](#the-access-token)).

## Definitions

- **Management Client**, **Context Client**, **HCP Provider**, **Context Subject**, **Authorizing Principal**, **delegate**, **entry**, **category**, **standing rule**, **delegation** and **export** have their HCP 2.0-draft-3 meanings.
- The export's shape is [`Export`](../../_shared/0.1/export.schema.json) in `hcp/_shared/0.1`. It is open, as HCP's own shape is, and its `entries` are left open, because an import accepts entries in any schema the provider accepts, HCP v1 forms included (HCP 5.4.3, 4.3); each entry is validated against the schema it names.

### HCP field mapping

| HCP field | Member | Notes |
|---|---|---|
| request `export` | `export` | Unchanged. |
| request `dryRun` | `dryRun` | Unchanged. |
| request `categoryMap` | `categoryMap` | Unchanged. At most 256 replacements, a framework bound HCP does not set. |
| `Authorization: Bearer` header (HCP 9.3) | `accessToken` | Moved from the transport into the payload, unchanged. |
| `HCP-Version` header | the Type URI and `carries.version` | Version 0.1 of this task carries HCP 2.0-draft-3 only. |
| result `outcomes` | `outcomes` | Unchanged: each with `id`, `outcome` and, for a rejection, `reason`. |
| result `dryRun` | `dryRun` | Unchanged. |

### Parties

| HCP role | Trust Task member |
|---|---|
| Management Client | `issuer` of the request, `recipient` of the response |
| HCP Provider | `recipient` of the request, `issuer` of the response |

As for [`hcp/entries/query`](../../entries/query/0.1/spec.md#parties), a provider **MUST** make its VID verifiably bound to its HCP identity (its discovery document, HCP 19.1), and a Management Client **MUST** check that binding before it sends a token to a VID.

## Request

The Management Client sends the export to the provider. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

- `export` (REQUIRED) is the HCP export to load, unchanged from the exporting provider.
- `dryRun` (OPTIONAL), where true, asks for the outcome for each entry and changes nothing. A Management Client **SHOULD** make a dry run first and show the person its outcomes, in particular every `rejected` one and every category the export uses that the provider does not hold, before it imports.
- `categoryMap` (OPTIONAL) names, for each category the export uses that the provider does not hold, the category the provider holds that replaces it. Its keys are the categories being replaced, its values their replacements. Absent means no category is replaced, and an entry with a category the provider does not hold is rejected (HCP 5.4.3).
- `accessToken` (REQUIRED) is the Management Client's token.

### A dry run, replacing a custom category

```json
{
  "id": "urn:uuid:fb59c1b6-a0ea-456b-b149-20aa7360cb6e",
  "type": "https://trusttasks.org/spec/hcp/import/0.1#request",
  "issuer": "did:web:wallet.person.example",
  "recipient": "did:web:new-context.provider.example",
  "issuedAt": "2026-11-25T10:00:00Z",
  "threadId": "urn:uuid:71b8857d-0a2a-4ceb-a93d-00f7829620c9",
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
          "id": "urn:uuid:13462b18-de39-4fcc-b94d-56b776184381",
          "type": ["ContextEntry"],
          "subject": { "id": "did:web:person.example" },
          "content": {
            "profile": "https://hcp.example/spec/profiles/general-context/2.0",
            "value": "Runs three mornings a week; prefers early slots.",
            "language": "en"
          },
          "assertion": { "declarationType": "person-declared" },
          "classification": { "categories": ["https://categories.example/fitness"] },
          "record": {
            "status": "active",
            "version": 1,
            "provenance": {
              "assertedBy": { "id": "did:web:person.example", "assurance": "authenticated" },
              "assertedAt": "2026-08-03T07:15:00Z",
              "source": "person-input",
              "submittedByClient": {
                "id": "wallet-app",
                "name": "Person Wallet",
                "operator": { "id": "wallet.person.example", "name": "Wallet Co" }
              },
              "authorizedUnder": "urn:uuid:7e702905-d484-487e-9c30-8f7fa67b6ac2",
              "recordedByProvider": "https://context.provider.example",
              "recordedAt": "2026-08-03T07:15:01Z"
            }
          }
        },
        {
          "@context": ["https://hcp.example/contexts/v2"],
          "schema": "https://hcp.example/spec/entry/2.0",
          "id": "urn:uuid:fa146974-0465-4023-9ec4-1c36ee8341f8",
          "type": ["ContextEntry"],
          "subject": { "id": "did:web:person.example" },
          "content": {
            "profile": "https://hcp.example/spec/profiles/general-context/2.0",
            "value": "Prefers window seats.",
            "language": "en"
          },
          "assertion": { "declarationType": "person-declared" },
          "classification": { "categories": ["general_preferences"] },
          "record": {
            "status": "active",
            "version": 3,
            "provenance": {
              "assertedBy": { "id": "did:web:person.example", "assurance": "authenticated" },
              "assertedAt": "2026-06-11T12:00:00Z",
              "source": "person-input",
              "submittedByClient": {
                "id": "wallet-app",
                "name": "Person Wallet",
                "operator": { "id": "wallet.person.example", "name": "Wallet Co" }
              },
              "authorizedUnder": "urn:uuid:7e702905-d484-487e-9c30-8f7fa67b6ac2",
              "recordedByProvider": "https://context.provider.example",
              "recordedAt": "2026-06-11T12:00:01Z"
            }
          }
        }
      ],
      "rules": [
        {
          "ruleId": "urn:uuid:712a4602-9af5-401f-b0bf-8f631dd82a5b",
          "effect": "deny",
          "operators": ["ads.example"],
          "categories": ["health_context"],
          "setBy": "context-subject",
          "createdAt": "2026-09-14T18:20:00Z"
        }
      ]
    },
    "dryRun": true,
    "categoryMap": { "https://categories.example/fitness": "general_preferences" },
    "accessToken": "hcp_mt_3c7e1a9f5b2d8e40"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-25T10:00:00Z",
    "verificationMethod": "did:web:wallet.person.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "zwkTQijD5JX2muLDZX2bjowk8DW6RLsrp6M5y7tgzscSUBgpcaivaJw4VVGef8cjurwPQhEJDZW3KJdQN8YceRtN"
  }
}
```

## Response

The provider answers with one outcome for each entry. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json).

- `outcomes` holds one item for each entry in the export: its `id`, its `outcome`, and, where the outcome is `rejected`, the `reason`. That a `rejected` outcome carries `reason` is not checked by the schema.
- `dryRun` is true where the request was a dry run, and nothing was changed.

Failures of the import as a whole use `trust-task-error`, not a `#response` document (see [Errors](#errors)). An entry that cannot be imported is not such a failure: it is a `rejected` outcome (see [Outcomes](#outcomes)).

### The dry run's outcomes

The new provider holds the second entry's `id` already, with different content, so it would reject it; the first entry would be imported with its category replaced.

```json
{
  "id": "urn:uuid:146208b0-3dc7-4845-9e9f-8f86b5640ff8",
  "type": "https://trusttasks.org/spec/hcp/import/0.1#response",
  "issuer": "did:web:new-context.provider.example",
  "recipient": "did:web:wallet.person.example",
  "issuedAt": "2026-11-25T10:00:01Z",
  "threadId": "urn:uuid:71b8857d-0a2a-4ceb-a93d-00f7829620c9",
  "payload": {
    "outcomes": [
      { "id": "urn:uuid:13462b18-de39-4fcc-b94d-56b776184381", "outcome": "imported" },
      {
        "id": "urn:uuid:fa146974-0465-4023-9ec4-1c36ee8341f8",
        "outcome": "rejected",
        "reason": "id held with different content"
      }
    ],
    "dryRun": true
  }
}
```

## Outcomes

Each entry in the export has exactly one outcome (HCP 5.4.3):

- **`imported`**: the entry is loaded as it was exported.
- **`converted`**: a v1 entry, converted under HCP 4.3 and loaded.
- **`skipped`**: a deleted entry the provider chose not to carry, or an exact duplicate of an entry it holds.
- **`rejected`**, with the reason: the entry is not loaded. HCP names these cases: an `id` already held with different content, and a category the provider does not hold that `categoryMap` does not replace (HCP 5.4.3).

HCP also says that an entry whose content credential fails verification "is rejected as credential invalid" (HCP 5.4.3, 14.6). Because the result has one outcome per entry, this specification reports such an entry as a `rejected` outcome, and the rest of the import proceeds; no `trust-task-error` is sent for it. Its `reason` names HCP's `credential-invalid` and which check failed (its proof, its securing mechanism, its verification method, or its status), and **MUST NOT** reveal the credential's position in a status list or any other status-list internals (HCP 5.5, 14.6). An entry in a schema the provider does not accept (HCP 4.3), and, under a delegation, an entry outside the delegation's scope (HCP 17.4), are reported the same way. All three readings are *pending HCP confirmation* ([question 34](#pending-hcp-confirmation)).

A `reason` is shown to the person. It **MUST NOT** disclose anything the condition forbids (HCP 5.5); in particular, an entry rejected because its `id` is held says only that, and never the content held under it.

## Errors

HCP defines nine Core errors (HCP 5.5). They map to Trust Task errors as follows. The provider uses the code in the right-hand column. It never sends HCP's Core error object or the HTTP binding's problem details (HCP 9.7).

| HCP error (5.5) | Code | Trust Task error |
|---|---|---|
| invalid request | `malformed-request`: the request fails schema validation, or the export is not an HCP export it can read (HCP 5.4.3) | `malformedRequest`. A request document larger than this task's `maxDocumentBytes` request bound is also refused with `malformedRequest`, as the framework requires; the Management Client loads such an export over HCP's HTTP binding (HCP 9.4.11). |
| unauthenticated | `invalid-token` | `hcp:invalidToken`, `retryable: false` |
| not authorized | `scope-denied`: the caller is not a Management Client acting for the Context Subject or a delegate (HCP 6.1.9) | `permissionDenied`, with one fixed `message` whatever the reason (HCP 5.5, [SPEC §8.2.1](/SPEC.md#821-what-a-message-may-not-say)) |
| conflict | `idempotency-conflict` | Does not arise: HCP defines it for a retried request with an idempotency key (HCP 9.6), and this operation carries none. A re-delivered document is handled under [Repeated requests](#repeated-requests). |
| version conflict | `version-conflict` | Does not arise: an import names no current version (HCP 5.4.3). An entry whose `id` is held with different content is a `rejected` outcome, never an error (HCP 5.4.3). |
| credential invalid | `credential-invalid` | Not sent as an error: an entry whose content credential fails verification is a `rejected` outcome whose `reason` names `credential-invalid` and the check that failed, *pending HCP confirmation* ([question 34](#pending-hcp-confirmation); see [Outcomes](#outcomes)). |
| subject unavailable | `subject-unavailable` | `unavailable`. Under end-to-end encrypted storage the import needs the Context Subject's trust domain (HCP 4.8, 14.6); the Management Client may retry later (HCP 5.5). |
| rate limited | `rate-limited` | `unavailable`, with `retryAfter` |
| unsupported version | `unsupported-version` | `unsupportedVersion`. The provider never serves a different HCP version silently (HCP 5.5). |

A Management Client that does not know `hcp:invalidToken` reads it as `taskFailed` ([SPEC §8.5](/SPEC.md#85-extension-by-individual-trust-task-specifications)) and still honours `retryable: false`.

## Rules carried from HCP

- **v1 entries.** A provider that imports v1 entries converts them as HCP 4.3 requires, and accepts only the entry schemas it declares in `accepts_entry_schemas` (HCP 4.3, 19). Conversion is not a revision: it does not increment `record.version`, adds no confirmation record, records every v1 asserter as `unverified` unless a content credential is validated at conversion, and records the conversion in `record.conversion`. Where it cannot establish the submitting client and grant, it records them as the import that carried the entry (HCP 4.3).
- **Nothing is overwritten.** An entry whose `id` is already held, with different content, **MUST** be rejected; it is never overwritten (HCP 5.4.3).
- **Deleted stays deleted.** The provider **MUST NOT** reinstate as active any entry carrying `record.status: deleted`. It either preserves the status or omits the entry, which is then `skipped` (HCP 5.4.3).
- **Provenance is preserved.** The provider **MUST** preserve each entry's `assertion.declarationType` and `record.provenance`, including its confirmation records, as exported. Assurance values remain the exporting provider's record, which `recordedByProvider` shows; nothing is upgraded (HCP 5.4.3, 3.11 invariant 15).
- **Credentials are verified again.** An entry whose content is a credential is verified again under the VC Evidence Profile, with its status (HCP 14.6, 14.7); one that fails is rejected (see [Outcomes](#outcomes)). A supporting credential is verified again and its `verification` recorded anew; its failure does not reject the entry (HCP 14.8).
- **No silent re-categorization.** The provider **MUST NOT** silently re-categorize an imported entry, because `classification.categories` governs which grants reach it (HCP 4.2.1). An entry with a category the provider does not hold is rejected, unless `categoryMap` replaces each such category. Each replacement is recorded as a `context.recategorized` event naming the prior and new categories, exactly as for any other change of categories (HCP 5.4.3, 4.2.1).
- **Standing rules and delegations.** Standing rules in the export are applied as the Context Subject's own (HCP 6.3). Delegations in the export are not established; the Management Client shows them to the Context Subject as a record (HCP 5.4.3, 17.2).
- **Audit.** The provider appends one `context.imported` event for each import other than a dry run, listing each entry's `id` and outcome (HCP 5.4.3, 8.1, 8.2), durable before this task's response is sent (HCP 8.4). A dry run appends none and changes nothing.

## Repeated requests

A re-delivered document with the same `id` and content is absorbed by [SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements): it is not executed twice, appends no second `context.imported` event, and is answered with the original response. A new request carrying the same export, with a new `id`, is a new import: entries already loaded by the first are then exact duplicates, `skipped`, and HCP's rules ensure that nothing held is overwritten (HCP 5.4.3).

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 1 — conformance claims.** HCP defines no conformance identifier for Trust Task carriage, so a provider **MUST NOT** claim this carriage in its HCP `conformsTo`; it advertises the task with [`trust-task-discovery`](../../../trust-task-discovery/0.4/spec.md) instead. The binding identifier for the carriage is the same question as [question 37](#pending-hcp-confirmation): whether a carriage other than the HTTP Binding may carry governance operations at all.
- **Question 22 — the client's identity.** The `issuer` VID, authenticated by the request's `proof`, identifies the HCP client the token was issued to; where the request carries a proof, a token issued to any other client is refused with `permissionDenied`, a strengthening HCP does not itself require (see [The access token](#the-access-token)).
- **Question 33 — export size.** The export travels in one request document, bounded at the framework's 16 MiB ceiling; a larger export is loaded over HCP's HTTP binding.
- **Question 34 — entries that cannot be imported.** An entry whose content credential fails verification, an entry in a schema the provider does not accept, and, under a delegation, an entry outside the delegation's scope, are each a `rejected` outcome, and the rest of the import proceeds (see [Outcomes](#outcomes)).
- **Question 37 — governance over a carriage other than the HTTP Binding.** HCP carries governance operations only over its HTTP Binding (HCP 1.6, 5.4, 9.1, 10.1). This specification takes it that another carriage, here a Trust Task, may carry them where the caller is a Management Client acting for the Authorizing Principal (HCP 6.1.9), and applies every rule HCP places on the operation unchanged (see [Conformance](#conformance)).

## What HCP requires that a document cannot express

- Whether an `id` is held, and with what content, and which categories the provider holds: provider-held state (HCP 5.4.3).
- v1 conversion and credential re-verification (HCP 4.3, 14.6–14.8).
- The scope check on a delegate's import (HCP 17.4).
- The `context.imported` and `context.recategorized` audit events and their durability (HCP 8.1, 8.4).
- Carriage by the HTTP Binding: HCP carries governance operations only over that binding (HCP 1.6, 5.4, 9.1), and a Trust Task is not it. This specification carries the operation for a Management Client acting for the Authorizing Principal (HCP 6.1.9) instead, *pending HCP confirmation* ([question 37](#pending-hcp-confirmation)).

## Security & Privacy

### Data carried

The request carries the person's whole exported context, which may hold High-sensitivity categories such as `health_context`, credentials as evidence, their standing rules, directives and delegations, and possibly their audit log; and the Management Client's access token, a bearer secret that **MUST** travel only to the provider. The response carries only entry identifiers, outcomes and reasons. A producer **MUST NOT** put context or the token in `ext` or in any `message`, and a provider **MUST NOT** put entry content in a `reason`.

### The access token

`accessToken` is a bearer secret: whoever presents it exercises the authority it evidences. It **MUST** travel only to the provider. A recipient **MUST NOT** log `accessToken` or retain it beyond the exchange: a duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) or an evidentiary record of the request keeps the document with `accessToken` removed, or a digest of the document, such as its task digest ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)), and never the token. Where the request carries a `proof`, the provider **MUST** refuse with `permissionDenied` a token issued to a client other than the one the `issuer` VID identifies, so that a token lifted from a stored or forwarded document is of no use from another VID. HCP does not itself bind a token to a VID: this is a strengthening of the carriage, resting on the `issuer` VID identifying the HCP client, *pending HCP confirmation* ([question 22](#pending-hcp-confirmation)).

### Correlation

The provider learns everything the export holds, including the identifiers of clients and delegates in the audit log and delegations; importing is the person's choice to give it that. Entry identifiers travel unchanged from the exporting provider, so the two providers can join their records of the same entry; that is what a move between providers is. `threadId` **SHOULD** be fresh per import.

### Retention

The imported entries are kept as the person's context, under the same rules as any other entry. The provider keeps the `context.imported` and `context.recategorized` audit events (HCP 8.1), readable by the Context Subject (HCP 8.3). Delegations and, where present, the audit log of the exporting provider are a record shown to the person, not state the provider acts on (HCP 5.4.3). The signed request is evidence of who loaded which export and when, and a provider **SHOULD** keep it with the `context, with `accessToken` removed (see [The access token](#the-access-token)).imported` event. A Management Client **SHOULD** delete its copy of the export once the person no longer needs it.

### Consent/purpose

An import is made at the Context Subject's direction (HCP 5.4.3): a Management Client **MUST NOT** import without the person asking for it. It **SHOULD** show the person a dry run's outcomes, and every category replacement it proposes in `categoryMap`, before importing, since a replacement changes what the person's existing and future grants reach (HCP 4.2.1, 5.4.3). Whether it asks for further confirmation is its own policy ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).
