---
slug: config/restart
version: "0.1"
title: Config — Restart
summary: Request a graceful restart so a process supervisor brings the maintainer back up with restart-gated configuration applied; refuses when no supervisor is present.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - config
  - restart
  - lifecycle
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
  rationale: A restart interrupts the maintainer's availability and is a denial-of-service vector if unattributed. It MUST be non-repudiable — who requested the bounce, and when.
sideEffects:
  level: mutating
  rationale: "Triggers a graceful shutdown and supervisor-driven restart; persistent state is preserved (not destructive) but the running process is replaced."
consequences:
  - "Interrupts availability for the drain-plus-startup window."
  - "Applies restart-gated configuration changes that were stored but not yet live."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: config/restart:permissionDenied
    meaning: The consumer lacks the configuration-admin capability.
    retryable: false
  - code: config/restart:supervisorRequired
    meaning: No process supervisor was detected, so a graceful exit would not be followed by a restart — it would just stop the maintainer. Refused. Configure a supervisor (or set the maintainer's explicit opt-in) and retry.
    retryable: false
---

## Abstract

The **Config — Restart** Trust Task requests a graceful restart of the maintainer so that restart-gated configuration — keys whose [`ConfigField.requiresRestart`](../../_shared/0.1/config.schema.json) is true, stored via [`config/patch`](../../patch/0.1/) but not yet live — takes effect. The maintainer drains in-flight work and exits; a process supervisor brings it back up.

It **refuses** unless it can identify the supervisor that will restart it. Without one, "restart" is just "stop": a graceful exit with nothing to bring the process back would take the maintainer down. The response echoes the detected `supervisor` and the `drainTimeoutSeconds` window so the caller knows what will happen and how long the outage is.

## Conformance

Producer: send with no parameters. Carry a proof.

Consumer: verify the configuration-admin capability. Detect the supervisor that manages the process; if none is found, return `supervisorRequired` and do **not** exit. Otherwise, audit the request **before** signalling shutdown (so the record survives even if the drain wedges), then begin a graceful shutdown: stop accepting new work, drain in flight up to `drainTimeoutSeconds`, and exit for the supervisor to restart. Return `supervisor` and `drainTimeoutSeconds`.

`supervisor` is an opaque maintainer-defined label — the framework does not enumerate it, so a maintainer names whatever it detected (a service manager, an orchestrator, an explicit operator opt-in) without canonical having to know the taxonomy.

## Security & Privacy

**Availability, so attributable.** A restart is the one config operation whose effect is an outage. An unattributed restart is a denial-of-service primitive — repeatedly bouncing the maintainer. Hence `proofRequirement: REQUIRED` and an audit record written *before* shutdown, so a restart cannot be triggered without a trace even if the process never cleanly comes back.

**Refuse-if-unsupervised is a safety gate, not an inconvenience.** Exiting without a supervisor turns a "restart" into an indefinite outage. The `supervisorRequired` refusal ensures the caller cannot accidentally (or maliciously) convert a reconfiguration into a shutdown; it is a precondition, not a transient error, so it is not retryable until a supervisor exists.
