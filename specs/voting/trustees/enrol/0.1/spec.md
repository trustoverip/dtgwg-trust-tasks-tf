---
slug: voting/trustees/enrol
version: "0.1"
title: "Voting — Trustees — Enrol"
summary: "A trustee of a sealed or tallied-encrypted election publishes the key its shares of the election key are sealed to."
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
    The transport key decides who can open the trustee's shares of the election key. A recount must be able to show that the trustee itself published it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3, so SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: mutating
  rationale: >-
    Appends a `trusteeStep` entry carrying the request as received; dealing opens when every trustee has one.
subjectPath: /community
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a public key and its proof of knowledge. Out: the election's summary and the ceremony's phase.
retention:
  class: durable
  rationale: >-
    On the board for as long as the board.
errorCodes:
  - code: voting/trustees/enrol:electionUnknown
    meaning: "No election with this identifier exists for this community."
    retryable: false
  - code: voting/trustees/enrol:notEnrolling
    meaning: "The ceremony is past enrolment, or the election is not sealed or tallied-encrypted."
    retryable: false
  - code: voting/trustees/enrol:alreadyEnrolled
    meaning: "This trustee has already published another transport key."
    retryable: false
  - code: voting/trustees/enrol:transportKeyInvalid
    meaning: "The transport key's proof of knowledge does not verify."
    retryable: false
related:
  - voting/trustees/deal
  - voting/trustees/status
  - vta/pcs/trustee/enrol
---

## Abstract

The **Voting — Trustees — Enrol** Trust Task is the first step of generating an election key (CONVENTIONS §9.6 item 1): each trustee publishes a transport key with a Schnorr proof that it holds the discrete log, so the other trustees can seal its shares to it. The family's shared rules are in [`CONVENTIONS.md`](../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../voting/_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the trustee, through its client) **MUST**:

1. Sign the request as the trustee's DID, and send the transport key its VTA built ([`vta/pcs/trustee/enrol`](../../../../vta/pcs/trustee/enrol/0.1/spec.md)) unchanged.

A conforming **consumer** (the ballot box) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` unless the proven signer is the manifest's `trustees.dids` entry at the payload's trustee index (VTI-VOT-062).
3. Refuse with `voting/trustees/enrol:electionUnknown` for an election not registered for `community`.
4. Refuse with `voting/trustees/enrol:notEnrolling` unless the election is sealed or tallied-encrypted and its ceremony is enrolling.
5. Refuse with `voting/trustees/enrol:transportKeyInvalid` unless the proof verifies (CONVENTIONS §9.3, domain `vti/vote/trustee/transport/v1`).
6. Answer a repeat of the same transport key as the first; refuse another with `voting/trustees/enrol:alreadyEnrolled`.
7. Append a `trusteeStep` entry whose `request` is this request exactly as received, proof included (CONVENTIONS §9.6); when every trustee has enrolled, the ceremony is dealing.

## Authorization

The authority is **being the trustee the manifest names at that index**, enforced by consumer item 2.

## Definitions

- **`transportKey`** — the trustee's index, transport key and its proof.
- **`phase`** — the ceremony's phase after this request.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "type": "https://trusttasks.org/spec/voting/trustees/enrol/0.1#request",
  "issuer": "did:webvh:QmTrustee:alice.example",
  "recipient": "did:webvh:QmBallotBox:vote.example.net",
  "issuedAt": "2026-11-02T09:00:00Z",
  "payload": {
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026",
    "transportKey": {
      "trustee": 1,
      "key": "z4Lp9VmXkCq2Jt7HbR1NfWd8YsGa3Ue6ZoQi5MxKcTvPy",
      "pok": {
        "c": "z3Hs8WkqL2mNvB7cR9xTyE4uJpQ1aZf6gDoK5iMt8n",
        "z": "z3Hs8WkqL2mNvB7cR9xTyE4uJpQ1aZf6gDoK5iMt8n"
      }
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
  "type": "https://trusttasks.org/spec/voting/trustees/enrol/0.1#response",
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

A public key and a proof. No secret crosses: the VTA keeps the transport key's secret.

### Correlation

The trustee's and the ballot box's identifier scope is `public`. Trustees are named in the manifest and public by design (VTI-VOT-090): anyone recounting the election must be able to tell which trustee published what, which a pairwise identifier would prevent. The ballot box's DID is public for the same reason — the community and every voter must reach the same box, and its signed heads are evidence only under a DID everyone knows. Nothing here concerns a voter.

### Retention

Kept on the board for as long as the board is (VTI-VOT-070): the result is recomputed from it.

### Consent/purpose

Collected to form the election key or decrypt its result, and for nothing else.
