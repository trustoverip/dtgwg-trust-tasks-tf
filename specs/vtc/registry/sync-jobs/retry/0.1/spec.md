---
slug: vtc/registry/sync-jobs/retry
version: "0.1"
title: "VTC Registry Sync Jobs — Retry"
summary: Requeue a trust-registry reconciliation job the community's reconciler has abandoned, so a membership change that never reached the registry is dispatched again.
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
    The task re-dispatches a membership assertion to a third party — the trust
    registry — under the community's own authority. A proofless document would
    let anyone who reaches the maintainer's transport replay a membership change
    the community had abandoned.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. It is also
    the substantive need here: a duplicate retry re-dispatches the same mutation,
    and §7.2 item 11 can only absorb a duplicate inside a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Changes queue state on the maintainer and causes a subsequent write to the
    trust registry. It does not itself write to the registry, which is why this
    is mutating rather than destructive — nothing is removed and the dispatch
    that follows is the one the community already decided to make.
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related: []
---

## Abstract

A Verifiable Trust Community publishes its membership to a trust registry through a queue, because the registry is a separate party that can be unreachable, misconfigured, or running an incompatible version at the moment a member joins. When the queue gives up on a job, the membership change is simply absent from the registry, and stays absent: the reconciler will not retry a job it has abandoned, and the audit position that produced the job has long since advanced past it.

This task is how that is undone. It is a Trust Task rather than an API call because the community administrator and the community maintainer are distinct parties who authenticate each other over the same transport as every other community operation, and because the act — re-asserting a member to a third-party registry — is exactly the kind of consequential step the framework's proof and authorization rules exist to govern.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A consumer **MUST** requeue only jobs in the terminal `failed` state. A job in any other state is still owned by the reconciler, and resetting it would cause the same mutation to be dispatched twice; the consumer **MUST** report such a job in `skipped` with reason `notFailed` rather than requeueing it, and **MUST NOT** fail the task on account of it.

A consumer **MUST** report an unknown `jobId` in `skipped` with reason `notFound`. A job may legitimately have been retried, discarded, or removed by the maintainer's retention sweep between the producer reading the list and issuing this request, and that race is ordinary rather than exceptional.

A consumer **MUST** reset a requeued job's attempt count. The count is a budget spent against a registry that would not answer; once an operator has changed something, carrying it over would let the first tick exhaust the budget again.

A consumer **MUST NOT** treat an empty `requeued` array as a failure. It is the correct answer when nothing was eligible.

## Authorization

The authority is **administrative control of the community whose membership is being published** — the same authority that admits and removes members. A consumer **MUST** establish that the `issuer` holds it before requeueing anything, and the conformance rules above constrain only *which* jobs may move, never *who* may ask.

That is distinct from identity and proof validation. Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient`, transport identity or `proof` establishes who sent the document and that it is unaltered; none of it establishes that the sender administers this community. A maintainer serving several communities **MUST** check the authority against the community the job belongs to, not merely that the issuer administers *some* community.

This task is not open to any caller. The queue names members in the clear and re-asserts their membership to a third party, so an unauthorized caller could both enumerate a community's roster and republish a membership the community had deliberately stopped publishing.

## Definitions

- **`jobId`** — identifier of a single queued job, as `vtc/registry/sync-jobs/list` reports it. Chosen by the maintainer. A consumer resolves it within the community the request addresses.
- **`allFailed`** — requests that every job in the terminal `failed` state be requeued. Chosen by the producer. It is spelled as its own member, and constrained to `true`, so that a client bug which drops `jobId` cannot silently widen a single retry into a bulk one; the schema admits exactly one of the two members.
- **`requeued`** — the jobs now scheduled for dispatch, each with the `memberDid` whose record it carries, so the response says what will actually be republished rather than only how many rows moved.
- **`skipped`** — the jobs the consumer declined, each with a machine-readable `reason`. Reported rather than raised as an error so that one ineligible row does not defeat a bulk retry.

## Request

The community administrator (`issuer`) sends the request to the community maintainer (`recipient`); the top-level schema is in [`payload.schema.json`](payload.schema.json).

### Retrying one abandoned job

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/registry/sync-jobs/retry/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "jobId": "07c6c189-8272-4761-96e9-0ef869380665"
  }
}
```

### Retrying everything after fixing a shared cause

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vtc/registry/sync-jobs/retry/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-00000000010f",
  "payload": {
    "allFailed": true
  }
}
```

## Response

The community maintainer — the `recipient` of the request, now responding — returns the two outcome arrays described by the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` rather than a `#response` document; a request that moved no jobs because none were eligible is not a failure and **MUST** be answered with a `#response` carrying empty arrays.

`requeued` lists the jobs now scheduled, `skipped` those declined with the reason each was declined. A producer reads `skipped` to decide whether to look again: `notFailed` means the job is moving on its own, `notFound` means it is gone.

### One job requeued, one no longer eligible

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/registry/sync-jobs/retry/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "requeued": [
      {
        "jobId": "07c6c189-8272-4761-96e9-0ef869380665",
        "memberDid": "did:example:member"
      }
    ],
    "skipped": [
      {
        "jobId": "1f2e3d4c-5b6a-4798-8765-43210fedcba9",
        "reason": "notFailed"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries an identifier or a single boolean and nothing else — it names work already held by the recipient rather than supplying any. The response carries member DIDs, which are the sensitive members here: `requeued[].memberDid` identifies people whose community membership is about to be re-asserted to a third party. It is present because an operator acting on a queue needs to know who is affected, and an identifier alone does not say. A producer **MUST NOT** place member identifiers, error prose, or any description of the queue's contents in `ext`; everything this task needs is in the named members.

The smallest payload that answers the task is exactly what is specified: the request needs one discriminator, and the response needs the identifiers it moved plus the identifiers it did not, with a reason for each refusal.

### Correlation

The response joins job identifiers to member DIDs, so anyone who sees it learns which member each queued change concerns. The maintainer already holds that mapping — it is the queue — so nothing new is disclosed to the recipient. It is disclosed to the *producer*, and to any intermediary that can read the document, which is why the transport's confidentiality matters as much as its integrity here.

`threadId` correlates request and response, and the `jobId` values are stable handles that persist across calls until the job is dispatched, discarded, or swept. A producer cannot vary either: both name state the recipient holds. Repeated retries of the same `jobId` are visibly the same job, and that is the intended behaviour, not a leak to be mitigated.

### Retention

The recipient needs to retain nothing beyond the queue rows it already holds; this task mutates them rather than adding a record. The response has evidentiary value for an operator reconstructing why a member was republished, so a producer that logs it retains member DIDs and should apply the same retention it applies to its membership records rather than to its request logs.

### Consent/purpose

The purpose is operational repair: completing a membership publication the community already decided to make and that failed in transit. It is not a new decision about the member, and the data **MUST NOT** be reused to construct a roster for any other purpose, nor to infer anything about a member from the fact that their publication failed — a failed job records an infrastructure fault, not a fact about the person.
