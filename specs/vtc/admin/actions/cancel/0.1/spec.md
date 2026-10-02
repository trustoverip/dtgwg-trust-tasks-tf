---
slug: vtc/admin/actions/cancel
version: "0.1"
title: "VTC Admin Actions — Cancel"
summary: "The requester of an open administrative action at a Verifiable Trust Community withdraws it, so the parked operation never executes and the action leaves every administrator's list."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords: [vtc, admin, actions, approval, withdraw, cancel]
parties:
  - role: Requester (administrator)
    requirement: REQUIRED
    member: issuer
  - role: Community maintainer (VTC)
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Only the action's requester may withdraw it, and the community establishes that by comparing the proven signer with
    the action's `requester`. A session cannot stand in: withdrawing another administrator's pending operation — a
    revocation awaiting its second approval, say — is itself an act the audit trail must attribute.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A cancellation names an action, not a moment. Replayed later it would find the action already closed and do
    nothing, but a consumer can only reason about that — and bound its duplicate-execution record (SPEC §7.2 item 11) —
    if the document can be placed in time.
sideEffects:
  level: mutating
  rationale: >-
    Closes an open action as `cancelled`: approvals recorded on it stop counting, outstanding challenges are withdrawn,
    and the parked operation will never execute from this action. Not destructive — the requester can submit the
    operation again, which parks a fresh action with fresh approvals.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: personal
  rationale: >-
    Returns the closed action, with the same administrative metadata `show` discloses but no `challenge`, since nothing
    remains to decide. Inbound, `reason` is free text written by a person about a decision they are withdrawing, read by
    every administrator who can see the action.
retention:
  class: durable
  rationale: >-
    The signed cancellation is the record that an operation other administrators were asked to approve was withdrawn,
    and by whom. Without it the audit trail shows an action that simply stopped, which is indistinguishable from one
    suppressed by a party with no right to.
errorCodes:
  - code: vtc/admin/actions/cancel:notFound
    meaning: No action with this `actionId` exists that the caller may see. Answered identically for both cases.
    retryable: false
  - code: vtc/admin/actions/cancel:notRequester
    meaning: The caller can see the action but is not its requester.
    retryable: false
  - code: vtc/admin/actions/cancel:notOpen
    meaning: The action is no longer `open` — it completed, was declined, expired, failed, or was already cancelled.
    retryable: false
related:
  - vtc/admin/actions/list
  - vtc/admin/actions/show
  - vtc/admin/actions/acknowledge
  - task-consent/decision
---

## Abstract

**VTC Admin Actions — Cancel** lets the administrator who submitted a parked operation withdraw it before it completes. The action closes as `cancelled`, the operation will never execute from it, and it leaves every administrator's list. The record is the shared [`Action`](../../_shared/0.1/action.schema.json); the model is described in [`vtc/admin/actions/list/0.1`](../../list/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Take the caller from the verified `proof`. An action the caller may not see is `notFound`, answered exactly as an unknown `actionId`.
2. Refuse a caller who is not the action's `requester` with `notRequester`.
3. Refuse an action that is not `open` with `notOpen`. A cancellation that races the threshold loses cleanly: closing the action and executing it are one atomic transition, so either the operation executed (and this task is `notOpen`) or the action is cancelled (and it never executes).
4. On success, set `status: cancelled`, `closedReason: cancelledByRequester` and `closedAt`, withdraw every outstanding `challenge`, and refuse any later [`task-consent/decision`](../../../../../task-consent/decision/0.2/spec.md) for it.

## Authorization

The authority is **being the action's requester**: the community compares the proven signer with the `requester` it recorded when it parked the operation. Withdrawing an operation one asked for is the requester's own decision and needs nobody else's. Administrative standing is still read at execution time — a requester who has since lost every administrative role cannot reach the action (`notFound`); the community closes such an action itself as `invalidated`. Approvers cannot cancel; an approver who does not want the operation declines it with a `deny` decision, which closes it for everyone. Verifying the `proof` establishes who signed ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); the entitlement is the match against `requester`.

## Definitions

- **`actionId`** (REQUIRED) — the open action to withdraw.
- **`reason`** (OPTIONAL, ≤500 characters) — a note for the approvers and the audit record.
- **`action`** (response) — the action as it now stands.

## Request

