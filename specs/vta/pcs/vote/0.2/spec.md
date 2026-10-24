---
slug: vta/pcs/vote
version: "0.2"
title: "VTA PCS — Vote"
summary: "A voter's client hands its VTA a ballot; the VTA checks it against the election's manifest, makes the ticket's attestation over it, and returns the complete voting/ballot/cast document — signed by a did:key used for this cast alone — for the client to send."
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords:
  - pcs
  - voting
  - election
  - ballot
  - zero-knowledge
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
    The request makes the VTA spend the voter's ticket on a ballot. Where the transport does
    not already authenticate the caller, the proof is the only thing between the voter's vote
    and anyone who can reach the VTA.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3, so SPEC §7.3 item 17 sets REQUIRED as the floor: a duplicate
    can only be absorbed inside a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Records, under the election, the board leaf hash of the cast it built, so that
    voter/receipt can recognise the voter's own cast on the board. Nothing else: the one-time
    key is never stored, and the tag is recomputable from the ticket.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: true
  rationale: >-
    The VTA exercises the voter's ticket to attest, as an eligible voter of the election, to the
    ballot. In: the ballot, the voter's choices and any text they wrote. Out: the signed cast
    document, which carries that ballot, its attestation and its tag, and names no voter.
retention:
  class: durable
  rationale: >-
    The leaf hash of each cast built is kept with the election's record for as long as the
    engine keeps the voter's receipts for it, so a receipt can be checked against the board.
errorCodes:
  - code: vta/pcs/vote:notFound
    meaning: "The engine holds no ticket for this election in this context and community."
    retryable: false
  - code: vta/pcs/vote:manifestMismatch
    meaning: "The manifest's digest is not the one recorded at voter/register, the manifest names another community, or the ballot's `electionId` or `manifestDigest` is not the manifest's."
    retryable: false
  - code: vta/pcs/vote:ballotInvalid
    meaning: "The ballot does not answer the manifest's questions as the manifest admits."
    retryable: false
  - code: vta/pcs/vote:notOpen
    meaning: "The engine's clock is outside the manifest's voting window."
    retryable: false
  - code: vta/pcs/vote:parametersInvalid
    meaning: "The voting parameters cannot be decoded, or name a voting key other than the one the ticket was issued under."
    retryable: false
related:
  - voting/ballot/cast
  - vta/pcs/voter/accept
  - vta/pcs/voter/receipt
  - vta/pcs/attest
---

## Abstract

