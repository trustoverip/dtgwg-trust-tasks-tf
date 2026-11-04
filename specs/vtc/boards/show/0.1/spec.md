---
slug: vtc/boards/show
version: '0.1'
title: "VTC Boards — Show"
summary: "Show one board and its ideas as the caller may see them, with each accepted idea's election stage, ballot count where its board is public, and result once tallied — never who took a ticket or voted."
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords: [vtc, boards, unconference, ideas, elections]
parties:
- role: member or facilitator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: "What is shown depends on who asks: authors and facilitators see more."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no side effects; a replay discloses only what the caller was already entitled to see."
sideEffects:
  level: none
  rationale: "Reads the board, its ideas, and its elections' public state; persists nothing."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Out: the board, its ideas with their authors' directory names, and its elections' identifiers, stages, public ballot counts and results."
retention:
  class: transient
  rationale: "A read."
errorCodes:
- code: vtc/boards/show:notFound
  meaning: "No board the caller may see has this identifier (VTI-BRD-003)."
  retryable: false
- code: vtc/boards/show:cursorInvalid
  meaning: "`cursor` was not issued for this caller, this board and these filters."
  retryable: false
related:
- vtc/boards/list
- vtc/boards/notes/list
- vtc/elections/show
---
## Abstract

The **VTC Boards — Show** Trust Task returns one board and a page of its ideas, as the caller may see them.

For each idea an election was opened for, it carries the election's identifier and stage; while voting is open and the ballot box's board is public, the number of ballots on it; and, once tallied, the result (VTI-BRD-033, VTI-BRD-041). It never says whether anyone — the caller included — took a ticket or cast a ballot: a member's portal learns that only from the member's own wallet. Facilitators also see `opening` for an accepted idea the ballot box has not registered yet (VTI-BRD-031).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the Community Boards chapter of the Verifiable Trust Infrastructure specification (VTI-BRD-001 – VTI-BRD-080). The design is recorded in the VTC boards design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-boards.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` for a board the caller may not see (VTI-BRD-003).
2. Return the board and a page of its ideas, filtered by `state` and `topic` where given, `withdrawn` ideas only to their proposer and the board's facilitators, ordered by `proposedAt`, newest first.
3. Show each author as the community's directory would show that member to the caller (`name` absent where it is withheld), mark the caller's own with `you`, and show a departed member as `formerMember` (VTI-BRD-060).
4. For an idea with an election, carry its identifier and stage; `ballotsOnBoard` and `boardHeadIssuedAt` only while voting is open and only from a head the ballot box signed of a public board; and `result` and `certified` once tallied. **MUST NOT** carry, or let the caller infer, whether any party took a ticket or cast a ballot (VTI-BRD-033), and **SHOULD NOT** carry a running count of choices while voting is open.
5. Carry `opening` only to the board's facilitators.
6. Count in `noteCount` only the notes the caller may read (VTI-BRD-021).

## Authorization

**Authority: being in the board's audience, or a facilitator of it** (VTI-BRD-003). The proof establishes who is asking.

## Definitions

- `Board` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `BoardId` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `Idea` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `IdeaState` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `Topic` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).

## Request

The member or facilitator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A board's open ideas

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "type": "https://trusttasks.org/spec/vtc/boards/show/0.1#request",
  "issuer": "did:example:elif",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-04T06:00:00Z",
  "payload": {
    "boardId": "brd_Uc7N2qLm4Rt8Vx1Z",
    "state": [
      "open"
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:elif#key-1",
    "created": "2026-11-04T06:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

On success the community answers with a document of type `https://trusttasks.org/spec/vtc/boards/show/0.1#response` whose `payload` validates against `$defs.Response`. A refusal is a `trust-task-error` document carrying one of the codes above or a framework standard code.

### One idea open for votes

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000002",
  "type": "https://trusttasks.org/spec/vtc/boards/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:elif",
  "issuedAt": "2026-11-04T06:00:00Z",
  "threadId": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "payload": {
    "board": {
      "boardId": "brd_Uc7N2qLm4Rt8Vx1Z",
      "title": "Trust Week Unconference",
      "description": "Propose a session for Friday. The sessions with the most support fill the twelve slots.",
      "status": "open",
      "audience": {
        "kind": "members"
      },
      "proposals": {
        "opensAt": "2026-11-02T00:00:00Z",
        "closesAt": "2026-11-10T00:00:00Z"
      },
      "voting": {
        "opensAt": "2026-11-09T00:00:00Z",
        "closesAt": "2026-11-12T09:00:00Z"
      },
      "topics": [
        "Identity",
        "Governance",
        "Voting"
      ],
      "formats": [
        "Discussion",
        "Talk",
        "Workshop",
        "Demo"
      ],
      "maxIdeasPerMember": 3,
      "selectionRule": "The twelve sessions the most members would attend.",
      "timetable": [
        {
          "slotId": "a-0900",
          "label": "Room A",
          "startsAt": "2026-11-13T01:00:00Z",
          "endsAt": "2026-11-13T02:00:00Z"
        }
      ],
      "template": {
        "questions": [
          {
            "key": "attend",
            "type": "choice",
            "text": "Would you attend this session?",
            "options": [
              {
                "key": "yes",
                "label": "Yes, I'd attend"
              },
              {
                "key": "no",
                "label": "Not for me"
              }
            ],
            "minSelections": 1,
            "maxSelections": 1,
            "optional": false
          }
        ],
        "secrecy": "open",
        "revote": "replace",
        "board": {
          "visibility": "public",
          "auditors": []
        },
        "ballotBox": "did:example:ballot-box",
        "allowEarlyClose": true,
        "resultsBeforeClose": false,
        "textReleaseThreshold": 10
      },
      "createdAt": "2026-11-01T03:00:00Z",
      "facilitator": false,
      "ideaCounts": {
        "proposed": 2,
        "open": 4,
        "selected": 0,
        "notSelected": 0,
        "declined": 0,
        "merged": 0,
        "withdrawn": 0
      }
    },
    "ideas": [
      {
        "ideaId": "ida_Pk3Wq9Ls2Dn6Hy4B",
        "boardId": "brd_Uc7N2qLm4Rt8Vx1Z",
        "state": "open",
        "title": "Accessible wallet sign-in",
        "description": "The on-screen number, screen readers, and people who can't scan a code. List the barriers and pick three to fix.",
        "format": "Discussion",
        "topic": "Identity",
        "proposer": {
          "entryId": "dir_Rb8Nc3Kq7Wz2Lp5D",
          "name": "Elif Polat"
        },
        "proposedAt": "2026-11-04T02:10:00Z",
        "noteCount": 2,
        "election": {
          "electionId": "board.brd_Uc7N2qLm4Rt8Vx1Z.ida_Pk3Wq9Ls2Dn6Hy4B",
          "state": "voting",
          "ballotsOnBoard": 38,
          "boardHeadIssuedAt": "2026-11-10T04:00:00Z"
        }
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:community#key-1",
    "created": "2026-11-04T06:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Security & Privacy

### Data carried

Out: the board, its ideas, their authors' directory names as the caller may see them, and each idea's election's identifier, stage, public ballot count and result.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by, and a pairwise identifier per caller would make a board's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. Authors are named, by design (VTI-BRD-070). Nothing links a member to a ticket or a ballot: the board holds no such record and the response carries none.

### Retention

Nothing is kept.

### Consent/purpose

A member reads what the board's audience was told it would see.
