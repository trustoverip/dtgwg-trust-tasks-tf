---
slug: vtc/schemas/accepts/register
version: "0.1"
title: "VTC Schemas — Accepts — Register"
summary: An administrator registers a named Accepts criterion — the DCQL query a ceremony runs over presented credentials, and any peer identity vetting it requires — which the join manifest then publishes to applicants.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The write changes which credentials the community will mint or accept as evidence, under the community's name, and is authorized only by the signer's administrator standing — so the signer must be attributable on every transport and the change attributable afterwards in the audit trail.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed registration would put back an entry an administrator has since replaced or deleted, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Creates or replaces one criterion. The criterion is what the community's ceremonies decide against and what vtc/join-requests/manifest publishes, so a registration changes what applicants are asked to present from then on. Recoverable by registering again or deleting.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries a DCQL query, vetting requirements and a description — community policy, not personal data; the response echoes the stored criterion.
retention:
  class: durable
  rationale: >-
    The criterion is community policy that ceremonies and the join manifest depend on, kept until replaced or deleted, and the change is audited.
errorCodes:
  - code: vtc/schemas/accepts/register:invalidQuery
    meaning: "`query` is not a valid DCQL query."
    retryable: false
  - code: vtc/schemas/accepts/register:unregisteredType
    meaning: "The query references a credential type (`meta.vct_values`) that is not in the schema registry. Register it with vtc/schemas/register first. `details.typeUri` names the first."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      maxProperties: 1
      required: [typeUri]
      properties:
        typeUri:
          type: string
          maxLength: 512
  - code: vtc/schemas/accepts/register:unregisteredStatementType
    meaning: "`vetting.statementType` is not a registered endorsement type. Register it with vtc/endorsement-types/register first."
    retryable: false
  - code: vtc/schemas/accepts/register:invalidVetting
    meaning: "`vetting` breaks a rule its schema cannot state — a method floor naming a method not in `acceptedMethods`, floors summing past `minStatements`, or similar — so no applicant could satisfy it."
    retryable: false
  - code: vtc/schemas/accepts/register:notPublishable
    meaning: "The criterion is well-formed but the join manifest could not publish it — for example its query is not an object the manifest's presentation-definition member accepts. Nothing is stored that applicants could not be shown."
    retryable: false
related:
  - vtc/schemas/accepts/list
  - vtc/schemas/accepts/show
  - vtc/schemas/accepts/delete
  - vtc/schemas/register
  - vtc/join-requests/manifest
  - vtc/join-requests/query
---

## Abstract

An **Accepts criterion** is a community's named rule for what evidence a holder must present: a DCQL query over credential types in the schema registry ([`vtc/schemas/register`](../../../register/0.1/spec.md)), and optionally the peer identity vetting the criterion requires — how many statements, by which methods, from which eligible vetters. Ceremonies decide against it; [`vtc/join-requests/manifest/0.2`](../../../../join-requests/manifest/0.2/spec.md) publishes it to applicants as a `Criterion` with a `requirementsDigest`; [`vtc/join-requests/query`](../../../../join-requests/query/0.1/spec.md) sends its query to a holder.

The **VTC Schemas — Accepts — Register** Trust Task adds a criterion, or replaces the one registered under the same id. Replacing a criterion that carries `vetting` changes its `requirementsDigest`, which is how applicants who started under the earlier version are told.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`.
2. **MUST** trim surrounding whitespace from `id`, and **MUST** refuse, with `malformedRequest`, one that is empty after trimming.
3. **MUST** refuse, in this order: `invalidQuery` for a query that does not parse as DCQL; `unregisteredType` for a query referencing a type not in the schema registry; `unregisteredStatementType` for `vetting.statementType` not registered as an endorsement type; `invalidVetting` for vetting requirements no applicant could satisfy; `notPublishable` for a criterion the join manifest could not publish. Nothing is stored unless every check passes.
4. **MUST** replace any existing criterion with the same `id`, recording the caller and the execution time.
5. **MUST** audit the registration, naming the criterion id and the administrator, and answer with the criterion it stored.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — read at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

## Definitions

- **`id`** — the criterion's name, chosen by the administrator; the key.
- **`query`** — the DCQL query, carried verbatim.
- **`vetting`** — a `VettingRequirements` object, the same shape vtc/join-requests/manifest/0.2 publishes. Every number in it is the community's own policy.
- Shapes are in [`vtc/_shared/0.1/schema-registry.schema.json`](../../../../_shared/0.1/schema-registry.schema.json).

## Request

A community administrator (`issuer`) sends the criterion to the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A membership criterion requiring two vetting statements

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/schemas/accepts/register/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "id": "membership",
    "query": {
      "credentials": [
        {
          "id": "membership",
          "format": "vc+sd-jwt",
          "meta": {
            "vct_values": [
              "https://openvtc.org/credentials/MembershipCredential"
            ]
          }
        }
      ]
    },
    "description": "Show a membership credential from a partner community, and be vetted by two members.",
    "vetting": {
      "version": "0.1",
      "statementType": "https://riverside.example/endorsements/IdentityVetting",
      "minStatements": 2,
      "acceptedMethods": [
        "inPerson",
        "video"
      ],
      "eligibleVetters": {
        "role": "vetter"
      }
    }
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`: the stored criterion. A refusal is a `trust-task-error`.

### Registered

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/schemas/accepts/register/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "criterion": {
      "id": "membership",
      "query": {
        "credentials": [
          {
            "id": "membership",
            "format": "vc+sd-jwt",
            "meta": {
              "vct_values": [
                "https://openvtc.org/credentials/MembershipCredential"
              ]
            }
          }
        ]
      },
      "description": "Show a membership credential from a partner community, and be vetted by two members.",
      "vetting": {
        "version": "0.1",
        "statementType": "https://riverside.example/endorsements/IdentityVetting",
        "minStatements": 2,
        "acceptedMethods": [
          "inPerson",
          "video"
        ],
        "eligibleVetters": {
          "role": "vetter"
        }
      },
      "createdAt": "2026-09-28T10:00:01Z",
      "createdByDid": "did:example:administrator"
    }
  }
}
```

## Security & Privacy

### Data carried

Registry configuration an administrator wrote. The description member is free text, bounded at 1024 characters, written by the registering administrator and read by other administrators (and, for an Accepts criterion, published to applicants and used as the `purpose` a join query shows the holder); it is untrusted, attributed to its author wherever rendered, and **MUST NOT** carry personal data.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The audit trail ties each change to the administrator who made it.

### Retention

The entry lives until it is replaced or deleted. The audit record names the entry and the administrator, not the entry's content.

### Consent/purpose

The purpose is to set which credentials the community mints, and which it accepts as evidence in its ceremonies. What is registered here changes what applicants are asked to present, so an administrator **SHOULD** register the least evidence the community's governance requires.
