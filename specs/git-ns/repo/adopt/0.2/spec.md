---
slug: git-ns/repo/adopt
version: "0.2"
title: "Git Namespaces — Adopt Repository"
summary: "A namespace admin brings an existing forge repository under the VTC's governance and names its owners — for repositories created outside the VTC, or to finish one a person had to create by hand."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - repository
  - adopt
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
  rationale: "Adoption makes the named DIDs owners of a repository, with authority over who may commit to it. It must be attributable to the actor on every transport."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "An adoption replayed after the repository was detached would re-attach it with the original owners."
sideEffects:
  level: mutating
  rationale: "Records the repository and its owners, publishes them to the Trust Registry, and has the bridge bootstrap the repository. Reversible in effect with git-ns/right/revoke and git-ns/repo/archive."
consequences:
  - "Each named owner is published as owner, and as a committer, of the repository in the community's Trust Registry."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries a resource and a list of DIDs; the response the repository record."
retention:
  class: durable
  rationale: "The repository record and its owner rights are kept for the life of the repository."
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
  - code: git-ns/repo/adopt:alreadyManaged
    meaning: "This VTC already manages the repository."
    retryable: false
related:
  - git-ns/repo/create
  - git-ns/bridge/job
  - git-ns/bridge/event
  - git-ns/right/grant
  - git-ns/view
---

## Abstract

Brings a repository that already exists on the forge under the VTC's governance, and names its owners. Two cases use it:

- a repository created outside the VTC — before the namespace was bound, or by someone clicking *New repository* on the forge — which the bridge reports as `unmanaged`;
- a repository reserved by [`git-ns/repo/create`](../../../../git-ns/repo/create/0.2/spec.md) in a namespace where a person had to create it, once they have.

In bridge mode the VTC then has the bridge inspect the repository and run whatever bootstrap steps are missing. In manual mode the steps are the adopter's to have done, typically with `vgi repo init`.

## Changes from 0.1

The schema pins [`git-ns/_shared/0.3`](../../../_shared/0.3/git-ns.schema.json), which narrows `Did` to the syntax of [W3C DID Core §3.1](https://www.w3.org/TR/did-core/#did-syntax). `0.1` pinned `_shared/0.1`, whose `Did` accepted anything without whitespace after `did:<method>:` — shell metacharacters, quotes, backticks, `/`, `?` and `#` included — so a value that was not a DID at all validated, and travelled on to every surface that later displayed, logged or quoted it. The members affected here are the request's `owners`, and `repo.owners` in the response. Each now carries a bare DID only: `did:`, a method name of lowercase letters and digits, and a method-specific id of colon-separated segments drawn from `A-Z a-z 0-9 . - _` and percent-encoded octets, the last one non-empty. A DID URL — a path, a query, or a `#` fragment such as a verification-method id — is refused: every one of these members names a party, never a key.

Narrowing a constraint is a breaking change, released as a `MINOR` increment under the `draft` allowance of [SPEC §5.2](/SPEC.md#52-compatibility-rules). A `0.1` document is a valid `0.2` document exactly when every DID it carries is well-formed. Everything else is unchanged from `0.1` and restated below, so that this version stands on its own.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **`git.ns.admin` on the namespace**, or, for a `pendingCreate` repository only, **an explicit `git.repo.own` record on it** — the reservation its creator holds, which lets the person who ran the manual steps finish the job. Anyone else is refused with `permissionDenied`. Naming owners is a grant of `git.repo.own` to each, and is subject to the same [fixed rules](../../../../git-ns/right/grant/0.2/spec.md#the-fixed-rules) and policy as [`git-ns/right/grant`](../../../../git-ns/right/grant/0.2/spec.md).

## Definitions

**`resource`** — the repository.

**`owners`** — the DIDs to record as owners. At least one, because a repository always has an owner. For a `pendingCreate` repository, the existing owners are kept and these are added.

## Request

The actor sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Refuses a resource inside no bound namespace with `git-ns:unknownNamespace`, and inside a `pending` one with `git-ns:namespaceNotBound`.
2. Refuses a repository it records as `active`, `orphaned` or `archived` with `git-ns/repo/adopt:alreadyManaged`. A `detached` repository — one whose namespace was unbound and bound again, or which moved back into the namespace — **MAY** be adopted again, and gets no rights back beyond `owners`.
3. Checks the authorization above, then policy for each owner, which may refuse with `git-ns:policyDenied`.
4. Records the repository as `active` and each owner's `git.repo.own`, with `grantedBy` set to the actor, and publishes them.
5. In bridge mode, sends the bridge an `inspect` job, then a `bootstrap` job for whatever the inspection finds missing ([`git-ns/bridge/job`](../../../../git-ns/bridge/job/0.3/spec.md)), and projects roles. Until the inspection's result arrives the repository has no `forgeId` and its sync state is `pending`. If the inspection finds no such repository, the VTC sets it `detached`.

### Adopting a repository created before the namespace was bound

```json
{
  "id": "urn:uuid:dfc51dc9-7f8e-497d-a198-3b85a75c6a01",
  "type": "https://trusttasks.org/spec/git-ns/repo/adopt/0.1",
  "threadId": "urn:uuid:dfc51dc9-7f8e-497d-a198-3b85a75c6a01",
  "issuer": "did:webvh:QmAliceScid1:acme-vtc.example:alice",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:00Z",
  "payload": {
    "resource": "github.com/acme/widgets",
    "owners": [
      "did:webvh:QmAliceScid1:acme-vtc.example:alice",
      "did:webvh:QmCarolScid3:acme-vtc.example:carol"
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmAliceScid1:acme-vtc.example:alice#key-1",
    "created": "2026-09-23T10:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "zLnCQGPuHG1wbA87XMKy7riryLHbPdwqoTSxN2DXECSFEEyPBkDU8UCc1epQdPWxM33xrr9PbbsJe7jGC1fXuqF"
  }
}
```

## Response

The VTC, now responding, returns the repository, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`.

### Adopted; the bridge is inspecting it

```json
{
  "id": "urn:uuid:dfc51dc9-7f8e-497d-a198-3b85a75c6a02",
  "type": "https://trusttasks.org/spec/git-ns/repo/adopt/0.1#response",
  "threadId": "urn:uuid:dfc51dc9-7f8e-497d-a198-3b85a75c6a01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmAliceScid1:acme-vtc.example:alice",
  "issuedAt": "2026-09-23T10:00:01Z",
  "payload": {
    "repo": {
      "resource": "github.com/acme/widgets",
      "visibility": "public",
      "state": "active",
      "owners": [
        "did:webvh:QmAliceScid1:acme-vtc.example:alice",
        "did:webvh:QmCarolScid3:acme-vtc.example:carol"
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
    "proofPurpose": "assertionMethod",
    "proofValue": "zuuoFLE7iAySistUC8tj1UmNrWU7ZFrgW5PSKCyn6bAKGuYwEi11NbLuPU3XvGHAp8V4ryuQg4FvDKzvXJD2YGw"
  }
}
```

## Security & Privacy

### Data carried

A resource and a list of DIDs in; the repository record out.

### Correlation

The VTC declares `identifierScope: public`, as the registry authority; the actor `pairwise`. Each owner's DID is published to the registry against the repository, publicly linking them.

### Retention

Durable, as for [`git-ns/repo/create`](../../../../git-ns/repo/create/0.2/spec.md).

### Consent/purpose

The purpose is to bring a repository under governance. Whether each named owner must agree first, and whether adoption warrants a step-up, are the VTC's policy ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).
