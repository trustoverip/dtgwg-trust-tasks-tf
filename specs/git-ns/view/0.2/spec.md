---
slug: git-ns/view
version: "0.2"
title: "Git Namespaces — View"
summary: "A member reads the namespaces, repositories and git rights they are entitled to see, optionally narrowed to one resource, and the forge accounts linked to their own DID; reasons and grant history only where they own or administer."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - repositories
  - rights
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
  rationale: "A read, but of one member's records: the VTC answers the caller about their own namespaces and links, so it must know from the document itself who the caller is. A proof binds the request to its `issuer` on every transport."
sideEffects:
  level: none
  rationale: "Reads the VTC's records and changes nothing."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns DIDs, rights, resources, forge ids and logins, the caller's own linked forge accounts, and — only to owners and admins of the resource — free-text reasons."
retention:
  class: exchange
  rationale: "The request carries at most one resource and is needed only to answer it."
errorCodes: []
related:
  - git-ns/right/grant
  - git-ns/right/revoke
  - git-ns/repo/create
  - git-ns/namespace/bind
  - git-ns/account/link
  - git-ns/drift/resolve
---

## Abstract

A member reads what the VTC governs on the forges: the namespaces bound to it, the repositories in them, and the rights the member is entitled to see — optionally narrowed to one resource — together with the forge accounts the member has linked to their own DID. It is the one read behind every surface that shows git rights: an admin console's repository table, a member's *My repos* panel with its linked-account row, a command-line listing.

## Changes from 0.1

The response carries one new member, **`accounts`**: the forge accounts linked to the caller's own DID with [`git-ns/account/link`](../../../git-ns/account/link/0.1/spec.md), each with when the link completed. `0.1` gave a member no way to read back which account they had linked — [`git-ns/account/link-status`](../../../git-ns/account/link-status/0.1/spec.md) answers only for an attempt whose `linkId` the caller still holds — so a surface that shows "linked as `bob-builds`" or offers **Link** had to remember the answer itself.

