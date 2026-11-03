---
slug: vtc/boards/ideas/decide
version: '0.1'
title: "VTC Boards — Ideas — Decide"
summary: "A facilitator accepts a proposed idea — opening an election for it from the board's template — declines it, merges it into another, or moves a declined idea back to proposed."
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords: [vtc, boards, unconference, ideas, elections]
parties:
- role: facilitator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "Accepting opens an election and declining takes an idea down; both must stay attributable to the facilitator."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed accept would be refused, but a replayed reopen could undo a later decline."
sideEffects:
  level: mutating
  rationale: "Accepting opens an election, which cannot be undone except by cancelling it; declining and merging change the idea's state."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "In: the decision and its reason. Out: the idea as decided."
retention:
  class: durable
  rationale: "The decision, its reason and the election it opened are board records."
errorCodes:
- code: vtc/boards/ideas/decide:notFound
  meaning: "No board the caller may see has this identifier (VTI-BRD-003)."
  retryable: false
- code: vtc/boards/ideas/decide:ideaNotFound
  meaning: "No idea on this board the caller may see has this identifier."
  retryable: false
- code: vtc/boards/ideas/decide:stateInvalid
  meaning: "The decision does not apply to the idea's state: `accept`, `decline` and `merge` need `proposed` (and not being opened); `reopen` needs `declined` while proposals are open."
  retryable: false
- code: vtc/boards/ideas/decide:ownIdea
  meaning: "The caller proposed this idea and another facilitator of the board exists (VTI-BRD-013)."
  retryable: false
- code: vtc/boards/ideas/decide:mergeTargetInvalid
  meaning: "`mergeInto` names the idea itself, an idea on another board, or one that is `withdrawn`, `declined` or `merged`."
  retryable: false
- code: vtc/boards/ideas/decide:reasonRequired
  meaning: "`decline` carries no `reason`."
  retryable: false
- code: vtc/boards/ideas/decide:votingClosed
  meaning: "`accept` after the board's voting window has closed."
  retryable: false
- code: vtc/boards/ideas/decide:electionRefused
  meaning: "The community or the ballot box refused the election; the idea stays `proposed` and `opening.refusal` carries the code."
  retryable: false
related:
- vtc/boards/ideas/select
- vtc/elections/create
- vtc/elections/show
---
## Abstract

The **VTC Boards — Ideas — Decide** Trust Task is how a facilitator moves a proposed idea along (VTI-BRD-013):

- `accept` opens one election for the idea from the board's template, with the idea's title and description in its manifest (VTI-BRD-031). The idea becomes `open` only once the ballot box has registered the election; until then facilitators see it `proposed` with `opening`, and the community keeps trying, as it does for every election it creates.
- `decline` takes the idea down, with a reason the board's audience sees.
- `merge` folds the idea into another on the same board.
- `reopen` moves a declined idea back to `proposed` while proposals are open.

A facilitator does not decide their own idea while another facilitator of the board exists.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

It implements the Community Boards chapter of the Verifiable Trust Infrastructure specification (VTI-BRD-001 – VTI-BRD-080). The design is recorded in the VTC boards design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-boards.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` and `ideaNotFound` as `vtc/boards/show` would; refuse a caller who is not a facilitator of the board with `permissionDenied`, and the idea's own proposer with `ownIdea` while the board has another facilitator. Where it has none, proceed and record `ownIdea` on the decision (VTI-BRD-013).
2. Refuse a decision its idea's state does not admit (`stateInvalid`), a decline without a reason (`reasonRequired`), and a merge into an idea that cannot take it (`mergeTargetInvalid`).
3. For `accept`: refuse after the voting window closed (`votingClosed`); form the manifest from the board's template as `ElectionTemplate` describes, with `electionId` `board.<boardId>.<ideaId>`, and create the election as `vtc/elections/create` would, under the community's own authority, without parking it (it is never binding). Record `opening` on the idea **before** asking the ballot box, and move the idea to `open` with its `election` only once the ballot box has registered it (VTI-BRD-031). A refusal leaves the idea `proposed` with `opening.refusal` and is answered `electionRefused`; an answer that has not come yet is answered with the idea as it stands, `opening` set, and the community's election sweeper finishes it.
4. **MUST NOT** open a second election for an idea: a repeated `accept` of an idea being opened is answered with the idea as it stands.
5. For `decline`, `merge` and `reopen`, store the decision and answer the idea; a merged idea's notes stay with it, and the idea it was merged into is named in `decision.mergedInto`.
6. Record an audit event for the decision, naming the idea by reference (VTI-BRD-080).

## Authorization

**Authority: `vtc.boards.manage` for the board**, unqualified or qualified `board:<boardId>`. Opening the election needs no `vtc.elections.manage` of the facilitator: the board's template, fixed by a holder of it, is the authority for every election accepting an idea opens (VTI-BRD-002, VTI-BRD-030). A portal key a member authorized for `vtc/boards` is never accepted for this task (VTI-BRD-051).

## Definitions

- `Idea` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `IdeaId` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).
- `Reason` — see [`board.schema.json`](../../../_shared/0.1/board.schema.json).

## Request

The facilitator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Accept an idea

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "type": "https://trusttasks.org/spec/vtc/boards/ideas/decide/0.1#request",
  "issuer": "did:example:aria",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-04T06:00:00Z",
  "payload": {
    "ideaId": "ida_Pk3Wq9Ls2Dn6Hy4B",
    "decision": "accept"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:aria#key-1",
    "created": "2026-11-04T06:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

On success the community answers with a document of type `https://trusttasks.org/spec/vtc/boards/ideas/decide/0.1#response` whose `payload` validates against `$defs.Response`. A refusal is a `trust-task-error` document carrying one of the codes above or a framework standard code.

### The idea, open for votes

```json
{
  "id": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000002",
  "type": "https://trusttasks.org/spec/vtc/boards/ideas/decide/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:aria",
  "issuedAt": "2026-11-04T06:00:00Z",
  "threadId": "urn:uuid:4b0a7c2e-9d1f-4e6a-8c3b-000000000001",
  "payload": {
    "idea": {
      "ideaId": "ida_Pk3Wq9Ls2Dn6Hy4B",
      "boardId": "brd_Uc7N2qLm4Rt8Vx1Z",
      "state": "open",
      "title": "Accessible wallet sign-in",
      "description": "The on-screen number, screen readers, and people who can't scan a code. List the barriers and pick three to fix.",
      "format": "Discussion",
      "topic": "Identity",
      "proposer": {
        "entryId": "dir_Rb8Nc3Kq7Wz2Lp5D",
        "name": "Elif Polat"
      },
      "proposedAt": "2026-11-04T02:10:00Z",
      "noteCount": 2,
      "election": {
        "electionId": "board.brd_Uc7N2qLm4Rt8Vx1Z.ida_Pk3Wq9Ls2Dn6Hy4B",
        "state": "registration"
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:community#key-1",
    "created": "2026-11-04T06:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Security & Privacy

### Data carried

In: the decision and its reason. Out: the idea, and for an accept, its election's identifier and stage.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members and administrators already know it by, and a pairwise identifier per caller would make a board's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The deciding facilitator is recorded and audited but not shown to the audience.

### Retention

The decision is kept with the idea; an opened election is kept as every election is.

### Consent/purpose

The facilitator acts for the community within the board they run, opening elections only from the template an elections manager fixed.
