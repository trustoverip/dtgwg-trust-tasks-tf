---
slug: vtc/boards/ideas/select
version: '0.1'
title: "VTC Boards — Ideas — Select"
summary: "After an idea's election is tallied, a facilitator records whether it is selected, optionally placing it in a timetable slot, with the result the decision was made with."
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
  rationale: "Selecting an idea is a facilitator's decision; it must stay attributable to them."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed select could undo a later one."
sideEffects:
  level: mutating
  rationale: "Records the decision; a facilitator may decide again, moving the idea between `selected` and `notSelected` or between slots."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "In: the decision, a slot and a reason. Out: the idea as decided."
retention:
  class: durable
  rationale: "The decision and the result it was made with are board records."
errorCodes:
- code: vtc/boards/ideas/select:notFound
  meaning: "No board the caller may see has this identifier (VTI-BRD-003)."
  retryable: false
- code: vtc/boards/ideas/select:ideaNotFound
  meaning: "No idea on this board the caller may see has this identifier."
  retryable: false
- code: vtc/boards/ideas/select:notTallied
  meaning: "The idea's election has not been tallied."
  retryable: false
- code: vtc/boards/ideas/select:stateInvalid
  meaning: "The idea is not `open`, `selected` or `notSelected`."
  retryable: false
- code: vtc/boards/ideas/select:slotUnknown
  meaning: "`slotId` names no slot of the board's timetable."
  retryable: false
- code: vtc/boards/ideas/select:slotTaken
  meaning: "Another selected idea is in `slotId`."
  retryable: false
- code: vtc/boards/ideas/select:reasonRequired
  meaning: "`departsFromRule` is true and no `reason` is given (VTI-BRD-041)."
  retryable: false
related:
- vtc/boards/ideas/decide
- vtc/elections/certify
---
## Abstract

The **VTC Boards — Ideas — Select** Trust Task records a facilitator's decision on an idea whose election has been tallied: `selected`, optionally in a slot of the board's timetable, or `notSelected` (VTI-BRD-040).

The community records with the decision the result it was made with — the election, the board head the result was computed from, and whether it was certified — read from its own records, never from the request. The decision may depart from the order of the results; where the board states a selection rule, the facilitator says so, and why (VTI-BRD-041). Selecting an idea grants nothing (VTI-BRD-042).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the Community Boards chapter of the Verifiable Trust Infrastructure specification (VTI-BRD-001 – VTI-BRD-080). The design is recorded in the VTC boards design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-boards.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` and `ideaNotFound` as `vtc/boards/show` would; refuse a caller who is not a facilitator of the board with `permissionDenied`.
2. Refuse an idea not `open`, `selected` or `notSelected` (`stateInvalid`), and one whose election has not been tallied (`notTallied`).
3. Refuse a slot not in the timetable (`slotUnknown`) or held by another selected idea (`slotTaken`), and `departsFromRule` without `reason` (`reasonRequired`).
4. Record the decision with the election's identifier, the head of the result it read, and whether the result was certified, all from the community's own records (VTI-BRD-040); answer the idea.
5. **MUST NOT** grant, widen or remove any authority, or admit anyone to anything, because of the decision (VTI-BRD-042).
6. Record an audit event for the decision (VTI-BRD-080).

## Authorization

**Authority: `vtc.boards.manage` for the board**, unqualified or qualified `board:<boardId>`. Never a portal key (VTI-BRD-051).

## Definitions

- `Idea` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `IdeaId` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `Reason` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).

## Request

The facilitator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Select an idea into a slot

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "type": "https://trusttasks.org/spec/vtc/boards/ideas/select/0.1#request",
  "issuer": "did:example:aria",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-04T06:00:00Z",
  "payload": {
    "ideaId": "ida_Pk3Wq9Ls2Dn6Hy4B",
    "decision": "selected",
    "slotId": "a-0900"
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

On success the community answers with a document of type `https://trusttasks.org/spec/vtc/boards/ideas/select/0.1#response` whose `payload` validates against `$defs.Response`. A refusal is a `trust-task-error` document carrying one of the codes above or a framework standard code.

### The idea, selected

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000002",
  "type": "https://trusttasks.org/spec/vtc/boards/ideas/select/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:aria",
  "issuedAt": "2026-11-04T06:00:00Z",
  "threadId": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "payload": {
    "idea": {
      "ideaId": "ida_Pk3Wq9Ls2Dn6Hy4B",
      "boardId": "brd_Uc7N2qLm4Rt8Vx1Z",
      "state": "selected",
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
        "state": "tallied",
        "certified": false,
        "result": {
          "electionId": "board.brd_Uc7N2qLm4Rt8Vx1Z.ida_Pk3Wq9Ls2Dn6Hy4B",
          "manifestDigest": "uEiDexampleDigestNotReal0000000000000000000000",
          "ballotsCounted": 92,
          "questions": [
            {
              "question": "attend",
              "abstained": 0,
              "choice": {
                "yes": 65,
                "no": 27
              }
            }
          ],
          "head": {
            "electionId": "board.brd_Uc7N2qLm4Rt8Vx1Z.ida_Pk3Wq9Ls2Dn6Hy4B",
            "treeSize": 97,
            "rootHash": "uEiBexampleRootNotReal00000000000000000000000",
            "issuedAt": "2026-11-12T09:05:00Z"
          }
        }
      },
      "decision": {
        "decidedAt": "2026-11-12T10:00:00Z",
        "slotId": "a-0900"
      }
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

In: the decision, slot and reason. Out: the idea with its election's result.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by, and a pairwise identifier per caller would make a board's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The result is the election's, computable by anyone who can read its board; nothing links it to a member.

### Retention

The decision is kept with the idea and audited.

### Consent/purpose

The facilitator decides for the board, in view of the results.
