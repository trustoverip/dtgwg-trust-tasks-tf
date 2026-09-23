---
slug: vtc/vetting/vetters/event-mode
version: "0.1"
title: "VTC — Vetting Event Mode"
summary: A vetter asks to vet at a named event at one of the community's published rates, and the community answers where that request stands — never whether the vetter granted it to themselves.
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords:
  - vetting
  - hidden-vetting
  - rate-limit
  - event
parties:
  - role: vetter
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: community
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The request asks for a higher rate under a member's own name, and the community's answer to it
    is recorded against that member. A proofless request would let anyone enrol anyone in an
    event — filling a group floor with members who never asked, which is precisely the check that
    makes the event's smaller anonymity set survivable.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: The request is judged against the event's window and the community's grace period, both of which are positions in time.
sideEffects:
  level: mutating
  rationale: The community records this vetter's request for this event, which counts towards the group floor and, once an approver acts, towards who may draw under the event's label.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
retention:
  class: durable
  rationale: The request outlives the exchange — it is the membership record the event's token label is checked against — and a community keeps it while that label is live.
errorCodes:
  - code: vtc/vetting/vetters/event-mode:notAVetter
    meaning: "The requester holds no live vetter grant in this community."
    retryable: false
  - code: vtc/vetting/vetters/event-mode:unknownEvent
    meaning: "This community is not running an event by that name."
    retryable: false
  - code: vtc/vetting/vetters/event-mode:unknownTier
    meaning: "`tier` is not one of the tiers this community publishes for this event."
    retryable: false
  - code: vtc/vetting/vetters/event-mode:badWindow
    meaning: "`window` is not a window this community will accept for this event — inverted, or outside the event's own dates."
    retryable: false
  - code: vtc/vetting/vetters/event-mode:alreadyRequested
    meaning: "This member has already asked for this event. The recorded request stands; nothing was changed."
    retryable: false
  - code: vtc/vetting/vetters/event-mode:eventClosed
    meaning: "The event's label has closed. Tokens under it are no longer issued or accepted."
    retryable: false
related:
  - vtc/vetting/vetters/pcs-tokens
  - vtc/vetting/vetters/pcs-root
  - vetting/attestation
---

## Abstract

A vetter's ordinary rate is a slow, constant drip — a few tokens a tick, whether or not they have
vetted anyone. That is the right rate for ordinary weeks and the wrong one for a conference, where
one member may sit at a desk and meet twenty people in a day.

Event mode is the exception, and it is deliberately not a bigger drip. It is a **separate token
label for a named event**, with its own rate, its own expiry, and a group of vetters that must be
large enough before it exists at all. This task is how a vetter asks to be in that group.

Three things it is not:

- **Not self-service.** The vetter asks; someone else approves. Raising your own cap is exactly
  what a coerced vetter would be made to do, so the request and the grant are separate acts by
  separate parties, and this task only carries the first.
- **Not a bigger number under the same key.** Tokens live and die with their label. Three days at
  twenty a day under the monthly key would leave sixty tokens spendable for the rest of the month;
  under an event key they expire shortly after the event, which is what bounds the stockpile.
- **Not free.** A spend under an event label narrows the anonymity set from *some vetter in this
  community* to *some vetter at this event*. That is why the group floor exists, and why the
  event's name has to describe a gathering rather than a desk.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

The entitlement is a **live vetter grant in this community**, exactly as for
[`vtc/vetting/vetters/pcs-root`](../../pcs-root/0.1/spec.md).

It is not the entitlement to *hold* event mode. A consumer MUST NOT treat a request as a grant,
and MUST NOT let the requesting member be the party that approves it. An approval is an act by an
admin or moderator of the community, carried by that community's own administrative surface, and
this specification deliberately does not describe a Trust Task for it: a task the vetter could
send would be a task a vetter could be made to send.

A consumer MUST refuse to open an event label for a group smaller than its published `groupFloor`,
and MUST NOT lower that floor for a particular event. The floor is what keeps an event key from
being a pseudonym.

## Definitions

