---
slug: git-ns/activity/list
version: "0.1"
title: "Git Namespaces — List Activity"
summary: "An administrator lists recent rights changes, drift reports and bridge jobs in the namespaces they administer — every namespace, for a community administrator — newest first."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - activity
  - audit
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
  rationale: "The answer is a namespace's operational history — who granted or lost what right, what drifted, what the bridge ran — disclosed only to its administrators. The VTC authorises the caller from its own records, so it must know from the document itself who the caller is; a proof binds the request to its `issuer` on every transport, and a bearer session proves nothing about the document."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no effect to replay, but an issue time dates the request for a VTC that audits administrative reads."
sideEffects:
  level: none
  rationale: "Reads the VTC's audit rows and job queue; changes nothing."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns, for administered namespaces only, a timeline of rights changes, drift reports and bridge jobs — actor and subject DIDs where the audit row still carries them, resources, rights and machine-readable qualifiers."
retention:
  class: exchange
  rationale: "The request carries at most a namespace and a paging cursor, and is needed only to answer it."
errorCodes:
  - code: git-ns/activity/list:notAdministrator
    meaning: "The caller administers no namespace — they hold neither the community-administrator capability nor a live, explicitly recorded `git.ns.admin` — or `namespace` names one they do not administer or one this VTC does not have. The three are answered alike, so the code cannot be used to learn which namespaces exist."
    retryable: false
related:
  - git-ns/right/list
  - git-ns/bridge/job/list
  - git-ns/namespace/list
  - git-ns/repo/list
  - git-ns/view
---

## Abstract

A namespace's audit rows and its bridge's jobs each answer a narrow question — [`git-ns/right/list`](../../../right/list/0.1/spec.md) shows what is granted *now*, [`git-ns/bridge/job/list`](../../../bridge/job/list/0.1/spec.md) shows what the bridge is doing *now* — but neither shows what changed and when, in the order it happened. This task interleaves both into one timeline: rights granted or revoked, drift reported, and bridge jobs, newest first. It is an admin console's *Activity* feed, a command-line `activity`.

Unlike the rest of the community-administrator-only reads in this family, this one is for any namespace's administrators — an ordinary namespace admin who is not a community administrator still needs to see their own namespace's history, and this task narrows the answer to exactly the namespaces they administer.

It is a read. It changes nothing.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

Not consequential, and declared for clarity, following [`git-ns/namespace/list`](../../../namespace/list/0.1/spec.md), [`git-ns/repo/list`](../../../repo/list/0.1/spec.md) and [`git-ns/bridge/job/list`](../../../bridge/job/list/0.1/spec.md), whose Authorization sections this one restates for activity. The caller is the DID the document's verified `proof` binds to its `issuer`; an unsigned document is refused with `proofRequired`.

The entitlement is, for each namespace, **the community-administrator capability** in the VTC's own access control, or **`git.ns.admin` on that namespace** held by explicit record and live at the instant the VTC evaluates the request, by a current member. An activity item belongs to the namespace its audit row or job names, so this task scopes exactly as the namespace, repository and job listings do: a namespace admin sees the activity of their own namespaces, and a community administrator sees every namespace's activity, including a community-wide audit row that names no namespace (one recorded before the namespace it concerned was unbound). A caller who administers no namespace is refused with `git-ns/activity/list:notAdministrator`, and so is a caller naming, in `namespace`, one they do not administer or one the VTC does not have.

The capability and `git.ns.admin` are read from the VTC's own records at execution time; the `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

**Administered namespace** — as in [`git-ns/namespace/list`](../../../namespace/list/0.1/spec.md): for a holder of the community-administrator capability, every namespace bound to the VTC; for anyone else, each namespace on which they hold a live, explicitly recorded `git.ns.admin`.

**Activity item** — one audit row or one bridge job, restated as a single timeline entry. An audit-sourced item's `at` is the row's own timestamp; a job-sourced item's `at` is when a bridge accepted it, or when it was queued if never accepted. `actor` and `subject` are absent on an audit item exactly when a right-to-be-forgotten erasure has removed them from the underlying row, and are always absent on a job item, which the VTC's queue does not attribute to an actor.

## Request

