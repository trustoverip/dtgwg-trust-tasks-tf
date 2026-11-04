---
slug: vtc/boards/notes/hide
version: '0.1'
title: "VTC Boards — Notes — Hide"
summary: "A facilitator hides a note on their board, with a reason the board's audience sees in its place."
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
  rationale: "Hiding a member's note is moderation; it must stay attributable to the facilitator."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor."
sideEffects:
  level: mutating
  rationale: "Hides the note; `unhide` shows it again."
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: "In: the note's identifier and a reason. Out: the note."
retention:
  class: durable
  rationale: "The moderation decision is a board record."
errorCodes:
- code: vtc/boards/notes/hide:noteNotFound
  meaning: "No note the caller may see has this identifier."
  retryable: false
- code: vtc/boards/notes/hide:stateInvalid
  meaning: "`hide` on a note not `visible`, or `unhide` on a note not `hidden`."
  retryable: false
- code: vtc/boards/notes/hide:reasonRequired
  meaning: "`hide` carries no `reason`."
  retryable: false
related:
- vtc/boards/notes/add
- vtc/boards/notes/withdraw
---
## Abstract

The **VTC Boards — Notes — Hide** Trust Task lets a facilitator hide a note on their board, stating a reason the audience sees in the note's place, or show a note they hid again (VTI-BRD-021). The author and facilitators still read a hidden note's text. A facilitator cannot change a note's text.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the Community Boards chapter of the Verifiable Trust Infrastructure specification (VTI-BRD-001 – VTI-BRD-080). The design is recorded in the VTC boards design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-boards.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `noteNotFound` for a note the caller may not see; refuse a caller who is not a facilitator of the note's board with `permissionDenied`.
2. Refuse `hide` without `reason` (`reasonRequired`) and an action the note's state does not admit (`stateInvalid`).
3. Move the note to `hidden` with `hiddenReason`, or back to `visible`, answer it, and record an audit event naming the note by reference (VTI-BRD-080).

## Authorization

**Authority: `vtc.boards.manage` for the note's board.** Never a portal key (VTI-BRD-051).

## Definitions

- `Note` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `NoteId` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `Reason` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).

## Request

The facilitator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Hide a note

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "type": "https://trusttasks.org/spec/vtc/boards/notes/hide/0.1#request",
  "issuer": "did:example:aria",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-04T06:00:00Z",
  "payload": {
    "noteId": "bnt_Xa4Fh7Ty2Gp9Lm6C",
    "action": "hide",
    "reason": "Off topic for this board."
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

On success the community answers with a document of type `https://trusttasks.org/spec/vtc/boards/notes/hide/0.1#response` whose `payload` validates against `$defs.Response`. A refusal is a `trust-task-error` document carrying one of the codes above or a framework standard code.

### The note, hidden

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000002",
  "type": "https://trusttasks.org/spec/vtc/boards/notes/hide/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:aria",
  "issuedAt": "2026-11-04T06:00:00Z",
  "threadId": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "payload": {
    "note": {
      "noteId": "bnt_Xa4Fh7Ty2Gp9Lm6C",
      "boardId": "brd_Uc7N2qLm4Rt8Vx1Z",
      "ideaId": "ida_Pk3Wq9Ls2Dn6Hy4B",
      "author": {
        "entryId": "dir_Hm2Vt6Xs9Qe4Jn1F",
        "name": "Tomás Brandt"
      },
      "text": "Happy to co-host the screen-reader half.",
      "createdAt": "2026-11-04T05:00:00Z",
      "state": "hidden",
      "hiddenReason": "Off topic for this board."
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

In: a note's identifier and a reason. Out: the note.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by, and a pairwise identifier per caller would make a board's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The hiding facilitator is audited, not shown.

### Retention

The decision is kept with the note.

### Consent/purpose

The facilitator moderates the board they run.
