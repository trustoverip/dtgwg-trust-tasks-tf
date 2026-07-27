---
slug: governance/capability/enable
version: "0.1"
title: Governance — Enable Capability
summary: A community operator enables a pluggable capability (its Trust Task handlers, registry vocabulary, lifecycle hooks, and management surfaces) for a community.
status: draft
targetFrameworkVersion: "0.1"
category: governance
keywords:
  - governance
  - capability
  - plugin
  - enable
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
  rationale: Enabling a capability changes what the community's registry answers for and which handlers execute on its behalf — an evidentiary, state-changing governance act that may be audited after the transport has closed.
sideEffects:
  level: mutating
  rationale: "Activates handlers, discovery advertisement, and lifecycle hooks for the capability; reversible via governance/capability/disable."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: governance/capability/enable:unknown_capability
    meaning: The host does not know the named capability/version (not built in, and no manifest supplied).
    retryable: false
  - code: governance/capability/enable:already_enabled
    meaning: The capability is already enabled for this community.
    retryable: false
  - code: governance/capability/enable:config_invalid
    meaning: The supplied config does not validate against the capability manifest's configSchema.
    retryable: false
related:
  - governance/capability/disable
  - governance/capability/list
---

## Abstract

The **Governance — Enable Capability** Trust Task turns on a pluggable
*capability module* for a community. A capability is a named bundle —
described by a [`CapabilityManifest`](../../../_shared/0.1/governance.schema.json) —
of Trust Task spec families, trust-registry vocabulary, membership lifecycle
hooks, management surfaces, and external adapters. Communities differ in
which capabilities they need (an open-source community may enable
`git-trust`; another community never will), so **everything is off by
default**: a community with no enable record carries none of a capability's
records, handlers, or attack surface.

Enabling has three effects at the *community governance host* (the
recipient): the capability's Trust Task handlers are activated for this
community, the capability is advertised via `trust-task-discovery`, and its
declared membership lifecycle hooks begin firing.

Capabilities come in two packagings with one contract: **built-in** (the
host compiles the capability and knows its manifest) and **companion** (a
separate service with its own DID serves the capability's namespace). For a
companion, the request carries `delegate` (the companion's DID) and MUST
carry the `manifest`; the host records a namespace delegation and MUST
enforce that the delegate only writes registry tuples whose `action` appears
in the manifest's `vocabulary.actions`.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

## Definitions

The `manifest` member follows the shared
[`CapabilityManifest`](../../../_shared/0.1/governance.schema.json) definition.
The `config` member is opaque to the framework: the recipient MUST validate
it against the schema named by the manifest's `configSchema` and reject
`config_invalid` on mismatch. Absent `config` means the capability's most
restrictive defaults (never its most permissive).

## Request

The *community operator* (document `issuer`) sends the request to the
*community governance host* (document `recipient`), naming the `capability`
and `version` (both REQUIRED), with optional `config`, and — for companion
capabilities — `delegate` plus the then-REQUIRED `manifest`.

### Example request

```json
{
  "capability": "git-trust",
  "version": "0.1",
  "config": {
    "grantOnRole": {
      "maintainer": { "resource": "openvtc" }
    },
    "revokeWithMembership": true
  }
}
```

## Response

The *community governance host* (now the `issuer` of the response) replies
with the `#response` document reachable via `$anchor: "response"` in
`payload.schema.json`, echoing `capability` and reporting `enabled`
(REQUIRED), with optional `version`, `enabledAt`, and `message`. Failures
use `trust-task-error`, not a `#response` document.

### Example response

```json
{
  "capability": "git-trust",
  "version": "0.1",
  "enabled": true,
  "enabledAt": "2026-07-17T00:00:00Z"
}
```

## Security & Privacy

Enabling a capability is a high-consequence governance act: it changes which
handlers execute under the community's authority and what its registry
answers for. Hosts SHOULD classify it as requiring elevated (e.g.
`destructive`-class) operator consent under their delegated-execution
policy, and MUST restrict it to the community's operator role. For companion
capabilities, the delegation boundary is the manifest's declared vocabulary:
the host MUST reject registry writes from the delegate outside
`vocabulary.actions`, and SHOULD treat the delegate DID's key rotation or
deactivation as suspending the delegation.
