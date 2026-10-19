---
slug: voting/election/cancel
version: "0.1"
title: "Voting — Election Cancel"
summary: "The community cancels one of its elections before voting closes; the election then admits no cast and is never tallied."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - voting
  - election
  - ballot-box
  - cancellation
parties:
  - role: Community (the election's registrar)
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: Ballot box (voting service)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    Cancelling discards the election's outcome. Members who voted, and anyone auditing the
    community later, must be able to show that the community cancelled it, not the ballot box,
    after the transport has closed. The response carries the final head, signed by being in a
    document the ballot box signs.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3, so SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: destructive
  rationale: >-
    Irreversible: the election admits no further cast and is never tallied, so the ballots
    already on its board will never produce a result.
subjectPath: /community
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: the election's identifier and the community's reason. Out: the election's summary and
    its final head.
retention:
  class: durable
  rationale: >-
    The signed request is the evidence that the community cancelled the election; it is kept
    with the board.
errorCodes:
  - code: voting/election/cancel:electionUnknown
    meaning: "No election with this identifier exists for this community."
    retryable: false
  - code: voting/election/cancel:notOpen
    meaning: "The election has already closed, been tallied, or been cancelled."
    retryable: false
related:
  - voting/election/register
  - voting/election/close
  - voting/result/show
  - vtc/elections/cancel
---

## Abstract

The **Voting — Election Cancel** Trust Task lets a community call off one of its elections at any point before voting closes — while it is registered, while tickets are being issued, or while voting is open. The ballot box appends a `cancelled` entry to the board, refuses every later cast, and never computes a result. The community's VTC sends it from inside [`vtc/elections/cancel`](../../../../vtc/elections/cancel/0.1/spec.md). The family's shared rules are in [`CONVENTIONS.md`](../../../_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the community) **MUST** sign the request as the community's DID and set `community` to it, and **MUST NOT** put a secret or a member's personal data in `reason`.

A conforming **consumer** (the ballot box) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` unless the proven signer is `community` (VTI-VOT-012). This comes first so that nobody but the community learns whether an election exists.
3. Refuse with `voting/election/cancel:electionUnknown` when no election `electionId` is registered for `community`.
4. Refuse with `voting/election/cancel:notOpen` unless the election is in state `registered`, `registration` or `voting`. An election that has closed — on schedule or early — is past cancelling: its ballots are fixed, and its result is the community's to certify or not.
5. Append a `LifecycleEntry` of kind `cancelled` and enter `cancelled`. From then on refuse every cast with `voting/ballot/cast:notOpen`, answer [`voting/result/show`](../../../result/show/0.1/spec.md) with `notTallied`, and never append a `ResultEntry`.
6. Answer with the election's summary, whose `head` includes the `cancelled` entry.

The board of a cancelled election stays readable under the same rules as before: the casts already admitted remain on it, uncounted, so that anyone may see what was cancelled.

## Authorization

The authority is **being the community the manifest names** ([conventions §4](../../../_shared/0.1/CONVENTIONS.md#4-authorization)), enforced by consumer item 2. Unlike an early close, a cancellation needs no permission in the manifest: a community may always withdraw a question it asked, and what it may not do — produce a result from part of the voting period — a cancellation does not do. Which administrators may ask the VTC to cancel is the community's own rule ([`vtc/elections/cancel`](../../../../vtc/elections/cancel/0.1/spec.md)). The proof establishes who signed; the authority is the equality with `community`.

## Definitions

- **`community`**, **`electionId`** — the election to cancel.
- **`reason`** — required free text: why the community cancels.
- **`election`** (response) — the summary after the cancellation; `state` is `cancelled`.

## Request

The community sends the request to the election's ballot box. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-00000000d001",
  "type": "https://trusttasks.org/spec/voting/election/cancel/0.1#request",
  "issuer": "did:webvh:QmCommunity:vtc.example.org",
  "recipient": "did:webvh:QmVote:vote.example.net",
  "issuedAt": "2026-11-05T15:30:00Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-00000000d0ff",
  "payload": {
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "board-2026",
    "reason": "A candidate withdrew before voting opened; the election will be re-run."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-05T15:30:00Z",
    "verificationMethod": "did:webvh:QmCommunity:vtc.example.org#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3qK8vWnExampleCommunitySignatureOverTheCancel"
  }
}
```

## Response

The ballot box answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-00000000d002",
  "type": "https://trusttasks.org/spec/voting/election/cancel/0.1#response",
  "issuer": "did:webvh:QmVote:vote.example.net",
  "recipient": "did:webvh:QmCommunity:vtc.example.org",
  "issuedAt": "2026-11-05T15:30:01Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-00000000d0ff",
  "payload": {
    "election": {
      "electionId": "board-2026",
      "community": "did:webvh:QmCommunity:vtc.example.org",
      "manifestDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
      "state": "cancelled",
      "ballotsAdmitted": 0,
      "head": {
        "electionId": "board-2026",
        "treeSize": 1,
        "rootHash": "zQmPZ9gcCEpqKTo6aq61g2nXGUhM4iCL3ewB6LDXZCtioEB",
        "issuedAt": "2026-11-05T15:30:01Z"
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-05T15:30:01Z",
    "verificationMethod": "did:webvh:QmVote:vote.example.net#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z5tR2cXbExampleBallotBoxSignatureOverTheFinalHead"
  }
}
```

## Security & Privacy

### Data carried

The election's identifier and `reason`: free text the community writes and signs. The ballot box keeps it with the signed request and shows it, where it shows it at all, attributed to the community; it is never written to the board. A producer **MUST NOT** put a secret or a member's personal data in it — "a candidate withdrew" names an event, not the person's circumstances.

### Correlation

None about voters. Both parties use public identifiers (identifier scope `public`), as in every task where the community acts on its own election.

### Retention

Durable: the signed cancellation is kept with the board, for as long as the board, as the evidence that the community withdrew the election.

### Consent/purpose

To withdraw an election the community no longer wants decided. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
