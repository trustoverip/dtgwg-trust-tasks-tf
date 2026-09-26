---
slug: git-ns/right/grant
version: "0.1"
title: "Git Namespaces — Grant Right"
summary: "A member grants one git right — namespace admin, repo creator, owner, maintainer or committer — on one forge-qualified resource. The VTC checks it against the actor's own rights and fixed rules no policy can loosen, records it, and publishes it to its Trust Registry."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - github
  - forgejo
  - delegation
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
  rationale: "A grant changes whose commits a repository's CI check accepts and who may govern it. It must be attributable to the actor on every transport, and it is the record an audit of the resource reads back."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A grant replayed after the right was revoked would restore it. Placing the request in time is what lets the VTC refuse the replay."
sideEffects:
  level: mutating
  rationale: "Records a right and publishes it to the VTC's Trust Registry, and through a bridge to the forge's roles. Reversible with git-ns/right/revoke."
consequences:
  - "The right is published to the community's Trust Registry, where anyone can read that the subject holds it on the resource."
  - "Granting git.repo.own or git.repo.maintain also publishes an explicit git.commit.sign for the subject on the same repository."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries a DID, a right, a resource and optional free text. The response returns the same record."
retention:
  class: durable
  rationale: "The grant is part of the audit history of the resource: whether a commit was trusted when it was made is answered from it."
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
  - code: git-ns:scopeViolation
    meaning: "The resource is wider than the actor's own scope, or is the wrong level for the right: a repository right named on a namespace, or a namespace right named on a repository."
    retryable: false
  - code: git-ns:escalation
    meaning: "No right the actor holds on this resource carries the authority to grant or revoke this right."
    retryable: false
  - code: git-ns:membersOnly
    meaning: "`git.ns.admin` and `git.repo.create` go only to current members of the community, and the subject is not one."
    retryable: false
  - code: git-ns:policyDenied
    meaning: "The community's git-namespace policy refused the request after the fixed rules passed."
    retryable: false
  - code: git-ns/right/grant:expiryInPast
    meaning: "`expiresAt` is not in the future."
    retryable: false
related:
  - git-ns/right/revoke
  - git-ns/view
  - git-ns/namespace/bind
  - git-ns/repo/create
  - git-trust/grant
  - registry/authorization
---

## Abstract

A VTC can govern repositories on a forge — GitHub, a Forgejo instance such as Codeberg — through **namespaces** bound with [`git-ns/namespace/bind`](../../../../git-ns/namespace/bind/0.1/spec.md). Who may do what inside a namespace is a set of **rights**, each held by one DID on one forge-qualified resource. The VTC is the source of truth for them. It publishes them to its Trust Registry, where CI checks such as `did-git-sign verify-trust` read them, and a bridge projects them onto the forge's own roles.

This task grants one right. The VTC checks the grant against the rights the actor already holds, against a fixed set of rules no policy can loosen, and against the community's own policy. It then records the right and publishes it.

This specification also carries the family's **rights model**, which every other `git-ns` task refers to.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **a right the actor already holds, on a resource that contains the grant's resource, whose grant authority covers the right being granted** — the grant authority table below. Membership of the community is not enough, and neither is the community-administrator capability: a VTC administrator who holds no git right grants nothing through this task. A namespace's first `git.ns.admin` comes from binding it ([`git-ns/namespace/bind`](../../../../git-ns/namespace/bind/0.1/spec.md)); every later right traces back to it.

Per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10, the `proof` establishes which DID is acting. The VTC then resolves that DID to its own rights records; the proof is never the authority itself.

## The rights model

### Rights

| Right | Resource | Holder may |
|---|---|---|
| `git.ns.admin` | namespace (`github.com/acme`) | everything below on every repository in the namespace; adopt repositories; unbind the namespace |
| `git.repo.create` | namespace | create a repository in the namespace with [`git-ns/repo/create`](../../../../git-ns/repo/create/0.1/spec.md), becoming its owner |
| `git.repo.own` | repository (`github.com/acme/widgets`) | grant and revoke `own`, `maintain` and `commit.sign` on this repository; transfer ownership; archive |
| `git.repo.maintain` | repository | merge and triage on the forge |
| `git.commit.sign` | repository or namespace | author commits the CI check accepts |

