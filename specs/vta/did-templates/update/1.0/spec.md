---
slug: vta/did-templates/update
version: "1.0"
title: VTA DID-Template — Update
summary: A super-administrator replaces an existing global DID template.
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
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Super-administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Replacing a template is a privileged, audited write to a VTA's provisioning surface. The VTA MUST attribute the change to a specific super-administrator for the audit record, so transport-independent producer identity is required.
sideEffects:
  level: mutating
  rationale: "Replaces an existing global DID template."
subjectPath: /name
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vta/did-templates/update:notFound
    meaning: No global template with this name to replace.
    retryable: false
related:
  - vta/did-templates/create
  - vta/did-templates/get
  - vta/did-templates/delete
  - vta/did-templates/list
  - vta/did-templates/render
  - vta/contexts/did-templates/update
---

## Abstract

The **VTA DID-Template — Update** Trust Task replaces an existing **global** DID template on a Verifiable Trust Agent. A *DID template* is the JSON shape of a DID document with `{TOKEN}` placeholders plus the variable contract the VTA's renderer enforces; the VTA fills the placeholders with keys it mints and caller-supplied variables when provisioning an integration. A *super-administrator* supplies the template's `name` and a complete replacement body; the VTA validates the body against the v1 template schema, refuses the change if no template with that name exists, replaces the stored body while preserving its original provenance, and returns the updated record.

Update is the way to evolve a template in place: the resource `name` is unchanged, every consumer that renders from that name immediately sees the new shape, and the VTA preserves `createdAt`/`createdBy` while advancing `updatedAt`. To scope a template change to a single context — managed by that context's admin — use [`vta/contexts/did-templates/update`](../../../contexts/did-templates/update/1.0/spec.md).

## Status of this Document

This specification is **retired** per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); it is superseded by [`vta/did-templates/update/2.0`](../2.0/spec.md), which merges the global and context-scoped families behind an optional `contextId`. The schema is frozen; the document is retained so already-issued documents remain verifiable.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the super-administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/did-templates/update/1.0`, with itself as `issuer` and the VTA as `recipient`.
2. Set `payload.name` to the name of the existing global template and populate `payload.template` with a complete replacement that validates against `#/$defs/DidTemplate`, where `template.name` equals `payload.name`.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the producer does not hold the super-administrator role, respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../SPEC.md#83-standard-error-codes)).
3. Where `payload.template` is not a valid v1 template (bad name grammar, missing `{DID}` placeholder, undeclared placeholder, reserved variable name), or where `payload.template.name` does not equal `payload.name`, respond with the framework's `malformedRequest`.
4. Where no global template with `payload.name` exists, respond with `vta/did-templates/update:notFound`.
5. On success, replace the stored template body in the global scope, preserve `createdAt` and `createdBy`, advance `updatedAt`, and return the stored [DidTemplateRecord](#response).

## Definitions

* **Super-administrator.** The party invoking the task; identified by `issuer`. Holds the unrestricted Admin role on the VTA.
* **VTA.** The Verifiable Trust Agent that stores and renders templates; identified by `recipient`.
* **DidTemplate.** The authored template shape — see `#/$defs/DidTemplate` in `payload.schema.json`.
* **DidTemplateRecord.** A stored template plus provenance (scope, timestamps, creator DID) — see `#/$defs/DidTemplateRecord`.

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/did-templates/update/1.0` with a payload that validates against the top-level schema in `payload.schema.json`.

### Replace a global mediator template

```json
{
  "id": "2c3d4e5f-6071-8293-a4b5-c6d7e8f9a0b1",
  "type": "https://trusttasks.org/spec/vta/did-templates/update/1.0",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-06-16T10:00:00Z",
  "payload": {
    "name": "messaging-bridge",
    "template": {
      "schemaVersion": 1,
      "name": "messaging-bridge",
      "kind": "messaging-bridge",
      "description": "DIDComm messaging bridge host (v2 endpoint).",
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
    "created": "2026-06-16T10:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/did-templates/update/1.0#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is the persisted **DidTemplateRecord**.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant — including the `vta/did-templates/update:notFound` case.

### The stored record

Response to the request example. `createdAt` and `createdBy` are unchanged from the original creation; `updatedAt` advances to this write:

```json
{
  "id": "3d4e5f60-7182-93a4-b5c6-d7e8f9a0b1c2",
  "type": "https://trusttasks.org/spec/vta/did-templates/update/1.0#response",
  "threadId": "2c3d4e5f-6071-8293-a4b5-c6d7e8f9a0b1",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-16T10:00:01Z",
  "payload": {
    "schemaVersion": 1,
    "name": "messaging-bridge",
    "kind": "messaging-bridge",
    "description": "DIDComm messaging bridge host (v2 endpoint).",
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
    "createdAt": 1781600401,
    "updatedAt": 1781604001,
    "createdBy": "did:web:admin.example"
  }
}
```

## Security & Privacy

**Privileged write, audited.** Replacing a template changes what the VTA will mint for future integrations, so the task is restricted to super-administrators and the VTA records the writer DID. The **REQUIRED** `proof` binds the change to a specific operator for the audit trail and prevents a captured request being attributed to the wrong party. Because update mutates a name every other context can render from, the consumer rejects any request whose `payload.name` does not match `payload.template.name`, so an operator can never silently retarget a different template than they intended.

**Templates are shapes, not secrets.** A template contains only placeholder tokens and public document structure — never key material. The VTA mints all keys at render time; a template never carries a private key. Even so, the optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload, so producers **MUST NOT** place data in `ext` they would not be comfortable signing.
