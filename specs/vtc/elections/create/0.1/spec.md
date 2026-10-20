---
slug: vtc/elections/create
version: "0.1"
title: "VTC Elections — Create"
summary: "An administrator creates a poll, survey or election of the community from its manifest; the community checks it, records it and registers it with the ballot box the manifest names, and a binding election first takes another administrator's consent."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - voting
  - election
  - poll
  - ballot-box
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
    Creating an election commits the community to a question, a roll and a ballot box, and a binding one decides something
    in the community's name. The creation must be attributable to its requester after the transport has closed — by the
    approvers of a parked creation, by the audit trail, and by any member later asking who put the question.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed creation of a cancelled election's manifest
    would otherwise be weighed again long after the administrator who asked for it moved on.
sideEffects:
  level: mutating
  rationale: >-
    Records an election and registers it with a ballot box, which opens ticket issuance at the manifest's
    `registrationOpens`. Recoverable before voting closes by cancelling it (vtc/elections/cancel).
consequences:
  - "From `registrationOpens`, every party the roll admits may request a ticket for this election."
  - "The ballot box the manifest names will hold the election's board and count its ballots."
  - "A manifest with `binding: true` states that the community treats the result as deciding something."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries the manifest: questions, the roll rule and any DIDs it lists by name, the schedule, and the
    ballot box and auditors. The response returns the election's identifier, the manifest's digest and its state.
retention:
  class: durable
  rationale: >-
    The election is a record of the community for as long as its result stands: the manifest is what every ticket, cast
    and certification of the election is bound to, and who created it, on whose consent, is the provenance of a decision.
errorCodes:
  - code: vtc/elections/create:electionExists
    meaning: "The community already holds an election, or a parked creation, under this electionId."
    retryable: false
  - code: vtc/elections/create:invalidManifest
    meaning: "The manifest does not satisfy the rules voting/election/register applies to one: its schedule is out of order or already past, a question or option key repeats, a selection bound is outside its options, a non-open manifest names no trustees or an open one does, or a rule the schema cannot state does not hold. `details.reason` names the rule."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [reason]
      properties:
        reason:
          type: string
          maxLength: 500
  - code: vtc/elections/create:unsupportedSecrecy
    meaning: "`manifest.secrecy` is a level the community does not serve: an encrypted level needs a community that implements CONVENTIONS §9 (trustees, encrypted casts and their certification)."
    retryable: false
  - code: vtc/elections/create:communityMismatch
    meaning: "`manifest.community` is not this community's DID. A community creates only its own elections."
    retryable: false
  - code: vtc/elections/create:ballotBoxNotTrusted
    meaning: "`manifest.ballotBox` is not on the community's list of trusted voting services (VTI-VOT-082)."
    retryable: false
  - code: vtc/elections/create:ballotBoxRefused
    meaning: "The ballot box refused the registration. Its own code and message are carried in `details`. The community has recorded nothing."
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
  - vtc/elections/list
  - vtc/elections/show
  - vtc/elections/close
  - vtc/elections/cancel
  - vtc/elections/certify
  - vtc/elections/tickets/issue
  - voting/election/register
  - vtc/admin/actions/show
  - trust-task-next-step
---

## Abstract

**VTC Elections — Create** is how a community starts a poll, a survey or an election. An administrator sends the election's manifest; the community, acting as the election's **registrar**, checks it, records the election and registers it with the voting service the manifest names as its **ballot box** ([`voting/election/register`](../../../../voting/election/register/0.1/spec.md)). From the manifest's `registrationOpens` the community issues tickets to the parties its roll admits ([`vtc/elections/tickets/issue`](../../tickets/issue/0.1/spec.md)); the ballot box admits their casts and keeps the board; the community never handles a ballot.

