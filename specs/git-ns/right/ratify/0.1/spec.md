---
slug: git-ns/right/ratify
version: "0.1"
title: "Git Namespaces — Ratify Break-Glass"
summary: "An administrator other than its subject ratifies an unratified break-glass record, turning a self-granted elevated right into an ordinary grant that is no longer flagged and counts toward the last-owner and last-admin invariants."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - break-glass
  - separation-of-duties
  - ratification
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTC
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "A ratification is the second person's half of a grant separation of duties requires two people for. It must be attributable to that person on every transport, and it is the record an audit of the right reads back."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Placing the request in time lets the VTC refuse a replay; `breakGlassAt` additionally binds it to the one break-glass the ratifier read."
sideEffects:
  level: mutating
  rationale: "Clears a break-glass record's unratified state: the record stops being flagged, and starts counting toward the last-owner and last-admin invariants, so it can no longer be revoked past them. Who holds which right is unchanged. Reversible in effect with git-ns/right/revoke."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request names a record and carries optional free text. The response returns the record with its break-glass history and justification, to an administrator entitled to see them."
retention:
  class: durable
  rationale: "Who ratified a break-glass, and when, is part of the right's audit history."
errorCodes:
  - code: git-ns:escalation
    meaning: "No right the actor holds on this resource carries the authority to grant or revoke this right."
    retryable: false
  - code: git-ns:policyDenied
    meaning: "The community's git-namespace policy refused the request after the fixed rules passed."
    retryable: false
  - code: git-ns/right/ratify:notBreakGlass
    meaning: "No live, unratified break-glass record matches this subject, right and resource: there is no such record, it was never a break-glass, or it is already ratified."
    retryable: false
  - code: git-ns/right/ratify:recordChanged
    meaning: "The matching break-glass record is not the one the ratifier read: its `breakGlass.at` is not `breakGlassAt`. Read the record again, and its justification, before ratifying."
    retryable: false
  - code: git-ns/right/ratify:selfRatification
    meaning: "The actor is the record's subject. A break-glass is ratified by someone else, or not at all."
    retryable: false
related:
  - git-ns/right/break-glass
  - git-ns/right/grant
  - git-ns/right/revoke
  - git-ns/right/break-glass-notice
  - git-ns/view
---

## Abstract

