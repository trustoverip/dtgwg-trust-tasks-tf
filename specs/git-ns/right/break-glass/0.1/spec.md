---
slug: git-ns/right/break-glass
version: "0.1"
title: "Git Namespaces — Break Glass"
summary: "A member records for themselves an elevated git right they may already grant — the one self-grant separation of duties forbids — with a justification. It takes effect at once, never lapses, and is shown to every other administrator until one ratifies or revokes it."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - break-glass
  - separation-of-duties
  - emergency-access
  - recovery
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
  rationale: "A break-glass widens its actor's own authority with nobody else involved. It must be attributable to that actor on every transport, and the signed request is the record every other administrator, and any later audit, reads back."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A break-glass replayed after it was revoked would restore a right another administrator deliberately took away. Placing the request in time is what lets the VTC refuse the replay."
sideEffects:
  level: destructive
  rationale: "Shifts authority to the actor on their own say-so: git.ns.admin is the namespace root and can replace every owner and unbind the namespace, git.repo.own governs a repository's rights, and git.repo.create makes its holder owner of what they create. Revoking the right later does not undo what was done with it."
consequences:
  - "The actor holds the right at once, and it is published to the community's Trust Registry like any other right."
  - "Every community administrator and every namespace admin of the namespace is told immediately — who, what, where and why — and sees the grant flagged in their consoles until another administrator ratifies or revokes it."
  - "The right does not expire. It lasts until another administrator revokes it, or ratifies it into an ordinary grant."
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: "The request carries a right, a resource and the actor's free-text justification, which is personal data the actor authored. The response returns the recorded right with its flag and justification."
retention:
  class: durable
  rationale: "The break-glass, its justification, the evidence the VTC gathered for it and how it ended are part of the resource's audit history for as long as the VTC governs the namespace."
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
    meaning: "An elevated right goes only to a current member of the community holding standing in the VTC's access-control records, and the actor is not one."
    retryable: false
  - code: git-ns:policyDenied
    meaning: "The community's git-namespace policy refused the request after the fixed rules passed."
    retryable: false
  - code: git-ns/right/break-glass:disabled
    meaning: "The community's git-namespace policy does not allow break-glass at all, or not for this right or resource. The actor must find another administrator to grant the right."
    retryable: false
  - code: git-ns/right/break-glass:notHeadless
    meaning: "The actor's only standing is the community-administrator capability, which reaches `git.ns.admin` only on a headless namespace, and this namespace has a live `git.ns.admin`. Its admins grant the right with git-ns/right/grant."
    retryable: false
related:
  - git-ns/right/grant
  - git-ns/right/ratify
  - git-ns/right/revoke
  - git-ns/right/break-glass-notice
  - git-ns/namespace/reseat
  - git-ns/view
---

## Abstract

