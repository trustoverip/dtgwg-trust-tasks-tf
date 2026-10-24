---
slug: git-ns/namespace/list
version: "0.2"
title: "Git Namespaces — List Namespaces"
summary: "An administrator lists the namespaces they administer — every one, for a community administrator — with each one's admins, repository count, bridge, role map and what the bridge last reported about the forge."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - namespaces
  - administration
  - bridge
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
  rationale: "Returns, for administered namespaces only, the DIDs of their admins, of who bound them and of their bridge; forge owner names and ids; and the bridge's report on its forge app and installation."
retention:
  class: exchange
  rationale: "The request carries at most one namespace identifier, and is needed only to answer it."
errorCodes:
  - code: git-ns/namespace/list:notAdministrator
    meaning: "The caller administers no namespace — they hold neither the community-administrator capability nor a live, explicitly recorded `git.ns.admin` — or `namespace` names one they do not administer or one this VTC does not have. The three are answered alike, so the code cannot be used to learn which namespaces exist."
    retryable: false
related:
  - git-ns/view
  - git-ns/repo/list
  - git-ns/namespace/bind
  - git-ns/namespace/unbind
  - git-ns/namespace/reseat
  - git-ns/roles/reproject
  - git-ns/bridge/event
---

## Abstract

A namespace's administrators need to see more about it than a member does: who else administers it, whether it has lost its last admin, whether its bridge can still reach the forge, which permissions the forge owner has not yet granted the community's app, and which role map the bridge applies. [`git-ns/view`](../../../view/0.5/spec.md) carries none of this — its `namespaces` are the binding and nothing else — because none of it is a member's business. This task is the administrator's list of their namespaces with that operational status: an admin console's *Namespaces* card, a command-line `namespace list`.

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

The entitlement is, for each namespace, **the community-administrator capability** in the VTC's own access control (the one [`git-ns/namespace/bind`](../../../namespace/bind/0.1/spec.md) requires), or **`git.ns.admin` on that namespace** held by explicit record and live at the instant the VTC evaluates the request, by a current member. These are the namespace's administrators, and the only people this task answers. Nothing else suffices: not `git.repo.own` or `git.repo.create`, not an administrator role in the VTC's access control scoped to less than the whole community, and not membership. A caller who administers no namespace is refused with `git-ns/namespace/list:notAdministrator`, and so is a caller naming, in `namespace`, a namespace they do not administer or one the VTC does not have.

The capability and `git.ns.admin` are read from the VTC's own records at execution time; the `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). Nothing in the payload names or widens whose namespaces are listed.

## Definitions

**Administered namespace** — for a holder of the community-administrator capability, every namespace bound to the VTC, `pending` ones included. For anyone else, each namespace on which they hold a live, explicitly recorded `git.ns.admin`.

**Headless** — a bound namespace with no live `git.ns.admin`: its last admin left the community or their right lapsed. Nobody can grant in it until a community administrator reseats one with [`git-ns/namespace/reseat`](../../../namespace/reseat/0.3/spec.md).

**Forge status** — what the bridge serving the namespace last reported about its standing on the forge owner. It is the bridge's report, carried for display; the VTC takes no decision on it.

**Role map** — as in [`git-ns/bridge/event`](../../../bridge/event/0.3/spec.md): the forge role each repository right projects to, as the bridge serving the namespace last reported it.

## Request

