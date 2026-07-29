---
slug: vta/did-templates/create
version: "2.0"
title: VTA DID-Template — Create
summary: An administrator uploads a new DID template — global (super-admin) when contextId is absent, context-scoped (context admin) when present.
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - did-template
  - did
  - provisioning
  - create
  - context
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Template creation is a privileged, audited write to a VTA's provisioning surface. The VTA MUST attribute the change to a specific administrator — a super-administrator for the global scope, a context administrator for a context scope — for the audit record, so transport-independent producer identity is required.
sideEffects:
  level: mutating
  rationale: "Uploads a new DID template in the selected scope; deletable."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vta/did-templates/create:duplicateName
    meaning: A template with this name already exists in the selected scope. Use update to replace it.
    retryable: false
  - code: vta/did-templates/create:contextNotFound
    meaning: The context named by contextId does not exist.
    retryable: false
related:
  - vta/did-templates/get
  - vta/did-templates/update
  - vta/did-templates/delete
  - vta/did-templates/list
  - vta/did-templates/render
---

## Abstract

The **VTA DID-Template — Create** Trust Task stores a new DID template in **one scope** on a Verifiable Trust Agent. A *DID template* is the JSON shape of a DID document with `{TOKEN}` placeholders plus the variable contract the VTA's renderer enforces; the VTA fills the placeholders with keys it mints and caller-supplied variables when provisioning an integration. The VTA validates the uploaded template against the v1 template schema, refuses duplicates within the scope, persists it, and returns the stored record.

The scope is selected by the **optional `contextId`** field:

* **Absent** — the **global** scope: the template becomes visible to every context on the VTA. Gated on a **super-administrator**.
* **Present** — that **context's** scope: the template is visible and manageable within a single context. Gated on that **context's administrator** (or a super-administrator).

Version 2.0 merges the 1.0 pair [`vta/did-templates/create/1.0`](../1.0/spec.md) and [`vta/contexts/did-templates/create/1.0`](../../../contexts/did-templates/create/1.0/spec.md) into this single task; the per-scope authorization moved from the slug structure to the `contextId` field.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/did-templates/create/2.0`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.template` with a complete template document that validates against the shared `DidTemplate` definition.
3. Populate `payload.contextId` with the target context to create a context-scoped template, or omit it to create a global template.
4. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where `payload.contextId` is absent and the producer is not a super-administrator, respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../SPEC.md#83-standard-error-codes)).
3. Where `payload.contextId` is present and the producer is neither an administrator of that context nor a super-administrator, respond with `permissionDenied`.
4. Where `payload.contextId` is present but does not name an existing context, respond with `vta/did-templates/create:contextNotFound`.
5. Where `payload.template` is not a valid v1 template (bad name grammar, missing `{DID}` placeholder, undeclared placeholder, reserved variable name), respond with `malformedRequest`.
6. Where a template with the same `name` already exists in the selected scope, respond with `vta/did-templates/create:duplicateName`.
7. On success, persist the template in the selected scope, record the creator DID and timestamps, and return the stored [DidTemplateRecord](#response).

## Definitions

* **Administrator.** The party invoking the task; identified by `issuer`. A **super-administrator** when `contextId` is absent; an administrator of `payload.contextId` (or a super-administrator) when it is present.
* **VTA.** The Verifiable Trust Agent that stores and renders templates; identified by `recipient`.
* **DidTemplate.** The authored template shape — see the [shared definition](../../../_shared/0.1/did-template.schema.json).
* **DidTemplateRecord.** A stored template plus provenance (scope, timestamps, creator DID) — see the same shared file.

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/did-templates/create/2.0` with a payload that validates against the top-level schema in `payload.schema.json`.

### Create a context-scoped mediator template

```json
{
  "id": "0a1b2c3d-4e5f-6071-8293-a4b5c6d7e8f9",
  "type": "https://trusttasks.org/spec/vta/did-templates/create/2.0",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-07-29T09:00:00Z",
  "payload": {
    "contextId": "primary",
    "template": {
      "schemaVersion": 1,
      "name": "messaging-bridge",
      "kind": "messaging-bridge",
      "description": "DIDComm messaging bridge host.",
      "methods": ["webvh", "web"],
      "requiredVars": ["MEDIATOR_DID"],
      "optionalVars": { "ACCEPT": ["didcomm/v2"] },
      "defaults": { "preRotationCount": 2 },
      "document": {
        "id": "{DID}",
        "verificationMethod": [
          {
            "id": "{DID}#{SIGNING_KEY_MB}",
            "type": "Multikey",
            "controller": "{DID}",
            "publicKeyMultibase": "{SIGNING_KEY_MB}"
          }
        ],
        "service": [
          {
            "id": "{DID}#didcomm",
            "type": "DIDCommMessaging",
            "serviceEndpoint": { "uri": "{MEDIATOR_DID}", "accept": "{ACCEPT}" }
          }
        ]
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-07-29T09:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

Omit `contextId` to create the same template in the global scope instead (super-admin gated).

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/did-templates/create/2.0#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is the persisted **DidTemplateRecord**.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant — including the `vta/did-templates/create:duplicateName` conflict and the `vta/did-templates/create:contextNotFound` error.

### The stored record

Response to the request example. Note the resolved `scope` carries the context the template was created in:

```json
{
  "id": "1b2c3d4e-5f60-7182-93a4-b5c6d7e8f9a0",
  "type": "https://trusttasks.org/spec/vta/did-templates/create/2.0#response",
  "threadId": "0a1b2c3d-4e5f-6071-8293-a4b5c6d7e8f9",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-07-29T09:00:01Z",
  "payload": {
    "schemaVersion": 1,
    "name": "messaging-bridge",
    "kind": "messaging-bridge",
    "description": "DIDComm messaging bridge host.",
    "methods": ["webvh", "web"],
    "requiredVars": ["MEDIATOR_DID"],
    "optionalVars": { "ACCEPT": ["didcomm/v2"] },
    "defaults": { "preRotationCount": 2 },
    "document": {
      "id": "{DID}",
      "verificationMethod": [
        {
          "id": "{DID}#{SIGNING_KEY_MB}",
          "type": "Multikey",
          "controller": "{DID}",
          "publicKeyMultibase": "{SIGNING_KEY_MB}"
        }
      ],
      "service": [
        {
          "id": "{DID}#didcomm",
          "type": "DIDCommMessaging",
          "serviceEndpoint": { "uri": "{MEDIATOR_DID}", "accept": "{ACCEPT}" }
        }
      ]
    },
    "scope": { "type": "context", "contextId": "primary" },
    "createdAt": 1785315601,
    "updatedAt": 1785315601,
    "createdBy": "did:web:admin.example"
  }
}
```

## Security & Privacy

**Privileged write, audited.** Creating a template changes what the VTA will mint for future integrations in the selected scope, so the task is gated per scope — super-administrators for the global scope, the context's administrators (or super-administrators) for a context scope — and the VTA records the creator DID. The **REQUIRED** `proof` binds the change to a specific operator for the audit trail and prevents a captured request being attributed to the wrong party.

**Scope is enforced server-side.** The VTA authorizes the write against `payload.contextId` (or its absence) and stamps the resolved scope onto the stored record; a context admin cannot create a template outside the contexts they administer, and cannot reach the global scope at all.

**Templates are shapes, not secrets.** A template contains only placeholder tokens and public document structure — never key material. The VTA mints all keys at render time; a template never carries a private key. Even so, the optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload, so producers **MUST NOT** place data in `ext` they would not be comfortable signing.