Only the caller's own accounts are returned, never another member's, whoever the caller is. `accounts` is **required**, and empty when the caller has linked none, so that "no linked account" is an answer rather than an absent member a client cannot tell from a `0.1` response. Adding a required response member is released as a `MINOR` increment under the `draft` allowance of [SPEC §5.2](/SPEC.md#52-compatibility-rules).

The schema pins [`git-ns/_shared/0.2`](../../_shared/0.2/git-ns.schema.json), which is wire-identical to `0.1` for every shape this task carries. Everything else — the request, `namespaces`, `repos`, `rights` and the rules that gate them — is unchanged from `0.1` and restated below, so that this version stands on its own.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

Not consequential, and declared for clarity. The entitlement is **membership of the community**: a caller the VTC does not resolve to a current member is refused with `permissionDenied`. A non-member holding a repository right through community policy — an outside contributor who signs commits — has no view through this task; what they hold is readable from the Trust Registry like anyone else's. What each member sees is then narrowed by their own rights, as below. The linked accounts returned are the caller's own, and no right — `git.ns.admin` included — widens that to anyone else's.

## Definitions

**`resource`** — narrows the answer to that resource and everything it contains, by whole-segment containment ([the fixed rules](../../../git-ns/right/grant/0.1/spec.md#the-fixed-rules)). A repository resource returns that repository and the rights on it; a namespace resource returns the namespace and everything in it. For `accounts`, it narrows to the resource's forge.

**`accounts`** — the forge accounts linked to the caller's DID, at most one per forge, each with `linkedAt`, the instant the VTC recorded the link. A link attempt still `pending` is not an account.

## Request

The member sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC returns, within `resource` or everywhere when it is absent:

1. **`namespaces`** — every namespace bound to it that contains or is contained by `resource`, `pending` ones included.
2. **`repos`** — every repository it records, except that `unmanaged` repositories are returned only to callers holding `git.ns.admin` over them, the only people who can adopt them. Each repository carries its owners: who owns a repository is visible to every member.
3. **`rights`** — the recorded rights (never implied ones) the caller may see:
   - every right the caller holds;
   - on a resource where the caller holds `git.repo.own` or `git.ns.admin`, explicitly or by implication, every right on that resource.

   **`reason` is omitted** from every record except those on a resource where the caller holds `git.repo.own` or `git.ns.admin`. A member reading their own grant on someone else's repository does not see why it was made.
4. **`accounts`** — every forge account linked to the caller's own DID; with `resource`, only the one on `resource`'s forge. A VTC **MUST NOT** return an account linked to any other DID in this member, and **MUST** return an empty array when the caller has none.

A resource that matches nothing yields empty lists, not an error. This family deliberately has no separate read-one task: a repository's existence on a forge is not the VTC's to assert, and an empty answer and a missing repository mean the same thing to a caller of this task — the VTC governs nothing there.

### Bob views the `acme` namespace

```json
{
  "id": "urn:uuid:4c1f0e52-9d7b-4a0e-8f15-2b6f0d1c7a01",
  "type": "https://trusttasks.org/spec/git-ns/view/0.2",
  "threadId": "urn:uuid:4c1f0e52-9d7b-4a0e-8f15-2b6f0d1c7a01",
  "issuer": "did:webvh:QmBobScid2:acme-vtc.example:bob",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-24T08:00:00Z",
  "payload": {
    "resource": "github.com/acme"
  }
}
```

## Response

The VTC, now responding, returns what the caller may see, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error` with `permissionDenied`.

### What Bob sees

Bob owns `gadgets`, so he sees every right on it, reasons included. On `widgets` he sees the owners and the drift, and none of its other rights. The request named a `github.com` resource, so `accounts` carries only his GitHub account, even though he has also linked one on Codeberg.

```json
{
  "id": "urn:uuid:4c1f0e52-9d7b-4a0e-8f15-2b6f0d1c7a02",
  "type": "https://trusttasks.org/spec/git-ns/view/0.2#response",
  "threadId": "urn:uuid:4c1f0e52-9d7b-4a0e-8f15-2b6f0d1c7a01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmBobScid2:acme-vtc.example:bob",
  "issuedAt": "2026-09-24T08:00:01Z",
  "payload": {
    "namespaces": [
      {
        "id": "ns_01J8Z6Q4M2",
        "forge": "github.com",
        "owner": "acme",
        "kind": "organization",
        "mode": "bridge",
        "state": "bound"
      }
    ],
    "repos": [
      {
        "resource": "github.com/acme/widgets",
        "forgeId": "812736451",
        "visibility": "public",
        "state": "active",
        "owners": [
          "did:webvh:QmAliceScid1:acme-vtc.example:alice",
          "did:webvh:QmCarolScid3:acme-vtc.example:carol"
        ],
        "bootstrap": {
          "workflow": true,
          "keyring": true,
          "variables": true,
          "requiredCheck": true
        },
        "sync": {
          "state": "drift",
          "checkedAt": "2026-09-23T09:58:00Z",
          "drift": [
            {
              "type": "roleAdded",
              "resource": "github.com/acme/widgets",
              "account": {
                "forge": "github.com",
                "id": "5550123",
                "login": "eve-dev"
              },
              "observed": "write"
            }
          ]
        }
      },
      {
        "resource": "github.com/acme/gadgets",
        "forgeId": "812736990",
        "visibility": "public",
        "state": "active",
        "owners": [
          "did:webvh:QmBobScid2:acme-vtc.example:bob"
        ],
        "bootstrap": {
          "workflow": true,
          "keyring": true,
          "variables": true,
          "requiredCheck": true
        },
        "sync": {
          "state": "inSync",
          "checkedAt": "2026-09-23T09:58:00Z",
          "drift": []
        }
      }
    ],
    "rights": [
      {
        "subject": "did:webvh:QmBobScid2:acme-vtc.example:bob",
        "right": "git.repo.create",
        "resource": "github.com/acme",
        "grantedBy": "did:webvh:QmAliceScid1:acme-vtc.example:alice",
        "grantedAt": "2026-09-23T10:00:01Z"
      },
      {
        "subject": "did:webvh:QmBobScid2:acme-vtc.example:bob",
        "right": "git.repo.own",
        "resource": "github.com/acme/gadgets",
        "grantedBy": "did:webvh:QmBobScid2:acme-vtc.example:bob",
        "grantedAt": "2026-09-23T10:05:00Z"
      },
      {
        "subject": "did:webvh:QmDanScid4:dan.example",
        "right": "git.commit.sign",
        "resource": "github.com/acme/gadgets",
        "grantedBy": "did:webvh:QmBobScid2:acme-vtc.example:bob",
        "grantedAt": "2026-09-23T11:00:01Z",
        "expiresAt": "2026-12-22T00:00:00Z",
        "reason": "External contributor for the 1.0 push"
      }
    ],
    "accounts": [
      {
        "account": {
          "forge": "github.com",
          "id": "9120045",
          "login": "bob-builds"
        },
        "linkedAt": "2026-09-23T10:02:14Z"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The response carries DIDs, rights, resources, forge ids, forge logins in drift reports, free-text reasons, and the caller's own linked forge accounts. The DIDs and rights largely duplicate what the Trust Registry publishes anyway; the parts that do not — `grantedBy`, `reason`, drift, unmanaged repositories, linked accounts — are exactly the parts this specification gates. `accounts` returns to a member only the join between their own DID and their own forge account, which they made themselves with [`git-ns/account/link`](../../../git-ns/account/link/0.1/spec.md); it discloses nothing about anyone else.

### Correlation

The VTC declares `identifierScope: public`, as the authority members already know it by; the member `pairwise`. A caller can join the DIDs in the answer with the registry and with forge activity, which is the same join the registry already allows. The member-to-forge-account join in `accounts` is the one piece of the answer the registry does not publish, and is why it is only ever returned to the member it describes. Like the rest of the answer, it is exposed to anyone who can read the transport, which is the transport binding's concern rather than this task's.

### Retention

The request is kept only as long as the VTC keeps request logs; nothing in it needs to outlive the exchange. The caller **SHOULD NOT** keep `reason`s it was shown beyond its need to act on them.

### Consent/purpose

The purpose is to let members see and manage the rights they hold or govern, and the accounts through which those rights reach the forge. A caller **MUST NOT** republish `reason`s or drift reports outside the community.
