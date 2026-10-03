---
slug: vtc/admin/actions/show
version: "0.2"
title: "VTC Admin Actions — Show"
summary: "Fetch one parked administrative action of a Verifiable Trust Community by its identifier — what a requester follows after a gated operation is parked, and what an approver opens before deciding."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords: [vtc, admin, actions, approval, pending, show]
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: Community maintainer (VTC)
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Whether the action exists for the caller, which role they hold on it, and whether they are handed a decision
    `challenge` all depend on who is asking. The community takes that from a proof over the request rather than from the
    session that carried it, so one administrator's session cannot fetch another's challenge.
sideEffects:
  level: none
  rationale: "Reads one record from the community's action store; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: >-
    Discloses one action's administrative metadata — requester, approvers so far, the parked payload and its summary —
    and, to an approver who may decide it now, a single-use challenge. No key material; the challenge authorizes nothing
    without the approver's own signature over it.
retention:
  class: transient
  rationale: >-
    A view of a record the community keeps; the caller needs it only to render or decide, and refetches because the
    action changes state as other administrators decide.
errorCodes:
  - code: vtc/admin/actions/show:notFound
    meaning: >-
      No action with this `actionId` exists that the caller may see. Returned identically for an action that does not exist
      and for one the caller is not entitled to see, so the answer is not an existence oracle.
    retryable: false
  - code: vtc/admin/actions/show:notAdministrator
    meaning: The proven caller holds no administrative role at this community.
    retryable: false
related:
  - vtc/admin/actions/list
  - vtc/admin/actions/cancel
  - vtc/admin/actions/acknowledge
  - vtc/members/authority-reduction-pending-notice
  - task-consent/decision
  - trust-task-next-step
---

## Abstract

**VTC Admin Actions — Show** fetches one parked administrative action of a Verifiable Trust Community by `actionId`, as the caller is entitled to see it. It is the single-record half of the split pair with [`vtc/admin/actions/list/0.2`](../../list/0.2/spec.md), which explains the model and why the pair exists; the record is the shared [`Action`](../../_shared/0.2/action.schema.json).

### Changes from 0.1

This version differs from [`vtc/admin/actions/show/0.1`](../../show/0.1/spec.md) only in the record it carries: the shared `Action` moved to [`_shared/0.2`](../../_shared/0.2/action.schema.json), which adds the `coolingOff` category with its `landsAt` and `cancellableBy` members, the `landedAfterCoolingOff` closed reason, the `subject` caller role, and the [`vtc/operator/offline-write/0.1`](../../../../operator/offline-write/0.1/spec.md) record type an `acknowledge` action may name. [`vtc/admin/actions/list/0.2`](../../list/0.2/spec.md) describes the changes in full. The request payload and the error codes are unchanged.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### How a requester arrives here

When an administrator submits an operation the community parks rather than executes, the community answers that operation with the framework-reserved [`trust-task-next-step/0.1`](../../../../../trust-task-next-step/0.1/spec.md), carrying `continuation: proceed` and one `expects` entry whose `typeUri` is `https://trusttasks.org/spec/vtc/admin/actions/show/0.2` and whose `hint` is `{"actionId": "…"}`. The requester follows it by issuing this task on the same `threadId`. `proceed` is the point: the requester does **not** re-submit the original operation, because the community executes the parked payload itself — exactly once — when its threshold is met. Re-submitting would park a second action.

A conforming **consumer** (the community) **MUST**:

1. Take the caller from the verified `proof`; a caller holding no administrative role is `notAdministrator`.
2. Answer `notFound` both for an `actionId` it does not hold **and** for one the caller may not see. The two **MUST** be indistinguishable in code, `details`, message and, as far as practicable, timing. Distinguishing them would let any administrator probe which operations other administrators have pending.
3. Compute `callerRole` and `challenge` for the proven caller, including `challenge` only when the caller may decide the action now.

A conforming **producer** **MUST** verify `payloadDigest` against `payload` and re-derive every summary field from `payload` by its pointer before rendering, and **MUST NOT** render an action that fails either check (VTI-APV-013).

## Authorization

The authority is **an administrative role at the community held by the proven caller, together with a relation to this action**: being its requester, an eligible approver or acknowledger, the subject a `coolingOff` operation acts on (`callerRole: subject` — so the administrator about to lose authority can follow the action an [`authority-reduction-pending-notice`](../../../../members/authority-reduction-pending-notice/0.1/spec.md) named), or holding the audit-read capability (which reaches closed actions as `observer`). Both are read from the community's own records at execution time. The `proof` identifies the caller ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); it is the lookup key for the entitlement, not the entitlement. A caller who has neither relation is answered as if the action did not exist.

