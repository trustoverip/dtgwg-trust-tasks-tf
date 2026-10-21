---
slug: voting/trustees/deal
version: "0.1"
title: "Voting — Trustees — Deal"
summary: "A trustee publishes its contribution to the election key: commitments to its polynomial and every trustee's share sealed to that trustee."
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
    A dealing fixes part of the election key. A recount must be able to show which trustee dealt it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3, so SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: mutating
  rationale: >-
    Appends a `trusteeStep` entry carrying the request as received; confirming opens when every trustee has dealt.
subjectPath: /community
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: commitments, a proof, and shares each sealed to its recipient. Out: the summary and phase.
retention:
  class: durable
  rationale: >-
    On the board for as long as the board.
errorCodes:
  - code: voting/trustees/deal:electionUnknown
    meaning: "No election with this identifier exists for this community."
    retryable: false
  - code: voting/trustees/deal:notDealing
    meaning: "The ceremony is not dealing."
    retryable: false
  - code: voting/trustees/deal:alreadyDealt
    meaning: "This trustee has already published another dealing."
    retryable: false
  - code: voting/trustees/deal:dealingInvalid
    meaning: "The dealing does not hold together: the wrong number of commitments, an identity element, a proof of knowledge that does not verify, or not exactly one share per trustee."
    retryable: false
related:
  - voting/trustees/enrol
  - voting/trustees/confirm
  - vta/pcs/trustee/deal
---

## Abstract

The **Voting — Trustees — Deal** Trust Task is step 2 of generating an election key (CONVENTIONS §9.6): each trustee publishes Feldman commitments to a random polynomial of degree `t − 1`, a proof of knowledge of its constant term, and its value at each trustee's index sealed to that trustee's transport key. The family's shared rules are in [`CONVENTIONS.md`](../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../voting/_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the trustee, through its client) **MUST**:

1. Sign the request as the trustee's DID, and send the dealing its VTA built ([`vta/pcs/trustee/deal`](../../../../vta/pcs/trustee/deal/0.1/spec.md)) unchanged.

A conforming **consumer** (the ballot box) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` unless the proven signer is the manifest's `trustees.dids` entry at the payload's trustee index (VTI-VOT-062). The index is `dealing.dealer`.
3. Refuse with `voting/trustees/deal:electionUnknown` for an election not registered for `community`.
4. Refuse with `voting/trustees/deal:notDealing` unless the ceremony is dealing.
5. Refuse with `voting/trustees/deal:dealingInvalid` unless the dealing has exactly `t` commitments, none the identity, a proof of knowledge of the first that verifies (CONVENTIONS §9.3, domain `vti/vote/trustee/pok/v1`), and exactly one sealed share per trustee index.
6. Answer a repeat of the same dealing as the first; refuse another with `voting/trustees/deal:alreadyDealt`.
7. Append a `trusteeStep` entry whose `request` is this request exactly as received, proof included (CONVENTIONS §9.6); when every trustee has dealt, the ceremony is confirming.

## Authorization

The authority is **being the trustee the manifest names at the dealer's index**.

## Definitions

- **`dealing`** — the commitments, proof and sealed shares.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "type": "https://trusttasks.org/spec/voting/trustees/deal/0.1#request",
  "issuer": "did:webvh:QmTrustee:alice.example",
  "recipient": "did:webvh:QmBallotBox:vote.example.net",
  "issuedAt": "2026-11-02T09:00:00Z",
  "payload": {
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026",
    "dealing": {
      "dealer": 1,
      "commitments": [
        "z4Lp9VmXkCq2Jt7HbR1NfWd8YsGa3Ue6ZoQi5MxKcTvPy",
        "z4Lp9VmXkCq2Jt7HbR1NfWd8YsGa3Ue6ZoQi5MxKcTvPy"
      ],
      "pok": {
        "c": "z3Hs8WkqL2mNvB7cR9xTyE4uJpQ1aZf6gDoK5iMt8n",
        "z": "z3Hs8WkqL2mNvB7cR9xTyE4uJpQ1aZf6gDoK5iMt8n"
      },
      "shares": [
        {
          "recipient": 1,
          "share": {
            "kem": {
              "a": "z4Lp9VmXkCq2Jt7HbR1NfWd8YsGa3Ue6ZoQi5MxKcTvPy",
              "b": "z4Lp9VmXkCq2Jt7HbR1NfWd8YsGa3Ue6ZoQi5MxKcTvPy"
            },
            "body": "z3Hs8WkqL2mNvB7cR9xTyE4uJpQ1aZf6gDoK5iMt8n"
          }
        },
        {
          "recipient": 2,
          "share": {
            "kem": {
              "a": "z4Lp9VmXkCq2Jt7HbR1NfWd8YsGa3Ue6ZoQi5MxKcTvPy",
              "b": "z4Lp9VmXkCq2Jt7HbR1NfWd8YsGa3Ue6ZoQi5MxKcTvPy"
            },
            "body": "z3Hs8WkqL2mNvB7cR9xTyE4uJpQ1aZf6gDoK5iMt8n"
          }
        }
      ]
    }
  }
}
```

## Response

The answer is the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1002",
  "type": "https://trusttasks.org/spec/voting/trustees/deal/0.1#response",
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

Commitments are public. Each share is sealed to its recipient's transport key and readable by nobody else.

### Correlation

The trustee's and the ballot box's identifier scope is `public`. Trustees are named in the manifest and public by design (VTI-VOT-090): anyone recounting the election must be able to tell which trustee published what, which a pairwise identifier would prevent. The ballot box's DID is public for the same reason — the community and every voter must reach the same box, and its signed heads are evidence only under a DID everyone knows. Nothing here concerns a voter.

### Retention

Kept on the board for as long as the board is (VTI-VOT-070): the result is recomputed from it.

### Consent/purpose

Collected to form the election key or decrypt its result, and for nothing else.
