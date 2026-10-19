---
slug: vtc/elections/show
version: "0.1"
title: "VTC Elections — Show"
summary: "A member or an administrator reads one election of the community whole — its manifest and digest, state, voting parameters, the disclosure a member reads before requesting a ticket, the board checkpoints the community kept and, once certified, the certified result."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - voting
  - election
  - poll
  - registrar
  - disclosure
  - certification
parties:
  - role: member or administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: community (the registrar)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    Whether the election exists for the caller, and whether the answer carries the count of tickets issued, depend on who
    is asking. The community can establish that from a verified proof or from a transport-authenticated sender; a proof
    makes the read attributable on every transport.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read that changes nothing; a bounded `issuedAt` keeps a captured request from being replayed indefinitely.
sideEffects:
  level: none
  rationale: "Reads one election's record. Persists nothing."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses one election's manifest, the community's voting verification key, the community's disclosure about how the
    election is run, the ballot box's signed heads, the certified result once there is one, and — to an administrator
    only — how many tickets were issued. Nothing names a voter.
retention:
  class: transient
  rationale: "A view of a record the community keeps; the caller refetches as the election moves through its states."
errorCodes:
  - code: vtc/elections/show:electionUnknown
    meaning: "No election with this identifier exists for this community, or the caller may not see it. The two are answered identically (CONVENTIONS §7)."
    retryable: false
related:
  - vtc/elections/list
  - vtc/elections/create
  - vtc/elections/certify
  - vtc/elections/tickets/issue
  - voting/election/show
  - voting/board/head
  - voting/board/consistency
  - voting/result/show
  - vta/pcs/voter/register
---

## Abstract

**VTC Elections — Show** reads one election of a community whole. It is what a member's client opens before requesting a ticket: the manifest says what is asked and the manifest digest is what the ticket request names; the **disclosure** says who holds and reads the board, whether the community also runs the ballot box, and that the system cannot protect a voter from being made to vote a certain way (VTI-VOT-090); the **voting parameters** are the key the ticket will be issued under, which the member's VTA needs ([`vta/pcs/voter/register`](../../../../vta/pcs/voter/register/0.1/spec.md)). After voting it is where the community's **checkpoints** of the board and its **certified result** are read.

It is the read-one half of a split pair with [`vtc/elections/list`](../../list/0.1/spec.md). The family's shared rules are in [`CONVENTIONS.md`](../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** answer `electionUnknown` both for an `electionId` it holds no election under and for one the caller may not see, identically in code, `details` and message. Who may see an election is as [`vtc/elections/list`](../../list/0.1/spec.md) Conformance items 1 and 2 define: a caller holding `vtc.elections.manage` (VTI-VOT-080) sees every election; any other caller sees one whose roll admits them now, once it is registered.
2. **MUST** return the manifest exactly as registered, and `manifestDigest` computed over it.
3. **MUST** always carry `disclosure` and `parameters` (VTI-VOT-090, VTI-VOT-020), generated from the manifest and the community's configuration — never written by hand — so that a member has read both before any ticket is requested. `disclosure.sameOperator` **MUST** be true whenever the party operating the community's VTC also operates the ballot box.
4. **MUST** carry `ticketsIssued` to a caller holding `vtc.elections.manage`, and **MUST NOT** carry it to any other caller.
5. **MUST** carry each checkpoint's `document` exactly as the ballot box sent it, proof included, and **MUST NOT** carry a checkpoint whose document did not verify against the ballot box's DID when it was read. A community **SHOULD** keep a checkpoint when voting opens, at intervals while voting is open, and when voting closes; certification checks the board against every one ([`vtc/elections/certify`](../../certify/0.1/spec.md), VTI-VOT-071).
6. **MUST** carry `certified` once, and only once, [`vtc/elections/certify`](../../certify/0.1/spec.md) has recorded a certification. A result the ballot box published but the community has not certified is read from the ballot box ([`voting/result/show`](../../../../voting/result/show/0.1/spec.md)), never here.

