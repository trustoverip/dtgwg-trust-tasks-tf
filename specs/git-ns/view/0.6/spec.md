---
slug: git-ns/view
version: "0.6"
title: "Git Namespaces — View"
summary: "A member reads the namespaces, repositories and git rights they may see, and their own linked forge accounts. An administrator may read everything in the namespaces they administer, and either may narrow the answer to break-glass records."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - repositories
  - rights
  - administration
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
  rationale: "A read, but of records whose visibility depends on who asks: the VTC answers from the caller's own rights, and with `scope: administrator` from their administrator standing, so it must know from the document itself who the caller is. A proof binds the request to its `issuer` on every transport; a bearer session proves nothing about the document."
sideEffects:
  level: none
  rationale: "Reads the VTC's records and changes nothing: no audit row, counter or last-seen time is written because of it."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns DIDs, rights, resources, forge ids and logins, the caller's own linked forge accounts, and — only to owners and administrators of the resource — free-text reasons and break-glass justifications."
retention:
  class: exchange
  rationale: "The request carries at most one resource and two switches, and is needed only to answer it."
errorCodes:
  - code: git-ns/view:notAdministrator
    meaning: "`scope: administrator` was asked for, and the caller administers no namespace within `resource` — they hold neither the community-administrator capability nor a live `git.ns.admin` on such a namespace. Returned identically for a resource that names no namespace this VTC has, so the code cannot be used to learn which namespaces exist."
    retryable: false
related:
  - git-ns/right/grant
  - git-ns/right/break-glass
  - git-ns/right/ratify
  - git-ns/right/revoke
  - git-ns/repo/create
  - git-ns/namespace/bind
  - git-ns/namespace/list
  - git-ns/repo/list
  - git-ns/account/link
  - git-ns/drift/resolve
---

## Abstract

A member reads what the VTC governs on the forges: the namespaces bound to it, the repositories in them, and the rights the member is entitled to see — optionally narrowed to one resource — together with the forge accounts the member has linked to their own DID. It is the one read behind every surface that shows git rights: an admin console's repository table, a member's *My repos* panel with its linked-account row, a command-line listing, and the persistent banner that tells every administrator a break-glass is waiting for them.

## Changes from 0.5

- **Nested repository paths.** A repository resource is `<forge-host>/<owner>/<repo-path>`. On a forge with nested groups (GitLab) the repository path holds the subgroups between the top-level owner and the project, up to 21 segments: `gitlab.com/acme/platform/tools/cli` is one repository of the namespace `gitlab.com/acme`. The namespace is still the top-level owner alone, and a subgroup is not a resource, so no right is held on one. A segment may be up to 255 characters.
- The schema pins [`git-ns/_shared/0.6`](../../_shared/0.6/git-ns.schema.json) (was `0.4`).

