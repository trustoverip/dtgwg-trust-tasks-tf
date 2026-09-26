---
slug: git-ns/namespace/reseat
version: "0.1"
title: "Git Namespaces — Reseat Namespace"
summary: "Recovery for a headless bound namespace, one left with no live git.ns.admin: a community administrator grants git.ns.admin on it to a current member. Refused unless the namespace is headless, and recorded with the administrator's statement of why."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - namespace
  - recovery
  - break-glass
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTC
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "A reseat hands the root of a namespace to a person no namespace admin chose, on the authority of a community administrator alone. It must be attributable to that administrator on every transport, and it is the record an audit of the namespace reads back."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A reseat replayed after the namespace was headless again would name an admin nobody chose this time. Placing the request in time is what lets the VTC refuse the replay."
sideEffects:
  level: destructive
  rationale: "Grants git.ns.admin, the namespace root: its holder can grant any right in the namespace, replace every repository's owners and unbind the namespace. Revoking the right later does not undo what was done with it, and the reseat itself is the one grant of that right no namespace admin chose."
consequences:
  - "The subject receives git.ns.admin on the namespace — and with it, by implication, ownership of every repository in it — and the right is published to the community's Trust Registry."
  - "The namespace's repository owners are told the namespace was reseated, by whom, to whom and why."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries a namespace identifier, a member's DID and the administrator's free-text statement. The response returns the recorded right."
retention:
  class: durable
  rationale: "The reseat is the root of every right later granted in the namespace, and the audit record is what shows that a community administrator, not a namespace admin, put it there."
errorCodes:
  - code: git-ns:unknownNamespace
    meaning: "No namespace bound to this VTC has this identifier, or contains this resource."
    retryable: false
  - code: git-ns:namespaceNotBound
    meaning: "The namespace is still `pending`. Nothing is created, adopted or granted in it until binding completes."
    retryable: false
  - code: git-ns:membersOnly
    meaning: "`git.ns.admin` and `git.repo.create` go only to current members of the community, and the subject is not one."
    retryable: false
  - code: git-ns:policyDenied
    meaning: "The community's git-namespace policy refused the request after the fixed rules passed."
    retryable: false
  - code: git-ns/namespace/reseat:notHeadless
    meaning: "The namespace has a live `git.ns.admin`, so it is not headless. Its admins grant `git.ns.admin` with git-ns/right/grant; a community administrator does not."
    retryable: false
related:
  - git-ns/namespace/bind
  - git-ns/namespace/unbind
  - git-ns/right/grant
  - git-ns/right/revoke
  - git-ns/view
---

## Abstract

