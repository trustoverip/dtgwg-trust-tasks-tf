---
slug: vtc/boards/update
version: '0.1'
title: "VTC Boards — Update"
summary: "A facilitator changes a board's title, description, topics, formats, timetable or selection rule, closes or archives it — which ends the voting on its ideas — or, before voting opens, its schedule; changing its election template takes an elections manager."
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords: [vtc, boards, unconference, ideas, elections]
parties:
- role: facilitator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "A change to a board, and closing it, must stay attributable to the facilitator who made it."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed update could undo a later one."
sideEffects:
  level: mutating
  rationale: "Changes the board; closing and archiving cannot be undone (VTI-BRD-005), and closing closes early or cancels the elections opened on it (VTI-BRD-006)."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "In: the changed settings. Out: the board as changed."
retention:
  class: durable
  rationale: "The board's settings are community records."
errorCodes:
- code: vtc/boards/update:notFound
  meaning: "No board the caller may see has this identifier (VTI-BRD-003)."
  retryable: false
- code: vtc/boards/update:fixedAfterVotingOpens
  meaning: "The change touches the audience or the voting window after voting has opened (VTI-BRD-004)."
  retryable: false
- code: vtc/boards/update:statusInvalid
  meaning: "`status` is not a step forward from the board's current status (`open` → `closed` → `archived`)."
  retryable: false
- code: vtc/boards/update:archived
  meaning: "The board is archived and takes no change (VTI-BRD-005)."
  retryable: false
- code: vtc/boards/update:templateInvalid
  meaning: "The new election template is not one the community would accept, or does not allow closing early (VTI-BRD-030)."
  retryable: false
- code: vtc/boards/update:ballotBoxNotTrusted
  meaning: "The new template's ballot box is not on the community's list (VTI-VOT-082)."
  retryable: false
- code: vtc/boards/update:scheduleInvalid
  meaning: "The schedule that would result breaks VTI-BRD-001's ordering."
  retryable: false
- code: vtc/boards/update:timetableInvalid
  meaning: "Two slots share a `slotId`, a slot ends before it starts, or a slot a selected idea is placed in is removed."
  retryable: false
related:
- vtc/boards/create
- vtc/boards/show
---
## Abstract

The **VTC Boards — Update** Trust Task changes a board. Every member it carries replaces the board's current value; a member it omits is left as it is. `status` moves the board forward: `closed` stops new ideas and notes and ends the voting on the board's ideas — each election open for voting is closed early, so its count is final, and each not yet open is cancelled (VTI-BRD-006) — and `archived` stops every change (VTI-BRD-005).

A facilitator of the board may change everything but the election template, which only a holder of `vtc.elections.manage` may change (VTI-BRD-002). The audience and the voting window are fixed once voting opens (VTI-BRD-004), and a changed template applies only to elections opened after it; an election already registered keeps its manifest (VTI-VOT-013).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the Community Boards chapter of the Verifiable Trust Infrastructure specification (VTI-BRD-001 – VTI-BRD-080). The design is recorded in the VTC boards design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-boards.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` for a board the caller may not see; refuse a caller who is not a facilitator of it with `permissionDenied`; refuse a `template` from a caller without `vtc.elections.manage` and `vtc.boards.manage` unqualified with `permissionDenied` (VTI-BRD-002).
2. Refuse any change to an archived board (`archived`), a change to `audience` or `voting` once voting has opened (`fixedAfterVotingOpens`), and a `status` that is not a step forward (`statusInvalid`).
3. Check the board as it would be after the change, as `vtc/boards/create` items 2–4 check a new one (`scheduleInvalid`, `timetableInvalid`, `templateInvalid`, `ballotBoxNotTrusted`), and refuse removing a slot a selected idea is placed in (`timetableInvalid`).
4. Apply the change atomically, answer the board as `vtc/boards/show` would, and record an audit event for it (VTI-BRD-080).
5. **MUST NOT** change the manifest of an election already opened for an idea; a changed template applies only to elections opened after it.
6. On a `status` that closes the board — `closed`, or `archived` from `open` — record the board as closed first, then end the voting on it (VTI-BRD-006): close early each election open for voting whose manifest allows it, cancel each not yet open for voting or not yet registered (its idea moves to `notSelected`), and leave each that does not allow closing early to close when its manifest says. Keep asking the ballot box until it has answered each request; the community's election retry owner owns those retries, and the answer does not wait for them. Answer `closing` with what was asked for and which ideas run on to their own close. The closing facilitator need not hold `vtc.elections.manage` (VTI-VOT-080).
7. Refuse a `template` that does not allow closing early (`templateInvalid`, VTI-BRD-030).

## Authorization

**Authority: `vtc.boards.manage` for the board**, unqualified or qualified `board:<boardId>`; for `template`, `vtc.boards.manage` unqualified and `vtc.elections.manage` (VTI-BRD-002). The proof establishes who signed; the access-control list decides what the signer may do.

## Definitions

- `Board` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `BoardAudience` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `BoardDescription` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `BoardId` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `BoardTitle` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `ElectionTemplate` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `IdeaFormat` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `Slot` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `Topic` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `Window` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).

## Request

The facilitator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Close a board to new ideas

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "type": "https://trusttasks.org/spec/vtc/boards/update/0.1#request",
  "issuer": "did:example:aria",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-04T06:00:00Z",
  "payload": {
    "boardId": "brd_Uc7N2qLm4Rt8Vx1Z",
    "status": "closed"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:aria#key-1",
    "created": "2026-11-04T06:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

On success the community answers with a document of type `https://trusttasks.org/spec/vtc/boards/update/0.1#response` whose `payload` validates against `$defs.Response`. A refusal is a `trust-task-error` document carrying one of the codes above or a framework standard code.

### The board, closed before voting opened: its four open ideas' elections cancelled

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000002",
  "type": "https://trusttasks.org/spec/vtc/boards/update/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:aria",
  "issuedAt": "2026-11-04T06:00:00Z",
  "threadId": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "payload": {
    "board": {
      "boardId": "brd_Uc7N2qLm4Rt8Vx1Z",
      "title": "Trust Week Unconference",
      "description": "Propose a session for Friday. The sessions with the most support fill the twelve slots.",
      "status": "closed",
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
      "facilitator": true,
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
    "closing": {
      "closedEarly": 0,
      "cancelled": 4,
      "runningToClose": []
    }
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

In: the changed settings — community settings, no member's personal data. Out: the board.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by, and a pairwise identifier per caller would make a board's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. Nothing a member wrote is carried.

### Retention

The change is kept as the board's current state, and audited.

### Consent/purpose

The facilitator acts for the community within the board they run.