A conforming **producer** — a member's client about to request a ticket — **SHOULD** show the member `disclosure` before requesting one, and **MUST** request the ticket under the `manifestDigest` read here.

## Authorization

The authority is **the caller's standing toward this election**: being a party its roll admits, or holding `vtc.elections.manage`, read from the community's own records at execution time. The proof or the transport identity says who is asking ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10); it is the lookup key for the standing, not the standing. A caller without it is answered as if the election did not exist. The count of tickets issued is an administrator's, because it is the bound certification holds the count to (VTI-VOT-072) and, while voting is open, a running measure of turnout the community chooses whether to publish.

## Definitions

- **`electionId`** (REQUIRED) — the election to read.
- **`manifest`**, **`manifestDigest`** (response) — the election's [`Manifest`](../../../../voting/_shared/0.1/voting.schema.json) and its digest.
- **`state`** (response) — a `RegistrarState`, as [`vtc/elections/list`](../../list/0.1/spec.md) defines it.
- **`parameters`** (response) — the community's [`VotingParameters`](../../../../voting/_shared/0.1/voting.schema.json).
- **`disclosure`** (response) — the community's [`Disclosure`](../../../../voting/_shared/0.1/voting.schema.json) for this election.
- **`checkpoints`** (response) — each a `Checkpoint`: a [`BoardHead`](../../../../voting/_shared/0.1/voting.schema.json) and the ballot box's signed [`voting/board/head`](../../../../voting/board/head/0.1/spec.md) response document that carried it, kept whole.
- **`ticketsIssued`** (response) — tickets issued for the election; administrators only.
- **`certified`** (response) — a `Certification`: the certified [`ElectionResult`](../../../../voting/_shared/0.1/voting.schema.json), when it was certified, and the auditors' attestations it was certified with.

## Request

A member's client reads the election before requesting a ticket. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Read one election

```json
{
  "id": "urn:uuid:708192a3-b4c5-4d6e-87f8-091a2b3c4d5e",
  "type": "https://trusttasks.org/spec/vtc/elections/show/0.1#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:webvh:QmCommunity:community.example",
  "issuedAt": "2026-11-02T08:00:00Z",
  "threadId": "urn:uuid:708192a3-b4c5-4d6e-87f8-091a2b3c4d5e",
  "payload": {
    "electionId": "board-2026"
  }
}
```

## Response

The community answers with the election — the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). A refusal is a `trust-task-error`.

### Registration open, one checkpoint kept

