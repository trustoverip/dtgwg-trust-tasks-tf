---
slug: vtc/elections/cancel
version: "0.1"
title: "VTC Elections — Cancel"
summary: "An administrator cancels an election before voting closes: the community stops issuing its tickets at once and has the ballot box cancel it, so no result is ever computed."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - voting
  - election
  - cancellation
  - registrar
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: community (the registrar)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Cancelling an election discards every ballot cast in it. It must be attributable to the administrator who did it after
    the transport has closed, because a cancelled vote is exactly the act members will ask to have accounted for.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A cancellation replayed against a later election
    under a reused identifier would discard that election's ballots.
sideEffects:
  level: destructive
  rationale: >-
    Ends the election without a result. Irreversible: no ticket is issued for it again, the ballot box admits no cast and
    never tallies, and a member's ticket is discarded by their VTA.
consequences:
  - "No result is ever computed for the election; ballots already cast are never counted."
  - "Members' agents discard the election's tickets."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request names an election and a reason; the response is the election's state."
retention:
  class: durable
  rationale: "A cancelled election is a governance record: who cancelled it, when and why outlives the election."
errorCodes:
  - code: vtc/elections/cancel:electionUnknown
    meaning: "No election with this identifier exists for this community, or the caller may not see it (CONVENTIONS §7)."
    retryable: false
  - code: vtc/elections/cancel:notOpen
    meaning: "The election can no longer be cancelled: voting has closed, it is tallied, or it is already cancelled (CONVENTIONS §7)."
    retryable: false
  - code: vtc/elections/cancel:ballotBoxUnavailable
    meaning: "The ballot box did not answer. The community has stopped issuing the election's tickets and keeps them stopped; the same request may be sent again to finish the cancellation."
    retryable: true
  - code: vtc/elections/cancel:ballotBoxRefused
    meaning: "The ballot box refused to cancel the election. Its own code and message are carried in `details`. The community keeps the election's tickets stopped."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        code:
          type: string
          maxLength: 128
        message:
          type: string
          maxLength: 500
related:
  - vtc/elections/close
  - vtc/elections/show
  - vtc/elections/create
  - vtc/elections/tickets/issue
  - voting/election/cancel
---

## Abstract

**VTC Elections — Cancel** abandons an election before voting closes: a question put wrongly, a manifest with a mistake in it, an election overtaken by events. The community stops issuing the election's tickets at once (VTI-VOT-024) and sends [`voting/election/cancel`](../../../../voting/election/cancel/0.1/spec.md) to the ballot box, which admits no further cast and never tallies. The family's shared rules are in [`CONVENTIONS.md`](../../../../voting/_shared/0.1/CONVENTIONS.md).

It is also how an administrator clears an election the ballot box never registered — one still `pendingRegistration`, or `registrationRefused`.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** answer `electionUnknown` for an election it does not hold or the caller may not see, as [`vtc/elections/show`](../../show/0.1/spec.md) does, and **MUST** refuse a requester without `vtc.elections.manage` (VTI-VOT-080), read from the signer's live access-control entry at execution time, with `permissionDenied`.
2. **MUST** refuse with `notOpen` an election that is closed, tallied or already cancelled. A repeat of the same signed document is answered with the answer first given ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 11).
3. **MUST** stop issuing the election's tickets durably, before anything else, and **MUST NOT** issue one again whatever the ballot box answers (VTI-VOT-024). Stopping issuance is the one effect that never waits on the ballot box: it only narrows what the community does.
4. **MUST** then send [`voting/election/cancel`](../../../../voting/election/cancel/0.1/spec.md) to the ballot box, signed as itself, and record the election as `cancelled` only after the ballot box answered with success. When the ballot box does not answer it answers `ballotBoxUnavailable`, and the requester sends the request again to finish; the ballot box treats a repeated cancel as the first. When the ballot box refuses it answers `ballotBoxRefused`.
5. For an election the ballot box has not registered (`pendingRegistration`, `registrationRefused`), **MUST** first stop delivering the registration, then send the cancel as in item 4, and **MUST** treat the ballot box's `electionUnknown` as success: there is nothing at the ballot box to cancel.
6. **MUST** audit the cancellation (VTI-VOT-100): the requester, the election, the time, `reason`, and the ballot box's answer.

A conforming **member's VTA** discards its ticket for a cancelled election (CONVENTIONS §5.3); it learns of the cancellation from [`vtc/elections/show`](../../show/0.1/spec.md) or [`voting/election/show`](../../../../voting/election/show/0.1/spec.md).

## Authorization

The authority is the **`vtc.elections.manage` capability at the community** (VTI-VOT-080), read from the signer's access-control entry at execution time. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The ballot box acts on the community's signed cancel and on nothing about the administrator (CONVENTIONS §4). Whether a cancellation — in particular of a binding election — takes a second administrator's consent or a step-up is the community's policy ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13); this specification describes the model it is designed for.

## Definitions

- **`electionId`** (REQUIRED) — the election to cancel.
- **`reason`** (REQUIRED) — why, 1 to 1024 characters and not blank, for the audit trail and the community's members.
- **`state`** (response) — `cancelled`.

## Request

The administrator names the election and says why. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Cancel an election put wrongly

```json
{
  "id": "urn:uuid:f8091a2b-3c4d-4e5f-9010-8192a3b4c5d6",
  "type": "https://trusttasks.org/spec/vtc/elections/cancel/0.1#request",
  "issuer": "did:web:carol.example",
  "recipient": "did:webvh:QmCommunity:community.example",
  "issuedAt": "2026-11-03T09:00:00Z",
  "threadId": "urn:uuid:f8091a2b-3c4d-4e5f-9010-8192a3b4c5d6",
  "payload": {
    "electionId": "board-2026",
    "reason": "The candidate list omitted a nominee; the election will be created again."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:carol.example#key-1",
    "created": "2026-11-03T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z7cDExampleProofValue"
  }
}
```

## Response

The community answers with the election's state once the ballot box has cancelled it — the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). A refusal is a `trust-task-error`.

### Cancelled

```json
{
  "id": "urn:uuid:091a2b3c-4d5e-4f60-a121-92a3b4c5d6e7",
  "type": "https://trusttasks.org/spec/vtc/elections/cancel/0.1#response",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-11-03T09:00:02Z",
  "threadId": "urn:uuid:f8091a2b-3c4d-4e5f-9010-8192a3b4c5d6",
  "payload": {
    "electionId": "board-2026",
    "state": "cancelled"
  }
}
```

## Security & Privacy

### Data carried

The request is an election identifier and a reason; the response is a state. `reason` is shown to the community's members and kept in the audit trail, so a producer **MUST NOT** name a member in it, and **MUST NOT** describe how the ballots cast so far went — on an open board anyone can read them, but a cancellation's stated reason must not read as a judgement on them.

### Correlation

The community is declared `identifierScope: public`: it is addressed by the DID its manifests name as `community`, which its members, its ballot box and its auditors all know it by, and which every ticket and cast is bound to (CONVENTIONS §3). A pairwise identifier for it would break that binding and tell the parties nothing they could check.

Nothing here concerns a voter. The tickets already issued stay unlinkable to the casts made with them; cancelling does not change what the community knows about who took a ticket.

### Retention

Durable. The community keeps the audit row of the cancellation for as long as it keeps its governance records. It **SHOULD** discard its per-signer issuance records for the election once cancelled, keeping only their count: with no result to certify, the records no longer bound anything (VTI-VOT-072).

### Consent/purpose

The reason is collected to account to the community's members for abandoning a vote, and for nothing else. Whether a cancellation takes a second administrator or a step-up is the community's policy ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).
