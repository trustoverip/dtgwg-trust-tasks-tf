---
slug: git-ns/repo/archive
version: "0.1"
title: "Git Namespaces — Archive Repository"
summary: "An owner archives a repository: the forge makes it read-only and the VTC revokes every commit-signing right on it."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - repository
  - archive
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
  rationale: "Archiving withdraws every commit right on a repository at once. It must be attributable to the owner on every transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "An archive replayed later is harmless — archiving is idempotent — but placing the request in time lets the VTC recognise it as one."
sideEffects:
  level: destructive
  rationale: "Revokes every git.commit.sign record on the repository and withdraws them from the Trust Registry. No git-ns task restores them or unarchives the repository."
consequences:
  - "Every commit-signing right on the repository is revoked; no git-ns task restores them."
  - "The forge makes the repository read-only."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries a resource; the response the repository record and a count."
retention:
  class: durable
  rationale: "The archive and the revocations it caused are part of the repository's audit history."
errorCodes:
  - code: git-ns:unknownRepo
    meaning: "The resource names no repository this VTC records."
    retryable: false
  - code: git-ns:repoNotActive
    meaning: "The repository's state does not allow this operation; each task says which states it accepts."
    retryable: false
  - code: git-ns:policyDenied
    meaning: "The community's git-namespace policy refused the request after the fixed rules passed."
    retryable: false
related:
  - git-ns/repo/create
  - git-ns/right/revoke
  - git-ns/bridge/job
  - git-ns/view
---

## Abstract

An owner retires a repository. The bridge archives it on the forge, which makes it read-only, and the VTC revokes every `git.commit.sign` record on it and withdraws them from the Trust Registry. Owner and maintainer records are kept, so the repository's history still says who governed it, but nothing more is trusted to land in it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **`git.repo.own` on the repository**, held explicitly or implied by `git.ns.admin`. Anyone else is refused with `permissionDenied`. The `proof` establishes who asked, never that they may.

## Definitions

**`resource`** — the repository.

**`rightsRevoked`** — how many `git.commit.sign` records the archive revoked.

## Request

The owner sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Refuses a resource it does not record with `git-ns:unknownRepo`. Refuses a `pendingCreate`, `detached` or `unmanaged` repository with `git-ns:repoNotActive`.
2. Checks the authorization above, and policy, which may refuse with `git-ns:policyDenied`.
3. For an already `archived` repository, returns it with `rightsRevoked: 0` and changes nothing. Repeating an archive is safe.
4. Otherwise sets the repository `archived`, revokes every `git.commit.sign` record whose resource is the repository, withdraws those and the explicit `git.commit.sign` published for owners and maintainers from its registry projection, and in bridge mode sends the bridge an `archive` job ([`git-ns/bridge/job`](../../../../git-ns/bridge/job/0.1/spec.md)).

This specification defines no task that reverses an archive. A repository unarchived on the forge stays `archived` in the VTC and is reported as drift.

### Archiving `widgets`

```json
{
  "id": "urn:uuid:201fbb2f-2ce8-495b-a1be-a9d86a385301",
  "type": "https://trusttasks.org/spec/git-ns/repo/archive/0.1",
  "threadId": "urn:uuid:201fbb2f-2ce8-495b-a1be-a9d86a385301",
  "issuer": "did:webvh:QmAliceScid1:acme-vtc.example:alice",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2027-03-01T12:00:00Z",
  "payload": {
    "resource": "github.com/acme/widgets"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmAliceScid1:acme-vtc.example:alice#key-1",
    "created": "2027-03-01T12:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "zN2rt1iKhWTjfKeYVjBvMrcibvzonCFnFopvdK8ezeFiL4Y9BXhnyBYnH3LpoD1jMnC8YQbMKU99KqLuPppsH9j"
  }
}
```

## Response

The VTC, now responding, returns the repository, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`.

### Archived

```json
{
  "id": "urn:uuid:201fbb2f-2ce8-495b-a1be-a9d86a385302",
  "type": "https://trusttasks.org/spec/git-ns/repo/archive/0.1#response",
  "threadId": "urn:uuid:201fbb2f-2ce8-495b-a1be-a9d86a385301",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmAliceScid1:acme-vtc.example:alice",
  "issuedAt": "2027-03-01T12:00:01Z",
  "payload": {
    "repo": {
      "resource": "github.com/acme/widgets",
      "forgeId": "812736451",
      "visibility": "public",
      "state": "archived",
      "owners": [
        "did:webvh:QmAliceScid1:acme-vtc.example:alice"
      ],
      "bootstrap": {
        "workflow": true,
        "keyring": true,
        "variables": true,
        "requiredCheck": true
      },
      "sync": {
        "state": "pending",
        "drift": []
      }
    },
    "rightsRevoked": 6
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2027-03-01T12:00:01Z",
    "proofPurpose": "authentication",
    "proofValue": "z2QGW1wZnYPVdPT1KsKoA38XFT47bQDDh8NJXEj3fsZh2JHgxa9GrM35JhnXEs9pKEgdZkvR7tkWbKkn33XiMtF"
  }
}
```

## Security & Privacy

### Data carried

A resource in; the repository record and a count out.

### Correlation

The VTC declares `identifierScope: public`, as the registry authority; the owner `pairwise`. The withdrawal of commit rights is visible to anyone who queries the registry.

### Retention

Durable. The archive and the revocations it caused stay in the repository's audit history.

### Consent/purpose

The purpose is to retire a repository. Whether archiving warrants a step-up is the VTC's policy ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).
