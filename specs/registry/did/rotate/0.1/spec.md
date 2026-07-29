---
slug: registry/did/rotate
version: "0.1"
title: Registry — Rotate DID Keys
summary: An administrator rotates the keys of a trust registry's own agent-managed did:webvh in place, preserving the registry's DID while refreshing its key material.
status: draft
targetFrameworkVersion: "0.2"
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

The **Registry — Rotate DID Keys** Trust Task instructs a trust registry to rotate the key material of its **own** DID — an agent-managed `did:webvh` — in place. The DID itself is preserved; the registry's managing agent appends a new entry to the webvh log with fresh keys (and, optionally, a caller-specified number of pre-rotation commitments) and reports the resulting `new_scid` and `new_version_id`.

Unlike the `registry/record/*` family, this task acts on the registry's identity rather than on its record store. It is defined here to document a pre-existing deployed wire contract; the payload field names are therefore frozen snake_case, matching both that wire form and the wider `registry/*` family convention.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Definitions

`pre_rotation_count` optionally overrides how many pre-rotation key commitments the managing agent publishes for the new key set; absent, the agent's default applies. `label` is an operator-facing audit label recorded with the rotation.

## Request

The *registry administrator* (document `issuer`) sends the rotation instruction to the *trust registry* (document `recipient`). A `proof` is REQUIRED. Both payload members are OPTIONAL; the empty payload `{}` requests a default rotation.

### Example request

```json
{
  "pre_rotation_count": 2,
  "label": "quarterly rotation 2026-Q3"
}
```

## Response

The *trust registry* replies with the `#response` document (`$anchor: "response"`) identifying the rotated DID and its post-rotation `new_scid` and `new_version_id`. A registry deployed without a managing agent capable of rotation rejects with `trust-task-error` (framework `taskFailed`); no task-specific error codes are defined.

### Example response

```json
{
  "did": "did:webvh:Qma6mc1q…:registry.example",
  "new_scid": "Qmb7nd2r…",
  "new_version_id": "5-Qmc8oe3s…"
}
```

## Security & Privacy

This is the highest-privilege task a registry exposes: whoever can rotate the registry's keys can eventually impersonate the registry. The registry MUST verify the `proof` and restrict the task to its administrator ACL, and SHOULD alert out-of-band on every rotation. The response discloses only public log metadata (`did`, SCID, versionId) — private key material never appears in the payload in either direction. Consumers of the registry's DID SHOULD tolerate rotation by resolving `did:webvh` freshly rather than pinning keys.
