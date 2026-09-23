---
slug: git-ns/view
version: "0.1"
title: "Git Namespaces — View"
summary: "A member reads the namespaces, repositories and git rights they are entitled to see, optionally narrowed to one resource; reasons and grant history only where they own or administer."
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
  requirement: RECOMMENDED
  rationale: "A read. The VTC identifies the caller from the transport or the proof; nothing is changed and the response is not evidence, so transport integrity suffices where the transport already authenticates the caller."
sideEffects:
  level: none
  rationale: "Reads the VTC's records and changes nothing."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns DIDs, rights, resources, forge ids and logins, and — only to owners and admins of the resource — free-text reasons."
retention:
  class: exchange
  rationale: "The request carries at most one resource and is needed only to answer it."
errorCodes: []
related:
  - git-ns/right/grant
  - git-ns/right/revoke
  - git-ns/repo/create
  - git-ns/namespace/bind
---

## Abstract

A member reads what the VTC governs on the forges: the namespaces bound to it, the repositories in them, and the rights the member is entitled to see — optionally narrowed to one resource. It is the one read behind every surface that shows git rights: an admin console's repository table, a member's *My repos* panel, a command-line listing.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

Not consequential, and declared for clarity. The entitlement is **membership of the community**: a caller the VTC does not resolve to a current member is refused with `permissionDenied`. A non-member holding a repository right through community policy — an outside contributor who signs commits — has no view through this task; what they hold is readable from the Trust Registry like anyone else's. What each member sees is then narrowed by their own rights, as below.

## Definitions

**`resource`** — narrows the answer to that resource and everything it contains, by whole-segment containment ([the fixed rules](../../../git-ns/right/grant/0.1/spec.md#the-fixed-rules)). A repository resource returns that repository and the rights on it; a namespace resource returns the namespace and everything in it.

## Request

The member sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC returns, within `resource` or everywhere when it is absent:

1. **`namespaces`** — every namespace bound to it that contains or is contained by `resource`, `pending` ones included.
2. **`repos`** — every repository it records, except that `unmanaged` repositories are returned only to callers holding `git.ns.admin` over them, the only people who can adopt them. Each repository carries its owners: who owns a repository is visible to every member.
3. **`rights`** — the recorded rights (never implied ones) the caller may see:
   - every right the caller holds;
   - on a resource where the caller holds `git.repo.own` or `git.ns.admin`, explicitly or by implication, every right on that resource.

   **`reason` is omitted** from every record except those on a resource where the caller holds `git.repo.own` or `git.ns.admin`. A member reading their own grant on someone else's repository does not see why it was made.

A resource that matches nothing yields empty lists, not an error. This family deliberately has no separate read-one task: a repository's existence on a forge is not the VTC's to assert, and an empty answer and a missing repository mean the same thing to a caller of this task — the VTC governs nothing there.

### Bob views the `acme` namespace

```json
{
  "id": "urn:uuid:ba0c0a3a-f777-47e9-af0c-75522e86cb01",
  "type": "https://trusttasks.org/spec/git-ns/view/0.1",
  "threadId": "urn:uuid:ba0c0a3a-f777-47e9-af0c-75522e86cb01",
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

Bob owns `gadgets`, so he sees every right on it, reasons included. On `widgets` he sees the owners and the drift, and none of its other rights.

```json
{
  "id": "urn:uuid:ba0c0a3a-f777-47e9-af0c-75522e86cb02",
  "type": "https://trusttasks.org/spec/git-ns/view/0.1#response",
  "threadId": "urn:uuid:ba0c0a3a-f777-47e9-af0c-75522e86cb01",
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
    ]
  }
}
```

## Security & Privacy

### Data carried

The response carries DIDs, rights, resources, forge ids, forge logins in drift reports, and free-text reasons. The DIDs and rights largely duplicate what the Trust Registry publishes anyway; the parts that do not — `grantedBy`, `reason`, drift, unmanaged repositories — are exactly the parts this specification gates.

### Correlation

The VTC declares `identifierScope: public`, as the authority members already know it by; the member `pairwise`. A caller can join the DIDs in the answer with the registry and with forge activity, which is the same join the registry already allows.

### Retention

The request is kept only as long as the VTC keeps request logs; nothing in it needs to outlive the exchange. The caller **SHOULD NOT** keep `reason`s it was shown beyond its need to act on them.

### Consent/purpose

The purpose is to let members see and manage the rights they hold or govern. A caller **MUST NOT** republish `reason`s or drift reports outside the community.
