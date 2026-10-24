---
slug: git-ns/repo/create
version: "0.4"
title: "Git Namespaces — Create Repository"
summary: "A holder of git.repo.create creates a repository and names its owners. A creator owns it only when their git.repo.create is an explicit record, never when it is implied by git.ns.admin. A bridge creates it, or the response lists manual steps."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - repository
  - github
  - forgejo
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
  rationale: "Creation makes someone the owner of a new repository, with authority over who may commit to it. It must be attributable to the requester on every transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A create replayed later is refused by name; placing it in time is what lets the VTC recognise it as a replay rather than a second request."
sideEffects:
  level: mutating
  rationale: "Reserves a name, creates a repository on the forge through the bridge, and records its owners. The repository is retired with git-ns/repo/archive."
consequences:
  - "Once the repository is active, its owners are published as owners, and as committers, in the community's Trust Registry."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries a namespace, a name, a visibility, a description and optionally the owners' DIDs. The response returns the repository record and, where needed, manual steps."
retention:
  class: durable
  rationale: "The repository record and its first owner right are kept for the life of the repository."
errorCodes:
  - code: git-ns:unknownNamespace
    meaning: "No namespace bound to this VTC has this identifier, or contains this resource."
    retryable: false
  - code: git-ns:namespaceNotBound
    meaning: "The namespace is still `pending`. Nothing is created, adopted or granted in it until binding completes."
    retryable: false
  - code: git-ns:policyDenied
    meaning: "The community's git-namespace policy refused the request after the fixed rules passed."
    retryable: false
  - code: git-ns:selfGrantNotAllowed
    meaning: "The requester would own the repository on the strength of a `git.repo.create` that is only implied by their `git.ns.admin`. Name another member in `owners`, or record `git.repo.create` for yourself once with git-ns/right/break-glass."
    retryable: false
  - code: git-ns:escalation
    meaning: "`owners` names someone other than the requester, and the requester holds no authority to grant `git.repo.own` in the namespace."
    retryable: false
  - code: git-ns:membersOnly
    meaning: "An owner named, or the requester, is not a current member of the community holding standing in the VTC's access-control records."
    retryable: false
  - code: git-ns/repo/create:nameTaken
    meaning: "This VTC already records a repository at this resource, in some state."
    retryable: false
related:
  - git-ns/repo/adopt
  - git-ns/repo/archive
  - git-ns/bridge/job
  - git-ns/view
  - git-ns/right/grant
  - git-ns/right/break-glass
---

## Abstract

A holder of `git.repo.create` on a namespace creates a repository in it and names its owners — by default, themselves. The VTC reserves the name at once. Where the namespace has a bridge that can create repositories — a GitHub organisation with the community's app, a Forgejo organisation with the community's bot — the bridge creates it and turns commit trust on: the verify-trust workflow, the trust-anchor variables, and a required check nobody can bypass. Where no bot can create repositories — a manual-mode namespace, or a personal account — the response lists the steps a person must take, and the repository becomes `active` when it is then adopted with [`git-ns/repo/adopt`](../../../../git-ns/repo/adopt/0.2/spec.md).

## Changes from 0.3

- **Nested repository paths.** A repository resource is `<forge-host>/<owner>/<repo-path>`. On a forge with nested groups (GitLab) the repository path holds the subgroups between the top-level owner and the project, up to 21 segments: `gitlab.com/acme/platform/tools/cli` is one repository of the namespace `gitlab.com/acme`. The namespace is still the top-level owner alone, and a subgroup is not a resource, so no right is held on one. A segment may be up to 255 characters.
- **`name` is still one segment.** This task creates a repository directly under the namespace's owner. A repository inside a subgroup is created on the forge and brought under the VTC with [`git-ns/repo/adopt`](../../adopt/0.3/spec.md).
- The schema pins [`git-ns/_shared/0.6`](../../../_shared/0.6/git-ns.schema.json) (was `0.4`).

