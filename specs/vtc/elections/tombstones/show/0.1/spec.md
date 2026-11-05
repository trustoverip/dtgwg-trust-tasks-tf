---
slug: vtc/elections/tombstones/show
version: "0.1"
title: "VTC Elections — Tombstones — Show"
summary: "A member reads the tombstone the community keeps of an election on a deleted board — identifier, manifest digest, ballot box and the board heads it kept, each in the ballot box's signed document — so a receipt can still be checked against the community's record."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - voting
  - election
  - boards
  - receipt
  - tombstone
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
    Whether the tombstone is shown depends on who is asking: an active member, or a holder of vtc.elections.manage. The
    community can establish that from a verified proof or from a transport-authenticated sender; a proof makes the read
    attributable on every transport.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read that changes nothing; a bounded `issuedAt` keeps a captured request from being replayed indefinitely."
sideEffects:
  level: none
  rationale: "Reads one tombstone. Persists nothing."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses an election's identifier, manifest digest and ballot box, and the ballot box's signed board heads — sizes and
    hashes. Nothing names a voter, and nothing anyone wrote is carried: the title, description and roll are gone with the
    board (VTI-BRD-007).
retention:
  class: transient
  rationale: "A view of a record the community keeps until its tombstone retention passes (VTI-BRD-008)."
errorCodes:
  - code: vtc/elections/tombstones/show:electionUnknown
    meaning: "The community keeps no tombstone of this election, or the caller may not read it. These are answered identically (VTI-BRD-009)."
    retryable: false
related:
  - vtc/boards/delete
  - vtc/elections/show
  - voting/board/head
  - voting/board/consistency
---
## Abstract

The **VTC Elections — Tombstones — Show** Trust Task reads what a community keeps of an election once the board it was opened on has been deleted: its **tombstone** (VTI-BRD-007). A voter's agent checks a receipt against a board head the community kept (VTI-VOT-053); while the election exists it reads those heads from [`vtc/elections/show`](../../../show/0.6/spec.md), and after the board is deleted, from here, for as long as the community's tombstone retention keeps them (VTI-BRD-008).

Each head comes with the ballot box's signed `voting/board/head` document exactly as the community received it, so a reader verifies it against the ballot box's DID without trusting the community.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements VTI-BRD-009 of the Verifiable Trust Infrastructure specification's Community Boards chapter.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer only a caller who is an active member, or holds `vtc.elections.manage`.
2. Answer `electionUnknown` when it keeps no tombstone of `electionId`, and identically when the caller may not read it.
3. Answer the tombstone as kept: `electionId`, `manifestDigest`, `ballotBox`, `checkpoints` oldest first, each with the ballot box's document exactly as received, `deletedAt`, and `removeAt` unless it is kept for ever.
4. **MUST NOT** carry anything VTI-BRD-007 forbids it to keep: the election's title, description or roll, or any record of the tickets it issued.

A conforming **producer** (a voter's agent) **SHOULD** verify each checkpoint's `document` against `ballotBox`'s DID before relying on its `head`, and **SHOULD** check a receipt's inclusion against the last checkpoint (VTI-VOT-053).

## Authorization

**Authority: an active member of the community, or `vtc.elections.manage`**, read at execution time. A board head names an election, a size and a hash, and the ballot box's signature over them; it says nothing about who voted or how, so it is shown without the election's roll, which a tombstone no longer holds (VTI-BRD-009).

## Definitions

- **Tombstone** — what the community keeps of an election on a deleted board: its identifier, its manifest's digest, its ballot box and the board heads it kept (VTI-BRD-007).
- `Checkpoint` — a kept board head with the ballot box's signed document that carried it; the same shape as [`vtc/elections/show/0.6`](../../../show/0.6/spec.md)'s.

## Request

The member (`issuer`) sends the community (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Read a deleted board's election

```json
{
  "id": "urn:uuid:9e3b1c7a-2d4f-4a8e-b6c1-000000000001",
  "type": "https://trusttasks.org/spec/vtc/elections/tombstones/show/0.1#request",
  "issuer": "did:example:alice",
  "recipient": "did:example:community",
  "issuedAt": "2026-12-01T10:00:00Z",
  "payload": {
    "electionId": "board.brd_Uc7N2qLm4Rt8Vx1Z.ida_Pk3Wq9Ls2Dn6Hy4B"
  }
}
```

## Response

On success the community answers with a document of type `https://trusttasks.org/spec/vtc/elections/tombstones/show/0.1#response` whose `payload` validates against `$defs.Response`. A refusal is a `trust-task-error` document carrying the code above or a framework standard code.

### The tombstone, kept for twelve months

```json
{
  "id": "urn:uuid:9e3b1c7a-2d4f-4a8e-b6c1-000000000002",
  "type": "https://trusttasks.org/spec/vtc/elections/tombstones/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:alice",
  "issuedAt": "2026-12-01T10:00:01Z",
  "threadId": "urn:uuid:9e3b1c7a-2d4f-4a8e-b6c1-000000000001",
  "payload": {
    "electionId": "board.brd_Uc7N2qLm4Rt8Vx1Z.ida_Pk3Wq9Ls2Dn6Hy4B",
    "manifestDigest": "uEiDexampleManifestDigestNotReal000000000000000",
    "ballotBox": "did:example:ballot-box",
    "checkpoints": [
      {
        "head": {
          "electionId": "board.brd_Uc7N2qLm4Rt8Vx1Z.ida_Pk3Wq9Ls2Dn6Hy4B",
          "treeSize": 41,
          "rootHash": "uEiDexampleRootHashNotReal0000000000000000000000",
          "issuedAt": "2026-11-13T09:00:05Z"
        },
        "document": {
          "type": "https://trusttasks.org/spec/voting/board/head/0.4#response",
          "issuer": "did:example:ballot-box",
          "payload": {
            "head": {
              "electionId": "board.brd_Uc7N2qLm4Rt8Vx1Z.ida_Pk3Wq9Ls2Dn6Hy4B",
              "treeSize": 41,
              "rootHash": "uEiDexampleRootHashNotReal0000000000000000000000",
              "issuedAt": "2026-11-13T09:00:05Z"
            }
          }
        }
      }
    ],
    "deletedAt": "2026-11-20T09:00:01Z",
    "removeAt": "2027-11-20T09:00:01Z"
  }
}
```

## Security & Privacy

### Data carried

In: an election identifier. Out: the election's identifier, manifest digest and ballot box, and the ballot box's signed board heads.

### Correlation

The community's identifier scope is public. A reader asking for an election's tombstone discloses to the community that they care about that election, as reading `vtc/elections/show` does; the community learns nothing about whether or how they voted.

### Retention

**Transient.** The tombstone itself is kept until the community's tombstone retention passes (VTI-BRD-008).

### Consent/purpose

Letting the members an election concerned check that the ballot box counted their ballot as cast, after the board it ran on is gone.
