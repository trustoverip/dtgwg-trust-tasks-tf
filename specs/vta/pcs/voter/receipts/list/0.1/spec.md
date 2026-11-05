---
slug: vta/pcs/voter/receipts/list
version: "0.1"
title: "VTA PCS — Voter Receipts — List"
summary: "A voter's client asks its VTA for the ballot box's receipts it keeps for one election, so that after voting closes it can check each cast against the head the community kept at close."
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords:
  - pcs
  - voting
  - election
  - receipt
  - transparency-log
  - vta
parties:
  - role: Client driving the PCS engine
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: PCS engine (the VTA)
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The answer carries each receipt's token, which on a private board authorizes reading that
    cast's place on the board. Where the transport does not already authenticate the caller, the
    proof is the only thing between those tokens and anyone who can reach the VTA.
issuedAtRequirement:
  requirement: OPTIONAL
  rationale: >-
    A read with no side effect; a replay returns what the caller may already read.
sideEffects:
  level: none
  rationale: >-
    A read of the election's record in the engine. Nothing changes.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: an election's record. Out: the receipts the voter's own client handed the engine at
    vta/pcs/voter/receipt, and the ballot box's DID — nothing the client did not once hold.
retention:
  class: transient
  rationale: >-
    The engine keeps nothing of the read; the receipts stay under vta/pcs/voter/receipt's retention.
errorCodes:
  - code: vta/pcs/voter/receipts/list:notFound
    meaning: "The engine records no election in this context and community for which it accepted a ticket, or no longer keeps it."
    retryable: false
related:
  - vta/pcs/voter/receipt
  - vta/pcs/voter/read
  - voting/board/inclusion
  - vtc/elections/show
  - vtc/elections/tombstones/show
---

## Abstract

The **VTA PCS — Voter Receipts — List** Trust Task gives a voter's client back the receipts its VTA keeps for one election. The client hands each receipt to the VTA at [`vta/pcs/voter/receipt`](../../../receipt/0.1/spec.md) the moment the ballot box answers a cast, before the head that will close the election exists; once voting closes, the head the community kept at close is in [`vtc/elections/show`](../../../../../../vtc/elections/show/0.6/spec.md) — or, once the board the election was opened on is deleted, in [`vtc/elections/tombstones/show`](../../../../../../vtc/elections/tombstones/show/0.1/spec.md) — and the client needs each receipt's `leafIndex` and `receiptToken` to fetch the cast's [`voting/board/inclusion`](../../../../../../voting/board/inclusion/0.1/spec.md) proof from the ballot box, from its own connection. This task returns them, so the client need keep nothing between the cast and the check, and a voter who voted from one device can check from another. The family's shared rules are in [`CONVENTIONS.md`](../../../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the [voting conventions](../../../../../../voting/_shared/0.1/CONVENTIONS.md) and of the [PCS engine conventions](../../../../../../vta/_shared/0.4/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Fetch each cast's inclusion proof from the ballot box the response names, from its own connection — never through the VTA or the community, either of which would then learn which leaf is the voter's.
2. Pass each receipt to [`vta/pcs/voter/receipt`](../../../receipt/0.1/spec.md) whole, exactly as returned here, with the checkpoint and that inclusion proof, and show the voter any `notIncluded` status (VTI-VOT-053).
3. Keep no receipt token beyond the check.

A conforming **consumer** (the PCS engine) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse a caller without standing under [Authorization](#authorization) with `permissionDenied`.
3. Refuse with `vta/pcs/voter/receipts/list:notFound` if it records no election (`contextId`, `community`, `electionId`) for which it accepted a ticket. The ticket itself need not still be held.
4. Answer with the ballot box it recorded at [`vta/pcs/voter/register`](../../../register/0.5/spec.md) and every receipt it keeps for the election, whole and exactly as received at `vta/pcs/voter/receipt`, in the order it kept them — an empty list when it keeps none.
5. Answer for this election's record only: never another election's receipts, and nothing the receipts do not already carry.

## Authorization

The authority is **standing over the context** the voter belongs to ([conventions §4](../../../../../../voting/_shared/0.1/CONVENTIONS.md#4-authorization)), expressed as the capability **`pcsVoter`**, as for [`vta/pcs/voter/receipt`](../../../receipt/0.1/spec.md), which keeps what this task returns. It is not `sign`, and a VTA **MUST NOT** require `sign` for this task. Verifying the `issuer`, the transport identity or the `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## Definitions

- **`contextId`**, **`community`**, **`electionId`** — the election's record in the engine.
- **`ballotBox`** (response) — the DID of the ballot box recorded at registration.
- **`receipts`** (response) — the ballot box's signed cast responses the engine keeps.

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a05001",
  "type": "https://trusttasks.org/spec/vta/pcs/voter/receipts/list/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-11-15T09:00:00Z",
  "payload": {
    "contextId": "personal/communities",
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026"
  }
}
```

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a05002",
  "type": "https://trusttasks.org/spec/vta/pcs/voter/receipts/list/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-11-15T09:00:01Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a05001",
  "payload": {
    "electionId": "board-2026",
    "ballotBox": "did:webvh:QmBallotBox:vote.example.net",
    "receipts": [
      {
        "id": "urn:uuid:9b2e4c71-5d3a-4f08-a6e1-0c7d8b3f2a95",
        "type": "https://trusttasks.org/spec/voting/ballot/cast/0.1#response",
        "issuer": "did:webvh:QmBallotBox:vote.example.net",
        "recipient": "did:key:z6MkpVoteCastKey2222222222222222222222222222222",
        "issuedAt": "2026-11-09T19:02:11Z",
        "threadId": "urn:uuid:3f6b9d2e-81c4-4a7f-9e05-6d1c2b8a4f70",
        "payload": {
          "electionId": "board-2026",
          "outcome": "counted",
          "leafIndex": 41,
          "entryHash": "zQmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG",
          "head": {
            "electionId": "board-2026",
            "treeSize": 42,
            "rootHash": "zQmSrPmbaUKA3ZodhzPWZnpFgcPMFWF4QsxXbkWfEptTBJd",
            "issuedAt": "2026-11-09T19:02:11Z"
          },
          "receiptToken": "z8VtQ3pXk2mJd7rLs5nWb9cYh4gFa6eUo1iZ"
        },
        "proof": {
          "type": "DataIntegrityProof",
          "cryptosuite": "eddsa-jcs-2022",
          "verificationMethod": "did:webvh:QmBallotBox:vote.example.net#key-1",
          "created": "2026-11-09T19:02:11Z",
          "proofPurpose": "assertionMethod",
          "proofValue": "z3kg…"
        }
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

In: an election's record. Out: the ballot box's DID and the receipts the client itself handed the engine — each naming a board entry, its position and a `receiptToken`.

### Correlation

The receipts pair the voter's casts with the voter inside the VTA, which acts for the voter; returning them to the voter's own client moves that pairing nowhere new. The engine's audit records only the operation, the caller, the context, the community, the election and the time, never a leaf index, an entry hash or a token (VTI-VOT-100). The inclusion read that follows reveals interest in one leaf to the ballot box, which is why the client makes it from its own connection, as it made the cast.

### Retention

Nothing is kept by this read. Receipts are kept, and swept, under [`vta/pcs/voter/receipt`](../../../receipt/0.1/spec.md)'s retention; once the engine no longer keeps the election, this task answers `notFound`.

### Consent/purpose

To let the voter check, after close, that each cast the ballot box admitted is in the head the community kept (VTI-VOT-053). This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
