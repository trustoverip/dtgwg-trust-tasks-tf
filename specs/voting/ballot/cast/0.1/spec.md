---
slug: voting/ballot/cast
version: "0.1"
title: "Voting — Ballot Cast"
summary: "A voter's client sends one ballot, admitted by a PCS attestation and signed by a key made for this cast alone, to the election's ballot box, which records it on the board and answers with a signed receipt."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - voting
  - ballot
  - cast
  - ballot-box
  - receipt
  - nullifier
  - anonymous
  - zero-knowledge
parties:
  - role: Voter's client (signing with a did:key made for this cast alone)
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Ballot box (voting service)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The request is signed by the one-time key the cast is sent under, so the ballot box can
    address its receipt to that key and nobody can alter the cast between the voter's agent and
    the board without the signature failing; the key identifies nobody, and authority comes from
    the attestation, not this proof. The response is the voter's receipt (VTI-VOT-043): the
    ballot box's signed statement that this cast is on the board at this index under this head,
    which the voter holds against the ballot box long after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3, so SPEC §7.3 item 17 sets REQUIRED as the floor. The time is on
    the document only; it is never written to the board.
sideEffects:
  level: mutating
  rationale: >-
    Appends the cast to the election's board and counts it — under a revote policy of
    `replace`, in place of the ballot earlier counted under the same tag.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: >-
    In: a ballot — how one eligible party voted, which may include free-text answers they wrote
    — and the attestation that admits it, with nothing that says who that party is. The ballot
    is personal in content and anonymous in attribution, and the task exists to keep it that
    way. Out: the receipt — a board index, a leaf hash, a signed head and a random token.
retention:
  class: durable
  rationale: >-
    The cast is the board: it is kept, as received, for as long as the board, so that anyone
    allowed to read the board can count the election again.
errorCodes:
  - code: voting/ballot/cast:electionUnknown
    meaning: "No election with this identifier exists for this community."
    retryable: false
  - code: voting/ballot/cast:notOpen
    meaning: "Voting in the election is not open: not yet opened, already closed, or cancelled."
    retryable: false
  - code: voting/ballot/cast:manifestMismatch
    meaning: "The ballot names another election, or binds a manifest digest other than the registered one."
    retryable: false
  - code: voting/ballot/cast:ballotInvalid
    meaning: "The ballot does not answer the manifest's questions as the manifest admits."
    retryable: false
  - code: voting/ballot/cast:classMismatch
    meaning: "The attestation is not under this election's ticket class."
    retryable: false
  - code: voting/ballot/cast:alreadyVoted
    meaning: "A different ballot under this tag is counted and the manifest refuses revotes."
    retryable: false
  - code: voting/ballot/cast:castInvalid
    meaning: "The attestation does not verify for this election and this ballot, or its tag is not `tag`."
    retryable: false
related:
  - voting/board/inclusion
  - voting/board/head
  - voting/election/show
  - vta/pcs/vote
  - vta/pcs/voter/receipt
---

## Abstract

The **Voting — Ballot Cast** Trust Task is the vote. A voter's VTA builds the cast in [`vta/pcs/vote`](../../../../vta/pcs/vote/0.1/spec.md): the ballot, and a PCS attestation made with the voter's ticket that binds this ballot, this manifest and this election, and that carries a tag the same for every cast made with that ticket in this election. The voter's client sends it to the ballot box under a `did:key` made for this one cast. The ballot box checks the cast answers the manifest and that the attestation admits it, records it on the election's board, and answers with a signed receipt the voter keeps. Nothing in the exchange says who voted. The family's shared rules are in [`CONVENTIONS.md`](../../../_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the voter's client, with the cast its VTA built) **MUST**:

1. Issue and sign the document with a `did:key` generated for this cast alone (VTI-VOT-003), never with the voter's membership DID or any identifier the voter uses elsewhere, and set nothing in the document — `ext` included — that identifies the voter (VTI-VOT-002).
2. Send `cast` exactly as [`vta/pcs/vote`](../../../../vta/pcs/vote/0.1/spec.md) returned it. The attestation binds the ballot's canonical bytes, so a changed ballot is a refused cast.
3. Keep the signed response as the voter's receipt (VTI-VOT-043), and check that `head` is signed by the ballot box the manifest names and covers `leafIndex`.
4. Resend the same document when the response is lost — the one-time key that signed it is gone once `vta/pcs/vote` returns, so no other document can carry the cast ([Retry safety](#retry-safety)); **MUST NOT** ask the VTA for a new cast of the same ballot to recover a lost response.

A conforming **consumer** (the ballot box) **MUST** run the checks below in this order, cheapest first, and stop at the first that fails (VTI-VOT-045):

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), the request's proof included. The proof shows only that the document is intact under its one-time key; it is not authority.
2. Refuse with `voting/ballot/cast:electionUnknown` when no election `electionId` is registered for `community`.
3. Refuse with `voting/ballot/cast:notOpen` unless the election is in state `voting`.
4. Refuse with `voting/ballot/cast:manifestMismatch` unless `cast.ballot.electionId` is `electionId` and `cast.ballot.manifestDigest` is the registered manifest's digest.
5. Refuse with `voting/ballot/cast:ballotInvalid` unless the ballot answers the manifest as the manifest admits:
   - exactly one answer per question of the manifest, naming the question by its `key`, and no answer to a question the manifest does not ask;
   - each answer's member matches its question's type — `choice`, `ranking`, `scores` or `text` — or is `abstain`, and `abstain` only for a question whose `optional` is true;
   - a `choice` selects between `minSelections` and `maxSelections` of the question's option keys;
   - a `ranking` lists option keys of the question, each once, every option unless `partial` is true;
   - `scores` gives every option of the question a score within [`min`, `max`], and scores no other key;
   - a `text` answer is at most the question's `maxLength` characters.
6. Refuse with `voting/ballot/cast:classMismatch` unless the attestation's class encoding is that of `voter/<electionId>` ([conventions §3.7](../../../_shared/0.1/CONVENTIONS.md#3-cryptographic-binding)).
7. If a cast identical to this one — the same `tag`, ballot and attestation — is on the board, answer with that cast's receipt ([Retry safety](#retry-safety)), skipping the remaining checks; it was verified when it was admitted.
8. If a ballot under `tag` is counted and its canonical form differs from this ballot's, and the manifest's `revote` is `refuse`, refuse with `voting/ballot/cast:alreadyVoted` (VTI-VOT-041).
9. Verify the attestation per [conventions §3.7](../../../_shared/0.1/CONVENTIONS.md#3-cryptographic-binding) under the registered voting key, with the ballot context rebuilt from the ballot **as received**; refuse with `voting/ballot/cast:castInvalid` if it does not verify or its tag is not `tag`.
10. If the ballot counted under `tag` has the same canonical form as this ballot, answer with that cast's receipt and outcome `alreadyCounted`: a re-attested copy of the counted ballot changes nothing and adds nothing to the board.
11. Otherwise record the cast: append a `CastEntry` carrying `cast` as received and, when a ballot under `tag` was counted (revote `replace`), a `ReplacementEntry` naming the earlier cast's index in `replaces` and the new one's in `by` (VTI-VOT-041). Answer with outcome `counted` for the first ballot under the tag and `replaced` otherwise, a fresh `receiptToken` of 32 random bytes, and a `head` that covers every entry the cast appended.
12. Sign the response (VTI-VOT-043).

Steps 11 and its answer are atomic: a ballot box **MUST NOT** answer before the entries are durable, and **MUST NOT** leave a `CastEntry` on the board without the `ReplacementEntry` its admission required.

## Channel requirement

A ballot box **MUST** accept this task over the HTTPS binding ([bindings/https/0.2](../../../../../bindings/https/0.2/spec.md)) and **MAY** accept it over any other binding it serves (VTI-VOT-004). It **MUST NOT** require the sender to hold a session, an account, a registration with it, or any identifier but the one-time key the document is signed with: every credential a ballot box could ask for is something that would tie the cast to a voter.

The HTTPS binding is the floor because it is the one every client can reach without an identity of its own. A voter's client sending over DIDComm or TSP sends from a DID with a mediator relationship, and that DID is exactly what this task must not carry. A client **MAY** reach the ballot box through an anonymising network, and a ballot box **MUST NOT** refuse a cast for the address it comes from.

## Retry safety

The task is **idempotent** (VTI-VOT-042, [conventions §8](../../../_shared/0.1/CONVENTIONS.md#8-retry-safety)). A cast resent unchanged — under the same document `id` or a new document — is answered with outcome `alreadyCounted`, the first receipt's `leafIndex`, `entryHash` and `receiptToken`, and a current `head`; it is counted once and appended once. A client **MAY** therefore resend a cast whose response it did not receive, as often as it needs to, without asking whether the first attempt arrived.

The same holds when the cast was later replaced: a cast already on the board is never admitted again, so resending an earlier cast — by the voter or by anyone who read it off a public board — cannot undo a later revote. Only a new attestation, which only the ticket's holder can make, changes what is counted.

## Authorization

**Nobody's identity.** The authority is the cast's attestation: a valid attestation under the election's ticket class proves that its sender holds a ticket the community issued for this election, and its tag limits each ticket to one counted ballot ([conventions §4](../../../_shared/0.1/CONVENTIONS.md#4-authorization), §3.7). Consumer steps 6 and 9 enforce it. The document's proof, its `issuer` and the transport identity establish only that the document is intact under a key made for it ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); they grant nothing, and a ballot box **MUST NOT** gate a cast on them beyond step 1.

## Definitions

- **`community`**, **`electionId`** — the election the cast is for.
- **`cast`** — the ballot and its attestation (`Cast`), as [`vta/pcs/vote`](../../../../vta/pcs/vote/0.1/spec.md) built it.
- **`outcome`** (response) — `counted`, `replaced` or `alreadyCounted`.
- **`leafIndex`** (response) — the board index of this cast's `CastEntry`.
- **`entryHash`** (response) — the RFC 9162 leaf hash of that entry ([conventions §3.8](../../../_shared/0.1/CONVENTIONS.md#3-cryptographic-binding)), so the voter can check an inclusion proof without fetching the entry.
- **`head`** (response) — a signed head whose tree includes the entry.
- **`receiptToken`** (response) — 32 random bytes. On a private board it authorizes [`voting/board/inclusion`](../../../board/inclusion/0.1/spec.md) for this one leaf; on a public board it authorizes nothing anyone could not already do.

## Request

The voter's client sends the request to the ballot box the manifest names, from the one-time key. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-00000000e001",
  "type": "https://trusttasks.org/spec/voting/ballot/cast/0.1#request",
  "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "recipient": "did:webvh:QmVote:vote.example.net",
  "issuedAt": "2026-11-12T19:42:07Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-00000000e0ff",
  "payload": {
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026",
    "cast": {
      "ballot": {
        "electionId": "board-2026",
        "manifestDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
        "answers": [
          { "question": "board", "choice": ["ana", "cai"] },
          { "question": "motion-7", "abstain": true }
        ]
      },
      "tag": "z2JsW8kNqT5vYbRx4mPcHdFgE7uA3zKwQ9",
      "attestation": "z5HmVq3dKs9wPzRt8YbN4cJxF2gTaE6uQ7kWnB1vMhSp3rLyCo"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-12T19:42:07Z",
    "verificationMethod": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH#z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
    "proofPurpose": "assertionMethod",
    "proofValue": "z4Gt7nYwExampleOneTimeKeySignatureOverTheCast"
  }
}
```

## Response

The ballot box answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json), addressed to the one-time key and signed: this document is the voter's receipt. Failures use `trust-task-error`, not a `#response` document.

### Example

The first ballot under this tag.

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-00000000e002",
  "type": "https://trusttasks.org/spec/voting/ballot/cast/0.1#response",
  "issuer": "did:webvh:QmVote:vote.example.net",
  "recipient": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "issuedAt": "2026-11-12T19:42:08Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-00000000e0ff",
  "payload": {
    "electionId": "board-2026",
    "outcome": "counted",
    "leafIndex": 128,
    "entryHash": "zQmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
    "head": {
      "electionId": "board-2026",
      "treeSize": 129,
      "rootHash": "zQmT5NvUtoM5nWFfrQdVrFtvGfKFmG7AHE8P34isapyhCxX",
      "issuedAt": "2026-11-12T19:42:08Z"
    },
    "receiptToken": "zFz9q2Xh7bKcR4mT8vNwY3pD6sJgA5eU1kHnQtBxWr"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-12T19:42:08Z",
    "verificationMethod": "did:webvh:QmVote:vote.example.net#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z2Wp8dKsExampleBallotBoxSignatureOverTheReceipt"
  }
}
```

## Security & Privacy

### Data carried

A ballot, its attestation and its tag, under a key that exists for this cast alone. No member of the request or the response identifies the voter, and a ballot box **MUST NOT** ask for one (VTI-VOT-002). A ballot's `text` answers are free text the voter wrote; they are covered by the attestation, are shown — where at all — as an anonymous voter's words and never as the community's or the ballot box's, and are released in a result only once the manifest's `textReleaseThreshold` of ballots have been counted (VTI-VOT-065). A voter **SHOULD NOT** write anything in one that identifies them, since that would undo for their own ballot what the rest of the task protects.

The board entry a cast becomes carries the cast and nothing else: no time of arrival, no document `id`, no key, no address ([conventions §6](../../../_shared/0.1/CONVENTIONS.md#6-privacy)). The document's `issuedAt` is required by the framework and is seen by the ballot box, which sees the arrival time anyway; it is never written to the board.

### Correlation

**The tag is public by design.** On a public board, anyone can see every cast and its tag, and so can see that two casts were made with the same ticket — that a voter changed their vote. That is what lets anyone check that no ticket is counted twice. The tag links nothing else: it is unlinkable to the voter, to the ticket's issuance and to the same voter's tag in any other election ([conventions §1](../../../_shared/0.1/CONVENTIONS.md#1-terms)).

The one-time key's identifier scope is `pairwise`, in the strongest form: it is used for one document and its response, never again. The ballot box's identifier scope is `public`, because every voter must reach the one ballot box the manifest names. What remains linkable is the network: a ballot box **MUST NOT** record a cast's network address against its tag beyond the retention its operator discloses, and never after the election is tallied ([conventions §6.2](../../../_shared/0.1/CONVENTIONS.md#6-privacy)). Timing is the voter's to vary: a client **SHOULD** let a random interval pass between receiving the ticket and sending the cast (VTI-VOT-033), so that the registrar's issuance log and the board's order cannot be paired.

A receipt proves how its holder voted to anyone they show it to. That is inherent in an open ballot — the voter can verify their vote was counted as cast — and is why an open election offers no coercion resistance (`Disclosure.coercionResistance` is always false).

### Retention

Durable at the ballot box: the cast is the board, kept as received for as long as the board, so that the election can be counted again by anyone allowed to read it (VTI-VOT-070). The one-time key is discarded once the receipt is checked. The voter's VTA keeps the receipt ([`vta/pcs/voter/receipt`](../../../../vta/pcs/voter/receipt/0.1/spec.md)) for as long as the voter may want to check that their cast is still on the board; the `receiptToken` is a bearer credential for one leaf's inclusion on a private board and is kept with it.

### Consent/purpose

To be counted in one election, once, without saying who voted. The ballot box uses the cast only to keep the board and compute the result. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