## Definitions

- **`actionId`** (REQUIRED) — the opaque identifier of the action, as minted by the community; see [`ActionId`](../../_shared/0.2/action.schema.json).
- **`action`** (response) — the [`Action`](../../_shared/0.2/action.schema.json) as the caller may see it.

## Request

Alice follows the next step she was given when her grant of administrative authority to Carol was parked. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Following a next-step hint

```json
{
  "id": "urn:uuid:5a2f8c1e-7d3b-4e69-b0a4-1c9e6f2d8b37",
  "type": "https://trusttasks.org/spec/vtc/admin/actions/show/0.2",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-02T08:00:05Z",
  "threadId": "urn:uuid:0e7b4d2a-9c61-4f38-a5e2-8d3f1b6c7a90",
  "payload": {
    "actionId": "act_7Hq2mZp9Lx4vRk8T"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-02T08:00:05Z",
    "verificationMethod": "did:web:alice.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z22CGm1nhwZAy7hs2bLGnaMHWw9arseiDz7LB4dhknf1EyKhoi2W69KsBiKwJZG4AawV2AT9YkWRwJHDVw4BfiJmw"
  }
}
```

## Response

The community answers with the action, per the `$anchor: "response"` sub-schema. Failures use `trust-task-error`.

### The requester's view

Alice is the requester, so her copy carries no `challenge`.

```json
{
  "id": "urn:uuid:b8e1c3f5-2a7d-4c90-9e6b-4d0f2a8c1e73",
  "type": "https://trusttasks.org/spec/vtc/admin/actions/show/0.2#response",
  "issuer": "did:web:community.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-10-02T08:00:06Z",
  "threadId": "urn:uuid:0e7b4d2a-9c61-4f38-a5e2-8d3f1b6c7a90",
  "payload": {
    "action": {
      "actionId": "act_7Hq2mZp9Lx4vRk8T",
      "category": "approval",
      "kind": "acl.grant.authority",
      "typeUri": "https://trusttasks.org/spec/acl/grant/0.1",
      "requester": "did:web:alice.example",
      "status": "open",
      "createdAt": "2026-10-02T08:00:00Z",
      "expiresAt": "2026-10-05T08:00:00Z",
      "threshold": 2,
      "approvals": [],
      "approversRemaining": 2,
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
      "requesterOpenActions": 1
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-02T08:00:06Z",
    "verificationMethod": "did:web:community.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z4y1Be3NjfjfFyWD8ZMHoXzMqfUhbHCDPmvs8XpH7xLmuStV1SnvDPQQuw2Gd1e3yHHvZCknfrdV51Gw45p9faqbo"
  }
}
```

### An action the caller may not see

Answered exactly as an unknown id would be.

```json
{
  "id": "urn:uuid:4c6a9e2b-1f8d-4b37-a0c5-7e3d9b1f6a24",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:web:community.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-10-02T08:00:06Z",
  "threadId": "urn:uuid:0e7b4d2a-9c61-4f38-a5e2-8d3f1b6c7a90",
  "payload": {
    "code": "vtc/admin/actions/show:notFound",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request carries one opaque identifier. The response carries one action, including its parked `payload` verbatim — the operation's own data, naming whoever it concerns — the administrators who have approved it, and, for an approver who may decide it now, a per-approver `challenge`. The payload is carried so a decision is taken on what will execute rather than on prose; a community **SHOULD** therefore park no more in an operation's payload than the operation needs.

### Correlation

`actionId` is stable for the action's life and joins every show, list entry, notification and decision about it; that is its purpose. It is opaque and long enough not to be guessable, which, with the uniform `notFound`, keeps it from being used to discover other administrators' pending operations. `challenge` differs per approver, so one approver's response does not expose another's binding.

### Retention

The response is a view of the community's retained approval record; a caller keeps it only to render or decide. A `challenge` is useless once the action closes and **SHOULD** be discarded with the view.

### Consent/purpose

The record is shown so an administrator can follow an operation they requested, or decide one they were asked to decide, against the exact payload that will run. Its contents are not for any other use — in particular not for profiling how administrators decide. This specification describes the read only; whether any operation is parked is the community's policy.
