---
slug: config/show
version: "0.1"
title: Config — Show
summary: Read the effective value of a maintainer's runtime configuration keys, each with the layer it came from and whether changing it needs a restart.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - config
  - runtime
  - administration
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
  - role: config maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Read-only view of runtime configuration. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads the effective configuration; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: config/show:permissionDenied
    meaning: The consumer lacks the configuration-admin capability.
    retryable: false
---

## Abstract

The **Config — Show** Trust Task returns the effective value of a maintainer's runtime configuration keys — the values actually in force — as [`ConfigField`](../_shared/0.1/config.schema.json)s. Each field names its `source` (which layer supplied the value, for a maintainer with a layered overlay) and `requiresRestart` (whether changing it takes effect immediately or on restart). It is the read half of the pair completed by [`config/patch`](../patch/0.1/).

With no `keys`, it returns every registered key; with `keys`, only those.

## Conformance

Producer: optionally supply `keys` to narrow the result.

Consumer: verify the configuration-admin capability. Return a `ConfigField` per requested key (or every registered key). Resolve `value` through whatever layering the maintainer applies and set `source` to the winning layer — an opaque label the maintainer defines, not a fixed enum. Set `requiresRestart` from the key's reload semantics. Redact the value of any secret-bearing key (see below) rather than returning it.

## Security & Privacy

**Configuration is admin-class.** The set and values of runtime keys describe the maintainer's operational posture, so this task is gated on the configuration-admin capability. `exposure.discloses` is `metadata`: the general case returns operational settings (ports, log levels, feature flags), not secret material.

**Secret-bearing keys.** Some keys reference or hold sensitive values — a storage path, a TLS private-key path, a token. A maintainer MUST redact the `value` of such a key (return a placeholder, or omit the field) rather than disclose it through a config read; the `metadata` classification holds only because secret values are redacted, not returned. This mirrors the redaction a maintainer already applies when the key is written (`config/patch`) and audited.
