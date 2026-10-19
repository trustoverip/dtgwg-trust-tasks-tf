---
slug: voting/board/head
version: "0.1"
title: "Voting — Board Head"
summary: "Reads an election board's current signed tree head, the commitment every inclusion and consistency proof is checked against."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - voting
  - board
  - tree-head
  - transparency-log
  - merkle
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
  response: REQUIRED
  rationale: >-
    The response's proof is the head's signature (VTI-VOT-050): a head is a commitment only
    because the ballot box signed it, and a reader who later finds the board does not extend it
    holds that signed document as proof the ballot box rewrote its board. On a private board the
    ballot box needs to know who asks, which a request proof gives it where the transport does
    not.
sideEffects:
  level: none
  rationale: >-
    A read. A ballot box may sign a fresh head to answer it, which changes no state a reader can
    observe beyond the head's own `issuedAt`.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Out: the board's size and root hash. Nothing about any voter.
retention:
  class: transient
  rationale: >-
    The ballot box keeps nothing of a read. The reader keeps the signed response for as long as
    it wants to hold the ballot box to it.
errorCodes:
  - code: voting/board/head:electionUnknown
    meaning: "No election with this identifier exists for this community, or the caller may not see it."
    retryable: false
related:
  - voting/board/read
  - voting/board/inclusion
  - voting/board/consistency
  - voting/ballot/cast
  - vtc/elections/certify
---

## Abstract

The **Voting — Board Head** Trust Task reads an election board's current tree head: the number of entries on the board and the RFC 9162 Merkle tree hash over them, signed by the ballot box. A head is what everything else is checked against — a voter's receipt by an inclusion proof, an auditor's earlier head by a consistency proof, the result by recomputing it from the entries the head covers. Readers who compare the heads they were shown detect a ballot box that shows different boards to different people. The family's shared rules are in [`CONVENTIONS.md`](../../../_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **consumer** (the ballot box) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `voting/board/head:electionUnknown` when no election `electionId` is registered for `community`, and — in the same form — on a private board when the proven caller is neither the community nor a DID in the manifest's `board.auditors` (VTI-VOT-052).
3. Answer with the head of the board as it stands: `treeSize` the number of entries, `rootHash` the RFC 9162 Merkle tree hash (SHA-256) over them ([conventions §3.8](../../../_shared/0.1/CONVENTIONS.md#3-cryptographic-binding)), `issuedAt` the time the head was signed.
4. Sign the response (VTI-VOT-050). The proof over the response document is the head's signature; a ballot box **MUST NOT** sign two heads of one board with the same `treeSize` and different `rootHash`, and **MUST NOT** sign a head that is not a consistent extension of every head it signed before (VTI-VOT-051).

A conforming **producer** that relies on a head **MUST** keep the whole signed response document, not the `head` object alone: the head is signed only as part of that document, and the document is what proves the ballot box committed to it (VTI-VOT-051). A producer holding an earlier head **SHOULD** ask for [`voting/board/consistency`](../../consistency/0.1/spec.md) between the two before relying on the newer one.

## Authorization

**Public board: none** — anyone may read a public board's head (VTI-VOT-052). **Private board:** the community and the manifest's auditors ([conventions §4](../../../_shared/0.1/CONVENTIONS.md#4-authorization)), enforced by consumer item 2. A voter on a private board holds the head in its receipt and later heads from its community, which publishes the heads it has verified. Verifying the caller's `issuer`, transport identity or `proof` establishes who asked, never that they may ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)).

## Definitions

- **`community`**, **`electionId`** — the board to read.
- **`head`** (response) — the board's current head (`BoardHead`).

## Request

A reader sends the request to the ballot box. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

An auditor named in a private board's manifest asks for the head.

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-00000000f001",
  "type": "https://trusttasks.org/spec/voting/board/head/0.1#request",
  "issuer": "did:webvh:QmAuditor:audit.example.com",
  "recipient": "did:webvh:QmVote:vote.example.net",
  "issuedAt": "2026-11-14T06:00:00Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-00000000f0ff",
  "payload": {
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-14T06:00:00Z",
    "verificationMethod": "did:webvh:QmAuditor:audit.example.com#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z3sN6yPqExampleAuditorSignatureOverTheRead"
  }
}
```

## Response

The ballot box answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json), signed. Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-00000000f002",
  "type": "https://trusttasks.org/spec/voting/board/head/0.1#response",
  "issuer": "did:webvh:QmVote:vote.example.net",
  "recipient": "did:webvh:QmAuditor:audit.example.com",
  "issuedAt": "2026-11-14T06:00:01Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-00000000f0ff",
  "payload": {
    "head": {
      "electionId": "board-2026",
      "treeSize": 212,
      "rootHash": "zQmPZ9gcCEpqKTo6aq61g2nXGUhM4iCL3ewB6LDXZCtioEB",
      "issuedAt": "2026-11-14T06:00:01Z"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-14T06:00:01Z",
    "verificationMethod": "did:webvh:QmVote:vote.example.net#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z5aB8kRmExampleBallotBoxSignatureOverTheHead"
  }
}
```

## Security & Privacy

### Data carried

A size and a hash. Nothing about any voter.

### Correlation

A reader of a public board need not identify itself. The ballot box's identifier scope is `public`, because a head is worth something only if every reader can attribute it to the one ballot box the manifest names. A ballot box that answers different readers with heads that are not consistent with each other is equivocating; readers — and the community, which publishes the heads it has checked — detect it by comparing what they hold. Frequent reads during voting reveal when the board grows, which is no more than its public entries already show.

### Retention

Transient at the ballot box. A reader keeps the signed response for as long as it wants to be able to show what the ballot box committed to; a community keeps the heads it publishes for as long as the election's record.

### Consent/purpose

To let anyone allowed to read the board hold the ballot box to one board. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