The **VTA PCS — Vote** Trust Task turns a voter's ballot into a cast. The VTA checks the ballot against the election's manifest, makes the ticket's attestation over it for the election point ([CONVENTIONS §3.6](../../../../voting/_shared/0.1/CONVENTIONS.md#3-cryptographic-binding)), and returns the complete [`voting/ballot/cast`](../../../../voting/ballot/cast/0.1/spec.md) request document, signed by a `did:key` it generated for this cast alone and never stored. The client sends that document to the ballot box from its own connection; the VTA never sends it. The ticket and keys stay in the engine under its custody rules ([VTA conventions §2](../../../../vta/_shared/0.4/CONVENTIONS.md#2-custody)). The family's shared rules are in [`CONVENTIONS.md`](../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.1

The request's `manifest` is now the [`Manifest`](../../../../voting/_shared/0.2/voting.schema.json) of `voting/_shared/0.2`, which renames three of its members. Each keeps its type (a `date-time` string), its requiredness and its meaning:

- `registrationOpens` → **`registrationOpensAt`**
- `votingOpens` → **`votingOpensAt`**
- `votingCloses` → **`votingClosesAt`**

The reason is the registry's naming consistency: a member holding an instant is named for its event with the suffix `At`. Nothing else in the payload changes.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the [voting conventions](../../../../voting/_shared/0.1/CONVENTIONS.md) and of the [PCS engine conventions](../../../../vta/_shared/0.4/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the client) **MUST**:

1. Send `castDocument` unchanged to the manifest's `ballotBox`, over the HTTPS binding or another binding the ballot box serves ([CONVENTIONS §2.4](../../../../voting/_shared/0.1/CONVENTIONS.md#2-separation)), from a connection that carries nothing else identifying the voter — never through the community's VTC, and never inside a document the voter signs.
2. Pass the ballot box's response, whole, to [`vta/pcs/voter/receipt`](../../../../vta/pcs/voter/receipt/0.1/spec.md).
3. When the ballot box's response is lost, resend the same `castDocument` (VTI-VOT-042), and not call this task again for the same ballot: the one-time key is gone, so the engine cannot re-sign the document, and a new cast of the same ballot, with a fresh attestation, is a different cast to the ballot box.

A conforming producer **SHOULD** let a random interval pass between this task and sending the cast, and **SHOULD NOT** call this task in the same exchange in which it accepted the ticket ([CONVENTIONS §6.3](../../../../voting/_shared/0.1/CONVENTIONS.md#6-privacy), VTI-VOT-033).

A conforming **consumer** (the PCS engine) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse a caller without standing under [Authorization](#authorization) with `permissionDenied`.
3. Compute `manifestDigest`, the SHA-256 multihash of the manifest's RFC 8785 (JCS) canonicalization, and refuse with `vta/pcs/vote:notFound` if it holds no ticket for (`contextId`, `community`, `manifest.electionId`).
4. Refuse with `vta/pcs/vote:manifestMismatch` (VTI-VOT-011) if `manifest.community` is not `community`, `manifestDigest` is not the digest recorded at voter/register, or the ballot's `electionId` or `manifestDigest` is not the manifest's.
5. Refuse with `vta/pcs/vote:parametersInvalid` a `votingKey` it cannot decode or that is not the one the ticket was issued under.
6. Refuse with `vta/pcs/vote:notOpen` when its clock is before the manifest's `votingOpensAt` or at or after its `votingClosesAt`.
7. Refuse with `vta/pcs/vote:ballotInvalid` (VTI-VOT-032) a ballot that does not answer the manifest as it admits — by the rules a ballot box applies at [`voting/ballot/cast`](../../../../voting/ballot/cast/0.1/spec.md):
   1. exactly one answer per question of the manifest, and none for a key the manifest does not name;
   2. each answer carries the member of its question's type — `choice`, `ranking`, `scores` or `text` — or `abstain`, which only an `optional` question admits;
   3. `choice`: options of the question only, at least `minSelections` and at most `maxSelections` of them;
   4. `ranking`: options of the question only, each once, every option unless the question is `partial`;
   5. `scores`: every option of the question exactly once, and no other key, each score within [`min`, `max`];
   6. `text`: at most the question's `maxLength` characters.
8. Build the cast ([CONVENTIONS §3.3–3.6](../../../../voting/_shared/0.1/CONVENTIONS.md#3-cryptographic-binding)): the election point `id_E`, the ballot context `app` with the ballot bytes taken as the RFC 8785 (JCS) canonicalization of `ballot`, then `attestation` and its `tag` from the ticket and the election's voter key.
9. Generate a fresh Ed25519 key pair and its `did:key`, used for this cast alone (VTI-VOT-003, [CONVENTIONS §5.1](../../../../voting/_shared/0.1/CONVENTIONS.md#5-custody-vta)), and build `castDocument`:
   1. `id` a random version 4 UUID URN; `type` `https://trusttasks.org/spec/voting/ballot/cast/0.1#request`; `issuer` that `did:key`; `recipient` the manifest's `ballotBox`; `issuedAt` the time of building; `expiresAt`, if set, the manifest's `votingClosesAt`;
   2. `payload` the `voting/ballot/cast/0.1` request payload — `community`, the manifest's `electionId` and the cast — with nothing the request to this task carried beyond what that payload requires — no `threadId`, `parentThreadId`, `ext` or other member taken from this request;
   3. `proof` a Data Integrity proof (`eddsa-jcs-2022`, `proofPurpose` `assertionMethod`) by that `did:key`.
10. Destroy the one-time private key once the proof is made. It **MUST NOT** be stored, logged or returned.
11. Record, under the election, the RFC 9162 leaf hash of the board entry `{"kind": "cast", "cast": <cast>}` ([CONVENTIONS §3.8](../../../../voting/_shared/0.1/CONVENTIONS.md#3-cryptographic-binding)), under the protection of the context's key records; and return `castDocument` and `tag`.

A consumer **MUST** refuse with `malformedRequest`, before building anything, a manifest whose `secrecy` is not `open`: its ballots are cast with [`vta/pcs/vote-encrypted`](../../vote-encrypted/0.3/spec.md), and a cleartext cast of one would disclose the ballot the election seals.

A consumer **MUST NOT** send the cast itself, to the ballot box or anyone else ([CONVENTIONS §2](../../../../voting/_shared/0.1/CONVENTIONS.md#2-separation), [§6.3](../../../../voting/_shared/0.1/CONVENTIONS.md#6-privacy)): the VTA is reachable as the voter's, and a cast that left from it would be paired with the voter by whoever watched it leave. It **SHOULD NOT** build a cast in reply to a request whose `threadId` is that of the `vta/pcs/voter/accept` that took the ticket (VTI-VOT-033).

**Revoting.** Calling this task again with another ballot builds another cast under the same ticket, so with the same `tag`, and a new one-time key. Whether a later cast replaces an earlier one or is refused is the manifest's `revote` rule, applied by the ballot box (VTI-VOT-041); the engine does not refuse a revote.

## Authorization

The authority is **standing over the context** the voter belongs to ([conventions §4](../../../../voting/_shared/0.1/CONVENTIONS.md#4-authorization)). Where it is expressed as a capability, the registered value is **`pcsVoter`** — see `Capability` in [`device/_shared`](../../../../device/_shared/0.2/device-binding.schema.json). It is not `sign`, and a VTA **MUST NOT** require `sign` for this task, nor satisfy it under `pcsVetter` or `pcsApplicant`: the one-time key signs only the document the engine built, never bytes the caller supplies ([VTA conventions §2.4](../../../../vta/_shared/0.4/CONVENTIONS.md#2-custody)). Verifying the `issuer`, the transport identity or the `proof` establishes who asked ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)), never that they may.

Casting a vote is the most consequential act in this family, and a VTA's own policy may place a further gate on it. That gate is the VTA's to choose ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).

## Definitions

- **`contextId`**, **`community`** — with the manifest's `electionId`, the election's record in the engine.
- **`parameters`** — the community's published voting parameters.
- **`manifest`** — the election's manifest.
- **`ballot`** — the voter's ballot (`Ballot`).
- **`castDocument`** (response) — the signed `voting/ballot/cast/0.1` request document to send.
- **`tag`** (response) — the cast's tag. The client may use it to find the voter's casts on the board.

## Request

The client sends the request to its VTA. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a03001",
  "type": "https://trusttasks.org/spec/vta/pcs/vote/0.2#request",
  "issuer": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "recipient": "did:webvh:QmVta:vta.example.com",
  "issuedAt": "2026-11-09T18:20:00Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a03009",
  "payload": {
    "contextId": "personal/communities",
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "parameters": {
      "suite": "ps-ddh-bls12381",
      "votingKey": "z6MkvVotingKey1abcXYZ"
    },
    "manifest": {
      "electionId": "board-2026",
      "community": "did:webvh:QmCommunity:vtc.example.org",
      "title": "Board election 2026",
      "questions": [
        {
          "key": "chair",
          "type": "choice",
          "text": "Who should chair the board?",
          "options": [
            { "key": "ana", "label": "Ana" },
            { "key": "ben", "label": "Ben" }
          ],
          "minSelections": 1,
          "maxSelections": 1,
          "optional": false
        },
        {
          "key": "budget",
          "type": "choice",
          "text": "Approve the 2027 budget?",
          "options": [
            { "key": "yes" },
            { "key": "no" }
          ],
          "minSelections": 1,
          "maxSelections": 1,
          "optional": true
        }
      ],
      "secrecy": "open",
      "roll": { "rule": "members" },
      "registrationOpensAt": "2026-11-01T00:00:00Z",
      "votingOpensAt": "2026-11-08T00:00:00Z",
      "votingClosesAt": "2026-11-15T00:00:00Z",
      "revote": "replace",
      "board": { "visibility": "public", "auditors": [] },
      "ballotBox": "did:webvh:QmBallotBox:vote.example.net",
      "binding": true,
      "allowEarlyClose": false,
      "resultsBeforeClose": false,
      "textReleaseThreshold": 10
    },
    "ballot": {
      "electionId": "board-2026",
      "manifestDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
      "answers": [
        { "question": "chair", "choice": ["ana"] },
        { "question": "budget", "abstain": true }
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
  "id": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a03002",
  "type": "https://trusttasks.org/spec/vta/pcs/vote/0.2#response",
  "issuer": "did:webvh:QmVta:vta.example.com",
  "recipient": "did:key:z6MkfClientWalletExampleKey1111111111111111111",
  "issuedAt": "2026-11-09T18:20:01Z",
  "threadId": "urn:uuid:7c1d0e5a-0000-4000-8000-000000a03009",
  "payload": {
    "castDocument": {
      "id": "urn:uuid:3f6b9d2e-81c4-4a7f-9e05-6d1c2b8a4f70",
      "type": "https://trusttasks.org/spec/voting/ballot/cast/0.1#request",
      "issuer": "did:key:z6MkpVoteCastKey2222222222222222222222222222222",
      "recipient": "did:webvh:QmBallotBox:vote.example.net",
      "issuedAt": "2026-11-09T18:20:01Z",
      "expiresAt": "2026-11-15T00:00:00Z",
      "payload": {
        "community": "did:webvh:QmCommunity:vtc.example.org",
        "electionId": "board-2026",
        "cast": {
          "ballot": {
            "electionId": "board-2026",
            "manifestDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
            "answers": [
              { "question": "chair", "choice": ["ana"] },
              { "question": "budget", "abstain": true }
            ]
          },
          "tag": "zTag8RkF3vQyW2mXo9sJpL4tBcN7dE1aHuG",
          "attestation": "zAtt5nVhQ2wK8rTyP4mLs9XcB3jD6fGzE1uN"
        }
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:key:z6MkpVoteCastKey2222222222222222222222222222222#z6MkpVoteCastKey2222222222222222222222222222222",
        "created": "2026-11-09T18:20:01Z",
        "proofPurpose": "assertionMethod",
        "proofValue": "z3kg…"
      }
    },
    "tag": "zTag8RkF3vQyW2mXo9sJpL4tBcN7dE1aHuG"
  }
}
```

## Security & Privacy

### Data carried

In: the ballot — the voter's choices, and any text they wrote — with the manifest and parameters, which are public to the voter. Out: the cast document, which carries the same ballot, the attestation and the tag, signed by a key that names nobody. It carries no member, identifier or timing that the request to this task carried beyond the time of building.

### Correlation

The `did:key`, the document `id` and the attestation are fresh for each cast; the tag is the same for every cast of this ticket in this election and unlinkable across elections ([CONVENTIONS §1](../../../../voting/_shared/0.1/CONVENTIONS.md#1-terms)). What could pair a cast with its voter is the path it travels and when, which is why the client sends it, from its own connection and after a random interval ([CONVENTIONS §6](../../../../voting/_shared/0.1/CONVENTIONS.md#6-privacy)). The VTA holds the voter's ballot and leaf hash in its own custody — it acts for the voter, so that is the voter's own record — but its audit records only the operation, the caller, the context, the community, the election and the time, and never a tag, a ballot, a cast or a leaf hash (VTI-VOT-100).

### Retention

The one-time key is destroyed once the proof is made. The leaf hash of each cast is kept with the election's record for as long as the engine keeps the voter's receipts for it ([`vta/pcs/voter/receipt`](../../../../vta/pcs/voter/receipt/0.1/spec.md)). The ballot itself is not kept.

### Consent/purpose

To cast the voter's ballot in the election. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
