---
slug: vtc/boards/list
version: '0.1'
title: "VTC Boards — List"
summary: "List the boards the caller may see: those whose audience admits them, and every board they facilitate."
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
  rationale: "What is listed depends on who asks; an attributable caller gets a correctly scoped page."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no side effects; a replay discloses only what the caller was already entitled to see."
sideEffects:
  level: none
  rationale: "Reads board records; persists nothing."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Out: the settings of the boards the caller may see, and how many ideas each holds in each state."
retention:
  class: transient
  rationale: "A read."
errorCodes:
- code: vtc/boards/list:cursorInvalid
  meaning: "`cursor` was not issued for this caller and these filters."
  retryable: false
related:
- vtc/boards/show
---
## Abstract

The **VTC Boards — List** Trust Task returns the boards the caller may see (VTI-BRD-003): every board whose audience admits them as the community's records stand now, and every board they facilitate. A board the caller may not see is not listed, not even by identifier. Archived boards are listed only to their facilitators unless `includeArchived` is false, which hides them from everyone.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the Community Boards chapter of the Verifiable Trust Infrastructure specification (VTI-BRD-001 – VTI-BRD-080). The design is recorded in the VTC boards design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-boards.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer an active member, or a party holding `vtc.boards.manage` for any board; refuse anyone else with `permissionDenied`.
2. List exactly the boards the caller may see (VTI-BRD-003), omitting archived boards for a caller who does not facilitate them and, when `includeArchived` is false, for everyone. Filter by `status` where given.
3. Order by `createdAt`, newest first, page with `cursor` and `limit`, and bind each cursor to the caller and the filters.

## Authorization

**Authority: being an active member, or a facilitator.** What is listed is decided per board by VTI-BRD-003; the proof establishes who is asking.

## Definitions

- `Board` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).
- `BoardStatus` — see [`board.schema.json`](../../_shared/0.1/board.schema.json).

## Request

The member or facilitator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### The open boards

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "type": "https://trusttasks.org/spec/vtc/boards/list/0.1#request",
  "issuer": "did:example:elif",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-04T06:00:00Z",
  "payload": {
    "status": "open"
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

On success the community answers with a document of type `https://trusttasks.org/spec/vtc/boards/list/0.1#response` whose `payload` validates against `$defs.Response`. A refusal is a `trust-task-error` document carrying one of the codes above or a framework standard code.

### One board

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000002",
  "type": "https://trusttasks.org/spec/vtc/boards/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:elif",
  "issuedAt": "2026-11-04T06:00:00Z",
  "threadId": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "payload": {
    "boards": [
      {
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

Out: board settings and counts of ideas by state; no idea, note or author.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by, and a pairwise identifier per caller would make a board's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. Only boards the caller may see are named, so a list does not reveal a group-only board to a non-member.

### Retention

Nothing is kept.

### Consent/purpose

A member reads what the community put in front of them.
