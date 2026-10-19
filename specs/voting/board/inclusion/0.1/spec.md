---
slug: voting/board/inclusion
version: "0.1"
title: "Voting — Board Inclusion"
summary: "Proves that one board entry — typically the voter's own cast — is in the board as a given signed head commits to it, with a receipt token that authorizes exactly that leaf on a private board."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - voting
  - board
  - inclusion-proof
  - receipt
  - merkle
  - transparency-log
parties:
  - role: Reader (a voter checking its receipt, or anyone allowed to read the board)
    requirement: OPTIONAL
    member: issuer
    identifierScope: any
  - role: Ballot box (voting service)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  request: OPTIONAL
  response: RECOMMENDED
  rationale: >-
    A voter checking its own cast on a private board is authorized by its receipt token, not by
    who it is, and should not have to sign anything that could identify it; the community and
    auditors may sign so a private board can answer them without a token. The response needs no
    proof to be trusted — the path either hashes to a head the reader holds signed, or it does
    not — and a proof on it only attributes a failed path to the ballot box.
sideEffects:
  level: none
  rationale: >-
    A read. Nothing at the ballot box changes.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a leaf index and, on a private board, a receipt token — a bearer value that names no one.
    Out: hashes. No entry content and nothing about any voter.
retention:
  class: transient
  rationale: >-
    The ballot box keeps nothing of a read beyond its ordinary request log, and keeps no record of
    which tokens were presented.
errorCodes:
  - code: voting/board/inclusion:electionUnknown
    meaning: "No election with this identifier exists for this community, or the caller may not see it — on a private board, including a receipt token that was not issued for this leaf."
    retryable: false
  - code: voting/board/inclusion:outOfRange
    meaning: "`treeSize` is greater than the board's current size, or `leafIndex` is not below `treeSize`."
    retryable: false
related:
  - voting/ballot/cast
  - voting/board/head
  - voting/board/consistency
  - vta/pcs/voter/receipt
---

## Abstract