A manifest that says its result is **binding** takes another administrator's consent before the community acts on it. The family's shared rules are in [`CONVENTIONS.md`](../../../../voting/_shared/0.1/CONVENTIONS.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (`issuer`) sends the manifest whole, carries a proof, and chooses an `electionId` the community has not used.

A conforming **community** (`recipient`) checks, in this order:

1. **The requester holds `vtc.elections.manage`** (VTI-VOT-080), read from the signer's live access-control entry — or, for a delegated signing key, the entry of the identity it acts for — at execution time. Else `permissionDenied`.
2. **The manifest is valid** as [`voting/election/register`](../../../../voting/election/register/0.1/spec.md) would judge it, and **MUST** be refused with `invalidManifest` otherwise — including a `votingCloses` already past, and a `registrationOpens` after `votingOpens` or a `votingOpens` not before `votingCloses`. The community **MUST NOT** register a manifest the ballot box would refuse for its shape.
3. **The secrecy level is served.** A `secrecy` the community does not serve **MUST** be refused with `unsupportedSecrecy`. A community serves `sealed` and `talliedEncrypted` only if it certifies them as CONVENTIONS §9 provides, and **MUST** then hold such a manifest to its trustee rules — trustees named, `2 ≤ threshold ≤` their number, neither the community nor the ballot box among them, no running totals, and only choice questions when tallied-encrypted — refusing one that breaks them with `invalidManifest`.
4. **The manifest is this community's.** `manifest.community` **MUST** equal the community's own DID, or `communityMismatch`.
5. **The ballot box is trusted.** `manifest.ballotBox` **MUST** be on the community's list of trusted voting services, or `ballotBoxNotTrusted` (VTI-VOT-082). That list is community configuration that alters trust semantics: a change to it is the community's to govern as it governs any such change, and never rides on this task.
6. **The identifier is new.** An `electionId` the community already holds an election, or a parked creation, under **MUST** be refused with `vtc/elections/create:electionExists`, whatever the manifest. A repeat of the same signed document is answered as [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 11 provides, with the answer first given.

Then:

7. **A binding election is parked** (VTI-VOT-081). When `manifest.binding` is `true`, a community implementing the administrator action list does not record or register the election on the requester's word alone: it parks the creation for another holder of `vtc.elections.manage` to consent to, records nothing of the election until the creation executes, and answers with the framework-reserved [`trust-task-next-step/0.1`](../../../../trust-task-next-step/0.1/spec.md), carrying `continuation: proceed` and one `expects` entry whose `typeUri` is `https://trusttasks.org/spec/vtc/admin/actions/show/0.2` and whose `hint` is `{"actionId": "…"}`, exactly as [`vtc/admin/actions/show/0.2`](../../../admin/actions/show/0.2/spec.md) describes. The requester does **not** re-submit; the community executes the parked document itself, **exactly once**, when its approvals are complete, and **MUST** first re-run items 1 to 6 against the community and the clock as they are then. An approval **MUST** count only from a holder of `vtc.elections.manage` other than the requester. The community answers on the request's `threadId` with this task's `#response`, carrying `actionId`, or a `trust-task-error`.
8. **Remote first.** On execution the community records the election as `pendingRegistration`, under its manifest and the manifest's digest, and sends `voting/election/register` to the ballot box, signed as itself. It **MUST** record the election as `registered` only after the ballot box answered with success. When the ballot box refuses, the community **MUST** discard the record and answer `ballotBoxRefused` with the ballot box's code and message. When the ballot box has not answered by the time the community answers, the community answers with `state: pendingRegistration` and **MUST** keep delivering the registration itself, with a capped backoff, until the ballot box answers. The community is the only retry owner: the requester **SHOULD NOT** re-submit, and registration is idempotent at the ballot box for the same manifest (CONVENTIONS §8.2). A registration the ballot box refuses on a later delivery leaves the election refused, which [`vtc/elections/show`](../../show/0.1/spec.md) reports and [`vtc/elections/cancel`](../../cancel/0.1/spec.md) clears.
9. **No ticket before registration.** The community **MUST NOT** issue a ticket for an election the ballot box has not registered (VTI-VOT-024).
10. **Audit** (VTI-VOT-100). The community **MUST** audit the creation: the requester, the `electionId`, the manifest's digest, whether it was binding, the parked action's identifier when there was one, and the outcome of the registration.

## Authorization

The authority this task presupposes is the **`vtc.elections.manage` capability at the community** (VTI-VOT-080), read from the signer's access-control entry at execution time. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

**A binding election takes two.** A binding result decides something in the community's name, so its creation is the agreement of two holders of `vtc.elections.manage`: the requester's signed document and another's signed decision on the parked action (VTI-VOT-081). The approver's own decision, not the requester's proof, carries the consent. A community running in a single-administrator mode, where nobody else could consent, may waive the consent on its own stated terms. Whether, and by how many, a creation must be approved is the community's policy ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13); this specification describes the model it is designed for.

The community, not the requester, speaks to the ballot box: `voting/election/register` is signed by the community's DID, and the ballot box authorizes it on that alone (CONVENTIONS §4).

## Definitions

- **`manifest`** — the election's [`Manifest`](../../../../voting/_shared/0.1/voting.schema.json). Every member is the producer's choice, within the checks above.
- **`manifestDigest`** (response) — the SHA-256 multihash of the RFC 8785 (JCS) canonicalization of the manifest as registered.
- **`state`** (response) — `registered` or `pendingRegistration`, as Conformance item 8 defines.
- **`actionId`** (response) — the parked action this response follows from, when the creation was parked.
- **Trusted voting services** — the community's configured list of ballot boxes it will register an election with.

## Request

The administrator sends the community the manifest. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A board election

```json
{
  "id": "urn:uuid:7c1e2a4b-3d5f-4a6b-8c7d-9e0f1a2b3c4d",
  "type": "https://trusttasks.org/spec/vtc/elections/create/0.1#request",
  "issuer": "did:web:carol.example",
  "recipient": "did:webvh:QmCommunity:community.example",
  "issuedAt": "2026-10-20T09:00:00Z",
  "threadId": "urn:uuid:7c1e2a4b-3d5f-4a6b-8c7d-9e0f1a2b3c4d",
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
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:carol.example#key-1",
    "created": "2026-10-20T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z7cDExampleProofValue"
  }
}
```

## Response

The community answers with the election as recorded — the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). A binding creation is first answered with `trust-task-next-step`, and this response follows on the same `threadId` when it executes. A refusal is a `trust-task-error`.

