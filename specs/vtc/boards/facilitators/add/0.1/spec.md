---
slug: vtc/boards/facilitators/add
version: '0.1'
title: "VTC Boards — Facilitators — Add"
summary: "A facilitator of a board appoints an active member a facilitator of that board — an access-control resource grant of vtc.boards.manage qualified to that board."
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
  rationale: "Gives a member authority over a board's ideas, notes and selection. Must be attributable after the transport has closed."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed appointment would restore a facilitator since removed."
sideEffects:
  level: mutating
  rationale: "Writes a resource grant onto the appointee's access-control entry. Recoverable with vtc/boards/facilitators/remove."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "Names the appointee by entry and returns the board's appointed facilitators by entry, never by DID."
retention:
  class: durable
  rationale: "The grant is the appointee's authority over the board for as long as it stands."
errorCodes:
- code: vtc/boards/facilitators/add:notFound
  meaning: "No board the caller facilitates has this identifier, and the caller may not see it (VTI-BRD-003)."
  retryable: false
- code: vtc/boards/facilitators/add:entryNotFound
  meaning: "No active member has this entry."
  retryable: false
- code: vtc/boards/facilitators/add:alreadyFacilitator
  meaning: "The member already facilitates the board — by an appointment to it, or by holding vtc.boards.manage unqualified."
  retryable: false
related:
- vtc/boards/facilitators/remove
- vtc/boards/facilitators/list
- vtc/boards/show
- vtc/groups/managers/add
---
## Abstract

The **VTC Boards — Facilitators — Add** Trust Task appoints a community member a facilitator of one board (VTI-BRD-090). A facilitator decides the board's ideas, hides its notes, selects among its tallied ideas, closes it, and appoints co-facilitators.

The appointment is not a field of the board. It is an **access-control resource grant** on the appointee's entry — `vtc.boards.manage` qualified `board:<boardId>` — so the community's access-control list stays the one place its authority is recorded, and every gate reads it there. It mirrors [`vtc/groups/managers/add`](../../../../groups/managers/add/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the Community Boards chapter of the Verifiable Trust Infrastructure specification (VTI-BRD-090 – VTI-BRD-093). The design is recorded in the VTC boards design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-boards.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who does not facilitate the board with `notFound` when they may not see it, and `permissionDenied` when they may.
2. Refuse an entry that is not an active member with `entryNotFound`, and one that already facilitates the board — by a grant qualified to it, or by holding `vtc.boards.manage` unqualified — with `alreadyFacilitator`.
3. Write the appointment as a resource grant on the appointee's access-control entry: capability `vtc.boards.manage`, qualifier `board:<boardId>`, delegated by the signer. The grant sits beside the entry's administrative role, never inside its ceiling, so a member with no administrative role may hold it.
4. Bound the grant by the signer's own holding: the signer covers `vtc.boards.manage` for this board at the moment of writing, and a grant the signer does not cover is never written (VTI-ACL-071).
5. Put the grant to review when its granter departs, or no longer covers it, and withdraw it unless an administrator who facilitates the board re-affirms it by the review's deadline; never leave it standing silently (VTI-BRD-092).
6. Withdraw the grant when the board is deleted, and when the appointee leaves the community (VTI-BRD-092).
7. Audit the appointment by reference — the board, the appointee's entry and the granter (VTI-BRD-080) — and **MUST NOT** treat the board's record as the source of who facilitates it.

A facilitator's authority is over the board alone. It confers nothing else: the elections they open come from the template an elections manager fixed, and deleting the board still takes `vtc.boards.manage` unqualified (VTI-BRD-007).

## Authorization

**Authority: `vtc.boards.manage`** for the board — unqualified, or qualified `board:<boardId>` — read from the signer's access-control entry at execution time. A facilitator may appoint a co-facilitator of the board they facilitate; only a holder of the unqualified capability appoints across boards. A key a member portal holds for the member (VTI-BRD-050) is never accepted for this task (VTI-BRD-051). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `BoardId` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `EntryId` — see [`directory-card.schema.json`](../../../../directory/_shared/0.1/directory-card.schema.json).

## Request

The board facilitator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Appoint a facilitator

```json
{
  "id": "urn:uuid:5b8e2f1a-3c4d-4e5f-9a0b-000000000111",
  "type": "https://trusttasks.org/spec/vtc/boards/facilitators/add/0.1#request",
  "issuer": "did:example:facilitator",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-05T09:00:00Z",
  "threadId": "urn:uuid:5b8e2f1a-3c4d-4e5f-9a0b-000000000110",
  "payload": {
    "boardId": "brd_W5k2N8xRb3Qz7TpLq4Ve",
    "entryId": "dir_Q7m2K9xVb4Tz8LpR",
    "reason": "Carol convenes the security track."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:facilitator#key-1",
    "created": "2026-11-05T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/boards/facilitators/add/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Appoint a facilitator — answered

```json
{
  "id": "urn:uuid:5b8e2f1a-3c4d-4e5f-9a0b-000000000112",
  "type": "https://trusttasks.org/spec/vtc/boards/facilitators/add/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:facilitator",
  "issuedAt": "2026-11-05T09:00:01Z",
  "threadId": "urn:uuid:5b8e2f1a-3c4d-4e5f-9a0b-000000000110",
  "payload": {
    "boardId": "brd_W5k2N8xRb3Qz7TpLq4Ve",
    "facilitators": [
      "dir_Q7m2K9xVb4Tz8LpR"
    ]
  }
}
```

## Security & Privacy

### Data carried

A board and an entry; the board's appointed facilitators by entry.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the community differ by who asks. Facilitators are named by their community-scoped `entryId`, never by DID (VTI-BRD-093).

### Retention

**Durable** for the grant, until it is removed, its board is deleted, or its holder leaves the community.

### Consent/purpose

Delegating the running of one board to a member. The grant is authority at the community, so it lives in the access-control list and is reviewed with every other delegated grant; the board's record never decides who facilitates it.

