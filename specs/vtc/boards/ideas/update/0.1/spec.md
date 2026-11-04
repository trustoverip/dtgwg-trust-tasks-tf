---
slug: vtc/boards/ideas/update
version: '0.1'
title: "VTC Boards — Ideas — Update"
summary: "A proposer changes their idea's title, description, format or topic while it is still proposed."
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
  rationale: "What an idea says is attributed to its proposer; a change must be theirs (VTI-BRD-050)."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed update could undo a later one."
sideEffects:
  level: mutating
  rationale: "Changes the idea; the proposer can change it again while it is proposed."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: "In: the changed text. Out: the idea."
retention:
  class: durable
  rationale: "The idea as it now reads is the board's record."
errorCodes:
- code: vtc/boards/ideas/update:notFound
  meaning: "No board the caller may see has this identifier (VTI-BRD-003)."
  retryable: false
- code: vtc/boards/ideas/update:ideaNotFound
  meaning: "No idea on this board the caller may see has this identifier."
  retryable: false
- code: vtc/boards/ideas/update:notProposer
  meaning: "The caller did not propose this idea."
  retryable: false
- code: vtc/boards/ideas/update:notProposed
  meaning: "The idea is no longer `proposed`; once an election is opened its title and description are its manifest's (VTI-BRD-012)."
  retryable: false
- code: vtc/boards/ideas/update:topicUnknown
  meaning: "The board states topics and `topic` is not one of them."
  retryable: false
- code: vtc/boards/ideas/update:formatUnknown
  meaning: "The board states formats and `format` is not one of them."
  retryable: false
related:
- vtc/boards/ideas/propose
---
## Abstract

The **VTC Boards — Ideas — Update** Trust Task lets an idea's proposer change what it says while it is `proposed`. Every member it carries replaces the idea's current value. After an election is opened for the idea, its title and description are those its manifest carries and cannot change (VTI-BRD-012, VTI-VOT-013).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the Community Boards chapter of the Verifiable Trust Infrastructure specification (VTI-BRD-001 – VTI-BRD-080). The design is recorded in the VTC boards design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-boards.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` and `ideaNotFound` as `vtc/boards/show` would; refuse anyone but the idea's proposer with `notProposer`, and an idea not `proposed` with `notProposed` (VTI-BRD-012). An idea being opened (`opening`) is not `proposed` for this item.
2. Check `topic` and `format` as `vtc/boards/ideas/propose` does, store the change, and answer the idea.
3. Accept the document signed as `vtc/boards/ideas/propose` item 4 says.

## Authorization

**Authority: being the idea's proposer.** The proof establishes who signed.

## Definitions

- `Idea` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `IdeaDescription` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `IdeaFormat` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `IdeaId` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `IdeaTitle` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `Topic` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Retitle an idea

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "type": "https://trusttasks.org/spec/vtc/boards/ideas/update/0.1#request",
  "issuer": "did:example:elif",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-04T06:00:00Z",
  "payload": {
    "ideaId": "ida_Pk3Wq9Ls2Dn6Hy4B",
    "title": "Wallet sign-in for everyone"
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

On success the community answers with a document of type `https://trusttasks.org/spec/vtc/boards/ideas/update/0.1#response` whose `payload` validates against `$defs.Response`. A refusal is a `trust-task-error` document carrying one of the codes above or a framework standard code.

### The idea as changed

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000002",
  "type": "https://trusttasks.org/spec/vtc/boards/ideas/update/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:elif",
  "issuedAt": "2026-11-04T06:00:00Z",
  "threadId": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "payload": {
    "idea": {
      "ideaId": "ida_Pk3Wq9Ls2Dn6Hy4B",
      "boardId": "brd_Uc7N2qLm4Rt8Vx1Z",
      "state": "proposed",
      "title": "Wallet sign-in for everyone",
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

In: the changed text. Out: the idea.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by, and a pairwise identifier per caller would make a board's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. As `vtc/boards/ideas/propose`.

### Retention

The idea's current text is kept; earlier text is not.

### Consent/purpose

The proposer changes what they said.
