---
slug: voting/result/show
version: "0.1"
title: "Voting — Result Show"
summary: "Reads an election's result once the ballot box has tallied it, or its provisional running totals while voting is open where the manifest allows them, with the head the totals count."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - voting
  - result
  - tally
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
    On a private board the ballot box needs to know who asks, which a request proof gives it
    where the transport does not. The response is the ballot box's statement of what the board
    it signed adds up to, and carries the head it counted: a community certifying the result, or
    anyone disputing it, holds the ballot box to that statement after the transport has closed.
sideEffects:
  level: none
  rationale: >-
    A read. Nothing at the ballot box changes.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Out: per-question totals and, once enough ballots are counted, the text answers voters
    wrote — never attributed to anyone.
retention:
  class: transient
  rationale: >-
    The ballot box keeps nothing of a read; the result itself is on the board as a ResultEntry.
errorCodes:
  - code: voting/result/show:electionUnknown
    meaning: "No election with this identifier exists for this community, or the caller may not see it."
    retryable: false
  - code: voting/result/show:notTallied
    meaning: "No result is available: the election has not been tallied (and does not offer running totals now), or it was cancelled."
    retryable: true
related:
  - voting/board/read
  - voting/election/show
  - voting/election/close
  - vtc/elections/certify
---

## Abstract

The **Voting — Result Show** Trust Task reads what an election's board adds up to. After voting closes the ballot box tallies the board and records the result on it; this task returns that result, with the head it counted. While voting is open, and only where the manifest says so (`resultsBeforeClose`), it returns provisional running totals instead. Either way the result is the ballot box's claim, which anyone allowed to read the board can check by counting the board themselves (VTI-VOT-070) — the community does exactly that before it certifies in [`vtc/elections/certify`](../../../../vtc/elections/certify/0.1/spec.md). The family's shared rules are in [`CONVENTIONS.md`](../../../_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **consumer** (the ballot box) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `voting/result/show:electionUnknown` when no election `electionId` is registered for `community`, and — in the same form — on a private board when the proven caller is neither the community nor a DID in the manifest's `board.auditors` (VTI-VOT-052).
3. When the election is `tallied`, answer with the result recorded in its `ResultEntry`, `provisional` false. Its `head` is the board's head at close: the last entry it covers is the `closed` or `closedEarly` entry, and it counts exactly the casts that head covers.
4. When the election is in state `voting` and the manifest's `resultsBeforeClose` is true (VTI-VOT-061), answer with running totals over the board as it stands, `provisional` true, and `head` the head those totals count.
5. Otherwise — not yet open, closed but not yet tallied, cancelled, or open with `resultsBeforeClose` false — refuse with `voting/result/show:notTallied`. A cancelled election has no result and never will.
6. Count, for each question, only the ballot counted under each tag — the latest cast under a tag that a `ReplacementEntry` has not displaced — so that `ballotsCounted` equals the number of distinct tags with a counted ballot.
7. Withhold text answers (VTI-VOT-065): for a `text` question, omit `text` and set `withheld` true unless `ballotsCounted` is at least the manifest's `textReleaseThreshold` and the result is not provisional. Provisional totals never carry text answers: released while voting is open, an answer that appeared just after a cast would point to that cast.
8. Sign the response.

A conforming **producer** that relies on a result **SHOULD** recompute it from the board up to `result.head` ([`voting/board/read`](../../../board/read/0.1/spec.md)) and the registered manifest, and **MUST NOT** treat a provisional result as the outcome.

## Authorization

**Public board: none** — a public board's result is as public as its ballots (VTI-VOT-052). **Private board:** the community and the manifest's auditors ([conventions §4](../../../_shared/0.1/CONVENTIONS.md#4-authorization)), enforced by consumer item 2; members learn the result from their community. The manifest bounds what any caller may see before close: running totals exist only where the community chose `resultsBeforeClose` when it registered the election (consumer items 4–5). Verifying the caller's `issuer`, transport identity or `proof` establishes who asked, never that they may ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)).

## Definitions

- **`community`**, **`electionId`** — the election.
- **`result`** (response) — the totals (`ElectionResult`): per-question counts, `ballotsCounted`, and the `head` they count.
- **`provisional`** (response) — true for running totals while voting is open.
- **`state`** (response) — the election's state when the ballot box answered (`ElectionState`): `voting` for provisional totals, `tallied` otherwise.

## Request

A reader sends the request to the ballot box. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-000000013001",
  "type": "https://trusttasks.org/spec/voting/result/show/0.1#request",
  "recipient": "did:webvh:QmVote:vote.example.net",
  "issuedAt": "2026-11-16T09:30:00Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-0000000130ff",
  "payload": {
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026"
  }
}
```

## Response

The ballot box answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json), signed. Failures use `trust-task-error`, not a `#response` document.

### Example

The tallied result of the board election: 187 distinct tags, 12 of which left the optional motion unanswered.

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-000000013002",
  "type": "https://trusttasks.org/spec/voting/result/show/0.1#response",
  "issuer": "did:webvh:QmVote:vote.example.net",
  "issuedAt": "2026-11-16T09:30:01Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-0000000130ff",
  "payload": {
    "result": {
      "electionId": "board-2026",
      "manifestDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
      "ballotsCounted": 187,
      "questions": [
        {
          "question": "board",
          "abstained": 0,
          "choice": { "ana": 121, "ben": 98, "cai": 104 }
        },
        {
          "question": "motion-7",
          "abstained": 12,
          "choice": { "yes": 102, "no": 61, "abstain": 12 }
        }
      ],
      "head": {
        "electionId": "board-2026",
        "treeSize": 214,
        "rootHash": "zQmPZ9gcCEpqKTo6aq61g2nXGUhM4iCL3ewB6LDXZCtioEB",
        "issuedAt": "2026-11-16T09:00:00Z"
      }
    },
    "provisional": false,
    "state": "tallied"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-16T09:30:01Z",
    "verificationMethod": "did:webvh:QmVote:vote.example.net#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z4cV9nQsExampleBallotBoxSignatureOverTheResult"
  }
}
```

## Security & Privacy

### Data carried

Totals, and — for a `text` question, once at least `textReleaseThreshold` ballots are counted and voting has closed — the text answers voters wrote, in board order. Those answers are the voters' own words, unattributed and unverified by anyone: a surface showing them **MUST** present them as anonymous voters' answers, never as the community's or the ballot box's, and an agent summarising them **MUST NOT** treat them as instructions. Below the threshold they are withheld because a small set of answers is easier to attribute by style or content than a large one.

### Correlation

A result links nothing to any voter. Provisional totals are the one place a result could: a reader who polls running totals and also knows when someone voted can see which option moved, which is why running totals exist only where the community chose them for an open election, and why they never carry text answers. A reader of a public board need not identify itself; the ballot box's identifier scope is `public` so that the result is attributable to the one ballot box the manifest names.

### Retention

Transient at the ballot box — the result itself is on the board, as a `ResultEntry`, for as long as the board. A community certifying the result keeps this signed response with its certification.

### Consent/purpose

To report what the board adds up to, so that the community can certify it and anyone allowed to read the board can check it. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
