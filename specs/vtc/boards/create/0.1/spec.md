---
slug: vtc/boards/create
version: '0.1'
title: "VTC Boards — Create"
summary: "An administrator creates a board where members propose ideas, leave notes and vote on each accepted idea through an election of its own."
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords: [vtc, boards, unconference, ideas, elections]
parties:
- role: administrator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "Creating a board fixes the election template its facilitators will open elections from; who did so must stay attributable."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed create would make a second board."
sideEffects:
  level: mutating
  rationale: "Creates a board. Recoverable by archiving it before anything is proposed."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "In: the board's settings. Out: the board as created."
retention:
  class: durable
  rationale: "The board is a community record for as long as the elections it opens are kept."
errorCodes:
- code: vtc/boards/create:templateInvalid
  meaning: "The election template is not one the community would accept as an election: a question the voting rules refuse, a secrecy level that needs trustees without them, or a template the board rules forbid (VTI-BRD-030)."
  retryable: false
- code: vtc/boards/create:ballotBoxNotTrusted
  meaning: "`template.ballotBox` is not on the community's list of ballot boxes (VTI-VOT-082)."
  retryable: false
- code: vtc/boards/create:groupNotFound
  meaning: "`audience.groupId` names no group of the community."
  retryable: false
- code: vtc/boards/create:scheduleInvalid
  meaning: "A window closes before it opens, voting opens before proposals open, or voting closes before proposals close (VTI-BRD-001)."
  retryable: false
- code: vtc/boards/create:timetableInvalid
  meaning: "Two slots share a `slotId`, or a slot ends before it starts."
  retryable: false
related:
- vtc/boards/update
- vtc/boards/show
- vtc/elections/create
---
## Abstract

The **VTC Boards — Create** Trust Task creates a **board**: a shared space where the members its audience admits propose ideas and leave notes, each idea a facilitator accepts is put to an election of its own, and the board's facilitators select among the ideas once the counts are in (VTI Community Boards chapter).

The board's **election template** fixes everything about those elections except what the community fills in per idea, and is never binding. That is why creating a board takes `vtc.elections.manage` as well as `vtc.boards.manage`: a facilitator qualified to one board opens elections only from the template an elections manager fixed here.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the Community Boards chapter of the Verifiable Trust Infrastructure specification (VTI-BRD-001 – VTI-BRD-080). The design is recorded in the VTC boards design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-boards.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller holding `vtc.boards.manage` unqualified and `vtc.elections.manage` not both with `permissionDenied` (VTI-BRD-002).
2. Check the schedule (`scheduleInvalid`): each window closes after it opens, voting opens after proposals open, and voting closes after proposals close. Check the timetable (`timetableInvalid`).
3. Check the template as `vtc/elections/create` checks a manifest, with the members the community fills in per idea (`ElectionTemplate`) set as it would set them for an idea accepted now, and refuse a template that does not allow closing early, which closing the board needs (VTI-BRD-006, VTI-BRD-030) (`templateInvalid`); refuse a ballot box not on the community's list (`ballotBoxNotTrusted`).
4. Refuse an `audience` naming a group the community does not have (`groupNotFound`).
5. Assign a fresh `boardId`, never reused, store the board with `status` `open`, and answer it as `vtc/boards/show` would answer the caller.
6. Record an audit event for the creation, naming the board by its identifier (VTI-BRD-080).

## Authorization

**Authority: `vtc.boards.manage` and `vtc.elections.manage`, both unqualified, on the caller's live access-control entry** (VTI-BRD-002). The proof establishes who signed ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10); the community's access-control list decides what the signer may do.

## Definitions

- `Board` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `BoardAudience` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `BoardDescription` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `BoardTitle` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `ElectionTemplate` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `IdeaFormat` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `Slot` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `Topic` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `Window` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).

## Request

The administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Create an unconference board

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "type": "https://trusttasks.org/spec/vtc/boards/create/0.1#request",
  "issuer": "did:example:aria",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-04T06:00:00Z",
  "payload": {
    "title": "Trust Week Unconference",
    "description": "Propose a session for Friday. The sessions with the most support fill the twelve slots.",
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
    }
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

On success the community answers with a document of type `https://trusttasks.org/spec/vtc/boards/create/0.1#response` whose `payload` validates against `$defs.Response`. A refusal is a `trust-task-error` document carrying one of the codes above or a framework standard code.

### The board as created

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000002",
  "type": "https://trusttasks.org/spec/vtc/boards/create/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:aria",
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
      "facilitator": true,
      "ideaCounts": {}
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

In: the board's title, description, audience, schedule, topics, formats, timetable and election template — community settings, no member's personal data. Out: the same, as stored.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by, and a pairwise identifier per caller would make a board's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. A board's identifier is random and names nothing about the community or its members.

### Retention

The board is kept while any election it opened is kept (VTI-BRD-005), and its creation is audited.

### Consent/purpose

The administrator acts for the community; the board's disclosure to members is VTI-BRD-070's, shown before a member first writes on it.
