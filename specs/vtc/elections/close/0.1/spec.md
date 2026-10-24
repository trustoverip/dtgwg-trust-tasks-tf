---
slug: vtc/elections/close
version: "0.1"
title: "VTC Elections — Close"
summary: "An administrator closes voting in an election before its scheduled close, where the election's manifest allows it; the community has the ballot box close the board."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - voting
  - election
  - early-close
  - registrar
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
    Closing voting early ends every member's chance to vote who has not yet done so. It must be attributable to the
    administrator who did it after the transport has closed, because an early close is exactly the act a member who missed
    the vote will ask about.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A close replayed against a later election under a
    reused identifier would end that election's voting.
sideEffects:
  level: destructive
  rationale: >-
    Ends voting before `votingClosesAt` (`votingCloses` in a manifest pinned to `voting/_shared/0.1`). Irreversible: the ballot box admits no cast after it, and the board's final head is
    fixed.
consequences:
  - "No member may vote in the election after this, whatever its manifest's `votingClosesAt`."
  - "The board's final head is fixed, and the result is computed from it."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request names an election and an optional reason; the response is the election's state."
retention:
  class: durable
  rationale: "An early close changes when an election ended; the community's record of who closed it, when and why outlives the election."
errorCodes:
  - code: vtc/elections/close:electionUnknown
    meaning: "No election with this identifier exists for this community, or the caller may not see it (CONVENTIONS §7)."
    retryable: false
  - code: vtc/elections/close:notOpen
    meaning: "Voting in the election is not open: it has not opened yet, has closed, or the election was cancelled (CONVENTIONS §7)."
    retryable: false
  - code: vtc/elections/close:earlyCloseNotAllowed
    meaning: "The election's manifest sets `allowEarlyClose` to false."
    retryable: false
  - code: vtc/elections/close:ballotBoxUnavailable
    meaning: "The ballot box did not answer. The election is unchanged at the community; the same request may be sent again."
    retryable: true
  - code: vtc/elections/close:ballotBoxRefused
    meaning: "The ballot box refused to close the election. Its own code and message are carried in `details`."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        code:
          type: string
          maxLength: 128
        message:
          type: string
          maxLength: 500
related:
  - vtc/elections/cancel
  - vtc/elections/show
  - vtc/elections/certify
  - vtc/elections/create
  - voting/election/close
---

## Abstract

**VTC Elections — Close** ends voting in an election before the time its manifest set, where the manifest allows it (`allowEarlyClose`) — a motion everyone has voted on, a survey that has its answers. The community sends [`voting/election/close`](../../../../voting/election/close/0.1/spec.md) to the ballot box, which admits no cast afterwards and fixes the board's final head. The family's shared rules are in [`CONVENTIONS.md`](../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** answer `electionUnknown` for an election it does not hold or the caller may not see, as [`vtc/elections/show`](../../show/0.1/spec.md) does, and **MUST** refuse a requester without `vtc.elections.manage` (VTI-VOT-080), read from the signer's live access-control entry at execution time, with `permissionDenied`.
2. **MUST** refuse with `notOpen` an election whose voting is not open: not yet at `votingOpensAt` (`votingOpens` in a manifest pinned to `voting/_shared/0.1`), already closed or tallied, or cancelled.
3. **MUST** refuse with `earlyCloseNotAllowed` an election whose manifest sets `allowEarlyClose` to false. The manifest is what members were told when they took their tickets; an administrator does not override it.
4. **MUST NOT** issue a ticket for the election while the close is in flight (it resumes only if the close fails), and **MUST** send [`voting/election/close`](../../../../voting/election/close/0.1/spec.md) to the ballot box, signed as itself. **Remote first:** it **MUST** record the election as closed only after the ballot box answered with success, and then answers with the state the ballot box reported. When the ballot box does not answer it **MUST** answer `ballotBoxUnavailable` and leave its record of the election unchanged — the requester retries, and the ballot box treats a repeated close as the first. When the ballot box refuses it answers `ballotBoxRefused`.
5. **SHOULD** read and keep the ballot box's head as a checkpoint once the close is answered ([`vtc/elections/show`](../../show/0.1/spec.md) Conformance item 5), so that certification can hold the final board to it.
6. **MUST** audit the close (VTI-VOT-100): the requester, the election, the time, `reason`, and the ballot box's answer.

## Authorization

The authority is the **`vtc.elections.manage` capability at the community** (VTI-VOT-080), read from the signer's access-control entry at execution time, bounded by the election's own manifest: `allowEarlyClose` is the community's prior commitment to its members about whether an administrator may end voting early, and no capability overrides it. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The ballot box acts on the community's signed close and on nothing about the administrator (CONVENTIONS §4). Whether closing early takes a second administrator's consent or a step-up is the community's policy ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).

