---
slug: vta/pcs/trustee/enrol
version: "0.2"
title: "VTA PCS — Trustee Enrol"
summary: "The trustee's VTA generates the election's transport key and returns the voting/trustees/enrol request signed as the trustee."
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
    On first use for the election, generates and keeps the transport key's secret; a repeat returns the same key.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    Out: the transport key and its proof, signed as the trustee. The secret stays in the VTA.
retention:
  class: durable
  rationale: >-
    The transport secret is kept until the trustee's shares are opened.
errorCodes:
  - code: vta/pcs/trustee/enrol:trusteeNotHeld
    meaning: "The VTA holds no `authentication` key of `trustee` that the caller may act as, or the manifest does not name `trustee` as a trustee."
    retryable: false
  - code: vta/pcs/trustee/enrol:notTrustee
    meaning: "The manifest's `trustees.dids` does not list `trustee`."
    retryable: false
related:
  - voting/trustees/enrol
  - vta/pcs/trustee/deal
---

## Abstract

The **VTA PCS — Trustee Enrol** Trust Task makes the trustee's VTA the custodian of its transport key (CONVENTIONS §9.6 item 1): the secret never leaves the VTA, and the client carries the signed enrolment to the ballot box. The family's shared rules are in [`CONVENTIONS.md`](../../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.1

The request's `manifest` is now the [`Manifest`](../../../../../voting/_shared/0.2/voting.schema.json) of `voting/_shared/0.2`, which renames three of its members. Each keeps its type (a `date-time` string), its requiredness and its meaning:

- `registrationOpens` → **`registrationOpensAt`**
- `votingOpens` → **`votingOpensAt`**
- `votingCloses` → **`votingClosesAt`**

The reason is the registry's naming consistency: a member holding an instant is named for its event with the suffix `At`. Nothing else in the payload changes.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../../voting/_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Pass the manifest exactly as the community published it, and send the returned document unchanged.

A conforming **consumer** (the VTA) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `vta/pcs/trustee/enrol:trusteeNotHeld` unless it holds an `authentication` key of `trustee` the caller may act as.
3. Refuse with `vta/pcs/trustee/enrol:notTrustee` unless `trustee` is in the manifest's `trustees.dids`.
4. Generate, once per (context, community, election, trustee), a transport secret `k` from a path allocated under the context's base, and keep it under the context's key records; a repeat uses the same.
5. Return the voting/trustees/enrol request for `T = k·G` with its Schnorr proof, signed as `trustee` for `authentication`.

## Authorization

The authority is **standing over the context** the trustee persona lives in, expressed as the capability **`pcsTrustee`**, and standing to act as `trustee`. It is not `sign`: the VTA signs only this task's document as the trustee.

## Definitions

- **`manifest`** — the election's manifest.
- **`document`** — the signed enrolment.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "type": "https://trusttasks.org/spec/vta/pcs/trustee/enrol/0.2#request",
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
      "registrationOpensAt": "2026-11-01T00:00:00Z",
      "votingOpensAt": "2026-11-08T00:00:00Z",
      "votingClosesAt": "2026-11-15T00:00:00Z",
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
  "type": "https://trusttasks.org/spec/vta/pcs/trustee/enrol/0.2#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-11-02T09:00:01Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "payload": {
    "document": {
      "id": "urn:uuid:0e2f9b1c-5d4a-4e3b-9a8c-7d6e5f4a3b2c",
      "type": "https://trusttasks.org/spec/voting/trustees/enrol/0.1",
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

A public key and proof out; the secret stays.

### Correlation

Trustees are public by design (VTI-VOT-090); nothing here concerns a voter.

### Retention

The transport secret is kept until the ceremony completes.

### Consent/purpose

The VTA acts as the trustee only for this election's ceremony and decryption.