The **Voting — Board Inclusion** Trust Task returns an RFC 9162 inclusion proof that the entry at one index is in the board of a given size. A voter uses it to check that its cast, at the `leafIndex` and with the `entryHash` its receipt names, is still in the board as later heads commit to it — the check [`vta/pcs/voter/receipt`](../../../../vta/pcs/voter/receipt/0.1/spec.md) makes. On a private board the voter's `receiptToken` from [`voting/ballot/cast`](../../../ballot/cast/0.1/spec.md) authorizes that one leaf and nothing else. The family's shared rules are in [`CONVENTIONS.md`](../../../_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** checking its own cast **SHOULD** send the request without an `issuer` and without a proof, or from a fresh `did:key`, and **MUST NOT** send it from the voter's membership DID or any identifier the voter uses elsewhere: a signed inclusion request for one leaf would pair that identifier with that cast (VTI-VOT-002).

A conforming **consumer** (the ballot box) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `voting/board/inclusion:electionUnknown` when no election `electionId` is registered for `community`.
3. On a private board, refuse with `voting/board/inclusion:electionUnknown` — in the same form — unless the proven caller is the community or a DID in the manifest's `board.auditors`, or `receiptToken` is the token the ballot box issued in the receipt for the cast at `leafIndex` (VTI-VOT-052). A token authorizes exactly the leaf it was issued for, at any `treeSize`; it remains valid for that leaf after the cast is replaced. A ballot box compares tokens in constant time.
4. Refuse with `voting/board/inclusion:outOfRange` when `treeSize` exceeds the board's current size or `leafIndex` is not below `treeSize`.
5. Answer with `leafIndex`, `treeSize`, the RFC 9162 leaf hash of the entry at `leafIndex` as `entryHash`, and its RFC 9162 §2.1.3 inclusion proof in the tree of `treeSize` entries as `path` ([conventions §3.8](../../../_shared/0.1/CONVENTIONS.md#3-cryptographic-binding)).

A conforming **producer** that relies on the answer **MUST** verify `path` against the root hash of a signed head of size `treeSize` that it holds, and **MUST** check that `entryHash` is the one its receipt names. On a private board a voter holds the head in its receipt and later heads its community published; it cannot fetch heads itself.

## Authorization

**Public board: none** — anyone may ask for any leaf's inclusion. **Private board:** the community and the manifest's auditors for any leaf, and **possession of the leaf's receipt token** for that one leaf ([conventions §4](../../../_shared/0.1/CONVENTIONS.md#4-authorization)), enforced by consumer item 3. The token is a bearer credential by design: the voter must be able to use it without identifying itself, so its possession is the whole authority. What it grants is deliberately small — hashes along one path, no entry content — so a leaked token discloses that the leaf exists, which its holder already knew. Verifying the caller's `issuer`, transport identity or `proof` establishes who asked, never that they may ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)).

## Definitions

- **`community`**, **`electionId`** — the board.
- **`leafIndex`** — the entry to prove; for a voter, the `leafIndex` of its receipt.
- **`treeSize`** — the size of the tree to prove it in; the size of a head the reader holds.
- **`receiptToken`** — the token of the receipt for the cast at `leafIndex`.
- **`entryHash`** (response) — the entry's RFC 9162 leaf hash.
- **`path`** (response) — the inclusion proof (`MerklePath`).

## Request

A reader sends the request to the ballot box. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

A voter checks its cast on a private board against a later head its community published: no `issuer`, no proof, the receipt token.

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-000000011001",
  "type": "https://trusttasks.org/spec/voting/board/inclusion/0.1#request",
  "recipient": "did:webvh:QmVote:vote.example.net",
  "issuedAt": "2026-11-17T08:30:00Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-0000000110ff",
  "payload": {
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026",
    "leafIndex": 128,
    "treeSize": 212,
    "receiptToken": "zFz9q2Xh7bKcR4mT8vNwY3pD6sJgA5eU1kHnQtBxWr"
  }
}
```

## Response

The ballot box answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-000000011002",
  "type": "https://trusttasks.org/spec/voting/board/inclusion/0.1#response",
  "issuer": "did:webvh:QmVote:vote.example.net",
  "issuedAt": "2026-11-17T08:30:01Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-0000000110ff",
  "payload": {
    "leafIndex": 128,
    "treeSize": 212,
    "entryHash": "zQmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
    "path": [
      "zQmT5NvUtoM5nWFfrQdVrFtvGfKFmG7AHE8P34isapyhCxX",
      "zQmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG",
      "zQmPZ9gcCEpqKTo6aq61g2nXGUhM4iCL3ewB6LDXZCtioEB",
      "zQmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n"
    ]
  }
}
```

## Security & Privacy

### Data carried

A leaf index and a receipt token in; hashes out. The response carries no entry content, so it discloses nothing about any ballot even to a party holding a leaked token.

### Correlation

The request names one leaf, and on a private board only that leaf's voter (or someone they gave the token to) can ask for it. The ballot box therefore learns that whoever sends this request holds that cast's receipt; a request sent from an identifier the voter uses elsewhere would hand it the pairing the whole family is built to prevent, which is why the producer rule above forbids it and why `issuer` is optional. The network address is the remaining link: a client **SHOULD** reach the ballot box for this read as it did for the cast, and a ballot box **MUST NOT** record an address against a leaf index beyond the retention its operator discloses ([conventions §6.2](../../../_shared/0.1/CONVENTIONS.md#6-privacy)). The ballot box's identifier scope is `public`, because the voter must reach the ballot box its receipt names.

### Retention

Transient. The ballot box keeps no record of which tokens were presented or when; it keeps only the token it issued, with the leaf, for as long as the board.

### Consent/purpose

To let a voter check that its cast is still counted, and anyone allowed to read the board check any entry. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
