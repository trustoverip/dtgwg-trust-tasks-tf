---
slug: voting/trustees/authorize
version: "0.1"
title: "Voting — Trustees — Authorize"
summary: "After voting closes, the community authorizes the decryption of exactly the targets it computed from the board, pinned by digest to a head."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - voting
  - trustee
  - threshold
  - decryption
parties:
  - role: Community (registrar)
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
    The request is the community's word on what may be decrypted; a trustee's VTA decrypts nothing
    without it, so it must verify as the community's wherever it is carried. The response is the
    ballot box's, signed like every answer it gives (VTI-VOT-050).
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3, so SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: mutating
  rationale: >-
    Appends a `decryptionAuthorized` entry carrying the request as received; a repeat of the same
    authorization changes nothing.
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
  - code: voting/trustees/authorize:electionUnknown
    meaning: "No election with this identifier exists for this community."
    retryable: false
  - code: voting/trustees/authorize:notClosed
    meaning: "Voting has not closed, or the election is not sealed or tallied-encrypted, or its key never formed."
    retryable: false
  - code: voting/trustees/authorize:targetsMismatch
    meaning: "The ballot box's decryption targets are not the ones the community authorized: another count, another digest, or another head."
    retryable: false
  - code: voting/trustees/authorize:alreadyAuthorized
    meaning: "The community has already authorized another set of targets for this election."
    retryable: false
related:
  - voting/trustees/decrypt
  - voting/trustees/status
  - vta/pcs/trustee/decrypt
---

## Abstract

The **Voting — Trustees — Authorize** Trust Task is how a community, after voting closes, tells
its trustees what they may decrypt. The community reads the closed board, re-admits every cast as
certification would, computes the decryption targets itself (CONVENTIONS §9.7) and signs their
digest together with the head it read them under. The ballot box accepts the authorization only
if its own targets have that digest, and appends it to the board. A trustee's VTA decrypts only
targets an authorization the community signed pins
([`vta/pcs/trustee/decrypt/0.2`](../../../../vta/pcs/trustee/decrypt/0.2/spec.md)), so a ballot box
alone can neither show the trustees a single voter's ciphertexts as a "total" nor ask for a
decryption before the close. The family's shared rules are in
[`CONVENTIONS.md`](../../../_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **producer** (the community) **MUST**:

1. Send it only after reading, from the ballot box, every entry under a head that includes the board's `closed` or `closedEarly` entry, and checking them as certification does up to that entry (CONVENTIONS §3.9, §9.6 – 9.7): the ceremony from its signed steps, every encrypted cast re-admitted under the key it forms, the board's order.
2. Compute the targets itself from those entries (CONVENTIONS §9.7), and set `targetCount` and `targetsDigest` from them, `head` to the head it read them under truncated to the board through the closing entry, and `closedAt` to that entry's `at`.
3. Send it once per election. A community **SHOULD** send it as soon as it has read the closed board.

A conforming **consumer** (the ballot box) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `voting/trustees/authorize:electionUnknown` for an election not registered for `community`, and with `permissionDenied` unless the proven signer is `community`.
3. Refuse with `voting/trustees/authorize:notClosed` unless voting has closed and the election's key formed.
4. Refuse with `voting/trustees/authorize:targetsMismatch` unless `manifestDigest` is the election's, `head` is the board through its closing entry, `closedAt` is that entry's `at`, and its own targets number `targetCount` and have the digest `targetsDigest`.
5. Answer a repeat of an authorization it holds as the first time, and refuse any other with `voting/trustees/authorize:alreadyAuthorized`.
6. Append a `decryptionAuthorized` entry whose `request` is this request exactly as received, proof included, and carry it as `authorization` in [`voting/trustees/status`](../../status/0.1/spec.md) from then on.

## Authorization

The community the payload names, by the request's proof ([conventions §4](../../../_shared/0.1/CONVENTIONS.md#4-authorization)).

## Definitions

- **`head`** — the board through its `closed` or `closedEarly` entry.
- **`closedAt`** — that entry's `at`.
- **`targetCount`**, **`targetsDigest`** — how many targets, and SHA-256 over the JCS of their array as a multihash.

## Request

The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c3001",
  "type": "https://trusttasks.org/spec/voting/trustees/authorize/0.1#request",
  "issuer": "did:webvh:QmCommunity:vtc.example.org",
  "recipient": "did:webvh:QmBallotBox:vote.example.net",
  "issuedAt": "2026-11-15T00:05:00Z",
  "payload": {
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026",
    "manifestDigest": "zQmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG",
    "head": {
      "electionId": "board-2026",
      "treeSize": 412,
      "rootHash": "zQmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG",
      "issuedAt": "2026-11-15T00:04:58Z"
    },
    "closedAt": "2026-11-15T00:00:00Z",
    "targetCount": 6,
    "targetsDigest": "zQmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG"
  }
}
```

## Response

The answer is the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c3002",
  "type": "https://trusttasks.org/spec/voting/trustees/authorize/0.1#response",
  "issuer": "did:webvh:QmBallotBox:vote.example.net",
  "recipient": "did:webvh:QmCommunity:vtc.example.org",
  "issuedAt": "2026-11-15T00:05:01Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-0000000c3001",
  "payload": {
    "election": {
      "electionId": "board-2026",
      "community": "did:webvh:QmCommunity:vtc.example.org",
      "manifestDigest": "zQmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG",
      "state": "closed",
      "ballotsAdmitted": 205,
      "head": {
        "electionId": "board-2026",
        "treeSize": 413,
        "rootHash": "zQmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG",
        "issuedAt": "2026-11-15T00:05:01Z"
      }
    },
    "phase": "decrypting"
  }
}
```

## Security & Privacy

### Data carried

A digest of the decryption targets and the head they were computed under. Nothing about a voter.

### Correlation

The community's and the ballot box's identifier scope is `public`. The authorization is board
evidence: a trustee's VTA and every auditor check that it was signed by the community that
registered the election, and that it reached the box whose heads they verify, so both must be
the DIDs the manifest and the board name. A pairwise identifier would leave nobody able to tell
the authorization from one a ballot box made for itself. The request names nothing else; no voter
appears in it.

### Retention

On the board, durably, as every entry is.

### Consent/purpose

The community authorizes only the decryption of totals or counted sealed casts it computed from
the closed board — never of an individual tallied ballot, and never before the close.
