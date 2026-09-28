---
slug: git-ns/bridge/job/list
version: "0.1"
title: "Git Namespaces — List Bridge Jobs"
summary: "An administrator lists the bridge jobs queued or run in the namespaces they administer — every namespace, for a community administrator — with each job's kind, queue state and last error."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - bridge
  - jobs
  - administration
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTC
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "The answer is operational detail about a namespace's automation — which jobs ran, which failed, and why — disclosed only to its administrators. The VTC authorises the caller from its own records, so it must know from the document itself who the caller is; a proof binds the request to its `issuer` on every transport, and a bearer session proves nothing about the document."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no effect to replay, but an issue time dates the request for a VTC that audits administrative reads."
sideEffects:
  level: none
  rationale: "Reads the VTC's own job queue and changes nothing: it does not retry, cancel or dispatch a job."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns, for administered namespaces only, each queued or run job's kind, repository, bridge DID, queue state, attempt count and last reported error."
retention:
  class: exchange
  rationale: "The request carries at most a namespace, a state filter and a paging cursor, and is needed only to answer it."
errorCodes:
  - code: git-ns/bridge/job/list:notAdministrator
    meaning: "The caller administers no namespace — they hold neither the community-administrator capability nor a live, explicitly recorded `git.ns.admin` — or `namespace` names one they do not administer or one this VTC does not have. The three are answered alike, so the code cannot be used to learn which namespaces exist."
    retryable: false
related:
  - git-ns/bridge/job
  - git-ns/bridge/result
  - git-ns/namespace/list
  - git-ns/repo/list
  - git-ns/view
---

## Abstract

[`git-ns/bridge/job`](../../0.4/spec.md) is the VTC's side of dispatching one piece of forge work; [`git-ns/bridge/result`](../../../result/0.1/spec.md) is the bridge's report on one. Neither gives an administrator the queue as a whole — which jobs are stuck pending, which failed and why, which repository a namespace's automation last touched. This task is that queue view: every job the VTC has dispatched or queued in the namespaces the caller administers, each with its kind, state, attempt count and last error. It is an admin console's *Jobs* table, a command-line `bridge jobs`.

It is a read. It changes nothing, retries nothing and cancels nothing.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

Not consequential, and declared for clarity, following [`git-ns/namespace/list`](../../../../namespace/list/0.1/spec.md) and [`git-ns/repo/list`](../../../../repo/list/0.1/spec.md), whose Authorization sections this one restates for jobs. The caller is the DID the document's verified `proof` binds to its `issuer`; an unsigned document is refused with `proofRequired`.

The entitlement is, for each namespace, **the community-administrator capability** in the VTC's own access control, or **`git.ns.admin` on that namespace** held by explicit record and live at the instant the VTC evaluates the request, by a current member. A job belongs to the namespace it acts in (`namespace` on the job itself), so this task scopes exactly as the namespace and repository listings do: a namespace admin sees the jobs in their own namespaces, and a community administrator sees every job. A caller who administers no namespace is refused with `git-ns/bridge/job/list:notAdministrator`, and so is a caller naming, in `namespace`, one they do not administer or one the VTC does not have.

The capability and `git.ns.admin` are read from the VTC's own records at execution time; the `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

**Administered namespace** — as in [`git-ns/namespace/list`](../../../../namespace/list/0.1/spec.md): for a holder of the community-administrator capability, every namespace bound to the VTC; for anyone else, each namespace on which they hold a live, explicitly recorded `git.ns.admin`.

**Job** — one dispatch of a [`git-ns/bridge/job`](../../0.4/spec.md) to a bridge, as the VTC's own queue tracks it. `kind` is the same seven values that specification defines; `state` is the VTC's own queue bookkeeping, driven by [`git-ns/bridge/result`](../../../result/0.1/spec.md) reports, and is not itself a Trust Task member of either.

## Request

The administrator sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Identifies the caller from the verified `proof` ([Authorization](#authorization)), refusing an unsigned document with `proofRequired`.
2. Determines the caller's administered namespaces from its own records. Refuses with `git-ns/bridge/job/list:notAdministrator` a caller who has none and does not hold the community-administrator capability, and a caller whose `namespace` is not one of them.
3. Returns `jobs`: every job dispatched or queued in an administered namespace — or in `namespace` only, when given — narrowed to `state` where given, newest (`createdAt`) first.
4. Returns at most `limit` jobs (clamped to 1..=500, default 100) and, when more match, a `nextCursor` to continue.

A VTC **MUST NOT** return a job outside the caller's administered namespaces, and **MUST NOT** change anything on this task.

### Carol lists failed jobs in her namespace

```json
{
  "id": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6d01",
  "type": "https://trusttasks.org/spec/git-ns/bridge/job/list/0.1",
  "threadId": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6d01",
  "issuer": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-26T09:20:00Z",
  "payload": {
    "namespace": "ns_01J8Z6Q4M2",
    "state": "failed"
  }
}
```

## Response

The VTC answers with the matching jobs, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`: `proofRequired`, or `git-ns/bridge/job/list:notAdministrator`.

### What Carol sees

```json
{
  "id": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6d02",
  "type": "https://trusttasks.org/spec/git-ns/bridge/job/list/0.1#response",
  "threadId": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6d01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
  "issuedAt": "2026-09-26T09:20:01Z",
  "payload": {
    "jobs": [
      {
        "jobId": "job_01J8Z8C3D4",
        "namespace": "ns_01J8Z6Q4M2",
        "bridgeDid": "did:webvh:QmBridgeScid9:bridge.acme.example",
        "kind": "bootstrap",
        "state": "failed",
        "repo": "github.com/acme/widgets",
        "attempts": 3,
        "createdAt": "2026-09-26T08:30:00Z",
        "acceptedAt": "2026-09-26T08:30:05Z",
        "lastError": "the installation lacks administration:write"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries at most a namespace, a state filter and a paging cursor. The response carries, for jobs in administered namespaces only: the repository each job acted on, the bridge it was dispatched to, its queue state and attempt count, and its last reported error verbatim. `lastError` is diagnostic prose from the bridge and can name forge permissions the community's installation lacks — operational detail for the people who fix it, which is why it goes to nobody but the namespace's administrators.

### Correlation

The VTC declares `identifierScope: public`; the administrator `pairwise`, matching [`git-ns/namespace/list`](../../../../namespace/list/0.1/spec.md) and [`git-ns/repo/list`](../../../../repo/list/0.1/spec.md). `jobId` and `bridgeDid` are already visible to the namespace's administrators through [`git-ns/bridge/job`](../../0.4/spec.md) and [`git-ns/bridge/result`](../../../result/0.1/spec.md) as those exchanges happen; this task adds the enumeration and the running state, not a new identifier.

### Retention

The request is kept only as long as the VTC keeps request logs; nothing in it needs to outlive the exchange. A VTC that audits administrative reads **MAY** record that the caller listed jobs, and **SHOULD NOT** record the response.

### Consent/purpose

The purpose is to let a namespace's administrators see what its automation has done and is doing: which jobs are stuck, which failed and on what error, so they can act (retry through a fresh job, fix a forge permission, or escalate). A caller **MUST NOT** republish `lastError` outside the community.
