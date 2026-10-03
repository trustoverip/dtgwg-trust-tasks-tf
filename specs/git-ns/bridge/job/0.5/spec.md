---
slug: git-ns/bridge/job
version: "0.5"
title: "Git Namespaces — Bridge Job"
summary: "The VTC asks its bridge — the separate service holding the community's forge credentials — to do one convergent piece of forge work: project roles, create, bootstrap, archive or inspect a repository, begin a binding or account link, or close a disallowed pull request."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - github
  - forgejo
  - bridge
  - automation
  - pull-request
parties:
  - role: VTC
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: bridge
    requirement: REQUIRED
    member: recipient
    identifierScope: pairwise
proofRequirement:
  requirement: REQUIRED
  rationale: "A job makes the bridge act on the forge with the community's credentials. The bridge must be able to tell its own VTC's jobs from anyone else's on every transport; nothing else authorises them."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Jobs are convergent, but a stale job replayed after later ones would push the forge back to an old desired state."
sideEffects:
  level: mutating
  rationale: "Changes repositories, roles and protection on the forge, and closes pull requests with a comment. Every change is convergent; a role or protection change is undone by a later job with a different desired state, and a closed pull request can be reopened by an owner or maintainer of the repository."
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: "Jobs carry member DIDs joined to their forge accounts, which is personal data held only so the bridge can give those accounts roles."
retention:
  class: exchange
  rationale: "A job is kept until its result is acknowledged and its identifier long enough to recognise a repeat."
errorCodes:
  - code: git-ns:unknownNamespace
    meaning: "No namespace bound to this VTC has this identifier, or contains this resource."
    retryable: false
  - code: git-ns/bridge/job:notCapable
    meaning: "This forge, or this namespace on it, cannot perform this kind of job — for example `createRepo` on a personal account."
    retryable: false
  - code: git-ns/bridge/job:jobIdReused
    meaning: "The bridge already holds a job with this `jobId` and different content."
    retryable: false
related:
  - git-ns/bridge/result
  - git-ns/bridge/event
  - git-ns/repo/create
  - git-ns/namespace/bind
  - git-ns/account/link
  - git-ns/drift/resolve
---

## Abstract

A VTC that governs forge namespaces does not talk to the forges itself. Each community runs a **bridge** next to its VTC: a separate service with its own DID, the only holder of the community's forge credentials — its own GitHub App key, or its Forgejo bot's token — and the home of the forge adapters that know how each forge does things. The VTC decides; the bridge carries it out and reports back.

This task is the VTC's half: one job, one piece of forge work in one namespace. Jobs carry desired state, never credentials, and every job is **convergent**: the bridge checks the forge and changes only what differs, so sending a job again is always safe. How it ended comes back as [`git-ns/bridge/result`](../../../../git-ns/bridge/result/0.1/spec.md); what the bridge sees happening on the forge comes back as [`git-ns/bridge/event`](../../../../git-ns/bridge/event/0.4/spec.md).

The job vocabulary is forge-neutral. A forge that lacks a capability — a personal GitHub account, where no bot can create repositories — is handled by the VTC not sending that job, and by the bridge refusing it if sent.

## Changes from 0.4

One kind is added, and nothing else in a job's meaning changes.

