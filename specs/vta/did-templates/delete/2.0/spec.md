---
slug: vta/did-templates/delete
version: "2.0"
title: VTA DID-Template — Delete
summary: An administrator removes a DID template by name — global (super-admin) when contextId is absent, context-scoped (context admin) when present.
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - did-template
  - did
  - provisioning
  - delete
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
  rationale: Removing a template is a privileged, audited write to a VTA's provisioning surface. The VTA MUST attribute the change to a specific administrator — a super-administrator for the global scope, a context administrator for a context scope — for the audit record, so transport-independent producer identity is required.
sideEffects:
  level: mutating
  rationale: "Removes a DID template from the selected scope; re-creatable config, not an issued identity."
subjectPath: /name
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vta/did-templates/delete:notFound
    meaning: No template with this name in the selected scope.
    retryable: false
related:
  - vta/did-templates/create
  - vta/did-templates/get
  - vta/did-templates/update
  - vta/did-templates/list
  - vta/did-templates/render
---

## Abstract

The **VTA DID-Template — Delete** Trust Task removes a DID template by name from **one scope** on a Verifiable Trust Agent. The success response echoes the deleted name for audit pipelines. Deleting a template does not affect integrations already provisioned from it — a template is re-creatable configuration, not an issued identity.

The scope is selected by the **optional `contextId`** field:

* **Absent** — the **global** scope. Gated on a **super-administrator**.
* **Present** — that **context's** scope. Gated on that **context's administrator** (or a super-administrator).

Version 2.0 merges the 1.0 pair [`vta/did-templates/delete/1.0`](../1.0/spec.md) and [`vta/contexts/did-templates/delete/1.0`](../../../contexts/did-templates/delete/1.0/spec.md) into this single task; the per-scope authorization moved from the slug structure to the `contextId` field.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/did-templates/delete/2.0`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.name` with the template to remove, and `payload.contextId` with the target context — or omit `contextId` to remove a global template.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where `payload.contextId` is absent and the producer is not a super-administrator, respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../SPEC.md#83-standard-error-codes)).
3. Where `payload.contextId` is present and the producer is neither an administrator of that context nor a super-administrator, respond with `permissionDenied`.
4. Where no template named `payload.name` exists in the selected scope, respond with `vta/did-templates/delete:notFound`.
5. On success, remove the template from the selected scope and return the [confirmation](#response).

## Definitions

* **Administrator.** The party invoking the task; identified by `issuer`. A **super-administrator** when `contextId` is absent; an administrator of `payload.contextId` (or a super-administrator) when it is present.
* **VTA.** The Verifiable Trust Agent that stores and renders templates; identified by `recipient`.

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/did-templates/delete/2.0` with a payload that validates against the top-level schema in `payload.schema.json`.

### Remove a context-scoped template

```json
{
  "id": "4e5f6071-8293-a4b5-c6d7-e8f9a0b1c2d3",
  "type": "https://trusttasks.org/spec/vta/did-templates/delete/2.0",
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

Omit `contextId` to remove the global template of the same name instead (super-admin gated).

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/did-templates/delete/2.0#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant — including `vta/did-templates/delete:notFound`.

### Deletion confirmed

```json
{
  "id": "5f607182-93a4-b5c6-d7e8-f9a0b1c2d3e4",
  "type": "https://trusttasks.org/spec/vta/did-templates/delete/2.0#response",
  "threadId": "4e5f6071-8293-a4b5-c6d7-e8f9a0b1c2d3",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-07-29T09:00:01Z",
  "payload": {
    "name": "messaging-bridge",
    "deleted": true
  }
}
```

## Security & Privacy

**Privileged write, audited.** Removing a template changes what the VTA can mint for future integrations in the selected scope, so the task is gated per scope — super-administrators for the global scope, the context's administrators (or super-administrators) for a context scope. The **REQUIRED** `proof` binds the removal to a specific operator for the audit trail and prevents a captured request being attributed to the wrong party.

**Scope is enforced server-side.** The VTA authorizes the write against `payload.contextId` (or its absence); a context admin cannot remove templates outside the contexts they administer, and cannot reach the global scope at all.

**Deletion is shallow.** Removing a template does not deactivate DIDs or integrations already provisioned from it; it only stops future renders. This is why the side-effect level is `mutating` rather than `destructive` — the removed configuration is re-creatable from source control or a repeated create.