Every right in a namespace traces back to a `git.ns.admin`. The binding administrator receives the first one from [`git-ns/namespace/bind`](../../../../git-ns/namespace/bind/0.1/spec.md), and [the last-admin invariant](../../../../git-ns/right/grant/0.1/spec.md#the-fixed-rules) stops the last one from being revoked. The invariant cannot stop every way a namespace loses its admins, though: the last one can leave the community — whose membership lifecycle revokes their rights — and a record granted with an expiry lapses on its own. A namespace with no live admin is **headless**. Its repositories keep working and its rights stay published, but nobody can adopt a repository, grant `git.repo.create`, name an owner for an orphaned repository, or unbind it short of a community administrator doing so.

This task is the recovery. A community administrator grants `git.ns.admin` on a headless namespace to a current member. It is refused on a namespace that still has an admin, so it can never be used to go around one.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is the **community-administrator capability** in the VTC's own access control — the one [`git-ns/namespace/bind`](../../../../git-ns/namespace/bind/0.1/spec.md) requires — **together with the namespace being headless**. No git right suffices, because a headless namespace has nobody holding the right that would. The second condition is not a detail of processing but half of the entitlement: [`git-ns/right/grant`](../../../../git-ns/right/grant/0.1/spec.md) is explicit that the community-administrator capability grants nothing in a namespace, and this task is the single exception to that, which exists only while nobody holds the authority the grant rules would otherwise require. On a namespace with a live admin the capability is worth exactly what it is worth in `git-ns/right/grant`: nothing.

The `proof` establishes which administrator asked, not that they may; that is checked against the VTC's access control and the namespace's state ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

**Live `git.ns.admin` record** — a `git.ns.admin` record on the namespace's own resource (`<forge>/<owner>`) that, at the instant the VTC evaluates the request, has not been revoked, carries no `expiresAt` or an `expiresAt` later than that instant, and whose subject is a current member of the community. A record past its `expiresAt` is not live even if the VTC has not yet swept it. The membership clause matters only in the window between a member's departure and the revocation of their rights that departure causes; a departed member's record never keeps a namespace from being headless.

**Headless** — a bound namespace is headless exactly when it has **no live `git.ns.admin` record**. Implied rights do not count, since nothing implies `git.ns.admin`; nor does any right on a repository inside the namespace, nor a community-administrator capability.

**`subject`** — the member who receives `git.ns.admin`.

**`statement`** — the administrator's account of why the namespace is headless and why this member. It is free text, read by the namespace's repository owners, by the subject, and by anyone later auditing the namespace, and it is retained with the audit record and as the recorded right's `reason`.

### The last-admin invariant and expiring records

[Fixed rule 4](../../../../git-ns/right/grant/0.1/spec.md#the-fixed-rules) of `git-ns/right/grant` requires a bound namespace to keep at least one `git.ns.admin` by explicit record, and [`git-ns/right/revoke`](../../../../git-ns/right/revoke/0.1/spec.md) refuses to remove the last one with `git-ns:lastAdmin`. Neither says whether a record with an `expiresAt` counts. If it did, an admin could resign in favour of a co-admin whose right expires next week, and the invariant would have been honoured while guaranteeing a headless namespace. A VTC that implements this specification therefore **MUST** apply the invariant as follows:

- **A `git.ns.admin` record that carries an `expiresAt` does not count toward the last-admin invariant.** Revoking, or resigning, the last live `git.ns.admin` record **without** an `expiresAt` is refused with `git-ns:lastAdmin` even while records with an expiry remain.
- A record with an expiry still makes the namespace **not headless** while it is live. The two rules measure different things: the invariant decides what a revocation may leave behind, and headlessness what is left right now.

This is stated here rather than in a new version of `git-ns/right/grant` because it is not strictly necessary to version the rights model for it: the invariant already binds every task that could remove the last admin, `0.1` is silent on expiring records rather than contrary to this reading, and no document valid under `git-ns/right/grant` 0.1 or `git-ns/right/revoke` 0.1 changes meaning. The only observable difference is that a revocation the silent reading would have allowed is refused with a code both already declare. The next versions of `git-ns/right/grant` and `git-ns/right/revoke` are expected to fold this into fixed rule 4 and revocation step 3 directly.

The invariant cannot keep a namespace from becoming headless. A departing last admin loses their rights with their membership, and a namespace can still hold only expiring admin records — through departures, or through records granted before this rule applied — which lapse. Those are the cases this task recovers.

## Request

The administrator sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Refuses a sender without the community-administrator capability with `permissionDenied`.
2. Refuses an identifier it does not know with `git-ns:unknownNamespace`, and a `pending` namespace with `git-ns:namespaceNotBound`: a pending namespace has not yet had an admin to lose, and the binding in progress will record one.
3. Refuses a namespace that is not headless with `git-ns/namespace/reseat:notHeadless`. The error **MUST NOT** say who the live admins are: every `git.ns.admin` is published to the Trust Registry, where an administrator who needs to know can read them like anyone else.
4. Refuses a `subject` who is not a current member with `git-ns:membersOnly` — the members-only floor of [fixed rule 5](../../../../git-ns/right/grant/0.1/spec.md#the-fixed-rules), which applies to this grant as to any other `git.ns.admin`.
5. Evaluates the community's git-namespace policy, which may refuse with `git-ns:policyDenied` (for example, a policy that the subject must be the member with the most recent ownership in the namespace, or that a second administrator must confirm) and may not override the rules above.
6. Records `git.ns.admin` on the namespace's resource for `subject`, with `grantedBy` set to the administrator, `reason` set to `statement`, and **no `expiresAt`**, so that the recovered namespace meets the last-admin invariant as stated above from the moment it has an admin again. Steps 3 and 6 **MUST** be atomic with respect to every other change to the namespace's `git.ns.admin` records: of two concurrent reseats, or a reseat racing a lapse sweep, exactly one outcome is recorded and the other request sees the result.
7. Records an audit event carrying the administrator, the subject, the `statement`, and the evidence that the namespace was headless: each `git.ns.admin` record it last held and how that record ended — revoked, and by whom, or lapsed, and when. The response **MUST NOT** be sent before the right and the audit event are durable.
8. Publishes the right as a grant is published, and in bridge mode queues the namespace-level forge projection as for any `git.ns.admin` grant. A VTC **SHOULD** tell every owner of a repository in the namespace, and the subject, that the namespace was reseated, by whom, to whom and why.

The reseat restores the VTC's own governance of the namespace. What the forge allows is a separate matter: where the bridge's credentials no longer reach the owner on the forge (`installationRemoved`), the new admin's forge-side role waits for a forge owner to restore them, and every repository's sync already shows `unchecked`.

A repeated request after a reseat succeeded finds the namespace no longer headless and is refused with `git-ns/namespace/reseat:notHeadless`; an exact replay of the same document is also caught by [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 11. An administrator who lost the response and sees `notHeadless` **SHOULD** confirm that the subject now holds `git.ns.admin` — from the Trust Registry, or with [`git-ns/view`](../../../../git-ns/view/0.2/spec.md) when they reseated to themselves — rather than send again.

Once reseated, `orphaned` repositories stay `orphaned` until the new admin names an owner: the reseat restores someone who can, and does not decide for them.

### Reseating the `acme` namespace after its last admin left

Alice, the namespace's only admin, has left the community, and her rights were revoked with her membership. Dana, a community administrator, reseats it to Carol, who owns most of its repositories.

```json
{
  "id": "urn:uuid:7e2b4d18-0c3f-4a95-b6e1-9f8a2c5d3b01",
  "type": "https://trusttasks.org/spec/git-ns/namespace/reseat/0.1",
  "threadId": "urn:uuid:7e2b4d18-0c3f-4a95-b6e1-9f8a2c5d3b01",
  "issuer": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-11-02T10:00:00Z",
  "payload": {
    "namespace": "ns_01J8Z6Q4M2",
    "subject": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
    "statement": "Alice, the only namespace admin, left the community on 2026-10-30. Carol owns 9 of the 12 repositories and agreed to take the namespace on; confirmed at the 2026-11-01 steering call."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmDanaScid8:acme-vtc.example:dana#key-1",
    "created": "2026-11-02T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z5Lm9Qx2Rb7Tk4Wd1Hc8Np3Fz6Ya2Js9Vg5Ue7Ti3Ob1Qn8Mk4Cr6Pw2El9Gh5Sy1Ad7Bv3Xf8Lj4Tq2Rz6Kn9M"
  }
}
```

## Response

The VTC, now responding, returns the recorded right, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error` with `permissionDenied` or one of this specification's codes.

### Carol is the namespace's admin

```json
{
  "id": "urn:uuid:7e2b4d18-0c3f-4a95-b6e1-9f8a2c5d3b02",
  "type": "https://trusttasks.org/spec/git-ns/namespace/reseat/0.1#response",
  "threadId": "urn:uuid:7e2b4d18-0c3f-4a95-b6e1-9f8a2c5d3b01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "issuedAt": "2026-11-02T10:00:01Z",
  "payload": {
    "right": {
      "subject": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
      "right": "git.ns.admin",
      "resource": "github.com/acme",
      "grantedBy": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
      "grantedAt": "2026-11-02T10:00:01Z",
      "reason": "Alice, the only namespace admin, left the community on 2026-10-30. Carol owns 9 of the 12 repositories and agreed to take the namespace on; confirmed at the 2026-11-01 steering call."
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-11-02T10:00:01Z",
    "proofPurpose": "authentication",
    "proofValue": "z3Pq7Wn1Kb5Tx8Lm2Rc6Hd9Fz4Ya7Js2Vg8Ue1Ti5Ob3Qn6Mk9Cr2Pw7El4Gh1Sy8Ad5Bv2Xf6Lj9Tq3Rz1Kn4M"
  }
}
```

## Security & Privacy

### Data carried

A namespace identifier, a member's DID and a free-text statement in; the recorded right, statement included as its `reason`, out. The statement is read by the namespace's repository owners, the subject and later auditors, and is kept for as long as the audit record: an administrator **MUST NOT** put in it anything about the departed admin, or anyone else, beyond what explains the reseat, and a VTC **MUST NOT** publish it to the registry.

### Correlation

The VTC declares `identifierScope: public`, as the registry authority under which the new right is published; the administrator `pairwise`. Once published, the new right publicly links the subject's DID to the namespace, as any `git.ns.admin` grant does. That a reseat happened, rather than an ordinary grant, is visible only inside the VTC: the registry shows the right, not how it was granted.

### Retention

Durable. The reseat and its audit event — who reseated the namespace, to whom, why, and the evidence that it was headless — are kept for as long as the namespace is bound, and in the audit history after. They are what distinguishes a legitimate recovery from a community administrator taking a namespace, and they must outlive the admin they created.

### Consent/purpose

The purpose is to restore governance to a namespace that has lost it, and only that. A VTC **MUST NOT** treat the community-administrator capability as authority in a namespace for any other purpose on the strength of this task. Whether a reseat warrants a step-up, a second administrator's confirmation, or the subject's agreement first is the VTC's policy; per [SPEC §7.3](/SPEC.md#73-specification-requirements) item 13, this specification does not decide it.
