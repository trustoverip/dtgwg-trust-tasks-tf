---
slug: voting/trustees/justify
version: "0.1"
title: "Voting — Trustees — Justify"
summary: "A dealer answers a complaint by publishing the share it dealt to the complaining trustee, for anyone to check."
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
    Only the dealer may reveal its own share; the reveal must be attributable to it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3, so SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: mutating
  rationale: >-
    Appends a `justification` entry; a share that checks voids the complaint.
subjectPath: /community
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: one share in the clear — it is one of n, so it reveals nothing about the key alone. Out: the summary and phase.
retention:
  class: durable
  rationale: >-
    On the board for as long as the board.
errorCodes:
  - code: voting/trustees/justify:electionUnknown
    meaning: "No election with this identifier exists for this community."
    retryable: false
  - code: voting/trustees/justify:notConfirming
    meaning: "The ceremony is not confirming."
    retryable: false
  - code: voting/trustees/justify:noComplaint
    meaning: "The recipient has not complained of this dealer."
    retryable: false
  - code: voting/trustees/justify:shareInvalid
    meaning: "The share does not match the dealer's commitments: the complaint stands."
    retryable: false
related:
  - voting/trustees/confirm
  - vta/pcs/trustee/justify
---

## Abstract

The **Voting — Trustees — Justify** Trust Task is step 4 of generating an election key (CONVENTIONS §9.6): a dealer complained of publishes the share in the clear, and anyone checks it against the dealer's commitments. A share that checks voids the complaint, and the complaining trustee uses the revealed value. The family's shared rules are in [`CONVENTIONS.md`](../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../voting/_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the trustee, through its client) **MUST**:

1. Sign the request as the dealer's DID, with the share its VTA revealed ([`vta/pcs/trustee/justify`](../../../../vta/pcs/trustee/justify/0.1/spec.md)).

A conforming **consumer** (the ballot box) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` unless the proven signer is the manifest's `trustees.dids` entry at the payload's trustee index (VTI-VOT-062). The index is `dealer`.
3. Refuse with `voting/trustees/justify:electionUnknown` for an election not registered for `community`.
4. Refuse with `voting/trustees/justify:notConfirming` unless the ceremony is confirming.
5. Refuse with `voting/trustees/justify:noComplaint` unless `recipient` complained of `dealer`.
6. Refuse with `voting/trustees/justify:shareInvalid` unless `share·G = Σ_k recipient^k·C_dealer,k`.
7. Append a `justification` entry and void the complaint; form the election key if the ceremony is now complete.

## Authorization

The authority is **being the dealer complained of**.

## Definitions

- **`dealer`** — the dealer answering.
- **`recipient`** — the trustee that complained.
- **`share`** — the share in the clear.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "type": "https://trusttasks.org/spec/voting/trustees/justify/0.1#request",
  "issuer": "did:webvh:QmTrustee:alice.example",
  "recipient": "did:webvh:QmBallotBox:vote.example.net",
  "issuedAt": "2026-11-02T09:00:00Z",
  "payload": {
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026",
    "dealer": 1,
    "recipient": 2,
    "share": "z3Hs8WkqL2mNvB7cR9xTyE4uJpQ1aZf6gDoK5iMt8n"
  }
}
```

## Response

The answer is the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1002",
  "type": "https://trusttasks.org/spec/voting/trustees/justify/0.1#response",
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

One share in the clear: below the threshold it reveals nothing about the election key.

### Correlation

The trustee's and the ballot box's identifier scope is `public`. Trustees are named in the manifest and public by design (VTI-VOT-090): anyone recounting the election must be able to tell which trustee published what, which a pairwise identifier would prevent. The ballot box's DID is public for the same reason — the community and every voter must reach the same box, and its signed heads are evidence only under a DID everyone knows. Nothing here concerns a voter.

### Retention

Kept on the board for as long as the board is (VTI-VOT-070): the result is recomputed from it.

### Consent/purpose

Collected to form the election key or decrypt its result, and for nothing else.
