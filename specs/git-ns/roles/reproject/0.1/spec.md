---
slug: git-ns/roles/reproject
version: "0.1"
title: "Git Namespaces — Re-project Roles"
summary: "A community administrator, a namespace admin, or a repository's owner has the VTC send its bridge the complete forge roles of a repository or a whole namespace again, so the forge matches the VTC's rights under the bridge's current role map. No right changes."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - bridge
  - role-map
  - reconciliation
parties:
  - role: requester
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTC
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "A re-projection has the bridge change people's roles on the forge with the community's credentials. It must be attributable to whoever asked on every transport, and it is part of the namespace's audit history."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Placing the request in time lets the VTC refuse a replay. A replay would do no harm — re-projecting is convergent — but it would put a decision in the audit history that nobody made at that time."
sideEffects:
  level: mutating
  rationale: "Queues projectRoles jobs, which change forge roles to match the VTC's rights under the bridge's current role map. No VTC right is created, changed or withdrawn, and nothing is published to the Trust Registry."
consequences:
  - "People's roles on the forge may change: raised or lowered to what the bridge's current role map gives their rights, and taken away where the forge holds a role the bridge projected that no right now asks for."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request names a namespace or a repository and an optional reason. The response lists repository names, which the caller can already read."
retention:
  class: durable
  rationale: "Who re-projected what, and why, is part of the namespace's audit history: it explains a change of forge roles no right change caused."
errorCodes:
  - code: git-ns:unknownNamespace
    meaning: "No namespace bound to this VTC has this identifier, or contains this resource."
    retryable: false
  - code: git-ns:namespaceNotBound
    meaning: "The namespace is still `pending`. Nothing is created, adopted or granted in it until binding completes."
    retryable: false
  - code: git-ns:unknownRepo
    meaning: "The resource names no repository this VTC records."
    retryable: false
  - code: git-ns:repoNotActive
    meaning: "The repository's state does not allow this operation; each task says which states it accepts."
    retryable: false
  - code: git-ns:policyDenied
    meaning: "The community's git-namespace policy refused the request after the fixed rules passed."
    retryable: false
  - code: git-ns/roles/reproject:manualMode
    meaning: "The namespace is governed in manual mode: no bridge projects its roles, so there is nothing to re-project."
    retryable: false
  - code: git-ns/roles/reproject:noForgeAccess
    meaning: "The bridge reported that it lost its access to the namespace (`installationRemoved`). Roles can be re-projected once a forge owner restores it."
    retryable: true
related:
  - git-ns/bridge/job
  - git-ns/bridge/event
  - git-ns/drift/resolve
  - git-ns/right/grant
  - git-ns/view
---

## Abstract

In a bridge-mode namespace the VTC **projects** its rights onto the forge: for each repository it sends the bridge a [`git-ns/bridge/job`](../../../../git-ns/bridge/job/0.3/spec.md) `projectRoles` job listing everyone who should hold a forge role there and their right, and the bridge maps each right to a forge role with its **role map** and converges the forge. The VTC sends a repository's roles when they change — a grant, a revocation, a linked account, a departure — and not otherwise.

So a change that alters what the rights *mean* on the forge, without altering any right, reaches a repository only at its next unrelated projection. The usual one is a change to the bridge's role map: a community that decides maintainers get `admin` on its Forgejo instance, or that committers on one repository get `write`. The bridge reports such a change, and each repository it leaves projected under the old map, with [`git-ns/bridge/event`](../../../../git-ns/bridge/event/0.3/spec.md) `roleMapReported`, and a VTC re-projects those repositories itself. This task is the same re-projection, asked for: by an administrator who changed the map with a bridge too old to report it, by an owner or administrator who wants a forge they suspect has drifted brought back into line, or by anyone entitled who is simply not willing to wait.

It changes no right and publishes nothing. It sends the bridge exactly what the VTC's records already say.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement depends on what `resource` names:

