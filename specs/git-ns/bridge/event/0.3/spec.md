---
slug: git-ns/bridge/event
version: "0.3"
title: "Git Namespaces — Bridge Event"
summary: "The bridge tells the VTC what happened on the forge — renames, transfers, unmanaged or deleted repositories, role and protection changes, lost access, completed links and bindings — with the drift it now sees, and reports the role map it projects rights with."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - bridge
  - webhook
  - drift
  - role-map
parties:
  - role: bridge
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTC
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "Events move rights after renames and complete namespace bindings and account links. The VTC must be able to attribute each to the bridge that serves the namespace on every transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "An event replayed later would report a forge state that no longer holds."
sideEffects:
  level: mutating
  rationale: "Updates the VTC's record of the forge — repository names and states, drift, pending bindings and links, the role map the bridge projects with — and may cause the VTC to move or withdraw published rights, or to re-project forge roles."
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: "Events carry forge account ids and logins, which identify people and which the VTC can join to members' DIDs."
retention:
  class: durable
  rationale: "Events are the history of what happened on the forge outside the VTC's control."
errorCodes:
  - code: git-ns:unknownNamespace
    meaning: "No namespace bound to this VTC has this identifier, or contains this resource."
    retryable: false
related:
  - git-ns/bridge/job
  - git-ns/bridge/result
  - git-ns/namespace/bind
  - git-ns/account/link
  - git-ns/repo/adopt
  - git-ns/drift/resolve
  - git-ns/roles/reproject
---

## Abstract

The bridge tells the VTC that something happened on the forge — translated from the forge's webhook, or found by an inspection sweep where the forge has no webhooks — together with any **drift** it now sees between the forge and the VTC's projection. Events are forge-neutral: the adapter translates each forge's webhooks into the nine forge types below, so the VTC never parses a forge payload. One more type, `roleMapReported`, is not about the forge at all: it is the bridge telling the VTC which forge role each right is projected to, so that what the VTC shows and decides about forge roles is what the bridge actually does.

The VTC is the source of truth, so an event never changes a right directly. It changes what the VTC knows about the forge, and the VTC decides what follows: rewrite rights after a rename, list an unmanaged repository for adoption, re-apply a weakened protection, alert the namespace admins.

## Changes from 0.2

One event type is added, and nothing else in an event's meaning changes.

