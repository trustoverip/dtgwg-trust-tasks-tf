---
slug: git-trust/revoke
version: "0.1"
title: Git Trust — Revoke Commit-Signing Trust
summary: A community operator revokes a member's commit-signing grant; the host marks the TRQP tuple unauthorized (retained for audit) so CI verifiers observe the denial on their next query.
status: draft
targetFrameworkVersion: "0.1"
category: governance
keywords:
  - git
  - commit-signing
  - capability
  - revoke
  - trust-registry
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Community operator
    requirement: REQUIRED
    member: issuer
  - role: Community governance host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Revocation is delivery- and audit-critical; it changes what CI enforcement answers and must carry transport-independent integrity.
sideEffects:
  level: mutating
  rationale: "Marks the grant tuple unauthorized; the record is retained for audit, not deleted."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: git-trust/revoke:not_granted
    meaning: No active grant exists for this subject and resource.
    retryable: false
related:
  - git-trust/grant
  - registry/authorization
---

## Abstract

The **Git Trust — Revoke** Trust Task withdraws a member's commit-signing
grant. The *community governance host* marks the grant's TRQP tuple
`authorized: false` rather than deleting it, so the revocation is itself
auditable; a CI verifier's next
[`registry/authorization`](../../../registry/authorization/0.1/spec.md)
query for that tuple answers `authorized: false`.

Note the scope semantics: verifiers configured with an org fallback treat a
grant as `repo OR org`, and the TRQP read contract cannot distinguish an
absent record from an explicit `authorized: false`. Revoking a member's
trust entirely therefore means revoking **every** scope they hold a grant
under (typically the org-wide one).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Request

The *community operator* (document `issuer`) sends the revocation to the
*community governance host* (document `recipient`) with `subject` and
`resource` (both REQUIRED) and an optional audit `reason`.

### Example request

```json
{
  "subject": "did:webvh:scid123:signer.example.com",
  "resource": "openvtc/openvtc",
  "reason": "Maintainer role ended 2026-07."
}
```

## Response

The *community governance host* (now the `issuer` of the response) replies
with the `#response` document reachable via `$anchor: "response"` in
`payload.schema.json`, echoing `subject` and `resource` and reporting
`revoked` (REQUIRED). `revoked: true` MUST NOT be sent before the tuple is
durably marked unauthorized. Failures use `trust-task-error`, not a
`#response` document.

### Example response

```json
{
  "subject": "did:webvh:scid123:signer.example.com",
  "resource": "openvtc/openvtc",
  "revoked": true
}
```

## Security & Privacy

Revocation is delivery-critical: hosts MUST apply it before acknowledging
(`revoked: true` after durable state, never before), and SHOULD alert when a
revocation cannot be recorded. The retained tuple is anonymously readable
via TRQP, as with all registry records.
