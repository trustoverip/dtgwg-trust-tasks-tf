---
slug: vta/backup/complete-export
version: "1.0"
title: "VTA Backup — Complete Export"
summary: "Acknowledge that an export bundle was downloaded, so the agent can release it."
status: draft
targetFrameworkVersion: "0.5.0"
category: key-management
keywords:
  - backup
  - disaster-recovery
  - acknowledgement
parties:
  - role: backup operator
    requirement: REQUIRED
    member: issuer
  - role: verifiable trust agent
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The acknowledgement is what closes an export in the recipient's record — the document that says the copy reached the operator's hands rather than expiring unfetched. That distinction only survives in an audit trail if the party who made it can be named.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Replay is harmless to state here, since the transition is idempotent, but not to the record: a re-sent acknowledgement re-dates a download that happened once, and the date is the whole evidentiary content of this document.
sideEffects:
  level: mutating
  rationale: >-
    Moves the bundle to a terminal state and releases the staged bytes and the transport token. The bundle cannot be fetched afterwards. Not classed destructive because nothing the agent holds is lost — the copy that was downloaded is the point, and the agent's own state is untouched.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
retention:
  class: durable
  rationale: >-
    The recipient keeps the fact of the acknowledgement, not the bundle. "A copy of this agent left here, acknowledged by this operator, on this date" is what a later investigation into a leaked copy starts from, and it stays relevant long after the bytes are gone.
errorCodes:
  - code: vta/backup/complete-export:notFound
    meaning: >-
      The recipient holds no export bundle under this identifier that this producer may act on. Deliberately conflates "no such bundle", "not an export bundle", and "not yours" — see Correlation.
    retryable: false
  - code: vta/backup/complete-export:terminalState
    meaning: >-
      The bundle was already aborted or expired, so there is nothing to acknowledge. Distinct from a second acknowledgement of a completed bundle, which succeeds.
    retryable: false
related:
  - vta/backup/initiate-export
  - vta/backup/abort
---

## Abstract

The **VTA Backup — Complete Export** Trust Task tells an agent that an export bundle it minted has been downloaded, and that it may release the staged bytes and retire the transport token.

It exists because the alternative — letting every bundle sit until it expires — leaves a fetchable copy of the agent alive for the remainder of its slot after the copy has already been made. Acknowledging closes that window at the moment it stops being needed.

The task is deliberately not required for correctness. An operator who downloads and never acknowledges loses nothing; the bundle expires and is collected on its own. What is lost is the recipient's ability to tell the two outcomes apart, and `downloaded` in the response is where that distinction is made explicit rather than assumed.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

The authority is **having created the bundle**. A recipient **MUST** refuse an acknowledgement from any party other than the one that issued the [`initiate-export`](../../initiate-export/1.0/spec.md) this bundle came from, and **MUST** answer such a request as `notFound` rather than as a refusal — see [Correlation](#correlation) for why the two are conflated.

No wider entitlement is implied and none is needed: the producer is closing something it already caused, and the operator authority that admitted the export is what admits its acknowledgement.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient`, transport identity or `proof` establishes who sent this document, never that they own the bundle it names. The ownership check is a separate comparison against the bundle's recorded creator, and a recipient that omits it lets any authenticated operator retire another's bundle.

## Definitions

**`bundleId`** — the handle returned in the descriptor from `initiate-export`. Opaque: a producer quotes what it was given and **MUST NOT** derive, guess or enumerate one.

**`downloaded`** — the recipient's account of whether the byte stream was actually fetched before this acknowledgement arrived. `false` is not a failure; it is the honest answer when the operator skipped the download, or when the transport slot expired before they got to it. It is reported rather than inferred because a producer that acknowledges without downloading has made no copy, and only the recipient knows which happened.

## Request

The producer is the operator that initiated the export; the recipient is the agent that minted the bundle. The request payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Acknowledging a completed download

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vta/backup/complete-export/1.0#request",
  "issuer": "did:example:operator",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T00:02:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "bundleId": "3f2504e0-4f89-41d3-9a0c-0305e82c3301"
  }
}
```

## Response

The producer of the response is the recipient of the request. Its payload is the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` with one of the codes declared in the front matter, not a `#response` document.

### The bundle was fetched before the acknowledgement

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000004",
  "type": "https://trusttasks.org/spec/vta/backup/complete-export/1.0#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:operator",
  "issuedAt": "2026-01-01T00:02:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "bundleId": "3f2504e0-4f89-41d3-9a0c-0305e82c3301",
    "downloaded": true
  }
}
```

## Security & Privacy

### Data carried

Almost nothing, and that is the design. The request carries one opaque handle; the response echoes it with a boolean. No password, no token, no URL, and no description of what the bundle contained.

A recipient **MUST NOT** include the bundle's `transportToken` or `transportUrl` in this response. They are spent by the time this task is sent, and re-stating a bearer credential in a second document doubles the number of places it can leak for no gain.

`bundleId` is a capability-like reference in one narrow sense: a party that both learns it and can authenticate as the bundle's creator can retire that bundle. Producers **SHOULD** keep it out of shared logs for the life of the bundle, and it stops mattering afterwards.

### Correlation

The recipient learns when this operator completed a download, which together with the `initiate-export` document gives it the elapsed time between minting and fetching. Over several exports that describes operator behaviour — automated pipelines and human ones look different — and it is intrinsic: an agent cannot record that a bundle was retired without recording when.

`bundleId` joins this document to the `initiate-export` that produced it and to any `abort` that raced it; `threadId` joins request to response. Both are intrinsic to the exchange.

The producer's identifier **MUST** be the same one that initiated the bundle, because the ownership check is a comparison against it. This is the one place this task family requires an identifier to be reused across documents, and it does not extend past the bundle's lifecycle.

Three distinct conditions are answered as `notFound`: no bundle with that id, a bundle of the import kind with that id, and a bundle owned by a different operator. Conflating them is deliberate. Distinguishing them would let anyone holding a guessed identifier learn whether a bundle exists and whether it belongs to someone else, which turns an opaque handle into an enumeration oracle over the agent's backup activity.

### Retention

The recipient keeps the acknowledgement as a `durable` record and releases the bundle's bytes. Those are opposite dispositions on purpose: the copy is what should not linger, and the fact that a copy was made is what should.

A recipient **SHOULD** retain enough to answer "did this export leave, or did it expire unfetched?" after the bundle itself is gone. That single bit is what separates an investigation into a leaked copy from a search for one that never existed.

### Consent/purpose

The purpose is closing out an export the same operator initiated. Nothing about this document licenses anything further: it is an acknowledgement, and a recipient **MUST NOT** treat it as authorization to mint a replacement bundle, re-open a terminal one, or extend any slot.

Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements), this specification does not declare a consent, approval or step-up requirement.
