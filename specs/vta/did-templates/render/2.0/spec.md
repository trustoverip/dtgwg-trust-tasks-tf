---
slug: vta/did-templates/render
version: "2.0"
title: VTA DID-Template — Render
summary: An authenticated caller renders a DID template to a DID document, supplying variables — global when contextId is absent, context-scoped when present.
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
  - role: Authenticated caller
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Rendering is a pure function over a stored template and caller-supplied variables — it persists nothing, discloses no secret (the output is a DID document shape with placeholders substituted; keys are minted only by the separate provisioning flow), and exercises no subject authority. The transport-level session that authenticates the caller (plus context access when contextId is present) is sufficient to authorize it; a producer proof adds a transport-independent identity for the VTA's audit trail, so it is RECOMMENDED rather than REQUIRED. The blanket REQUIRED of 1.0 was re-derived per task for 2.0 — mutations keep REQUIRED, pure reads do not.
sideEffects:
  level: none
  rationale: "Renders a DID template to a DID document; produces output, persists nothing."
subjectPath: /name
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vta/did-templates/render:notFound
    meaning: No template with this name in the selected scope.
    retryable: false
related:
  - vta/did-templates/create
  - vta/did-templates/get
  - vta/did-templates/update
  - vta/did-templates/delete
  - vta/did-templates/list
---

## Abstract

The **VTA DID-Template — Render** Trust Task renders a stored DID template to a DID document on a Verifiable Trust Agent. The caller supplies variables; the VTA injects **ambient variables** server-side, merges the caller's on top, substitutes every `{TOKEN}` placeholder, and returns the rendered document. Rendering persists nothing — it is a dry-run of what provisioning would mint.

The scope is selected by the **optional `contextId`** field:

* **Absent** — render a **global** template. Ambient variables: `VTA_DID`, `VTA_URL`, `NOW`.
* **Present** — render a template scoped to that **context** (the VTA MAY fall back to a global template of the same name, per its scope-fallback rule). The caller must have access to the context. Ambient variables additionally include **`CONTEXT_ID`** (the context's id) and — if a DID is set on the context — **`CONTEXT_DID`**.

Version 2.0 merges the 1.0 pair [`vta/did-templates/render/1.0`](../1.0/spec.md) and [`vta/contexts/did-templates/render/1.0`](../../../contexts/did-templates/render/1.0/spec.md) into this single task; the per-scope authorization moved from the slug structure to the `contextId` field.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the authenticated caller) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/did-templates/render/2.0`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.name` with the template to render, and `payload.contextId` with the target context — or omit `contextId` to render a global template.
3. Supply every variable the template's `requiredVars` declares in `payload.vars`, and **MUST NOT** supply reserved ambient names (`DID`, `SIGNING_KEY_MB`, `KA_KEY_MB`, `VTA_DID`, `VTA_URL`, `CONTEXT_ID`, `CONTEXT_DID`, `NOW`).

A conforming producer **SHOULD** include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof) so the render is attributable independent of transport.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof` when present.
2. Where the producer is not an authenticated caller — or, when `payload.contextId` is present, lacks access to that context — respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../SPEC.md#83-standard-error-codes)).
3. Where no template named `payload.name` exists in the selected scope (after any global fallback the VTA applies for context renders), respond with `vta/did-templates/render:notFound`.
4. Where `payload.vars` is missing a required variable or names a reserved ambient variable, respond with `malformedRequest`.
5. Inject the ambient variables server-side — `VTA_DID`, `VTA_URL`, `NOW`, plus `CONTEXT_ID` and (if set on the context) `CONTEXT_DID` when `payload.contextId` is present — merge the caller's `vars` on top, substitute every placeholder, and return the [rendered document](#response).

## Definitions

* **Authenticated caller.** The party invoking the task; identified by `issuer`. Any caller the VTA can authenticate; access to `payload.contextId` is additionally required when it is present.
* **VTA.** The Verifiable Trust Agent that stores and renders templates; identified by `recipient`.
* **Ambient variables.** Values the VTA injects server-side and callers can never override: `VTA_DID`, `VTA_URL`, `NOW` always; `CONTEXT_ID` and `CONTEXT_DID` for context-scoped renders.

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/did-templates/render/2.0` with a payload that validates against the top-level schema in `payload.schema.json`.

### Render a context-scoped template

```json
{
  "id": "6071829a-a4b5-c6d7-e8f9-a0b1c2d3e4f5",
  "type": "https://trusttasks.org/spec/vta/did-templates/render/2.0",
  "issuer": "did:web:operator.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-07-29T09:00:00Z",
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
    "verificationMethod": "did:web:operator.example#key-1",
    "created": "2026-07-29T09:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

Omit `contextId` to render the global template of the same name; `CONTEXT_ID`/`CONTEXT_DID` are then not injected and a template referencing them fails with `malformedRequest`.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/did-templates/render/2.0#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response payload carries the rendered `document`.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant — including `vta/did-templates/render:notFound`.

### The rendered document

```json
{
  "id": "718293a4-b5c6-d7e8-f9a0-b1c2d3e4f5a6",
  "type": "https://trusttasks.org/spec/vta/did-templates/render/2.0#response",
  "threadId": "6071829a-a4b5-c6d7-e8f9-a0b1c2d3e4f5",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:operator.example",
  "issuedAt": "2026-07-29T09:00:01Z",
  "payload": {
    "document": {
      "id": "did:web:primary.vta.example",
      "service": [
        {
          "id": "did:web:primary.vta.example#didcomm",
          "type": "DIDCommMessaging",
          "serviceEndpoint": {
            "uri": "did:web:mediator.example",
            "accept": ["didcomm/v2"]
          }
        }
      ]
    }
  }
}
```

## Security & Privacy

**Scope selection is authorization, not filtering.** The `contextId` field selects which scope is rendered, and the VTA authorizes against it server-side; a caller without access to the context gets `permissionDenied`, never a rendered document.

**Ambient variables are server-authoritative.** `VTA_DID`, `VTA_URL`, `NOW`, `CONTEXT_ID`, and `CONTEXT_DID` are injected by the VTA and cannot be supplied or overridden by the caller — a template cannot be tricked into rendering another context's identity into a document.

**Pure read, proportionate proof.** Rendering persists nothing and returns only the substituted document shape — key material is minted by the separate provisioning flow, never by render. The producer `proof` is therefore **RECOMMENDED** rather than REQUIRED: the transport session already authorizes the render, and the proof's value here is a transport-independent identity for the VTA's audit trail. Deployments that require attributable renders SHOULD reject unproven requests as a matter of local policy.
