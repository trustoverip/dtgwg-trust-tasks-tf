---
slug: vta/pcs/vote-encrypted
version: "0.1"
title: "VTA PCS — Vote Encrypted"
summary: "The voter's VTA checks a ballot against the manifest, encrypts it to the election key as the election's level requires, and returns the cast signed by a key made for it alone."
status: retired
supersededBy: vta/pcs/vote-encrypted/0.2
targetFrameworkVersion: "0.6.0"
category: identity
keywords:
  - pcs
  - voting
  - ballot
  - encrypted
  - vta
parties:
  - role: Client driving the PCS engine
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: PCS engine (the VTA)
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    As `vta/pcs/vote`: the request makes the VTA exercise the election's voter key and the ticket.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3, so SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: mutating
  rationale: >-
    As `vta/pcs/vote`: records the cast it built, so a receipt can be checked against it.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    In: the plaintext ballot, which stays in the VTA. Out: the encrypted cast, signed by a one-time key.
retention:
  class: durable
  rationale: >-
    The cast's board leaf is kept until the election is tallied.
errorCodes:
  - code: vta/pcs/vote-encrypted:notFound
    meaning: "The engine holds no ticket for this election."
    retryable: false
  - code: vta/pcs/vote-encrypted:wrongLevel
    meaning: "The manifest's level is open: use vta/pcs/vote."
    retryable: false
  - code: vta/pcs/vote-encrypted:electionKeyInvalid
    meaning: "The election key does not hold together, or names other trustees than the manifest."
    retryable: false
  - code: vta/pcs/vote-encrypted:ballotInvalid
    meaning: "The ballot does not answer the manifest, or a tallied-encrypted ballot asks a question that is not a choice."
    retryable: false
  - code: vta/pcs/vote-encrypted:notOpen
    meaning: "Voting is not open."
    retryable: false
related:
  - vta/pcs/vote
  - voting/ballot/cast-encrypted
---

## Abstract

The **VTA PCS — Vote Encrypted** Trust Task is [`vta/pcs/vote`](../../vote/0.1/spec.md) for the encrypted levels (CONVENTIONS §9.5): the ballot is sealed to the election key, or encrypted option by option with proofs, and the attestation binds the encrypted form. The family's shared rules are in [`CONVENTIONS.md`](../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is **retired** per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); it is superseded by [`vta/pcs/vote-encrypted/0.2`](../0.2/spec.md), which acts only on the signed evidence the ballot box cannot forge (CONVENTIONS §9.6 – 9.7) where this version took values the ballot box reported. The schema is frozen; the document is retained so already-issued documents remain verifiable.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../voting/_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Pass the election key voting/trustees/status reports, the manifest and parameters exactly as published, and send the returned document to the ballot box itself, unchanged.

A conforming **consumer** (the VTA) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `vta/pcs/vote-encrypted:notFound` unless it holds the ticket.
3. Refuse with `vta/pcs/vote-encrypted:wrongLevel` for an open manifest.
4. Refuse with `vta/pcs/vote-encrypted:ballotInvalid` a ballot that does not answer the manifest (as `vta/pcs/vote`), or, tallied-encrypted, one with a question other than a choice.
5. Refuse with `vta/pcs/vote-encrypted:notOpen` outside the manifest's voting window.
6. Build the encrypted form (CONVENTIONS §9.4 or §9.5), the attestation over its JCS bytes, and the cast document signed by a key made for it alone, as `vta/pcs/vote` does.

## Authorization

As `vta/pcs/vote`: standing over the context, as `pcsVoter`.

## Definitions

- **`electionKey`** — the formed election key.
- **`castDocument`** — the signed cast.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "type": "https://trusttasks.org/spec/vta/pcs/vote-encrypted/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-11-02T09:00:00Z",
  "payload": {
    "contextId": "personal",
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "parameters": {
      "suite": "ps-ddh-bls12381",
      "votingKey": "z6MkvVotingKey1abcXYZ"
    },
    "manifest": {
      "electionId": "board-2026",
      "community": "did:webvh:QmCommunity:vtc.example.org",
      "title": "Board election 2026",
      "questions": [
        {
          "key": "chair",
          "type": "choice",
          "text": "Who should chair the board?",
          "options": [
            {
              "key": "ana",
              "label": "Ana"
            },
            {
              "key": "ben",
              "label": "Ben"
            }
          ],
          "minSelections": 1,
          "maxSelections": 1,
          "optional": false
        }
      ],
      "secrecy": "talliedEncrypted",
      "roll": {
        "rule": "members"
      },
      "registrationOpens": "2026-11-01T00:00:00Z",
      "votingOpens": "2026-11-08T00:00:00Z",
      "votingCloses": "2026-11-15T00:00:00Z",
      "revote": "replace",
      "board": {
        "visibility": "public",
        "auditors": []
      },
      "ballotBox": "did:webvh:QmBallotBox:vote.example.net",
      "allowEarlyClose": false,
      "resultsBeforeClose": false,
      "textReleaseThreshold": 10,
      "trustees": {
        "dids": [
          "did:webvh:QmTrustee:alice.example",
          "did:webvh:QmTrustee2:bob.example",
          "did:webvh:QmTrustee3:carol.example"
        ],
        "threshold": 2
      },
      "binding": true
    },
    "electionKey": {
      "key": "z4Lp9VmXkCq2Jt7HbR1NfWd8YsGa3Ue6ZoQi5MxKcTvPy",
      "qualified": [
        1,
        2
      ],
      "verification": [
        {
          "trustee": 1,
          "key": "z4Lp9VmXkCq2Jt7HbR1NfWd8YsGa3Ue6ZoQi5MxKcTvPy"
        },
        {
          "trustee": 2,
          "key": "z4Lp9VmXkCq2Jt7HbR1NfWd8YsGa3Ue6ZoQi5MxKcTvPy"
        }
      ]
    },
    "ballot": {
      "electionId": "board-2026",
      "manifestDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
      "answers": [
        {
          "question": "chair",
          "choice": [
            "ana"
          ]
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
  "type": "https://trusttasks.org/spec/vta/pcs/vote-encrypted/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-11-02T09:00:01Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "payload": {
    "castDocument": {
      "id": "urn:uuid:0e2f9b1c-5d4a-4e3b-9a8c-7d6e5f4a3b2c",
      "type": "https://trusttasks.org/spec/voting/ballot/cast-encrypted/0.1",
      "issuer": "did:key:z6MkpVoteCastKey2222222222222222222222222222222",
      "recipient": "did:webvh:QmBallotBox:vote.example.net",
      "issuedAt": "2026-11-08T09:00:01Z",
      "payload": {},
      "proof": {}
    },
    "tag": "z3Hs8WkqL2mNvB7cR9xTyE4uJpQ1aZf6gDoK5iMt8n"
  }
}
```

## Security & Privacy

### Data carried

The plaintext ballot stays in the VTA; only its encryption leaves.

### Correlation

As `vta/pcs/vote`.

### Retention

As `vta/pcs/vote`.

### Consent/purpose

To cast the voter's ballot.