```json
{
  "id": "urn:uuid:8192a3b4-c5d6-4e7f-98a9-1a2b3c4d5e6f",
  "type": "https://trusttasks.org/spec/vtc/elections/show/0.1#response",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-11-02T08:00:01Z",
  "threadId": "urn:uuid:708192a3-b4c5-4d6e-87f8-091a2b3c4d5e",
  "payload": {
    "manifest": {
      "electionId": "board-2026",
      "community": "did:webvh:QmCommunity:community.example",
      "title": "Board election 2026",
      "questions": [
        {
          "key": "chair",
          "type": "choice",
          "text": "Who should chair the board?",
          "options": [
            { "key": "alice", "label": "Alice" },
            { "key": "bob", "label": "Bob" }
          ],
          "minSelections": 1,
          "maxSelections": 1,
          "optional": true
        }
      ],
      "secrecy": "open",
      "roll": { "rule": "members" },
      "registrationOpens": "2026-11-01T00:00:00Z",
      "votingOpens": "2026-11-08T00:00:00Z",
      "votingCloses": "2026-11-15T00:00:00Z",
      "revote": "replace",
      "board": { "visibility": "public", "auditors": ["did:web:audit.example"] },
      "ballotBox": "did:web:ballots.example",
      "binding": true,
      "allowEarlyClose": false,
      "resultsBeforeClose": false,
      "textReleaseThreshold": 10
    },
    "manifestDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
    "state": "registration",
    "parameters": {
      "suite": "ps-ddh-bls12381",
      "votingKey": "z3Xef1JY2x1s2vY6yKWfLu8Ap2vKuoZGwaQLZ2tjq8QkEZbGeZyaNgtznZ8B7N1yf14"
    },
    "disclosure": {
      "ballotBox": "did:web:ballots.example",
      "ballotBoxOperator": "Ballots Example Ltd",
      "sameOperator": false,
      "boardVisibility": "public",
      "auditors": ["did:web:audit.example"],
      "revote": "replace",
      "coercionResistance": false
    },
    "checkpoints": [
      {
        "head": {
          "electionId": "board-2026",
          "treeSize": 0,
          "rootHash": "zQmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n",
          "issuedAt": "2026-11-01T00:00:05Z"
        },
        "document": {
          "id": "urn:uuid:92a3b4c5-d6e7-4f80-a9ba-2b3c4d5e6f70",
          "type": "https://trusttasks.org/spec/voting/board/head/0.1#response",
          "issuer": "did:web:ballots.example",
          "recipient": "did:webvh:QmCommunity:community.example",
          "issuedAt": "2026-11-01T00:00:05Z",
          "threadId": "urn:uuid:a3b4c5d6-e7f8-4091-bacb-3c4d5e6f7081",
          "payload": {
            "head": {
              "electionId": "board-2026",
              "treeSize": 0,
              "rootHash": "zQmdfTbBqBPQ7VNxZEYEj14VmRuZBkqFbiwReogJgS1zR1n",
              "issuedAt": "2026-11-01T00:00:05Z"
            }
          },
          "proof": {
            "type": "DataIntegrityProof",
            "cryptosuite": "eddsa-jcs-2022",
            "verificationMethod": "did:web:ballots.example#key-1",
            "created": "2026-11-01T00:00:05Z",
            "proofPurpose": "assertionMethod",
            "proofValue": "z5BallotBoxHeadProofValue"
          }
        }
      }
    ]
  }
}
```

The checkpoint's `document` is illustrative of its shape; its payload is whatever [`voting/board/head`](../../../../voting/board/head/0.1/spec.md) defines, kept as received.

### Unknown, or not visible to the caller

```json
{
  "id": "urn:uuid:b4c5d6e7-f809-4a1b-8cdc-4d5e6f708192",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-11-02T08:00:01Z",
  "threadId": "urn:uuid:708192a3-b4c5-4d6e-87f8-091a2b3c4d5e",
  "payload": {
    "code": "vtc/elections/show:electionUnknown",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The response carries what the community publishes about one election to the parties who may take part: the manifest, the community's voting verification key, the disclosure, the ballot box's signed heads and, once certified, the result and the auditors' attestations. `ticketsIssued` goes to administrators only. Nothing names a voter: the community never holds a ballot (CONVENTIONS §2), and a checkpoint is a tree size and a hash. A community **MUST NOT** put into `ext` anything about who took a ticket.

### Correlation

The community is declared `identifierScope: public`: it is addressed by the DID its manifests name as `community`, which its members, its ballot box and its auditors all know it by, and which every ticket and cast is bound to (CONVENTIONS §3). A pairwise identifier for it would break that binding and tell the parties nothing they could check.

The community learns that a member read an election, and when. Reading the election shortly before requesting a ticket is the expected flow and tells the community nothing it does not learn from the issuance itself. A member's client **SHOULD NOT** read an election and then cast at once: the ballot box sees when casts arrive, and the community sees when reads arrive, and the two together are the timing that CONVENTIONS §6 and VTI-VOT-033 ask both parties not to supply.

### Retention

Transient. The community keeps nothing of the request beyond its ordinary request log. The checkpoints and the certification it returns are records it keeps for as long as the election's result stands, under [`vtc/elections/certify`](../../certify/0.1/spec.md).

### Consent/purpose

The read exists so that a member can decide, on the community's own disclosure, whether to take part, and so that anyone who may see the election can check the board against what the community kept. The community **SHOULD NOT** use a member's reads of an election for any other purpose, and in particular not as a measure of who intends to vote.
