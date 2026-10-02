---
slug: vtc/schemas/accepts/register
version: "0.2"
title: "VTC Schemas — Accepts — Register"
summary: An administrator registers an Accepts criterion — one way into the community, stating what it requires (credentials, vetting, an invitation, or nothing) and whether meeting it admits automatically or is reviewed.
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
    Creates or replaces one criterion. The criterion is one of the ways into the community, what its join decisions are made against and what vtc/join-requests/manifest publishes, so a registration changes who can join and how from then on. Recoverable by registering again or deleting.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries the criterion — its admission mode and any DCQL query, vetting requirements, invitation requirement and description — community policy, not personal data; the response echoes the stored criterion.
retention:
  class: durable
  rationale: >-
    The criterion is community policy that ceremonies and the join manifest depend on, kept until replaced or deleted, and the change is audited.
errorCodes:
  - code: vtc/schemas/accepts/register:invalidQuery
    meaning: "`query` is present and is not a valid DCQL query."
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
    meaning: "`vetting.statementType` is not a predicate the community has registered. Register it with vtc/endorsement-types/register first."
    retryable: false
  - code: vtc/schemas/accepts/register:invalidVetting
    meaning: "`vetting` breaks a rule its schema cannot state — a method floor naming a method not in `acceptedMethods`, floors summing past `minStatements`, or similar — so no applicant could satisfy it."
    retryable: false
  - code: vtc/schemas/accepts/register:notPublishable
    meaning: "The criterion is well-formed but the join manifest could not publish it — for example its query is not an object the manifest's presentation-definition member accepts. Nothing is stored that applicants could not be shown."
    retryable: false
  - code: vtc/schemas/accepts/register:unsupportedRequirement
    meaning: "The criterion states a requirement this community cannot evaluate — for example `credentialIssuers: recognised` at a community that recognises no other community, or `invitationRequired` at one that does not issue invitations. A criterion is published only if the community can decide a submission against every requirement it states."
    retryable: false
related:
  - vtc/schemas/accepts/list
  - vtc/schemas/accepts/show
  - vtc/schemas/accepts/delete
  - vtc/schemas/register
  - vtc/join-requests/manifest
  - vtc/join-requests/query
  - vtc/join-requests/submit
  - vtc/recognition/check
  - vtc/invitations/issue
---

## Abstract

An **Accepts criterion** is one way into a community. It states what an applicant must present, which may be nothing, and how a submission that presents it is decided. The community's criteria together are its join policy, and they are alternatives: an applicant meets one of them, not all. [`vtc/join-requests/manifest/0.3`](../../../../join-requests/manifest/0.3/spec.md) publishes them to applicants; [`vtc/join-requests/submit/0.3`](../../../../join-requests/submit/0.3/spec.md) names the one a submission is made under; [`vtc/join-requests/query`](../../../../join-requests/query/0.1/spec.md) sends a criterion's credential query to a holder.

A criterion's requirements are any of:

- **credentials** (`query`, a DCQL query) from issuers the criterion names (`credentialIssuers`);
- **peer identity vetting** (`vetting`): statements from eligible vetters, counted as the community requires;
- **an invitation** (`invitationRequired`) the community issued to the applicant.

Every requirement a criterion states must be met. A requirement it does not state is not one, so a criterion stating none is met by every submission.

**`admission`** says what meeting the criterion does. `automatic` admits the applicant. `review` refers the submission to an administrator, who decides.

The **VTC Schemas — Accepts — Register** Trust Task adds a criterion, or replaces the one registered under the same id.

This specification takes no position on which criteria a community should have. Open admission, admission only after review, admission only by invitation, and any combination of credentials, vetting and invitations are equally expressible, and which a community publishes is its administrators' decision.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

### Changes from 0.1

In 0.1 a criterion was a DCQL query, with optional vetting, and nothing else. It could not say:

- that nothing is required — a DCQL query must ask for at least one credential, so an open community could only be had by deleting every criterion;
- that a submission meeting the criterion is reviewed rather than admitted;
- that an invitation is required, except inside `vetting`;
- whose credentials meet the query.

