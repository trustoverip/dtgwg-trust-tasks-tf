---
slug: vta/did-templates/list
version: "2.0"
title: VTA DID-Template — List
summary: An authenticated caller lists the DID templates in one scope on a VTA — global when contextId is absent, context-scoped when present.
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - did-template
  - did
  - provisioning
  - list
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
  rationale: Listing is a pure read that discloses only non-secret provisioning metadata and changes nothing. The transport-level session that authenticates the caller (plus context access when contextId is present) is sufficient to authorize it; a producer proof adds a transport-independent identity for the VTA's audit trail, so it is RECOMMENDED rather than REQUIRED. The blanket REQUIRED of 1.0 was re-derived per task for 2.0 — mutations keep REQUIRED, pure reads do not.
sideEffects:
  level: none
  rationale: "Read-only listing of DID templates in the selected scope."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related:
  - vta/did-templates/get
  - vta/did-templates/create
  - vta/did-templates/update
  - vta/did-templates/delete
  - vta/did-templates/render
---

## Abstract

The **VTA DID-Template — List** Trust Task returns every DID template stored in **one scope** on a Verifiable Trust Agent. A *DID template* is the JSON shape of a DID document with `{TOKEN}` placeholders plus the variable contract the VTA's renderer enforces; the VTA fills the placeholders with keys it mints and caller-supplied variables when provisioning an integration.

The scope is selected by the **optional `contextId`** field:

* **Absent** — the **global** scope: templates visible to every context on the VTA. Any authenticated caller may list them.
* **Present** — that **context's** scope: templates visible within a single context. The caller must have access to the context.

Version 2.0 merges the 1.0 pair [`vta/did-templates/list/1.0`](../1.0/spec.md) and [`vta/contexts/did-templates/list/1.0`](../../../contexts/did-templates/list/1.0/spec.md) into this single task; the per-scope authorization moved from the slug structure to the `contextId` field.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the authenticated caller) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/did-templates/list/2.0`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.contextId` with the target context to list that context's templates, or omit it to list the global templates.

A conforming producer **SHOULD** include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof) so the read is attributable independent of transport.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof` when present.
2. Where the producer is not an authenticated caller — or, when `payload.contextId` is present, lacks access to that context — respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../SPEC.md#83-standard-error-codes)).
3. On success, return every stored template in the selected scope as `payload.templates` — an empty array when none exist.

## Definitions

* **Authenticated caller.** The party invoking the task; identified by `issuer`. Any caller the VTA can authenticate; access to `payload.contextId` is additionally required when it is present.
* **VTA.** The Verifiable Trust Agent that stores and renders templates; identified by `recipient`.
* **DidTemplateRecord.** A stored template plus provenance (scope, timestamps, creator DID) — see the [shared definition](../../../_shared/0.1/did-template.schema.json).

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/did-templates/list/2.0` with a payload that validates against the top-level schema in `payload.schema.json`. The only parameter is the optional `contextId` scope selector.

### List the global templates

```json
{
  "id": "4e5f6071-8293-a4b5-c6d7-e8f9a0b1c2d3",
  "type": "https://trusttasks.org/spec/vta/did-templates/list/2.0",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-07-29T09:00:00Z",
  "payload": {},
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

### List a context's templates

```json
{
  "id": "5f607182-93a4-b5c6-d7e8-f9a0b1c2d3e4",
  "type": "https://trusttasks.org/spec/vta/did-templates/list/2.0",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-07-29T09:05:00Z",
  "payload": {
    "contextId": "primary"
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/did-templates/list/2.0#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload carries the `templates` array of persisted **DidTemplateRecord** values for the selected scope.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant. Listing has no task-specific errors.

### The stored templates

Response to the global request example. Each record's resolved `scope` names where it lives:

```json
{
  "id": "60718293-a4b5-c6d7-e8f9-a0b1c2d3e4f5",
  "type": "https://trusttasks.org/spec/vta/did-templates/list/2.0#response",
  "threadId": "4e5f6071-8293-a4b5-c6d7-e8f9a0b1c2d3",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-07-29T09:00:01Z",
  "payload": {
    "templates": [
      {
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
        "scope": { "type": "global" },
        "createdAt": 1785315601,
        "updatedAt": 1785315601,
        "createdBy": "did:web:admin.example"
      }
    ]
  }
}
```

## Security & Privacy

**Scope selection is authorization, not filtering.** The `contextId` field selects which scope is read, and the VTA authorizes against it server-side: the global scope is open to any authenticated caller, a context scope only to callers with access to that context. A caller cannot enumerate a context's templates by guessing its id — lack of access yields `permissionDenied`, the same as for any other context operation.

**Attributed read, proportionate proof.** Listing templates discloses only non-secret provisioning metadata, so the producer `proof` is **RECOMMENDED** rather than REQUIRED: the transport session already authorizes the read, and the proof's value here is a transport-independent identity for the VTA's audit trail. Deployments that require attributable reads SHOULD reject unproven requests as a matter of local policy.

**Templates are shapes, not secrets.** A template contains only placeholder tokens and public document structure — never key material. The VTA mints all keys at render time; a template never carries a private key. Even so, the optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload when a proof is present, so producers **MUST NOT** place data in `ext` they would not be comfortable signing.