Accepting what `0.5` refused is a backwards-compatible change, released as a `MINOR` increment ([SPEC §5.2](/SPEC.md#52-compatibility-rules)); a `0.5` document is a valid `0.6` document. A producer that also serves `0.5` **MUST NOT** put a resource `0.5` cannot carry — a repository path of more than one segment, or a segment longer than 100 characters — into a `0.5` document: a `0.5` consumer validates against the `0.5` schema and would refuse the whole document. It leaves such repositories, and the rights on them, out of a `0.5` answer instead. Everything else is unchanged from `0.5` and restated below, so that this version stands on its own.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

Not consequential, and declared for clarity. The caller is the DID the document's verified `proof` binds to its `issuer`, or the DID a key the VTC has recorded as delegated by that DID acts for; an unsigned document is refused with `proofRequired`. Nothing in the payload names or widens whose view is returned.

- **`scope: member`** (the default) — the entitlement is **membership of the community**: a caller the VTC does not resolve to a current member is refused with `permissionDenied`. A non-member holding a repository right through community policy — an outside contributor who signs commits — has no view through this task; what they hold is readable from the Trust Registry like anyone else's. What each member sees is then narrowed by their own rights, as below.
- **`scope: administrator`** — the entitlement is, for each namespace, **the community-administrator capability** in the VTC's own access control (the one [`git-ns/namespace/bind`](../../namespace/bind/0.1/spec.md) requires), or **`git.ns.admin` on that namespace** held by explicit record and live at the instant the VTC evaluates the request. The caller must be a current member, as for the default scope. A caller entitled to no namespace within `resource` is refused with `git-ns/view:notAdministrator`. A context-scoped administrator of the VTC who holds no `git.ns.admin` administers no namespace here: the capability that counts is the community-wide one.

The linked accounts returned are the caller's own under either scope, and no right or capability — `git.ns.admin` and the community-administrator capability included — widens that to anyone else's.

## Definitions

**`resource`** — narrows the answer to that resource and everything it contains, by whole-segment containment ([the fixed rules](../../right/grant/0.3/spec.md#the-fixed-rules)). A repository resource returns that repository and the rights on it; a namespace resource returns the namespace and everything in it. For `accounts`, it narrows to the resource's forge.

**Administered namespace** — for a holder of the community-administrator capability, every namespace bound to the VTC, `pending` ones included. For anyone else, each namespace on which they hold a live, explicitly recorded `git.ns.admin`.

**`scope`** — `member`, the default, or `administrator`. It chooses which entitlement answers the request ([Authorization](#authorization)); it never changes the response's shape.

**`breakGlass`** — `true` narrows the answer to break-glass records: those carrying `breakGlass` ([`git-ns/right/break-glass`](../../right/break-glass/0.1/spec.md)), ratified or not.

**`accounts`** — the forge accounts linked to the caller's DID, at most one per forge, each with `linkedAt`, the instant the VTC recorded the link. A link attempt still `pending` is not an account.

## Request

The member sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json).

### `scope: member`

A conforming VTC returns, within `resource` or everywhere when it is absent:

1. **`namespaces`** — every namespace bound to it that contains or is contained by `resource`, `pending` ones included.
2. **`repos`** — every repository it records, except that `unmanaged` repositories are returned only to callers holding `git.ns.admin` over them, the only people who can adopt them. Each repository carries its owners: who owns a repository is visible to every member.
3. **`rights`** — the recorded rights (never implied ones) the caller may see:
   - every right the caller holds;
   - on a resource where the caller holds `git.repo.own` or `git.ns.admin`, explicitly or by implication, every right on that resource;
   - every **unratified break-glass record** — one carrying `breakGlass` with no `ratifiedBy` — in a namespace where the caller holds the community-administrator capability or `git.ns.admin`, or on a resource where the caller holds `git.repo.own`, explicitly or by implication. A VTC **MUST NOT** withhold these from those callers for any reason, community policy included.
4. **`accounts`** — every forge account linked to the caller's own DID; with `resource`, only the one on `resource`'s forge. A VTC **MUST NOT** return an account linked to any other DID in this member, and **MUST** return an empty array when the caller has none.

### `scope: administrator`

A conforming VTC first determines the administered namespaces that contain or are contained by `resource` (all administered namespaces when it is absent). When there are none and the caller does not hold the community-administrator capability, it refuses with `git-ns/view:notAdministrator` — for a `resource` that names no namespace it has as for one that names a namespace the caller does not administer, so the refusal says nothing about which namespaces exist. Otherwise it returns, within `resource`:

1. **`namespaces`** — those administered namespaces.
2. **`repos`** — every repository it records in them, `unmanaged` ones included. To a holder of the community-administrator capability it also returns every repository within `resource` whose namespace is no longer bound to it: nobody else administers those, and they are what that administrator decides whether to bind again.
3. **`rights`** — every recorded right (never an implied one) on those namespaces and on the repositories returned, that is live or is an unratified break-glass record.
4. **`accounts`** — as for `scope: member`: the caller's own, and nobody else's.

A holder of the community-administrator capability administers every namespace, so for them a `resource` that matches nothing yields empty lists, not an error.

### Under both scopes

**`reason` is omitted** from every record except those on a resource where the caller holds `git.repo.own` or `git.ns.admin`, explicitly or by implication, or — under `scope: administrator` — on a resource in an administered namespace. A member reading their own grant on someone else's repository does not see why it was made. **`breakGlass`** is returned in full, justification included, on every record that carries it — ratified ones included, since it is the record's history. Everyone this task shows such a record to is an administrator of its namespace, an owner of its resource, or the member who broke the glass, all of whom the justification was written for.

**With `breakGlass: true`**, `rights` holds only the records that carry `breakGlass`, and `namespaces` and `repos` only those that contain one of them; `accounts` is unchanged. Under `scope: administrator` this is every break-glass record in the administered namespaces, ratified ones included; a client shows a record whose `breakGlass` has no `ratifiedBy` as awaiting ratification — or, while its `effectiveAt` is in the future, as not yet in effect — and a ratified one as history.

A resource that matches nothing the caller may see yields empty lists, not an error, except where `scope: administrator` refuses as above. This family deliberately has no separate read-one task: a repository's existence on a forge is not the VTC's to assert, and an empty answer and a missing repository mean the same thing to a caller of this task — the VTC governs nothing there.

### Bob views the `acme` namespace

```json
{
  "id": "urn:uuid:4c1f0e52-9d7b-4a0e-8f15-2b6f0d1c7a01",
  "type": "https://trusttasks.org/spec/git-ns/view/0.6",
  "threadId": "urn:uuid:4c1f0e52-9d7b-4a0e-8f15-2b6f0d1c7a01",
  "issuer": "did:webvh:QmBobScid2:acme-vtc.example:bob",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-24T08:00:00Z",
  "payload": {
    "resource": "github.com/acme"
  }
}
```

### Dana reviews the break-glass records she must ratify

Dana holds the community-administrator capability and no git right. She asks for the break-glass records in every namespace.

```json
{
  "id": "urn:uuid:4c1f0e52-9d7b-4a0e-8f15-2b6f0d1c7a11",
  "type": "https://trusttasks.org/spec/git-ns/view/0.6",
  "threadId": "urn:uuid:4c1f0e52-9d7b-4a0e-8f15-2b6f0d1c7a11",
  "issuer": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-26T08:00:00Z",
  "payload": {
    "scope": "administrator",
    "breakGlass": true
  }
}
```

## Response

The VTC, now responding, returns what the caller may see, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`: `proofRequired` for an unsigned document, `permissionDenied` for a caller who is not a current member, and `git-ns/view:notAdministrator` as above.

### What Bob sees

Bob owns `gadgets`, so he sees every right on it, reasons included. On `widgets` he sees the owners and the drift, and none of its other rights. The request named a `github.com` resource, so `accounts` carries only his GitHub account, even though he has also linked one on Codeberg.

```json
{
  "id": "urn:uuid:4c1f0e52-9d7b-4a0e-8f15-2b6f0d1c7a02",
  "type": "https://trusttasks.org/spec/git-ns/view/0.5#response",
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

### What Dana sees

Carol, the only admin of `acme`, was away when a release had to ship, and Bob gave himself `git.ns.admin` with [`git-ns/right/break-glass`](../../right/break-glass/0.1/spec.md). Nobody has ratified it yet, so Dana's console shows it as awaiting her. Dana linked no forge account.

```json
{
  "id": "urn:uuid:4c1f0e52-9d7b-4a0e-8f15-2b6f0d1c7a12",
  "type": "https://trusttasks.org/spec/git-ns/view/0.5#response",
  "threadId": "urn:uuid:4c1f0e52-9d7b-4a0e-8f15-2b6f0d1c7a11",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "issuedAt": "2026-09-26T08:00:01Z",
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
    "repos": [],
    "rights": [
      {
        "subject": "did:webvh:QmBobScid2:acme-vtc.example:bob",
        "right": "git.ns.admin",
        "resource": "github.com/acme",
        "grantedBy": "did:webvh:QmBobScid2:acme-vtc.example:bob",
        "grantedAt": "2026-09-25T22:14:03Z",
        "breakGlass": {
          "by": "did:webvh:QmBobScid2:acme-vtc.example:bob",
          "at": "2026-09-25T22:14:03Z",
          "justification": "Release 2.4 must ship tonight and Carol, our only admin, is on a flight until Monday."
        }
      }
    ],
    "accounts": []
  }
}
```

## Security & Privacy

### Data carried

The response carries DIDs, rights, resources, forge ids, forge logins in drift reports, free-text reasons and break-glass justifications, and the caller's own linked forge accounts. The DIDs and rights largely duplicate what the Trust Registry publishes anyway; the parts that do not — `grantedBy`, `reason`, `breakGlass`, drift, unmanaged repositories, linked accounts — are exactly the parts this specification gates. `accounts` returns to a member only the join between their own DID and their own forge account, which they made themselves with [`git-ns/account/link`](../../account/link/0.1/spec.md); it discloses nothing about anyone else.

### The administrator's read

`scope: administrator` discloses every reason and every justification in the namespaces the caller administers, and only there. That is no more than a namespace admin already sees of their own namespace under the default scope; what it adds is the same view for a holder of the community-administrator capability, who holds no git right and is accountable for every namespace. It is gated on a proof, never on a session: the reasons written for a namespace's administrators should reach a person the VTC can name, not whoever presents a bearer token. A VTC **MUST NOT** answer `scope: administrator` on the strength of an administrator role scoped to less than the whole community, and **MUST NOT** return anything outside the administered namespaces under it — a namespace admin's administrator read of the whole VTC is their own namespaces, and not a refusal only because it is useful to them. `notAdministrator` is answered the same way for a namespace that does not exist as for one the caller does not administer.

### Correlation

The VTC declares `identifierScope: public`, as the authority members already know it by; the member `pairwise`. A caller can join the DIDs in the answer with the registry and with forge activity, which is the same join the registry already allows. The member-to-forge-account join in `accounts` is the one piece of the answer the registry does not publish, and is why it is only ever returned to the member it describes, under either scope. Like the rest of the answer, it is exposed to anyone who can read the transport, which is the transport binding's concern rather than this task's.

### Break-glass records

A break-glass is justified by nobody else being available, and the flag is how the people who *were* available find out. So every unratified one is returned to every administrator of its namespace even where this task would otherwise show them nothing — a community administrator holds no git right, and sees these anyway — and to the resource's owners, whose repository it concerns. The justification goes to the same people; it is free text its author wrote knowing they would read it. `breakGlass: true` narrows what is returned and never widens it: it cannot show a record the same request without it would not.

### Retention

The request is kept only as long as the VTC keeps request logs; nothing in it needs to outlive the exchange. The caller **SHOULD NOT** keep `reason`s or justifications it was shown beyond its need to act on them.

### Consent/purpose

The purpose is to let members see and manage the rights they hold or govern, and the accounts through which those rights reach the forge, and to let administrators oversee the namespaces they are accountable for. A caller **MUST NOT** republish `reason`s, justifications or drift reports outside the community.
