---
slug: vta/did-templates/list
version: "1.0"
title: VTA DID-Template — List
summary: An authenticated caller lists all global DID templates on a VTA.
status: retired
supersededBy: vta/did-templates/list/2.0
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - did-template
  - did
  - provisioning
  - list
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
  requirement: REQUIRED
  rationale: A read is attributed to a specific caller for the VTA's audit trail and to keep the vta/* family's authorization model uniform, so transport-independent producer identity is required.
sideEffects:
  level: none
  rationale: "Read-only listing of global DID templates."
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
  - vta/contexts/did-templates/list
---

## Abstract

The **VTA DID-Template — List** Trust Task returns every **global** DID template stored on a Verifiable Trust Agent. A *DID template* is the JSON shape of a DID document with `{TOKEN}` placeholders plus the variable contract the VTA's renderer enforces; the VTA fills the placeholders with keys it mints and caller-supplied variables when provisioning an integration. Any authenticated caller may list global templates; the request takes no parameters and the VTA returns the full set, or an empty array when none are stored.

Global templates are visible to every context on the VTA. To list the templates scoped to a single context, use [`vta/contexts/did-templates/list`](../../../contexts/did-templates/list/1.0/spec.md).

## Status of this Document

This specification is **retired** per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); it is superseded by [`vta/did-templates/list/2.0`](../2.0/spec.md), which merges the global and context-scoped families behind an optional `contextId`. The schema is frozen; the document is retained so already-issued documents remain verifiable.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the authenticated caller) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/did-templates/list/1.0`, with itself as `issuer` and the VTA as `recipient`.
2. Send an empty payload — the list request takes no parameters.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the producer is not an authenticated caller, respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../SPEC.md#83-standard-error-codes)).
3. On success, return every stored global template as `payload.templates` — an empty array when none exist.

## Definitions

* **Authenticated caller.** The party invoking the task; identified by `issuer`. Any caller the VTA can authenticate.
* **VTA.** The Verifiable Trust Agent that stores and renders templates; identified by `recipient`.
* **DidTemplateRecord.** A stored template plus provenance (scope, timestamps, creator DID) — see `#/$defs/DidTemplateRecord`.

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/did-templates/list/1.0` with a payload that validates against the top-level schema in `payload.schema.json`. The list request takes no parameters; the payload is empty.

### List the global templates

```json
{
  "id": "4e5f6071-8293-a4b5-c6d7-e8f9a0b1c2d3",
  "type": "https://trusttasks.org/spec/vta/did-templates/list/1.0",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-06-16T09:00:00Z",
  "payload": {},
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

A success *response* document carries `type: https://trusttasks.org/spec/vta/did-templates/list/1.0#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload carries the `templates` array of persisted **DidTemplateRecord** values.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant. Listing has no task-specific errors.

### The stored templates

Response to the request example:

```json
{
  "id": "5f607182-93a4-b5c6-d7e8-f9a0b1c2d3e4",
  "type": "https://trusttasks.org/spec/vta/did-templates/list/1.0#response",
  "threadId": "4e5f6071-8293-a4b5-c6d7-e8f9a0b1c2d3",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-16T09:00:01Z",
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
        "createdAt": 1781600401,
        "updatedAt": 1781600401,
        "createdBy": "did:web:admin.example"
      }
    ]
  }
}
```

## Security & Privacy

**Attributed read.** Listing templates is a read, but the **REQUIRED** `proof` keeps the `vta/*` family's authorization model uniform and lets the VTA attribute the read to a specific caller in its audit trail. A captured request cannot be replayed under another party's identity.

**Templates are shapes, not secrets.** A template contains only placeholder tokens and public document structure — never key material. The VTA mints all keys at render time; a template never carries a private key. Even so, the optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload, so producers **MUST NOT** place data in `ext` they would not be comfortable signing.
