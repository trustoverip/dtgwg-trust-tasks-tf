---
slug: vtc/boards/notes/add
version: '0.1'
title: "VTC Boards — Notes — Add"
summary: "A member attaches a short note, under their own name, to a board or to one idea on it."
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords: [vtc, boards, unconference, ideas, elections]
parties:
- role: member
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "A note is shown under its author's name; it must stay attributable to them after the transport has closed (VTI-BRD-050)."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed note would appear twice."
sideEffects:
  level: mutating
  rationale: "Adds a note. Its author can withdraw it; nobody can change its text (VTI-BRD-021)."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: "In: the note. Out: the note as stored."
retention:
  class: durable
  rationale: "The note is a board record."
errorCodes:
- code: vtc/boards/notes/add:notFound
  meaning: "No board the caller may see has this identifier (VTI-BRD-003)."
  retryable: false
- code: vtc/boards/notes/add:ideaNotFound
  meaning: "No idea on this board the caller may see has this identifier."
  retryable: false
- code: vtc/boards/notes/add:boardClosed
  meaning: "The board is closed or archived (VTI-BRD-005)."
  retryable: false
- code: vtc/boards/notes/add:ideaWithdrawn
  meaning: "The idea is withdrawn (VTI-BRD-020)."
  retryable: false
- code: vtc/boards/notes/add:rateLimited
  meaning: "The caller added too many notes too recently; retry later."
  retryable: true
related:
- vtc/boards/notes/list
- vtc/boards/notes/withdraw
- vtc/boards/notes/hide
---
## Abstract

The **VTC Boards — Notes — Add** Trust Task attaches a note to a board, or to one idea on it when `ideaId` is given. A note is shown under its author's name to the board's audience, and is never carried into any election (VTI-BRD-022).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the Community Boards chapter of the Verifiable Trust Infrastructure specification (VTI-BRD-001 – VTI-BRD-080). The design is recorded in the VTC boards design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-boards.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` and `ideaNotFound` as `vtc/boards/show` would; refuse a caller the board's audience does not admit with `permissionDenied`.
2. Refuse on a closed or archived board (`boardClosed`) and on a withdrawn idea (`ideaWithdrawn`); bound the rate at which one caller adds notes (`rateLimited`, VTI-BRD-020).
3. Store the note `visible` against the caller's directory entry, with a fresh `noteId`, and answer it.
4. **MUST NOT** carry a note's text into any election, manifest, ballot or ballot box (VTI-BRD-022).
5. Accept the document signed as `vtc/boards/ideas/propose` item 4 says.

## Authorization

**Authority: being in the board's audience.** The proof establishes who signed: the member, or a portal key the member authorized for `vtc/boards` for the session.

## Definitions

- `BoardId` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `IdeaId` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `Note` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `NoteText` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A note on an idea

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "type": "https://trusttasks.org/spec/vtc/boards/notes/add/0.1#request",
  "issuer": "did:example:elif",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-04T06:00:00Z",
  "payload": {
    "boardId": "brd_Uc7N2qLm4Rt8Vx1Z",
    "ideaId": "ida_Pk3Wq9Ls2Dn6Hy4B",
    "text": "Happy to co-host the screen-reader half."
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

On success the community answers with a document of type `https://trusttasks.org/spec/vtc/boards/notes/add/0.1#response` whose `payload` validates against `$defs.Response`. A refusal is a `trust-task-error` document carrying one of the codes above or a framework standard code.

### The note

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000002",
  "type": "https://trusttasks.org/spec/vtc/boards/notes/add/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:elif",
  "issuedAt": "2026-11-04T06:00:00Z",
  "threadId": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "payload": {
    "note": {
      "noteId": "bnt_Xa4Fh7Ty2Gp9Lm6C",
      "boardId": "brd_Uc7N2qLm4Rt8Vx1Z",
      "ideaId": "ida_Pk3Wq9Ls2Dn6Hy4B",
      "author": {
        "entryId": "dir_Hm2Vt6Xs9Qe4Jn1F",
        "name": "Tomás Brandt",
        "you": true
      },
      "text": "Happy to co-host the screen-reader half.",
      "createdAt": "2026-11-04T05:00:00Z",
      "state": "visible"
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

In: the note's text. Out: the note with its author's directory name.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by, and a pairwise identifier per caller would make a board's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The note is attributed to the author's directory entry and shown under their name to the board's audience (VTI-BRD-070).

### Retention

Kept with the board; on erasure its text is removed (VTI-BRD-060).

### Consent/purpose

The member chose to say this to the board's audience.
