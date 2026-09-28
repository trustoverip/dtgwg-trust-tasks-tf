---
slug: vtc/schemas/register
version: "0.1"
title: "VTC Schemas — Register"
summary: An administrator registers a credential type the community issues or accepts, optionally binding it to a DTG catalog type and a JSON Schema its credentials must conform to.
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
    Creates or replaces one registry entry. An `issues` entry is what issuance consults before minting a type, and its `credentialSchema` is what a minted credential is validated against, so a registration changes what the community will issue from then on. Recoverable by registering again or deleting.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries registry configuration — a type URI, a catalog binding, a JSON Schema and a description; the response echoes the stored entry. No personal data.
retention:
  class: durable
  rationale: >-
    The entry is community configuration that issuance and every Accepts criterion depend on, kept until replaced or deleted, and the change is audited.
maxDocumentBytes:
  request: 262144
  response: 327680
  rationale: >-
    The request carries a whole JSON Schema, which routinely exceeds the 64 KiB a community applies to a Trust Task by default. 256 KiB accommodates any credential schema an issuance path should be validating on every mint, and stays under the ~352 KiB that backup/put-chunk derives as the largest document surviving a 1 MiB mediator message after DIDComm encoding and two forward wrappers. The response echoes the entry and adds only metadata.
errorCodes:
  - code: vtc/schemas/register:invalidCredentialSchema
    meaning: "`credentialSchema` is not a JSON Schema the community's validator can compile, so no credential could be checked against it."
    retryable: false
related:
  - vtc/schemas/list
  - vtc/schemas/show
  - vtc/schemas/delete
  - vtc/schemas/accepts/register
  - vtc/endorsement-types/register
---

## Abstract

A community keeps a **schema registry**: the credential types it deals in. The **issues** half lists the types it mints — Invitation, Membership, Role, operator-defined endorsements — and issuance refuses a type that is not registered there; an entry's optional `credentialSchema` is what every credential of that type is validated against when it is minted. The **accepts** half lists the types the community recognises as evidence, and an Accepts criterion ([`vtc/schemas/accepts/register`](../../accepts/register/0.1/spec.md)) may reference only a registered type.

The **VTC Schemas — Register** Trust Task adds an entry, or replaces the entry for a type already registered. The type URI is the key, so registering it again is how an administrator changes its schema, binding or description.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`.
2. **MUST** trim surrounding whitespace from `typeUri` before using it as the key, and **MUST** refuse, with `malformedRequest`, one that is empty after trimming.
3. **MUST** refuse, with `invalidCredentialSchema`, a `credentialSchema` its validator cannot compile. A schema that does not compile would make every later issuance of the type fail, which is worse than refusing the registration.
4. **MUST** replace any existing entry for the same `typeUri`, recording the caller as `createdByDid` and the execution time as `createdAt`. This is create-or-replace, not create: there is no `exists` refusal.
5. **MUST** audit the registration, naming the type and the administrator, and answer with the entry it stored.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — read at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

## Definitions

- **`typeUri`** — the credential type URI (the W3C `type` value or the SD-JWT-VC `vct`); the registry key.
- **`kind`** — `issues` or `accepts`.
- **`dtgType`** — the DTG catalog type the entry binds to; absent for a community-defined endorsement type.
- **`credentialSchema`** — the JSON Schema a credential of the type must conform to; absent for no constraint.
- Shapes are in [`vtc/_shared/0.1/schema-registry.schema.json`](../../../_shared/0.1/schema-registry.schema.json).

## Request

A community administrator (`issuer`) sends the entry to the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Register an endorsement type the community issues, with a schema

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/schemas/register/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "typeUri": "https://riverside.example/credentials/WorkshopSafety",
    "kind": "issues",
    "credentialSchema": {
      "$schema": "https://json-schema.org/draft/2020-12/schema",
      "type": "object",
      "required": [
        "credentialSubject"
      ]
    },
    "description": "Completed the workshop safety induction."
  }
}
```

### Register a type the community accepts as evidence

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/schemas/register/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001fe",
  "payload": {
    "typeUri": "https://openvtc.org/credentials/MembershipCredential",
    "kind": "accepts",
    "dtgType": "MembershipCredential"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`: the stored entry. A refusal is a `trust-task-error`.

### Registered

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vtc/schemas/register/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "schema": {
      "typeUri": "https://riverside.example/credentials/WorkshopSafety",
      "kind": "issues",
      "credentialSchema": {
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "required": [
          "credentialSubject"
        ]
      },
      "description": "Completed the workshop safety induction.",
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
