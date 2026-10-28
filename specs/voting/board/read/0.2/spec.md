---
slug: voting/board/read
version: "0.2"
title: "Voting — Board Read"
summary: "Reads a page of an election board's entries — casts, replacements, lifecycle entries and the result — against the current head or an earlier one, so anyone allowed to read the board can count it again."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - voting
  - board
  - transparency-log
  - audit
  - recount
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
  response: RECOMMENDED
  rationale: >-
    On a private board the ballot box needs to know who asks, which a request proof gives it
    where the transport does not. The entries need no signature of their own: a reader checks
    them against a signed head by recomputing the tree hash, and a page that does not hash to the
    head is refuted by the data itself.
sideEffects:
  level: none
  rationale: >-
    A read. Nothing at the ballot box changes.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Out: board entries — anonymous casts with their tags and ballots, replacements, lifecycle
    entries and the result. Nothing says who cast anything.
retention:
  class: transient
  rationale: >-
    The ballot box keeps nothing of a read beyond its ordinary request log.
errorCodes:
  - code: voting/board/read:electionUnknown
    meaning: "No election with this identifier exists for this community, or the caller may not see it."
    retryable: false
  - code: voting/board/read:outOfRange
    meaning: "`treeSize` is greater than the board's current size, or `from` is greater than the tree size read against."
    retryable: false
related:
  - voting/board/head
  - voting/board/inclusion
  - voting/board/consistency
  - voting/result/show
  - vtc/elections/certify
---

## Abstract

The **Voting — Board Read** Trust Task returns consecutive entries of an election's board. Walking the board from index 0 to a head's `treeSize` gives a reader everything it needs to recompute that head's root hash and, from the casts and replacements it covers, the election's result (VTI-VOT-070) — which is how the community certifies a result in [`vtc/elections/certify`](../../../../vtc/elections/certify/0.1/spec.md), and how anyone else allowed to read the board checks it. The family's shared rules are in [`CONVENTIONS.md`](../../../_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.1

The response's board entries are now the [`BoardEntry`](../../../_shared/0.2/voting.schema.json) of `voting/_shared/0.2`. A lifecycle entry's (`opened`, `closed`, `closedEarly`, `cancelled`) instant is renamed; it keeps its type (a `date-time` string), its requiredness and its meaning:

- `at` → **`occurredAt`**

The reason is the registry's naming consistency: a member recording when an event happened is named for it with the suffix `At`. Every other entry kind, and the rest of the payload, is unchanged. A lifecycle entry's leaf hash is computed over its member names, so the same event hashes differently under `0.1` and `0.2`.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **consumer** (the ballot box) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `voting/board/read:electionUnknown` when no election `electionId` is registered for `community`, and — in the same form — on a private board when the proven caller is neither the community nor a DID in the manifest's `board.auditors` (VTI-VOT-052).
3. Read against `treeSize` when it is present and against the board's current size when it is not. Refuse with `voting/board/read:outOfRange` when `treeSize` exceeds the current size or `from` exceeds the size read against.
4. Answer with that size as `treeSize` and the entries at indexes `from` up to the smaller of `from + limit` and `treeSize`, each with its `index`, in board order, exactly as recorded — so that each entry's RFC 9162 leaf hash is the one the board's heads cover ([conventions §3.8](../../../_shared/0.1/CONVENTIONS.md#3-cryptographic-binding)). A ballot box **MAY** return fewer entries than `limit` allows, but at least one when `from` < `treeSize`.
5. Set `next` to the index after the last entry returned when that index is below `treeSize`, and omit it otherwise. `from` equal to `treeSize` is an empty page with no `next`, not an error.

A conforming **producer** that relies on what it read **MUST** check it against a signed head of the same `treeSize` — by recomputing the root hash over every entry from 0, or by an inclusion proof per entry — before treating any entry as on the board. A page is not evidence on its own.

## Authorization

**Public board: none** — anyone may read every entry of a public board (VTI-VOT-052); that is what makes the election recountable by anyone. **Private board:** the community and the manifest's auditors ([conventions §4](../../../_shared/0.1/CONVENTIONS.md#4-authorization)), enforced by consumer item 2. A voter's `receiptToken` does **not** authorize this task: it authorizes the inclusion of one leaf, never the reading of other voters' ballots. Verifying the caller's `issuer`, transport identity or `proof` establishes who asked, never that they may ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)).

