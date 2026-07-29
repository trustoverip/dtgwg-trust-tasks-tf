---
slug: vta/contexts/did-templates/create
version: "1.0"
title: VTA Context DID-Template — Create
summary: A context administrator (or super-admin) creates a DID template scoped to a context.
status: retired
supersededBy: vta/did-templates/create/2.0
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
  - role: Context administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Template creation is a privileged, audited write to a VTA's provisioning surface. The VTA MUST attribute the change to a specific context administrator for the audit record, so transport-independent producer identity is required.
sideEffects:
  level: mutating
  rationale: "Creates a context-scoped DID template; deletable."
subjectPath: /contextId
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vta/contexts/did-templates/create:duplicateName
    meaning: A template with this name already exists in the context.
    retryable: false
  - code: vta/contexts/did-templates/create:contextNotFound
    meaning: The target context does not exist.
    retryable: false
related:
  - vta/contexts/did-templates/get
  - vta/contexts/did-templates/update
  - vta/contexts/did-templates/delete
  - vta/contexts/did-templates/list
  - vta/contexts/did-templates/render
  - vta/did-templates/create
---

## Abstract

The **VTA Context DID-Template — Create** Trust Task stores a new DID template **scoped to a context** on a Verifiable Trust Agent. A *DID template* is the JSON shape of a DID document with `{TOKEN}` placeholders plus the variable contract the VTA's renderer enforces; the VTA fills the placeholders with keys it mints and caller-supplied variables when provisioning an integration. A *context administrator* (or a super-administrator) uploads the template; the VTA validates it against the v1 template schema, refuses duplicates within the context, persists it, and returns the stored record.

Context-scoped templates are visible and manageable within a single context and are managed by that context's admin. This contrasts with the **global** family, which is visible to every context on the VTA and managed only by super-administrators. To create a global template instead, use [`vta/did-templates/create`](../../../../did-templates/create/1.0/spec.md).

## Status of this Document

This specification is **retired** per [SPEC.md §5.3](../../../../../../SPEC.md#53-maturity-levels); it is superseded by [`vta/did-templates/create/2.0`](../../../../did-templates/create/2.0/spec.md), which merges the global and context-scoped families behind an optional `contextId`. The schema is frozen; the document is retained so already-issued documents remain verifiable.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the context administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/contexts/did-templates/create/1.0`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.contextId` with the context the template is scoped to.
3. Populate `payload.template` with a complete template document that validates against `#/$defs/DidTemplate`.
4. Include a `proof` member per [SPEC.md §4.7](../../../../../../SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the producer is neither an administrator of `payload.contextId` nor a super-administrator, respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../../SPEC.md#83-standard-error-codes)).
3. Where `payload.contextId` does not name an existing context, respond with `vta/contexts/did-templates/create:contextNotFound`.
4. Where `payload.template` is not a valid v1 template (bad name grammar, missing `{DID}` placeholder, undeclared placeholder, reserved variable name), respond with `malformedRequest`.
5. Where a template with the same `name` already exists in the context, respond with `vta/contexts/did-templates/create:duplicateName`.
6. On success, persist the template in the context scope, record the creator DID and timestamps, and return the stored [DidTemplateRecord](#response).

## Definitions

* **Context administrator.** The party invoking the task; identified by `issuer`. Holds the Admin role for `payload.contextId` on the VTA. A super-administrator MAY also invoke the task.
* **VTA.** The Verifiable Trust Agent that stores and renders templates; identified by `recipient`.
* **DidTemplate.** The authored template shape — see `#/$defs/DidTemplate` in `payload.schema.json`.
* **DidTemplateRecord.** A stored template plus provenance (scope, timestamps, creator DID) — see `#/$defs/DidTemplateRecord`.

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/contexts/did-templates/create/1.0` with a payload that validates against the top-level schema in `payload.schema.json`.

### Create a context-scoped mediator template

```json
{
  "id": "0a1b2c3d-4e5f-6071-8293-a4b5c6d7e8f9",
  "type": "https://trusttasks.org/spec/vta/contexts/did-templates/create/1.0",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-06-16T09:00:00Z",
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
    "created": "2026-06-16T09:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/contexts/did-templates/create/1.0#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is the persisted **DidTemplateRecord**.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../../SPEC.md#8-error-responses)), not the `#response` variant — including the `vta/contexts/did-templates/create:duplicateName` conflict and the `vta/contexts/did-templates/create:contextNotFound` error.

### The stored record

Response to the request example. Note the resolved `scope` carries the context the template was created in:

```json
{
  "id": "1b2c3d4e-5f60-7182-93a4-b5c6d7e8f9a0",
  "type": "https://trusttasks.org/spec/vta/contexts/did-templates/create/1.0#response",
  "threadId": "0a1b2c3d-4e5f-6071-8293-a4b5c6d7e8f9",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-16T09:00:01Z",
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
    "createdAt": 1781600401,
    "updatedAt": 1781600401,
    "createdBy": "did:web:admin.example"
  }
}
```

## Security & Privacy

**Privileged write, audited.** Creating a template changes what the VTA will mint for future integrations in the context, so the task is restricted to the context's administrators (and super-administrators) and the VTA records the creator DID. The **REQUIRED** `proof` binds the change to a specific operator for the audit trail and prevents a captured request being attributed to the wrong party.

**Scope is enforced server-side.** The VTA authorises the write against `payload.contextId` and stamps the resolved scope onto the stored record; a context admin cannot create a template outside the contexts they administer.

**Templates are shapes, not secrets.** A template contains only placeholder tokens and public document structure — never key material. The VTA mints all keys at render time; a template never carries a private key. Even so, the optional `ext` extension (see [SPEC.md §4.5.1](../../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload, so producers **MUST NOT** place data in `ext` they would not be comfortable signing.
