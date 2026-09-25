---
slug: git-ns/drift/resolve
version: "0.3"
title: "Git Namespaces — Resolve Drift"
summary: "An owner of a repository, or a namespace admin over it, resolves one reported drift item: adopt records the forge-side role as a VTC right for the member the resolver names, as git-ns/right/grant would; revert has the bridge undo the forge-side change."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - drift
  - reconciliation
  - commit-signing
parties:
  - role: member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTC
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "Adopting records a right that changes whose commits the repository's CI check accepts; reverting takes a role away from someone on the forge. Either must be attributable to the resolver on every transport, and both are part of the repository's audit history."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A resolution replayed after the forge changed again would adopt or revert a state nobody decided about. Placing the request in time is what lets the VTC refuse the replay."
sideEffects:
  level: mutating
  rationale: "Adopt records and publishes a right, as git-ns/right/grant does, and is undone with git-ns/right/revoke. Revert changes roles or protection on the forge through the bridge, and is undone by granting the right that projects the role, or by the change being made on the forge again."
consequences:
  - "Adopting publishes the recorded right to the community's Trust Registry, exactly as git-ns/right/grant does, where anyone can read that the member holds it on the repository."
  - "Reverting a role takes it away from the person on the forge. Reverting an owner-level role — the role git.repo.own projects to, admin on GitHub — removes their administrative control of the repository on the forge."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request names a repository, a drift type and at most one forge account, which the caller read from git-ns/view, and for adopt the DID of the member the resolver read as linked to that account. The response returns the adopted right and the repository's sync, whose drift items carry forge accounts and logins."
retention:
  class: durable
  rationale: "The resolution — who decided what about which forge-side change — is part of the repository's audit history, and an adopted right is a grant kept as git-ns/right/grant keeps it."
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
  - code: git-ns/drift/resolve:driftNotFound
    meaning: "No outstanding drift item on the repository matches the selector: it was resolved already, the forge has changed since it was read (`observed` no longer matches), or it was never reported."
    retryable: false
  - code: git-ns/drift/resolve:notAdoptable
    meaning: "There is no right to record for this item: `adopt` was asked of a `roleRemoved`, `requiredCheckMissing`, `protectionWeakened` or `bootstrapMissing` item, or of a `roleChanged` item whose observed role is no higher than what the account's member already holds. Revert it, or revoke with git-ns/right/revoke."
    retryable: false
  - code: git-ns/drift/resolve:accountNotLinked
    meaning: "`adopt` was asked of a role held by a forge account that is linked to no current member. Rights are held by DIDs, so there is nobody to record the right for; the role can only be reverted."
    retryable: false
  - code: git-ns/drift/resolve:subjectChanged
    meaning: "`adopt` named a `subject` that is not the member the item's account is linked to now: the account was unlinked and linked again, to someone else, after the resolver read it. Nothing is adopted. Read the item again, and decide about the member it now names."
    retryable: false
  - code: git-ns/drift/resolve:noMatchingRight
    meaning: "`adopt` was asked of a forge role that no git right projects to on this repository — `triage` or `read`, or `write` where committers get no forge role. Revert it, or grant a right with git-ns/right/grant and then revert the role."
    retryable: false
  - code: git-ns/drift/resolve:notRevertible
    meaning: "The bridge serving the namespace cannot perform the job this revert needs: it refused the job, or the revert needs `removeAccounts` and the bridge implements only git-ns/bridge/job 0.1."
    retryable: false
related:
  - git-ns/view
  - git-ns/right/grant
  - git-ns/right/revoke
  - git-ns/bridge/job
  - git-ns/bridge/event
  - git-ns/bridge/result
  - git-ns/account/link
---

## Abstract

