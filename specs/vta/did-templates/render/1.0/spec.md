---
slug: vta/did-templates/render
version: "1.0"
title: VTA DID-Template — Render
summary: An authenticated caller renders a global DID template to a DID document, supplying variables.
status: retired
supersededBy: vta/did-templates/render/2.0
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - did-template
  - did
  - provisioning
  - render
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
  rationale: Rendering reads a VTA's provisioning surface and injects VTA-scoped ambient variables, so the VTA MUST attribute the request to a specific authenticated caller. Transport-independent producer identity is required.
sideEffects:
  level: none
  rationale: "Renders a global DID template to a DID document; produces output, persists nothing."
subjectPath: /name
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vta/did-templates/render:notFound
    meaning: No global template with this name.
    retryable: false
related:
  - vta/did-templates/create
  - vta/did-templates/get
  - vta/did-templates/update
  - vta/did-templates/delete
  - vta/did-templates/list
  - vta/contexts/did-templates/render
---

## Abstract

The **VTA DID-Template — Render** Trust Task substitutes a **global** DID template's `{TOKEN}` placeholders to produce a concrete DID document. A *DID template* is the JSON shape of a DID document with `{TOKEN}` placeholders plus the variable contract the VTA's renderer enforces. An *authenticated caller* names the template and supplies variables; the VTA injects its ambient variables (`VTA_DID`, `VTA_URL`, `NOW`), merges the caller's values on top, substitutes every placeholder, and returns the rendered document.

Render lets a caller preview or materialise the document a template produces without provisioning an integration. It is read-only with respect to template storage — it never mints keys and never persists anything. To render a template scoped to a single context — managed by that context's admin — use [`vta/contexts/did-templates/render`](../../../contexts/did-templates/render/1.0/spec.md).

## Status of this Document

This specification is **retired** per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); it is superseded by [`vta/did-templates/render/2.0`](../2.0/spec.md), which merges the global and context-scoped families behind an optional `contextId`. The schema is frozen; the document is retained so already-issued documents remain verifiable.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the authenticated caller) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/did-templates/render/1.0`, with itself as `issuer` and the VTA as `recipient`.
2. Set `payload.name` to the name of an existing global template and supply every variable the template declares as required in `payload.vars`.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where no global template with `payload.name` exists, respond with `vta/did-templates/render:notFound`.
3. Inject the ambient variables (`VTA_DID`, `VTA_URL`, `NOW`) server-side and merge `payload.vars` on top before substitution.
4. Where a required variable is missing, or a placeholder in the template cannot be resolved from the merged variable set, respond with the framework's `malformedRequest`.
5. On success, substitute every placeholder and return the rendered DID document in the [response](#response).

## Definitions

* **Authenticated caller.** The party invoking the task; identified by `issuer`. Holds any authenticated role on the VTA.
* **VTA.** The Verifiable Trust Agent that stores and renders templates; identified by `recipient`.
* **DidTemplate.** The authored template shape — see `#/$defs/DidTemplate` in the create spec's `payload.schema.json`.
* **Ambient variables.** VTA-scoped values the renderer supplies itself — `VTA_DID`, `VTA_URL`, `NOW` — which the caller cannot override.

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/did-templates/render/1.0` with a payload that validates against the top-level schema in `payload.schema.json`.

### Render a global mediator template

```json
{
  "id": "60718293-a4b5-c6d7-e8f9-a0b1c2d3e4f5",
  "type": "https://trusttasks.org/spec/vta/did-templates/render/1.0",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-06-16T12:00:00Z",
  "payload": {
    "name": "messaging-bridge",
    "vars": {
      "MEDIATOR_DID": "did:web:mediator.example",
      "ACCEPT": ["didcomm/v2"]
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-06-16T12:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/did-templates/render/1.0#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload carries the rendered `document`.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant — including the `vta/did-templates/render:notFound` case.

### The rendered document

Response to the request example. Ambient placeholders (`{DID}`, `{SIGNING_KEY_MB}`) and the supplied variables are resolved:

```json
{
  "id": "718293a4-b5c6-d7e8-f9a0-b1c2d3e4f5a6",
  "type": "https://trusttasks.org/spec/vta/did-templates/render/1.0#response",
  "threadId": "60718293-a4b5-c6d7-e8f9-a0b1c2d3e4f5",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-16T12:00:01Z",
  "payload": {
    "document": {
      "id": "did:web:vta.example",
      "verificationMethod": [
        {
          "id": "did:web:vta.example#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
          "type": "Multikey",
          "controller": "did:web:vta.example",
          "publicKeyMultibase": "z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
        }
      ],
      "service": [
        {
          "id": "did:web:vta.example#didcomm",
          "type": "DIDCommMessaging",
          "serviceEndpoint": { "uri": "did:web:mediator.example", "accept": ["didcomm/v2"] }
        }
      ]
    }
  }
}
```

## Security & Privacy

**Authenticated read with ambient injection.** Rendering returns a document derived from the VTA's own identity — the renderer injects `VTA_DID`, `VTA_URL`, and `NOW` and the caller cannot override them — so the task requires an authenticated caller. The **REQUIRED** `proof` binds the request to a specific operator for the audit trail and prevents a captured request being attributed to the wrong party.

**No key material, no persistence.** Render substitutes only placeholder tokens and public document structure; it never mints keys and never writes to template storage, so a rendered document carries no secrets. The optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload, so producers **MUST NOT** place data in `ext` they would not be comfortable signing.
