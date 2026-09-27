---
slug: auth/step-up/start
version: "0.1"
title: Auth — Step-up Start
summary: A session holder asks a relying party to begin raising the assurance level of one of its sessions; the relying party answers with the signed approve-request the approver then answers.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - auth
  - step-up
  - aal
  - session
  - approval
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Session holder
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: Relying party
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "The task names a session and asks for a challenge bound to it. The relying party must know from the document itself that the asker is the session's subject — on a transport with no bearer token there is nothing else — so that nobody can open step-up ceremonies against a session that is not theirs."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Each start mints a challenge the relying party holds until it expires. A replayed start would mint challenges the holder never asked for and spend the relying party's step-up budget for the session."
sideEffects:
  level: mutating
  rationale: "Creates pending step-up state — a challenge bound to the session, the subject and an expiry — that the relying party holds until the approve-response consumes it or it lapses."
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries a session identifier. The response carries the relying party's signed approve-request, whose challenge is binding material, not data about anyone."
retention:
  class: exchange
  rationale: "The challenge lives only until the approve-response arrives or it expires; nothing of the request is needed after that."
errorCodes:
  - code: auth/step-up/start:sessionUnknown
    meaning: "No live session with this `sessionId` belongs to the proven issuer. Returned alike for a session that does not exist, one that has expired, and one that belongs to someone else, so the code cannot be used to probe sessions."
    retryable: false
  - code: auth/step-up/start:notNeeded
    meaning: "The session is already at or above the requested assurance level."
    retryable: false
  - code: auth/step-up/start:rateLimited
    meaning: "The holder has started too many step-ups for this session recently."
    retryable: true
related:
  - auth/step-up/approve-request
  - auth/step-up/approve-response
  - auth/refresh
  - auth/passkey/login/start
---

## Abstract

An out-of-band step-up is two documents: the relying party's signed [`approve-request`](../../approve-request/0.3/spec.md), which carries a challenge bound to the session, and the approver's signed [`approve-response`](../../approve-response/0.5/spec.md) over it. What was missing is how the holder of a session *asks* for the first one. `approve-request` 0.3 can travel inline in the refusal of an operation, but only for a step-up bound to that operation; raising a *session* needed a bespoke call to the relying party — a REST `start` path returning the document, and a REST `finish` path taking the approval.

**Step-up Start** is that call as a Trust Task. The holder sends it on any transport; the relying party answers with the signed approve-request. The holder's approver answers that with an `approve-response` sent to the relying party as an ordinary Trust Task, which is the whole ceremony on the relying party's standard dispatch — HTTPS `POST /trust-tasks`, DIDComm or TSP — and no bespoke path at all.

The `approve-response` 0.5 `#response` reports the elevated *session*, not tokens. A relying party that issues bearer tokens carrying the assurance level issues new ones through [`auth/refresh`](../../../refresh/0.1/spec.md) once the session is elevated; this task does not change that.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming relying party:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`), and one with no `issuedAt` with `malformedRequest`.
2. Answers `sessionUnknown` unless `sessionId` names a live session whose subject is the proven issuer — or whose recorded session key is the key the proof was made with — and answers the three failing cases identically.
3. Answers `notNeeded` when the session's assurance level is already at or above `targetAcr` (or, when it is absent, at the relying party's highest level), and `rateLimited` when its per-session budget is spent.
4. Otherwise generates a challenge and binds it to `(subject, sessionId, expiresAt)` exactly as `approve-request` 0.3 Conformance items 3 and 4 require, and answers with `approveRequest`: a complete `auth/step-up/approve-request/0.3` document for that session, signed by the relying party, with the relying party as `issuer` and the approver as `recipient`.
5. **MUST NOT** elevate anything on this task. Elevation happens only on a verified `approve-response` whose challenge matches the one bound here.

A conforming holder — and its approver — **MUST** verify `approveRequest` on its own terms, as `approve-request` 0.3 requires of a consumer, before surfacing its `reason` or signing anything over its challenge. The proof on this task's `#response` authenticates the envelope; it does not substitute for the approve-request's own.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **being the subject of the named session**, or holding the key the relying party recorded for that session: the proof-verified `issuer` (or the proof's key) is compared with the relying party's own session record, and the payload's `sessionId` only says which record to compare with. The proof establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). Starting a step-up grants nothing: the approver's signed decision, verified at the relying party, is what elevates.

## Definitions

**Approver** — the party whose signed `approve-response` the relying party accepts for the session's subject: the subject itself, or a delegate the relying party recognises.

## Request

```json
{
  "id": "urn:uuid:6b1734a8-2846-479a-8d9e-7a8b9c0d1e01",
  "type": "https://trusttasks.org/spec/auth/step-up/start/0.1",
  "issuer": "did:webvh:QmAliceScid7:did.example.com:alice",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": { "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b", "targetAcr": "aal2" }
}
```

## Response

The embedded approve-request is abbreviated; its full shape and its proof are defined by [`approve-request` 0.3](../../approve-request/0.3/spec.md).

```json
{
  "id": "urn:uuid:6b1734a8-2846-479a-8d9e-7a8b9c0d1e02",
  "type": "https://trusttasks.org/spec/auth/step-up/start/0.1#response",
  "threadId": "urn:uuid:6b1734a8-2846-479a-8d9e-7a8b9c0d1e01",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmAliceScid7:did.example.com:alice",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": {
    "approveRequest": {
      "id": "urn:uuid:6b1734a8-2846-479a-8d9e-7a8b9c0d1e03",
      "type": "https://trusttasks.org/spec/auth/step-up/approve-request/0.3",
      "issuer": "did:webvh:QmControlScid2:control.example.com",
      "recipient": "did:webvh:QmAliceScid7:did.example.com:alice",
      "issuedAt": "2026-09-27T09:00:01Z",
      "expiresAt": "2026-09-27T09:05:01Z",
      "payload": {
        "subject": "did:webvh:QmAliceScid7:did.example.com:alice",
        "sessionId": "ec5d3c89-3f49-49b2-9d7d-2a8c0a8a7b9b",
        "challenge": "u0Zp3cQ8mV2kX9sN4bT7yR1wE6aL5fH0",
        "reason": "Raise your console session to manage domains.",
        "targetAcr": "aal2"
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:webvh:QmControlScid2:control.example.com#key-2",
        "created": "2026-09-27T09:00:01Z",
        "proofPurpose": "authentication",
        "proofValue": "z3sXm..."
      }
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries a session identifier and optionally a target level. The response carries the relying party's signed approve-request: the subject, the same session identifier, a challenge and the relying party's reason. Nothing here is a credential: the challenge only becomes useful when the approver signs it, and a signature over it is accepted only for this session.

### Correlation

The request links the holder to one of its sessions, at the relying party that issued the session. The holder's `identifierScope` is `any`: the task works with whatever identifier the session was opened under, pairwise or not, since the relying party compares it with its own record. The relying party declares `identifierScope: public`: it is the party the holder authenticated to and the signer the approver must recognise, and a pairwise identifier for it would leave the approver nothing to check the approve-request's issuer against.

### Retention

The relying party keeps the pending challenge until it is consumed or expires, and nothing of the request after that. A relying party that audits step-ups records the ceremony's outcome from the `approve-response`, not this task.

### Consent/purpose

The task exists so a session holder can obtain the document their approver must see before an elevation. Whether an elevation is needed for any operation, and what counts as approval, are the relying party's policy; this task neither states nor implies either.
