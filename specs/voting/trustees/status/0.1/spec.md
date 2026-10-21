---
slug: voting/trustees/status
version: "0.1"
title: "Voting — Trustees — Status"
summary: "Where an election's trustee ceremony stands: transport keys, dealings, complaints, the election key, and what remains to decrypt."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - voting
  - trustee
  - threshold
parties:
  - role: Anyone who may read the board, or a trustee
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Ballot box (voting service)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  request: OPTIONAL
  response: REQUIRED
  rationale: >-
    A read of what the board already holds. On a private board the signer must be one the board admits, or a trustee the manifest names.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Places a signed read in time; the answer is a snapshot.
sideEffects:
  level: none
  rationale: >-
    Reads the ceremony; changes nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Out: what the board holds about the ceremony, every member of which is public on a public board.
retention:
  class: transient
  rationale: >-
    The ballot box keeps nothing about the read.
errorCodes:
  - code: voting/trustees/status:electionUnknown
    meaning: "No election with this identifier exists for this community, or the caller may neither read its board nor is one of its trustees."
    retryable: false
related:
  - voting/trustees/enrol
  - voting/trustees/deal
  - voting/trustees/decrypt
---

## Abstract

The **Voting — Trustees — Status** Trust Task tells trustees what the ceremony needs of them next, and lets anyone follow it: the transport keys, the dealings (with the shares sealed to each trustee), the complaints and justifications, the election key once formed, and after close the targets left to decrypt. The family's shared rules are in [`CONVENTIONS.md`](../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../voting/_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (a trustee, a community or an auditor) **MUST**:

1. Sign the request on a private board, as a party the board admits or as a trustee.

A conforming **consumer** (the ballot box) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Answer `voting/trustees/status:electionUnknown` for an election not registered for `community`, or whose board the caller may not read unless the proven signer is one of the manifest's `trustees.dids` (CONVENTIONS §4, §9.6): a trustee reads its ceremony on any board.
3. Answer from the board as it stands, every member as recorded there.

## Authorization

Whoever may read the election's board, and — on any board — every trustee the manifest names (CONVENTIONS §4). A trustee's standing is the manifest's, never the board's: it reads the ceremony it takes part in, and nothing else the board holds.

## Definitions

- **`phase`** — the ceremony's phase.
- **`targets`** — what remains to decrypt.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "type": "https://trusttasks.org/spec/voting/trustees/status/0.1#request",
  "issuer": "did:webvh:QmTrustee:alice.example",
  "recipient": "did:webvh:QmBallotBox:vote.example.net",
  "issuedAt": "2026-11-02T09:00:00Z",
  "payload": {
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026"
  }
}
```

## Response

The answer is the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1002",
  "type": "https://trusttasks.org/spec/voting/trustees/status/0.1#response",
  "issuer": "did:webvh:QmBallotBox:vote.example.net",
  "recipient": "did:webvh:QmTrustee:alice.example",
  "issuedAt": "2026-11-02T09:00:01Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "payload": {
    "phase": "dealing",
    "threshold": 2,
    "transportKeys": [
      {
        "trustee": 1,
        "key": "z4Lp9VmXkCq2Jt7HbR1NfWd8YsGa3Ue6ZoQi5MxKcTvPy",
        "pok": {
          "c": "z3Hs8WkqL2mNvB7cR9xTyE4uJpQ1aZf6gDoK5iMt8n",
          "z": "z3Hs8WkqL2mNvB7cR9xTyE4uJpQ1aZf6gDoK5iMt8n"
        }
      }
    ],
    "dealings": [],
    "complaints": [],
    "confirmed": [],
    "targets": [],
    "decrypted": []
  }
}
```

## Security & Privacy

### Data carried

Public ceremony material: keys, commitments, sealed shares.

### Correlation

The ballot box's identifier scope is `public`: everyone following the election must reach the same box. Nothing here concerns a voter.

### Retention

Nothing is kept about the read.

### Consent/purpose

A read.
