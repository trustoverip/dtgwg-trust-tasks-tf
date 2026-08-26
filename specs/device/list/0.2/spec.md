---
slug: device/list
version: "0.2"
wireCompatibleWith: "0.1"
title: Device — List
summary: List DeviceBindings (Companions and Services) registered on the maintainer, optionally filtered by kind, capability, status, or last-seen time.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords:
  - device
  - list
  - inventory
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: vault consumer
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: vault maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: Read-only inventory query.
sideEffects:
  level: none
  rationale: "Read-only listing of registered device bindings."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
retention:
  class: transient
  rationale: >-
    The maintainer consumes the filters and the cursor to compute one page and
    keeps nothing; `sideEffects` is `none` and the returned `cursor` encodes a
    position rather than content. The asymmetry worth naming is on the other
    side: the caller leaves holding a snapshot of the whole inventory, which
    this task neither constrains nor observes, and which goes stale the moment
    a device is disabled or wiped.
errorCodes: []
---

## Abstract

The **Device — List** Trust Task drives the "my devices" UI: every Companion the user has registered, plus Services authorised on their VTA. Used by the user-facing device-manager to spot unfamiliar devices, disable lost ones, and audit AI-agent presence.

## Conformance

Producer: optional filters; treat `cursor` as opaque. Consumer: scope returned devices to those the requesting consumer can see (admin sees all; Service consumers see only themselves unless explicitly granted broader visibility).

## Payload

All optional: `consumerKindFilter`, `formFactorFilter`, `serviceKindFilter`, `capabilityFilter`, `includeDisabled`, `includeWiped`, `lastSeenSince`, pagination.

## Response

`devices` — list of DeviceBinding. Recommended order: `lastSeenAt` descending.

## Security & Privacy

### Data carried

The request carries nothing the maintainer did not already hold. Every member is
optional, and every one of them is control data: four enumerated filters, two
`include*` booleans, a `lastSeenSince` timestamp the caller chose, a `pageSize`,
and a `cursor` the maintainer itself minted. That is why `ingests` is `none`.

The response is the entire content of this task, and it is a good deal more than
its `metadata` class suggests at a glance. Each entry is a **full**
`DeviceBinding`, not a summary: `deviceId` and `consumerDid`, the `displayName`
the device chose at registration, its `platform` build string, its `attestation`
— including the `aaguid` naming its authenticator model — its `keyCustody` tier
and algorithms, the `capabilities` it was granted, and the `registeredAt`,
`lastSeenAt`, `disabledAt`, and `wipedAt` timestamps. One call therefore returns,
for every device a principal owns, what it is called, what it runs, what
hardware protects its keys, what it is allowed to do, and when it was last
awake. The `metadata` classification is a statement that none of this is
released credential material, not a statement that it is uninteresting.

Minimisation here is enforced by the consumer rather than expressed in the
payload, because the filters narrow a page and not an authorisation. A consumer
**MUST** scope the returned devices to what the requesting party may see:
admin-class Companions see all bindings, while a Service consumer — a mediator,
an AI agent, a daemon — sees only its own record unless it has been granted the
`deviceAdmin` capability. The `includeDisabled` and `includeWiped` defaults of
`false` are part of the same discipline: the ordinary "my devices" screen does
not resurface the laptop that was wiped after it was stolen.

### Correlation

Enumeration is itself information, and this is the task that performs it. Any
one `DeviceBinding` is modest; the set is a profile. It says how many devices a
person has, which platforms they live on, how current their software is, and —
through `lastSeenAt` — which devices they actually use and which they abandoned
months ago without disabling.

`lastSeenSince` is the sharpest member, because it converts the maintainer's
accumulated presence data into a query surface. A caller that pages the same
list at a series of thresholds recovers an activity ranking across a principal's
devices without ever seeing a single `device/heartbeat`, and a caller that asks
for `lastSeenSince` a few minutes ago is running a live presence probe — built
entirely from a read that declares `sideEffects: none` and leaves no trace
unless the maintainer chooses to audit reads. A maintainer that records
`device/list` calls can tell the difference between a person opening their
device manager and a consumer polling it on a timer; one that does not, cannot.

Two members correlate outward. `displayName` was chosen at registration for its
owner's convenience and is disclosed here to every consumer authorised to list,
so a name that identifies a person reaches an audience the person did not pick.
`attestation.aaguid` alongside `platform`, read down the whole list, is a
hardware and software inventory of a household or a desk.

Across principals nothing joins, because the response is scoped to what the
caller may see — with the single exception that a `deviceAdmin` grant is
precisely a grant of cross-principal visibility, and should be read as one.

The vault consumer declares `identifierScope: pairwise`. Its DID must be stable
for the maintainer to resolve its visibility scope, but no third party is asked
to recognise it, and a consumer reusing one identifier across maintainers would
let them align the device inventories they each return to it.

### Retention

For the maintainer the read is transient: the filters and cursor are consumed to
produce one page, `truncated` and `cursor` describe a position rather than
content, and no state changes.

The retention question that matters is on the caller's side, and this task
neither constrains nor observes it. A consumer that caches the page holds device
records that will outlive their own accuracy — a device disabled five minutes
after the read is still `disabledAt: null` in the cached copy — so a consumer
**SHOULD** re-read rather than persist, and **MUST NOT** treat a cached binding
as current when making a security decision about a device.

One asymmetry follows from the family's retention design. Because bindings
survive decommissioning by design, a caller that sets `includeDisabled` or
`includeWiped` is not asking for a larger inventory; it is asking for history,
including devices that were wiped because they were lost. That is a legitimate
request for an audit view and a poor default for a user-facing list, which is
why both default to `false`.

### Consent/purpose

The purpose is self-inspection. This task drives the "my devices" screen so that
a person can recognise their own devices, spot one they do not recognise,
disable a lost one, and see which AI agents are enrolled against their VTA. It
exists to make the maintainer's device inventory legible to the principal whose
devices they are.

The visibility scoping is what keeps it that. A Service consumer sees only
itself by default, so an enrolled AI agent does not acquire an ambient view of
its principal's laptops and phones merely by being enrolled; seeing them
requires a `deviceAdmin` grant, which is a deliberate act. The reuse worth
naming is the polling one described above: the same call that answers "what
devices do I have" answers "is this person at their desk right now" when it is
run on a schedule, and nothing in the payload distinguishes the two. Whether a
maintainer gates or audits that pattern is a consumer policy question on which
this specification takes no position.
