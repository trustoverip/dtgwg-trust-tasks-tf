---
slug: vtc/registry/sync-jobs/discard
version: "0.1"
title: "VTC Registry Sync Jobs — Discard"
summary: Delete an abandoned trust-registry reconciliation job without dispatching it, accepting that the member's registry record stays as it is.
status: draft
targetFrameworkVersion: "0.5.0"
category: reputation
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Irreversible. The job is the community's only remaining record that a
    membership change never reached the registry, and a proofless document would
    let anyone reaching the maintainer's transport destroy that evidence.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed
    discard is harmless only because the second attempt finds nothing; placing
    the document in time is what lets §7.2 item 11 absorb the duplicate instead.
sideEffects:
  level: destructive
  rationale: >-
    Deletes queue state that cannot be reconstructed. The audit position that
    produced the job has advanced past it, so nothing will re-derive it and the
    member's pending change is lost for good.
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related: []
---

## Abstract

Not every abandoned reconciliation job should be retried. A member may have left since; an operator may have written the registry record by hand; the change may have been superseded by a later one that did land. In those cases the queue holds work that should never be dispatched, and leaving it there leaves a permanent warning about a problem that no longer exists.

This task deletes one such job. It is deliberately narrow and deliberately consequential: it is a Trust Task rather than an API call because destroying the community's only record of an unpublished membership change is exactly the kind of act the framework's authorization, proof and freshness rules are for.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A consumer **MUST** delete only a job in the terminal `failed` state, and **MUST** reject a request naming a job in any other state. A pending or in-flight job is work the reconciler is still doing, and deleting it would drop a membership change the community had not given up on.

A consumer **MUST NOT** alter the trust registry as part of this task. Discarding removes the community's intent to publish; it does not retract anything already published, and a consumer that also deleted the registry record would silently turn a cleanup into a removal.

A consumer **MUST** return the discarded job's `memberDid`. The caller discards by identifier, and the response is the only place the act is named in terms of who it affected.

## Authorization

The authority is **administrative control of the community whose membership queue is being modified** — the same authority that admits and removes members, and the same one [`vtc/registry/sync-jobs/retry`](../../retry/0.1/spec.md) requires. A consumer **MUST** establish that the `issuer` holds it for the community the job belongs to.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient`, transport identity or `proof` establishes who sent the document and that it is unaltered; none of it establishes that the sender administers this community. A maintainer serving several communities **MUST** scope the check to the job's own community.

This task is not open to any caller. It destroys evidence that a membership change failed, and an unauthorized caller could use it to make a community's publication gap permanently invisible.

## Definitions

- **`jobId`** — the job to delete, as [`vtc/registry/sync-jobs/list`](../../list/0.1/spec.md) reports it. Required, and single: there is no bulk form, because the act is irreversible and a bulk discard would let one request erase every record of a systemic failure.
- **`memberDid`** — returned in the response: the member whose pending change was dropped. Present so that the response says what the discard actually cost, rather than echoing an opaque identifier back.

## Request

The community administrator (`issuer`) sends the request to the community maintainer (`recipient`); the top-level schema is in [`payload.schema.json`](payload.schema.json).

### Discarding a job that should not be retried

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/registry/sync-jobs/discard/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "jobId": "07c6c189-8272-4761-96e9-0ef869380665"
  }
}
```

## Response

The community maintainer — the `recipient` of the request, now responding — confirms the deletion with the job identifier and the member it concerned, per the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` rather than a `#response` document: an unknown job, or a job in a non-terminal state, is an error here rather than a reported skip, because the request names exactly one job and there is no partial outcome to report.

### Confirming what was dropped

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/registry/sync-jobs/discard/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "jobId": "07c6c189-8272-4761-96e9-0ef869380665",
    "memberDid": "did:example:member"
  }
}
```

## Security & Privacy

### Data carried

The request carries one identifier. The response carries one member DID — the sensitive member — because the act is otherwise unnamed: an operator confirming a discard needs to see whose publication they just abandoned, and an opaque identifier does not say. A producer **MUST NOT** place member identifiers or a rationale for the discard in `ext`; the request needs nothing beyond the identifier, and any justification belongs in the maintainer's audit record rather than on the wire.

### Correlation

The response ties a job identifier to a member, which the recipient already knows and the producer learns. `threadId` correlates request and response. Because the task is single-target, it discloses strictly less than an enumeration: a caller learns about one member, and only one they already named.

### Retention

The recipient deletes a row and **SHOULD** retain an audit record that it did, including who asked — this is the one point where the community's record of an unpublished membership change ceases to exist, and the deletion is more worth recording than the row was. That record's evidentiary value is what a later reader needs to explain a member's absence from the registry, so it **SHOULD** outlive the queue row it replaces.

### Consent/purpose

The purpose is operational cleanup: retiring a pending change that should not be dispatched. It **MUST NOT** be used to suppress evidence of a systemic publication failure — that is what the single-target constraint and the audit expectation above are for — and the member identifier it returns **MUST NOT** be reused for any purpose beyond confirming the act.