In a bridge-mode namespace the bridge compares each repository on the forge with the VTC's projection of it and reports every difference as a **drift item** ([`git-ns/bridge/event`](../../../../git-ns/bridge/event/0.2/spec.md)); members read them in each repository's `sync.drift` through [`git-ns/view`](../../../../git-ns/view/0.3/spec.md). Some drift the VTC answers by itself — by policy it re-applies a weakened ruleset — and some it only reports: most often a collaborator somebody added, or a role somebody raised, in the forge's own interface.

This task is how an owner of the repository answers a reported item. **`adopt`** accepts the forge-side change by recording it as a VTC right for the member the resolver names, under exactly the rules a [`git-ns/right/grant`](../../../../git-ns/right/grant/0.2/spec.md) would meet, so the projection comes to match the forge. **`revert`** keeps the projection as it is and has the bridge undo the forge-side change with a [`git-ns/bridge/job`](../../../../git-ns/bridge/job/0.3/spec.md). The VTC stays the source of truth either way: adopting changes the truth deliberately, reverting restores the forge to it.

## Changes from 0.2

`adopt` names the member who receives the right. A `0.2` adoption did not: the VTC found the member by resolving the item's forge account to whoever had linked it **when the task ran**. A resolver reads the drift item and the account's owner, decides, and signs; in between, the account could be unlinked and linked again — to someone else — through [`git-ns/account/link`](../../../../git-ns/account/link/0.1/spec.md). The signed `0.2` document then granted a right, published to the Trust Registry, to a member the resolver never saw, and nothing in what they signed said otherwise. The `observed` value bounds *which role* an adoption can record; nothing bounded *to whom*.

`0.3` adds **`subject`**, the DID of the member the resolver read as linked to the account. It is **required for `adopt`** and absent for `revert`, refused as `malformedRequest` otherwise, as a missing `observed` is. The VTC adopts only while the account is linked to exactly that member — checked where the right is recorded, so no relink can fall between the check and the write — and refuses otherwise with the new **`git-ns/drift/resolve:subjectChanged`**. What the resolver signs now names both halves of the grant it amounts to: the right, through `observed`, and its holder, through `subject`.

