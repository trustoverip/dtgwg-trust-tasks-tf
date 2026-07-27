---
slug: governance/capability/list
version: "0.1"
title: Governance — List Capabilities
summary: A community member or surface queries which pluggable capabilities a community has enabled (or has available), with their manifests, to render management and status views.
status: draft
targetFrameworkVersion: "0.1"
category: governance
keywords:
  - governance
  - capability
  - plugin
  - list
  - community
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Requester
    requirement: REQUIRED
    member: issuer
  - role: Community governance host
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: A read whose integrity is normally guaranteed by the transport; RECOMMENDED rather than REQUIRED so the query remains usable on bindings without an in-band verifier.
sideEffects:
  level: none
  rationale: "Read-only listing."
exposure:
  discloses: metadata
  actsAsSubject: false
related:
  - governance/capability/enable
  - governance/capability/disable
  - trust-task-discovery
---

## Abstract

The **Governance — List Capabilities** Trust Task is the query behind a
community's capability-management surface: which pluggable capabilities are
enabled, since when, under which manifest, and (for companion capabilities)
which DID serves them. `status: "available"` additionally lets a management
UX offer what *could* be enabled.

This complements `trust-task-discovery`: discovery advertises the flat set
of Trust Task types a host currently serves, while this task returns the
capability-level view (manifests, enablement state) that management surfaces
render.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Definitions

Each returned entry's `manifest` follows the shared
[`CapabilityManifest`](../../../_shared/0.1/governance.schema.json) definition.

## Request

The *requester* (document `issuer`) sends the query to the *community
governance host* (document `recipient`). `status` is OPTIONAL; **absence
means `enabled`** — the most restrictive listing.

### Example request

```json
{
  "status": "enabled"
}
```

## Response

The *community governance host* (now the `issuer` of the response) replies
with the `#response` document reachable via `$anchor: "response"` in
`payload.schema.json`: the `capabilities` array (REQUIRED, possibly empty),
each entry carrying `manifest` and `enabled`, with optional `enabledAt` and
`delegate`. Failures use `trust-task-error`, not a `#response` document.

### Example response

```json
{
  "capabilities": [
    {
      "manifest": {
        "capability": "git-trust",
        "version": "0.1",
        "title": "Git Commit Trust",
        "specs": ["git-trust/*"],
        "vocabulary": {
          "actions": ["git.commit.sign"],
          "resourcePattern": "<org>[/<repo>]"
        },
        "lifecycleHooks": ["member.enrolled", "member.revoked", "role.changed"],
        "externalAdapters": [
          { "kind": "github-action", "ref": "OpenVTC/openvtc/.github/actions/verify-trust" }
        ]
      },
      "enabled": true,
      "enabledAt": "2026-07-17T00:00:00Z"
    }
  ]
}
```

## Security & Privacy

The capability list reveals a community's governance posture; hosts MAY
restrict `available`/`all` listings to community members or operators while
leaving `enabled` open, per their read policy.
