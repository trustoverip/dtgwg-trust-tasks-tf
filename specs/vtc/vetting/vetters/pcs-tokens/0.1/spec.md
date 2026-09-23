---
slug: vtc/vetting/vetters/pcs-tokens
version: "0.1"
title: "VTC — Vetting PCS Tokens"
summary: A vetter draws its scheduled tick of attestation tokens from the community, unconditionally and at a published rate, so that anonymous attestations stay rate-limited without anyone learning who has been busy.
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords:
  - vetting
  - hidden-vetting
  - blind-signature
  - rate-limit
  - drip
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
    The community serves a quota to a named member once per tick. A proofless request would let
    one member exhaust another's quota, or draw an unbounded number of tokens under a member who
    never asked — and tokens are the only thing bounding how many anonymous attestations exist.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: The once-per-tick rule is only enforceable against documents that can be placed in time relative to the tick they name.
sideEffects:
  level: mutating
  rationale: The community records that this member was served for this label and tick, and signs tokens that will be spendable until the label closes.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
retention:
  class: durable
  rationale: The served-tick record outlives the exchange — it is what makes "once per tick" a rule rather than an intention — and a community keeps it while the label it names is live.
errorCodes:
  - code: vtc/vetting/vetters/pcs-tokens:notAVetter
    meaning: "The requester holds no live vetter grant in this community."
    retryable: false
  - code: vtc/vetting/vetters/pcs-tokens:labelNotLive
    meaning: "`label` is not a token label this community is currently issuing under."
    retryable: false
  - code: vtc/vetting/vetters/pcs-tokens:alreadyServed
    meaning: "This member has already been served for this label and tick."
    retryable: false
  - code: vtc/vetting/vetters/pcs-tokens:overQuota
    meaning: "More tokens were asked for than the community's published drip rate for this label. Nothing was signed."
    retryable: false
  - code: vtc/vetting/vetters/pcs-tokens:badOpeningProof
    meaning: "A request's opening proof does not verify. Nothing was signed."
    retryable: false
  - code: vtc/vetting/vetters/pcs-tokens:eventRefused
    meaning: "`label` is an event label and this member is not in the approved group for it."
    retryable: false
related:
  - vtc/vetting/vetters/pcs-root
  - vetting/attestation
  - vetting/decline
---

## Abstract

An anonymous attestation cannot be rate-limited by who made it, because nobody knows who made it.
It is rate-limited by what it spends: a token, blind-signed by the community, revealed when the
attestation is counted. This task is how a vetter gets them.

The draw is **unconditional**. A vetter asks on its schedule whether or not it has vetted anyone,
because a fetch that happened only when someone was busy would announce that they were busy — and
the whole point of the exchange is that the community learns nothing about a vetter's activity.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

The entitlement is a **live vetter grant in this community**, checked against the community's own
membership and grant records, exactly as for
[`vtc/vetting/vetters/pcs-root`](../../pcs-root/0.1/spec.md). For an event label there is a second
entitlement: membership of the group approved for that event.

The quota is authorization too, and it belongs to the consumer: a community MUST refuse a batch
larger than the rate it publishes, before signing any of it. A request is not a negotiation, and a
producer's restraint is not a control.

## Definitions

**`label`** — the token label to draw under: the community's current monthly label, or an event
label it has approved. Tokens are only spendable while their label is live, which is what makes an
event's higher rate end with the event.

**`tick`** — the vetter's own schedule counter for this label. It exists so a community can enforce
"once per tick" without knowing anything about the vetter's week.

**`requests`** — one blinded serial per token asked for, each with a proof that it opens to a serial
the vetter holds. A community verifies every one of them before signing any.

**`preCredentials`** — the blind signatures, positionally matching `requests`.

## Request

The vetter issues the request to its community; the payload is the top-level object in
[`payload.schema.json`](payload.schema.json).

### A scheduled draw at the published rate

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/pcs-tokens/0.1#request",
  "issuer": "did:example:vetter",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-01T09:05:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "label": "token/2026-09",
    "tick": 1,
    "requests": [
      {
        "commitment": "z2umykFwGKzcv489j6kMGJnPTgKqAqMvCSPVkpyPCqAKA",
        "openingProof": "zP3kHy6ZpnVAaRt7Y3PQRa2AeKkFSHJpQnoAneHhnDQxEJmWq7qy8H1oqRTPDtG8Zc"
      }
    ]
  }
}
```

## Response

The community answers with one blind signature per request, in the sub-schema reachable via
`$anchor: "response"`. Every refusal above is a `trust-task-error`.

A community SHOULD record the served tick only after the signatures exist, for the same reason
enrolment does: losing the record costs a repeated tick, recording one that was never served costs
the vetter their draw.

### The signed batch

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/pcs-tokens/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:vetter",
  "issuedAt": "2026-09-01T09:05:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "label": "token/2026-09",
    "tick": 1,
    "preCredentials": [
      "z26q5oFrp6i2aTKLp6Y6jsLESJMoNfZQwk8VKXc37c24bJPj5fwBoUEsqv51ADbnUNM"
    ]
  }
}
```

## Security & Privacy

### Data carried

Blinded commitments and signatures over them. Nothing here names an applicant, and a producer MUST
NOT put one in `ext`: a token drawn today is spent on an applicant the vetter has not met yet, and
carrying any hint of one would defeat the exchange it exists for.

### Correlation

A community learns which members drew tokens and when — and because the drip is constant, that
tells it nothing it did not already decide. The **timing** of draws is the exposure worth naming:
a vetter that asks only when it is about to vet turns a schedule into a signal, so a producer
SHOULD draw on a fixed schedule and a consumer SHOULD serve a fixed rate rather than one that
tracks demand.

Both parties are declared `identifierScope: public` for the same reason as enrolment: the quota
is per member per tick, so the community has to recognise the same member across ticks, and a
pairwise identifier would turn one quota into as many as the vetter cared to mint. As with
enrolment, being named here costs the vetter nothing downstream — what they spend these tokens on
carries no identifier.


### Retention

The served-tick record is kept while its label is live. The community also records each serial
when it is later spent; a serial is a random scalar and links to no vetter, which is what allows
double-spend detection without attribution.

### Consent/purpose

Tokens are issued to bound the rate of anonymous attestation in this community. Using the draw
record to infer a vetter's activity is outside that purpose, and a constant drip is the design
that makes the inference empty.
