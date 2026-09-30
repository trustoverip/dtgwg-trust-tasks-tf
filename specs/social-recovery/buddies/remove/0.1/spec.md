---
slug: social-recovery/buddies/remove
version: "0.1"
title: "Social Recovery — Remove Buddy"
summary: "A device owner withdraws a previously enrolled recovery buddy's standing custodial authority, a high-risk write since it weakens the identity's own recovery path rather than strengthening it."
status: draft
targetFrameworkVersion: "0.6.0"
category: key-management
keywords:
  - social-recovery
  - guardian
  - key-recovery
  - revocation
parties:
  - role: device owner
    requirement: REQUIRED
    member: issuer
  - role: recovery maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Removing a buddy withdraws standing custodial authority; an
    unattributed removal could not later be shown to have been the
    owner's own decision, which matters as much for a removal as for the
    enrolment `social-recovery/buddies/add` performs.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A replayed removal after the owner has since re-enrolled the same
    buddy would silently reverse a later decision to trust them again.
sideEffects:
  level: mutating
  rationale: >-
    Withdraws one buddy's standing custodial authority. Reversible by
    re-enrolling via `social-recovery/buddies/add`, but not inert in the
    interim: the removed party's authority ends the moment this task
    completes, and — the reference implementation's own risk framing —
    removal weakens the identity's available recovery path rather than
    strengthening it, which is why its `riskLevel` is declared `high`
    rather than the `critical` level enrolment itself carries: the
    immediate risk of a WRONG removal is smaller than the immediate risk
    of a wrongly ADDED custodial party, but a removal that leaves too few
    buddies is still a real degradation of recovery capability.
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: social-recovery/buddies/remove:buddyNotFound
    meaning: >-
      This contact is not currently an enrolled recovery buddy for this
      identity. Not an error of intent — the owner should query
      `social-recovery/buddies/list` first.
    retryable: false
related:
  - social-recovery/buddies/add
  - social-recovery/buddies/list
  - social-recovery/status/get
---

## Abstract

The **Social Recovery — Remove Buddy** Trust Task withdraws one
previously enrolled recovery buddy's standing custodial authority. It is
the reverse of
[`social-recovery/buddies/add`](../../add/0.1/spec.md), and the reference
implementation frames its risk asymmetrically from enrolment: adding a
buddy is `riskLevel: "critical"` (a wrong addition plants an attacker's
own recovery path ahead of a device takeover, per `buddies/add`'s own
exemplar); removing one is `riskLevel: "high"` — still governance-tier and
approval-gated, but the immediate hazard of a wrong removal is smaller
than the immediate hazard of a wrong addition, even though a removal that
leaves too few buddies enrolled is its own real degradation of the
identity's recovery capability.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)).
It targets framework version 0.6.0 and may change without a version bump
while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It documents an operation a maintainer already implements
(`removeRecoveryBuddy`, `src/packages/devices/manifest-operations-part-2.ts`),
written down so the shape stops being recoverable only by reading an
implementation.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD
NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be
interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14)
when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy
[SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the
requirements stated here.

## Definitions

- **`recoveryBuddyDid`** — REQUIRED, the DID of the buddy to remove.
- **`keyId`** — returned in the response: the maintainer's own internal
  identifier for the enrolment key record that was removed (the same value
  `social-recovery/buddies/add`'s response returns at enrolment time).
- **`seq`** — returned in the response: the chain-commit sequence number of
  the `guardian.removed` event this removal produced.

## Request

The device owner names the buddy to remove. The top-level schema is in
[`payload.schema.json`](payload.schema.json).

### Removing an estranged former contact

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000701",
  "type": "https://trusttasks.org/spec/social-recovery/buddies/remove/0.1#request",
  "issuer": "did:example:owner",
  "recipient": "did:example:recovery-maintainer",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000007ff",
  "payload": {
    "recoveryBuddyDid": "did:example:estranged-former-partner"
  }
}
```

## Response

The maintainer confirms the buddy is removed. The
response shape below is normative prose: `payload.schema.json` governs the
REQUEST only and carries no response sub-schema. Failures are `trust-task-error`
documents.

### Buddy removed

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000702",
  "type": "https://trusttasks.org/spec/social-recovery/buddies/remove/0.1#response",
  "issuer": "did:example:recovery-maintainer",
  "recipient": "did:example:owner",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000007ff",
  "payload": {
    "recoveryBuddyDid": "did:example:estranged-former-partner",
    "keyId": "g-key-estranged-former-partner",
    "seq": 12
  }
}
```

## Security & Privacy

### Data carried

The request carries only the buddy's identifier. The response echoes that
identifier together with the maintainer's own internal `keyId` for the
enrolment record that was removed and the `seq` (chain-commit sequence
number) the removal produced — never key material, and never a roster
count: the reference implementation does not compute or return one on this
operation, so a client needing the post-removal count MUST call
[`social-recovery/buddies/list`](../../../buddies/list/0.1/spec.md). A
previous revision of this section named a "remaining roster count" as part
of the response. **That was incorrect** and is corrected here rather than
removed silently, because a reader who saw it may have relied on it.

### Correlation

A removal event is itself a durable, sensitive social-graph fact — that
the owner has withdrawn trust from a specific party. A maintainer
**SHOULD** treat the removal event with the same care
`social-recovery/buddies/add`'s Correlation section states for the
enrolment event itself, and **MUST NOT** notify the removed buddy of the
reason (if any is supplied) beyond confirming their standing has ended.

### Retention

The removal is retained as a state change against the roster
`social-recovery/buddies/list` reads; there is no implicit re-enrolment —
a removed buddy stays removed until a fresh
`social-recovery/buddies/add` call re-establishes them.

### Consent/purpose

The owner's own removal request is the act; per
[SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) this
specification does not itself require a further approval beyond the
reference implementation's own approval gate on this destructive-standing
write (`requiresApproval: true`, `riskLevel: "high"`).

### Custody scope

A recovery maintainer typically serves more than one owner's identity. It
**MUST** scope removal authority to the requesting owner's own roster, and
**MUST** answer a `recoveryBuddyDid` that is enrolled for a different
owner's identity identically to one that is not enrolled at all, the same
enumeration-refusal discipline `social-recovery/buddies/add`'s Custody
scope section states for its own boundary.
