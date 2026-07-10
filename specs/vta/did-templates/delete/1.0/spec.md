---
slug: vta/did-templates/delete
version: "1.0"
title: VTA DID-Template — Delete
summary: A super-administrator removes a global DID template by name.
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - did-template
  - did
  - provisioning
  - delete
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
  rationale: Removing a template is a privileged, audited write to a VTA's provisioning surface. The VTA MUST attribute the change to a specific super-administrator for the audit record, so transport-independent producer identity is required.
sideEffects:
  level: mutating
  rationale: "Removes a global DID template; re-creatable config, not an issued identity."
subjectPath: /name
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vta/did-templates/delete:notFound
    meaning: No global template with this name.
    retryable: false
related:
  - vta/did-templates/create
  - vta/did-templates/get
  - vta/did-templates/update
  - vta/did-templates/list
  - vta/did-templates/render
  - vta/contexts/did-templates/delete
---

## Abstract

The **VTA DID-Template — Delete** Trust Task removes a **global** DID template from a Verifiable Trust Agent by name. A *DID template* is the JSON shape of a DID document with `{TOKEN}` placeholders plus the variable contract the VTA's renderer enforces; the VTA fills the placeholders with keys it mints and caller-supplied variables when provisioning an integration. A *super-administrator* supplies the template's `name`; the VTA refuses the request if no template with that name exists, otherwise removes it and returns a confirmation that echoes the deleted name.

Deletion is permanent and affects every context: once removed, no caller can render from the name until a new template is created under it. To remove a template scoped to a single context — managed by that context's admin — use [`vta/contexts/did-templates/delete`](../../../contexts/did-templates/delete/1.0/spec.md).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the super-administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vta/did-templates/delete/1.0`, with itself as `issuer` and the VTA as `recipient`.
2. Set `payload.name` to the name of the global template to remove.
3. Include a `proof` member per [SPEC.md §4.7](../../../../../SPEC.md#47-proof).

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC.md §7.2](../../../../../SPEC.md#72-consumer-requirements) and verify the `proof`.
2. Where the producer does not hold the super-administrator role, respond with the framework's `permissionDenied` ([SPEC.md §8.3](../../../../../SPEC.md#83-standard-error-codes)).
3. Where no global template with `payload.name` exists, respond with `vta/did-templates/delete:notFound`.
4. On success, remove the template from the global scope and return the confirmation [response](#response) with `deleted: true`.

## Definitions

* **Super-administrator.** The party invoking the task; identified by `issuer`. Holds the unrestricted Admin role on the VTA.
* **VTA.** The Verifiable Trust Agent that stores and renders templates; identified by `recipient`.
* **DidTemplate.** The authored template shape — see `#/$defs/DidTemplate` in the create spec's `payload.schema.json`.

## Request

A *request* document carries `type: https://trusttasks.org/spec/vta/did-templates/delete/1.0` with a payload that validates against the top-level schema in `payload.schema.json`.

### Delete a global template

```json
{
  "id": "4e5f6071-8293-a4b5-c6d7-e8f9a0b1c2d3",
  "type": "https://trusttasks.org/spec/vta/did-templates/delete/1.0",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-06-16T11:00:00Z",
  "payload": {
    "name": "messaging-bridge"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-rdfc-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-06-16T11:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3kg..."
  }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/vta/did-templates/delete/1.0#response`, with a payload that validates against the `$anchor: "response"` sub-schema in `payload.schema.json`. The response echoes the deleted template's `name` and sets `deleted: true`.

Failures use `trust-task-error` ([SPEC.md §8](../../../../../SPEC.md#8-error-responses)), not the `#response` variant — including the `vta/did-templates/delete:notFound` case.

### Deletion confirmed

Response to the request example:

```json
{
  "id": "5f607182-93a4-b5c6-d7e8-f9a0b1c2d3e4",
  "type": "https://trusttasks.org/spec/vta/did-templates/delete/1.0#response",
  "threadId": "4e5f6071-8293-a4b5-c6d7-e8f9a0b1c2d3",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-06-16T11:00:01Z",
  "payload": {
    "name": "messaging-bridge",
    "deleted": true
  }
}
```

## Security & Privacy

**Privileged write, audited.** Removing a template changes what the VTA can mint for future integrations, so the task is restricted to super-administrators and the VTA records the deleter DID. The **REQUIRED** `proof` binds the change to a specific operator for the audit trail and prevents a captured request being attributed to the wrong party. The response echoes the deleted `name` so downstream audit pipelines can record exactly which resource was removed without re-deriving it from the request.

**Deletion is irreversible.** There is no soft-delete or trash; once removed, a name can only be reinstated via [`vta/did-templates/create`](../create/1.0/spec.md). Any data in the optional `ext` extension (see [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member)) is signed alongside the rest of the payload, so producers **MUST NOT** place data in `ext` they would not be comfortable signing.