Accepting what `0.3` refused is a backwards-compatible change, released as a `MINOR` increment ([SPEC §5.2](/SPEC.md#52-compatibility-rules)); a `0.3` document is a valid `0.4` document. A producer that also serves `0.3` **MUST NOT** put a resource `0.3` cannot carry — a repository path of more than one segment, or a segment longer than 100 characters — into a `0.3` document: a `0.3` consumer validates against the `0.3` schema and would refuse the whole document. It leaves such repositories, and the rights on them, out of a `0.3` answer instead. Everything else is unchanged from `0.3` and restated below, so that this version stands on its own.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **`git.repo.create` on the namespace**, held explicitly or implied by `git.ns.admin` ([the rights model](../../../../git-ns/right/grant/0.3/spec.md#the-rights-model)). Anyone else is refused with `permissionDenied`. The `proof` establishes who asked; the VTC's rights records decide whether they may.

Who may **own** what is created is narrower:

| The requester's `git.repo.create` | Requester may be an owner | May name other owners |
|---|---|---|
| an explicit record (granted by someone else, or a break-glass record, ratified or not) | yes | only with the authority to grant `git.repo.own` there |
| only implied by `git.ns.admin` | no — `git-ns:selfGrantNotAllowed`, unless the [single-administrator waiver](../../../right/grant/0.3/spec.md#the-single-administrator-waiver) applies | yes (a namespace admin may grant `git.repo.own`) |

An actor holding both an explicit record and `git.ns.admin` is treated as holding the explicit record. The requester and every owner must be current members of the community holding standing in the VTC's access-control records (`git-ns:membersOnly`); community policy cannot waive either rule.

## Definitions

**`namespace`** — where to create the repository.

**`name`** — the repository name, lowercase. The repository's resource is `<forge>/<owner>/<name>`.

**`visibility`**, **`description`** — how the forge shows it.

**`owners`** — who owns the repository. Absent, the requester alone.

**`manualSteps`** — what a person must do, in order, when no bot can. The wording and commands are the VTC's; the last step is always to adopt the repository.

## Request

The member sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Refuses an unknown namespace with `git-ns:unknownNamespace`, and a `pending` one with `git-ns:namespaceNotBound`.
2. Checks the authorization above: the entitlement (`permissionDenied`); then, for the owners (the requester, when `owners` is absent), separation of duties (`git-ns:selfGrantNotAllowed`, whose message **SHOULD** name [`git-ns/right/break-glass`](../../../right/break-glass/0.1/spec.md); where the [single-administrator waiver](../../../right/grant/0.3/spec.md#the-single-administrator-waiver) applies instead, the VTC meets every requirement it states — step-up, audit before the write, marking the owner record and the response — and continues), the authority to grant `git.repo.own` for any owner other than the requester (`git-ns:escalation`), and members only (`git-ns:membersOnly`). It then evaluates its policy (allowed visibilities, naming, who may create), which may refuse with `git-ns:policyDenied`.
3. Refuses a name whose resource it already records, in any state, with `git-ns/repo/create:nameTaken`.
4. Records the repository as `pendingCreate` and records `git.repo.own` on it for each owner, with `grantedBy` set to the requester. The owner rights are published to the Trust Registry only when the repository becomes `active`.
5. Where the namespace's bridge can create repositories, sends it a `createRepo` job ([`git-ns/bridge/job`](../../../../git-ns/bridge/job/0.3/spec.md)) carrying the desired roles. On a `succeeded` result the VTC records the forge id and bootstrap status and sets the repository `active`; on `partial` or `failed` it stays `pendingCreate` with the failing step recorded, and the VTC **MAY** send the job again — every step is check-then-apply.
6. Otherwise returns `manualSteps`.

The response describes the reservation. Creation on the forge is asynchronous, and a client learns its outcome from [`git-ns/view`](../../../../git-ns/view/0.4/spec.md).

### Bob creates a repository in a GitHub organisation

```json
{
  "id": "urn:uuid:ae64c15d-2cbf-491b-a20a-a04ed80c3601",
  "type": "https://trusttasks.org/spec/git-ns/repo/create/0.4",
  "threadId": "urn:uuid:ae64c15d-2cbf-491b-a20a-a04ed80c3601",
  "issuer": "did:webvh:QmBobScid2:acme-vtc.example:bob",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:00Z",
  "payload": {
    "namespace": "ns_01J8Z6Q4M2",
    "name": "gadgets",
    "visibility": "public",
    "description": "Small tools for widgets"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmBobScid2:acme-vtc.example:bob#key-1",
    "created": "2026-09-23T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3cYpu9WxGrCPqvwQVDTnKBwKBEMo1bKAzAwoNcDNvB9BTNNL82y5QmhQuGcrGJNpjKZ1pqZbrbr4CPxr1r2SC2"
  }
}
```

### A namespace admin creates one in a manual-mode namespace, for Bob

```json
{
  "id": "urn:uuid:ae64c15d-2cbf-491b-a20a-a04ed80c3603",
  "type": "https://trusttasks.org/spec/git-ns/repo/create/0.4",
  "threadId": "urn:uuid:ae64c15d-2cbf-491b-a20a-a04ed80c3603",
  "issuer": "did:webvh:QmAliceScid1:acme-vtc.example:alice",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:00Z",
  "payload": {
    "namespace": "ns_01J8Z7C1TX",
    "name": "sprockets",
    "visibility": "public",
    "owners": [
      "did:webvh:QmBobScid2:acme-vtc.example:bob"
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmAliceScid1:acme-vtc.example:alice#key-1",
    "created": "2026-09-23T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "zLmooyi3mJ1rQwMTCzQMPMijoR4MS75bwiek5Sqr5f91DRrJuTepibbCAKpEZjN1Am5arqnhmFZQSUW7bvv2gG7"
  }
}
```

## Response

The VTC, now responding, returns the repository, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`.

### Reserved; the bridge is creating it

```json
{
  "id": "urn:uuid:ae64c15d-2cbf-491b-a20a-a04ed80c3602",
  "type": "https://trusttasks.org/spec/git-ns/repo/create/0.3#response",
  "threadId": "urn:uuid:ae64c15d-2cbf-491b-a20a-a04ed80c3601",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmBobScid2:acme-vtc.example:bob",
  "issuedAt": "2026-09-23T10:00:01Z",
  "payload": {
    "repo": {
      "resource": "github.com/acme/gadgets",
      "visibility": "public",
      "state": "pendingCreate",
      "owners": [
        "did:webvh:QmBobScid2:acme-vtc.example:bob"
      ],
      "bootstrap": {
        "workflow": false,
        "keyring": false,
        "variables": false,
        "requiredCheck": false
      },
      "sync": {
        "state": "pending",
        "drift": []
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:01Z",
    "proofPurpose": "authentication",
    "proofValue": "zFPoKoLdEDjJrL4E5QUC88CFkBsBoHhFcpUDhvWAZFGpAxkVZ7Trye6JrW8HJb1R18UHnEbGKQxJLJuPpksX11u"
  }
}
```

### Reserved; a person must create it

```json
{
  "id": "urn:uuid:ae64c15d-2cbf-491b-a20a-a04ed80c3604",
  "type": "https://trusttasks.org/spec/git-ns/repo/create/0.3#response",
  "threadId": "urn:uuid:ae64c15d-2cbf-491b-a20a-a04ed80c3603",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmAliceScid1:acme-vtc.example:alice",
  "issuedAt": "2026-09-23T10:00:01Z",
  "payload": {
    "repo": {
      "resource": "codeberg.org/acme/sprockets",
      "visibility": "public",
      "state": "pendingCreate",
      "owners": [
        "did:webvh:QmBobScid2:acme-vtc.example:bob"
      ],
      "bootstrap": {
        "workflow": false,
        "keyring": false,
        "variables": false,
        "requiredCheck": false
      },
      "sync": {
        "state": "unchecked",
        "drift": []
      }
    },
    "manualSteps": [
      "Create the repository `acme/sprockets` on codeberg.org, public, with no initial commit.",
      "In a clone of it, run `vgi repo init --vtc did:webvh:QmVtcScid7:acme-vtc.example --resource codeberg.org/acme/sprockets` to commit the verify-trust workflow and set its variables and branch protection.",
      "Run `cnm git adopt codeberg.org/acme/sprockets` to tell the VTC the repository exists."
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:01Z",
    "proofPurpose": "authentication",
    "proofValue": "zS67v51YSfJ9GvcvW8itNsEm6Q5TDMo3Wo35PbTFSqjZAE1yQpFLjzqN51zyfFN4JxufZjYwsEXENzqVEtHrEUr"
  }
}
```

## Security & Privacy

### Data carried

A namespace identifier, a name, a visibility and an optional description in; the repository record and optional instructions out. A requester **MUST NOT** put in `description` anything they would not publish: on a public repository the forge shows it to everyone.

### Correlation

The VTC declares `identifierScope: public`, as the registry authority; the requester `pairwise`. Once the repository is active, its owner is published to the registry, so the requester's DID becomes publicly linked to it.

### Retention

Durable. The repository record and its first owner right are kept for the life of the repository, and in the audit history after.

### Separation of duties

A namespace admin holds `git.repo.create` by implication, so if creating made the creator an owner, every namespace admin could hold `git.repo.own` — and a forge admin role — on repositories nobody else had a say in, one per `create`. This version closes that: implied `git.repo.create` lets a namespace admin create a repository for someone else, never for themselves. A community with a single git administrator breaks the glass once, for `git.repo.create` on the namespace, which is audited and stays in front of every other administrator until ratified or revoked; the repositories created on that record are then the creator's. A VTC its host operator has configured for single-administrator operation, where nobody else could make the grant, **MAY** instead apply the [single-administrator waiver](../../../right/grant/0.3/spec.md#the-single-administrator-waiver) to each creation that would make the requester an owner; the owner record it writes counts toward the last-owner invariant. Two member DIDs held by one person are out of scope: a VTC compares DIDs, and whether members are distinct people is the membership process's question.

### Consent/purpose

The purpose is to create a governed repository. Whether creation warrants a step-up is the VTC's policy ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).