A `0.2` document is a `0.3` document with no `subject`. That is still a valid `revert`, and it is now a malformed `adopt`, so a `0.2` client can revert through `0.3` but not adopt. Adding a member is backwards-compatible on the wire, but making it required for one action is not, so this is a `MINOR` increment under the `draft` allowance of [SPEC §5.2](/SPEC.md#52-compatibility-rules). A VTC that serves both **SHOULD NOT** accept a `0.1` or `0.2` `adopt`, because it cannot know whom the resolver meant; see [Security & Privacy](#security--privacy). Everything else is unchanged from `0.2` and restated below, so that this version stands on its own.

## Changes from 0.1

`0.2` pinned [`git-ns/_shared/0.3`](../../../_shared/0.3/git-ns.schema.json), which narrows `Did` to the syntax of [W3C DID Core §3.1](https://www.w3.org/TR/did-core/#did-syntax). `0.1` pinned `_shared/0.2`, whose `Did` accepted anything without whitespace after `did:<method>:` — shell metacharacters, quotes, backticks, `/`, `?` and `#` included — so a value that was not a DID at all validated, and travelled on to every surface that later displayed, logged or quoted it. The members affected here are `right.subject` and `right.grantedBy` in the response. Each now carries a bare DID only: `did:`, a method name of lowercase letters and digits, and a method-specific id of colon-separated segments drawn from `A-Z a-z 0-9 . - _` and percent-encoded octets, the last one non-empty. A DID URL — a path, a query, or a `#` fragment such as a verification-method id — is refused: every one of these members names a party, never a key.

That narrowing, made in `0.2`, is kept.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **`git.repo.own` on the repository, held explicitly or by implication** — which includes every holder of `git.ns.admin` on its namespace ([the rights model](../../../../git-ns/right/grant/0.2/spec.md#the-rights-model)). Anyone else is refused with `permissionDenied`, maintainers and committers included: deciding whether a forge-side change stands is deciding who holds a right on the repository, which is an owner's authority.

For `adopt`, that authority is exercised exactly as a grant is: an owner may grant `git.repo.own`, `git.repo.maintain` and `git.commit.sign` on their repository, which are the only rights an adoption can record, and the VTC applies the grant's fixed rules and the community's policy as if the resolver had sent the equivalent `git-ns/right/grant` themselves. The forge-side change is never evidence of entitlement: whoever made it on the forge had forge permission, not a VTC right, and it is the resolver's authority, not theirs, that the recorded right traces back to. For `revert`, the entitlement is the same ownership; the bridge acts on the VTC's instruction and does no authorization of its own.

The `proof` establishes which DID is acting; the VTC then resolves it to its rights records ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

**Drift item** — one difference between the forge and the VTC's projection of a repository, as the `DriftItem` shape of [`git-ns/_shared/0.3`](../../../_shared/0.3/git-ns.schema.json) describes it. A repository's **outstanding** drift is the list the VTC holds in its `sync.drift`, which each `git-ns/bridge/event` concerning the repository replaces wholesale.

**`drift`** — the selector naming the item to resolve. Drift items have **no identifier of their own**, and this task deliberately does not give them one: they are recomputed by the bridge from the forge's current state and replaced wholesale by every event, so an identifier would be minted by one comparison and gone at the next, while the thing it named — *this account's role on this repository differs* — is still there. The selector therefore names the item by what makes it distinct:

- **`type`** — always.
- **`account`** — for the three role types, and only for them. An account's role on a repository either is missing, is present at another level, or is present without being projected, so among a repository's outstanding items there is **at most one role item per account**, and `resource` + `type` + `account` names it exactly. Accounts are matched by `forge` and `id`; `login` is display only, because logins are renamed.
- For `requiredCheckMissing`, `protectionWeakened` and `bootstrapMissing` there is no account, and the selector names **every** outstanding item of that type on the repository. That is not a loss of precision: those items are reverted by re-applying the repository's protection or bootstrap plan as a whole, which answers all of them at once, and none of them can be adopted.
- **`observed`** — optionally, the item's `observed` value as the caller last read it. When present the item is selected only if the forge still shows exactly that. It is **required for `adopt`**, because the right an adoption records is derived from the observed role: without it, an owner who decided to accept a `write` could find they had accepted an `admin` someone set in the meantime.

**`action`** — `adopt` or `revert`, as below.

**`subject`** — for `adopt`, the DID of the member the resolver read as linked to the item's account: the member who receives the right. Required for `adopt`, absent for `revert`. A resolver learns it wherever they read who linked an account — an administrator's console, or any surface the VTC gives an owner that join on; [`git-ns/view`](../../../../git-ns/view/0.3/spec.md) returns a member only their own links, so a client that has only `git-ns/view` cannot adopt another member's role.

**Projected right of a role** — the right that the namespace's forge adapter maps to a given forge role on this repository: the inverse of the mapping `desiredRoles` uses in [`git-ns/bridge/job`](../../../../git-ns/bridge/job/0.3/spec.md). On a GitHub organisation, `admin` is the projection of `git.repo.own` and `maintain` of `git.repo.maintain`; `write` is the projection of `git.commit.sign` only on a repository that projects its committers to `write`, and of nothing otherwise. Where two rights project to the same role — on a personal account, where `own` and `maintain` both collapse to collaborator `write` — the projected right is the lower one. A role that no right projects to (`triage`, `read`) has none.

**`reason`** — the resolver's free text.

## Request

The resolver sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Applies the [SPEC §7.2](/SPEC.md#72-consumer-requirements) pipeline and resolves the `issuer` to its rights records.
2. Refuses a resource inside no bound namespace with `git-ns:unknownNamespace`, one inside a `pending` namespace with `git-ns:namespaceNotBound`, one it does not record with `git-ns:unknownRepo`, and a repository that is not `active` or `orphaned` with `git-ns:repoNotActive`.
3. Refuses a caller without `git.repo.own` on the repository, explicit or implied, with `permissionDenied`.
4. Refuses with the framework's `malformedRequest` a selector that carries `account` for a type other than the three role types or lacks it for one of them, an `adopt` whose selector carries no `observed` or which names no `subject`, and a `revert` that names a `subject`.
5. Selects the outstanding item or items the selector names. When none matches, refuses with `git-ns/drift/resolve:driftNotFound`. A client that only needs the item gone — because it resolved it already and lost the response — **MAY** treat that code as success.
6. Evaluates the community's git-namespace policy for the resolution — for `adopt`, as part of evaluating the grant below — which may refuse either action with `git-ns:policyDenied` (for example, a policy that no forge-side change is ever adopted) and may not override the rules of this specification.
7. Performs `adopt` or `revert`, as below.
8. Records the resolution — the resolver, the action, the item as it stood and `reason` — as an audit record of the repository, and removes the resolved item from the repository's outstanding drift. The repository's sync becomes `drift` while other items remain and `pending` otherwise. The response **MUST NOT** be sent before the resolution, and for `adopt` the right, are durable.
9. Sends the job the action calls for to the namespace's bridge. The work on the forge is asynchronous, as every projection is: when the bridge's [`git-ns/bridge/result`](../../../../git-ns/bridge/result/0.1/spec.md) for it is `failed`, or a later event still reports the item, the item is outstanding again and the resolver sees it again in `git-ns/view`. A VTC **SHOULD** follow a resolution with an `inspect` job for the repository, so that the resolution is confirmed rather than assumed.

### Adopt

Only a role that a member holds, and a right can express, can be adopted. A conforming VTC:

1. Refuses with `git-ns/drift/resolve:notAdoptable` an item of type `roleRemoved`, `requiredCheckMissing`, `protectionWeakened` or `bootstrapMissing`: a missing role or a weakened protection records no right, and accepting the loss of a projected role is a revocation, for [`git-ns/right/revoke`](../../../../git-ns/right/revoke/0.2/spec.md).
2. Refuses with `git-ns/drift/resolve:accountNotLinked` an item whose account is linked, with [`git-ns/account/link`](../../../../git-ns/account/link/0.1/spec.md), to no current member of the community. Rights are held by DIDs; a forge account with none can only be reverted.
3. Refuses with `git-ns/drift/resolve:subjectChanged` when that member is not `subject`, compared by exact string equality.
4. Determines the projected right of the observed role. Refuses with `git-ns/drift/resolve:noMatchingRight` a role that has none.
5. For a `roleChanged` item, refuses with `git-ns/drift/resolve:notAdoptable` when that right is not strictly higher, in the order `git.commit.sign` < `git.repo.maintain` < `git.repo.own`, than the member's highest effective right on the repository: a forge-side lowering is accepted by revoking, not by adopting.
6. Evaluates the right exactly as [`git-ns/right/grant`](../../../../git-ns/right/grant/0.2/spec.md) evaluates a grant sent by the resolver with `subject` this request's `subject`, `right` the projected right, `resource` the repository, no `expiresAt`, and `reason` this request's `reason`: its request steps 3 to 7, including [the fixed rules](../../../../git-ns/right/grant/0.2/spec.md#the-fixed-rules) in order, policy, the idempotent return of a live record that already exists, and `grantedBy` set to the resolver. A refusal there is this task's refusal, with the same code. The VTC **MUST** make the checks of steps 2 and 3, and the selection of the item, again under the same exclusion as the write that records the right, and refuse as those steps do if either no longer holds: a relink or a new drift report between the first check and the write adopts nothing.
7. Publishes the right as a grant is published, and sends the bridge a `projectRoles` job for the repository with its complete `desiredRoles`, now including the member at the adopted right. Because the adopted right is the one that projects the observed role, the forge already matches and the job converges without change.

### Revert

A revert never changes a VTC right. It sends the bridge the job that makes the forge match the projection again:

| Item | Job ([`git-ns/bridge/job`](../../../../git-ns/bridge/job/0.3/spec.md)) |
|---|---|
| `roleAdded` | `projectRoles` on the repository, with its complete `desiredRoles` and `removeAccounts` naming the item's account. **Requires `git-ns/bridge/job` 0.2 or later.** |
| `roleChanged`, `roleRemoved` | `projectRoles` on the repository, with its complete `desiredRoles`, which list the account at its projected level. `0.1` suffices. |
| `requiredCheckMissing`, `protectionWeakened` | `bootstrap` on the repository with `steps: ["requiredCheck"]`. |
| `bootstrapMissing` | `bootstrap` on the repository with no `steps`: the whole plan, which changes only what is missing. |

A `roleAdded` item is the one `git-ns/bridge/job` 0.1 cannot revert. A role someone gave on the forge outside the bridge is not a role the bridge manages, and `0.1` reports such a role as drift rather than remove it however often `desiredRoles` is re-sent — which is what makes it drift in the first place. `0.2`'s `removeAccounts` says explicitly that the account's role goes. The other rows reuse jobs `0.1` already defines, with the meaning it already gives them. A conforming VTC:

1. Refuses with `git-ns/drift/resolve:notRevertible` a revert whose job the bridge refuses, or, for a `roleAdded` item, a bridge it knows implements only `git-ns/bridge/job` 0.1. It **MUST NOT** answer a `roleAdded` revert by re-sending `desiredRoles` alone and reporting success: the role would stay.
2. Never reverts by changing the projection: if the item names a member who should hold the role, the remedy is `adopt`, or a grant.

A revert that removes or lowers the role `git.repo.own` projects to — `admin` on GitHub — takes administrative control of the repository on the forge away from the person holding it. Its impact is that of revoking `git.repo.own`; every other revert has at most the impact of revoking `git.repo.maintain`. An adoption has the impact of the equivalent grant — for an `admin` role, of granting `git.repo.own`. A VTC that grades consent by impact has what it needs here to grade each resolution as it grades the equivalent `git-ns/right/grant` or `git-ns/right/revoke`; whether any of them warrants a step-up is the VTC's policy, which per [SPEC §7.3](/SPEC.md#73-specification-requirements) item 13 this specification does not decide.

### Carol reverts a collaborator added on GitHub

`eve-dev` was given `write` on `widgets` in GitHub's own interface. Carol, an owner, does not want it to stand.

```json
{
  "id": "urn:uuid:c3a7e0d4-51b2-4f6e-9a8d-0e1f2a3b4c01",
  "type": "https://trusttasks.org/spec/git-ns/drift/resolve/0.3",
  "threadId": "urn:uuid:c3a7e0d4-51b2-4f6e-9a8d-0e1f2a3b4c01",
  "issuer": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-24T09:09:58Z",
  "payload": {
    "resource": "github.com/acme/widgets",
    "drift": {
      "type": "roleAdded",
      "account": {
        "forge": "github.com",
        "id": "5550123",
        "login": "eve-dev"
      },
      "observed": "write"
    },
    "action": "revert",
    "reason": "Not a contributor to widgets; access requests go through the VTC"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmCarolScid3:acme-vtc.example:carol#key-1",
    "created": "2026-09-24T09:09:58Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z4Vx8mQp2Lr7TbN5kWc9Hd3Ja6Ys1Fg8Ue4Zo2Ri7Pn5Xt3Mb9Cq6Dw1Ek8Sh4Gv2Ly7Au5Bj3Kf9Tr6Np1Zm"
  }
}
```

### Bob adopts a maintainer role he was given on GitHub

Alice, a namespace admin, made Bob a maintainer of `widgets` in GitHub's interface; Bob has linked his GitHub account, and the bridge reported it as `roleAdded`. Carol accepts it rather than revert it, naming Bob as the member she read as linked to `bob-builds`.

```json
{
  "id": "urn:uuid:c3a7e0d4-51b2-4f6e-9a8d-0e1f2a3b4c03",
  "type": "https://trusttasks.org/spec/git-ns/drift/resolve/0.3",
  "threadId": "urn:uuid:c3a7e0d4-51b2-4f6e-9a8d-0e1f2a3b4c03",
  "issuer": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-24T09:20:00Z",
  "payload": {
    "resource": "github.com/acme/widgets",
    "drift": {
      "type": "roleAdded",
      "account": {
        "forge": "github.com",
        "id": "9120045",
        "login": "bob-builds"
      },
      "observed": "maintain"
    },
    "action": "adopt",
    "subject": "did:webvh:QmBobScid2:acme-vtc.example:bob",
    "reason": "Bob triages widgets releases"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmCarolScid3:acme-vtc.example:carol#key-1",
    "created": "2026-09-24T09:20:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z2Hn6Tq9Wd4Lx1Cb7Rm3Ks8Fv5Pa2Yg9Jt6Ne4Uz1Qo7Mh3Xr8Db5Gc2Lw9Sk6Ap1Vf4Ej7Bn3Ty8Rz5Kq2Mc"
  }
}
```

## Response

The VTC, now responding, returns what the resolution did, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). `right` is present exactly for `adopt`. Refusals use `trust-task-error` with `permissionDenied`, `malformedRequest` or one of this specification's codes.

### The revert is on its way

The item is no longer outstanding and nothing else was, so `widgets` is `pending` until the bridge confirms the role is gone.

```json
{
  "id": "urn:uuid:c3a7e0d4-51b2-4f6e-9a8d-0e1f2a3b4c02",
  "type": "https://trusttasks.org/spec/git-ns/drift/resolve/0.3#response",
  "threadId": "urn:uuid:c3a7e0d4-51b2-4f6e-9a8d-0e1f2a3b4c01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
  "issuedAt": "2026-09-24T09:09:59Z",
  "payload": {
    "action": "revert",
    "sync": {
      "state": "pending",
      "checkedAt": "2026-09-23T09:58:00Z",
      "drift": []
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-24T09:09:59Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z8Kp3Wm6Qa1Tz9Lc4Rb7Hn2Xv5Fd8Jg3Ys6Ue1No4Mt9Ci2Pr7Dh5Ek3Gw8Lq1Sx6Bz4Av9Tj2Kn7Rf5Ym3Hc"
  }
}
```

### Bob's maintainer right, recorded

```json
{
  "id": "urn:uuid:c3a7e0d4-51b2-4f6e-9a8d-0e1f2a3b4c04",
  "type": "https://trusttasks.org/spec/git-ns/drift/resolve/0.3#response",
  "threadId": "urn:uuid:c3a7e0d4-51b2-4f6e-9a8d-0e1f2a3b4c03",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
  "issuedAt": "2026-09-24T09:20:01Z",
  "payload": {
    "action": "adopt",
    "right": {
      "subject": "did:webvh:QmBobScid2:acme-vtc.example:bob",
      "right": "git.repo.maintain",
      "resource": "github.com/acme/widgets",
      "grantedBy": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
      "grantedAt": "2026-09-24T09:20:01Z",
      "reason": "Bob triages widgets releases"
    },
    "sync": {
      "state": "pending",
      "checkedAt": "2026-09-24T09:15:00Z",
      "drift": []
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-24T09:20:01Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z6Rt2Yb8Kq4Nm1Wx7Hc3Lp9Dz5Fa2Js8Vg6Ue3Ti1Ob4Qn7Mk9Cr5Pw2El8Gh3Sy6Ad1Bv4Xf7Lj2Tq9Rz5Kn3M"
  }
}
```

### Refused: the account belongs to nobody in the community

Had Carol asked to adopt `eve-dev`'s role instead, the VTC would refuse: no member has linked that account.

```json
{
  "id": "urn:uuid:c3a7e0d4-51b2-4f6e-9a8d-0e1f2a3b4c06",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:c3a7e0d4-51b2-4f6e-9a8d-0e1f2a3b4c05",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
  "issuedAt": "2026-09-24T09:11:01Z",
  "payload": {
    "code": "git-ns/drift/resolve:accountNotLinked",
    "message": "github.com account 5550123 is not linked to a member; it can only be reverted.",
    "retryable": false,
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/git-ns/drift/resolve/0.3",
      "id": "urn:uuid:c3a7e0d4-51b2-4f6e-9a8d-0e1f2a3b4c05"
    }
  }
}
```

### Refused: the account was relinked after Carol read it

Had `bob-builds` been unlinked from Bob and linked to Dan between Carol reading the item and her adoption arriving, the VTC would adopt nothing.

```json
{
  "id": "urn:uuid:c3a7e0d4-51b2-4f6e-9a8d-0e1f2a3b4c08",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:c3a7e0d4-51b2-4f6e-9a8d-0e1f2a3b4c03",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
  "issuedAt": "2026-09-24T09:20:01Z",
  "payload": {
    "code": "git-ns/drift/resolve:subjectChanged",
    "message": "github.com account 9120045 is now linked to another member than the one this adoption names; read the drift item again.",
    "retryable": false,
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/git-ns/drift/resolve/0.3",
      "id": "urn:uuid:c3a7e0d4-51b2-4f6e-9a8d-0e1f2a3b4c03"
    }
  }
}
```

## Security & Privacy

### Data carried

The request names a repository, a drift type, at most one forge account and the role the caller saw, all of which the caller read from `git-ns/view`, an optional reason, and for `adopt` the member's DID. That DID joins the member to the forge account in the document; the resolver already knew the join, and the adoption publishes the member's right on the repository anyway, so it discloses nothing the registry and the forge together would not. The response returns, for an adoption, the recorded right with its reason — to a caller who by construction owns the repository and may see both — and the repository's sync, whose remaining drift items carry forge accounts and logins, which owners already see. `reason` is free text read by the repository's owners and namespace admins and kept with the audit record, and as the adopted right's `reason`; a resolver **MUST NOT** put in it anything they would not show them, and a VTC **MUST NOT** publish it to the registry.

### Binding the recipient

An adoption is a grant whose recipient the resolver did not type. Without `subject`, the VTC would pick the recipient at execution time from a link the resolver may never have seen in its current state, and the proof on the document would attest a decision the resolver did not make. Naming the recipient, and refusing when the link no longer resolves to it, keeps the signed document a complete statement of the grant. A VTC that also serves `0.1` or `0.2` **SHOULD NOT** accept an `adopt` over them — it has no way to know whom that resolver meant — and **SHOULD** answer one with a refusal that names this version.

### Correlation

The VTC declares `identifierScope: public`, as the authority whose published rights an adoption changes and every repository names; the resolver `pairwise`. A revert names a forge account that frequently belongs to someone who is not a member and has no DID the VTC knows. The VTC learned that account from the bridge's drift report and holds it only as drift and in the resolution's audit record; it **MUST NOT** use a reverted account to look the person up anywhere else. An adoption joins a member's DID to a right on the repository publicly, exactly as a grant does, and that is its purpose.

### Retention

Durable. The resolution — who adopted or reverted which forge-side change, when, and why — is kept with the repository's audit history, which is where a later question about a collaborator nobody remembers adding is answered. An adopted right is retained as every grant is.

### Consent/purpose

The purpose is to decide whether a change made on the forge outside the VTC stands. What a request carries **MUST NOT** be used for anything else. Adoption is a grant in effect, and whether the member must agree to receive the right is the VTC's policy, as it is for a grant; whether any resolution warrants a step-up is the VTC's policy too ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13). The impact that policy weighs is stated above under [Revert](#revert).
