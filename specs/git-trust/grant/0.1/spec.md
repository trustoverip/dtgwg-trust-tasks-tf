---
slug: git-trust/grant
version: "0.1"
title: Git Trust — Grant Commit-Signing Trust
summary: A community operator grants a member's DID the authority to sign git commits for an org or repository; the host records it as a TRQP authorization tuple CI verifiers query.
status: draft
targetFrameworkVersion: "0.1"
category: governance
keywords:
  - git
  - commit-signing
  - capability
  - grant
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
  rationale: A grant changes what the community's registry answers to CI enforcement — an evidentiary, state-changing governance act.
sideEffects:
  level: mutating
  rationale: "Records a TRQP authorization tuple; reversible via git-trust/revoke."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: git-trust/grant:already_granted
    meaning: An active grant already exists for this subject and resource.
    retryable: false
related:
  - git-trust/revoke
  - registry/authorization
  - governance/capability/enable
---

## Abstract

The **Git Trust — Grant** Trust Task is the enrollment operation of the
`git-trust` capability (a pluggable community capability enabled via
[`governance/capability/enable`](../../../governance/capability/enable/0.1/spec.md)):
a *community operator* grants a member's DID (`subject`) the authority to
sign git commits for a `resource` — an org (`<org>`, org-wide) or a
repository slug (`<org>/<repo>`).

The *community governance host* records the grant in the community's trust
registry as the TRQP authorization tuple `{entity_id: subject, authority_id:
the community's authority DID (from the capability's config), action:
"git.commit.sign", resource}`. CI verifiers (e.g. `did-git-sign
verify-trust`) query that tuple anonymously via
[`registry/authorization`](../../../registry/authorization/0.1/spec.md);
repo-scoped queries may fall back to the org-scoped tuple per the verifier's
configuration.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Request

The *community operator* (document `issuer`) sends the grant to the
*community governance host* (document `recipient`) with `subject` (a DID)
and `resource` (both REQUIRED).

### Example request

```json
{
  "subject": "did:webvh:scid123:signer.example.com",
  "resource": "openvtc/openvtc"
}
```

## Response

The *community governance host* (now the `issuer` of the response) replies
with the `#response` document reachable via `$anchor: "response"` in
`payload.schema.json`, echoing `subject` and `resource` and reporting
`granted` (REQUIRED). `granted: true` MUST NOT be sent before the tuple is
durably recorded. Failures use `trust-task-error`, not a `#response`
document.

### Example response

```json
{
  "subject": "did:webvh:scid123:signer.example.com",
  "resource": "openvtc/openvtc",
  "granted": true
}
```

## Security & Privacy

Grants gate CI enforcement, so hosts MUST require the capability's declared
operator role and the framework's REQUIRED proof, and SHOULD write an audit
record. The tuple itself becomes anonymously readable via TRQP — do not
encode private information in `resource` values.
