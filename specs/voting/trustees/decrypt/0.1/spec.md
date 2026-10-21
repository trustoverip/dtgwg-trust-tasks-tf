---
slug: voting/trustees/decrypt
version: "0.1"
title: "Voting — Trustees — Decrypt"
summary: "After voting closes, a trustee publishes its partial decryption of every target, each with a proof it used its share of the election key."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - voting
  - trustee
  - threshold
  - decryption
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
    A partial decryption decides the result together with others. It must be attributable to the trustee.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3, so SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: mutating
  rationale: >-
    Appends a `partials` entry; once every target has `t` verified partials, the ballot box opens the sealed ballots or reads the totals, and tallies.
subjectPath: /community
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: partial decryptions with proofs. Out: the summary and phase.
retention:
  class: durable
  rationale: >-
    On the board for as long as the board (VTI-VOT-063).
errorCodes:
  - code: voting/trustees/decrypt:electionUnknown
    meaning: "No election with this identifier exists for this community."
    retryable: false
  - code: voting/trustees/decrypt:notDecrypting
    meaning: "Voting has not closed, or the election is already decrypted."
    retryable: false
  - code: voting/trustees/decrypt:partialInvalid
    meaning: "A partial decryption's target is not one of the election's, or its proof does not verify."
    retryable: false
related:
  - voting/trustees/status
  - voting/result/show
  - vta/pcs/trustee/decrypt
---

## Abstract

The **Voting — Trustees — Decrypt** Trust Task publishes one trustee's share of every decryption the result needs (CONVENTIONS §9.7): each counted sealed cast, or each option's total in a tallied-encrypted election. No decryption is made before voting closes (VTI-VOT-063). The family's shared rules are in [`CONVENTIONS.md`](../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../voting/_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the trustee, through its client) **MUST**:

1. Sign the request as the trustee's DID, with the partials its VTA made ([`vta/pcs/trustee/decrypt`](../../../../vta/pcs/trustee/decrypt/0.2/spec.md)) for the targets [`voting/trustees/status`](../../status/0.1/spec.md) lists.

A conforming **consumer** (the ballot box) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` unless the proven signer is the manifest's `trustees.dids` entry at the payload's trustee index (VTI-VOT-062).
3. Refuse with `voting/trustees/decrypt:electionUnknown` for an election not registered for `community`.
4. Refuse with `voting/trustees/decrypt:notDecrypting` unless voting has closed, the community's authorization is on the board (CONVENTIONS §9.7), and the election is not yet tallied.
5. Refuse with `voting/trustees/decrypt:partialInvalid` unless every partial's target is one of the election's targets, with its ciphertext as the ballot box computes it, and its Chaum–Pedersen proof verifies against the trustee's verification key.
6. Append a `partials` entry. When every target has `t` verified partials of distinct trustees, open each sealed ballot (appending an `unsealed` entry per ballot) or read each total, then tally and append the result.

## Authorization

The authority is **being the trustee the manifest names at that index**, after voting has closed.

## Definitions

- **`trustee`** — the decrypting trustee.
- **`partials`** — one partial decryption per target.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "type": "https://trusttasks.org/spec/voting/trustees/decrypt/0.1#request",
  "issuer": "did:webvh:QmTrustee:alice.example",
  "recipient": "did:webvh:QmBallotBox:vote.example.net",
  "issuedAt": "2026-11-02T09:00:00Z",
  "payload": {
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026",
    "trustee": 1,
    "partials": [
      {
        "target": {
          "question": "chair",
          "option": "ana",
          "ciphertext": {
            "a": "z4Lp9VmXkCq2Jt7HbR1NfWd8YsGa3Ue6ZoQi5MxKcTvPy",
            "b": "z4Lp9VmXkCq2Jt7HbR1NfWd8YsGa3Ue6ZoQi5MxKcTvPy"
          }
        },
        "d": "z4Lp9VmXkCq2Jt7HbR1NfWd8YsGa3Ue6ZoQi5MxKcTvPy",
        "proof": {
          "c": "z3Hs8WkqL2mNvB7cR9xTyE4uJpQ1aZf6gDoK5iMt8n",
          "z": "z3Hs8WkqL2mNvB7cR9xTyE4uJpQ1aZf6gDoK5iMt8n"
        }
      }
    ]
  }
}
```

## Response

The answer is the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1002",
  "type": "https://trusttasks.org/spec/voting/trustees/decrypt/0.1#response",
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

Partial decryptions are below the threshold individually and reveal nothing on their own; together they reveal what the level allows — each ballot of a sealed election, only totals of a tallied-encrypted one.

### Correlation

The trustee's and the ballot box's identifier scope is `public`. Trustees are named in the manifest and public by design (VTI-VOT-090): anyone recounting the election must be able to tell which trustee published what, which a pairwise identifier would prevent. The ballot box's DID is public for the same reason — the community and every voter must reach the same box, and its signed heads are evidence only under a DID everyone knows. Nothing here concerns a voter.

### Retention

Kept on the board for as long as the board is (VTI-VOT-070): the result is recomputed from it.

### Consent/purpose

Collected to form the election key or decrypt its result, and for nothing else.
