---
slug: vta/did-templates/get
version: "1.0"
title: VTA DID-Template — Get
summary: An authenticated caller fetches one global DID template by name.
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - did-template
  - did
  - provisioning
  - get
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
  rationale: "Read-only read of a global DID template."
subjectPath: /name
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: vta/did-templates/get:notFound
    meaning: No global template with this name.
    retryable: false
related:
  - vta/did-templates/list
  - vta/did-templates/create
  - vta/did-templates/update
  - vta/did-templates/delete
  - vta/did-templates/render
  - vta/contexts/did-templates/get
---

## Abstract

The **VTA DID-Template — Get** Trust Task fetches one **global** DID template stored on a Verifiable Trust Agent, addressed by `name`. A *DID template* is the JSON shape of a DID document with `{TOKEN}` placeholders plus the variable contract the VTA's renderer enforces; the VTA fills the placeholders with keys it mints and caller-supplied variables when provisioning an integration. Any authenticated caller may read a global template; the VTA returns the stored record or, when no template carries that name, the `notFound` error.

Global templates are visible to every context on the VTA. To fetch a template scoped to a single context, use [`vta/contexts/did-templates/get`](../../../contexts/did-templates/get/1.0/spec.md).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the authenticated caller) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/did-templates/get/1.0`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.name` with the name of the global template to fetch.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the producer is not an authenticated caller, respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../SPEC.md#83-standard-error-codes)).
3. Where no global template with `payload.name` exists, respond with `vta/did-templates/get:notFound`.
4. On success, return the stored [DidTemplateRecord](#response) for that name.

## Definitions

* **Authenticated caller.** The party invoking the task; identified by `issuer`. Any caller the VTA can authenticate.
* **VTA.** The Verifiable Trust Agent that stores and renders templates; identified by `recipient`.
* **DidTemplateRecord.** A stored template plus provenance (scope, timestamps, creator DID) — see `#/$defs/Response`.

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/did-templates/get/1.0` with a payload that validates against the top-level schema in `payload.schema.json`.

### Fetch a global template by name

```json
{
  "id": "2c3d4e5f-6071-8293-a4b5-c6d7e8f9a0b1",
  "type": "https://trusttasks.org/spec/vta/did-templates/get/1.0",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-06-16T09:00:00Z",
  "payload": {
    "name": "messaging-bridge"
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

A success *response* document carries `type: https://trusttasks.org/spec/vta/did-templates/get/1.0#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is the persisted **DidTemplateRecord**.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant — including the `vta/did-templates/get:notFound` case.

### The stored record

Response to the request example:

```json
{
  "id": "3d4e5f60-7182-93a4-b5c6-d7e8f9a0b1c2",
  "type": "https://trusttasks.org/spec/vta/did-templates/get/1.0#response",
  "threadId": "2c3d4e5f-6071-8293-a4b5-c6d7e8f9a0b1",
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
    "scope": { "type": "global" },
    "createdAt": 1781600401,
    "updatedAt": 1781600401,
    "createdBy": "did:web:admin.example"
  }
}
```

## Security & Privacy

**Attributed read.** Fetching a template is a read, but the **REQUIRED** `proof` keeps the `vta/*` family's authorization model uniform and lets the VTA attribute the read to a specific caller in its audit trail. A captured request cannot be replayed under another party's identity.

**Templates are shapes, not secrets.** A template contains only placeholder tokens and public document structure — never key material. The VTA mints all keys at render time; a template never carries a private key. Even so, the optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload, so producers **MUST NOT** place data in `ext` they would not be comfortable signing.
