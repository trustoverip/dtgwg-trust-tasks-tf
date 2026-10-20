---
slug: voting/trustees/confirm
version: "0.1"
title: "Voting — Trustees — Confirm"
summary: "A trustee confirms the shares it was dealt, or complains of the dealers whose shares failed their check."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - voting
  - trustee
  - threshold
  - key-generation
parties:
  - role: Trustee
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: Ballot box (voting service)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    A complaint can disqualify a dealer. It must be attributable to the trustee who made it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3, so SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: mutating
  rationale: >-
    Appends a `confirmation` entry; the election key forms once every trustee has confirmed and every complaint is answered.
subjectPath: /community
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: which dealers' shares failed. Out: the summary and phase.
retention:
  class: durable
  rationale: >-
    On the board for as long as the board.
errorCodes:
  - code: voting/trustees/confirm:electionUnknown
    meaning: "No election with this identifier exists for this community."
    retryable: false
  - code: voting/trustees/confirm:notConfirming
    meaning: "The ceremony is not confirming."
    retryable: false
  - code: voting/trustees/confirm:alreadyConfirmed
    meaning: "This trustee has already confirmed otherwise."
    retryable: false
related:
  - voting/trustees/deal
  - voting/trustees/justify
  - vta/pcs/trustee/keys
---

## Abstract

The **Voting — Trustees — Confirm** Trust Task is step 3 of generating an election key (CONVENTIONS §9.6): each trustee opens the shares it was dealt, checks each against its dealer's commitments, and confirms — naming any dealer whose share failed. The family's shared rules are in [`CONVENTIONS.md`](../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../voting/_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the trustee, through its client) **MUST**:

1. Sign the request as the trustee's DID, with the complaints its VTA found ([`vta/pcs/trustee/keys`](../../../../vta/pcs/trustee/keys/0.1/spec.md)).

A conforming **consumer** (the ballot box) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` unless the proven signer is the manifest's `trustees.dids` entry at the payload's trustee index (VTI-VOT-062).
3. Refuse with `voting/trustees/confirm:electionUnknown` for an election not registered for `community`.
4. Refuse with `voting/trustees/confirm:notConfirming` unless the ceremony is confirming.
5. Answer a repeat of the same confirmation as the first; refuse another with `voting/trustees/confirm:alreadyConfirmed`.
6. Refuse with `malformedRequest` a complaint naming a trustee index the manifest does not have, or the trustee itself.
7. Append a `confirmation` entry. When every trustee has confirmed and every complaint has a justification that verifies, form the election key (CONVENTIONS §9.6 item 5).

## Authorization

The authority is **being the trustee the manifest names at that index**.

## Definitions

- **`trustee`** — the confirming trustee's index.
- **`complaints`** — the dealers whose share failed.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "type": "https://trusttasks.org/spec/voting/trustees/confirm/0.1#request",
  "issuer": "did:webvh:QmTrustee:alice.example",
  "recipient": "did:webvh:QmBallotBox:vote.example.net",
  "issuedAt": "2026-11-02T09:00:00Z",
  "payload": {
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026",
    "trustee": 2,
    "complaints": []
  }
}
```

## Response

The answer is the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1002",
  "type": "https://trusttasks.org/spec/voting/trustees/confirm/0.1#response",
  "issuer": "did:webvh:QmBallotBox:vote.example.net",
  "recipient": "did:webvh:QmTrustee:alice.example",
  "issuedAt": "2026-11-02T09:00:01Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "payload": {
    "election": {
      "electionId": "board-2026",
      "community": "did:webvh:QmCommunity:vtc.example.org",
      "manifestDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
      "state": "registration",
      "ballotsAdmitted": 0,
      "head": {
        "electionId": "board-2026",
        "treeSize": 3,
        "rootHash": "zQmPXME1oRtoT627YKaDPDQ3PwA8tdP9rWuAAweLzqSwAWT",
        "issuedAt": "2026-11-02T09:00:01Z"
      }
    },
    "phase": "dealing"
  }
}
```

## Security & Privacy

### Data carried

Trustee indices only.

### Correlation

The trustee's and the ballot box's identifier scope is `public`. Trustees are named in the manifest and public by design (VTI-VOT-090): anyone recounting the election must be able to tell which trustee published what, which a pairwise identifier would prevent. The ballot box's DID is public for the same reason — the community and every voter must reach the same box, and its signed heads are evidence only under a DID everyone knows. Nothing here concerns a voter.

### Retention

Kept on the board for as long as the board is (VTI-VOT-070): the result is recomputed from it.

### Consent/purpose

Collected to form the election key or decrypt its result, and for nothing else.
