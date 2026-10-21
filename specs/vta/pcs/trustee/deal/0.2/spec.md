---
slug: vta/pcs/trustee/deal
version: "0.2"
title: "VTA PCS — Trustee Deal"
summary: "The trustee's VTA deals its contribution to the election key and returns the voting/trustees/deal request signed as the trustee."
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
  level: mutating
  rationale: >-
    Draws the trustee's polynomial once and keeps its own share; a repeat returns the same dealing.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    Out: commitments, a proof and shares sealed to each trustee. The polynomial stays in the VTA, until the ceremony forms the key.
retention:
  class: durable
  rationale: >-
    The dealing is kept so a repeat is identical and a complaint can be justified.
errorCodes:
  - code: vta/pcs/trustee/deal:trusteeNotHeld
    meaning: "The VTA holds no `authentication` key of `trustee` that the caller may act as, or the manifest does not name `trustee` as a trustee."
    retryable: false
  - code: vta/pcs/trustee/deal:notEnrolled
    meaning: "This VTA has no transport key for the trustee in this election: enrol first."
    retryable: false
  - code: vta/pcs/trustee/deal:ceremonyInvalid
    meaning: "A step of `ceremony` is not its trustee's (CONVENTIONS §9.6), or the ceremony is not dealing: not every trustee the manifest names has enrolled, or this trustee's own transport key is not the one this VTA holds."
    retryable: false
  - code: vta/pcs/trustee/deal:ceremonyChanged
    meaning: "The ceremony's transport keys are not those this VTA first dealt to: it deals to one set of keys, once."
    retryable: false
related:
  - voting/trustees/deal
  - vta/pcs/trustee/keys
---

## Abstract

The **VTA PCS — Trustee Deal** Trust Task draws the trustee's random polynomial and builds its dealing (CONVENTIONS §9.6 item 2), sealing each trustee's share to that trustee's transport key. The family's shared rules are in [`CONVENTIONS.md`](../../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../../voting/_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Pass the ballot box's `steps` (voting/trustees/status) unchanged as `ceremony`, and send the returned document unchanged.

A conforming **consumer** (the VTA) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `vta/pcs/trustee/deal:trusteeNotHeld` unless it holds an `authentication` key of `trustee` the caller may act as.
3. Refuse with `vta/pcs/trustee/deal:notEnrolled` unless it holds this trustee's transport secret for the election.
4. Refuse with `vta/pcs/trustee/deal:ceremonyInvalid` unless every step of `ceremony` is its trustee's — checked as CONVENTIONS §9.6 requires, the proof verified by the VTA itself — and the steps, taken in order, leave the ceremony dealing, with this trustee's own transport key the one the VTA holds. It takes the transport keys from those steps and from nothing else.
5. On the first dealing, keep a digest of the transport keys dealt to; refuse with `vta/pcs/trustee/deal:ceremonyChanged` any later request whose keys differ. A share sealed to a key the ballot box substituted would be the ballot box's to open.
6. Draw the polynomial once and keep it; a repeat returns the dealing built the first time.
7. Return the voting/trustees/deal request signed as `trustee`.

## Authorization

The authority is **standing over the context** the trustee persona lives in, expressed as the capability **`pcsTrustee`**, and standing to act as `trustee`. It is not `sign`: the VTA signs only this task's document as the trustee.

## Definitions

- **`ceremony`** — the ceremony as the board holds it (CONVENTIONS §9.6).
- **`document`** — the signed dealing.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "type": "https://trusttasks.org/spec/vta/pcs/trustee/deal/0.2#request",
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
  "type": "https://trusttasks.org/spec/vta/pcs/trustee/deal/0.2#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-11-02T09:00:01Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "payload": {
    "document": {
      "id": "urn:uuid:0e2f9b1c-5d4a-4e3b-9a8c-7d6e5f4a3b2c",
      "type": "https://trusttasks.org/spec/voting/trustees/deal/0.1",
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

Public commitments and sealed shares out; the polynomial stays.

### Correlation

Trustees are public by design (VTI-VOT-090); nothing here concerns a voter.

### Retention

Kept until the election key is formed.

### Consent/purpose

The VTA acts as the trustee only for this election's ceremony and decryption.
