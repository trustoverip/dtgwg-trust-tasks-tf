---
slug: vtc/boards/ideas/withdraw
version: '0.1'
title: "VTC Boards — Ideas — Withdraw"
summary: "A proposer withdraws their idea while it is still proposed."
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
  rationale: "Withdrawing an idea is the proposer's own decision and must be attributable to them (VTI-BRD-050)."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor."
sideEffects:
  level: mutating
  rationale: "Withdraws the idea for good (VTI-BRD-011); its notes stay with it."
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: true
  rationale: "In: the idea's identifier. Out: the idea, withdrawn."
retention:
  class: durable
  rationale: "The withdrawal is why the idea left the board."
errorCodes:
- code: vtc/boards/ideas/withdraw:notFound
  meaning: "No board the caller may see has this identifier (VTI-BRD-003)."
  retryable: false
- code: vtc/boards/ideas/withdraw:ideaNotFound
  meaning: "No idea on this board the caller may see has this identifier."
  retryable: false
- code: vtc/boards/ideas/withdraw:notProposer
  meaning: "The caller did not propose this idea."
  retryable: false
- code: vtc/boards/ideas/withdraw:notProposed
  meaning: "The idea is no longer `proposed`, or is being opened (VTI-BRD-012)."
  retryable: false
related:
- vtc/boards/ideas/propose
---
## Abstract

The **VTC Boards — Ideas — Withdraw** Trust Task lets an idea's proposer take it back while it is `proposed` (VTI-BRD-012). A withdrawn idea is shown only to its proposer and the board's facilitators, and stops counting against the proposer's limit.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the Community Boards chapter of the Verifiable Trust Infrastructure specification (VTI-BRD-001 – VTI-BRD-080). The design is recorded in the VTC boards design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-boards.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` and `ideaNotFound` as `vtc/boards/show` would; refuse anyone but the proposer (`notProposer`) and an idea not `proposed` or being opened (`notProposed`).
2. Move the idea to `withdrawn`, which is final (VTI-BRD-011), and answer it.

## Authorization

**Authority: being the idea's proposer.** The proof establishes who signed.

## Definitions

- `Idea` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `IdeaId` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Withdraw an idea

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "type": "https://trusttasks.org/spec/vtc/boards/ideas/withdraw/0.1#request",
  "issuer": "did:example:elif",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-04T06:00:00Z",
  "payload": {
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

On success the community answers with a document of type `https://trusttasks.org/spec/vtc/boards/ideas/withdraw/0.1#response` whose `payload` validates against `$defs.Response`. A refusal is a `trust-task-error` document carrying one of the codes above or a framework standard code.

### The idea, withdrawn

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000002",
  "type": "https://trusttasks.org/spec/vtc/boards/ideas/withdraw/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:elif",
  "issuedAt": "2026-11-04T06:00:00Z",
  "threadId": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "payload": {
    "idea": {
      "ideaId": "ida_Pk3Wq9Ls2Dn6Hy4B",
      "boardId": "brd_Uc7N2qLm4Rt8Vx1Z",
      "state": "withdrawn",
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

In: an idea's identifier. Out: the idea.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by, and a pairwise identifier per caller would make a board's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. None beyond the idea's own.

### Retention

The idea is kept, withdrawn, with the board.

### Consent/purpose

The proposer decides.
