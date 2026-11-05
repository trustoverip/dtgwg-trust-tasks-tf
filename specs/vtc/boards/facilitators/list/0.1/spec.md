---
slug: vtc/boards/facilitators/list
version: '0.1'
title: "VTC Boards — Facilitators — List"
summary: "A facilitator of a board reads who is appointed to facilitate it: each appointee by entry, when and by whom they were appointed, and whether the appointment is under review."
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
  requirement: RECOMMENDED
  rationale: "Who may read depends on who asks; an attributable caller gets a correctly scoped answer."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no side effects; a replay discloses only what the caller was already entitled to see."
sideEffects:
  level: none
  rationale: "Reads the access-control list; persists nothing."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Out: the board's appointed facilitators by entry, never by DID."
retention:
  class: transient
  rationale: "A read."
errorCodes:
- code: vtc/boards/facilitators/list:notFound
  meaning: "No board the caller facilitates has this identifier, and the caller may not see it (VTI-BRD-003)."
  retryable: false
related:
- vtc/boards/facilitators/add
- vtc/boards/facilitators/remove
- vtc/boards/show
---
## Abstract

The **VTC Boards — Facilitators — List** Trust Task returns a board's appointed facilitators (VTI-BRD-093): every entry holding `vtc.boards.manage` qualified to the board, with when and by whom it was appointed and whether the appointment is under review. It is a read of its own rather than a member of [`vtc/boards/show`](../../../show/0.1/spec.md), because what a board shows its audience does not change with who facilitates it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the Community Boards chapter of the Verifiable Trust Infrastructure specification (VTI-BRD-090 – VTI-BRD-093). The design is recorded in the VTC boards design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-boards.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who does not facilitate the board with `notFound` when they may not see it, and `permissionDenied` when they may.
2. Return every live grant of `vtc.boards.manage` qualified `board:<boardId>`, read from the access-control list at the time of the read, naming each holder and granter by directory entry and never by DID.
3. Not list holders of `vtc.boards.manage` unqualified, or through a role or a group: they are not appointed to this board.

## Authorization

**Authority: `vtc.boards.manage`** for the board — unqualified, or qualified `board:<boardId>` — read from the caller's access-control entry at execution time.

## Definitions

- `BoardId` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `EntryId` — see [`directory-card.schema.json`](../../../../directory/_shared/0.1/directory-card.schema.json).

## Request

The board facilitator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Read a board's facilitators

```json
{
  "id": "urn:uuid:5b8e2f1a-3c4d-4e5f-9a0b-000000000131",
  "type": "https://trusttasks.org/spec/vtc/boards/facilitators/list/0.1#request",
  "issuer": "did:example:facilitator",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-05T09:00:00Z",
  "threadId": "urn:uuid:5b8e2f1a-3c4d-4e5f-9a0b-000000000130",
  "payload": {
    "boardId": "brd_W5k2N8xRb3Qz7TpLq4Ve"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/boards/facilitators/list/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Read a board's facilitators — answered

```json
{
  "id": "urn:uuid:5b8e2f1a-3c4d-4e5f-9a0b-000000000132",
  "type": "https://trusttasks.org/spec/vtc/boards/facilitators/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:facilitator",
  "issuedAt": "2026-11-05T09:00:01Z",
  "threadId": "urn:uuid:5b8e2f1a-3c4d-4e5f-9a0b-000000000130",
  "payload": {
    "boardId": "brd_W5k2N8xRb3Qz7TpLq4Ve",
    "facilitators": [
      {
        "entryId": "dir_Q7m2K9xVb4Tz8LpR",
        "appointedAt": "2026-11-01T10:00:00Z",
        "appointedBy": "dir_K3n8P2xWc5Rz9MqT"
      },
      {
        "entryId": "dir_T4v9L3xZd6Sq2NpW",
        "appointedAt": "2026-11-02T14:30:00Z",
        "reviewEndsAt": "2026-11-12T14:30:00Z"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

A board; its appointed facilitators and their granters by entry, with times.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the community differ by who asks. Facilitators are named by their community-scoped `entryId`, never by DID (VTI-BRD-093).

### Retention

**Transient**: a read.

### Consent/purpose

Letting a board's facilitators see who shares the board's running, and which appointments are about to lapse.

