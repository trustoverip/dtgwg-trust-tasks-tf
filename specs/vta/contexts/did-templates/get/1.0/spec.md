---
slug: vta/contexts/did-templates/get
version: "1.0"
title: VTA Context DID-Template — Get
summary: A caller with context access fetches one context-scoped template by name.
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - did-template
  - did
  - provisioning
  - get
  - context
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Context member
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Reading a context-scoped template requires context access. The VTA MUST authenticate the caller to decide whether they may see the context's templates, so transport-independent producer identity is required.
sideEffects:
  level: none
  rationale: "Read-only read of a context-scoped template."
subjectPath: /contextId
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vta/contexts/did-templates/get:notFound
    meaning: No template with this name in the context.
    retryable: false
related:
  - vta/contexts/did-templates/list
  - vta/contexts/did-templates/create
  - vta/contexts/did-templates/update
  - vta/contexts/did-templates/delete
  - vta/contexts/did-templates/render
  - vta/did-templates/get
---

## Abstract

The **VTA Context DID-Template — Get** Trust Task fetches a single DID template **scoped to a context** from a Verifiable Trust Agent by name. A *DID template* is the JSON shape of a DID document with `{TOKEN}` placeholders plus the variable contract the VTA's renderer enforces. A caller with access to the context names the template; the VTA returns the stored record.

Context-scoped templates are visible and manageable within a single context and are managed by that context's admin. This contrasts with the **global** family, which is visible to every context on the VTA and managed only by super-administrators. To fetch a global template instead, use [`vta/did-templates/get`](../../../../did-templates/get/1.0/spec.md).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the context member) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/contexts/did-templates/get/1.0`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.contextId` with the context the template is scoped to and `payload.name` with the template's name.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../../SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the producer does not have access to `payload.contextId`, respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../../SPEC.md#83-standard-error-codes)).
3. Where no template named `payload.name` exists in the context, respond with `vta/contexts/did-templates/get:notFound`.
4. On success, return the stored [DidTemplateRecord](#response).

## Definitions

* **Context member.** The party invoking the task; identified by `issuer`. Holds a role granting access to `payload.contextId` on the VTA.
* **VTA.** The Verifiable Trust Agent that stores and renders templates; identified by `recipient`.
* **DidTemplateRecord.** A stored template plus provenance (scope, timestamps, creator DID) — see `#/$defs/DidTemplateRecord`.

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/contexts/did-templates/get/1.0` with a payload that validates against the top-level schema in `payload.schema.json`.

### Fetch a context-scoped template by name

```json
{
  "id": "2c3d4e5f-6071-8293-a4b5-c6d7e8f9a0b1",
  "type": "https://trusttasks.org/spec/vta/contexts/did-templates/get/1.0",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-06-16T09:05:00Z",
  "payload": {
    "contextId": "primary",
    "name": "messaging-bridge"
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

A success *response* document carries `type: https://trusttasks.org/spec/vta/contexts/did-templates/get/1.0#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is the persisted **DidTemplateRecord**.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../../SPEC.md#8-error-responses)), not the `#response` variant — including the `vta/contexts/did-templates/get:notFound` error.

### The stored record

Response to the request example:

```json
{
  "id": "3d4e5f60-7182-93a4-b5c6-d7e8f9a0b1c2",
  "type": "https://trusttasks.org/spec/vta/contexts/did-templates/get/1.0#response",
  "threadId": "2c3d4e5f-6071-8293-a4b5-c6d7e8f9a0b1",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-16T09:05:01Z",
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

**Read gated on context access.** A context-scoped template is only returned to a caller with access to its context, so the **REQUIRED** `proof` lets the VTA authenticate the caller before disclosing the template. A caller without context access receives `permissionDenied`, not the record.

**Templates are shapes, not secrets.** A template contains only placeholder tokens and public document structure — never key material; the VTA mints all keys at render time. Even so, the optional `ext` extension (see [SPEC.md §4.5.1](../../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload, so producers **MUST NOT** place data in `ext` they would not be comfortable signing.
