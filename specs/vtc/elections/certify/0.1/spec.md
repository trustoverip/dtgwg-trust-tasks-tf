---
slug: vtc/elections/certify
version: "0.1"
title: "VTC Elections — Certify"
summary: "An administrator has the community verify a tallied election's board for itself — every cast's attestation, every tag counted once, the result recomputed, the board consistent with every checkpoint the community kept, no more ballots than tickets — and certify the result."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - voting
  - election
  - certification
  - recount
  - audit
  - transparency-log
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: community (the registrar)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    A certification is the community's statement that a result is the one its members cast. Who asked for it must be
    attributable after the transport has closed, beside the verification the community made.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A certification is placed in time against the board
    it verified; a replay long after must not read as a fresh verification.
sideEffects:
  level: mutating
  rationale: >-
    Records the community's certification of the election's result, with the auditors' attestations it was certified
    with. Changes nothing at the ballot box, and confers no authority by itself.
consequences:
  - "The community's members see the result as certified by the community, with the auditors' attestations beside it."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request names an election and may carry auditors' credentials about its board. The response is the result, the
    board head it counts and the certified attestations — what the board already publishes to everyone who may read it.
retention:
  class: durable
  rationale: >-
    A certified result is a governance record: the community keeps it, the head it verified and the attestations for as
    long as the result stands.
errorCodes:
  - code: vtc/elections/certify:alreadyCertified
    meaning: "The election is already certified."
    retryable: false
  - code: vtc/elections/certify:electionUnknown
    meaning: "No election with this identifier exists for this community, or the caller may not see it (CONVENTIONS §7)."
    retryable: false
  - code: vtc/elections/certify:notTallied
    meaning: "The ballot box has not published a result for the election: voting is open, the election is closed but not yet tallied, or it was cancelled."
    retryable: false
  - code: vtc/elections/certify:boardInconsistent
    meaning: "The board does not hold together: its entries do not hash to the signed head, a checkpoint the community kept is not consistent with the final head, a tag is counted more than once, or a replacement does not follow the manifest's revote rule. `details` names the first fault."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [fault]
      properties:
        fault:
          type: string
          enum: [rootMismatch, checkpointInconsistent, tagRepeated, replacementInvalid]
        index:
          type: integer
          minimum: 0
        checkpointTreeSize:
          type: integer
          minimum: 0
  - code: vtc/elections/certify:resultMismatch
    meaning: "The result recomputed from the board differs from the result the ballot box published."
    retryable: false
  - code: vtc/elections/certify:countExceedsTickets
    meaning: "The board counts more ballots than the community issued tickets for the election (VTI-VOT-072)."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [ballotsCounted, ticketsIssued]
      properties:
        ballotsCounted:
          type: integer
          minimum: 0
        ticketsIssued:
          type: integer
          minimum: 0
  - code: vtc/elections/certify:castInvalid
    meaning: "A cast on the board is not admissible: its attestation does not verify for this election and its ballot, it is under another class, its tag is not the attestation's, or its ballot does not answer the manifest (CONVENTIONS §3.7). `details.index` is its board index."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [index]
      properties:
        index:
          type: integer
          minimum: 0
  - code: vtc/elections/certify:auditAttestationInvalid
    meaning: "An entry of `auditAttestations` does not verify, is not issued by an auditor the manifest names, or does not attest this election's result at the verified head. `details.index` is its position."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [index]
      properties:
        index:
          type: integer
          minimum: 0
  - code: vtc/elections/certify:ballotBoxUnavailable
    meaning: "The ballot box did not answer a read the verification needs. Nothing was recorded; the same request may be sent again."
    retryable: true
related:
  - vtc/elections/show
  - vtc/elections/close
  - vtc/elections/tickets/issue
  - voting/result/show
  - voting/board/head
  - voting/board/read
  - voting/board/consistency
  - voting/board/inclusion
---

## Abstract

**VTC Elections — Certify** is how a community puts its name to a result. The ballot box counts; the community does not take its word for it. On an administrator's request the community reads the election's board itself and verifies it end to end — that every cast on it would have been admitted, that each ticket's tag counts once, that the result recomputed from the board is the ballot box's, that the board is the one the community watched grow, and that no more ballots were counted than tickets were issued — and only then records the result as certified, with any auditors' attestations beside it (VTI-VOT-071 – 073).

Certification grants no authority. A binding result says what the community decided; acting on it is a task of its own, under its own authority. The family's shared rules are in [`CONVENTIONS.md`](../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** answer `electionUnknown` for an election it does not hold or the caller may not see, as [`vtc/elections/show`](../../show/0.1/spec.md) does, and **MUST** refuse a requester without `vtc.elections.manage` (VTI-VOT-080), read from the signer's live access-control entry at execution time, with `permissionDenied`.
2. **MUST** refuse with `notTallied` an election the ballot box has not tallied, and with `vtc/elections/certify:alreadyCertified` one it has already certified.
3. **MUST** read, itself and from the ballot box the manifest names, the published result ([`voting/result/show`](../../../../voting/result/show/0.1/spec.md)), a signed head of the board as it stood at close ([`voting/board/head`](../../../../voting/board/head/0.1/spec.md)) and every entry under that head ([`voting/board/read`](../../../../voting/board/read/0.1/spec.md)). It **MUST NOT** take any of these from the requester. A read that does not answer is `ballotBoxUnavailable`, and nothing is recorded.

