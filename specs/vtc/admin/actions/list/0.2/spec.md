---
slug: vtc/admin/actions/list
version: "0.2"
title: "VTC Admin Actions — List"
summary: "An administrator enumerates the Verifiable Trust Community's parked administrative actions they may see — waiting for them, requested by them, cooling off before they land, or closed — with the badge counts a console shows."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords: [vtc, admin, actions, approval, approvals, inbox, pending, break-glass, cooling-off, list]
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
    The answer is computed for the caller: which actions they may see, which they may decide, and the per-approver
    `challenge` on each of those. The community therefore needs the caller's identity from a proof over the request,
    not from whatever session carried it — a bearer session proves who opened a channel, and handing a session-holder
    another administrator's decision challenges is how an approval would be collected from the wrong party.
sideEffects:
  level: none
  rationale: >-
    Reads the community's action store and persists nothing. Minting a per-approver `challenge` for an action is part of
    parking the action, not of listing it, so listing twice returns the same challenge.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: >-
    Discloses administrative metadata about the community's pending and past operations: who requested what, who has
    approved, the parked payloads themselves (which name the DIDs being granted authority, the capabilities granted, the
    members being admitted), and for the caller's own pending decisions a single-use challenge. No key material or
    credential is disclosed; a challenge authorizes nothing without the approver's own signature over it.
retention:
  class: transient
  rationale: >-
    The response is a view of state the community keeps anyway; a console holds it only to render the list and the badge,
    and refetches rather than caching, because an action changes state under it as other administrators decide.
errorCodes:
  - code: vtc/admin/actions/list:notAdministrator
    meaning: The proven caller holds no administrative role at this community.
    retryable: false
  - code: vtc/admin/actions/list:invalidCursor
    meaning: The `cursor` is not one this community issued, has expired, or was issued for a different `view` or `since`.
    retryable: false
  - code: vtc/admin/actions/list:invalidFilter
    meaning: "`since` was supplied with a view other than `history`."
    retryable: false
related:
  - vtc/admin/actions/show
  - vtc/admin/actions/cancel
  - vtc/admin/actions/acknowledge
  - task-consent/decision
  - trust-task-next-step
  - vtc/members/authority-reduction-pending-notice
  - vtc/operator/offline-write
---

## Abstract

A Verifiable Trust Community (VTC) does not refuse an administrative operation that needs other administrators' agreement: it **parks** it as an *action* and asks them. **VTC Admin Actions — List** is how an administrator sees those actions — the ones waiting for their decision, the ones they requested, and the closed ones — together with the two counts a console badge shows. The record shape is the shared [`Action`](../../_shared/0.2/action.schema.json).

### Changes from 0.1

This version differs from [`vtc/admin/actions/list/0.1`](../../list/0.1/spec.md) only in the record it returns: the shared `Action` moved from [`_shared/0.1`](../../_shared/0.1/action.schema.json) to [`_shared/0.2`](../../_shared/0.2/action.schema.json), which adds the **cooling-off** action.

- `category` gains `coolingOff`: an operation that needs a third party's consent where no such party exists — the removal of an administrator when nobody but the requester and the subject holds the authority to consent. It has no `threshold`, no approvers and no `expiresAt`. It **lands** at the new `landsAt` member unless its requester cancels it first, and the new `cancellableBy` member says who may (`requester`).
- `closedReason` gains `landedAfterCoolingOff`, for a cooling-off action whose operation executed at `landsAt`.
- `callerRole` gains `subject`, the party a cooling-off operation acts on: shown the action so they learn of it before it lands, unable to decide or cancel it.
- `typeUri` on an `acknowledge` action may name the record type [`vtc/operator/offline-write/0.1`](../../../../operator/offline-write/0.1/spec.md) for an operator's offline write, which has no Trust Task of its own.

