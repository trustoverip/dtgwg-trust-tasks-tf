---
slug: config/patch
version: "0.1"
title: Config — Patch
summary: Write per-key overrides into a maintainer's runtime configuration, reporting which took effect, which await a restart, and which were rejected.
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
  requirement: REQUIRED
  rationale: A configuration change alters the maintainer's runtime posture — a log level, a storage path, a feature flag. The producer's identity MUST be verifiable for audit and to prevent a stealth reconfiguration.
sideEffects:
  level: mutating
  rationale: "Writes per-key configuration overrides; recoverable by patching the keys back."
consequences:
  - "Changes runtime behaviour for keys applied immediately; stages the rest until the next restart."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: config/patch:permissionDenied
    meaning: The consumer lacks the configuration-admin capability.
    retryable: false
---

## Abstract

The **Config — Patch** Trust Task writes per-key overrides into a maintainer's runtime configuration. It is partial-success by design: each override is validated independently, and the response sorts every key into `applied` (in effect now), `pendingRestart` (stored, effective after a restart), or `rejected` (not written, with a reason). It is the write half of the pair completed by [`config/show`](../show/0.1/).

Overrides are carried under a single `overrides` object rather than as top-level members, so the payload keeps a fixed `additionalProperties: false` envelope while the override keys themselves stay open.

## Conformance

Producer: supply a non-empty `overrides` map. Carry a proof.

Consumer: verify the configuration-admin capability. For each override, validate the key and value against the registry; on failure add it to `rejected` with a reason (unknown key, wrong type, out-of-range, allowlist mismatch) and continue with the rest. For each accepted key, persist the value and report it as `applied` when it is now in force or `pendingRestart` when it takes effect on restart (matching the key's `ConfigField.requiresRestart`). Audit the change, redacting the value of any secret-bearing key before it is written to the audit record.

## Security & Privacy

**Posture change.** A patch changes how the maintainer runs. Hence `proofRequirement: REQUIRED` and mandatory audit — a reconfiguration must be non-repudiable, and a stealth change of, say, a TLS path or a storage location must be attributable.

**Secret-bearing values.** An override may carry a sensitive value. The maintainer MUST redact such values in the audit record (never log the plaintext), the same redaction `config/show` applies on read. The `metadata` exposure classification reflects that the *response* returns only key names and dispositions, not the written values.

**Partial success is not silent failure.** A key in `rejected` was not written; a caller MUST read `rejected` and not assume a 2xx-equivalent response means the whole patch applied. Reporting rejections as data (rather than failing the whole call) is deliberate — one bad key should not discard a batch of good ones — but it puts the onus on the caller to check.
