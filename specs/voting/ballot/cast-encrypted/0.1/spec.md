---
slug: voting/ballot/cast-encrypted
version: "0.1"
title: "Voting — Ballot — Cast Encrypted"
summary: "A voter casts a sealed or tallied-encrypted ballot, admitted by its PCS attestation and, when tallied-encrypted, by its proofs of validity."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - voting
  - ballot
  - cast
  - encrypted
  - pcs
parties:
  - role: Voter, as a key made for this cast alone
    requirement: REQUIRED
    member: issuer
  - role: Ballot box (voting service)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The cast is signed by a key made for it alone (VTI-VOT-003); the receipt is what the voter holds against the ballot box.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3, so SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: mutating
  rationale: >-
    Appends the cast to the board, and a replacement where the manifest allows a later vote.
subjectPath: /community
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: an encrypted ballot, a tag and an attestation — nothing identifying the voter. Out: a receipt.
retention:
  class: durable
  rationale: >-
    The board keeps the cast for as long as it is kept.
errorCodes:
  - code: voting/ballot/cast-encrypted:electionUnknown
    meaning: "No election with this identifier exists for this community."
    retryable: false
  - code: voting/ballot/cast-encrypted:notOpen
    meaning: "Voting is not open, or the election key is not formed."
    retryable: false
  - code: voting/ballot/cast-encrypted:wrongLevel
    meaning: "The election is open, or the cast's form (`sealed` or `encrypted`) is not the election's level."
    retryable: false
  - code: voting/ballot/cast-encrypted:ballotInvalid
    meaning: "An encrypted ballot's proofs do not verify, or it does not answer the manifest's questions in order."
    retryable: false
  - code: voting/ballot/cast-encrypted:castInvalid
    meaning: "The attestation does not verify for this election and this ballot."
    retryable: false
  - code: voting/ballot/cast-encrypted:alreadyVoted
    meaning: "The manifest refuses a revote and a cast under this tag is counted."
    retryable: false
related:
  - voting/ballot/cast
  - vta/pcs/vote-encrypted
  - voting/trustees/status
---

## Abstract

The **Voting — Ballot — Cast Encrypted** Trust Task admits a cast of a sealed or tallied-encrypted election (CONVENTIONS §9). Its admission is that of [`voting/ballot/cast`](../../cast/0.1/spec.md) — the tag counts once, the attestation is checked — with the attestation binding the encrypted form, and, for a tallied-encrypted ballot, its proofs that every option is 0 or 1 and every question's selections are a count the manifest admits (VTI-VOT-064). The family's shared rules are in [`CONVENTIONS.md`](../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../../voting/_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the voter's client) **MUST**:

1. Send the cast document its VTA built ([`vta/pcs/vote-encrypted`](../../../../vta/pcs/vote-encrypted/0.2/spec.md)) unchanged, signed by the key made for it, over HTTPS (CONVENTIONS §2.4), and resend it unchanged if the answer is lost.

A conforming **consumer** (the ballot box) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `voting/ballot/cast-encrypted:electionUnknown` for an election not registered for `community`.
3. Refuse with `voting/ballot/cast-encrypted:wrongLevel` unless the election is sealed and the cast carries `sealed`, or tallied-encrypted and it carries `encrypted`.
4. Refuse with `voting/ballot/cast-encrypted:notOpen` unless voting is open and the election key is formed.
5. Refuse with `voting/ballot/cast-encrypted:ballotInvalid` a tallied-encrypted ballot whose election and manifest digest are not the election's, whose questions are not the manifest's in order with one ciphertext per option, or whose proofs do not verify under the cast's `tag` (CONVENTIONS §9.3, §9.5); and either form whose ciphertexts — a sealed `kem`, or any option's — already appear in another cast on the board, under another tag. A copied ballot is not a vote.
6. Admit the cast as `voting/ballot/cast` items 4 – 8 do, with the ballot context over the JCS bytes of `sealed` or `encrypted` (CONVENTIONS §9.5), answering `castInvalid` and `alreadyVoted` under this task's codes, and append an `encryptedCast` entry.

## Authorization

Nobody's identity: the attestation is the authority (CONVENTIONS §4).

## Definitions

- **`cast`** — the encrypted ballot, tag and attestation.
- **`entryHash`** — the RFC 9162 leaf hash of the `encryptedCast` entry at `leafIndex` — the receipt, as voting/ballot/cast's.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "type": "https://trusttasks.org/spec/voting/ballot/cast-encrypted/0.1#request",
  "issuer": "did:key:z6MkpVoteCastKey2222222222222222222222222222222",
  "recipient": "did:webvh:QmBallotBox:vote.example.net",
  "issuedAt": "2026-11-02T09:00:00Z",
  "payload": {
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026",
    "cast": {
      "sealed": {
        "kem": {
          "a": "z4Lp9VmXkCq2Jt7HbR1NfWd8YsGa3Ue6ZoQi5MxKcTvPy",
          "b": "z4Lp9VmXkCq2Jt7HbR1NfWd8YsGa3Ue6ZoQi5MxKcTvPy"
        },
        "body": "z3Hs8WkqL2mNvB7cR9xTyE4uJpQ1aZf6gDoK5iMt8n"
      },
      "tag": "z3Hs8WkqL2mNvB7cR9xTyE4uJpQ1aZf6gDoK5iMt8n",
      "attestation": "z3Hs8WkqL2mNvB7cR9xTyE4uJpQ1aZf6gDoK5iMt8n"
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
  "type": "https://trusttasks.org/spec/voting/ballot/cast-encrypted/0.1#response",
  "issuer": "did:webvh:QmBallotBox:vote.example.net",
  "recipient": "did:key:z6MkpVoteCastKey2222222222222222222222222222222",
  "issuedAt": "2026-11-02T09:00:01Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c1001",
  "payload": {
    "electionId": "board-2026",
    "outcome": "counted",
    "leafIndex": 7,
    "entryHash": "zQmPXME1oRtoT627YKaDPDQ3PwA8tdP9rWuAAweLzqSwAWT",
    "receiptToken": "z3Hs8WkqL2mNvB7cR9xTyE4uJpQ1aZf6gDoK5iMt8n",
    "head": {
      "electionId": "board-2026",
      "treeSize": 8,
      "rootHash": "zQmPXME1oRtoT627YKaDPDQ3PwA8tdP9rWuAAweLzqSwAWT",
      "issuedAt": "2026-11-08T09:00:01Z"
    }
  }
}
```

## Security & Privacy

### Data carried

An encrypted ballot: a sealed one is opened only after close, with the trustees; a tallied-encrypted one is never opened (VTI-VOT-064).

### Correlation

As `voting/ballot/cast`: the cast carries no identifier of the voter, only a key made for it alone. The ballot box's identifier scope is `public` because every voter must reach the same box and its receipts are evidence only under a DID everyone knows.

### Retention

On the board for as long as the board.

### Consent/purpose

Collected to count the vote.
