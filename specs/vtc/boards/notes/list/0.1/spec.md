---
slug: vtc/boards/notes/list
version: '0.1'
title: "VTC Boards — Notes — List"
summary: "List a board's notes, or one idea's, as the caller may read them."
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
  rationale: "What is listed depends on who asks: authors and facilitators also read withdrawn and hidden notes' text."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no side effects."
sideEffects:
  level: none
  rationale: "Reads notes; persists nothing."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Out: notes and their authors' directory names."
retention:
  class: transient
  rationale: "A read."
errorCodes:
- code: vtc/boards/notes/list:notFound
  meaning: "No board the caller may see has this identifier (VTI-BRD-003)."
  retryable: false
- code: vtc/boards/notes/list:ideaNotFound
  meaning: "No idea on this board the caller may see has this identifier."
  retryable: false
- code: vtc/boards/notes/list:cursorInvalid
  meaning: "`cursor` was not issued for this caller, board and filters."
  retryable: false
related:
- vtc/boards/notes/add
- vtc/boards/show
---
## Abstract

The **VTC Boards — Notes — List** Trust Task returns a page of notes: those on the board itself when `ideaId` is absent and `scope` is `board` (the default), every note on the board with `scope` `all`, or one idea's with `ideaId`. A withdrawn or hidden note is listed with its state but without its text, except to its author and the board's facilitators (VTI-BRD-021).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the Community Boards chapter of the Verifiable Trust Infrastructure specification (VTI-BRD-001 – VTI-BRD-080). The design is recorded in the VTC boards design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-boards.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` and `ideaNotFound` as `vtc/boards/show` would.
2. List the notes the filters select, oldest first, paged with `cursor` and `limit`.
3. Omit `text` from a `withdrawn` or `hidden` note for anyone but its author and the board's facilitators; show authors as `vtc/boards/show` item 3 does.

## Authorization

**Authority: being in the board's audience, or a facilitator of it.**

## Definitions

- `BoardId` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `IdeaId` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `Note` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).

## Request

The member or facilitator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### An idea's notes

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "type": "https://trusttasks.org/spec/vtc/boards/notes/list/0.1#request",
  "issuer": "did:example:elif",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-04T06:00:00Z",
  "payload": {
    "boardId": "brd_Uc7N2qLm4Rt8Vx1Z",
    "ideaId": "ida_Pk3Wq9Ls2Dn6Hy4B"
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

On success the community answers with a document of type `https://trusttasks.org/spec/vtc/boards/notes/list/0.1#response` whose `payload` validates against `$defs.Response`. A refusal is a `trust-task-error` document carrying one of the codes above or a framework standard code.

### One note

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000002",
  "type": "https://trusttasks.org/spec/vtc/boards/notes/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:elif",
  "issuedAt": "2026-11-04T06:00:00Z",
  "threadId": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "payload": {
    "notes": [
      {
        "noteId": "bnt_Xa4Fh7Ty2Gp9Lm6C",
        "boardId": "brd_Uc7N2qLm4Rt8Vx1Z",
        "ideaId": "ida_Pk3Wq9Ls2Dn6Hy4B",
        "author": {
          "entryId": "dir_Hm2Vt6Xs9Qe4Jn1F",
          "name": "Tomás Brandt"
        },
        "text": "Happy to co-host the screen-reader half.",
        "createdAt": "2026-11-04T05:00:00Z",
        "state": "visible"
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

Out: notes and their authors' directory names as the caller may see them.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by, and a pairwise identifier per caller would make a board's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. Authors are named, by design (VTI-BRD-070).

### Retention

Nothing is kept.

### Consent/purpose

A member reads what the board's audience was told it would see.
