---
slug: vta/pcs/trustee/keys
version: "0.1"
title: "VTA PCS — Trustee Keys"
summary: "The trustee's VTA opens the shares it was dealt, checks each, keeps its share of the election key, and returns the voting/trustees/confirm request signed as the trustee."
status: retired
supersededBy: vta/pcs/trustee/keys/0.2
targetFrameworkVersion: "0.6.0"
category: identity
keywords:
  - pcs
  - voting
  - trustee
  - vta
parties:
  - role: Client driving the trustee's VTA
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: The trustee's VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The request makes the VTA act as the trustee with keys that decide the election's secrecy. Where the transport does not already authenticate the caller, the proof is the only thing between those keys and anyone who can reach the VTA.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3, so SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: mutating
  rationale: >-
    Keeps the trustee's share of the election key once the key is formed.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    Out: which dealers' shares failed, and the signed confirmation. The key share stays in the VTA.
retention:
  class: durable
  rationale: >-
    The key share is kept until the election is decrypted.
errorCodes:
  - code: vta/pcs/trustee/keys:trusteeNotHeld
    meaning: "The VTA holds no `authentication` key of `trustee` that the caller may act as, or the manifest does not name `trustee` as a trustee."
    retryable: false
  - code: vta/pcs/trustee/keys:notEnrolled
    meaning: "This VTA has no transport key for the trustee in this election."
    retryable: false
  - code: vta/pcs/trustee/keys:keyMismatch
    meaning: "The election key's verification key for this trustee is not the share this VTA computed."
    retryable: false
related:
  - voting/trustees/confirm
  - vta/pcs/trustee/decrypt
---

## Abstract

The **VTA PCS — Trustee Keys** Trust Task is the trustee's side of steps 3 and 5 (CONVENTIONS §9.6): called with the dealings, it opens and checks each share and returns the confirmation to publish; called again with the formed election key, it keeps the key share and checks it against the trustee's published verification key. The family's shared rules are in [`CONVENTIONS.md`](../../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is **retired** per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); it is superseded by [`vta/pcs/trustee/keys/0.2`](../0.2/spec.md), which acts only on the signed evidence the ballot box cannot forge (CONVENTIONS §9.6 – 9.7) where this version took values the ballot box reported. The schema is frozen; the document is retained so already-issued documents remain verifiable.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../../voting/_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Call it with the dealings once every trustee has dealt, publish the returned confirmation, and call it again with `electionKey` (and any `justifications`) once the key is formed.

A conforming **consumer** (the VTA) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `vta/pcs/trustee/keys:trusteeNotHeld` unless it holds an `authentication` key of `trustee` the caller may act as.
3. Refuse with `vta/pcs/trustee/keys:notEnrolled` unless it holds the transport secret.
4. Open each share addressed to the trustee with the transport secret, check it against its dealer's commitments, and list the dealers whose share fails or does not open as `complaints`.
5. Without `electionKey`: return the voting/trustees/confirm request for those complaints, signed as `trustee`.
6. With `electionKey`: compute the key share from the qualified dealers' shares (a justified share in place of the one that failed), refuse with `vta/pcs/trustee/keys:keyMismatch` unless it matches this trustee's verification key, keep it, and answer `keyHeld: true`.

## Authorization

The authority is **standing over the context** the trustee persona lives in, expressed as the capability **`pcsTrustee`**, and standing to act as `trustee`. It is not `sign`: the VTA signs only this task's document as the trustee.

## Definitions

- **`dealings`** — every dealing on the board.
- **`electionKey`** — the formed key, once there is one.
- **`keyHeld`** — whether the key share is kept.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "type": "https://trusttasks.org/spec/vta/pcs/trustee/keys/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-11-02T09:00:00Z",
  "payload": {
    "contextId": "trustees",
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026",
    "trustee": "did:webvh:QmTrustee:alice.example",
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
    "dealings": [
      {
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
  "type": "https://trusttasks.org/spec/vta/pcs/trustee/keys/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-11-02T09:00:01Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "payload": {
    "complaints": [],
    "keyHeld": false,
    "document": {
      "id": "urn:uuid:0e2f9b1c-5d4a-4e3b-9a8c-7d6e5f4a3b2c",
      "type": "https://trusttasks.org/spec/voting/trustees/confirm/0.1",
      "issuer": "did:webvh:QmTrustee:alice.example",
      "recipient": "did:webvh:QmBallotBox:vote.example.net",
      "issuedAt": "2026-11-02T09:00:01Z",
      "payload": {},
      "proof": {}
    }
  }
}
```

## Security & Privacy

### Data carried

Complaints out; shares and the key share stay.

### Correlation

Trustees are public by design (VTI-VOT-090); nothing here concerns a voter.

### Retention

The key share is kept until the result is decrypted.

### Consent/purpose

The VTA acts as the trustee only for this election's ceremony and decryption.
