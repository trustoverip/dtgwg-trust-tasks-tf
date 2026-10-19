---
slug: voting/election/close
version: "0.1"
title: "Voting — Election Close"
summary: "The community closes voting in one of its elections before the manifest's closing time, where the manifest allows an early close."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - voting
  - election
  - ballot-box
  - early-close
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
    Closing voting early ends every member's chance to vote and fixes the set of ballots the
    result counts. A recount has to be able to show that the community itself closed it, and
    when, after the transport has closed; the response carries the head the result will count,
    signed by being in a document the ballot box signs.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential under SPEC §3, so SPEC §7.3 item 17 sets REQUIRED as the floor.
sideEffects:
  level: destructive
  rationale: >-
    Irreversible: voting ends and cannot be reopened. Every cast that arrives afterwards is
    refused, and the board gains a `closedEarly` entry the result is computed after.
subjectPath: /community
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: the election's identifier and the community's optional reason. Out: the election's
    summary and the head at close.
retention:
  class: durable
  rationale: >-
    The signed request is the evidence that the community, not the ballot box, ended voting
    early; it is kept with the board.
errorCodes:
  - code: voting/election/close:electionUnknown
    meaning: "No election with this identifier exists for this community."
    retryable: false
  - code: voting/election/close:notOpen
    meaning: "Voting in the election is not open: not yet opened, already closed, or cancelled."
    retryable: false
  - code: voting/election/close:earlyCloseNotAllowed
    meaning: "The election's manifest does not allow an early close (`allowEarlyClose` is false)."
    retryable: false
related:
  - voting/election/register
  - voting/election/cancel
  - voting/result/show
  - vtc/elections/close
---

## Abstract

The **Voting — Election Close** Trust Task lets a community end voting before the manifest's `votingCloses`, where its manifest says it may (`allowEarlyClose`). The ballot box appends a `closedEarly` entry to the board, admits no further cast, and tallies the election from the board as it then stands. The community's VTC sends it from inside [`vtc/elections/close`](../../../../vtc/elections/close/0.1/spec.md). The family's shared rules are in [`CONVENTIONS.md`](../../../_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements), and the rules of the family's [conventions](../../../_shared/0.1/CONVENTIONS.md), in addition to the requirements stated here.

A conforming **producer** (the community) **MUST** sign the request as the community's DID and set `community` to it, and **MUST NOT** put a secret or a member's personal data in `reason`.

A conforming **consumer** (the ballot box) **MUST**, in this order:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` unless the proven signer is `community` (VTI-VOT-012). This comes first so that nobody but the community learns whether an election exists.
3. Refuse with `voting/election/close:electionUnknown` when no election `electionId` is registered for `community`.
4. Refuse with `voting/election/close:notOpen` unless the election is in state `voting`.
5. Refuse with `voting/election/close:earlyCloseNotAllowed` unless the manifest's `allowEarlyClose` is true.
6. Append a `LifecycleEntry` of kind `closedEarly`, enter `closed`, and from then on refuse every cast with `voting/ballot/cast:notOpen`. A cast admitted before the entry is counted; none after it is.
7. Answer with the election's summary, whose `head` includes the `closedEarly` entry.
8. Tally the election from the board up to and including that entry, as at a scheduled close.

A repeat of a close that already took effect is refused with `notOpen`: the effect is the one the first request had, and the producer reads it with [`voting/election/show`](../../show/0.1/spec.md).

## Authorization

The authority is **being the community the manifest names** ([conventions §4](../../../_shared/0.1/CONVENTIONS.md#4-authorization)), enforced by consumer item 2, and bounded by the manifest the community itself registered: `allowEarlyClose` is fixed at registration and covered by the digest every cast is bound to, so voters knew before they voted whether voting could be cut short (consumer item 5). Which administrators of the community may ask its VTC to close an election is the community's own rule ([`vtc/elections/close`](../../../../vtc/elections/close/0.1/spec.md)), outside this task. The proof establishes who signed; the authority is the equality with `community`.

## Definitions

- **`community`**, **`electionId`** — the election to close.
- **`reason`** — optional free text: why the community closes early.
- **`election`** (response) — the summary after the close; `state` is `closed`, or `tallied` if the ballot box tallied before answering.

## Request

The community sends the request to the election's ballot box. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-00000000c001",
  "type": "https://trusttasks.org/spec/voting/election/close/0.1#request",
  "issuer": "did:webvh:QmCommunity:vtc.example.org",
  "recipient": "did:webvh:QmVote:vote.example.net",
  "issuedAt": "2026-11-20T12:00:00Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-00000000c0ff",
  "payload": {
    "community": "did:webvh:QmCommunity:vtc.example.org",
    "electionId": "survey-q4",
    "reason": "Every member has responded."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-20T12:00:00Z",
    "verificationMethod": "did:webvh:QmCommunity:vtc.example.org#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z2bN7rTqExampleCommunitySignatureOverTheClose"
  }
}
```

## Response

The ballot box answers with the payload under `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### Example

```json
{
  "id": "urn:uuid:5a0e2c11-0000-4000-8000-00000000c002",
  "type": "https://trusttasks.org/spec/voting/election/close/0.1#response",
  "issuer": "did:webvh:QmVote:vote.example.net",
  "recipient": "did:webvh:QmCommunity:vtc.example.org",
  "issuedAt": "2026-11-20T12:00:01Z",
  "threadId": "urn:uuid:5a0e2c11-0000-4000-8000-00000000c0ff",
  "payload": {
    "election": {
      "electionId": "survey-q4",
      "community": "did:webvh:QmCommunity:vtc.example.org",
      "manifestDigest": "zQmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG",
      "state": "closed",
      "ballotsAdmitted": 87,
      "head": {
        "electionId": "survey-q4",
        "treeSize": 96,
        "rootHash": "zQmXoypizjW3WknFiJnKLwHCnL72vedxjQkDDP1mXWo6uco",
        "issuedAt": "2026-11-20T12:00:01Z"
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-20T12:00:01Z",
    "verificationMethod": "did:webvh:QmVote:vote.example.net#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z6mW3pYkExampleBallotBoxSignatureOverTheClosingHead"
  }
}
```

## Security & Privacy

### Data carried

The election's identifier and, optionally, `reason`: free text the community writes and signs. The ballot box keeps it with the signed request and shows it, where it shows it at all, attributed to the community; it is never written to the board, which records only that voting closed early, and never when relative to any cast beyond the board's own order. A producer **MUST NOT** put a secret or a member's personal data in it.

### Correlation

None about voters. Both parties use public identifiers (identifier scope `public`): the community is named by the manifest, and the ballot box is the one the manifest names.

### Retention

Durable: the signed close is evidence, for any recount, that the community ended voting and that the ballot box did not do so on its own account. It is kept as long as the board.

### Consent/purpose

To end voting early where the community told voters, in the manifest, that it might. This section is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare whether consent, approval or a step-up is required.
