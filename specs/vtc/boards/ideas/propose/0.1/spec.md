---
slug: vtc/boards/ideas/propose
version: '0.1'
title: "VTC Boards — Ideas — Propose"
summary: "A member proposes an idea on a board, under their own name, while the board takes proposals."
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
  rationale: "An idea is shown under its proposer's name; it must stay attributable to them after the transport has closed (VTI-BRD-050)."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed proposal would make a second idea."
sideEffects:
  level: mutating
  rationale: "Adds an idea. Its proposer can withdraw it while it is proposed."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: "In: the idea. Out: the idea as stored. The member acts in their own name."
retention:
  class: durable
  rationale: "The idea is a board record; once accepted its text is also its election's manifest."
errorCodes:
- code: vtc/boards/ideas/propose:notFound
  meaning: "No board the caller may see has this identifier (VTI-BRD-003)."
  retryable: false
- code: vtc/boards/ideas/propose:proposalsClosed
  meaning: "The board is not taking proposals: outside its proposal window, or closed (VTI-BRD-010)."
  retryable: false
- code: vtc/boards/ideas/propose:limitReached
  meaning: "The caller already has the board's `maxIdeasPerMember` ideas that are not withdrawn or declined."
  retryable: false
- code: vtc/boards/ideas/propose:topicUnknown
  meaning: "The board states topics and `topic` is not one of them, or omits `topic`."
  retryable: false
- code: vtc/boards/ideas/propose:formatUnknown
  meaning: "The board states formats and `format` is not one of them, or omits `format`."
  retryable: false
related:
- vtc/boards/ideas/update
- vtc/boards/ideas/withdraw
- vtc/boards/ideas/decide
---
## Abstract

The **VTC Boards — Propose** Trust Task adds an idea to a board. The idea is shown under its proposer's name to the board's audience, and waits, `proposed`, for a facilitator to accept, decline or merge it.

A member portal holding a key the member authorized to sign `vtc/boards` documents for them may send it (VTI-BRD-050).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the Community Boards chapter of the Verifiable Trust Infrastructure specification (VTI-BRD-001 – VTI-BRD-080). The design is recorded in the VTC boards design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-boards.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` for a board the caller may not see; refuse a caller the board's audience does not admit — a facilitator who is not in it included — with `permissionDenied` (VTI-BRD-010).
2. Refuse outside the proposal window or on a closed or archived board (`proposalsClosed`), beyond the caller's limit (`limitReached`), and a topic or format the board does not state where it states them (`topicUnknown`, `formatUnknown`).
3. Store the idea `proposed` against the caller's directory entry, never their DID, with a fresh `ideaId`, and answer it as `vtc/boards/show` would show it to them.
4. Accept the document signed by the member's own key or by a portal key they authorized for `vtc/boards` for the live session (VTI-BRD-050).

## Authorization

**Authority: being in the board's audience.** The proof establishes who signed: the member, or a portal key the member authorized for `vtc/boards` documents for the session.

## Definitions

- `BoardId` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `Idea` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `IdeaDescription` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `IdeaFormat` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `IdeaTitle` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `Topic` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Propose a session

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "type": "https://trusttasks.org/spec/vtc/boards/ideas/propose/0.1#request",
  "issuer": "did:example:elif",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-04T06:00:00Z",
  "payload": {
    "boardId": "brd_Uc7N2qLm4Rt8Vx1Z",
    "title": "Accessible wallet sign-in",
    "description": "The on-screen number, screen readers, and people who can't scan a code. List the barriers and pick three to fix.",
    "format": "Discussion",
    "topic": "Identity"
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

On success the community answers with a document of type `https://trusttasks.org/spec/vtc/boards/ideas/propose/0.1#response` whose `payload` validates against `$defs.Response`. A refusal is a `trust-task-error` document carrying one of the codes above or a framework standard code.

### The idea, proposed

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000002",
  "type": "https://trusttasks.org/spec/vtc/boards/ideas/propose/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:elif",
  "issuedAt": "2026-11-04T06:00:00Z",
  "threadId": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "payload": {
    "idea": {
      "ideaId": "ida_Pk3Wq9Ls2Dn6Hy4B",
      "boardId": "brd_Uc7N2qLm4Rt8Vx1Z",
      "state": "proposed",
      "title": "Accessible wallet sign-in",
      "description": "The on-screen number, screen readers, and people who can't scan a code. List the barriers and pick three to fix.",
      "format": "Discussion",
      "topic": "Identity",
      "proposer": {
        "entryId": "dir_Rb8Nc3Kq7Wz2Lp5D",
        "name": "Elif Polat",
        "you": true
      },
      "proposedAt": "2026-11-04T02:10:00Z",
      "noteCount": 0
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

In: the idea's title, description, format and topic, written by the member. Out: the idea, with the member's directory name.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by, and a pairwise identifier per caller would make a board's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The idea is attributed to the member's directory entry, which survives a rotation of their DID, and is shown under their name to the board's audience (VTI-BRD-070).

### Retention

Kept with the board. On departure a proposed idea is withdrawn; on erasure its text is removed unless an election was opened for it (VTI-BRD-060).

### Consent/purpose

The member chose to say this to the board's audience, having been told so before they first wrote (VTI-BRD-070).