The right strings are carried verbatim: each is also the TRQP `action` under which the VTC publishes the right, with `authority` set to the VTC's DID.

**Implied rights.** `git.repo.own` implies `git.repo.maintain`, which implies `git.commit.sign`, on the same resource. `git.ns.admin` implies `git.repo.create` on its namespace and `git.repo.own` on every repository in it. Implication is evaluated by the VTC and is never a record. Because verifiers query only `git.commit.sign`, the VTC's registry projection **MUST** publish the implied `git.commit.sign` of every recorded `git.repo.own` and `git.repo.maintain` explicitly, on the same repository resource, and of every recorded `git.ns.admin` on its namespace resource. A namespace-level `git.commit.sign` counts for a repository only where the repository's check also queries its namespace as the fallback resource, so a bridge that sets up the check **MUST** configure that fallback.

### Grant authority

| The actor holds | May grant |
|---|---|
| `git.ns.admin` on a namespace | `git.ns.admin` and `git.repo.create` on that namespace; any right on a repository in it; `git.commit.sign` on the namespace |
| `git.repo.create` on a namespace | nothing — it is not re-delegable |
| `git.repo.own` on a repository | `git.repo.own`, `git.repo.maintain` and `git.commit.sign` on that repository only |
| `git.repo.maintain` on a repository | nothing, unless the community has configured maintainers to grant `git.commit.sign` on that repository |

Implied rights carry their authority: a namespace admin has an owner's authority on every repository in the namespace.

### The fixed rules

A VTC **MUST** enforce these in its own code, before and independently of policy. Community policy cannot waive any of them.

1. **Scope containment.** Containment is by whole path segment: `github.com/acme` contains `github.com/acme/widgets` and itself; it does not contain `github.com/acme-labs/x` or `codeberg.org/acme/widgets`. The forge host is a segment, so a right never crosses forges. A grant's resource **MUST** be contained by the resource of the right that authorises it, and **MUST** be the level its right applies to (namespace for `git.ns.admin` and `git.repo.create`, repository for `git.repo.own` and `git.repo.maintain`). Otherwise: `git-ns:scopeViolation`.
2. **No escalation.** Nobody grants a right their own rights do not carry grant authority for (the table above). Otherwise: `git-ns:escalation`. An actor holding no right on or above the resource at all is refused with the framework's `permissionDenied`.
3. **Last-owner invariant.** A repository that is not `unmanaged` always has at least one owner by explicit record. Revoking or transferring away the last one is refused with `git-ns:lastOwner`. When the last owner leaves the community, ownership passes to the namespace admins by implication and the repository becomes `orphaned` until one of them names a new owner.
4. **Last-admin invariant.** A bound namespace always has at least one `git.ns.admin` by explicit record. Revoking the last one is refused with `git-ns:lastAdmin`.
5. **Members-only floor.** `git.ns.admin` and `git.repo.create` go only to current members of the community, because a non-member cannot be reached by the membership lifecycle that revokes rights on departure. Otherwise: `git-ns:membersOnly`. Whether repository rights may go to non-members — an open-source contributor with a DID, signing commits — is the community's policy, not this rule.
6. **Policy may only narrow.** After the rules above pass, the VTC evaluates the community's git-namespace policy (which forges and visibilities are allowed, who may receive `git.repo.create`, whether and for how long non-members may hold `git.commit.sign`, and so on). Policy can refuse, with `git-ns:policyDenied`; it can never admit a request the rules above refuse.

### Resources

Resources are forge-qualified and lowercase — `<forge-host>/<owner>` or `<forge-host>/<owner>/<repo>` — and are never accepted without their forge. A producer lowercases before sending, because forges compare owner and repository names case-insensitively and the registry compares by exact string. Rights on a repository are keyed internally by the forge's repository id, so that when the forge renames a repository the VTC moves its rights to the new resource, and a new repository later created at the old name inherits nothing.

