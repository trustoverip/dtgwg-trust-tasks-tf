---
slug: device/heartbeat
version: "0.2"
wireCompatibleWith: "0.1"
title: Device — Heartbeat
summary: Periodic check-in from a Companion or Service that refreshes lastSeenAt, carries optional state digests, and lets the maintainer deliver queued operations (queued wipes especially).
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords:
  - device
  - heartbeat
  - keepalive
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: device
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: vault maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Heartbeat is frequent and low-stakes; transport-level auth is sufficient. Recommended for non-session-bound transports for attribution.
sideEffects:
  level: none
  rationale: "Periodic check-in refreshing server-managed lastSeenAt; no user-visible state change."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    `queuedOperations` returns the full Trust Task documents the maintainer
    parked while the device was offline — including a `device/wipe`, whose
    payload carries the operator's human-readable `reason` for wiping —
    alongside `syncHint` and the authoritative `serverTime`. Operational data
    about the device and its maintainer, so `metadata` rather than the `none`
    an acknowledgement-only response would carry. Inbound the request is almost
    empty — an optional `platform` build string and an optional `vaultSeq`
    integer, neither of which describes a person — so `ingests` is `metadata`.
    What is sensitive about a heartbeat is not its contents but its arrival
    time, and the repetition of that arrival; `ingests` has no vocabulary for
    timing, so that property is described in the body rather than declared here.
retention:
  class: transient
  rationale: >-
    The document is consumed on arrival: the maintainer overwrites a single
    `lastSeenAt` field on the existing DeviceBinding, hands back anything queued
    in the outbox, and keeps nothing else. The nuance worth stating is that
    `lastSeenAt` is an overwrite and not an append — a maintainer that instead
    records a row per check-in has built a durable minute-resolution presence
    log that this task never asked for, and it should treat that log as the
    durable record it is.
errorCodes:
  - code: device/heartbeat:notRegistered
    meaning: The issuer's DID has no DeviceBinding. The consumer SHOULD complete device/register.
    retryable: false
---

## Abstract

The **Device — Heartbeat** Trust Task is the periodic check-in every Companion and Service makes against the maintainer. It refreshes `lastSeenAt`, gives the maintainer a chance to push queued operations (notably **queued wipes** for targets that were offline at the moment a wipe was issued), and lets the maintainer hint when a sync is due.

Recommended cadence: every 5 minutes for online Companions, every 60 minutes for Services.

## Conformance

A conforming **producer** populates optional state digests if available. A conforming **consumer** updates `lastSeenAt`, returns any queued operations from the maintainer's outbox (the consumer MUST execute them in order before continuing), and computes a `syncHint`.

The consumer MUST execute returned `wipe` operations before any other op (including any subsequent heartbeat).

## Payload

Optional: `platform` update, `vaultSeq` baseline.

## Response

`serverTime`, `queuedOperations`, `syncHint`.

## Security & Privacy

### Data carried

The request carries almost nothing, and that is deliberate: the schema declares
no required members at all. `platform` is sent only when the build string
changed since registration, and `vaultSeq` is an integer sync baseline. Neither
is a statement about a person. The heartbeat's information content is its
existence.

The response is the heavier half. `serverTime` is the maintainer's authoritative
clock, which a consumer **MAY** compare against its own to detect drift; a
divergence beyond roughly five minutes indicates a manipulated client clock, an
NTP failure, or an attempt to backdate, and a consumer **SHOULD** log it and
refresh before any security-sensitive operation. `syncHint` says only whether a
`vault/sync` is due. `queuedOperations` is where real content appears: each
entry embeds a **complete Trust Task document** the device would have received
had it been online, and for `kind: wipe` that is a `device/wipe` payload
including the operator-authored, free-text `reason` for the wipe. An operator
writing that reason should understand that it is written for the maintainer's
records and delivered to the device.

Each embedded document carries its own proof from the maintainer's admin key,
and a consumer verifies the **inner** document before executing it. This is why
the heartbeat envelope only RECOMMENDS a proof: the envelope is a carrier, and
the authority travels with the operation inside it rather than with the
check-in that delivered it.

### Correlation

This is the member of the family with a correlation problem, and it is not in
the payload. At the recommended cadence — every five minutes for an online
Companion — the *sequence* of heartbeats is a continuous, minute-resolution
record of when a person's device was awake, which for a browser or mobile
Companion is a close proxy for when that person was at it. No single heartbeat
reveals this. The series does, and a maintainer accumulates the series by
operating normally.

What the series supports is more than presence. Gaps describe working hours,
weekends, and illness; a block of shifted arrival times describes travel across
time zones, and the `serverTime` drift check makes that shift explicit rather
than inferred. `platform` transitions date a device's software updates. A
principal with several enrolled devices produces several traces against the same
maintainer, all joined by `consumerDid`, so "the laptop went quiet at 18:40 and
the phone did not" is derivable without reading a single payload member.

The mitigation is retention rather than redaction, because the data cannot be
minimised out of a liveness protocol: heartbeats are high-volume, and a
maintainer **MAY** sample its audit records, recording only state transitions —
a `lastSeenAt` gap beyond some threshold, a `syncHint` change, a queued-operation
delivery — rather than every check-in. A maintainer that records only transitions
holds an operational signal; one that records every heartbeat holds an
attendance log.

The device party declares `identifierScope: pairwise`. Liveness requires a
stable identifier — a heartbeat that could not be attached to the right
`DeviceBinding` would refresh nothing — but that stability is needed only
between this device and its own maintainer, and a device that reused one
identifier across maintainers would let them align their presence traces into a
single timeline.

### Retention

The document is transient: the maintainer overwrites `lastSeenAt` on the
existing binding, drains anything queued for the device, and keeps nothing of
the check-in itself. Queued operations leave the outbox once delivered, so the
response's content is retained by the *device* for as long as it takes to
execute them, not by the maintainer.

The distinction that matters is between overwriting and appending. `lastSeenAt`
is a single field holding one answer to "when was this device last seen"; a
maintainer that satisfies the same requirement by appending a row per check-in
has, without changing anything on the wire, converted a transient exchange into
a durable presence log. Both implementations conform. Only one of them holds a
record of a person's day, and an implementer **SHOULD** know which one it built.

### Consent/purpose

The heartbeat exists for liveness and for delivery: to tell the maintainer the
device is reachable, and to give the maintainer a window in which to hand over
operations — a queued wipe above all — that it could not deliver while the
device was dark. That is what the presence data is collected for.

Nothing in this task authorises reading the same data as a record of a person's
availability, attendance, or working pattern, and that reuse is the one worth
naming because it requires no new access, no new task, and no new member — only
a decision to keep what a liveness protocol produces. A maintainer that retains
only transitions has made that reuse impractical for itself. Whether it should
be prevented by policy, and whether a principal is told what cadence their
devices report at, are consumer questions on which this specification takes no
position.
