---
slug: vtc/boards/delete
version: '0.1'
title: "VTC Boards — Delete"
summary: "A board manager deletes a closed board whose voting has ended: its ideas, notes and each election's content are removed, and each election keeps only a tombstone — identifier, manifest digest and kept board heads — for the community's tombstone retention."
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords: [vtc, boards, unconference, ideas, elections]
parties:
- role: board manager
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "Removes what members wrote and the community's record of the board's elections; must stay attributable after the transport has closed."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. Placing the document in time bounds the duplicate-execution window."
sideEffects:
  level: destructive
  rationale: "Removes the board, its ideas and notes, and every election's content but a tombstone, which is itself removed when the retention passes (VTI-BRD-007, VTI-BRD-008). Nothing is restored by creating a board again."
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: "Names a board and a reason; returns counts and when the tombstones go."
retention:
  class: durable
  rationale: "The community keeps the record that it deleted the board, and why; never what the board held."
errorCodes:
- code: vtc/boards/delete:notFound
  meaning: "No board the caller may see has this identifier (VTI-BRD-003)."
  retryable: false
- code: vtc/boards/delete:notClosed
  meaning: "The board is open. Close it first (`vtc/boards/update`, `status: closed`), which ends its voting (VTI-BRD-006)."
  retryable: false
- code: vtc/boards/delete:electionsRunning
  meaning: "An idea's election has not closed or been cancelled — one that does not allow closing early, or one the ballot box has not yet answered closing for. Retry once it has; `vtc/boards/show` names the ideas."
  retryable: true
related:
- vtc/boards/update
- vtc/boards/show
- vtc/elections/cancel
---
## Abstract

The **VTC Boards — Delete** Trust Task deletes a board. Only a closed or archived board whose voting has ended can be deleted: closing a board ends the voting on it (VTI-BRD-006), and deleting it removes what remains (VTI-BRD-007).

Deleting removes the board, its ideas, its notes and each of its elections' content — title, description, roll and the record of the tickets it issued. Of each election the community keeps a **tombstone**: its identifier, its manifest's digest and the board heads the community kept, which are what a voter or auditor needs to check that the ballot box's board was not rewritten (VTI-VOT-051, VTI-VOT-053). Tombstones carry nothing anyone wrote, and are themselves removed when the community's **tombstone retention** passes (VTI-BRD-008): by default twelve months, possibly never, or at once where it is zero.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the Community Boards chapter of the Verifiable Trust Infrastructure specification (VTI-BRD-001 – VTI-BRD-080). The design is recorded in the VTC boards design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-boards.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the board manager's client) **SHOULD** show the board manager, before the request is signed and before any step-up gesture bound to it, how long the board's tombstones will be kept and that, once they are gone, no voter can check a receipt against the community's record of those elections (VTI-BRD-008).

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` for a board the caller may not see, and refuse a caller without `vtc.boards.manage` unqualified with `permissionDenied` (VTI-BRD-007). A facilitator qualified to this board alone may close it, but not delete it.
2. Require a step-up of the requester's bound to this one document (VTI-BRD-007).
3. Refuse a board that is `open` (`notClosed`), and one with an idea whose election has not closed or been cancelled (`electionsRunning`).
4. Remove the board, its ideas and its notes, and, for each election opened on it, everything the community keeps of it but its identifier, its manifest's digest and the board heads it kept. Keep those as the election's tombstone for the tombstone retention in force now (VTI-BRD-008), or keep none where it is zero. **MUST NOT** keep a tombstone's election title, description or roll, or any record of the tickets it issued.
5. Withdraw every appointment to facilitate the board — each resource grant of `vtc.boards.manage` qualified `board:<boardId>` (VTI-BRD-092, [`vtc/boards/facilitators/add`](../../facilitators/add/0.1/spec.md)).
6. Never assign the board's identifier, or any of its elections' identifiers, again.
7. Record an audit event for the deletion, naming the board by its identifier and carrying the `reason`, the counts removed and the retention applied, and a further event when the tombstones are removed (VTI-BRD-080). An audit record **MUST NOT** carry any idea's or note's text.
8. Answer with the counts and `tombstones`.

## Authorization

**Authority: `vtc.boards.manage` unqualified**, read from the signer's access-control entry at execution time — held by those who may create boards and by the community's administrators — and a step-up bound to the request (VTI-BRD-007). The proof establishes who signed; the access-control list decides what the signer may do ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `BoardId` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- **Tombstone** — what the community keeps of a deleted board's election: its identifier, its manifest's digest and the board heads the community kept (VTI-VOT-051).
- **Tombstone retention** — how long the community keeps a deleted board's tombstones, from the deletion: a period in its configuration, by default twelve months, possibly unbounded, or zero (VTI-BRD-008).

## Request

The board manager (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Delete a closed board

```json
{
  "id": "urn:uuid:5c2e8a1d-3f7b-4b9e-9a0c-000000000001",
  "type": "https://trusttasks.org/spec/vtc/boards/delete/0.1#request",
  "issuer": "did:example:aria",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-20T09:00:00Z",
  "payload": {
    "boardId": "brd_Uc7N2qLm4Rt8Vx1Z",
    "reason": "Trust Week is over; the timetable is published elsewhere."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:aria#key-1",
    "created": "2026-11-20T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

On success the community answers with a document of type `https://trusttasks.org/spec/vtc/boards/delete/0.1#response` whose `payload` validates against `$defs.Response`. A refusal is a `trust-task-error` document carrying one of the codes above or a framework standard code.

### Deleted, tombstones kept for twelve months

```json
{
  "id": "urn:uuid:5c2e8a1d-3f7b-4b9e-9a0c-000000000002",
  "type": "https://trusttasks.org/spec/vtc/boards/delete/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:aria",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:5c2e8a1d-3f7b-4b9e-9a0c-000000000001",
  "payload": {
    "boardId": "brd_Uc7N2qLm4Rt8Vx1Z",
    "ideasRemoved": 18,
    "notesRemoved": 42,
    "electionsRemoved": 14,
    "tombstones": {
      "kept": 14,
      "removeAt": "2027-11-20T09:00:01Z"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:community#key-1",
    "created": "2026-11-20T09:00:01Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Security & Privacy

### Data carried

In: a board identifier and a reason. Out: counts, and when the tombstones go.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by. Nothing a member wrote is carried, and no member is named.

### Retention

**Durable** for the record of the deletion. The board, its ideas and notes, and its elections' content are removed. Tombstones are kept for the tombstone retention and then removed; while kept they carry no text and no party.

### Consent/purpose

Ending a board the community no longer needs, and, where it chooses, keeping the evidence that its elections were counted as cast for as long as it wants it checkable.