The administrator sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Identifies the caller from the verified `proof` ([Authorization](#authorization)), refusing an unsigned document with `proofRequired`.
2. Determines the caller's administered namespaces from its own records. Refuses with `git-ns/namespace/list:notAdministrator` a caller who has none and does not hold the community-administrator capability, and a caller whose `namespace` is not one of them — for a namespace that does not exist as for one they do not administer.
3. Returns `namespaces`: every administered namespace, or only `namespace` when it is given, each as a `NamespaceStatus`, ordered by `resource`. A holder of the community-administrator capability on a VTC with no namespace receives an empty array.

A VTC **MUST NOT** return a namespace the caller does not administer, and **MUST NOT** change anything on this task.

`admins` lists the explicit holders of `git.ns.admin`; the community-administrator capability is not a git right, and its holders are not listed there. `roleDrift` and `cascadeOnDeparture` are the community's policy settings in effect; they are the same on every namespace and are repeated so that each entry stands alone.

### Carol lists her namespaces

Carol holds `git.ns.admin` on `github.com/acme` and nothing else.

```json
{
  "id": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6a01",
  "type": "https://trusttasks.org/spec/git-ns/namespace/list/0.2",
  "threadId": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6a01",
  "issuer": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-26T09:00:00Z",
  "payload": {}
}
```

## Response

The VTC answers with the administered namespaces, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`: `proofRequired`, or `git-ns/namespace/list:notAdministrator`.

### What Carol sees

The community also binds `codeberg.org/acme`, which Carol does not administer, so it is not listed. The bridge last reported that the app has not been granted one permission it needs.

```json
{
  "id": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6a02",
  "type": "https://trusttasks.org/spec/git-ns/namespace/list/0.1#response",
  "threadId": "urn:uuid:7a2e4c10-3b5d-4f6e-9a1b-2c3d4e5f6a01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
  "issuedAt": "2026-09-26T09:00:01Z",
  "payload": {
    "namespaces": [
      {
        "id": "ns_01J8Z6Q4M2",
        "forge": "github.com",
        "owner": "acme",
        "resource": "github.com/acme",
        "mode": "bridge",
        "state": "bound",
        "kind": "organization",
        "ownerId": "4411023",
        "bridgeDid": "did:webvh:QmBridgeScid9:bridge.acme.example",
        "boundBy": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
        "requestedAt": "2026-09-23T09:40:00Z",
        "boundAt": "2026-09-23T09:42:10Z",
        "admins": [
          "did:webvh:QmCarolScid3:acme-vtc.example:carol"
        ],
        "repoCount": 2,
        "headless": false,
        "installationRemoved": false,
        "forgeStatus": {
          "installationId": "55120034",
          "appName": "Acme VTC",
          "appSlug": "acme-vtc",
          "missingPermissions": ["administration:write"],
          "permissionUpgradePending": true,
          "orgRulesets": true,
          "requiredWorkflow": true,
          "reportedAt": "2026-09-26T08:30:00Z"
        },
        "roleDrift": "report",
        "cascadeOnDeparture": false,
        "roleMap": {
          "own": "admin",
          "maintain": "maintain",
          "commit": "write"
        },
        "roleMapSource": "reported",
        "roleMapReportedAt": "2026-09-23T09:42:30Z"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries at most a namespace identifier. The response carries, for administered namespaces only: the DIDs of their admins, of who bound them and of their bridge; forge owner names and ids; the community's policy settings; and the bridge's report on its forge app and installation, including permissions the app lacks. Most of it is operational: it tells an administrator what to fix. Some of it — `missingPermissions`, `installationRemoved` — also tells a reader where the namespace's automation is weak, which is why it goes to nobody but the namespace's administrators.

### Who sees what

A namespace admin sees their own namespaces and nothing of the others: two namespaces in one community can belong to different teams, and a team's admin is accountable for their own. A holder of the community-administrator capability sees every namespace, because they are accountable for the VTC and bind and reseat namespaces. A VTC **MUST NOT** answer this task on the strength of a bearer session, or of an administrator role scoped to less than the whole community: before this task, VTCs served this list to any admin session, and every context-scoped administrator could read every namespace's status.

`notAdministrator` is returned alike for a namespace that does not exist and for one the caller does not administer, so the refusal is not an oracle for which namespaces a community has.

### Correlation

The VTC declares `identifierScope: public`; the administrator `pairwise`. The admin DIDs returned are already published as `git.ns.admin` tuples in the Trust Registry. The bridge's DID and the forge owner's id are not secrets, but the join of them with the forge status is what this task gates.

### Retention

The request is kept only as long as the VTC keeps request logs; nothing in it needs to outlive the exchange. A VTC that audits administrative reads **MAY** record that the caller listed namespaces, and **SHOULD NOT** record the response.

### Consent/purpose

The purpose is to let a namespace's administrators keep it working: see who else can act, notice a lost admin or a lost installation, and see which role map the forge is projected under. A caller **MUST NOT** republish the forge status outside the community.
