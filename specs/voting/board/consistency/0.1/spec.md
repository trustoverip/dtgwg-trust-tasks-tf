---
slug: voting/board/consistency
version: "0.1"
title: "Voting — Board Consistency"
summary: "Proves that an election board as one signed head commits to it is a prefix of the board as a later head commits to it — that the ballot box only appended."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - voting
  - board
  - consistency-proof
  - merkle
  - transparency-log
  - audit
parties:
  - role: Reader (anyone on a public board; the community or an auditor on a private one)
    requirement: OPTIONAL
    member: issuer
    identifierScope: any
  - role: Ballot box (voting service)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  request: RECOMMENDED
  response: RECOMMENDED
  rationale: >-
    On a private board the ballot box needs to know who asks, which a request proof gives it
    where the transport does not. The response needs no proof to be trusted: the path either
    reconciles two heads the reader holds signed, or it does not.
sideEffects:
  level: none
  rationale: >-
    A read. Nothing at the ballot box changes.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Out: hashes. No entry content and nothing about any voter.
retention:
  class: transient
  rationale: >-
    The ballot box keeps nothing of a read beyond its ordinary request log.
errorCodes:
  - code: voting/board/consistency:electionUnknown
    meaning: "No election with this identifier exists for this community, or the caller may not see it."
    retryable: false
  - code: voting/board/consistency:outOfRange
    meaning: "`secondSize` is greater than the board's current size, or `firstSize` is greater than `secondSize`."
    retryable: false
related:
  - voting/board/head
  - voting/board/inclusion
  - voting/board/read
  - vtc/elections/certify
---

## Abstract

The **Voting — Board Consistency** Trust Task returns an RFC 9162 §2.1.4 consistency proof between two sizes of an election's board. A reader holding two signed heads — one from earlier in the election, one from now — checks with it that the later board contains the earlier one unchanged: nothing removed, nothing reordered, nothing rewritten. It is what turns a sequence of signed heads into an append-only record, and how the community checks the board between the checkpoints it publishes. The family's shared rules are in [`CONVENTIONS.md`](../../../_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **consumer** (the ballot box) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `voting/board/consistency:electionUnknown` when no election `electionId` is registered for `community`, and — in the same form — on a private board when the proven caller is neither the community nor a DID in the manifest's `board.auditors` (VTI-VOT-052).
3. Refuse with `voting/board/consistency:outOfRange` when `secondSize` exceeds the board's current size or `firstSize` exceeds `secondSize`.
4. Answer with `firstSize`, `secondSize` and, as `path`, the RFC 9162 §2.1.4 consistency proof between the tree of the first `firstSize` entries and the tree of the first `secondSize` entries ([conventions §3.8](../../../_shared/0.1/CONVENTIONS.md#3-cryptographic-binding)) — empty when the two are equal.

A conforming **producer** that relies on the answer **MUST** verify `path` against the root hashes of two signed heads it holds, of sizes `firstSize` and `secondSize`. A path that does not verify, or a ballot box that cannot produce one between two heads it signed, is evidence that the ballot box did not keep the board append-only (VTI-VOT-051); the producer keeps both signed head documents as that evidence. A first size of zero needs no proof — every tree extends the empty one — and is outside the schema.

## Authorization

**Public board: none** (VTI-VOT-052). **Private board:** the community and the manifest's auditors ([conventions §4](../../../_shared/0.1/CONVENTIONS.md#4-authorization)), enforced by consumer item 2. Verifying the caller's `issuer`, transport identity or `proof` establishes who asked, never that they may ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)).

## Definitions

- **`community`**, **`electionId`** — the board.
- **`firstSize`**, **`secondSize`** — the sizes of the earlier and the later head.
- **`path`** (response) — the consistency proof (`MerklePath`).

## Request

A reader sends the request to the ballot box. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

The community checks the board between its last checkpoint and the current head.

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-000000012001",
  "type": "https://trusttasks.org/spec/voting/board/consistency/0.1#request",
  "issuer": "did:webvh:QmCommunity:vtc.example.org",
  "recipient": "did:webvh:QmVote:vote.example.net",
  "issuedAt": "2026-11-14T06:05:00Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-0000000120ff",
  "payload": {
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026",
    "firstSize": 129,
    "secondSize": 212
  }
}
```

## Response

The ballot box answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-000000012002",
  "type": "https://trusttasks.org/spec/voting/board/consistency/0.1#response",
  "issuer": "did:webvh:QmVote:vote.example.net",
  "recipient": "did:webvh:QmCommunity:vtc.example.org",
  "issuedAt": "2026-11-14T06:05:01Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-0000000120ff",
  "payload": {
    "firstSize": 129,
    "secondSize": 212,
    "path": [
      "zQmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG",
      "zQmT5NvUtoM5nWFfrQdVrFtvGfKFmG7AHE8P34isapyhCxX",
      "zQmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n"
    ]
  }
}
```

## Security & Privacy

### Data carried

Two sizes in; hashes out. No entry content and nothing about any voter.

### Correlation

None about voters. A reader of a public board need not identify itself. The ballot box's identifier scope is `public`, because the heads a consistency proof reconciles are worth something only when attributed to the one ballot box the manifest names.

### Retention

Transient at the ballot box. A reader that finds an inconsistency keeps the two signed heads and the failed answer as evidence for as long as it may need to show it.

### Consent/purpose

To hold the ballot box to an append-only board. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
