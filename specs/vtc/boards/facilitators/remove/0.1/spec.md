---
slug: vtc/boards/facilitators/remove
version: '0.1'
title: "VTC Boards — Facilitators — Remove"
summary: "A facilitator steps down, or a holder of vtc.boards.manage unqualified ends another member's appointment as a board's facilitator."
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords: [vtc, boards, facilitators, access-control, delegation]
parties:
- role: board facilitator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "Withdraws a member's authority over a board. Must be attributable after the transport has closed."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed removal would remove a facilitator since appointed again."
sideEffects:
  level: mutating
  rationale: "Removes a resource grant from an access-control entry. Recoverable with vtc/boards/facilitators/add."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "Names the facilitator by entry and returns the board's appointed facilitators by entry, never by DID."
retention:
  class: durable
  rationale: "The removal is why the member no longer facilitates the board."
errorCodes:
- code: vtc/boards/facilitators/remove:notFound
  meaning: "No board the caller facilitates has this identifier, and the caller may not see it (VTI-BRD-003)."
  retryable: false
- code: vtc/boards/facilitators/remove:notFacilitator
  meaning: "The entry holds no appointment to facilitate this board."
  retryable: false
related:
- vtc/boards/facilitators/add
- vtc/boards/facilitators/list
- vtc/groups/managers/remove
---
## Abstract

The **VTC Boards — Facilitators — Remove** Trust Task ends a member's appointment as a board's facilitator: the resource grant `vtc.boards.manage` qualified to the board is removed from their access-control entry (VTI-BRD-091). A facilitator may step down; only a holder of `vtc.boards.manage` unqualified ends somebody else's appointment.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the Community Boards chapter of the Verifiable Trust Infrastructure specification (VTI-BRD-090 – VTI-BRD-093). The design is recorded in the VTC boards design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-boards.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who does not facilitate the board with `notFound` when they may not see it, and `permissionDenied` when they may.
2. Refuse with `permissionDenied` a caller removing an entry other than their own unless they hold `vtc.boards.manage` unqualified. The facilitators of one board do not end one another's appointments.
3. Refuse an entry that holds no grant of `vtc.boards.manage` qualified to this board with `notFacilitator`. Holding the capability unqualified, or through a role, is not an appointment this task removes.
4. Remove that grant from the entry, leaving every other grant and the entry's role untouched, and audit the change by reference (VTI-BRD-080).

The removal ends the member's authority over the board. Ideas they decided and selected, and notes they hid, stand.

## Authorization

**Authority: `vtc.boards.manage`** — qualified `board:<boardId>` to step down from that board, unqualified to end another member's appointment — read from the signer's access-control entry at execution time. A key a member portal holds for the member is never accepted (VTI-BRD-051). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `BoardId` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `EntryId` — see [`directory-card.schema.json`](../../../../directory/_shared/0.1/directory-card.schema.json).

## Request

The board facilitator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Step down

```json
{
  "id": "urn:uuid:5b8e2f1a-3c4d-4e5f-9a0b-000000000121",
  "type": "https://trusttasks.org/spec/vtc/boards/facilitators/remove/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-05T09:00:00Z",
  "threadId": "urn:uuid:5b8e2f1a-3c4d-4e5f-9a0b-000000000120",
  "payload": {
    "boardId": "brd_W5k2N8xRb3Qz7TpLq4Ve",
    "entryId": "dir_Q7m2K9xVb4Tz8LpR"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:carol#key-1",
    "created": "2026-11-05T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/boards/facilitators/remove/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Step down — answered

```json
{
  "id": "urn:uuid:5b8e2f1a-3c4d-4e5f-9a0b-000000000122",
  "type": "https://trusttasks.org/spec/vtc/boards/facilitators/remove/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-11-05T09:00:01Z",
  "threadId": "urn:uuid:5b8e2f1a-3c4d-4e5f-9a0b-000000000120",
  "payload": {
    "boardId": "brd_W5k2N8xRb3Qz7TpLq4Ve",
    "facilitators": []
  }
}
```

## Security & Privacy

### Data carried

A board and an entry; the board's appointed facilitators by entry.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the community differ by who asks. Facilitators are named by their community-scoped `entryId`, never by DID (VTI-BRD-093).

### Retention

**Durable**: the audit row records why the appointment ended.

### Consent/purpose

Ending a delegation of a board's running. Nothing the facilitator did while appointed is undone.

