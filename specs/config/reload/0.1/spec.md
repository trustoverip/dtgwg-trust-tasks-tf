---
slug: config/reload
version: "0.1"
title: Config — Reload
summary: Re-apply hot-reloadable configuration to the running maintainer without a restart, reporting which keys changed live.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - config
  - runtime
  - hot-reload
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
  requirement: REQUIRED
  rationale: A reload changes the maintainer's running behaviour by making stored config take effect. It MUST be attributable — who reloaded, and when — so the change is non-repudiable in the audit trail.
sideEffects:
  level: mutating
  rationale: "Updates the running in-memory configuration for hot-reloadable keys; the underlying stored values are unchanged, so it is idempotent."
consequences:
  - "Changes runtime behaviour for every hot-reloadable key whose stored value differed from the live value."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: config/reload:permissionDenied
    meaning: The consumer lacks the configuration-admin capability.
    retryable: false
---

## Abstract

The **Config — Reload** Trust Task makes already-stored configuration take effect on the running maintainer without a process restart. It applies to **hot-reloadable** keys only — those whose [`ConfigField.requiresRestart`](../_shared/0.1/config.schema.json) is false. For each such key whose stored value differs from the live value, the maintainer updates the running value and names the key in `keysReloaded`; a key already live (a no-op) is omitted. Restart-gated keys are never reloaded here — they need [`config/restart`](../restart/0.1/).

It is the apply step that pairs with [`config/patch`](../patch/0.1/): patch stores the value, reload makes a hot-reloadable one live.

## Conformance

Producer: send with no parameters. Carry a proof.

Consumer: verify the configuration-admin capability. Diff the stored effective configuration against the live in-memory configuration; for every hot-reloadable key whose value differs, update the live value and add the key to `keysReloaded`. Leave restart-gated keys untouched. Audit the reload.

## Security & Privacy

**Posture change, so attributable.** Reload does not introduce new values — those were authorised at `config/patch` — but it is the moment a stored value becomes live, so it changes running behaviour and MUST be attributable. Hence `proofRequirement: REQUIRED` and an audited event.

**Idempotent.** Because reload only re-applies already-stored values, repeating it is safe: a second call with nothing changed returns an empty `keysReloaded`. It cannot introduce a value the write path (`config/patch`, itself gated) did not already accept.
