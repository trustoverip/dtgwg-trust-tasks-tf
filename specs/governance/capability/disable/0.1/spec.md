---
slug: governance/capability/disable
version: "0.1"
title: Governance — Disable Capability
summary: A community operator disables a previously enabled capability; its handlers stop, it leaves discovery, and its lifecycle hooks stop firing, while its registry records are retained for audit.
status: draft
targetFrameworkVersion: "0.1"
category: governance
keywords:
  - governance
  - capability
  - plugin
  - disable
  - community
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
  rationale: Disabling a capability is a state-changing governance act with the same audit requirements as enabling one.
sideEffects:
  level: mutating
  rationale: "Deactivates handlers, discovery advertisement, and lifecycle hooks; the capability's registry records are retained (auditable), not deleted."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: governance/capability/disable:not_enabled
    meaning: The named capability is not currently enabled for this community.
    retryable: false
related:
  - governance/capability/enable
  - governance/capability/list
---

## Abstract

The **Governance — Disable Capability** Trust Task turns a capability off
for a community. **Disable is not delete**: the capability's records in the
community's trust registry are retained so past decisions stay auditable,
but the capability's Trust Task handlers stop executing for this community,
it is no longer advertised via `trust-task-discovery`, and its membership
lifecycle hooks stop firing. External adapters that consult the community's
registry will observe queries under the capability's vocabulary answering
negatively or the capability absent from discovery, per the host's policy.

Re-enabling later with [`governance/capability/enable`](../../enable/0.1/spec.md)
resumes service over the retained records.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Request

The *community operator* (document `issuer`) sends the request to the
*community governance host* (document `recipient`), naming the `capability`
(REQUIRED) with an optional audit `reason`.

### Example request

```json
{
  "capability": "git-trust",
  "reason": "Community vote 2026-07: pausing enforcement during the migration."
}
```

## Response

The *community governance host* (now the `issuer` of the response) replies
with the `#response` document reachable via `$anchor: "response"` in
`payload.schema.json`, echoing `capability` and reporting `enabled: false`.
Failures use `trust-task-error`, not a `#response` document.

### Example response

```json
{
  "capability": "git-trust",
  "enabled": false
}
```

## Security & Privacy

Disabling can itself be disruptive (external adapters relying on the
capability start failing closed), so hosts SHOULD require the same operator
role and consent class as enabling. The retained records remain subject to
the registry's own read policy.
