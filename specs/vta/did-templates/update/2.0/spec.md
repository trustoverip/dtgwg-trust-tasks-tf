---
slug: vta/did-templates/update
version: "2.0"
title: VTA DID-Template — Update
summary: An administrator replaces an existing DID template — global (super-admin) when contextId is absent, context-scoped (context admin) when present.
status: draft
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
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Replacing a template is a privileged, audited write to a VTA's provisioning surface. The VTA MUST attribute the change to a specific administrator — a super-administrator for the global scope, a context administrator for a context scope — for the audit record, so transport-independent producer identity is required.
sideEffects:
  level: mutating
  rationale: "Replaces an existing DID template in the selected scope."
subjectPath: /name
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vta/did-templates/update:notFound
    meaning: No template with this name in the selected scope to replace.
    retryable: false
related:
  - vta/did-templates/create
  - vta/did-templates/get
  - vta/did-templates/delete
  - vta/did-templates/list
  - vta/did-templates/render
---

## Abstract

The **VTA DID-Template — Update** Trust Task replaces an existing DID template in **one scope** on a Verifiable Trust Agent. The `name` is the resource id within the scope and MUST equal `template.name`; the VTA validates the replacement against the v1 template schema, swaps the stored body, preserves `createdAt`/`createdBy`, advances `updatedAt`, and returns the stored record.

The scope is selected by the **optional `contextId`** field:

* **Absent** — the **global** scope. Gated on a **super-administrator**.
* **Present** — that **context's** scope. Gated on that **context's administrator** (or a super-administrator).

Version 2.0 merges the 1.0 pair [`vta/did-templates/update/1.0`](../1.0/spec.md) and [`vta/contexts/did-templates/update/1.0`](../../../contexts/did-templates/update/1.0/spec.md) into this single task; the per-scope authorization moved from the slug structure to the `contextId` field.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/did-templates/update/2.0`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.name` with the resource id and `payload.template` with the full replacement document, with `template.name` equal to `payload.name`.
3. Populate `payload.contextId` with the target context to replace a context-scoped template, or omit it to replace a global template.
4. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where `payload.contextId` is absent and the producer is not a super-administrator, respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../SPEC.md#83-standard-error-codes)).
3. Where `payload.contextId` is present and the producer is neither an administrator of that context nor a super-administrator, respond with `permissionDenied`.
4. Where `payload.name` does not equal `payload.template.name`, or the template is not a valid v1 template, respond with `malformedRequest`.
5. Where no template named `payload.name` exists in the selected scope, respond with `vta/did-templates/update:notFound`.
6. On success, replace the stored body, preserve `createdAt`/`createdBy`, advance `updatedAt`, and return the stored [DidTemplateRecord](#response).

## Definitions

* **Administrator.** The party invoking the task; identified by `issuer`. A **super-administrator** when `contextId` is absent; an administrator of `payload.contextId` (or a super-administrator) when it is present.
* **VTA.** The Verifiable Trust Agent that stores and renders templates; identified by `recipient`.
* **DidTemplate / DidTemplateRecord.** The authored and persisted template shapes — see the [shared definition](../../../_shared/0.1/did-template.schema.json).

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/did-templates/update/2.0` with a payload that validates against the top-level schema in `payload.schema.json`.

### Replace a global template

```json
{
  "id": "2c3d4e5f-6071-8293-a4b5-c6d7e8f9a0b1",
  "type": "https://trusttasks.org/spec/vta/did-templates/update/2.0",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-07-29T09:00:00Z",
  "payload": {
    "name": "messaging-bridge",
    "template": {
      "schemaVersion": 1,
      "name": "messaging-bridge",
      "kind": "messaging-bridge",
      "description": "DIDComm messaging bridge host (rev 2).",
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
    "created": "2026-07-29T09:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

Add `"contextId": "primary"` to the payload to replace the context-scoped template of the same name instead.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/did-templates/update/2.0#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is the persisted **DidTemplateRecord**.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant — including `vta/did-templates/update:notFound`.

### The stored record

Response to the request example. `createdAt`/`createdBy` are preserved; `updatedAt` is advanced:

```json
{
  "id": "3d4e5f60-7182-93a4-b5c6-d7e8f9a0b1c2",
  "type": "https://trusttasks.org/spec/vta/did-templates/update/2.0#response",
  "threadId": "2c3d4e5f-6071-8293-a4b5-c6d7e8f9a0b1",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-07-29T09:00:01Z",
  "payload": {
    "schemaVersion": 1,
    "name": "messaging-bridge",
    "kind": "messaging-bridge",
    "description": "DIDComm messaging bridge host (rev 2).",
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
    "scope": { "type": "global" },
    "createdAt": 1785315601,
    "updatedAt": 1785319201,
    "createdBy": "did:web:admin.example"
  }
}
```

## Security & Privacy

**Privileged write, audited.** Replacing a template changes what the VTA will mint for future integrations in the selected scope, so the task is gated per scope — super-administrators for the global scope, the context's administrators (or super-administrators) for a context scope. The **REQUIRED** `proof` binds the change to a specific operator for the audit trail and prevents a captured request being attributed to the wrong party.

**Scope is enforced server-side.** The VTA authorizes the write against `payload.contextId` (or its absence); a context admin cannot replace templates outside the contexts they administer, and cannot reach the global scope at all.

**Templates are shapes, not secrets.** A template contains only placeholder tokens and public document structure — never key material. The VTA mints all keys at render time; a template never carries a private key. Even so, the optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload, so producers **MUST NOT** place data in `ext` they would not be comfortable signing.
