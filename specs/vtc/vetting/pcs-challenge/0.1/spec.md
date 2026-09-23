---
slug: vtc/vetting/pcs-challenge
version: "0.1"
title: "VTC — Vetting PCS Challenge"
summary: An applicant asks a community for the single-use challenge its hidden-vetting proof must be bound to, so that a proof counted once cannot be submitted again.
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords:
  - vetting
  - hidden-vetting
  - challenge
  - freshness
  - replay
parties:
  - role: applicant
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
    The community binds the challenge it issues to the applicant that asked, and spends it
    against that applicant's submission. A proofless request would let anyone consume another
    applicant's open challenge, which is a denial of service against the one party who cannot
    simply ask again without re-gathering a proof.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A challenge is a freshness anchor with a lifetime. A request that cannot be placed in time cannot be told from one replayed after the previous challenge expired.
sideEffects:
  level: mutating
  rationale: The community records the challenge it issued, and replaces any it had open for this applicant.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
retention:
  class: exchange
  rationale: The challenge lives for one submission attempt. A community discards it when the proof is counted, and sweeps it when it expires unused.
errorCodes:
  - code: vtc/vetting/pcs-challenge:notHiddenVetting
    meaning: "This community publishes no criterion that accepts a hidden-vetting proof, so there is no challenge to issue."
    retryable: false
related:
  - vtc/join-requests/manifest
  - vtc/join-requests/submit
  - vetting/attestation
---

## Abstract

A hidden-vetting proof is bound to a challenge. Without one the proof verifies as often as it is
submitted, and the second submission of the same bytes counts as readily as the first. This task
is the half that makes the binding mean something: the community issues the challenge, keeps it,
and accepts it exactly once.

It is a Trust Task rather than a field on the manifest because a manifest is public, cacheable
and the same for everyone, and a freshness anchor is none of those things.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

Any party the community would accept a join submission from may ask for a challenge: the
entitlement is the same one that admits an applicant to apply at all, and a community that
publishes an open criterion is open here too. What the challenge does **not** confer is any
standing in the admission decision — it is a nonce, and [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)
already says that verifying `issuer` establishes who asked, never that they are entitled to
anything. A community that restricts who may apply refuses here with its own code rather than
issuing a challenge it will not honour.

## Definitions

**`criterionId`** — OPTIONAL. The `id` of the admission criterion the coming submission answers,
chosen by the applicant from the community's manifest. A community running a single hidden-vetting
criterion has no use for it; one running several MAY use it to refuse at this point rather than at
submit. A consumer MUST NOT treat it as a commitment: the submission names its own criterion.

**`challenge`** — 16 bytes as lowercase hex, chosen by the community. It is compared for equality
and never parsed. The applicant binds it into the proof's context; a proof carrying any other
value is refused.

**`expiresAt`** — when the community stops accepting it. An applicant that misses the window asks
for another and builds another proof.

## Request

The applicant issues the request to the community; the payload is the top-level object in
[`payload.schema.json`](payload.schema.json), and every member of it is OPTIONAL — the applicant
is identified by `issuer`, and that is the whole input the community needs.

### An applicant asks for a challenge before submitting

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/vetting/pcs-challenge/0.1#request",
  "issuer": "did:example:applicant",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-23T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "criterionId": "kernel-developer-private"
  }
}
```

## Response

The community answers with the challenge and its expiry, in the sub-schema reachable via
`$anchor: "response"`. A community that cannot issue one answers with `trust-task-error` and the
code above, never with a `#response` document.

A community MUST hold at most one open challenge per applicant: asking again replaces the
previous one rather than adding to it, so an applicant cannot accumulate challenges and spend
them across several submissions.

### The challenge, and how long it stands

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/vetting/pcs-challenge/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:applicant",
  "issuedAt": "2026-09-23T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "challenge": "a961aa3e63df4d15c9ad565feed87c46",
    "expiresAt": "2026-09-23T10:15:01Z"
  }
}
```

## Security & Privacy

### Data carried

A nonce and a deadline. The request carries nothing about the applicant beyond the fact that they
intend to submit, which the community learns at submit anyway. A producer MUST NOT put anything
identifying a vetter in `ext`: the point of the exchange this challenge belongs to is that the
community never learns who vetted.

### Correlation

The community learns that this applicant is preparing a submission, and when. That timing is
unavoidable — a freshness anchor must be issued before the thing it anchors — and it says nothing
about which vetters were involved. An applicant that asks and does not submit tells the community
only that.

Both parties are declared `identifierScope: public`, and for once that is not a shortcut. The
community is a community: applicants find it, read its manifest and cite it by an identifier that
has to be the same one for everybody. The applicant's identifier is the one the challenge is bound
to and the one the coming submission arrives under — a pairwise identifier would mean the
community could not tell that the party submitting is the party it issued to, which is the entire
function of the exchange.


### Retention

A community keeps the challenge until it is spent or expires, and SHOULD sweep expired rows
rather than leave them. Nothing about it is worth keeping afterwards: it proves freshness at the
moment it is consumed and has no evidentiary value later.

### Consent/purpose

The challenge is collected for one submission attempt by the applicant that asked for it. Reusing
it to correlate that applicant with anything else is outside the purpose it was issued for.