It **MUST** then verify, and refuse on the first failure (VTI-VOT-071):

4. **The board is the head's.** The entries hash, as RFC 9162 defines (CONVENTIONS §3.8), to the head's `rootHash` at its `treeSize`, and the head's document verifies against the ballot box's DID. Else `boardInconsistent` (`rootMismatch`).
5. **The board is the one the community watched.** For every checkpoint the community kept ([`vtc/elections/show`](../../show/0.1/spec.md) Conformance item 5), the final head is consistent with it, by a [`voting/board/consistency`](../../../../voting/board/consistency/0.1/spec.md) proof the community checks itself. Else `boardInconsistent` (`checkpointInconsistent`, with the checkpoint's tree size). A ballot box that removed or rewrote an entry after a checkpoint is caught here.
6. **Every cast would have been admitted.** For every cast entry, the admission check of CONVENTIONS §3.7 holds under the community's voting key — class, tag and attestation, with the ballot context rebuilt from the ballot as recorded — and the ballot names this election and manifest digest and answers the manifest's questions as the manifest admits. Else `castInvalid`, with the entry's index.
7. **Every tag counts once, and replacements follow the manifest.** Under `revote: refuse`, no two cast entries carry the same tag. Under `revote: replace`, a later cast under a tag already on the board is followed by a replacement entry naming the earlier cast and the later, and every replacement names a cast under its own `tag`. Only the last cast under each tag is counted. Else `boardInconsistent` (`tagRepeated` or `replacementInvalid`, with the index).
8. **The result is the board's.** For a sealed or tallied-encrypted election the counted ballots or totals are recomputed from the board alone: the ceremony replayed from its `trusteeStep` entries, each verified as CONVENTIONS §9.6 requires, every encrypted cast re-admitted under the key it forms, and every partial decryption re-verified (§9.7). The result recomputed from the counted ballots, the manifest and its `textReleaseThreshold` (VTI-VOT-070) is equal to the published result, and the published result's `head` is the verified head. Else `resultMismatch`.
9. **No more ballots than tickets.** The result's `ballotsCounted` does not exceed the number of tickets the community issued for the election (VTI-VOT-072). Else `countExceedsTickets`.
10. **The auditors' attestations hold** (VTI-VOT-073). Each entry of `auditAttestations` verifies as a credential, is issued by a DID in the manifest's `board.auditors`, and attests this election, its manifest digest and the verified head. Else `auditAttestationInvalid`, with its position. An auditor's attestation is evidence beside the community's own verification, never instead of it.

Then:

11. **MUST** record the certification — the result, the verified head, the time, and the attestations exactly as received — and only then answer. It **MUST** audit it (VTI-VOT-100): the requester, the election, the head's tree size and root, the result's digest and each attestation's issuer.
12. **MUST NOT** treat a certification as authority for anything else. A task that acts on a binding result is authorized by its own rules; the certified result is evidence it may consider.

## Authorization

The authority is the **`vtc.elections.manage` capability at the community** (VTI-VOT-080), read from the signer's access-control entry at execution time. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

What the community certifies rests on nothing the requester supplies: the board, the head and the result are read from the ballot box and verified by the community, and the attestations are verified against the auditors the manifest named before any member took a ticket. An administrator can ask for a certification; no administrator can make the community certify a board that does not verify. That is why the capability that creates elections also certifies them: a certification adds the community's verification, never anyone's discretion.

A certification confers no authority (VTI-VOT-080): it does not seat anyone, change a policy or spend anything. A community that acts on a binding result does so through the task that act takes, under that task's authorization, with the certification as its evidence.

## Definitions

- **`electionId`** (REQUIRED) — the election to certify.
- **`auditAttestations`** — verifiable credentials, at most 32, from auditors the manifest names, each attesting the auditor's own recount of the board.
- **`result`** (response) — the [`ElectionResult`](../../../../voting/_shared/0.1/voting.schema.json) the community verified.
- **`head`** (response) — the [`BoardHead`](../../../../voting/_shared/0.1/voting.schema.json) the result counts.
- **`certifiedAt`** (response) — when the community recorded the certification.
- **Checkpoint** — a signed board head the community read during the election and kept ([`vtc/elections/show`](../../show/0.1/spec.md)).

## Request

The administrator names the election and passes the auditors' attestations it has received. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Certify with one auditor's attestation

```json
{
  "id": "urn:uuid:1a2b3c4d-5e6f-4071-b232-a3b4c5d6e7f8",
  "type": "https://trusttasks.org/spec/vtc/elections/certify/0.1#request",
  "issuer": "did:web:carol.example",
  "recipient": "did:webvh:QmCommunity:community.example",
  "issuedAt": "2026-11-16T09:00:00Z",
  "threadId": "urn:uuid:1a2b3c4d-5e6f-4071-b232-a3b4c5d6e7f8",
  "payload": {
    "electionId": "board-2026",
    "auditAttestations": [
      {
        "@context": ["https://www.w3.org/ns/credentials/v2"],
        "type": ["VerifiableCredential"],
        "issuer": "did:web:audit.example",
        "validFrom": "2026-11-15T18:00:00Z",
        "credentialSubject": {
          "electionId": "board-2026",
          "community": "did:webvh:QmCommunity:community.example",
          "treeSize": 84,
          "rootHash": "zQmPXME1oRtoT627YKaDPDQ3PwA8tdP9rWuAAweLzqSwAWT"
        }
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:carol.example#key-1",
    "created": "2026-11-16T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z7cDExampleProofValue"
  }
}
```

The attestation is abbreviated; an auditor's credential carries its own proof, and its subject's shape is the auditor's.

## Response

The community answers with what it certified — the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Every failed check is a `trust-task-error`, and nothing is recorded.

### Certified

```json
{
  "id": "urn:uuid:2b3c4d5e-6f70-4182-8343-b4c5d6e7f809",
  "type": "https://trusttasks.org/spec/vtc/elections/certify/0.1#response",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-11-16T09:00:20Z",
  "threadId": "urn:uuid:1a2b3c4d-5e6f-4071-b232-a3b4c5d6e7f8",
  "payload": {
    "electionId": "board-2026",
    "result": {
      "electionId": "board-2026",
      "manifestDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
      "ballotsCounted": 80,
      "questions": [
        { "question": "chair", "abstained": 2, "choice": { "alice": 41, "bob": 37 } }
      ],
      "head": {
        "electionId": "board-2026",
        "treeSize": 84,
        "rootHash": "zQmPXME1oRtoT627YKaDPDQ3PwA8tdP9rWuAAweLzqSwAWT",
        "issuedAt": "2026-11-15T00:00:03Z"
      }
    },
    "head": {
      "electionId": "board-2026",
      "treeSize": 84,
      "rootHash": "zQmPXME1oRtoT627YKaDPDQ3PwA8tdP9rWuAAweLzqSwAWT",
      "issuedAt": "2026-11-15T00:00:03Z"
    },
    "certifiedAt": "2026-11-16T09:00:19Z",
    "auditAttestations": [
      {
        "@context": ["https://www.w3.org/ns/credentials/v2"],
        "type": ["VerifiableCredential"],
        "issuer": "did:web:audit.example",
        "validFrom": "2026-11-15T18:00:00Z",
        "credentialSubject": {
          "electionId": "board-2026",
          "community": "did:webvh:QmCommunity:community.example",
          "treeSize": 84,
          "rootHash": "zQmPXME1oRtoT627YKaDPDQ3PwA8tdP9rWuAAweLzqSwAWT"
        }
      }
    ]
  }
}
```

84 entries, 80 counted: four were replaced by later casts under the same tags.

### Refused: the board was rewritten after a checkpoint

```json
{
  "id": "urn:uuid:3c4d5e6f-7081-4293-9454-c5d6e7f8091a",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-11-16T09:00:20Z",
  "threadId": "urn:uuid:1a2b3c4d-5e6f-4071-b232-a3b4c5d6e7f8",
  "payload": {
    "code": "vtc/elections/certify:boardInconsistent",
    "message": "The final head is not consistent with the checkpoint kept at tree size 40.",
    "retryable": false,
    "details": { "fault": "checkpointInconsistent", "checkpointTreeSize": 40 }
  }
}
```

## Security & Privacy

### Data carried

The request is an election identifier and, optionally, auditors' credentials. The response is the result, the head it counts, and those credentials. Everything in the response is already readable by anyone who may read the board; nothing names a voter, and the community, which never handles a ballot (CONVENTIONS §2), learns from the board only what the board publishes — casts under tags it cannot link to the tickets it issued. A producer **MUST NOT** put into `ext` anything about who voted.

### Correlation

The community is declared `identifierScope: public`: it is addressed by the DID its manifests name as `community`, which its members, its ballot box and its auditors all know it by, and which every ticket and cast is bound to (CONVENTIONS §3). A pairwise identifier for it would break that binding and tell the parties nothing they could check.

The community reads the whole board here. Doing so tells it nothing about who voted: a cast carries a tag, an attestation and a ballot, and the attestation is unlinkable to the issuance the community recorded (CONVENTIONS §3). Board entries carry no time of arrival (CONVENTIONS §6.1), so the community cannot pair its issuance times with cast times either. An auditor's credential names the auditor, who the manifest already named.

### Retention

Durable. The community keeps the certification, the verified head and the attestations for as long as the result stands; a later recount needs the head to compare against. Once an election is certified the community **SHOULD** discard its per-signer issuance records for it, keeping only their count: that count is all certification needed, and the per-signer record is the one thing on the community's side that names who took part.

### Consent/purpose

The certification exists so that members can rely on a result without each reading the board, and so that anyone who doubts it can check the same board against the same head. The auditors' attestations are collected to be published beside the result and for nothing else. Whether certifying takes a second administrator or a step-up is the community's policy ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).