1. **`closePullRequest`: the bridge comments on a pull request and closes it.** A community may restrict who may open pull requests on its governed repositories. The bridge reports each pull request opened or reopened as a `pullRequestOpened` event ([`git-ns/bridge/event`](../../../../git-ns/bridge/event/0.4/spec.md) 0.4); when the VTC's pull-request policy does not allow its author, the VTC sends this job, and the bridge posts the community's `message` on pull request `number` of `repo` and then closes it ([Definitions](#definitions)).
2. **The job is idempotent.** Closing a pull request that is already closed succeeds, and posts no second comment ([Request](#request), step 7).
3. **It is hygiene, not the merge gate.** The required commit-trust check (`requiredCheck`) stays the control that keeps untrusted commits out. A `closePullRequest` that never runs — the VTC or the bridge is down, the bridge does not take `0.5` — leaves a pull request open, and nothing that check refuses can be merged through it.

The new kind adds two members, `number` and `message`, and the schema requires them together, with `repo` (`dependentRequired`); which kind may carry them is, as for every other member, the kind table's rule. A `0.4` document is a valid `0.5` document with the same meaning. Adding a kind is released as a `MINOR` increment under the `draft` allowance of [SPEC §5.2](/SPEC.md#52-compatibility-rules).

**Which version a VTC sends.** A VTC **MUST NOT** send a `0.5` job to a bridge that has not listed the `0.5` *Type URI* in answer to a [`trust-task-discovery`](/SPEC.md#10-discovery-and-capability-negotiation) request from that VTC, and a bridge that takes `0.5` **MUST** answer one. To a bridge that lists `0.4` and not `0.5`, the VTC keeps sending `0.4` jobs for every other kind, exactly as before — such a bridge is unaffected by this version — and the namespaces it serves have no pull-request gate: the VTC sends it no `closePullRequest`, and **SHOULD** tell its operator that the policy is not enforced there until the bridge is upgraded. A bridge that takes `0.5` **SHOULD** go on accepting `0.4` jobs, which mean the same thing. The rule of `0.4` still holds: a VTC never sends a bridge a version earlier than `0.4`.

### Carried forward: what 0.4 changed from 0.3


A namespace admin gets no role on the forge for being one: `git.ns.admin` itself never projects a forge role. A namespace admin may still hold ordinary, non-elevated rights in their own name — `git.repo.maintain`, `git.commit.sign` — which project to non-admin forge roles as anyone's do, and granting those to oneself is an allowed self-grant. `git.ns.admin` is exercised through the VTC and its bridge — binding, adopting, reseating, granting — and never through a forge role: the bridge does not make its holder an owner of the organisation, and gives them no role on any repository for it. Making someone an owner of the organisation on the forge is left to the community, outside the VTC and its bridge. `0.3` said the opposite in two places, which is why this is a new version rather than a correction:

- **`desiredRoles` carries rights held in the person's own name.** In `0.3` each entry carried the person's "highest effective right" on the target, which counts what implication gives: a namespace admin was listed as `git.repo.own` on every repository in the namespace, and the bridge could not tell them from a recorded owner. In `0.4` the right listed is counted only from the rights recorded for that person, and a namespace admin with none of their own on the repository is listed as `git.ns.admin`, which every adapter maps to no role. Each forge account is listed at most once.
- **There is no namespace-level `projectRoles`.** In `0.3` a `projectRoles` job without `repo` converged "the namespace itself" — its organisation owners, to which only `git.ns.admin` projected. Nothing projects there now, so `repo` is required for `projectRoles`, and a job without it is malformed.

A bridge now refuses a role map under which `git.repo.maintain` or `git.commit.sign` maps to the forge's administrator or owner role ([`desiredRoles`](#definitions)): only `git.repo.own` may. `0.3` left the map entirely to the community, so a maintainer — a right its holder may grant themselves — could be made an administrator of the repository on the forge.

`removeAccounts` may now name an account that `desiredRoles` lists at `git.ns.admin`: that entry asks for no role, and the removal says the same thing more strongly. It still may not name an account listed at any other right.

These are breaking changes, released as a `MINOR` increment under the `draft` allowance of [SPEC §5.2](/SPEC.md#52-compatibility-rules). A `0.3` document is a valid `0.4` document exactly when every `projectRoles` job names `repo`, no account appears twice in `desiredRoles`, and no `desiredRoles` entry lists a right the person holds only by implication. A `0.3` bridge would read a `git.ns.admin` entry as ownership — and one that processes later minor versions forward-compatibly ([SPEC §5.2](/SPEC.md#52-compatibility-rules)) would accept a `0.4` job to do so — so a VTC **MUST NOT** send a `0.4` job to a bridge that has not listed the `0.4` *Type URI* in answer to a [`trust-task-discovery`](/SPEC.md#10-discovery-and-capability-negotiation) request from that VTC, and a bridge that takes `0.4` **MUST** answer one. A VTC **MUST NOT** fall back to an earlier version for a bridge that does not list `0.4`: it sends that bridge nothing, and tells its operator to upgrade the bridge. Everything else is unchanged from `0.3` and restated below, so that this version stands on its own.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **being the VTC this bridge was deployed for**. A bridge serves exactly one VTC and **MUST** refuse, with `permissionDenied`, a job from any other issuer, whatever its proof. It does no authorization of its own beyond that: every rule about who may hold which right was applied by the VTC before the job was sent, and a bridge that second-guessed it would be a second source of truth. The `proof` is what lets the bridge tell its VTC's jobs from anyone else's, on any transport.

## Definitions

**`jobId`** — the VTC's identifier for the job, unique across its jobs.

**`namespace`** — the namespace the job acts in; it selects the forge credentials the bridge uses.

**`kind`** — what to do. Each kind uses a fixed set of the optional members, and a bridge **MUST** refuse a job that lacks one its kind requires or carries one its kind does not use, with the framework's `malformedRequest`:

| `kind` | Requires | May carry | Does |
|---|---|---|---|
| `projectRoles` | `repo`, `desiredRoles` | `removeAccounts` | converges forge roles on `repo`; removes the role of each of `removeAccounts` on `repo` |
| `createRepo` | `repo`, `spec` | `desiredRoles` | creates the repository, runs the bootstrap plan, projects roles |
| `bootstrap` | `repo` | `steps` | runs the bootstrap plan, or only `steps` of it |
| `archive` | `repo` | — | archives the repository |
| `inspect` | — | `repo` | compares the forge with the projection for `repo`, or sweeps the namespace |
| `beginBind` | `target` | — | starts the proof of control over `target` |
| `beginAccountLink` | `subject` | — | starts linking `subject`'s account on this forge |
| `closePullRequest` | `repo`, `number`, `message` | — | posts `message` as a comment on pull request `number` of `repo`, then closes it |

**`desiredRoles`** — the complete set of people the projection reaches on the repository, each with their linked account and the one right they are projected at there. The forge adapter maps each right to one of its roles (on GitHub, `own` → `admin`, `maintain` → `maintain`; on Forgejo, `maintain` → `write` plus the merge allow-list; `git.ns.admin` → none on every forge). People not listed lose any role the bridge manages; roles the bridge does not manage are reported as drift, not removed.

Only `git.repo.own` may map to the forge's administrator or owner role (`admin` on GitHub and Forgejo). A bridge **MUST** refuse, when it loads its configuration and whenever the map is changed, a role map under which `git.repo.maintain` or `git.commit.sign` maps to that role, and **MUST NOT** project with one: it would make a non-elevated right — one its holder may grant themselves — an administrator of the repository on the forge. On a personal account `git.repo.own` and `git.repo.maintain` both map to collaborator `write`, the only role a personal account can give, so the forge cannot tell an owner from a maintainer there: separation of duties is enforced at the VTC, not the forge.

The VTC counts the right listed only from the rights recorded for that person: `git.repo.own`, `git.repo.maintain` or `git.commit.sign` on the repository, or `git.commit.sign` on its namespace — the highest of them. It **MUST NOT** count what `git.ns.admin` implies. A namespace admin who holds a right of their own on the repository is listed at the highest of those, so one who is also recorded as its owner is listed as its owner. Creator ownership from a `git-ns/repo/create` authorised only by the `git.repo.create` that `git.ns.admin` implies does not arise (`git-ns/right/grant` 0.3), so a namespace admin is recorded as a repository's owner only by a grant someone else made, or by break-glass. A namespace admin who holds none is listed at `git.ns.admin`: listing them, rather than leaving them out, has the bridge take away a role it manages that they still hold on the repository, where leaving them out would leave it and report drift. A bridge **MUST** give an account listed at `git.ns.admin` no role on the repository.

Each forge account appears in `desiredRoles` at most once, matched by `forge` and `id`: an account has one role on a repository, and two entries for it would leave the bridge to choose between them.

**`removeAccounts`** — forge accounts whose direct role on `repo` the bridge removes, whatever the role is and whether or not the bridge manages it. Accounts are matched by `forge` and `id`; `login` is display only. An account that holds no role on `repo` is already converged. `removeAccounts` never touches the namespace itself — no job changes an organisation's owners — and it removes only a role held directly on the repository: access the account has through a team or as an organisation owner is not a role on the repository, and the bridge reports it as a failed `roles` step rather than change the team or the organisation.

**`steps`** — names from the adapter's bootstrap plan. The forge-neutral names are `create`, `workflow`, `keyring`, `variables`, `requiredCheck`, `roles` and `archive`; an adapter **MAY** add its own.

**`number`** — for `closePullRequest`: the number of the pull request in `repo`, as the `pullRequestOpened` event reported it.

**`message`** — for `closePullRequest`: Markdown, at most 16384 characters, that the bridge posts verbatim as a comment on the pull request. The VTC renders it from the template its pull-request policy names ([`git-ns/bridge/event`](../../../../git-ns/bridge/event/0.4/spec.md), *Pull-request policy*); the bridge does not interpret, template or translate it. The bridge **MAY** append a marker the forge does not display — on GitHub and Forgejo an HTML comment naming the `jobId` — so that it can recognise its own comment later (step 7). It adds nothing else.

**`next`** — for the two `begin*` kinds: where the VTC sends the person, a device-flow code where the forge has one, and when the chance lapses.

## Request

The VTC sends the job to its bridge. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming bridge:

1. Refuses a namespace it does not serve with `git-ns:unknownNamespace`.
2. Refuses a kind this forge or namespace cannot perform — `createRepo` on a personal account, `projectRoles` where it has no role management — with `git-ns/bridge/job:notCapable`.
3. Refuses a `jobId` it already holds with different content with `git-ns/bridge/job:jobIdReused`. Refuses, with the framework's `malformedRequest`, a `projectRoles` job without `repo`; a `desiredRoles` that lists one account twice; and a `removeAccounts` that names an account `desiredRoles` lists at a right other than `git.ns.admin`, or an account on another forge than the namespace's.
4. Otherwise records the job durably before answering `accepted: true`, and later sends exactly one [`git-ns/bridge/result`](../../../../git-ns/bridge/result/0.1/spec.md) for it. For a `jobId` it has already finished, answers `accepted: false`, does not run it again, and sends its result again: that is how a VTC that lost a result recovers it.
5. For `beginBind` and `beginAccountLink`, returns `next` with a single-use nonce bound to the job, and reports completion as a `bindCompleted` or `accountLinked` event, then its result. An attempt nobody completes ends in a `failed` result with error code `expired`.
6. Uses credentials scoped as narrowly as the forge allows — on GitHub, one installation token per job, limited to the job's repository where the API permits — and discards them after.
7. For `closePullRequest`, runs two steps, in order, and reports each in its [`git-ns/bridge/result`](../../../../git-ns/bridge/result/0.1/spec.md) under the step names `comment` and `close`:
   1. Reads the pull request. If it is **already closed** — closed by anyone, merged included — the bridge posts nothing and closes nothing, and reports `succeeded` with both steps `unchanged`. A closed pull request never gets a second comment.
   2. If the pull request is open and was **reopened after the job's `issuedAt` by an account other than the bridge's own**, the bridge posts nothing and closes nothing, and reports `succeeded` with both steps `unchanged` and a `detail` saying so. That reopen is a newer fact than the job, and the bridge reports it as a `pullRequestOpened` event for the VTC to judge; closing on the strength of the older job would defeat an owner's or maintainer's override.
   3. Otherwise, `comment`: posts `message` as a comment, unless a comment the bridge itself posted for this `jobId` is already there — it **MUST** be able to tell, by a durable record of the comments it posted for the job or by its marker on its own comment — in which case the step is `unchanged`. Then `close`: closes the pull request without merging it (`applied`).

   A retry after a failure therefore converges without a second comment: a job that commented and failed to close finds its comment, reports `comment` `unchanged`, and closes. The step outcomes follow the result's vocabulary: when `comment` fails, `close` is `skipped` and the outcome is `failed`; when `comment` succeeds and `close` fails, the outcome is `partial`. The error codes a bridge reports for this kind are those of [`git-ns/bridge/result`](../../../../git-ns/bridge/result/0.1/spec.md): `notFound` (there is no pull request `number` in `repo`, or the repository is gone), `forbidden` (its forge credentials may not comment on or close pull requests there), `notCapable` (the forge or repository has no pull requests, or they are disabled), `rateLimited`, and `forgeError`. A pull request that does not exist is `notFound`, never `succeeded`: the VTC asked to close something that is not there, and should know. On `succeeded`, a VTC records that the pull request was closed under its policy; on `failed` or `partial` it **MAY** send the job again, which is safe.

### Creating a repository

```json
{
  "id": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6d01",
  "type": "https://trusttasks.org/spec/git-ns/bridge/job/0.5",
  "threadId": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6d01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:00Z",
  "payload": {
    "jobId": "job_01J8ZA3K7F",
    "namespace": "ns_01J8Z6Q4M2",
    "kind": "createRepo",
    "repo": "github.com/acme/gadgets",
    "spec": {
      "visibility": "public",
      "description": "Small tools for widgets"
    },
    "desiredRoles": [
      {
        "subject": "did:webvh:QmBobScid2:acme-vtc.example:bob",
        "account": {
          "forge": "github.com",
          "id": "9120045",
          "login": "bob-builds"
        },
        "right": "git.repo.own"
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "zk5f5NdC3fyVGTjHt3wAfoua65baH7EMsYGngmCMHUp8sV5gfHLq5eCqkQi6wZn3HAfnsE8Rzu9XAaaK4nxDHRy"
  }
}
```

### Beginning an account link

```json
{
  "id": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6d03",
  "type": "https://trusttasks.org/spec/git-ns/bridge/job/0.5",
  "threadId": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6d03",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:00Z",
  "payload": {
    "jobId": "job_01J8ZB0P2C",
    "namespace": "ns_01J8Z6Q4M2",
    "kind": "beginAccountLink",
    "subject": "did:webvh:QmBobScid2:acme-vtc.example:bob"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "zsMQG7URHk2DRiFd9kuB5WojM61NFSJj7eLqEjFLvuU4LTcw2FSegdznkN8yDuRtdrVo1twjjPb4Ndfzh3sTqBV"
  }
}
```

### Taking a collaborator added on the forge off a repository

Someone gave `eve-dev`, who has no right on `widgets`, the `write` role on GitHub. An owner chose to revert it. `desiredRoles` is the repository's complete projection, as always: Alice and Carol are its recorded owners, and Dan, a namespace admin with no right of his own on `widgets`, is listed at `git.ns.admin`, so he gets no role there. `removeAccounts` names the account `desiredRoles` alone would leave in place, because nothing maps to `write` here.

```json
{
  "id": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6d07",
  "type": "https://trusttasks.org/spec/git-ns/bridge/job/0.5",
  "threadId": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6d07",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "issuedAt": "2026-09-24T09:10:00Z",
  "payload": {
    "jobId": "job_01J8ZF2R6W",
    "namespace": "ns_01J8Z6Q4M2",
    "kind": "projectRoles",
    "repo": "github.com/acme/widgets",
    "desiredRoles": [
      {
        "subject": "did:webvh:QmAliceScid1:acme-vtc.example:alice",
        "account": {
          "forge": "github.com",
          "id": "4410987",
          "login": "alice-acme"
        },
        "right": "git.repo.own"
      },
      {
        "subject": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
        "account": {
          "forge": "github.com",
          "id": "7781202",
          "login": "carol-c"
        },
        "right": "git.repo.own"
      },
      {
        "subject": "did:webvh:QmDanScid4:acme-vtc.example:dan",
        "account": {
          "forge": "github.com",
          "id": "6630415",
          "login": "dan-ops"
        },
        "right": "git.ns.admin"
      }
    ],
    "removeAccounts": [
      {
        "forge": "github.com",
        "id": "5550123",
        "login": "eve-dev"
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-24T09:10:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3sQm1bXv9Hk2TfWcN8rJ4pLdE6yG7uA5oKiZ2qRx1VnB8cM3wF9tS4hD6jP2eL7gU5aY1kQ3rT8mW4nX9vC2bZ"
  }
}
```

### Re-running one bootstrap step after drift

```json
{
  "id": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6d05",
  "type": "https://trusttasks.org/spec/git-ns/bridge/job/0.5",
  "threadId": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6d05",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:00Z",
  "payload": {
    "jobId": "job_01J8ZC9D4H",
    "namespace": "ns_01J8Z6Q4M2",
    "kind": "bootstrap",
    "repo": "github.com/acme/widgets",
    "steps": [
      "requiredCheck"
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "zfNBCVyyxc4YJ4LBc3MC4LswfMaAVgwC7Pv5mW2ZAnXCCP84g3KpfQX3UjZL63ovm2257Fbk78GdZNt2rZTCDPF"
  }
}
```

### Closing a pull request the policy does not allow

The community allows only committers to open pull requests on `widgets`. `eve-dev`, who is not one, opened pull request 42 ([`git-ns/bridge/event`](../../../../git-ns/bridge/event/0.4/spec.md), *A non-member opened a pull request from a fork*).

```json
{
  "id": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6d09",
  "type": "https://trusttasks.org/spec/git-ns/bridge/job/0.5",
  "threadId": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6d09",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "issuedAt": "2026-10-07T15:12:02Z",
  "payload": {
    "jobId": "job_01J9C4M7QX",
    "namespace": "ns_01J8Z6Q4M2",
    "kind": "closePullRequest",
    "repo": "github.com/acme/widgets",
    "number": 42,
    "message": "Thanks for your interest in **acme/widgets**. Pull requests here are open to the community's committers only, so this one has been closed automatically.\n\nTo contribute, ask to join at https://acme-vtc.example/join, or open an issue to discuss the change."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-10-07T15:12:02Z",
    "proofPurpose": "authentication",
    "proofValue": "z7Lq2Wn5Tb8Kx1Vd4Fm9Ys3Jc6Pg2Ue8Zo5Ti1Nn7Xb4Cq9Dw6Ek1Sh8Gv5Ly2Au4Bj7Kf9Tr6Np3Zm1Rc"
  }
}
```

Its result reports `succeeded`, with steps `comment` and `close` both `applied`; sent again, the same job reports both `unchanged`.

## Response

The bridge, now responding, says whether it took the job, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`.

### Accepted

```json
{
  "id": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6d02",
  "type": "https://trusttasks.org/spec/git-ns/bridge/job/0.5#response",
  "threadId": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6d01",
  "issuer": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:01Z",
  "payload": {
    "jobId": "job_01J8ZA3K7F",
    "accepted": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:01Z",
    "proofPurpose": "authentication",
    "proofValue": "zB6QrBqXENi973GjjCeVjVVUUDempgtCdj4GY4ztNUpL2ZvJv97ZVmZo66EkCYFZbu1YdaWPJnTj42jJ57qx6cv"
  }
}
```

### Accepted, with where to send the member

```json
{
  "id": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6d04",
  "type": "https://trusttasks.org/spec/git-ns/bridge/job/0.5#response",
  "threadId": "urn:uuid:95dedda2-126b-419d-a9c5-5babd36f6d03",
  "issuer": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:01Z",
  "payload": {
    "jobId": "job_01J8ZB0P2C",
    "accepted": true,
    "next": {
      "url": "https://github.com/login/device",
      "userCode": "WDJB-MJHT",
      "expiresAt": "2026-09-23T10:15:00Z"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmBridgeScid5:bridge.acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:01Z",
    "proofPurpose": "authentication",
    "proofValue": "z6xNLK38BZKGr3S86pfLF3E6y77dxH2i8Kv7mpPG7uVLBBDGgEac6CRC8T9UrHP9L8CSpUmp3yc3GETRsV2dB2N"
  }
}
```

## Security & Privacy

### Data carried

Jobs carry DIDs joined to forge accounts (`desiredRoles`), forge accounts to remove (`removeAccounts`) — often of people who are not members and have no DID the VTC knows — repository names and descriptions, and, for `closePullRequest`, a pull request's number and the community's message. That join is personal data, and it is why the bridge exists as a separate service: it holds the forge credentials and the account mappings it is sent, and nothing else of the community's. Jobs never carry credentials; the bridge **MUST NOT** accept one that tries (there is no member for it, and `additionalProperties` refuses it).

### Correlation

The VTC declares `identifierScope: public`, because it is the DID repositories name. The bridge declares `pairwise`: only its VTC needs to recognise it, and each community runs its own. Forge-side, every action the bridge takes is attributed to the community's app or bot, not to the member it acts for.

### Retention

A bridge keeps a job until its result is acknowledged, and its `jobId` and result long enough to answer a repeated job with `accepted: false` — at least as long as the VTC may retry. It **SHOULD NOT** keep `desiredRoles` beyond that: the VTC is the source of truth and sends the full set every time.

### Consent/purpose

The purpose is to make the forge match what the VTC decided. A `closePullRequest` `message` is public once posted: a VTC **MUST NOT** render into it anything about the author beyond what the forge already shows (their login), and in particular no DID, membership state or reason drawn from the VTC's records. A bridge **MUST NOT** use what jobs carry — forge accounts, DIDs, repository names — for anything else.