The administrator sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Identifies the caller from the verified `proof` ([Authorization](#authorization)), refusing an unsigned document with `proofRequired`.
2. Determines the caller's administered namespaces from its own records. Refuses with `git-ns/activity/list:notAdministrator` a caller who has none and does not hold the community-administrator capability, and a caller whose `namespace` is not one of them.
3. Returns `items`: every audit row and bridge job in an administered namespace — or in `namespace` only, when given — restated as `ActivityItem`s, newest first.
4. Returns at most `limit` items (clamped to 1..=500, default 100) and, when more match, a `nextCursor` to continue.

A VTC **MUST NOT** return an item outside the caller's administered namespaces, and **MUST NOT** change anything on this task.

### Carol checks recent activity in her namespace

```json
{
  "id": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f7001",
  "type": "https://trusttasks.org/spec/git-ns/activity/list/0.1",
  "threadId": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f7001",
  "issuer": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-26T09:35:00Z",
  "payload": {
    "namespace": "ns_01J8Z6Q4M2",
    "limit": 20
  }
}
```

## Response

The VTC answers with the matching items, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`: `proofRequired`, or `git-ns/activity/list:notAdministrator`.

### What Carol sees

A grant, a rename with its old name in `detail`, and a bridge job's queue state — a right-to-be-forgotten erasure has already removed the grant's `actor` and `subject`.

```json
{
  "id": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f7002",
  "type": "https://trusttasks.org/spec/git-ns/activity/list/0.1#response",
  "threadId": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f7001",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
  "issuedAt": "2026-09-26T09:35:01Z",
  "payload": {
    "items": [
      {
        "at": "2026-09-26T08:30:00Z",
        "action": "gitNs.job.bootstrap",
        "source": "job",
        "namespace": "ns_01J8Z6Q4M2",
        "resource": "github.com/acme/widgets",
        "detail": "failed"
      },
      {
        "at": "2026-09-25T16:00:00Z",
        "action": "gitNs.repo.renamed",
        "source": "audit",
        "namespace": "ns_01J8Z6Q4M2",
        "resource": "github.com/acme/widgets",
        "detail": "gizmos"
      },
      {
        "at": "2026-09-23T09:50:00Z",
        "action": "gitNs.right.granted",
        "source": "audit",
        "namespace": "ns_01J8Z6Q4M2",
        "resource": "github.com/acme/widgets",
        "right": "git.repo.own"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries at most a namespace and a paging cursor. The response carries, for administered namespaces only: a timeline of what changed — rights granted or revoked, repositories renamed or archived, drift reported, bridge jobs run — with actor and subject DIDs where the underlying audit row still carries them, and a bounded `detail` qualifier. `detail` is deliberately narrow: a machine-readable qualifier, never free-form prose, so a producer cannot use it to smuggle in personal data the rest of the schema does not model.

### Correlation

Interleaving audit rows and job state into one timeline is itself the point of this task, and its principal privacy property. A namespace admin who is not a community administrator sees only their own namespaces' history — the same boundary [`git-ns/namespace/list`](../../../namespace/list/0.1/spec.md) and [`git-ns/repo/list`](../../../repo/list/0.1/spec.md) draw — and a right-to-be-forgotten erasure that has removed `actor`/`subject` from an audit row removes them from this timeline identically: this task reads the same rows, not a separate copy that could retain what the erasure removed.

The **VTC** declares `identifierScope: public`; the **administrator** `pairwise`, matching the rest of the read family. `namespace`, `resource` and `right` are already visible to a namespace's administrators through the tasks that produced each row ([`git-ns/right/grant`](../../../right/grant/0.3/spec.md), [`git-ns/bridge/job`](../../../bridge/job/0.4/spec.md)); this task adds the ordering and the interleaving, not a new identifier.

### Retention

The request is kept only as long as the VTC keeps request logs; nothing in it needs to outlive the exchange. A VTC that audits administrative reads **MAY** record that the caller listed activity, and **SHOULD NOT** record the response, whose items are the VTC's own durable audit rows and job records under their own retention.

### Consent/purpose

The purpose is operational awareness: letting a namespace's administrators see what changed recently without correlating an audit export and a job queue by hand. A caller **MUST NOT** republish `actor`, `subject` or `detail` outside the community's own administration.