### Parked for consent

The manifest is binding, so the community parks it.

```json
{
  "id": "urn:uuid:2b3c4d5e-6f70-4a81-92a3-b4c5d6e7f809",
  "type": "https://trusttasks.org/spec/trust-task-next-step/0.1",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-20T09:00:01Z",
  "threadId": "urn:uuid:7c1e2a4b-3d5f-4a6b-8c7d-9e0f1a2b3c4d",
  "payload": {
    "continuation": "proceed",
    "expects": [
      {
        "typeUri": "https://trusttasks.org/spec/vtc/admin/actions/show/0.2",
        "hint": { "actionId": "act_Vb3k9Qm2Lp7xR4tZ" }
      }
    ]
  }
}
```

### Executed after approval

Another administrator approved; the community recorded the election, and the ballot box answered.

```json
{
  "id": "urn:uuid:3c4d5e6f-7081-4a92-a3b4-c5d6e7f8091a",
  "type": "https://trusttasks.org/spec/vtc/elections/create/0.1#response",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-20T14:30:00Z",
  "threadId": "urn:uuid:7c1e2a4b-3d5f-4a6b-8c7d-9e0f1a2b3c4d",
  "payload": {
    "electionId": "board-2026",
    "manifestDigest": "zQmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
    "state": "registered",
    "actionId": "act_Vb3k9Qm2Lp7xR4tZ"
  }
}
```

### Refused: the ballot box is not trusted

```json
{
  "id": "urn:uuid:4d5e6f70-8192-4aa3-b4c5-d6e7f8091a2b",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2026-10-20T09:00:01Z",
  "threadId": "urn:uuid:7c1e2a4b-3d5f-4a6b-8c7d-9e0f1a2b3c4d",
  "payload": {
    "code": "vtc/elections/create:ballotBoxNotTrusted",
    "message": "did:web:ballots.example is not one of this community's trusted voting services.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request is the manifest: the questions and their options, the roll rule and any DIDs it enfranchises by name, the schedule, the ballot box, the auditors and the trustees. A manifest is published to every party the roll admits and, on a public board, to anyone, so a producer **MUST NOT** put in it anything the community would not publish — a reason for enfranchising someone, a note about a candidate — and **MUST NOT** use `ext` for it. A DID in `roll.listed` is disclosed to everyone who reads the manifest; a producer **SHOULD** list only parties who agreed to be named. The response is an identifier, a digest and a state.

### Correlation

The manifest names the community, the ballot box and the auditors, all public by design: a member must be able to see who will hold and read the board before requesting a ticket ([`Disclosure`](../../../../voting/_shared/0.1/voting.schema.json), VTI-VOT-090). The administrator declares `identifierScope: pairwise`: accountability for a creation belongs inside the community, where the audit trail must name the same administrator over time. Nothing in this task concerns a voter; who takes a ticket is learnt only by [`vtc/elections/tickets/issue`](../../tickets/issue/0.1/spec.md), and how anyone voted never reaches the community at all (CONVENTIONS §2).

### Retention

Durable. The community keeps the manifest and its digest for as long as the election's result stands, because certification ([`vtc/elections/certify`](../../certify/0.1/spec.md)) and every later recount are bound to them. The audit row of the creation — who asked, who consented, which manifest — **SHOULD** be kept on the schedule the community keeps other governance decisions on.

### Consent/purpose

The manifest is collected to run the election it describes and for nothing else; the DIDs a roll lists are collected to issue them tickets. Whether a creation needs a second administrator, or a step-up, is the community's policy; this specification describes the model it is designed for and does not require a gate ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).