- **A namespace** — the **community-administrator capability** in the VTC's own access control (the one [`git-ns/namespace/bind`](../../../../git-ns/namespace/bind/0.1/spec.md) requires), or **`git.ns.admin` on that namespace** held by explicit record, live at the instant the VTC evaluates the request. Re-projecting a namespace acts on everyone with a role on every repository in it, which is a namespace-level act: owning some of its repositories does not suffice.
- **A repository** — either of the above for its namespace, or **`git.repo.own` on that repository**, held explicitly or by implication ([the rights model](../../../../git-ns/right/grant/0.2/spec.md#the-rights-model)), live at that instant. An owner already decides who holds a right on the repository, and a re-projection decides nothing: it asks the bridge to apply what the repository's records already say.

Nothing else suffices: not `git.repo.maintain`, not `git.commit.sign`, and not `git.repo.own` on another repository.

The community-administrator capability grants nothing *in* a namespace ([`git-ns/right/grant`](../../../../git-ns/right/grant/0.2/spec.md)), and it does not here either: a re-projection decides no right. It is allowed this task because keeping the forge in line with the VTC's records is operating the VTC, which is the capability's own business, and because a namespace whose forge has drifted is exactly where its admins may be absent or unsure. The `proof` establishes who asked, not that they may; that is checked against the VTC's access control and records ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

**Re-projection** — for one repository: a `projectRoles` job with its **complete** `desiredRoles`, recomputed from the VTC's records at the time it is queued exactly as for any projection, sent whether or not the set differs from the one last sent. For a namespace: a re-projection of every repository the VTC records in it as `active` or `orphaned`. `pendingCreate`, `unmanaged`, `archived` and `detached` repositories have no roles to project.

**Role map** — as in [`git-ns/bridge/event`](../../../../git-ns/bridge/event/0.3/spec.md). The bridge applies its own when it runs the job; the request cannot name one, and the VTC does not send one.

**`reason`** — the caller's account of why, kept with the audit record. Free text; it is never sent to the bridge or published.

## Request

The requester sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Resolves `resource` to a namespace: the namespace it names, or the one containing the repository it names. Refuses one it does not have with `git-ns:unknownNamespace`, and a `pending` one with `git-ns:namespaceNotBound`.
2. Checks the entitlement ([Authorization](#authorization)) for what `resource` names, and refuses a caller without it with `permissionDenied`. A caller entitled to nothing in the namespace is refused before step 4, so an unrecorded repository name tells them nothing.
3. Refuses a manual-mode namespace with `git-ns/roles/reproject:manualMode`, and one whose bridge reported `installationRemoved` and has not since reported access again with `git-ns/roles/reproject:noForgeAccess`.
4. For a repository `resource`, refuses one it does not record with `git-ns:unknownRepo`, and one not `active` or `orphaned` with `git-ns:repoNotActive`.
5. Evaluates the community's git-namespace policy, which may refuse with `git-ns:policyDenied` and may not override the rules above.
6. Queues a re-projection of each repository covered. Where a `projectRoles` job for a repository is already queued and not yet sent, the VTC **MAY** replace it with the re-projection rather than queue a second; it **MUST NOT** drop the re-projection in favour of a job computed earlier.
7. Records an audit event carrying the caller, `resource`, `reason` and the repositories covered, and answers with `repos`, the repositories it queued a re-projection for. The response **MUST NOT** be sent before the jobs and the audit event are durable.

The work on the forge is asynchronous, as every projection is: each job's [`git-ns/bridge/result`](../../../../git-ns/bridge/result/0.1/spec.md) says how it ended, and a VTC **SHOULD** show a repository whose re-projection failed as it shows any failed projection. A re-projection that succeeds clears a repository from the `stale` list of the last `roleMapReported` the VTC holds ([`git-ns/bridge/event`](../../../../git-ns/bridge/event/0.3/spec.md), request step 5).

Sending the request again is harmless: every job is convergent, and one that finds the forge already matching changes nothing.

### Re-projecting a namespace after its role map changed

Dana, a community administrator, changed the Forgejo bridge's configuration so that maintainers in `acme` get `admin`. The bridge is too old to report its role map, so she re-projects the namespace.

```json
{
  "id": "urn:uuid:5c1d2e7f-8a9b-4c3d-9e2f-1a0b3c4d5e01",
  "type": "https://trusttasks.org/spec/git-ns/roles/reproject/0.1",
  "threadId": "urn:uuid:5c1d2e7f-8a9b-4c3d-9e2f-1a0b3c4d5e01",
  "issuer": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-06T09:05:00Z",
  "payload": {
    "resource": "codeberg.org/acme",
    "reason": "Maintainers now get admin on Forgejo (bridge config change of 6 October)"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmDanaScid8:acme-vtc.example:dana#key-1",
    "created": "2026-10-06T09:05:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z2Kd7Ns4Wq9Lb3Rx6Vc1Fm8Yt5Jh2Pg7Ue4Zo9Ti3Nn6Xb8Cq1Dw5Ek2Sh7Gv4Ly1Au9Bj6Kf3Tr8Np5Zm2Rc"
  }
}
```

### Re-projecting one repository

Alice, who owns `widgets`, suspects someone's role on `widgets` was changed by hand and the change was missed.

```json
{
  "id": "urn:uuid:5c1d2e7f-8a9b-4c3d-9e2f-1a0b3c4d5e03",
  "type": "https://trusttasks.org/spec/git-ns/roles/reproject/0.1",
  "threadId": "urn:uuid:5c1d2e7f-8a9b-4c3d-9e2f-1a0b3c4d5e03",
  "issuer": "did:webvh:QmAliceScid2:acme-vtc.example:alice",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-06T11:00:00Z",
  "payload": {
    "resource": "github.com/acme/widgets"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmAliceScid2:acme-vtc.example:alice#key-1",
    "created": "2026-10-06T11:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z5Wm2Hq7Lr4Tn9Wb1Kx6Vd3Fm8Ys5Jc2Pg7Ue4Zo1Ti6Nn3Xb9Cq4Dw1Ek6Sh3Gv8Ly5Au2Bj9Kf4Tr1Np7Zm3"
  }
}
```

## Response

The VTC answers with the repositories it queued a re-projection for, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`.

### Queued

```json
{
  "id": "urn:uuid:5c1d2e7f-8a9b-4c3d-9e2f-1a0b3c4d5e02",
  "type": "https://trusttasks.org/spec/git-ns/roles/reproject/0.1#response",
  "threadId": "urn:uuid:5c1d2e7f-8a9b-4c3d-9e2f-1a0b3c4d5e01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "issuedAt": "2026-10-06T09:05:01Z",
  "payload": {
    "repos": ["codeberg.org/acme/gadgets", "codeberg.org/acme/widgets"]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-10-06T09:05:01Z",
    "proofPurpose": "authentication",
    "proofValue": "z4Tq1Lb8Rx5Kc2Hm9Fd6Ya3Js7Vg4Ue1Zo8Ti5Pn2Xb6Cq3Dw9Ek4Sh1Gv7Ly2Au5Bj8Kf6Tr3Np9Zm1Rc7"
  }
}
```

## Security & Privacy

### Data carried

A resource and an optional reason in the request; repository names in the response. The jobs the task causes carry member DIDs joined to forge accounts, as every `projectRoles` job does, and go only to the namespace's bridge.

### Impact

A re-projection records, changes and publishes no right. What it can change is forge roles, and only toward what the VTC's records and the bridge's role map already call for: it cannot give anyone a role their rights do not ask for, and it cannot keep one they no longer should hold. Its impact is therefore bounded by the role map, which the community sets in its bridge's configuration and which the bridge would apply at the next projection in any case. A VTC that grades consent by impact has what it needs to grade this task as below any grant or revocation; whether any of it warrants a step-up is the VTC's policy, which per [SPEC §7.3](/SPEC.md#73-specification-requirements) item 13 this specification does not decide.

The one change a caller should expect to be noticed is a lowering: a role map narrowed since the last projection — owners given `maintain` rather than `admin`, say — takes the higher role away from everyone it covers when the re-projection runs. That is the point of re-projecting, and it is also why the task is refused on a namespace the bridge can no longer reach rather than queued to run whenever access returns.

### Correlation

The VTC declares `identifierScope: public`; the caller `pairwise`. Nothing in the request or response identifies anyone but the caller.

### Retention

Durable: the audit record is what explains, later, a change of forge roles no change of rights caused.

### Consent/purpose

The purpose is to keep the forge in agreement with the VTC's rights. `reason` is read by the namespace's admins and the community's administrators with the audit record; a caller **MUST NOT** put in it anything they would not show them, and a VTC **MUST NOT** send it to the bridge or publish it.