## Definitions

- **`community`**, **`electionId`** — the board to read.
- **`from`**, **`limit`** — the page: the first index and the most entries.
- **`treeSize`** — the size to read against; in the response, the size actually read against.
- **`entries`** (response) — `{index, entry}` pairs (`BoardEntry`).
- **`next`** (response) — the `from` of the next page.

## Request

A reader sends the request to the ballot box. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

The first three entries of a public board, read against a head the reader holds.

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-000000010001",
  "type": "https://trusttasks.org/spec/voting/board/read/0.2#request",
  "recipient": "did:webvh:QmVote:vote.example.net",
  "issuedAt": "2026-11-16T10:00:00Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-0000000100ff",
  "payload": {
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026",
    "from": 0,
    "limit": 3,
    "treeSize": 212
  }
}
```

## Response

The ballot box answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-000000010002",
  "type": "https://trusttasks.org/spec/voting/board/read/0.2#response",
  "issuer": "did:webvh:QmVote:vote.example.net",
  "issuedAt": "2026-11-16T10:00:01Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-0000000100ff",
  "payload": {
    "treeSize": 212,
    "entries": [
      {
        "index": 0,
        "entry": { "kind": "opened", "occurredAt": "2026-11-09T09:00:00Z" }
      },
      {
        "index": 1,
        "entry": {
          "kind": "cast",
          "cast": {
            "ballot": {
              "electionId": "board-2026",
              "manifestDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
              "answers": [
                { "question": "board", "choice": ["ben"] },
                { "question": "motion-7", "choice": ["yes"] }
              ]
            },
            "tag": "z7Kd3NwPqX8vYtR5mHcJbFgE2uA9zSwQ4",
            "attestation": "z3RtY8wPzKs9dHmVq5bN4cJxF2gTaE6uQ7kWnB1vMhSp3rGyCo"
          }
        }
      },
      {
        "index": 2,
        "entry": {
          "kind": "cast",
          "cast": {
            "ballot": {
              "electionId": "board-2026",
              "manifestDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
              "answers": [
                { "question": "board", "choice": ["ana", "ben"] },
                { "question": "motion-7", "choice": ["no"] }
              ]
            },
            "tag": "z9Pq4RwXk2vYtN5mHcJbFgE7uA3zSwD8",
            "attestation": "z6WnB1vMhSp3rGyCoKs9dHmVq5bN4cJxF2gTaE6uQ7kRtY8wPz"
          }
        }
      }
    ],
    "next": 3
  }
}
```

## Security & Privacy

### Data carried

Board entries: casts (ballots, tags, attestations), replacements, lifecycle entries and, once tallied, the result. No entry carries a voter's identity, a document `id`, a key, an address or a time of arrival ([conventions §6](../../../_shared/0.1/CONVENTIONS.md#6-privacy)); lifecycle entries carry the time of the lifecycle event only. Ballots' `text` answers are on the board as voters wrote them and are the voters' words, never the ballot box's or the community's: a surface that shows them attributes them to an anonymous voter, and a manifest that does not want text readable before the count uses a private board.

### Correlation

The board is designed to link exactly one thing: casts made with the same ticket share a tag, which shows a revote and lets anyone check that no ticket is counted twice. It links nothing to a voter. Board order is arrival order, so a reader who also saw when a particular voter cast could narrow which entry is theirs; that is why entries carry no time and why a client delays its cast after receiving its ticket (VTI-VOT-033). A reader of a public board need not identify itself; the ballot box's identifier scope is `public` so that every reader reaches the one board the manifest names.

### Retention

Transient at the ballot box. A reader that recounts an election keeps what it read for as long as it wants to show its recount; the board itself is the ballot box's durable record.

### Consent/purpose

To let anyone allowed to read the board count the election again and check the ballot box kept it honestly. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
