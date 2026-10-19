---
slug: voting/election/show
version: "0.1"
title: "Voting — Election Show"
summary: "Reads one election at its ballot box — state, manifest digest, ballots admitted and the board's current head — with the registered manifest."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - voting
  - election
  - ballot-box
  - poll
parties:
  - role: Reader (anyone on a public board; the community or an auditor on a private one)
    requirement: OPTIONAL
    member: issuer
    identifierScope: any
  - role: Ballot box (voting service)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  request: RECOMMENDED
  response: REQUIRED
  rationale: >-
    On a private board the ballot box answers only the community and the manifest's auditors, so
    it needs to know who asked; a proof gives it that where the transport does not. The response
    carries the board's current head, and a head is signed by being carried in a document the
    ballot box signs: a holder who keeps it as evidence of what the board held needs the proof.
sideEffects:
  level: none
  rationale: >-
    A read. Nothing at the ballot box changes.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Out: the election's state, counts, head and manifest — what the community published.
    Nothing about any voter.
retention:
  class: transient
  rationale: >-
    The ballot box keeps nothing of a read beyond its ordinary request log.
errorCodes:
  - code: voting/election/show:electionUnknown
    meaning: "No election with this identifier exists for this community, or the caller may not see it."
    retryable: false
related:
  - voting/election/register
  - voting/board/head
  - voting/result/show
  - vtc/elections/show
---

## Abstract

The **Voting — Election Show** Trust Task reads one election as its ballot box holds it: the state of its lifecycle, the digest of the registered manifest, the number of distinct tickets with a counted ballot, the board's current signed head and the manifest itself. A voter's client reads it to check that the election its community describes is the one the ballot box runs before it casts; an auditor reads it before it walks the board. The family's shared rules are in [`CONVENTIONS.md`](../../../_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **consumer** (the ballot box) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `voting/election/show:electionUnknown` when no election `electionId` is registered for `community`.
3. On a private board, refuse with `voting/election/show:electionUnknown` — the same code, in the same form — unless the proven caller is the community or a DID in the manifest's `board.auditors` (VTI-VOT-052). A caller who may not read the board learns nothing, not even that the election exists.
4. Answer with the election's summary, its `head` the board's current head, and with the manifest exactly as registered. `manifest` is optional in the schema so that a later version can answer a summary alone; under this version a ballot box that answers includes it.
5. Sign the response (VTI-VOT-050).

A conforming **producer** that relies on the answer — a voter's client about to cast, an auditor — **SHOULD** compare `election.manifestDigest` with the digest the community published, and refuse to proceed on a mismatch.

## Authorization

**Public board: none.** Anyone may read an election whose board is public; the manifest and board are published by design (VTI-VOT-052). **Private board:** the community and the auditors the manifest names ([conventions §4](../../../_shared/0.1/CONVENTIONS.md#4-authorization)). A voter holding a receipt reads only the inclusion of its own cast ([`voting/board/inclusion`](../../../board/inclusion/0.1/spec.md)), and learns the election from its community. Verifying the caller's `issuer`, transport identity or `proof` establishes who asked, never that they may ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); the authority is membership of that set, which consumer item 3 enforces.

## Definitions

- **`community`**, **`electionId`** — the election to read.
- **`election`** (response) — the ballot box's summary (`ElectionSummary`): `state`, `manifestDigest`, `ballotsAdmitted` (distinct tags with a counted ballot) and the current `head`.
- **`manifest`** (response) — the registered manifest.

## Request

A reader sends the request to the ballot box. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

An anonymous read of a public board: no `issuer`, no proof.

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-00000000b001",
  "type": "https://trusttasks.org/spec/voting/election/show/0.1#request",
  "recipient": "did:webvh:QmVote:vote.example.net",
  "issuedAt": "2026-11-10T08:00:00Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-00000000b0ff",
  "payload": {
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "survey-q4"
  }
}
```

## Response

The ballot box answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-00000000b002",
  "type": "https://trusttasks.org/spec/voting/election/show/0.1#response",
  "issuer": "did:webvh:QmVote:vote.example.net",
  "issuedAt": "2026-11-10T08:00:01Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-00000000b0ff",
  "payload": {
    "election": {
      "electionId": "survey-q4",
      "community": "did:webvh:QmCommunity:vtc.example.org",
      "manifestDigest": "zQmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG",
      "state": "voting",
      "ballotsAdmitted": 41,
      "head": {
        "electionId": "survey-q4",
        "treeSize": 44,
        "rootHash": "zQmT5NvUtoM5nWFfrQdVrFtvGfKFmG7AHE8P34isapyhCxX",
        "issuedAt": "2026-11-10T08:00:01Z"
      }
    },
    "manifest": {
      "electionId": "survey-q4",
      "community": "did:webvh:QmCommunity:vtc.example.org",
      "title": "Q4 member survey",
      "questions": [
        {
          "key": "priorities",
          "type": "ranking",
          "text": "Rank next year's priorities",
          "options": [
            { "key": "events", "label": "More events" },
            { "key": "docs", "label": "Better documentation" },
            { "key": "fees", "label": "Lower fees" }
          ],
          "partial": true,
          "optional": false
        },
        {
          "key": "comments",
          "type": "text",
          "text": "Anything else?",
          "maxLength": 1000,
          "optional": true
        }
      ],
      "secrecy": "open",
      "roll": { "rule": "members" },
      "registrationOpens": "2026-11-02T09:00:00Z",
      "votingOpens": "2026-11-09T09:00:00Z",
      "votingCloses": "2026-11-23T09:00:00Z",
      "revote": "replace",
      "board": { "visibility": "public", "auditors": [] },
      "ballotBox": "did:webvh:QmVote:vote.example.net",
      "binding": false,
      "allowEarlyClose": true,
      "resultsBeforeClose": true,
      "textReleaseThreshold": 20
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-10T08:00:01Z",
    "verificationMethod": "did:webvh:QmVote:vote.example.net#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z4kP9wQmExampleBallotBoxSignatureOverTheHead"
  }
}
```

## Security & Privacy

### Data carried

The election's public description and the board's counters: nothing about any voter. `ballotsAdmitted` and `treeSize` are what a public board already shows anyone who reads it. On a private board they are as private as the board, which is why the same rule governs both.

### Correlation

A reader of a public board need not identify itself: `issuer` is optional, and the ballot box's identifier scope is `public` because every voter and auditor must reach the one ballot box the manifest names. A voter who reads the election from the address it later casts from links nothing to its ballot that the cast does not already show, but a client **SHOULD NOT** send this read and the cast in one exchange, for the same timing reason that keeps a cast apart from ticket issuance ([conventions §6](../../../_shared/0.1/CONVENTIONS.md#6-privacy)). Repeated reads show interest in an election, no more.

### Retention

Transient at the ballot box. A reader who keeps the signed response keeps evidence of the head the ballot box showed it at that moment, which a later [`voting/board/consistency`](../../../board/consistency/0.1/spec.md) proof holds the board to.

### Consent/purpose

To let voters and auditors see the election they are taking part in or checking. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
