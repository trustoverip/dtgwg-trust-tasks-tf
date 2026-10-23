---
slug: git-ns/repo/list
version: "0.2"
title: "Git Namespaces — List Repositories"
summary: "An administrator lists the repositories in the namespaces they administer — every one, for a community administrator — with owners, right counts, bootstrap and sync status, and what the bridge last reported on each."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - repositories
  - administration
  - bootstrap
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
  rationale: "The answer is operational detail about namespaces — who administers them, what the bridge reported, what failed — that is disclosed only to their administrators. The VTC authorises the caller from its own records, so it must know from the document itself who the caller is; a proof binds the request to its `issuer` on every transport, and a bearer session proves nothing about the document."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no effect to replay, but an issue time dates the request for a VTC that audits administrative reads."
sideEffects:
  level: none
  rationale: "Reads the VTC's records and changes nothing: no audit row, counter or last-seen time is written because of it."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns, for repositories in administered namespaces only, owner and creator DIDs, counts of recorded rights, and the bridge's report: bootstrap steps, the last check run, the guard in force and the last error."
retention:
  class: exchange
  rationale: "The request carries at most one namespace identifier, and is needed only to answer it."
errorCodes:
  - code: git-ns/repo/list:notAdministrator
    meaning: "The caller administers no namespace — they hold neither the community-administrator capability nor a live, explicitly recorded `git.ns.admin` — or `namespace` names one they do not administer or one this VTC does not have. The three are answered alike, so the code cannot be used to learn which namespaces exist."
    retryable: false
related:
  - git-ns/view
  - git-ns/namespace/list
  - git-ns/repo/create
  - git-ns/repo/adopt
  - git-ns/roles/reproject
  - git-ns/drift/resolve
  - git-ns/bridge/result
---

## Abstract

An administrator's repository table shows more than a member's: how many people hold each right, which bootstrap step failed and what the bridge said about it, which guard stops a pull request from satisfying its own check, when the check last ran, and whether the repository is still projected under an old role map. [`git-ns/view`](../../../view/0.5/spec.md) returns a repository's owners, bootstrap and drift, because every member may see those; the rest is operational detail for the people who fix it. This task is the administrator's list of the repositories in the namespaces they administer, with that status: an admin console's *Repos* table, a command-line `repos`.

It is a read. It changes nothing, grants nothing and publishes nothing.

## Changes from 0.1

- **Nested repository paths.** A repository resource is `<forge-host>/<owner>/<repo-path>`. On a forge with nested groups (GitLab) the repository path holds the subgroups between the top-level owner and the project, up to 21 segments: `gitlab.com/acme/platform/tools/cli` is one repository of the namespace `gitlab.com/acme`. The namespace is still the top-level owner alone, and a subgroup is not a resource, so no right is held on one. A segment may be up to 255 characters.
- The schema pins [`git-ns/_shared/0.6`](../../../_shared/0.6/git-ns.schema.json) (was `0.4`).