**`eventId`** — the community's name for a gathering. It is the anonymity set for every token spent
under the event's label, so it names something people attend, not a shift or a room.

**`tier`** — one of a small published menu of rates (a community might publish `desk` at 20 a tick
and `busy-desk` at 40). Picking from a menu rather than naming a number keeps a vetter's requested
rate from being a distinguishing detail in itself.

**`window`** — the days the vetter expects to be vetting, within the event's own dates.

**`groupFloor`** — the smallest group the community will open an event label for.

**`state`** — where the request stands: `pending` while it waits for an approver and for the floor,
`approved` once the label is live for this vetter.

## Request

The vetter issues the request to its community; the payload is the top-level object in
[`payload.schema.json`](payload.schema.json).

### Asking to work a desk at a summit

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/event-mode/0.1#request",
  "issuer": "did:example:vetter",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-01T09:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "eventId": "kernel-summit-2026",
    "tier": "desk",
    "window": {
      "startDate": "2026-10-12",
      "endDate": "2026-10-14"
    }
  }
}
```

## Response

The community answers in the sub-schema reachable via `$anchor: "response"`. Every refusal above is
a `trust-task-error`; `pending` is an answer, not one of them.

The response carries `groupSize` and `groupFloor` so that a vetter waiting on an event can tell the
two reasons for waiting apart — nobody has approved it yet, or not enough people have asked. It
carries a **count** and never a list: who else is at the event is the anonymity set itself.

Once `state` is `approved`, the vetter draws under `label` through
[`vtc/vetting/vetters/pcs-tokens`](../../pcs-tokens/0.1/spec.md) at `dripPerTick`, on the same
unconditional schedule as any other label. A consumer MUST stop issuing under the label after
`closesAfter`, and MUST stop accepting spends under it at the same point.

### Recorded, waiting for the floor

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/event-mode/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:vetter",
  "issuedAt": "2026-10-01T09:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "eventId": "kernel-summit-2026",
    "state": "pending",
    "tier": "desk",
    "window": {
      "startDate": "2026-10-12",
      "endDate": "2026-10-14"
    },
    "groupSize": 2,
    "groupFloor": 3
  }
}
```

### Approved, and the label is live

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/event-mode/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:vetter",
  "issuedAt": "2026-10-03T11:20:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fe",
  "payload": {
    "eventId": "kernel-summit-2026",
    "state": "approved",
    "tier": "desk",
    "window": {
      "startDate": "2026-10-12",
      "endDate": "2026-10-14"
    },
    "groupSize": 5,
    "groupFloor": 3,
    "label": "token/event/kernel-summit-2026",
    "dripPerTick": 20,
    "closesAfter": "2026-10-28"
  }
}
```

## Security & Privacy

### Data carried

An event name, a tier and a pair of dates. Nothing here names an applicant, and a producer MUST NOT
put one in `ext` — the tokens this unlocks are spent on people the vetter has not met yet.

### Correlation

This exchange is the one place in hidden vetting where the community deliberately learns something
about a particular vetter: that they expect to be at a named event. That is **capacity, not
activity** — a member sitting at a vetting desk at a public summit is visible to everyone walking
past it — and it is the price of the higher rate.

What it costs downstream is the narrower anonymity set: a token spent under the event's label came
from *someone in that event's group*, not from *someone in this community*. Three things keep that
survivable, and a consumer MUST implement all three rather than picking among them: the group
floor, the event label's expiry shortly after the event, and the rule that the event's name
describes a gathering. A community that approves a two-person event with a floor of three has
published a coin flip.

Both parties are declared `identifierScope: public` because the request is recorded against a
member and checked against that same member when they draw. A pairwise identifier would let one
vetter fill a group floor alone.

### Retention

The request is kept while the event's label is live, and dropped with it. What it authorised — the
tokens — leaves no record tying a spend to the member who drew it, so dropping the request drops
the last thing that could.

### Consent/purpose

Event mode exists to let a community's vetting keep up with a gathering without abandoning the rate
limit. Using the record of who asked for event mode to infer who vetted whom at that event is
outside that purpose, and the group floor is the design that makes the inference weak rather than
the policy that forbids it.
