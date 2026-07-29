---
slug: vta/contexts/did-templates/update
version: "1.0"
title: VTA Context DID-Template — Update
summary: A context administrator (or super-admin) replaces a context-scoped DID template.
status: retired
supersededBy: vta/did-templates/update/2.0
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - did-template
  - did
  - provisioning
  - update
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
  rationale: Template replacement is a privileged, audited write to a VTA's provisioning surface. The VTA MUST attribute the change to a specific context administrator for the audit record, so transport-independent producer identity is required.
sideEffects:
  level: mutating
  rationale: "Replaces a context-scoped DID template."
subjectPath: /contextId
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vta/contexts/did-templates/update:notFound
    meaning: No template with this name in the context.
    retryable: false
related:
  - vta/contexts/did-templates/create
  - vta/contexts/did-templates/get
  - vta/contexts/did-templates/delete
  - vta/contexts/did-templates/list
  - vta/contexts/did-templates/render
  - vta/did-templates/update
---

## Abstract

The **VTA Context DID-Template — Update** Trust Task replaces an existing DID template **scoped to a context** on a Verifiable Trust Agent. A *DID template* is the JSON shape of a DID document with `{TOKEN}` placeholders plus the variable contract the VTA's renderer enforces; the VTA fills the placeholders with keys it mints and caller-supplied variables when provisioning an integration. A *context administrator* (or a super-administrator) submits a full replacement template; the VTA validates it against the v1 template schema, requires the template to already exist in the context, persists the new shape while preserving the original creation provenance, and returns the stored record.

Context-scoped templates are visible and manageable within a single context and are managed by that context's admin. This contrasts with the **global** family, which is visible to every context on the VTA and managed only by super-administrators. To update a global template instead, use [`vta/did-templates/update`](../../../../did-templates/update/1.0/spec.md).

## Status of this Document

This specification is **retired** per [SPEC.md §5.3](../../../../../../SPEC.md#53-maturity-levels); it is superseded by [`vta/did-templates/update/2.0`](../../../../did-templates/update/2.0/spec.md), which merges the global and context-scoped families behind an optional `contextId`. The schema is frozen; the document is retained so already-issued documents remain verifiable.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the context administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/contexts/did-templates/update/1.0`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.contextId` with the context the template is scoped to.
3. Populate `payload.name` with the resource id of the template to replace, and `payload.template` with the full replacement document; `payload.name` MUST equal `payload.template.name`.
4. Include a `proof` member per [SPEC.md §4.7](../../../../../../SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the producer is neither an administrator of `payload.contextId` nor a super-administrator, respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../../SPEC.md#83-standard-error-codes)).
3. Where `payload.template` is not a valid v1 template (bad name grammar, missing `{DID}` placeholder, undeclared placeholder, reserved variable name), or where `payload.template.name` does not equal `payload.name`, respond with `malformedRequest`.
4. Where no template with `payload.name` exists in `payload.contextId`, respond with `vta/contexts/did-templates/update:notFound`.
5. On success, replace the stored template in the context scope, preserving `createdAt` and `createdBy`, advancing `updatedAt`, and return the stored [DidTemplateRecord](#response).

## Definitions

* **Context administrator.** The party invoking the task; identified by `issuer`. Holds the Admin role for `payload.contextId` on the VTA. A super-administrator MAY also invoke the task.
* **VTA.** The Verifiable Trust Agent that stores and renders templates; identified by `recipient`.
* **DidTemplate.** The authored template shape — see `#/$defs/DidTemplate` in `payload.schema.json`.
* **DidTemplateRecord.** A stored template plus provenance (scope, timestamps, creator DID) — see `#/$defs/DidTemplateRecord`.

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/contexts/did-templates/update/1.0` with a payload that validates against the top-level schema in `payload.schema.json`.

### Replace a context-scoped mediator template

```json
{
  "id": "2c3d4e5f-6071-7283-94a5-b6c7d8e9f0a1",
  "type": "https://trusttasks.org/spec/vta/contexts/did-templates/update/1.0",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-06-16T09:05:00Z",
  "payload": {
    "contextId": "primary",
    "name": "messaging-bridge",
    "template": {
      "schemaVersion": 1,
      "name": "messaging-bridge",
      "kind": "messaging-bridge",
      "description": "DIDComm messaging bridge host (now accepts didcomm/v2 only).",
      "methods": ["webvh", "web"],
      "requiredVars": ["MEDIATOR_DID"],
      "optionalVars": { "ACCEPT": ["didcomm/v2"] },
      "defaults": { "preRotationCount": 3 },
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
    "created": "2026-06-16T09:05:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/contexts/did-templates/update/1.0#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is the persisted **DidTemplateRecord**.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../../SPEC.md#8-error-responses)), not the `#response` variant — including the `vta/contexts/did-templates/update:notFound` error.

### The stored record

Response to the request example. Note that `createdAt`/`createdBy` are preserved from the original creation while `updatedAt` advances:

```json
{
  "id": "3d4e5f60-7182-7394-a5b6-c7d8e9f0a1b2",
  "type": "https://trusttasks.org/spec/vta/contexts/did-templates/update/1.0#response",
  "threadId": "2c3d4e5f-6071-7283-94a5-b6c7d8e9f0a1",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-16T09:05:01Z",
  "payload": {
    "schemaVersion": 1,
    "name": "messaging-bridge",
    "kind": "messaging-bridge",
    "description": "DIDComm messaging bridge host (now accepts didcomm/v2 only).",
    "methods": ["webvh", "web"],
    "requiredVars": ["MEDIATOR_DID"],
    "optionalVars": { "ACCEPT": ["didcomm/v2"] },
    "defaults": { "preRotationCount": 3 },
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
    "updatedAt": 1781600701,
    "createdBy": "did:web:admin.example"
  }
}
```

## Security & Privacy

**Privileged write, audited.** Replacing a template changes what the VTA will mint for future integrations in the context, so the task is restricted to the context's administrators (and super-administrators) and the VTA records the writer DID. The **REQUIRED** `proof` binds the change to a specific operator for the audit trail and prevents a captured request being attributed to the wrong party.

**Scope is enforced server-side.** The VTA authorises the write against `payload.contextId` and stamps the resolved scope onto the stored record; a context admin cannot update a template outside the contexts they administer, and a request that does not name an existing template in the context is rejected rather than silently creating one.

**Templates are shapes, not secrets.** A template contains only placeholder tokens and public document structure — never key material. The VTA mints all keys at render time; a template never carries a private key. Even so, the optional `ext` extension (see [SPEC.md §4.5.1](../../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload, so producers **MUST NOT** place data in `ext` they would not be comfortable signing.
