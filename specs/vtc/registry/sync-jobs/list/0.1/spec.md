---
slug: vtc/registry/sync-jobs/list
version: "0.1"
title: "VTC Registry Sync Jobs — List"
summary: Enumerate a Verifiable Trust Community's trust-registry reconciliation queue, so an operator can see which membership changes have not reached the registry and why.
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
  requirement: RECOMMENDED
  rationale: >-
    Read-only enumeration. RECOMMENDED because the transport already
    authenticates the caller and guarantees integrity; the disclosure is
    governed by the authorization rule rather than by proof strength.
sideEffects:
  level: none
  rationale: Reads queue state; persists nothing and dispatches nothing.
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related: []
---

## Abstract

A community publishes membership to a trust registry through a durable queue, because the registry is a separate party that can be unreachable or incompatible at the moment a member joins. When a job in that queue is abandoned, the membership change is absent from the registry and nothing in the ordinary flow says so.

This task makes the queue legible: which member, which mutation, how many attempts, and the registry's own last answer. It is a Trust Task rather than an API call because what it enumerates is a roster — the answer names members in the clear — and the framework's authorization and proof rules are what keep that behind the same door as the rest of the community's administrative surface.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A consumer **MUST** return `lastError` verbatim as the registry gave it, or `null`. The value's purpose is to let an operator distinguish a registry that refused the request from one that never answered, and paraphrasing it destroys exactly that.

A consumer **MUST** report `state: "failed"` only for jobs it will not dispatch again without an operator, and **MUST NOT** give such a job a `nextAttemptAt`. A failed job with a future attempt time would claim a schedule that does not exist.

A nullable member of this response may be sent either as `null` or omitted, and a producer **MUST** treat the two identically. The distinction is an artefact of how a serialiser spells absence, never a signal — in particular, an absent `nextCursor` means the last page exactly as `null` does.

A consumer **SHOULD** populate `purgeDueAt` where it operates a retention sweep over abandoned jobs. A producer **MUST** treat it as a deadline after which the failure disappears and the member remains unpublished, and **MUST NOT** treat it as a scheduled repair.

A consumer **MUST** order results stably across pages so that `cursor` enumerates without repeating or skipping.

## Definitions

- **`state`** — the job's position in the queue's lifecycle. `pending` and `inFlight` are moving on their own; `failed` is terminal and needs an operator. Chosen by the maintainer.
- **`kind`** — which mutation the job carries: first publication, an update, a removal, or a departure marker. Chosen by the maintainer from the membership event that created the job.
- **`memberDid`** — the member whose registry record the job carries, in the clear. It is the field that makes the list actionable: an operator holding only an opaque job identifier cannot tell who stopped publishing.
- **`attempts`** — how many dispatches have been made. On a failed job the value discriminates two unrelated causes: a low count means the registry answered and refused, a count at the maintainer's ceiling means it never answered at all.
- **`lastError`** — the registry's last answer, bounded diagnostic prose for a human. A consumer of this task **MUST NOT** parse it; `state` and `attempts` carry everything a program should decide on.
- **`purgeDueAt`** — when the maintainer's retention sweep will delete the row.

## Request

The community administrator (`issuer`) sends the request to the community maintainer (`recipient`); the top-level schema is in [`payload.schema.json`](payload.schema.json). Every member is optional: an empty payload enumerates the whole queue.

### Listing only the jobs that need an operator

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/registry/sync-jobs/list/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "state": "failed",
    "limit": 50
  }
}
```

## Response

The community maintainer — the `recipient` of the request, now responding — returns the matching jobs and a continuation token, per the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` rather than a `#response` document; an empty queue is not a failure and is answered with an empty `items` array.

`nextCursor` is `null` on the last page. A producer paginates by echoing it back as `cursor`.

### One abandoned job, with the registry's own answer

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/registry/sync-jobs/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "items": [
      {
        "jobId": "07c6c189-8272-4761-96e9-0ef869380665",
        "kind": "publishMember",
        "memberDid": "did:example:member",
        "state": "failed",
        "attempts": 1,
        "lastError": "registry rejected registry/record/put: unsupportedType",
        "createdAt": "2026-01-01T00:00:00Z",
        "lastAttemptedAt": "2026-01-01T00:00:00Z",
        "purgeDueAt": "2026-01-31T00:00:00Z"
      }
    ]
  }
}
```

`nextAttemptAt` and `nextCursor` are absent here rather than explicitly `null`: this is the last page of a job that will never be dispatched again, and both spellings mean that.

## Security & Privacy

### Data carried

The request carries a filter and a page size. The response carries the community's roster in the clear, one member DID per queued change, together with diagnostic prose from a third party. Those two are the sensitive members. `memberDid` is unavoidable — the task exists to say who is affected — and `lastError` is bounded at 1024 characters precisely because it originates outside the community and may quote an upstream URL or status. A producer **MUST NOT** place member identifiers or queue contents in `ext`.

The smallest payload that answers the task is the one specified: enumerating fewer fields would leave an operator unable to tell a refusal from a timeout, which is the distinction the task exists to surface.

### Correlation

The response joins member identifiers to timestamps — when a change was enqueued, when it was last attempted — so a reader can infer when each member joined, changed role, or left, to the resolution of the queue. That is more than a roster: it is a roster with a timeline. It is inherent to enumerating a queue and cannot be varied by the producer; it is the main reason this task is administratively gated rather than open.

`threadId` correlates request and response, and `jobId` is a stable handle across pages and across calls until the job leaves the queue.

### Retention

The recipient retains nothing on account of this task; it reads rows it already holds for its own reconciliation. A producer that stores the answer has taken a copy of the community roster with timestamps, and **SHOULD** retain it under the same policy as its membership records rather than its diagnostic logs — the fact that it arrived through an operational surface does not make it operational data.

### Consent/purpose

The purpose is operational: letting an administrator see which membership publications have not landed, and decide whether to retry or discard them. The answer **MUST NOT** be reused to build a membership directory, and a failed job **MUST NOT** be read as a fact about the member — it records that a third-party write failed, which is a statement about infrastructure.