1. **`roleMapReported`: the bridge reports the role map it projects with.** A bridge maps each right in a [`git-ns/bridge/job`](../../../../git-ns/bridge/job/0.3/spec.md) `desiredRoles` entry to a forge role, and the map is the community's to configure per bridge, forge, namespace and repository (a Forgejo community may give maintainers `admin`; a repository may give committers `write`). In `0.2` nothing told the VTC what map was in force, so it could only assume the default: it showed operators forge roles the bridge did not give, and [`git-ns/drift/resolve`](../../../../git-ns/drift/resolve/0.2/spec.md) derived the right to adopt from a map that was not the bridge's — adopting a forge `admin` as `git.repo.own` on a repository where `admin` is what `git.repo.maintain` projects to. The bridge now reports, per namespace, the map as the forge applies it, each repository whose map differs, and each repository whose roles were projected under a map it no longer applies (`stale`) — which a change of configuration otherwise leaves in place until some unrelated right changes.
2. **The report is how the VTC learns the mapping `git-ns/drift/resolve` inverts.** Every version of that task defines the *projected right of a role* as the inverse of the mapping the bridge applies to `desiredRoles`, without saying how the VTC knows it. This version says how, and which right is the inverse when two rights share a role (request step 5.4).
3. **Without a report the map is unknown, and nothing is derived from a guess.** A VTC used to assume the default map, so after a binding, or after the namespace came to be served by another bridge, it adopted a forge `admin` as `git.repo.own` whatever the bridge actually gave owners. A VTC now holds a map only when the bridge serving the namespace reports one — the default included. Until then it refuses an adoption with `git-ns:roleMapUnknown`, and weighs a revert as the gravest it could be (request step 5.7). [`git-ns/drift/resolve`](../../../../git-ns/drift/resolve/0.2/spec.md) 0.1 and 0.2 now declare the code and state the conservative weighing. Nothing else in them changes.
4. **The newest report wins.** Reports are ordered by `issuedAt`. A VTC ignores a report issued before the one it holds from the same bridge, and a bridge builds every report afresh, a resend included, so that a later `issuedAt` always carries the later map (request step 5.2).
5. **A report names the forge's ladder.** `ladder` lists the levels the forge offers in the namespace, and a VTC refuses a map with a role that is not on it — `own: admin` on a GitHub personal account, or `triage` on Forgejo (request step 5.1).
6. **The VTC re-projects stale repositories.** A map change reaches a repository only at its next `projectRoles`. A VTC **SHOULD** send one for each repository in `stale`, with its complete `desiredRoles`, on receiving the report ([Request](#request), step 5). Re-projecting changes no right: it asks the bridge to apply, now, what it would apply at the next projection anyway.

The schema pins [`git-ns/_shared/0.3`](../../../_shared/0.3/git-ns.schema.json). The shapes an event carries are identical in `_shared/0.2` and `0.3` (`0.3` narrows only `Did`, which no event carries). A `0.2` document is a valid `0.3` document with the same meaning. Adding an event type is released as a `MINOR` increment under the `draft` allowance of [SPEC §5.2](/SPEC.md#52-compatibility-rules), because a VTC implementing only `0.2` refuses the new type. Everything else is restated unchanged below, so that this version stands on its own.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **being the bridge that serves the namespace**, and it extends no further than that namespace. A VTC **MUST** refuse, with `permissionDenied`, an event from any other issuer, and an event that names a repository outside the namespace in its `namespace` member ([Request](#request)). The bridge's own authority on the forge — the app installation, the bot's organisation membership — is what lets it observe; the VTC trusts its report of what it observed and nothing more. In particular a `bindCompleted` or `accountLinked` event is accepted only for a `begin*` job the VTC sent this bridge and has not yet seen completed. A `roleMapReported` event is the bridge's statement of what it does with the rights the VTC sends it — which the bridge, holding the forge credentials, decides in any case — so the VTC takes it as that and as nothing more: it never grants, revokes or re-keys a right on the strength of one.

## Definitions

**`event.type`** — one of:

| Type | Means | The VTC |
|---|---|---|
| `repoRenamed` | a managed repository was renamed | moves every right keyed by `forgeId` to `to`, withdraws the tuples published for `from` before publishing any for `to`, and never lets a later repository at `from` inherit them |
| `repoTransferred` | a managed repository moved to another forge owner, and so out of this namespace | sets the repository `detached` and withdraws its rights, **wherever `to` is**, including another namespace this VTC governs; never moves a right |
| `repoCreatedUnmanaged` | a repository exists in the namespace that the VTC did not create or adopt, including one transferred in from elsewhere | if the VTC records a repository at `resource` under a different forge id, first sets that repository `detached` and withdraws its rights; then records the new one `unmanaged`, for a namespace admin to adopt |
| `repoDeleted` | a repository was deleted | sets it `detached` and withdraws its rights |
| `roleChanged` | someone's forge role changed outside the bridge | records drift; by policy, re-projects or reports |
| `protectionChanged` | branch protection, a ruleset, or the namespace's required workflow changed outside the bridge | records drift; by policy, re-applies (the default when the required check is no longer required) and alerts the namespace admins |
| `installationRemoved` | the bridge lost its access to the namespace | marks every repository's sync `unchecked` and alerts the namespace admins; rights are unaffected |
| `accountLinked` | a member completed a `beginAccountLink` job | completes the link begun with [`git-ns/account/link`](../../../../git-ns/account/link/0.1/spec.md) |
| `bindCompleted` | the forge-side proof for a `beginBind` job arrived | completes the binding begun with [`git-ns/namespace/bind`](../../../../git-ns/namespace/bind/0.1/spec.md) |
| `roleMapReported` | the bridge reports the role map it projects rights with in the namespace — whenever it starts serving the namespace, whenever it (re)establishes its link to the VTC, and whenever the map changes | replaces the map it holds for the namespace, unless it holds a later report from the same bridge; uses it wherever it shows or derives a forge role; re-projects each repository in `stale` |

**Role map** — the forge role each repository right is given on a repository, **as the forge applies it**: the role the bridge's configured map asks for, rounded down onto the forge's ladder for this namespace (below). Roles are levels of the bridge's forge-neutral ladder, lowest first — `none`, `read`, `triage`, `write`, `maintain`, `admin` — the same vocabulary a role drift item's `observed` and `expected` use. A map has exactly three members, `own`, `maintain` and `commit`, and is **ordered**: `own` ≥ `maintain` ≥ `commit`, and `commit` ≤ `write`. It has **no member for `git.ns.admin`** or `git.repo.create`: a namespace admin projects to no forge role ([`git-ns/bridge/job`](../../../../git-ns/bridge/job/0.3/spec.md), `desiredRoles`), and no bridge configuration can change that. Where a level is more than a forge role — Forgejo's `maintain` is `write` on the repository and a place on the merge allow-list — the map reports the level, and role drift is reported in the same levels.

**Ladder** — the levels a forge offers as a direct role on a repository in a namespace, lowest first. `none` is not listed: every forge has it. The ladder depends on the forge and on the namespace's kind:

| Forge and kind | `ladder` |
|---|---|
| GitHub organisation | `read`, `triage`, `write`, `maintain`, `admin` |
| GitHub personal account | `write` (a collaborator has only one level) |
| Forgejo, either kind | `read`, `write`, `maintain`, `admin` (no `triage`; `maintain` is the adapter's own level — `write` plus the default branch's merge allow-list) |

A forge adapter not listed here defines its own ladder from the same levels. Every role in a map is `none` or a level of the namespace's ladder, because a map is reported as the forge applies it.

**Default role map** — `own` → `admin`, `maintain` → `maintain`, `commit` → `none`, rounded down onto the forge's ladder as above. A bridge applies it where its configuration names no map. **A VTC never assumes it.** It holds the default only when the bridge reports the default.

**Unknown role map** — the VTC's state for a namespace when it holds no report from the bridge that serves the namespace now. This is the case from binding until the bridge's first report, and from the moment the namespace is served by a bridge with a different DID until that bridge reports. It is also the case on a namespace whose bridge implements only `0.1` or `0.2`, which never reports. When a later report is lost in transit, the VTC keeps the one before it, from the same bridge, until the next arrives. That report is still the bridge's latest statement the VTC holds, so the map is not unknown.

**`roleMap`, `ladder`, `repos`, `stale`** — in a `roleMapReported` event:

- `roleMap`: the map the bridge applies to every repository in the namespace not listed in `repos`;
- `ladder`: the namespace's ladder, as the forge offers it;
- `repos`: each repository whose own map differs, with that map;
- `stale`: each repository whose forge roles the bridge last projected under a different map from the one it now applies to it.

A repository is matched by `resource`, so a repository renamed on the forge takes the map of its new name.

**`drift`** — the complete outstanding drift for each repository the event concerns, replacing what the VTC held for them. A repository whose drift is now empty is back in sync.

## Request

The bridge sends the event to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming bridge verifies the forge's webhook signature before translating it and **MUST NOT** report an event from an unverified webhook. It **MUST NOT** report its own namespace-level repositories — a workflow repository it created when the namespace was bound — as `repoCreatedUnmanaged`; they are part of the binding, not repositories to govern. A repository transferred *into* the namespace is reported as `repoCreatedUnmanaged`, never as `repoTransferred`, whose `from` would lie outside it.

A conforming bridge sends `roleMapReported` for a namespace **whenever it starts serving it** — when it starts, and when a namespace becomes its to serve (a binding completing, after `bindCompleted`, or the namespace being handed to it) — **whenever it establishes or re-establishes its link to the VTC** (a reconnection, a new session, a transport it had lost coming back), and **whenever the role map it applies there changes**. A VTC may have lost or never seen an earlier report, or may have been told of another bridge's map in between; sending at each of these moments means the VTC's view is never older than the link it holds with this bridge. It reports the map **as the forge applies it**, rounded onto the forge's ladder, and reports that ladder in `ladder` ([Definitions](#definitions)). It lists in `repos` only repositories whose map differs from `roleMap`. It lists in `stale` every repository it manages whose forge roles it last projected under a map other than the one it now applies to it. A bridge that cannot tell under which map a repository was projected — one projected before the bridge kept that record — **SHOULD** take it to be the default role map. A bridge **MUST NOT** report a map that rounding leaves unordered (a defective adapter): it sends no report for the namespace, and records why for its operator.

A report with a later `issuedAt` replaces an earlier one, so the next report makes good a report lost in transit. For that to hold, `issuedAt` must order what reports say, not only when they were signed. A bridge that sends a report again because the VTC did not acknowledge it **MUST** build it afresh: the map it applies at that moment, with a new `id` and `issuedAt`. It **MUST NOT** re-sign an earlier report's content under a new `issuedAt`. A resend therefore never carries a map older than one it already reported, and the document stays inside the VTC's freshness window however long the link was down. A bridge **MAY** also send a report at any other time.

A conforming VTC:

1. Refuses a namespace it does not have with `git-ns:unknownNamespace`.
2. Refuses with `permissionDenied`, and applies no part of, an event where any of these lies outside the namespace named in `namespace`, by the whole-segment containment of [the fixed rules](../../../../git-ns/right/grant/0.1/spec.md#the-fixed-rules): the `resource` of any event type; the `from` of `repoRenamed` or `repoTransferred`; the `to` of `repoRenamed`; the `resource` of any drift item; each `repos[].resource` and each member of `stale` of `roleMapReported`. A `repoTransferred`'s `to` is exempt, since a transfer leaves the namespace by definition. The VTC records it as where the repository went and acts on nothing there. An event that crosses namespaces is either a defective bridge or one acting beyond its authority, and neither may change what the VTC governs elsewhere.
3. Applies the event as the table above says. For `repoTransferred`, it **MUST NOT** move, copy or re-key any right to `to`, even when `to` lies in another namespace bound to this VTC. For `repoCreatedUnmanaged` at a resource where it records a repository with a different `forgeId`, it **MUST** detach that repository and withdraw its rights before recording the new one. A recorded repository with no `forgeId` yet (`pendingCreate`) is not detached this way; its `createRepo` job reports the name as taken.
4. Treats a repeated event as harmless: every effect is keyed by forge id and is idempotent. A repeated `roleMapReported` replaces the map with itself.
5. For `roleMapReported`:
   1. Refuses with the framework's `malformedRequest`, and records nothing of, a report that:
      - has a map that is not ordered ([Definitions](#definitions));
      - has a `ladder` that lists `none` or is not strictly ascending;
      - has a map with a role that is neither `none` nor on `ladder`;
      - lists a repository twice in `repos`.

      A VTC that knows the namespace's ladder independently — it knows the forge at the namespace's host, and it records the namespace's kind from `bindCompleted` — **MUST** also refuse a report whose `ladder` is not that ladder. For example, it refuses `own: admin` on a GitHub personal account, and `triage` on Forgejo.
   2. Compares the report's `issuedAt` with that of the report it holds for the namespace from the same bridge. It acknowledges a report issued **before** the one it holds, and applies no part of it. Such a report is an earlier statement arriving late, and the report the VTC holds is newer. A report is never compared with one from another bridge, which is not the serving bridge's report (step 7).
   3. Otherwise, replaces the map it holds for the namespace with the report — `roleMap`, `repos` and `stale` together — and keeps the report's `issuedAt`. It **MAY** leave out of `stale` a repository that it does not record as `active` or `orphaned` in the namespace, since it re-projects no other repository (step 5).
   4. Uses the map — the entry in `repos` for a repository listed there, `roleMap` otherwise — wherever it shows the forge role a right projects to on a repository, and wherever it derives a right from a forge role: the *projected right of a role* of [`git-ns/drift/resolve`](../../../../git-ns/drift/resolve/0.2/spec.md) — which every version of that task defines as the inverse of the mapping the namespace's bridge projects `desiredRoles` with — is, given a report, the **lowest** right whose role in the map is that role, and a role that no right's is, or `none`, has none. A namespace admin still projects to no forge role, whatever the map.
   5. **SHOULD** send a `projectRoles` job, with its complete `desiredRoles`, for every repository in `stale` that it records `active` or `orphaned` in the namespace, without waiting for anyone to ask ([`git-ns/roles/reproject`](../../../../git-ns/roles/reproject/0.1/spec.md) is the same thing, asked for). Until a repository's re-projection succeeds, the VTC **SHOULD** show it as projected under an earlier role map. Re-projecting changes no right, and the map it applies was already the bridge's to apply at the next projection; what it changes is only *when* the forge comes to match — at a moment the community chose by changing the bridge's configuration, rather than at the next unrelated change of rights.
   6. **MUST NOT** carry a report over to another bridge. When the namespace comes to be served by a bridge with a different DID (a reseat of its bridge, a new bridge configured for its forge), the map is unknown until that bridge reports.
   7. While the map is **unknown** ([Definitions](#definitions)), **MUST NOT** derive anything from any map, the default included:
      - It refuses an `adopt` under every version of [`git-ns/drift/resolve`](../../../../git-ns/drift/resolve/0.2/spec.md) with `git-ns:roleMapUnknown`, before it evaluates the right. The code names the reason: the VTC cannot tell which right the observed role is the projection of. `noMatchingRight` would say that no right is.
      - It weighs the impact of reverting a role conservatively, as that of revoking the highest right the role could project from. Under some ordered map, any role but `none` could be the role `git.repo.own` projects to, so a revert that removes or lowers such a role has the impact of revoking `git.repo.own`.
      - It **SHOULD** show the map as unknown wherever it shows it, and **MUST NOT** show it as the default, or as a bridge that no longer serves the namespace reported it.

      A bridge that implements `0.3` reports when it starts serving the namespace and whenever its link comes up (above), so the state lasts until its first report arrives. On a namespace whose bridge implements only `0.1` or `0.2`, the map stays unknown. Adoption is refused there, and revert, grant and revoke still work.

### A repository renamed on the forge

```json
{
  "id": "urn:uuid:a98c01fe-2fd9-457d-ac53-a7900258b001",
  "type": "https://trusttasks.org/spec/git-ns/bridge/event/0.3",
  "threadId": "urn:uuid:a98c01fe-2fd9-457d-ac53-a7900258b001",
  "issuer": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-02T14:20:00Z",
  "payload": {
    "namespace": "ns_01J8Z6Q4M2",
    "event": {
      "type": "repoRenamed",
      "forgeId": "812736451",
      "from": "github.com/acme/widgets",
      "to": "github.com/acme/widgets-core"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example#key-1",
    "created": "2026-10-02T14:20:00Z",
    "proofPurpose": "authentication",
    "proofValue": "zX1BNqDY7HHzMVndp638Ye2Qw7s2xvZVrfbz7ZvnMWvVZHKJ1EEQHyUQGNwEB5ZhAzpCNYXntvAbAmk5tm46YaN"
  }
}
```

### A repository transferred to another organisation

`gadgets` moved from `acme` to `acme-labs`. It does not matter that `github.com/acme-labs` is also a namespace this VTC governs: the repository is detached here and its rights are withdrawn. If `acme-labs`' bridge reports it there as `repoCreatedUnmanaged`, that namespace's admins adopt it and grant afresh.

```json
{
  "id": "urn:uuid:a98c01fe-2fd9-457d-ac53-a7900258b007",
  "type": "https://trusttasks.org/spec/git-ns/bridge/event/0.3",
  "threadId": "urn:uuid:a98c01fe-2fd9-457d-ac53-a7900258b007",
  "issuer": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-05T11:00:00Z",
  "payload": {
    "namespace": "ns_01J8Z6Q4M2",
    "event": {
      "type": "repoTransferred",
      "forgeId": "812736990",
      "from": "github.com/acme/gadgets",
      "to": "github.com/acme-labs/gadgets"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example#key-1",
    "created": "2026-10-05T11:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z4Tn8Wq2Lb6Rx1Kc9Hm3Fd7Ya5Js2Vg8Ue4Zo1Ti6Pn3Xb9Cq5Dw2Ek7Sh4Gv1Ly8Au6Bj3Kf9Tr2Np5Zm7Rc"
  }
}
```

### Someone made the required check optional

```json
{
  "id": "urn:uuid:a98c01fe-2fd9-457d-ac53-a7900258b003",
  "type": "https://trusttasks.org/spec/git-ns/bridge/event/0.3",
  "threadId": "urn:uuid:a98c01fe-2fd9-457d-ac53-a7900258b003",
  "issuer": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-03T08:00:00Z",
  "payload": {
    "namespace": "ns_01J8Z6Q4M2",
    "event": {
      "type": "protectionChanged",
      "forgeId": "812736990",
      "resource": "github.com/acme/gadgets",
      "requiredCheck": false
    },
    "drift": [
      {
        "type": "requiredCheckMissing",
        "resource": "github.com/acme/gadgets",
        "observed": "Verify commit trust: not required",
        "expected": "Verify commit trust: required"
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example#key-1",
    "created": "2026-10-03T08:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "zLKQi5maEpU3bgPaMfECN14do9qszZbcEDatyWhNH5es3Pc9UYi5jmbZ7AsPdE5TQEeoj8LcV5BBYY8w3s78nVS"
  }
}
```

### A GitHub organisation installed the community's app

```json
{
  "id": "urn:uuid:a98c01fe-2fd9-457d-ac53-a7900258b005",
  "type": "https://trusttasks.org/spec/git-ns/bridge/event/0.3",
  "threadId": "urn:uuid:a98c01fe-2fd9-457d-ac53-a7900258b005",
  "issuer": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-23T10:03:00Z",
  "payload": {
    "namespace": "ns_01J8Z6Q4M2",
    "event": {
      "type": "bindCompleted",
      "jobId": "job_01J8Z6Q4N0",
      "ownerId": "91827364",
      "kind": "organization"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example#key-1",
    "created": "2026-09-23T10:03:00Z",
    "proofPurpose": "authentication",
    "proofValue": "zErKBa1j26x1aicAi862KvPr5RvRvFaKU5EtMUicEE6BTmNsRMLKgPP6sMorj7tM5Kx2rKHE7KV73KziML7f9Vb"
  }
}
```

### The bridge restarted with a new role map

The community's Forgejo bridge now gives maintainers `admin` in `acme` (full repository administration rather than `write` plus the merge allow-list), and gives committers `write` on `widgets`, which takes branch-based contributions. `gadgets` and `widgets` were projected under the old map; the VTC re-projects both.

```json
{
  "id": "urn:uuid:a98c01fe-2fd9-457d-ac53-a7900258b009",
  "type": "https://trusttasks.org/spec/git-ns/bridge/event/0.3",
  "threadId": "urn:uuid:a98c01fe-2fd9-457d-ac53-a7900258b009",
  "issuer": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-06T09:00:00Z",
  "payload": {
    "namespace": "ns_01J8Z6Q4M2",
    "event": {
      "type": "roleMapReported",
      "roleMap": { "own": "admin", "maintain": "admin", "commit": "none" },
      "ladder": ["read", "write", "maintain", "admin"],
      "repos": [
        {
          "resource": "codeberg.org/acme/widgets",
          "roleMap": { "own": "admin", "maintain": "admin", "commit": "write" }
        }
      ],
      "stale": ["codeberg.org/acme/gadgets", "codeberg.org/acme/widgets"]
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example#key-1",
    "created": "2026-10-06T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3Hq8Lr5Tn2Wb7Kx4Vd9Fm1Ys6Jc3Pg8Ue5Zo2Ti7Nn4Xb1Cq6Dw3Ek8Sh5Gv2Ly9Au7Bj4Kf1Tr3Np6Zm8Rc"
  }
}
```

A report on a GitHub organisation with the default map, where every repository's roles are current, carries only `roleMap`, `{ "own": "admin", "maintain": "maintain", "commit": "none" }`, and `ladder`, `["read", "triage", "write", "maintain", "admin"]`. On a GitHub personal account the same configuration is reported as `{ "own": "write", "maintain": "write", "commit": "none" }` with `ladder` `["write"]`, the only collaborator level there. The projected right of `write` is then `git.repo.maintain`, the lower of the two.

## Response

The VTC acknowledges the event with an empty payload, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`.

### Recorded

```json
{
  "id": "urn:uuid:a98c01fe-2fd9-457d-ac53-a7900258b002",
  "type": "https://trusttasks.org/spec/git-ns/bridge/event/0.3#response",
  "threadId": "urn:uuid:a98c01fe-2fd9-457d-ac53-a7900258b001",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "issuedAt": "2026-10-02T14:20:01Z",
  "payload": {},
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-10-02T14:20:01Z",
    "proofPurpose": "authentication",
    "proofValue": "z5mPrASQ5cpsmsQik4scivNBxCv1fJCdQq7sNCvHL7XE4AHMMM4Emo86yAL6DnBDwF1PdKfcgz9bf6nKXS2KHh1"
  }
}
```

## Security & Privacy

### Data carried

Forge ids, repository names, forge account ids and logins, and role and setting names. A `roleMapReported` event carries only role names and repository names. Forge accounts are personal data: they identify people, and the VTC can join them to members' DIDs. A bridge **MUST NOT** forward raw webhook payloads, which carry far more — commit messages, email addresses, IP-derived metadata — than any event here needs.

### Correlation

The VTC declares `identifierScope: public`; the bridge `pairwise`. Events for non-members (someone added on the forge outside the VTC) carry forge accounts the VTC has no DID for; the VTC keeps them only as drift, for as long as the drift stands.

### Retention

Durable. Events are the history of what happened on the forge outside the VTC's control, which is exactly what an audit of a weakened protection or an unexpected collaborator needs. Drift items are replaced as they are resolved.

### Consent/purpose

The purpose is to keep the forge and the VTC's projection in agreement. Nothing in an event may be used for anything else — in particular not to monitor members' activity on the forge.
