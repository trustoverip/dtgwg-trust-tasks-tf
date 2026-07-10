---
slug: vta/contexts/did-templates/list
version: "1.0"
title: VTA Context DID-Template — List
summary: A caller with context access lists templates scoped to a context.
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
  - role: Context member
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Listing a context's templates requires context access. The VTA MUST authenticate the caller to decide whether they may see the context's templates, so transport-independent producer identity is required.
sideEffects:
  level: none
  rationale: "Read-only listing of context-scoped templates."
subjectPath: /contextId
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related:
  - vta/contexts/did-templates/get
  - vta/contexts/did-templates/create
  - vta/contexts/did-templates/update
  - vta/contexts/did-templates/delete
  - vta/contexts/did-templates/render
  - vta/did-templates/list
---

## Abstract

The **VTA Context DID-Template — List** Trust Task returns the DID templates **scoped to a context** on a Verifiable Trust Agent. A *DID template* is the JSON shape of a DID document with `{TOKEN}` placeholders plus the variable contract the VTA's renderer enforces. A caller with access to the context names it; the VTA returns the stored records for that context, sorted by name.

Context-scoped templates are visible and manageable within a single context and are managed by that context's admin. This contrasts with the **global** family, which is visible to every context on the VTA and managed only by super-administrators. To list global templates instead, use [`vta/did-templates/list`](../../../../did-templates/list/1.0/spec.md).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the context member) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/contexts/did-templates/list/1.0`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.contextId` with the context the templates are scoped to.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../../SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the producer does not have access to `payload.contextId`, respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../../SPEC.md#83-standard-error-codes)).
3. On success, return every [DidTemplateRecord](#response) scoped to the context — sorted by name, and an empty array when the context has none.

## Definitions

* **Context member.** The party invoking the task; identified by `issuer`. Holds a role granting access to `payload.contextId` on the VTA.
* **VTA.** The Verifiable Trust Agent that stores and renders templates; identified by `recipient`.
* **DidTemplateRecord.** A stored template plus provenance (scope, timestamps, creator DID) — see `#/$defs/DidTemplateRecord`.

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/contexts/did-templates/list/1.0` with a payload that validates against the top-level schema in `payload.schema.json`.

### List a context's templates

```json
{
  "id": "4e5f6071-8293-a4b5-c6d7-e8f9a0b1c2d3",
  "type": "https://trusttasks.org/spec/vta/contexts/did-templates/list/1.0",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-06-16T09:10:00Z",
  "payload": {
    "contextId": "primary"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-06-16T09:10:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/contexts/did-templates/list/1.0#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload carries the `templates` array.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../../SPEC.md#8-error-responses)), not the `#response` variant.

### The template list

Response to the request example:

```json
{
  "id": "5f607182-93a4-b5c6-d7e8-f9a0b1c2d3e4",
  "type": "https://trusttasks.org/spec/vta/contexts/did-templates/list/1.0#response",
  "threadId": "4e5f6071-8293-a4b5-c6d7-e8f9a0b1c2d3",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-16T09:10:01Z",
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
        "scope": { "type": "context", "contextId": "primary" },
        "createdAt": 1781600401,
        "updatedAt": 1781600401,
        "createdBy": "did:web:admin.example"
      }
    ]
  }
}
```

## Security & Privacy

**Read gated on context access.** A context's template set is only returned to a caller with access to that context, so the **REQUIRED** `proof` lets the VTA authenticate the caller before disclosing the list. A caller without context access receives `permissionDenied`, not the records.

**Templates are shapes, not secrets.** A template contains only placeholder tokens and public document structure — never key material; the VTA mints all keys at render time. Even so, the optional `ext` extension (see [SPEC.md §4.5.1](../../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload, so producers **MUST NOT** place data in `ext` they would not be comfortable signing.
