---
slug: vta/pcs/trustee/justify
version: "0.2"
title: "VTA PCS — Trustee Justify"
summary: "The dealer's VTA reveals the share it dealt to a trustee that complained, and returns the voting/trustees/justify request signed as the dealer."
status: draft
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
  level: none
  rationale: >-
    Reveals a share it already dealt; keeps nothing new.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    Out: one share in the clear, which alone reveals nothing about the election key.
retention:
  class: transient
  rationale: >-
    Nothing new is kept.
errorCodes:
  - code: vta/pcs/trustee/justify:trusteeNotHeld
    meaning: "The VTA holds no `authentication` key of `trustee` that the caller may act as, or the manifest does not name `trustee` as a trustee."
    retryable: false
  - code: vta/pcs/trustee/justify:noDealing
    meaning: "This VTA has not dealt for the trustee in this election."
    retryable: false
  - code: vta/pcs/trustee/justify:ceremonyInvalid
    meaning: "A step of `ceremony` is not its trustee's (CONVENTIONS §9.6)."
    retryable: false
  - code: vta/pcs/trustee/justify:noComplaint
    meaning: "No confirmation in `ceremony` from the trustee at `recipient` complains of this trustee."
    retryable: false
related:
  - voting/trustees/justify
---

## Abstract

The **VTA PCS — Trustee Justify** Trust Task answers a complaint (CONVENTIONS §9.6 item 4): the dealer's VTA publishes the share it dealt to the complaining trustee, so anyone can check it against the commitments. The family's shared rules are in [`CONVENTIONS.md`](../../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../../voting/_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Pass the manifest exactly as the community published it and the ballot box's `steps` unchanged as `ceremony`, and call it only for a complaint they hold naming this trustee as dealer.

A conforming **consumer** (the VTA) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `vta/pcs/trustee/justify:trusteeNotHeld` unless it holds an `authentication` key of `trustee` the caller may act as.
3. Refuse with `vta/pcs/trustee/justify:noDealing` unless it holds the dealing it made for this election.
4. Refuse with `vta/pcs/trustee/justify:ceremonyInvalid` unless every step of `ceremony` is its trustee's (CONVENTIONS §9.6, the proof verified by the VTA itself).
5. Refuse with `vta/pcs/trustee/justify:noComplaint` unless those steps hold a confirmation from the trustee the manifest names at `recipient` complaining of this trustee. A share revealed without one would be revealed to whoever asked: enough of them are the polynomial, and with every dealer's, the election key.
6. Return the voting/trustees/justify request with the share for `recipient`, signed as `trustee`.

## Authorization

The authority is **standing over the context** the trustee persona lives in, expressed as the capability **`pcsTrustee`**, and standing to act as `trustee`. It is not `sign`: the VTA signs only this task's document as the trustee.

## Definitions

- **`manifest`** — the election's manifest: its trustees are whom the steps are checked against.
- **`ceremony`** — the ceremony as the board holds it (CONVENTIONS §9.6).
- **`recipient`** — the complaining trustee.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "type": "https://trusttasks.org/spec/vta/pcs/trustee/justify/0.2#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-11-02T09:00:00Z",
  "payload": {
    "contextId": "trustees",
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026",
    "trustee": "did:webvh:QmTrustee:alice.example",
    "recipient": 2,
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
    "ceremony": [
      {
        "id": "urn:uuid:0e2f9b1c-5d4a-4e3b-9a8c-7d6e5f4a3b2c",
        "type": "https://trusttasks.org/spec/voting/trustees/enrol/0.1",
        "issuer": "did:webvh:QmTrustee2:bob.example",
        "recipient": "did:webvh:QmBallotBox:vote.example.net",
        "issuedAt": "2026-11-02T09:00:01Z",
        "payload": {
          "community": "did:webvh:QmCommunity:vtc.example.org",
          "electionId": "board-2026",
          "transportKey": {
            "trustee": 2,
            "key": "z4Lp9VmXkCq2Jt7HbR1NfWd8YsGa3Ue6ZoQi5MxKcTvPy",
            "pok": {
              "c": "z3Hs8WkqL2mNvB7cR9xTyE4uJpQ1aZf6gDoK5iMt8n",
              "z": "z3Hs8WkqL2mNvB7cR9xTyE4uJpQ1aZf6gDoK5iMt8n"
            }
          }
        },
        "proof": {
          "type": "DataIntegrityProof",
          "cryptosuite": "eddsa-jcs-2022",
          "verificationMethod": "did:webvh:QmTrustee2:bob.example#key-0",
          "proofPurpose": "authentication",
          "created": "2026-11-02T09:00:01Z",
          "proofValue": "z4ExampleTrusteeProofValue"
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
  "type": "https://trusttasks.org/spec/vta/pcs/trustee/justify/0.2#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-11-02T09:00:01Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "payload": {
    "document": {
      "id": "urn:uuid:0e2f9b1c-5d4a-4e3b-9a8c-7d6e5f4a3b2c",
      "type": "https://trusttasks.org/spec/voting/trustees/justify/0.1",
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

One share in the clear.

### Correlation

Trustees are public by design (VTI-VOT-090); nothing here concerns a voter.

### Retention

Nothing is kept.

### Consent/purpose

The VTA acts as the trustee only for this election's ceremony and decryption.