0.2 adds `admission` (REQUIRED), makes `query` OPTIONAL, and adds `credentialIssuers` (REQUIRED with `query`) and `invitationRequired`. Shapes move to [`vtc/_shared/0.2/schema-registry.schema.json`](../../../../_shared/0.2/schema-registry.schema.json). A 0.1 criterion has no `admission`, so it is not a 0.2 criterion; the administrator who registered it says how it is decided by registering it again.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`.
2. **MUST** trim surrounding whitespace from `id`, and **MUST** refuse, with `malformedRequest`, one that is empty after trimming.
3. **MUST** refuse, in this order: `invalidQuery` for a `query` that does not parse as DCQL; `unregisteredType` for a query referencing a type not in the schema registry; `unregisteredStatementType` for a `vetting.statementType` predicate not registered through vtc/endorsement-types/register; `invalidVetting` for vetting requirements no applicant could satisfy; `unsupportedRequirement` for a requirement the community cannot evaluate; `notPublishable` for a criterion the join manifest could not publish. Nothing is stored unless every check passes.
4. **MUST** store the criterion as given, adding no requirement it does not state and dropping none it does, and **MUST NOT** change its `admission`.
5. **MUST** replace any existing criterion with the same `id`, recording the caller and the execution time.
6. **MUST** audit the registration, naming the criterion id, its `admission`, and the administrator, and answer with the criterion it stored.

How a community decides a submission against a criterion — what meeting it means, and what `admission` obliges the community to do — is stated once, in [`vtc/join-requests/submit/0.3`](../../../../join-requests/submit/0.3/spec.md).

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — read at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

## Definitions

- **`id`** — the criterion's name, chosen by the administrator; the key.
- **`admission`** — `automatic` or `review`: what meeting the criterion does. See `Admission` in the shared definitions.
- **`query`** — the credentials required, as a DCQL query carried verbatim. Absent: no credential is required.
- **`credentialIssuers`** — whose credentials meet `query`: `community` (this community's own), `recognised` (this community's, or a community it recognises through [`vtc/recognition/check`](../../../../recognition/check/0.1/spec.md)), or `any` (any issuer whose credential verifies, further limited only by the query, including DCQL `trusted_authorities`). Present exactly when `query` is.
- **`invitationRequired`** — true: a valid, unconsumed invitation this community issued to the applicant ([`vtc/invitations/issue`](../../../../invitations/issue/0.1/spec.md)) is required. Absent or false: an invitation plays no part in meeting this criterion.
- **`vetting`** — a `VettingRequirements` object, the shape vtc/join-requests/manifest publishes. Every number in it is the community's own policy.
- Shapes are in [`vtc/_shared/0.2/schema-registry.schema.json`](../../../../_shared/0.2/schema-registry.schema.json).

## Request

A community administrator (`issuer`) sends the criterion to the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json). The examples show the payload only; each is a complete criterion, and none is a recommendation.

### Open: anyone may join, straight through

```json
{
  "id": "open",
  "admission": "automatic",
  "description": "Anyone may join."
}
```

### Open by review: anyone may apply, and an administrator decides

```json
{
  "id": "apply",
  "admission": "review",
  "description": "Tell us about yourself; an administrator will review your application."
}
```

### Invitation only

```json
{
  "id": "invited",
  "admission": "automatic",
  "invitationRequired": true,
  "description": "Members join by invitation."
}
```

### A credential from this community or one it recognises

```json
{
  "id": "partner-member",
  "admission": "automatic",
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
  "credentialIssuers": "recognised",
  "description": "Members of a partner community may join."
}
```

### Combined, and reviewed: a credential and two vetting statements, then an administrator decides

```json
{
  "id": "membership",
  "admission": "review",
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
  "credentialIssuers": "any",
  "description": "Show a membership credential and be vetted by two members; an administrator then decides.",
  "vetting": {
    "version": "0.1",
    "statementType": "https://registry.trustoverip.org/dtg/vsc/vetted/1",
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
```

A community wanting "invited or vetted, otherwise reviewed" registers three criteria — one with `invitationRequired`, one with `vetting`, and one with neither and `admission: review` — and a community wanting "reviewed, whatever is presented" registers only the last. Nothing presented outside a criterion admits anyone.

A full document:

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/schemas/accepts/register/0.2#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-02T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "id": "apply",
    "admission": "review",
    "description": "Tell us about yourself; an administrator will review your application."
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`: the stored criterion. A refusal is a `trust-task-error`.

### Registered

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/schemas/accepts/register/0.2#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-02T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "criterion": {
      "id": "apply",
      "admission": "review",
      "description": "Tell us about yourself; an administrator will review your application.",
      "createdAt": "2026-10-02T10:00:01Z",
      "createdByDid": "did:example:administrator"
    }
  }
}
```

## Security & Privacy

### Data carried

Registry configuration an administrator wrote. The description member is free text, bounded at 1024 characters, written by the registering administrator and read by other administrators, published to applicants, and used as the `purpose` a join query shows the holder; it is untrusted, attributed to its author wherever rendered, and **MUST NOT** carry personal data.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The audit trail ties each change to the administrator who made it.

### Retention

The criterion lives until it is replaced or deleted. The audit record names the criterion, its `admission` and the administrator, not the rest of its content.

### Consent/purpose

The purpose is to set the ways into the community. A criterion with `admission: automatic` admits without a person deciding, and one that requires nothing admits anyone who applies; both are legitimate policies and both are the community's to choose. What a criterion asks for is what applicants are asked to present, so an administrator **SHOULD** ask for the least evidence the community's governance requires.