Accepting what `0.1` refused is a backwards-compatible change, released as a `MINOR` increment ([SPEC §5.2](/SPEC.md#52-compatibility-rules)); a `0.1` document is a valid `0.2` document. A producer that also serves `0.1` **MUST NOT** put a resource `0.1` cannot carry — a repository path of more than one segment, or a segment longer than 100 characters — into a `0.1` document: a `0.1` consumer validates against the `0.1` schema and would refuse the whole document. It leaves such repositories, and the rights on them, out of a `0.1` answer instead. Everything else is unchanged from `0.1` and restated below, so that this version stands on its own.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

Not consequential, and declared for clarity. The caller is the DID the document's verified `proof` binds to its `issuer`, or the DID a key the VTC has recorded as delegated by that DID acts for; an unsigned document is refused with `proofRequired`.

The entitlement is, for each namespace, **the community-administrator capability** in the VTC's own access control (the one [`git-ns/namespace/bind`](../../../namespace/bind/0.1/spec.md) requires), or **`git.ns.admin` on that namespace** held by explicit record and live at the instant the VTC evaluates the request, by a current member. These are the namespace's administrators, and the only people this task answers. Nothing else suffices: not `git.repo.own` or `git.repo.create`, not an administrator role in the VTC's access control scoped to less than the whole community, and not membership. A caller who administers no namespace is refused with `git-ns/repo/list:notAdministrator`, and so is a caller naming, in `namespace`, a namespace they do not administer or one the VTC does not have.

The capability and `git.ns.admin` are read from the VTC's own records at execution time; the `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). Nothing in the payload names or widens whose namespaces are listed.

## Definitions

**Administered namespace** — for a holder of the community-administrator capability, every namespace bound to the VTC, `pending` ones included. For anyone else, each namespace on which they hold a live, explicitly recorded `git.ns.admin`.

**Recorded right** — a live right record, never an implied one ([the rights model](../../../right/grant/0.3/spec.md)). `maintainers` and `committers` count records of `git.repo.maintain` and `git.commit.sign`; an owner's implied rights are not counted.

**Bridge report** — `steps`, `lastCheck`, `guard`, `failedStep` and `lastError` are what the bridge serving the namespace last reported for the repository ([`git-ns/bridge/result`](../../../bridge/result/0.1/spec.md)). They are carried for display; the VTC takes no decision on them.

## Request

The administrator sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Identifies the caller from the verified `proof` ([Authorization](#authorization)), refusing an unsigned document with `proofRequired`.
2. Determines the caller's administered namespaces from its own records. Refuses with `git-ns/repo/list:notAdministrator` a caller who has none and does not hold the community-administrator capability, and a caller whose `namespace` is not one of them — for a namespace that does not exist as for one they do not administer.
3. Returns `repos`: every repository it records in an administered namespace — or in `namespace` only, when given — in any state, `unmanaged` ones included, each as a `RepoStatus`, ordered by `resource`. To a holder of the community-administrator capability who gave no `namespace` it also returns every repository whose namespace is no longer bound: nobody else administers those.

A VTC **MUST NOT** return a repository outside the caller's administered namespaces, and **MUST NOT** change anything on this task. The drift items themselves are not in this answer; `driftCount` says how many there are, and `git-ns/view` returns them.

### Carol lists the repositories in `acme`

```json
{
  "id": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6a11",
  "type": "https://trusttasks.org/spec/git-ns/repo/list/0.2",
  "threadId": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6a11",
  "issuer": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-26T09:05:00Z",
  "payload": {
    "namespace": "ns_01J8Z6Q4M2"
  }
}
```

## Response

The VTC answers with the repositories, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`: `proofRequired`, or `git-ns/repo/list:notAdministrator`.

### What Carol sees

```json
{
  "id": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6a12",
  "type": "https://trusttasks.org/spec/git-ns/repo/list/0.1#response",
  "threadId": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6a11",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
  "issuedAt": "2026-09-26T09:05:01Z",
  "payload": {
    "repos": [
      {
        "id": "repo_01J8Z7A1B2",
        "namespace": "ns_01J8Z6Q4M2",
        "resource": "github.com/acme/gadgets",
        "forgeId": "812736990",
        "visibility": "public",
        "state": "active",
        "owners": ["did:webvh:QmBobScid2:acme-vtc.example:bob"],
        "maintainers": 0,
        "committers": 1,
        "bootstrap": {
          "workflow": true,
          "keyring": true,
          "variables": true,
          "requiredCheck": true
        },
        "syncState": "inSync",
        "checkedAt": "2026-09-26T08:58:00Z",
        "driftCount": 0,
        "createdBy": "did:webvh:QmBobScid2:acme-vtc.example:bob",
        "createdAt": "2026-09-23T10:05:00Z",
        "guard": "requiredWorkflow",
        "steps": [
          { "step": "workflow", "outcome": "unchanged" },
          { "step": "requiredCheck", "outcome": "applied" }
        ],
        "lastCheck": {
          "conclusion": "success",
          "at": "2026-09-26T08:40:12Z",
          "sha": "9f2c4e1ab37d"
        },
        "roleMap": {
          "own": "admin",
          "maintain": "maintain",
          "commit": "write"
        },
        "roleMapStale": false
      },
      {
        "id": "repo_01J8Z7A1B1",
        "namespace": "ns_01J8Z6Q4M2",
        "resource": "github.com/acme/widgets",
        "forgeId": "812736451",
        "visibility": "public",
        "state": "active",
        "owners": [
          "did:webvh:QmAliceScid1:acme-vtc.example:alice",
          "did:webvh:QmCarolScid3:acme-vtc.example:carol"
        ],
        "maintainers": 1,
        "committers": 4,
        "bootstrap": {
          "workflow": true,
          "keyring": true,
          "variables": true,
          "requiredCheck": false
        },
        "syncState": "drift",
        "checkedAt": "2026-09-26T08:58:00Z",
        "driftCount": 1,
        "failedStep": "requiredCheck",
        "lastError": "the installation lacks administration:write",
        "createdAt": "2026-09-23T09:50:00Z",
        "guard": "none",
        "steps": [
          { "step": "requiredCheck", "outcome": "failed", "detail": "the installation lacks administration:write" }
        ],
        "roleMap": {
          "own": "admin",
          "maintain": "maintain",
          "commit": "write"
        },
        "roleMapStale": false
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries at most a namespace identifier. The response carries, for repositories in administered namespaces only: owner and creator DIDs, counts of recorded rights, forge repository ids, and the bridge's report — bootstrap steps and their errors, the last check run and the commit it ran on, and the guard in force. A failed step's error can name forge permissions the community's app lacks, and `guard: none` says a repository's pull requests can satisfy their own check; both tell a reader where the repository is weakest, which is why they go to nobody but its namespace's administrators.

### Who sees what

A namespace admin sees the repositories in their own namespaces and nothing of the others. A holder of the community-administrator capability sees every repository, including those left behind by an unbound namespace, which only they can act on by binding again. The owners of a repository are visible to every member through `git-ns/view`; the rest of an entry is not, and a repository's owner who does not administer its namespace reads their repository through `git-ns/view` rather than here. A VTC **MUST NOT** answer this task on the strength of a bearer session, or of an administrator role scoped to less than the whole community.

`notAdministrator` is returned alike for a namespace that does not exist and for one the caller does not administer.

### Correlation

The VTC declares `identifierScope: public`; the administrator `pairwise`. Owner DIDs are already published as `git.repo.own` tuples in the Trust Registry; the counts disclose how many hold a right without saying who.

### Retention

The request is kept only as long as the VTC keeps request logs; nothing in it needs to outlive the exchange. A VTC that audits administrative reads **MAY** record that the caller listed repositories, and **SHOULD NOT** record the response.

### Consent/purpose

The purpose is to let a namespace's administrators keep its repositories trusted: see which bootstrap steps are missing and why, which repositories have drifted, and which still need re-projecting. A caller **MUST NOT** republish bridge errors or guard status outside the community.