A [break-glass](../../break-glass/0.1/spec.md) record is an elevated right its subject gave themselves, because nobody else could. It is a real right from the moment it is recorded, and it stays flagged — in every console, in [`git-ns/view`](../../../view/0.4/spec.md), in front of every other administrator — until one of them either takes it away with [`git-ns/right/revoke`](../../revoke/0.3/spec.md), or decides it should stand. This task is the second: the ratifier supplies, after the fact, the second person [separation of duties](../../grant/0.3/spec.md#the-fixed-rules) asked for.

Ratification changes nothing about who holds what. It clears the flag, and it makes the record count toward the last-owner and last-admin invariants, which an unratified break-glass record never does.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The ratifier **MUST NOT** be the record's subject, and **MUST** be one of:

- **an authority over the right on the resource** — a holder of a right whose grant authority covers the ratified right on the resource, per the [grant authority table](../../grant/0.3/spec.md#grant-authority) of `git-ns/right/grant`, where that right is **not itself held through an unratified break-glass record**. Ratification is the second person's decision; a second person whose own authority is a break-glass nobody has confirmed is not one;
- **a community administrator** — the community-administrator capability in the VTC's own access control, with or without a git right. It is the only entitlement there is for a break-glass `git.ns.admin` on a namespace that was headless, and it lets a community's administrators confirm any break-glass in any of its namespaces.

The subject is refused with `git-ns/right/ratify:selfRatification`, whatever else they hold. Anyone else is refused with `permissionDenied`, or with `git-ns:escalation` when they hold rights on the resource but none that covers this right. The `proof` establishes who is acting, never that they may; the VTC resolves the signer, after any delegation it honours, before comparing it with the subject.

## Definitions

**`subject`**, **`right`**, **`resource`** — identify the record exactly, as for [`git-ns/right/revoke`](../../revoke/0.3/spec.md).

**`breakGlassAt`** — the record's `breakGlass.at`, as the ratifier read it.

**`statement`** — the ratifier's free text, kept in the audit record and carried in the notice.

## Request

The administrator sends the request to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Refuses a triple that matches no live record carrying `breakGlass` without `ratifiedBy` with `git-ns/right/ratify:notBreakGlass`. A pending record — one whose `breakGlass.effectiveAt` has not arrived — matches: ratifying it does not bring its effect forward.
2. Refuses a record whose `breakGlass.at` is not `breakGlassAt` with `git-ns/right/ratify:recordChanged`.
3. Refuses the record's subject with `git-ns/right/ratify:selfRatification`, then checks the rest of the [authorization](#authorization).
4. Evaluates the community's git-namespace policy, which may refuse with `git-ns:policyDenied` (for example, a policy that only community administrators ratify a break-glass `git.ns.admin`) and may not override the rules above.
5. Sets `breakGlass.ratifiedBy` to the actor and `breakGlass.ratifiedAt` to now, leaving every other member of the record as it was. Steps 1–3 and 5 **MUST** be atomic with respect to every other change to the resource's rights, so that a ratification and a revocation racing each other have exactly one outcome.
6. Records an audit event carrying the ratifier, the record, `breakGlassAt` and `statement`, at the severity the break-glass was recorded at, and tells every administrator of the namespace — as [`git-ns/right/break-glass`](../../break-glass/0.1/spec.md#definitions) defines them — other than the ratifier, with a [`git-ns/right/break-glass-notice`](../../break-glass-notice/0.1/spec.md) of event `ratified`, as the break-glass itself was told. The response **MUST NOT** be sent before the change and the audit event are durable.

The registry projection does not change: the right was already published as an ordinary one.

### Dana ratifies Carol's break-glass on `widgets`

```json
{
  "id": "urn:uuid:0b3c9f4e-8a21-4d6b-9e57-3f1a2c4d5e11",
  "type": "https://trusttasks.org/spec/git-ns/right/ratify/0.1",
  "threadId": "urn:uuid:0b3c9f4e-8a21-4d6b-9e57-3f1a2c4d5e11",
  "issuer": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-25T09:00:00Z",
  "payload": {
    "subject": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
    "right": "git.repo.own",
    "resource": "github.com/acme/widgets",
    "breakGlassAt": "2026-09-25T02:10:31Z",
    "statement": "Confirmed with Alice: the fix was needed and Carol should stay a co-owner."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmDanaScid8:acme-vtc.example:dana#key-1",
    "created": "2026-09-25T09:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z5Tn2Qw8Kb4Rx6Ld9Mc3Hp7Fz1Ya5Js8Vg2Ue4Ti6Ob1Qn9Mk3Cr7Pw5El2Gh8Sy4Ad6Bv1Xf3Lj9Tq2Rz5Kn7M"
  }
}
```

## Response

The VTC, now responding, returns the ratified record, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error` with `permissionDenied` or one of this specification's codes.

### The record, ratified

```json
{
  "id": "urn:uuid:0b3c9f4e-8a21-4d6b-9e57-3f1a2c4d5e12",
  "type": "https://trusttasks.org/spec/git-ns/right/ratify/0.1#response",
  "threadId": "urn:uuid:0b3c9f4e-8a21-4d6b-9e57-3f1a2c4d5e11",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "issuedAt": "2026-09-25T09:00:01Z",
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
        "justification": "CVE-2026-4411 fix must ship tonight; both owners (Alice, Bob) unreachable since 22:00, paged twice. Will ask Dana to ratify in the morning.",
        "ratifiedBy": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
        "ratifiedAt": "2026-09-25T09:00:01Z"
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-09-25T09:00:01Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3Wm7Qx1Rb9Tk5Ld2Hc6Np8Fz3Ya1Js7Vg4Ue6Ti2Ob8Qn5Mk7Cr1Pw3El6Gh2Sy9Ad5Bv8Xf4Lj1Tq6Rz2Kn3M"
  }
}
```

## Security & Privacy

### Data carried

The request names a record and carries an optional statement, free text shown to the same people as the break-glass justification; a ratifier **MUST NOT** put in it anything they would not show them. The response returns the record with its justification to an actor entitled by construction to see it.

### What ratifying means

A ratifier takes on the second half of a two-person decision after the first half has already been acted on. They should read the justification, and what was done with the right since, before ratifying; `breakGlassAt` ensures that what they ratify is the break-glass they read, and not a later one by the same person for the same right. Ratifying does not endorse what the right was used for; it confirms that the subject should keep holding it. An administrator who thinks the subject should not keep it revokes it instead.

### Correlation

The VTC declares `identifierScope: public`, as the registry authority; the ratifier declares `pairwise`. Ratification changes nothing published.

### Retention

Durable, with the record and its break-glass history.

### Consent/purpose

The purpose is to confirm, or to decline to confirm, a self-granted right. The VTC **MUST NOT** use a ratification or its statement for anything else. Whether ratifying warrants a step-up for the ratifier is the VTC's policy ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).
