---
slug: push/wake
version: "0.2"
wireCompatibleWith: "0.1"
title: Push — Wake
summary: A trigger (the device's mediator or its VTA) asks the push gateway to deliver a contentless wake to a handle. The gateway authorizes against the VTA-provisioned allowlist, then fires the doorbell.
status: draft
targetFrameworkVersion: "0.5"
category: notifications
keywords:
  - push
  - wake
  - trigger
  - gateway
  - doorbell
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: trigger
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: push gateway
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: The gateway authorizes the wake against the handle's allowlist, so the trigger's authenticated identity is load-bearing. Over the DIDComm binding the authcrypt sender provides it intrinsically; over HTTPS the caller carries a did-signed proof. A spoofed/replayed wake is harmless (a contentless doorbell — the device connects and finds the same or empty queue), so proof is RECOMMENDED, not REQUIRED.
sideEffects:
  level: none
  rationale: "Delivers a contentless wake to a handle; no persisted state change and a spoofed wake is harmless."
subjectPath: /handle
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
retention:
  class: transient
  rationale: >-
    The gateway authorises the wake against the handle's allowlist, fires the
    contentless doorbell, and returns a status; `sideEffects` is `none` and the
    task persists nothing. The only durable effect is a negative one — on
    `tokenUnregistered` the gateway drops the handle's dead token. What the task
    does not retain, the infrastructure around it does: the gateway's own
    operational logs and the push provider's delivery records are where the
    timing of every wake accumulates, and that is described in the body rather
    than declared here because it is outside this document's scope.
errorCodes:
  - code: push/wake:unknownHandle
    meaning: No such handle at this gateway.
    retryable: false
  - code: push/wake:notAllowed
    meaning: The authenticated trigger DID is not on the handle's VTA-provisioned allowlist.
    retryable: false
  - code: push/wake:tokenUnregistered
    meaning: The push service reports the handle's token permanently unregistered. The gateway drops the token; the device must re-register (push/register). The queued message remains for the consumer's next voluntary pickup.
    retryable: false
---

## Abstract

**Push — Wake** is how a **trigger** — the device's **mediator** (queue-driven: it alone knows the device is offline with messages waiting) or its **VTA** (policy-driven: e.g. a delegated step-up) — asks the gateway to deliver a **contentless wake** to a device. The gateway authorizes the request against the handle's VTA-provisioned allowlist ([`push/provision/0.1`](../../provision/0.1/spec.md)) and, if the trigger is allowed, fires the doorbell defined by the [push wake-up binding](../../../../bindings/push/0.1/spec.md) §2 (gateway → device, via APNs / FCM / Web Push).

**This task carries only the binding's contentless hint fields** — `v`, and optionally `mediator` / `count` / `urgency`. It **MUST NOT** carry any Trust Task content, `reason`, relying-party identity, or task type: the wake is a doorbell, and the actual messages are drained from the mediator over the DIDComm binding after the device wakes.

Carried over the **DIDComm binding** (preferred — the authcrypt sender authenticates the trigger; this is how a `did:webvh` mediator or VTA authenticates) or HTTPS. The `recipient` is the gateway.

## Conformance

A conforming **producer** (the trigger) **MUST**:

1. Be on the handle's allowlist (a mediator or VTA the controller VTA provisioned).
2. Populate `handle` and `v`; OPTIONALLY `mediator` (so a multi-mediator consumer knows which to drain), `count`, `urgency`.
3. Carry **no** task content — only the fields above.
4. A **mediator** SHOULD fire only when the consumer's pickup queue is non-empty *and* it is offline, and SHOULD coalesce multiple queued messages into at most one wake per short window. A **VTA** fires on its own policy decision and MAY wake an already-connected device (a harmless redundant doorbell).

A conforming **consumer** (the push gateway) **MUST**:

1. Resolve `handle`; unknown → `push/wake:unknownHandle`.
2. Verify the authenticated trigger DID is on the handle's allowlist; otherwise `push/wake:notAllowed` (no push sent).
3. Deliver a push containing **only** the binding §2 contentless fields — never the handle, never task content.
4. On a push-service "permanently unregistered" report, drop the stored token and return `push/wake:tokenUnregistered`, leaving the queued message for the consumer's next pickup.

## Payload

`handle` (REQUIRED); `v` (REQUIRED — binding wire version, currently `1`); `mediator`, `count`, `urgency` (OPTIONAL hints). No other fields.

## Response

`status` — `delivered` (the push service accepted the wake) or `tokenUnregistered` (handled per the error above; included for symmetry where the gateway reports outcome in-band).

## Security & Privacy

### Data carried

The wake is defined by what it refuses to carry. It holds no Trust Task content
at all — the delivered push is the [push binding](../../../../bindings/push/0.1/spec.md)'s
contentless doorbell — so the request is `handle`, the wire version `v`, and
three optional hints: `mediator`, naming which mediator holds the queue so that a
multi-mediator consumer knows which to drain; `count`, an approximate and
advisory number of queued messages; and `urgency`, either `interactive` or
`background`, which a consumer **MAY** map to platform priority and alert
behaviour.

Those three hints are the only place any information about what is waiting can
escape, and each of them leaks a little. `count` is queue depth. `urgency:
interactive` asserts that a human is expected to look at something now rather
than later. `mediator` names routing infrastructure. None of them says what a
message contains, and — importantly — none of them is required: a trigger that
sends only `handle` and `v` delivers a fully conforming wake. Omitting them is a
real minimisation choice rather than a formality, and a producer handling
sensitive flows **SHOULD** treat it as one.

The response carries `status`, either `delivered` or `tokenUnregistered`. The
second value is a small disclosure in its own right: it tells the trigger that
the device's token is permanently dead, which is an inference about the device —
uninstalled, wiped, restored to new hardware — that the trigger did not
previously hold.

### Correlation

This is the residual leak of the whole push design, and it is not in any member.
It is in the pattern.

The gateway sees every wake for every handle it hosts, timestamped. The push
provider — Apple, Google, or a browser vendor's service — sees every delivery to
the underlying token, also timestamped, and unlike the gateway it usually knows
which device and which platform account that token belongs to. Neither party
sees content. Both see rhythm, and rhythm is informative: a burst of
`interactive` wakes at 02:00 says that something happened which someone was
expected to attend to immediately; a long silence followed by resumption says a
device was off, or its owner was away; a steady low-`count` cadence looks nothing
like a step-up approval, and `urgency` makes that distinction explicit rather
than leaving it to be inferred.

The mitigations are behavioural, because there is no member to remove.
Deployments handling sensitive flows **SHOULD** coalesce and jitter their wakes,
and **MUST NOT** vary the payload by task type. The second rule carries more
weight than the first: a payload that differs per task converts a contentless
doorbell into a channel of at least one bit per wake, delivered to a party
outside the exchange entirely. `handle` is the stable key all of this joins on,
and it changes only when the device's platform token rotates.

Both parties declare `identifierScope: public`, and both declarations are forced
by how authorisation works here rather than chosen for convenience. The
**trigger**'s DID **MUST** be the same value the controller VTA wrote into
`allowedTriggers` at [`push/provision`](../../provision/0.2/spec.md) — an
identifier assigned in one relationship and recognised in another — so a
pairwise trigger identifier would make the allowlist unmatchable and every wake
would fail with `push/wake:notAllowed`. The **push gateway** must be addressable
by one value held simultaneously by the device that registered, the VTA that
provisioned, and this trigger, none of whom obtained it from each other. The
price is that a gateway can group every wake it ever handled by trigger, and so
can see which mediator is busy, and when.

### Retention

The task itself retains nothing. The gateway checks the allowlist, fires the
doorbell, and answers with a status; `sideEffects` is `none`. The single durable
consequence is a deletion — on `tokenUnregistered` the gateway drops the dead
token, and the device must re-register before it can be woken again.

The wake is also not delivery, which matters for what is *not* consumed: the
queued message stays at the mediator for the consumer's next voluntary pickup,
so a wake that is never acted on loses nothing.

What does accumulate lives outside this document. The gateway's operational logs
and the push provider's delivery records are precisely where the timing pattern
described above is retained, and neither is governed by this specification. A
gateway that keeps per-wake log lines indefinitely holds the traffic history of
every device it serves; one that keeps aggregate counters holds almost nothing.
That choice is invisible on the wire and is the most consequential retention
decision in the push family.

### Consent/purpose

A wake happens on the authority of the allowlist, never on possession of a
handle. The gateway authenticates the trigger first — intrinsically via the
authcrypt sender over DIDComm, or via a did-signed proof over HTTPS — and only
then checks membership of the handle's VTA-provisioned `allowedTriggers`. A
party that obtains a handle by any means still cannot wake the device, which is
what bounds the abuse of a value that necessarily circulates.

The design tolerates forgery because the forgery is worthless. A spoofed or
replayed wake causes the device, at worst, to connect to its mediator and find
the same queue or an empty one: a wasted wake rather than a security event. The
framework places confidentiality and sender authentication in the DIDComm
authcrypt envelope at pickup, never in the wake, so nothing is trusted here that
would be damaging to have wrong.

The hints exist for routing and for priority, and that is the boundary of their
purpose. A trigger that encoded meaning into `count` or `urgency` — using
queue-depth values or the interactive flag to signal something about the
message — would be operating a covert channel to the push provider, a party that
is not in this exchange, cannot be authenticated by it, and retains what it
sees.