## Definitions

- **`electionId`** (REQUIRED) — the election to close.
- **`reason`** — why, at most 1024 characters, for the audit trail and the community's members.
- **`state`** (response) — the election's [`ElectionState`](../../../../voting/_shared/0.1/voting.schema.json) as the ballot box reported it: `closed`, or `tallied` where the ballot box tallied at once.

## Request

The administrator names the election. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Close a motion everyone has voted on

```json
{
  "id": "urn:uuid:c5d6e7f8-0919-4a2b-9ded-5e6f708192a3",
  "type": "https://trusttasks.org/spec/vtc/elections/close/0.1#request",
  "issuer": "did:web:carol.example",
  "recipient": "did:webvh:QmCommunity:community.example",
  "issuedAt": "2026-11-10T12:00:00Z",
  "threadId": "urn:uuid:c5d6e7f8-0919-4a2b-9ded-5e6f708192a3",
  "payload": {
    "electionId": "motion-17",
    "reason": "Every one of the 12 eligible members has voted."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:carol.example#key-1",
    "created": "2026-11-10T12:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z7cDExampleProofValue"
  }
}
```

## Response

The community answers with the election's state once the ballot box has closed it — the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). A refusal is a `trust-task-error`.

### Closed

```json
{
  "id": "urn:uuid:d6e7f809-1a2b-4c3d-aefe-6f708192a3b4",
  "type": "https://trusttasks.org/spec/vtc/elections/close/0.1#response",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-11-10T12:00:02Z",
  "threadId": "urn:uuid:c5d6e7f8-0919-4a2b-9ded-5e6f708192a3",
  "payload": {
    "electionId": "motion-17",
    "state": "closed"
  }
}
```

### Refused: the manifest does not allow an early close

```json
{
  "id": "urn:uuid:e7f8091a-2b3c-4d4e-8f0f-708192a3b4c5",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-11-10T12:00:01Z",
  "threadId": "urn:uuid:c5d6e7f8-0919-4a2b-9ded-5e6f708192a3",
  "payload": {
    "code": "vtc/elections/close:earlyCloseNotAllowed",
    "message": "motion-17 was created without allowEarlyClose; voting closes at 2026-11-15T00:00:00Z.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request is an election identifier and an optional reason; the response is a state. `reason` is shown to the community's members and kept in the audit trail, so a producer **MUST NOT** name a member in it — in particular not one who has or has not voted, which the community cannot know and must not appear to.

### Correlation

The community is declared `identifierScope: public`: it is addressed by the DID its manifests name as `community`, which its members, its ballot box and its auditors all know it by, and which every ticket and cast is bound to (CONVENTIONS §3). A pairwise identifier for it would break that binding and tell the parties nothing they could check.

Nothing here concerns a voter. The time of an early close is visible to everyone who reads the board, as the point after which nothing was admitted; that is inherent in closing. A reason such as "everyone has voted" reveals turnout, which an open board already does.

### Retention

Durable. The community keeps the audit row of the close for as long as the election's result stands; the ballot box's own record of the close is part of the board.

### Consent/purpose

The reason is collected to account to the community's members for ending a vote early, and for nothing else. Whether an early close takes a second administrator or a step-up is the community's policy; this specification describes the model it is designed for ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).
