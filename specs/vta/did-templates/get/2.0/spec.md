---
slug: vta/did-templates/get
version: "2.0"
title: VTA DID-Template — Get
summary: An authenticated caller fetches one DID template by name — global when contextId is absent, context-scoped when present.
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
  - role: Authenticated caller
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Fetching a template is a pure read that discloses only non-secret provisioning metadata and changes nothing. The transport-level session that authenticates the caller (plus context access when contextId is present) is sufficient to authorize it; a producer proof adds a transport-independent identity for the VTA's audit trail, so it is RECOMMENDED rather than REQUIRED. The blanket REQUIRED of 1.0 was re-derived per task for 2.0 — mutations keep REQUIRED, pure reads do not.
sideEffects:
  level: none
  rationale: "Read-only read of a DID template in the selected scope."
subjectPath: /name
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vta/did-templates/get:notFound
    meaning: No template with this name in the selected scope.
    retryable: false
related:
  - vta/did-templates/list
  - vta/did-templates/create
  - vta/did-templates/update
  - vta/did-templates/delete
  - vta/did-templates/render
---

## Abstract

The **VTA DID-Template — Get** Trust Task fetches one DID template by name from **one scope** on a Verifiable Trust Agent. A *DID template* is the JSON shape of a DID document with `{TOKEN}` placeholders plus the variable contract the VTA's renderer enforces; the VTA fills the placeholders with keys it mints and caller-supplied variables when provisioning an integration.

The scope is selected by the **optional `contextId`** field:

* **Absent** — the **global** scope: templates visible to every context on the VTA. Any authenticated caller may read them.
* **Present** — that **context's** scope: templates visible within a single context. The caller must have access to the context.

Version 2.0 merges the 1.0 pair [`vta/did-templates/get/1.0`](../1.0/spec.md) and [`vta/contexts/did-templates/get/1.0`](../../../contexts/did-templates/get/1.0/spec.md) into this single task; the per-scope authorization moved from the slug structure to the `contextId` field.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the authenticated caller) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/did-templates/get/2.0`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.name` with the template name to fetch, and `payload.contextId` with the target context — or omit `contextId` to read from the global scope.

A conforming producer **SHOULD** include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof) so the read is attributable independent of transport.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof` when present.
2. Where the producer is not an authenticated caller — or, when `payload.contextId` is present, lacks access to that context — respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../SPEC.md#83-standard-error-codes)).
3. Where no template named `payload.name` exists in the selected scope, respond with `vta/did-templates/get:notFound`.
4. On success, return the stored [DidTemplateRecord](#response).

## Definitions

* **Authenticated caller.** The party invoking the task; identified by `issuer`. Any caller the VTA can authenticate; access to `payload.contextId` is additionally required when it is present.
* **VTA.** The Verifiable Trust Agent that stores and renders templates; identified by `recipient`.
* **DidTemplateRecord.** A stored template plus provenance (scope, timestamps, creator DID) — see the [shared definition](../../../_shared/0.1/did-template.schema.json).

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/did-templates/get/2.0` with a payload that validates against the top-level schema in `payload.schema.json`.

### Fetch a context-scoped template

```json
{
  "id": "0a1b2c3d-4e5f-6071-8293-a4b5c6d7e8f9",
  "type": "https://trusttasks.org/spec/vta/did-templates/get/2.0",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-07-29T09:00:00Z",
  "payload": {
    "contextId": "primary",
    "name": "messaging-bridge"
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

Omit `contextId` to fetch the global template of the same name instead.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/did-templates/get/2.0#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is the persisted **DidTemplateRecord**.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant — including `vta/did-templates/get:notFound`.

### The stored record

Response to the request example. The resolved `scope` names where the record lives:

```json
{
  "id": "1b2c3d4e-5f60-7182-93a4-b5c6d7e8f9a0",
  "type": "https://trusttasks.org/spec/vta/did-templates/get/2.0#response",
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

**Scope selection is authorization, not filtering.** The `contextId` field selects which scope is read, and the VTA authorizes against it server-side: the global scope is open to any authenticated caller, a context scope only to callers with access to that context. Lack of access yields `permissionDenied` — never `notFound` — so the error does not leak whether a template of that name exists in a context the caller cannot see.

**Attributed read, proportionate proof.** Fetching a template discloses only non-secret provisioning metadata, so the producer `proof` is **RECOMMENDED** rather than REQUIRED: the transport session already authorizes the read, and the proof's value here is a transport-independent identity for the VTA's audit trail. Deployments that require attributable reads SHOULD reject unproven requests as a matter of local policy.

**Templates are shapes, not secrets.** A template contains only placeholder tokens and public document structure — never key material. The VTA mints all keys at render time; a template never carries a private key. Even so, the optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload when a proof is present, so producers **MUST NOT** place data in `ext` they would not be comfortable signing.