## Definitions

**`subject`** — who receives the right. For `git.commit.sign` it is the DID whose commit signatures the CI check accepts.

**`right`**, **`resource`** — as in the rights model.

**`expiresAt`** — when the right lapses. The VTC withdraws a lapsed right from its registry projection and records the lapse, just as it would a revocation.

**`reason`** — the granter's free text, kept with the record.

## Request

The actor sends the grant to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Applies the [SPEC §7.2](/SPEC.md#72-consumer-requirements) pipeline and resolves the `issuer` to its rights records.
2. Refuses a resource inside no bound namespace with `git-ns:unknownNamespace`, and one inside a `pending` namespace with `git-ns:namespaceNotBound`.
3. For a repository resource, refuses one it does not record with `git-ns:unknownRepo`, and one that is `archived`, `detached` or `unmanaged` with `git-ns:repoNotActive`. A `pendingCreate` repository **MAY** receive grants; they are published when it becomes `active`.
4. Applies the fixed rules in order, then policy.
5. Refuses an `expiresAt` that is not in the future with `git-ns/right/grant:expiryInPast`.
6. Where the subject already holds a live record of the same `right` on the same `resource`, **MUST** return that record unchanged and ignore `expiresAt` and `reason`. Repeating a grant is therefore safe ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 11). To change an expiry, revoke and grant again.
7. Otherwise records the right with `grantedBy` set to the `issuer`, and publishes it. The response **MUST NOT** be sent before the record is durable. Publication and forge projection are asynchronous; the record is what the response confirms.

During a migration from unqualified `owner/repo` resources, a VTC **MAY** also publish each `git.commit.sign` tuple in the unqualified form. It never accepts an unqualified resource on the wire.

### A namespace admin lets Bob create repositories

```json
{
  "id": "urn:uuid:6f992ea5-95bf-40b9-a6be-b7dec24cbf01",
  "type": "https://trusttasks.org/spec/git-ns/right/grant/0.1",
  "threadId": "urn:uuid:6f992ea5-95bf-40b9-a6be-b7dec24cbf01",
  "issuer": "did:webvh:QmAliceScid1:acme-vtc.example:alice",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-23T10:00:00Z",
  "payload": {
    "subject": "did:webvh:QmBobScid2:acme-vtc.example:bob",
    "right": "git.repo.create",
    "resource": "github.com/acme",
    "reason": "Starting the gadgets line"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmAliceScid1:acme-vtc.example:alice#key-1",
    "created": "2026-09-23T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z1AUm77BtBhgbq5F32U342uC65uF2CTarSEpsY6TycKX63465sLaxoS1qT855iZWG2cAtJnL5yi4oAyY8FZYi7s"
  }
}
```

### Bob, owner of `github.com/acme/gadgets`, lets an outside contributor sign commits for ninety days

Whether Dan, who is not a member, may hold `git.commit.sign` at all is the community's policy; the fixed rules only require that Bob owns the repository.

```json
{
  "id": "urn:uuid:6f992ea5-95bf-40b9-a6be-b7dec24cbf03",
  "type": "https://trusttasks.org/spec/git-ns/right/grant/0.1",
  "threadId": "urn:uuid:6f992ea5-95bf-40b9-a6be-b7dec24cbf03",
  "issuer": "did:webvh:QmBobScid2:acme-vtc.example:bob",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-23T11:00:00Z",
  "payload": {
    "subject": "did:webvh:QmDanScid4:dan.example",
    "right": "git.commit.sign",
    "resource": "github.com/acme/gadgets",
    "expiresAt": "2026-12-22T00:00:00Z",
    "reason": "External contributor for the 1.0 push"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmBobScid2:acme-vtc.example:bob#key-1",
    "created": "2026-09-23T11:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "zruBgAAFQz8BXbHiPTQCnaFN3AaPrStzvD6xULYjck5grZhMF4r27L3r6M1qS1pp7rMWZCELSxTeBJ4pqTR9KMx"
  }
}
```

