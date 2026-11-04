---
slug: vtc/boards/notes/withdraw
version: '0.1'
title: "VTC Boards — Notes — Withdraw"
summary: "A note's author withdraws it; its text is no longer shown to the board's audience."
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
  rationale: "Withdrawing a note is its author's decision (VTI-BRD-050)."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor."
sideEffects:
  level: mutating
  rationale: "Withdraws the note for good."
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: true
  rationale: "In: the note's identifier. Out: the note, withdrawn."
retention:
  class: durable
  rationale: "The withdrawal is why the note's text is no longer shown."
errorCodes:
- code: vtc/boards/notes/withdraw:noteNotFound
  meaning: "No note the caller may see has this identifier."
  retryable: false
- code: vtc/boards/notes/withdraw:notAuthor
  meaning: "The caller did not write this note."
  retryable: false
- code: vtc/boards/notes/withdraw:alreadyWithdrawn
  meaning: "The note is already withdrawn."
  retryable: false
related:
- vtc/boards/notes/add
---
## Abstract

The **VTC Boards — Notes — Withdraw** Trust Task lets a note's author take it back. The board's audience then sees only that a note was withdrawn (VTI-BRD-021). It is allowed on a closed board too.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the Community Boards chapter of the Verifiable Trust Infrastructure specification (VTI-BRD-001 – VTI-BRD-080). The design is recorded in the VTC boards design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-boards.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `noteNotFound` for a note the caller may not see; refuse anyone but its author (`notAuthor`) and a note already withdrawn (`alreadyWithdrawn`). A hidden note may still be withdrawn by its author.
2. Move the note to `withdrawn` and answer it.

## Authorization

**Authority: being the note's author.**

## Definitions

- `Note` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `NoteId` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Withdraw a note

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "type": "https://trusttasks.org/spec/vtc/boards/notes/withdraw/0.1#request",
  "issuer": "did:example:elif",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-04T06:00:00Z",
  "payload": {
    "noteId": "bnt_Xa4Fh7Ty2Gp9Lm6C"
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

On success the community answers with a document of type `https://trusttasks.org/spec/vtc/boards/notes/withdraw/0.1#response` whose `payload` validates against `$defs.Response`. A refusal is a `trust-task-error` document carrying one of the codes above or a framework standard code.

### The note, withdrawn

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000002",
  "type": "https://trusttasks.org/spec/vtc/boards/notes/withdraw/0.1#response",
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
      "state": "withdrawn"
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

In: a note's identifier. Out: the note.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by, and a pairwise identifier per caller would make a board's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. None beyond the note's own.

### Retention

The note is kept, withdrawn; on erasure its text is removed (VTI-BRD-060).

### Consent/purpose

The author decides.