[`git-ns/right/grant`](../../grant/0.3/spec.md) enforces **separation of duties** ([fixed rule 7](../../grant/0.3/spec.md#the-fixed-rules)): nobody grants themselves an *elevated* right — `git.ns.admin`, `git.repo.create` or `git.repo.own` — even when their own rights carry the authority to grant it to anyone else. Someone else must. That is the right default, and it fails exactly when it matters most: at night, during an incident, in a namespace whose other admins have left, or in a two-person community where the other person is on a plane.

This task is the escape hatch. The actor records the elevated right for themselves, explicitly and with a stated justification, and the VTC makes the act impossible to miss:

- it **takes effect at once**, so it works when nobody else is there;
- it **never expires on its own**, for the same reason — an expiry would need someone to extend it;
- the record is **flagged** `breakGlass` until **another administrator** either ratifies it, with [`git-ns/right/ratify`](../../ratify/0.1/spec.md), which turns it into an ordinary grant, or revokes it, with [`git-ns/right/revoke`](../../revoke/0.3/spec.md) — which any community administrator may do, git rights or not;
- the VTC records it at its **highest audit severity**, **tells every other administrator immediately** ([`git-ns/right/break-glass-notice`](../../break-glass-notice/0.1/spec.md)), and **shows it** on every surface that shows the record until it is ratified or revoked.

The community's policy may disable break-glass or make it harder. It cannot make it quieter.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

Break-glass lends no authority the actor does not already have. It lifts exactly one rule — separation of duties — for exactly one grant, to the actor. The entitlement is therefore one of:

1. **Grant authority over the right on the resource** — a right the actor holds, explicitly or by implication, on a resource containing the request's `resource`, whose grant authority covers `right` there, per the [grant authority table](../../grant/0.3/spec.md#grant-authority) of `git-ns/right/grant`. The typical case is a namespace admin recording an explicit `git.repo.own` on a repository in their namespace: they hold an owner's authority there by implication, but an explicit record is what the forge projection and the last-owner invariant read.
2. **The community-administrator capability, for `git.ns.admin` on a headless namespace** — the entitlement of [`git-ns/namespace/reseat`](../../../namespace/reseat/0.2/spec.md), which is the one path by which that capability reaches a git right. A community administrator reseating a headless namespace *to themselves* is a self-grant, which `git-ns/right/grant` 0.3 routes here.

Rights the actor holds through an unratified break-glass record count toward the first entitlement like any other live right. A community administrator who reseated a headless namespace to themselves by break-glass needs to be able to act in it, and every further break-glass is as visible as the first.

The `proof` establishes which member is acting; the VTC resolves that DID, after any delegation it honours, to its own records ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The DID it resolves to is both the actor and the subject of the right.

## Definitions

**Elevated right** — `git.ns.admin`, `git.repo.create` or `git.repo.own` (`ElevatedRight` in [`git-ns/_shared/0.4`](../../../_shared/0.4/git-ns.schema.json)).

**Break-glass record** — a right record carrying `breakGlass`. It is **unratified** while `breakGlass.ratifiedBy` is absent, and **pending** while `breakGlass.effectiveAt` is in the future.

**Administrators of a namespace** — every holder of the community-administrator capability in the VTC's access control, and every subject of a live `git.ns.admin` record on the namespace. These are the people this task tells, and the people who may ratify or revoke.

**`justification`** — the actor's statement of why nobody else could grant the right. It is free text, read by every administrator of the namespace and every owner of the resource, and it is retained with the record and the audit history.

## Request

The actor sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Applies the [SPEC §7.2](/SPEC.md#72-consumer-requirements) pipeline and resolves the `issuer` to the actor.
2. Refuses a resource inside no bound namespace with `git-ns:unknownNamespace`, and one inside a `pending` namespace with `git-ns:namespaceNotBound`. For a repository resource, refuses one it does not record with `git-ns:unknownRepo`, and one that is not `active`, `orphaned` or `pendingCreate` with `git-ns:repoNotActive`.
3. Refuses a `resource` at the wrong level for `right` with `git-ns:scopeViolation` ([fixed rule 1](../../grant/0.3/spec.md#the-fixed-rules)).
4. Checks the [authorization](#authorization) above. An actor who has neither entitlement is refused as `git-ns/right/grant` refuses them — `permissionDenied`, `git-ns:scopeViolation` or `git-ns:escalation` — except that an actor holding the community-administrator capability and asking for `git.ns.admin` on a namespace that is not headless is refused with `git-ns/right/break-glass:notHeadless`. As in `git-ns/namespace/reseat`, that refusal **MUST NOT** say who the namespace's admins are.
5. Refuses an actor who is not a current member, holding standing in its access-control records, with `git-ns:membersOnly` ([fixed rule 5](../../grant/0.3/spec.md#the-fixed-rules)), for every elevated right.
6. Where the actor already holds a live record of `right` on `resource` — ordinary or break-glass — **MUST** return it unchanged, and does nothing else: nothing is recorded, audited as a break-glass, or announced, because nothing changed. A break-glass for a right the actor holds only by implication is not a repeat; recording it explicitly is the point.
7. Evaluates the community's git-namespace policy ([Policy](#policy)), which may refuse with `git-ns/right/break-glass:disabled` or `git-ns:policyDenied`, or defer the right's effect.
8. Records the right: `subject` and `grantedBy` the actor, `grantedAt` now, **no `expiresAt`**, no `reason`, and `breakGlass` carrying `by` (the actor), `at` (now), `justification`, and `effectiveAt` where policy deferred it. Steps 4, 6 and 8 **MUST** be atomic with respect to every other change to the resource's rights and, for the second entitlement, to the namespace's `git.ns.admin` records, exactly as `git-ns/namespace/reseat` requires of its own check and write.
9. Records an audit event at the **highest severity the VTC's audit history has**, carrying the actor, the right, the resource, the `justification`, the time, the entitlement relied on (grant authority, and through which right; or the community-administrator capability on a headless namespace, with the evidence that it was headless as `git-ns/namespace/reseat` step 7 describes), the policy version, and the evidence of every authentication step the VTC required for this request — its kind, the credential that answered, and what it was bound to. The response **MUST NOT** be sent before the right and this event are durable.
10. Tells every administrator of the namespace other than the actor, immediately, with a signed [`git-ns/right/break-glass-notice`](../../break-glass-notice/0.1/spec.md) of event `breakGlass`, over the VTC's own messaging channel to each. A notice that cannot be queued **MUST NOT** undo the break-glass, and **MUST** itself be recorded in the audit history. When there is nobody to tell, the VTC records that too: a break-glass nobody was told about is the case an audit most needs to find.
11. Publishes the right as a grant is published — unless it is pending, in which case it publishes nothing until `effectiveAt`, and the right confers nothing until then. The registry projection is exactly that of an ordinary grant: `breakGlass` is **never** published.

A break-glass that another administrator revokes stays revoked: a replay of the same document is refused by [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 11, and a new request is a new break-glass, audited and announced as the first was.

### Carol takes ownership of `widgets` to ship a security fix

Carol is a namespace admin of `acme`. The repository's two owners are both unreachable, a security fix is waiting, and a namespace admin gets no role on the forge. Separation of duties stops her granting herself `git.repo.own`, so she breaks the glass.

```json
{
  "id": "urn:uuid:0b3c9f4e-8a21-4d6b-9e57-3f1a2c4d5e01",
  "type": "https://trusttasks.org/spec/git-ns/right/break-glass/0.1",
  "threadId": "urn:uuid:0b3c9f4e-8a21-4d6b-9e57-3f1a2c4d5e01",
  "issuer": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-25T02:10:00Z",
  "payload": {
    "right": "git.repo.own",
    "resource": "github.com/acme/widgets",
    "justification": "CVE-2026-4411 fix must ship tonight; both owners (Alice, Bob) unreachable since 22:00, paged twice. Will ask Dana to ratify in the morning."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmCarolScid3:acme-vtc.example:carol#key-1",
    "created": "2026-09-25T02:10:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z4Hn8Qw2Tb6Rk9Xd3Lc7Mp1Fz5Ya8Js3Vg6Ue2Ti4Ob9Qn7Mk1Cr5Pw8El3Gh6Sy2Ad4Bv9Xf1Lj7Tq5Rz3Kn2M"
  }
}
```

### Policy

The community's git-namespace policy is evaluated after the steps above that refuse, as for every git-ns task, and may only narrow ([fixed rule 6](../../grant/0.3/spec.md#the-fixed-rules)). For this task it **MAY**:

- **disable** break-glass entirely, or for some rights, resources or actors — refused with `git-ns/right/break-glass:disabled`;
- **defer** its effect by a delay, so that other administrators have a window to revoke it before it confers anything — recorded as `breakGlass.effectiveAt`;
- **require more** of the actor than the VTC otherwise would — a stronger or an additional authentication step, a minimum justification — refused with `git-ns:policyDenied` when the request falls short.

Policy **MUST NOT** suppress, delay, narrow the audience of, or lower the severity of steps 9 and 10, or of the surfaces in [Visibility](#visibility); a VTC **MUST** apply them whatever the policy says. Break-glass is **enabled** when the policy says nothing about it.

## Visibility

Beyond the audit event and the notices, a VTC that implements this task:

- **MUST** return every unratified break-glass record, with its `breakGlass` and `justification`, through [`git-ns/view`](../../../view/0.4/spec.md) to every administrator of its namespace and every owner of its resource, whatever else the caller may see;
- **SHOULD** show every unratified break-glass record, persistently and to every administrator of its namespace, on every administrative surface it offers — a banner in an administrator console, say, until the record is ratified or revoked — together with a list of break-glass records, ratified ones included, and each record's flag wherever the record itself is shown;
- **MUST** record the break-glass in whatever activity history it offers administrators of the namespace.

## Response

The VTC, now responding, returns the recorded right, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error` with `permissionDenied`, `malformedRequest`, or one of this specification's codes. A VTC that asks the actor for an authentication step before it records the right answers with the refusal its own step-up mechanism defines, and the same document is sent again once the step is done.

### Carol holds `git.repo.own`, flagged

```json
{
  "id": "urn:uuid:0b3c9f4e-8a21-4d6b-9e57-3f1a2c4d5e02",
  "type": "https://trusttasks.org/spec/git-ns/right/break-glass/0.1#response",
  "threadId": "urn:uuid:0b3c9f4e-8a21-4d6b-9e57-3f1a2c4d5e01",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
  "issuedAt": "2026-09-25T02:10:31Z",
  "payload": {
    "right": {
      "subject": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
      "right": "git.repo.own",
      "resource": "github.com/acme/widgets",
      "grantedBy": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
      "grantedAt": "2026-09-25T02:10:31Z",
      "breakGlass": {
        "by": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
        "at": "2026-09-25T02:10:31Z",
        "justification": "CVE-2026-4411 fix must ship tonight; both owners (Alice, Bob) unreachable since 22:00, paged twice. Will ask Dana to ratify in the morning."
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-25T02:10:31Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z2Kp6Wm9Rb3Tx7Ln1Qc5Hd8Fz2Ya6Js1Vg7Ue9Ti3Ob5Qn2Mk8Cr4Pw6El1Gh9Sy3Ad7Bv5Xf2Lj8Tq4Rz6Kn1M"
  }
}
```

### Break-glass is disabled in this community

```json
{
  "id": "urn:uuid:0b3c9f4e-8a21-4d6b-9e57-3f1a2c4d5e04",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:0b3c9f4e-8a21-4d6b-9e57-3f1a2c4d5e03",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
  "issuedAt": "2026-09-25T02:10:01Z",
  "payload": {
    "code": "git-ns/right/break-glass:disabled",
    "message": "This community's policy does not allow break-glass of git.repo.own. Ask another owner or namespace admin to grant it.",
    "retryable": false,
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/git-ns/right/break-glass/0.1",
      "id": "urn:uuid:0b3c9f4e-8a21-4d6b-9e57-3f1a2c4d5e03"
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries an elevated right, a resource and the actor's justification. The justification is free text read by every administrator of the namespace and every owner of the resource, carried in every notice, and kept with the audit record: an actor **MUST NOT** put in it anything they would not show all of them. A VTC **MUST NOT** publish it to the Trust Registry, and **MUST NOT** use it for anything but explaining the break-glass.

### Why nothing expires

An emergency grant that lapses is the usual design, and it fails the case this task exists for: when there is nobody else, there is nobody to extend it either, and an actor mid-incident is left to break the glass again — each time as loudly — or to stop. So the right lasts until another administrator acts on it. The safety comes from visibility instead: nothing about a break-glass is quiet, and it stays flagged in front of every other administrator until one of them ratifies it or takes it away.

### Authentication

Whether the VTC asks the actor for an authentication step before recording a break-glass, and which, is its own policy ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13): this specification does not decide it. The act is one a VTC would ordinarily protect with its strongest step: it is the one way a single person widens their own authority. A step a VTC does require is most useful **bound to this one request** — to a digest of the document — and spent by it, rather than read from a session, which would let a process holding the actor's signing key spend the same gesture on acts the actor never saw. Whatever the VTC requires, step 9 records its evidence.

### Abuse

A break-glass is not a way around another administrator: the second entitlement requires the namespace to be headless, and the first requires authority the actor already holds. What it bypasses is only the second person. Its costs are therefore made to fall on the actor: a written justification in front of every peer, a notice to each of them, a flag no policy can hide, and a right any of them can take away without needing the actor's cooperation or the last-owner and last-admin invariants' permission. A community for which that is not enough disables break-glass, and accepts that an elevated right then always needs two people.

### Correlation

The VTC declares `identifierScope: public`, as the Trust Registry authority whose published rights this changes. The actor declares `pairwise`: only this VTC needs to recognise it. Once published, the right is as public as any other: anyone who queries the registry learns that the actor holds it, though not that it was a break-glass.

### Retention

Durable. The break-glass, its justification, the authentication evidence and how the record ended — ratified, by whom and when, or revoked, by whom and when — stay in the resource's audit history for as long as the VTC governs the namespace.

### Consent/purpose

The purpose is to let an administrator act when nobody else can, visibly. The VTC **MUST NOT** use a break-glass, its justification or its notices for anything else.