In 0.1 a cooling-off action could only be expressed by omitting `threshold` and `expiresAt` against the schema's prose and carrying the landing time in `ext`. The request payload, the error codes and every other member are unchanged.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Take the caller from the verified `proof`, and refuse a caller holding no administrative role with `notAdministrator`.
2. Return an action only to a party entitled to see it (see [Authorization](#authorization)), and compute `callerRole`, `challenge` and `counts` for the proven caller.
3. Include `challenge` on an action **only** when the caller may decide it now: an open `approval` or `queue` action on which the caller is an eligible approver who has not yet decided.
4. Refuse `since` with any view but `history` (`invalidFilter`), and a `cursor` it did not issue for this `view` and `since` (`invalidCursor`).
5. Compute `counts` over the whole collection, independently of `view`, `limit` and `cursor`.
6. Never place a `coolingOff` action in `waitingForMe` — nobody decides it — and never show one a `challenge`. Show it to its requester under `requestedByMe`, and to the subject it acts on under `all` (and, once closed, `history`) with `callerRole: subject`.

A conforming **producer** (a console) **MUST** render each action from its `payload` through the summary fields' pointers, and **MUST NOT** render an action whose `payloadDigest` or field `value`s do not match its `payload` (VTI-APV-013) — see the shared schema's description.

### How a requester arrives here

The gated operation itself is not answered with an error. The community answers it with the framework-reserved [`trust-task-next-step/0.1`](../../../../../trust-task-next-step/0.1/spec.md), carrying `continuation: proceed` and one `expects` entry naming `https://trusttasks.org/spec/vtc/admin/actions/show/0.2` with `hint: {"actionId": "…"}`. `proceed` means the requester does **not** re-submit the operation — the community executes the parked payload itself, exactly once, when the threshold is met — so the requester follows the action with `show` or this list's `requestedByMe` view.

### Why a list and a show

This family ships the registry's default split pair ([CONTRIBUTING-SPECS → Read-one and read-many tasks](/CONTRIBUTING-SPECS.md#read-one-and-read-many-tasks)): this task enumerates, and [`vtc/admin/actions/show/0.2`](../../show/0.2/spec.md) fetches one action by `actionId`. The two answer different questions. The next-step reply hands a requester an `actionId`, and the requester needs a definite answer about *that* action — `notFound` when it is gone or not theirs to see — where a list filtered to one id would return an empty page, a success indistinguishable from "filtered out". A list also has no per-record authorization outcome to report, and `show` does.

## Authorization

The authority is **an administrative role at the community, held by the proven caller** — and, per action, a relation to that action. The community returns an action only to a caller who is its `requester`, one of its eligible approvers or acknowledgers, the `subject` a `coolingOff` operation acts on, or — on the `history` and `all` views — holds the community's **audit-read capability**, under which they see closed actions they had no part in as `observer`. Administrative standing is read at execution time, so an administrator removed while an action is open stops seeing it.

The `proof` establishes *who* is asking; per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements) it is not itself the entitlement. It is what lets the community look the caller up in its own administrative records, which is where the entitlement lives. Being shown a `challenge` authorizes nothing: an approval counts only as a [`task-consent/decision`](../../../../../task-consent/decision/0.2/spec.md) signed by the approver over it.

## Definitions

- **Action** — a parked administrative operation; see the shared [`Action`](../../_shared/0.2/action.schema.json) for every member.
- **`view`** (REQUIRED) — `waitingForMe`, `requestedByMe`, `history` or `all`; see the schema.
- **`since`** (OPTIONAL) — lower bound on `closedAt`, `history` only.
- **`limit`** (OPTIONAL) — 1–100, default 25.
- **`cursor`** (OPTIONAL) — opaque continuation from `nextCursor`.
- **Cooling-off action** — an action of category `coolingOff`: an operation with no eligible third party to consent, which lands at `landsAt` unless its requester cancels it first.

## Request

An administrator's console asks the community for the actions waiting on that administrator. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Actions waiting for me

```json
{
  "id": "urn:uuid:3b1e6f0a-8c2d-4f57-9a41-6d0e2b7c9f13",
  "type": "https://trusttasks.org/spec/vtc/admin/actions/list/0.2",
  "issuer": "did:web:bob.example",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-10-02T09:15:00Z",
  "threadId": "urn:uuid:3b1e6f0a-8c2d-4f57-9a41-6d0e2b7c9f13",
  "payload": {
    "view": "waitingForMe",
    "limit": 25
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-02T09:15:00Z",
    "verificationMethod": "did:web:bob.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z2H5F9D7tdhAyf2qp78wEWv5fD8oXmaocDv9RtuUq9hFHfvfzFcwen9wkmkG8aGKK7BQHiustQ2LJNiiAnMGJ3Ebb"
  }
}
```

## Response

The community answers with one page of [`Action`](../../_shared/0.2/action.schema.json) records and the caller's `counts`, per the `$anchor: "response"` sub-schema. Failures use `trust-task-error`, not a `#response` document.

### One action awaiting Bob's approval

Alice asked to grant Carol administrative authority; the community requires two approvals and Dana has given one. Bob is an eligible approver, so his copy carries his `challenge`.

```json
{
  "id": "urn:uuid:9d4c2a71-0e6b-4b38-8f15-2a7c3e9d6b04",
  "type": "https://trusttasks.org/spec/vtc/admin/actions/list/0.2#response",
  "issuer": "did:web:community.example",
  "recipient": "did:web:bob.example",
  "issuedAt": "2026-10-02T09:15:01Z",
  "threadId": "urn:uuid:3b1e6f0a-8c2d-4f57-9a41-6d0e2b7c9f13",
  "payload": {
    "actions": [
      {
        "actionId": "act_7Hq2mZp9Lx4vRk8T",
        "category": "approval",
        "kind": "acl.grant.authority",
        "typeUri": "https://trusttasks.org/spec/acl/grant/0.1",
        "requester": "did:web:alice.example",
        "status": "open",
        "createdAt": "2026-10-02T08:00:00Z",
        "expiresAt": "2026-10-05T08:00:00Z",
        "threshold": 2,
        "approvals": [
          { "subject": "did:web:dana.example", "at": "2026-10-02T08:40:12Z" }
        ],
        "approversRemaining": 1,
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
        "challenge": "c7f19a3e5b2d48f0a6e1d9c4b8f20a37",
        "callerRole": "approver",
        "requesterOpenActions": 1
      }
    ],
    "counts": { "waitingForMe": 1, "requestedByMe": 0 }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-02T09:15:01Z",
    "verificationMethod": "did:web:community.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z3iqnfdb9PSqMFVQLyTub4mKJVXjPSqfd3BPgiENsqQk42g3s9f11nz31h6m3ZtDyXH4xpZqduE3yS7pWw38ELb2k"
  }
}
```

### Alice's removal of a co-administrator, cooling off

Alice asked to revoke Carol's administrative entry. The community has no administrator other than Alice and Carol to consent, so it parked the revocation for its 24-hour cooling-off period. Alice's `requestedByMe` view shows when it lands and that she alone can stop it.

```json
{
  "id": "urn:uuid:4c8e2a6f-1b3d-4f95-a7c0-5e9d3b1f7a28",
  "type": "https://trusttasks.org/spec/vtc/admin/actions/list/0.2#response",
  "issuer": "did:web:community.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-10-02T09:30:01Z",
  "threadId": "urn:uuid:a1f3c5e7-9b2d-4e6f-8a0c-3d5f7b9e1c24",
  "payload": {
    "actions": [
      {
        "actionId": "act_Cf9xT2kWq7Ln4Rv8",
        "category": "coolingOff",
        "kind": "acl.revoke.authority",
        "typeUri": "https://trusttasks.org/spec/acl/revoke/0.2",
        "requester": "did:web:alice.example",
        "status": "open",
        "createdAt": "2026-10-02T09:00:00Z",
        "landsAt": "2026-10-03T09:00:00Z",
        "cancellableBy": "requester",
        "approvals": [],
        "summary": {
          "title": "Revoke administrative authority",
          "effect": "The named DID loses its administrative entry at this community when the cooling-off period ends, unless the requester cancels first.",
          "fields": {
            "subject": { "pointer": "/subject", "format": "did", "value": "did:web:carol.example" },
            "reason": { "pointer": "/reason", "format": "text", "value": "Signing key reported compromised." }
          },
          "templateDigest": "zQmQz7a6AokrH2qoAXQtrJGyR8JwFs5mSEACGkQDSpNgKaU"
        },
        "payload": {
          "subject": "did:web:carol.example",
          "revocation": { "kind": "entry" },
          "reason": "Signing key reported compromised."
        },
        "payloadDigest": "zQmbWe4dguXEB8tQoDmx26tj1jaeSja3yAurjDEVhB3Hig9",
        "callerRole": "requester",
        "requesterOpenActions": 1
      }
    ],
    "counts": { "waitingForMe": 0, "requestedByMe": 1 }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-02T09:30:01Z",
    "verificationMethod": "did:web:community.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z2jnHoAisaWwsUu6Bzm5N3MqUAydUaTXXUZ2pjBjQ1anoirgp398sQ1DQJiuNvrnfMYTuH2aybYzXCL73PA1hLUZ5"
  }
}
```

### A stale cursor

```json
{
  "id": "urn:uuid:e2a8f4c6-1d3b-4a9e-b7c5-0f6d8e2a4b19",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:web:community.example",
  "recipient": "did:web:bob.example",
  "issuedAt": "2026-10-02T09:20:01Z",
  "threadId": "urn:uuid:7f0c3e9a-5b2d-4c81-a6e4-9d1b7f3a2c58",
  "payload": {
    "code": "vtc/admin/actions/list:invalidCursor",
    "message": "The cursor was issued for a different view.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request carries a view selector and paging, nothing personal. The response is where the weight is: each action carries its parked `payload` verbatim — so an approver can verify what will execute — and that payload names the people an operation concerns (the DID being granted authority, the applicant being admitted) and what it gives them. The summary repeats a subset of the same values. `approvals` names every administrator who has agreed so far, and `requesterOpenActions` says how busy the requester is. A `coolingOff` action is additionally shown to the subject it acts on, so that subject sees the parked operation — including the requester's stated reason — before it lands. A community **SHOULD NOT** put more into a parked operation than the operation needs, because everything it parks is shown to every eligible approver; and `summary.title` and `effect` are template prose a community **MUST NOT** use to carry anything the fields do not.

### Correlation

Every action is tied to named administrators by design — the requester, each approver — and `actionId` is stable for the action's life, so anyone holding two responses can join them on it. That is inherent in a shared approval list. What is per-caller is the `challenge`: two approvers see different challenges for one action, so a leaked response does not reveal another approver's binding. `counts` and `requesterOpenActions` disclose activity levels to every administrator who can see the action; a community with reason to hide one administrator's workload from another can omit `requesterOpenActions`.

### Retention

The response is a view of records the community retains as its approval and audit trail; a console needs it only until it renders and **SHOULD NOT** cache it, since actions change state as other administrators decide. A `challenge` in particular is worthless once its action closes and **SHOULD** be dropped with the view.

### Consent/purpose

The data is shown so administrators can make, follow and audit collective decisions about the community's administration — to approve or decline what they are asked, to follow what they asked for, and to review what was done. It is not for scoring administrators on how quickly or how often they approve, nor for choosing which approver to route a request to; routing for agreeableness defeats a threshold without any signature failing. This specification describes the list; whether any operation is parked at all is the community's policy, not something this specification requires.
