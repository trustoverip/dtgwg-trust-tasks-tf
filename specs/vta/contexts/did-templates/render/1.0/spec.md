---
slug: vta/contexts/did-templates/render
version: "1.0"
title: VTA Context DID-Template — Render
summary: A caller with context access renders a context-scoped template to a DID document.
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - did-template
  - did
  - provisioning
  - render
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
  rationale: Rendering reads context-scoped configuration and binds caller-supplied variables to a VTA's provisioning surface. The VTA MUST attribute the render to a specific context member for the audit record, so transport-independent producer identity is required.
sideEffects:
  level: none
  rationale: "Renders a context-scoped template to a DID document; produces output, persists nothing."
subjectPath: /contextId
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vta/contexts/did-templates/render:notFound
    meaning: No template with this name in the context.
    retryable: false
related:
  - vta/contexts/did-templates/create
  - vta/contexts/did-templates/get
  - vta/contexts/did-templates/update
  - vta/contexts/did-templates/delete
  - vta/contexts/did-templates/list
  - vta/did-templates/render
---

## Abstract

The **VTA Context DID-Template — Render** Trust Task renders a DID template **scoped to a context** on a Verifiable Trust Agent to a concrete DID document. A *DID template* is the JSON shape of a DID document with `{TOKEN}` placeholders plus the variable contract the VTA's renderer enforces. A *caller with context access* names the template and supplies the declared variables; the VTA injects ambient values (`VTA_DID`, `VTA_URL`, `NOW`, `CONTEXT_ID`, and `CONTEXT_DID` if set on the context) server-side, fills every placeholder, and returns the rendered document.

Context-scoped templates are visible and renderable within a single context. This contrasts with the **global** family, which is visible to every context on the VTA. To render a global template instead, use [`vta/did-templates/render`](../../../../did-templates/render/1.0/spec.md).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the context member) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/contexts/did-templates/render/1.0`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.contextId` with the context the template is scoped to, and `payload.name` with the resource id of the template to render.
3. Supply the template's declared `requiredVars` in `payload.vars`; MUST NOT supply reserved ambient names (the VTA injects those server-side).
4. Include a `proof` member per [SPEC.md §4.7](../../../../../../SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the producer lacks access to `payload.contextId`, respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../../SPEC.md#83-standard-error-codes)).
3. Where no template with `payload.name` exists in `payload.contextId`, respond with `vta/contexts/did-templates/render:notFound`.
4. Where `payload.vars` omits a declared `requiredVar`, includes an undeclared variable, or attempts to set a reserved ambient name, respond with `malformedRequest`.
5. On success, inject the ambient variables, fill every `{TOKEN}` placeholder, and return the rendered DID document in the [response](#response).

## Definitions

* **Context member.** The party invoking the task; identified by `issuer`. Holds access to `payload.contextId` on the VTA.
* **VTA.** The Verifiable Trust Agent that stores and renders templates; identified by `recipient`.
* **DidTemplate.** The authored template shape — see `#/$defs/DidTemplate` in the create spec's `payload.schema.json`.
* **Ambient variables.** Values the VTA injects server-side at render time: `VTA_DID`, `VTA_URL`, `NOW`, `CONTEXT_ID`, and `CONTEXT_DID` (if set on the context). Callers MUST NOT supply these.

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/contexts/did-templates/render/1.0` with a payload that validates against the top-level schema in `payload.schema.json`.

### Render a context-scoped template

```json
{
  "id": "60718293-a4b5-76c7-d8e9-f0a1b2c3d4e5",
  "type": "https://trusttasks.org/spec/vta/contexts/did-templates/render/1.0",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-06-16T09:15:00Z",
  "payload": {
    "contextId": "primary",
    "name": "messaging-bridge",
    "vars": {
      "MEDIATOR_DID": "did:web:mediator.example"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-06-16T09:15:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/contexts/did-templates/render/1.0#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload is the rendered DID document.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../../SPEC.md#8-error-responses)), not the `#response` variant — including the `vta/contexts/did-templates/render:notFound` error.

### The rendered document

Response to the request example. The VTA has minted the signing key and filled the ambient and caller-supplied variables:

```json
{
  "id": "71829304-b5c6-77d8-e9f0-a1b2c3d4e5f6",
  "type": "https://trusttasks.org/spec/vta/contexts/did-templates/render/1.0#response",
  "threadId": "60718293-a4b5-76c7-d8e9-f0a1b2c3d4e5",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-16T09:15:01Z",
  "payload": {
    "document": {
      "id": "did:web:vta.example:templates:messaging-bridge",
      "verificationMethod": [
        {
          "id": "did:web:vta.example:templates:messaging-bridge#z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
          "type": "Multikey",
          "controller": "did:web:vta.example:templates:messaging-bridge",
          "publicKeyMultibase": "z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH"
        }
      ],
      "service": [
        {
          "id": "did:web:vta.example:templates:messaging-bridge#didcomm",
          "type": "DIDCommMessaging",
          "serviceEndpoint": { "uri": "did:web:mediator.example", "accept": ["didcomm/v2"] }
        }
      ]
    }
  }
}
```

## Security & Privacy

**Reads are scoped and audited.** Rendering reads context-scoped configuration and binds caller-supplied variables, so the task is restricted to members of `payload.contextId`. The **REQUIRED** `proof` binds the render to a specific caller for the audit trail and prevents a captured request being attributed to the wrong party.

**Ambient variables are server-controlled.** The VTA injects `VTA_DID`, `VTA_URL`, `NOW`, `CONTEXT_ID`, and `CONTEXT_DID` itself and rejects any attempt by the caller to set them, so a caller cannot spoof the VTA's identity, the context binding, or the render timestamp through `payload.vars`.

**Render output is a shape, not a secret.** The rendered document contains only public DID-document structure — the VTA mints all key material at render time and never emits private keys in the response. Even so, the optional `ext` extension (see [SPEC.md §4.5.1](../../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload, so producers **MUST NOT** place data in `ext` they would not be comfortable signing.
