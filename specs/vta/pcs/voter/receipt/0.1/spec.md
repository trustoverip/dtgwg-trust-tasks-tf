---
slug: vta/pcs/voter/receipt
version: "0.1"
title: "VTA PCS — Voter Receipt"
summary: "A voter's client hands its VTA the ballot box's receipt for a cast, and optionally a checkpoint and an inclusion proof; the VTA checks the receipt is the voter's, keeps it, and says whether the cast is in the checkpoint."
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
    The request makes the VTA store evidence the voter may later rely on against the ballot
    box, and tells the voter whether their vote is counted. Where the transport does not
    already authenticate the caller, the proof is the only thing between that evidence and
    anyone who can reach the VTA.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3, so SPEC §7.3 item 17 sets REQUIRED as the floor: a duplicate
    can only be absorbed inside a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Keeps the receipt with the election's record in the engine.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Out: a status and the cast's board position, both already on the board. No key is used:
    the receipt is recognised as the voter's by the board entry the engine built at vta/pcs/vote.
retention:
  class: durable
  rationale: >-
    The receipt is the voter's evidence that the ballot box admitted the cast; it is kept so
    the voter can check the cast against the board at close and contest an omission.
errorCodes:
  - code: vta/pcs/voter/receipt:notFound
    meaning: "The engine records no election in this context and community for which it accepted a ticket."
    retryable: false
  - code: vta/pcs/voter/receipt:receiptInvalid
    meaning: "The receipt is not a voting/ballot/cast #response for this election signed by the election's ballot box, or it acknowledges an entry that is not a cast this engine built for the voter."
    retryable: false
  - code: vta/pcs/voter/receipt:evidenceInvalid
    meaning: "The checkpoint is not a head of this election signed by its ballot box, or the inclusion proof is not for the receipt's entry and the checkpoint's tree size, or is malformed. The evidence says nothing either way; the receipt is still kept."
    retryable: false
related:
  - voting/ballot/cast
  - voting/board/head
  - voting/board/inclusion
  - vtc/elections/show
  - vta/pcs/vote
---

## Abstract

