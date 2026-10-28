---
slug: registry/did/rotate
version: "0.2"
title: Registry — Rotate DID Keys
summary: An administrator rotates the keys of a trust registry's own agent-managed did:webvh in place, preserving the registry's DID while refreshing its key material.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - trust-registry
  - did-rotation
  - webvh
  - key-rotation
  - admin
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Registry administrator
    requirement: REQUIRED
    member: issuer
  - role: Trust registry
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Rotating the registry's identity keys is the most consequential administrative act the registry accepts; the instruction must be attributable and verifiable independently of the transport, and auditable after the fact.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Rotating the registry's own DID keys retires the previous key irreversibly. A replayed rotation retires the key the registry is currently signing with, so this is the family's availability risk as much as its integrity one.
sideEffects:
  level: destructive
  rationale: Advances the registry's did:webvh log irreversibly and supersedes the current signing keys — material signed under the old keys can no longer be freshly produced, and the rotation cannot be undone, only followed by another rotation.
exposure:
  discloses: metadata
  actsAsSubject: false
related:
  - registry/record/put
  - registry/record/query
  - registry/record/delete
---

## Abstract

The **Registry — Rotate DID Keys** Trust Task instructs a trust registry to rotate the key material of its **own** DID — an agent-managed `did:webvh` — in place. The DID itself is preserved; the registry's managing agent appends a new entry to the webvh log with fresh keys (and, optionally, a caller-specified number of pre-rotation commitments) and reports the resulting `newScid` and `newVersionId`.

Unlike the `registry/record/*` family, this task acts on the registry's identity rather than on its record store. Its members are lowerCamelCase, as throughout the `registry/*` family.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Changes from 0.1

- Members are renamed to lowerCamelCase, as throughout the `registry/*` family: `pre_rotation_count` → `preRotationCount` in the request, and `new_scid` → `newScid`, `new_version_id` → `newVersionId` in the response. Types, requiredness and meaning are unchanged. 0.1 froze snake_case to document an already-deployed wire form; that form moves with the rest of the family.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Definitions

`preRotationCount` optionally overrides how many pre-rotation key commitments the managing agent publishes for the new key set; absent, the agent's default applies. `label` is an operator-facing audit label recorded with the rotation.

## Request

The *registry administrator* (document `issuer`) sends the rotation instruction to the *trust registry* (document `recipient`). A `proof` is REQUIRED. Both payload members are OPTIONAL; the empty payload `{}` requests a default rotation.

### Example request

```json
{
  "preRotationCount": 2,
  "label": "quarterly rotation 2026-Q3"
}
```

## Response

The *trust registry* replies with the `#response` document (`$anchor: "response"`) identifying the rotated DID and its post-rotation `newScid` and `newVersionId`. A registry deployed without a managing agent capable of rotation rejects with `trust-task-error` (framework `taskFailed`); no task-specific error codes are defined.

### Example response

```json
{
  "did": "did:webvh:Qma6mc1q…:registry.example",
  "newScid": "Qmb7nd2r…",
  "newVersionId": "5-Qmc8oe3s…"
}
```

## Security & Privacy

### Data carried

The request carries the optional `preRotationCount` and an operator-facing `label`. The response carries public did:webvh log metadata only: the `did`, its `newScid` and its `newVersionId`. Private key material never appears in the payload in either direction.

### Correlation

The task concerns the registry's own public identity, which every consumer already knows. The document's `issuer` and its `proof` tie the rotation to the administrator who ordered it; `threadId` correlates request and response.

### Retention

The registry SHOULD record every rotation, with its `label` and the administrator, in its audit trail, and SHOULD alert out-of-band on each one. The did:webvh log itself keeps the rotation permanently.

### Consent/purpose

This is the highest-privilege task a registry exposes: whoever can rotate the registry's keys can eventually impersonate the registry. The registry MUST verify the `proof` and restrict the task to its administrator ACL. Consumers of the registry's DID SHOULD tolerate rotation by resolving `did:webvh` freshly rather than pinning keys.