## Response

The VTC, now responding, returns the record, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error` with `permissionDenied` or one of this specification's codes.

### The recorded right

```json
{
  "id": "urn:uuid:6f992ea5-95bf-40b9-a6be-b7dec24cbf02",
  "type": "https://trusttasks.org/spec/git-ns/right/grant/0.1#response",
  "threadId": "urn:uuid:6f992ea5-95bf-40b9-a6be-b7dec24cbf01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmAliceScid1:acme-vtc.example:alice",
  "issuedAt": "2026-09-23T10:00:01Z",
  "payload": {
    "right": {
      "subject": "did:webvh:QmBobScid2:acme-vtc.example:bob",
      "right": "git.repo.create",
      "resource": "github.com/acme",
      "grantedBy": "did:webvh:QmAliceScid1:acme-vtc.example:alice",
      "grantedAt": "2026-09-23T10:00:01Z",
      "reason": "Starting the gadgets line"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-23T10:00:01Z",
    "proofPurpose": "authentication",
    "proofValue": "zVby74ybhcW78BE5wAyg7Y4pQN5GGcDCZp9FC8eXPZ8wCJcFwLHi6Fc6T9P8Ma7EKVQQ7d5DWinWJJB3pyB4Nmc"
  }
}
```

### Dan's commit right

```json
{
  "id": "urn:uuid:6f992ea5-95bf-40b9-a6be-b7dec24cbf04",
  "type": "https://trusttasks.org/spec/git-ns/right/grant/0.1#response",
  "threadId": "urn:uuid:6f992ea5-95bf-40b9-a6be-b7dec24cbf03",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmBobScid2:acme-vtc.example:bob",
  "issuedAt": "2026-09-23T11:00:01Z",
  "payload": {
    "right": {
      "subject": "did:webvh:QmDanScid4:dan.example",
      "right": "git.commit.sign",
      "resource": "github.com/acme/gadgets",
      "grantedBy": "did:webvh:QmBobScid2:acme-vtc.example:bob",
      "grantedAt": "2026-09-23T11:00:01Z",
      "expiresAt": "2026-12-22T00:00:00Z",
      "reason": "External contributor for the 1.0 push"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-23T11:00:01Z",
    "proofPurpose": "authentication",
    "proofValue": "z6KcSaifkLYVnwMJYGU6GdqtGuM9XY1ofD382spt35DWmSnAuVKZ7iXiNj1VTJsMPkjJAuQxikkvFxZ2x7U69nk"
  }
}
```

## Security & Privacy

### Data carried

The request names one DID, one right and one resource, with an optional expiry and reason. The response returns the record. `reason` is free text: a granter **MUST NOT** put in it anything they would not show every owner and namespace admin of the resource, and a VTC **MUST NOT** publish it to the registry.

### Correlation

The VTC declares `identifierScope: public`. Its DID is the `authority` of every published right and is what each repository's CI check names as its trust anchor; a pairwise identifier would make the rights unverifiable to exactly the verifiers they are for. The actor declares `pairwise`: only this VTC needs to recognise it.

The right itself is public by design once published: the registry answers anonymous queries, so anyone can learn that `subject` holds `right` on `resource`. That is what makes the CI check verifiable, and a granter should treat every grant as a public statement linking the subject's DID to the resource. `grantedBy` and `reason` stay inside the VTC.

### Retention

Durable. The VTC keeps every grant, and its revocation or lapse, as an audit record for as long as it governs the namespace: whether a commit was trusted when it was made is answered from that history, as is a review of who granted what after a member leaves.

### Consent/purpose

The purpose is to decide who may act on a repository and whose commits its CI check accepts. The VTC **MUST NOT** use grants, or the linking of DIDs to resources, for anything else. Whether the subject must agree to receive a right, and whether any grant warrants a step-up for the actor, are the VTC's policy; per [SPEC §7.3](/SPEC.md#73-specification-requirements) item 13, this specification does not decide them.