The **VTA PCS — Voter Receipt** Trust Task keeps the ballot box's receipt for a cast the voter's VTA built, and checks that the cast is on the board. The client passes the signed [`voting/ballot/cast`](../../../../../voting/ballot/cast/0.1/spec.md) response it received; the VTA verifies it is the ballot box's and acknowledges this voter's cast, and keeps it. Given a checkpoint — a signed [`voting/board/head`](../../../../../voting/board/head/0.1/spec.md), such as the one the community kept at close and shows in [`vtc/elections/show`](../../../../../vtc/elections/show/0.1/spec.md) — and a [`voting/board/inclusion`](../../../../../voting/board/inclusion/0.1/spec.md) proof, it answers whether the cast is in that head. The engine keeps receipts under its custody rules ([VTA conventions §2](../../../../../vta/_shared/0.4/CONVENTIONS.md#2-custody)). The family's shared rules are in [`CONVENTIONS.md`](../../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the [voting conventions](../../../../../voting/_shared/0.1/CONVENTIONS.md) and of the [PCS engine conventions](../../../../../vta/_shared/0.4/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Pass `receipt` and `checkpoint` whole, exactly as received — never re-serialised from a typed form, whose output need not be the bytes that were signed.
2. Fetch the inclusion proof from the ballot box itself, from its own connection, and pass both `checkpoint` and `inclusion` or neither.
3. Show the voter a `notIncluded` status, plainly and without being asked (VTI-VOT-053), and **SHOULD** fetch the inclusion proof again before the voter raises it with the community.

A conforming **consumer** (the PCS engine) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse a caller without standing under [Authorization](#authorization) with `permissionDenied`.
3. Refuse with `vta/pcs/voter/receipt:notFound` if it records no election (`contextId`, `community`, `electionId`) for which it accepted a ticket. The ticket itself need not still be held: this task is most useful after close, and the ticket is discarded at tally.
4. Refuse with `vta/pcs/voter/receipt:receiptInvalid` unless `receipt`:
   1. is a `https://trusttasks.org/spec/voting/ballot/cast/0.1#response` document whose `issuer` is the ballot box recorded at voter/register, and whose `proof` verifies, over the document as received, by a verification method of that DID;
   2. names this election in its payload's `electionId`;
   3. names an `entryHash` the engine recorded at [`vta/pcs/vote`](../../../../../vta/pcs/vote/0.1/spec.md) for this election. That entry is a cast the engine built with the voter's ticket, so it carries the voter's tag ([CONVENTIONS §3.6](../../../../../voting/_shared/0.1/CONVENTIONS.md#3-cryptographic-binding)), and no receipt for another voter's cast can pass.
5. Keep the receipt whole with the election's record — beside any earlier receipt, since each revote has its own.
6. With no `checkpoint`, answer `kept` and the receipt's leaf index.
7. With a `checkpoint` and an `inclusion`:
   1. refuse with `vta/pcs/voter/receipt:evidenceInvalid`, the receipt still kept, unless `checkpoint` is a `https://trusttasks.org/spec/voting/board/head/0.1#response` document issued by the same ballot box, whose `proof` verifies over the document as received and whose head names this election;
   2. answer `notIncluded` if the receipt's leaf index is not less than the head's `treeSize`: the ballot box admitted the cast and the head omits it;
   3. refuse with `vta/pcs/voter/receipt:evidenceInvalid` unless `inclusion.leafIndex` is the receipt's leaf index, `inclusion.treeSize` is the head's `treeSize`, and `inclusion.path` has the length RFC 9162 §2.1.3 gives for that index and size;
   4. compute the root from the entry hash the engine recorded for the cast — not from `inclusion.entryHash` — and `inclusion.path` (RFC 9162 §2.1.3.2, SHA-256), and answer `included` if it equals the head's `rootHash` and `notIncluded` otherwise.

A `notIncluded` status is evidence against the ballot box, and the engine keeps the checkpoint and inclusion that produced it with the receipt.

## Authorization

The authority is **standing over the context** the voter belongs to ([conventions §4](../../../../../voting/_shared/0.1/CONVENTIONS.md#4-authorization)). Where it is expressed as a capability, the registered value is **`pcsVoter`** — see `Capability` in [`device/_shared`](../../../../../device/_shared/0.2/device-binding.schema.json). It is not `sign`, and a VTA **MUST NOT** require `sign` for this task, nor satisfy it under `pcsVetter` or `pcsApplicant`. Verifying the `issuer`, the transport identity or the `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

## Definitions

- **`contextId`**, **`community`**, **`electionId`** — the election's record in the engine.
- **`receipt`** — the ballot box's signed `voting/ballot/cast` response.
- **`checkpoint`** — a signed `voting/board/head` response: the `document` of a checkpoint `vtc/elections/show` lists, or one the client read from the ballot box itself.
- **`inclusion`** — the audit path of `voting/board/inclusion` for the receipt's entry in the checkpoint's tree.
- **`status`** (response) — `kept`, `included` or `notIncluded`.
- **`leafIndex`** (response) — the board position the receipt assigns to the cast.

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a04001",
  "type": "https://trusttasks.org/spec/vta/pcs/voter/receipt/0.1#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-11-15T09:00:00Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a04009",
  "payload": {
    "contextId": "personal/communities",
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026",
    "receipt": {
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
    },
    "checkpoint": {
      "id": "urn:uuid:c1a7e2d4-6b38-4f91-8d05-2e9f4a7b1c63",
      "type": "https://trusttasks.org/spec/voting/board/head/0.1#response",
      "issuer": "did:webvh:QmBallotBox:vote.example.net",
      "recipient": "did:webvh:QmCommunity:vtc.example.org",
      "issuedAt": "2026-11-15T00:00:05Z",
      "payload": {
        "head": {
          "electionId": "board-2026",
          "treeSize": 213,
          "rootHash": "zQmNLei78zWmzUdBeRYh8XhKdLpsmELx8Sy8hdxqbpbYnc1",
          "issuedAt": "2026-11-15T00:00:05Z"
        }
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:webvh:QmBallotBox:vote.example.net#key-1",
        "created": "2026-11-15T00:00:05Z",
        "proofPurpose": "assertionMethod",
        "proofValue": "z3kg…"
      }
    },
    "inclusion": {
      "leafIndex": 41,
      "treeSize": 213,
      "entryHash": "zQmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG",
      "path": [
        "zQmT5NvUtoM5nWFfrQdVrFtvGfKFmG7AHE8P34isapyhCxX",
        "zQmPZ9gcCEpqKTo6aq61g2nXGUhM4iCL3ewB6LDXZCtioEB",
        "zQmUNLLsPACCz1vLxQVkXqqLX5R1X345qqfHbsf67hvA3Nn"
      ]
    }
  }
}
```

## Response

The VTA answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a04002",
  "type": "https://trusttasks.org/spec/vta/pcs/voter/receipt/0.1#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-11-15T09:00:01Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a04009",
  "payload": {
    "electionId": "board-2026",
    "status": "included",
    "leafIndex": 41
  }
}
```

## Security & Privacy

### Data carried

In: a receipt naming a board entry, its position and a `receiptToken`, and, optionally, a public head and an audit path. Out: a status and the board position. Nothing the board does not already show.

### Correlation

The receipt pairs the voter's cast with the voter inside the VTA, which acts for the voter; that is the voter's own record and leaves the VTA under no task. The engine's audit records only the operation, the caller, the context, the community, the election and the time, never a tag, a leaf index or an entry hash (VTI-VOT-100). Fetching the inclusion proof reveals interest in one leaf to the ballot box, which is why the client fetches it from its own connection, not the VTA.

### Retention

Receipts, and the evidence of any `notIncluded` status, are kept with the election's record after the ticket is discarded at tally or cancellation, for as long as the voter's principal keeps them; they hold no key and admit no further cast.

### Consent/purpose

To keep the voter's evidence that the cast was admitted, and to tell the voter whether it was counted. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