Alice withdraws the grant she asked for. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Withdrawing an open grant

```json
{
  "id": "urn:uuid:1d9f3b7e-4a2c-4e85-b6d1-8f0a3c5e7b29",
  "type": "https://trusttasks.org/spec/vtc/admin/actions/cancel/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-02T10:30:00Z",
  "threadId": "urn:uuid:1d9f3b7e-4a2c-4e85-b6d1-8f0a3c5e7b29",
  "payload": {
    "actionId": "act_7Hq2mZp9Lx4vRk8T",
    "reason": "Wrong role — Carol needs operator, not admin. Resubmitting."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-02T10:30:00Z",
    "verificationMethod": "did:web:alice.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z4bNExampleProofValueForActionsCancel"
  }
}
```

## Response

The community answers with the closed action, per the `$anchor: "response"` sub-schema. Failures use `trust-task-error`.

### The action, cancelled

```json
{
  "id": "urn:uuid:6e2c8a4f-0b3d-4f71-9c5e-2a7d1f9b3e46",
  "type": "https://trusttasks.org/spec/vtc/admin/actions/cancel/0.1#response",
  "issuer": "did:web:community.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-10-02T10:30:01Z",
  "threadId": "urn:uuid:1d9f3b7e-4a2c-4e85-b6d1-8f0a3c5e7b29",
  "payload": {
    "action": {
      "actionId": "act_7Hq2mZp9Lx4vRk8T",
      "category": "approval",
      "kind": "acl.grant.authority",
      "typeUri": "https://trusttasks.org/spec/acl/grant/0.1",
      "requester": "did:web:alice.example",
      "status": "cancelled",
      "createdAt": "2026-10-02T08:00:00Z",
      "expiresAt": "2026-10-05T08:00:00Z",
      "closedAt": "2026-10-02T10:30:01Z",
      "closedReason": "cancelledByRequester",
      "threshold": 2,
      "approvals": [
        { "subject": "did:web:dana.example", "at": "2026-10-02T08:40:12Z" }
      ],
      "summary": {
        "title": "Grant administrative authority",
        "effect": "The named DID becomes an administrator of this community with the role shown.",
        "fields": {
          "grantee": { "pointer": "/entry/subject", "format": "did", "value": "did:web:carol.example" },
          "role": { "pointer": "/entry/role", "format": "text", "value": "admin" }
        },
        "templateDigest": "zQmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco"
      },
      "payload": {
        "entry": { "subject": "did:web:carol.example", "role": "admin", "label": "Carol — operations" }
      },
      "payloadDigest": "zQmb1XVvHqbCe5nUPFxpJcRz3RtP4pQyKgTsWJgNBzVhE7d",
      "callerRole": "requester",
      "requesterOpenActions": 0
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-02T10:30:01Z",
    "verificationMethod": "did:web:community.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z5cWExampleProofValueForActionsCancelResponse"
  }
}
```

### Too late — the threshold was met first

```json
{
  "id": "urn:uuid:8a3e5c1f-7d2b-4c96-b0e4-3f9a1d7c5b82",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:web:community.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-10-02T10:30:01Z",
  "threadId": "urn:uuid:1d9f3b7e-4a2c-4e85-b6d1-8f0a3c5e7b29",
  "payload": {
    "code": "vtc/admin/actions/cancel:notOpen",
    "message": "The action completed before it could be cancelled.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request carries an identifier and an optional `reason`. `reason` is free text every administrator who can see the action will read, and is kept in the audit record; a requester **SHOULD** confine it to why the operation is withdrawn and **MUST NOT** use it for anything they would not show every approver. The response repeats the closed action — no longer with any `challenge`.

### Correlation

The cancellation joins to the action by `actionId` and to the requester by signature, as intended: it is a statement by a named administrator about their own pending operation. Nothing else in it is a stable handle.

### Retention

The signed cancellation is part of the community's administrative audit trail and **SHOULD** be retained with the action it closes, for as long as the community keeps that action's record. The approvals the action had already gathered stay in the record as given; cancellation stops them counting, it does not erase that they were given.

### Consent/purpose

The data is used to withdraw a pending operation and to record that it was withdrawn and why. `reason` is for the approvers and the audit trail, not for any other use. This specification describes the withdrawal; it imposes no approval or step-up requirement on it, which is the community's policy to decide.
