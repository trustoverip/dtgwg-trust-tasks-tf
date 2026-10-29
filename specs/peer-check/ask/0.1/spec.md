---
slug: peer-check/ask
version: "0.1"
title: "Peer Check — Ask"
summary: "One party asks another a short yes-or-no question, such as \"did you just ask me for this?\", over a channel they already have; the other party's device prompts its user, and the signed answer lets the asker verify it against the responder's DID."
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - peer
  - confirmation
  - challenge
  - presence
  - impersonation
parties:
  - role: Asker
    requirement: REQUIRED
    member: issuer
  - role: Responder
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The question is text the responder's user relies on to decide, so a proof binds it to the asker's key and stops a
    relay from rewriting it. The response is the whole point of the task: an unsigned yes is a message anyone on the
    channel could have typed, and the asker takes the responder's identity from the proof, never from the channel the
    answer arrived on.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The exchange is only meaningful while it is fresh. An old `confirmed` answer replayed later would read as a recent
    confirmation, so the asker needs the issue time to apply the request's deadlines and its own freshness window.
sideEffects:
  level: none
  rationale: >-
    Raising a prompt and recording an answer changes nothing on either side beyond the short-lived pending exchange.
    What the asker then does with a yes is outside this task.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: >-
    The request delivers asker-authored text (`question`, `note`) to the responder's screen, which can be personal or
    deliberately misleading, so it is declared as ingested personal data and treated as untrusted. The response tells
    the asker that this responder's key was used to give this answer at this time, and nothing more.
retention:
  class: exchange
  rationale: >-
    Both sides need the documents only for the life of the pending exchange: the responder to render the prompt, the
    asker until it has checked the answer or the question's deadline has passed. An asker that keeps a `confirmed` answer as a record
    does so for its own purposes; this task asks it to keep nothing.
errorCodes:
  - code: peer-check/ask:noSurface
    meaning: The responder's device cannot show a prompt now (locked, headless, or no consent surface available). No question was put to the user.
    retryable: true
  - code: peer-check/ask:askerNotAccepted
    meaning: The responder's device is configured not to prompt for this asker, so no question was put to the user. The reason is the responder's policy and is not disclosed.
    retryable: false
related:
  - task-consent/request
  - task-consent/decision
  - auth/step-up/approve-request
---

## Abstract

Someone phones, messages or video-calls you and asks for something. You want to know whether the person you think you are talking to really did. **Peer Check — Ask** lets the party who has doubts (the asker) send a short yes-or-no question to the other party's own device, over a channel the two already share. The responder's user sees the question, answers yes or no, and the device returns a signed answer the asker can verify against the responder's DID.

It is a Trust Task, rather than an API call, because the value is in the signed answer: it can be checked by the asker alone, with any DID method, and it is bound to one exact question so it cannot be reused for another.

This task is deliberately generic. It does not say what the question is about, and it does not authorise anything: it records that the holder of one key, using that key's device, gave one deliberate answer to one question at one time. See [Security & Privacy](#security--privacy) for what that does and does not prove.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **asker** (the request's `issuer`, the response's consumer):

1. **MUST** generate `challenge` from a cryptographically secure random source with at least 128 bits of entropy, a fresh value for every request.
2. **MUST** set `payload.expiresAt`, the deadline for the answer, and **SHOULD** keep the window short (minutes, not hours). It **SHOULD** also set the envelope `expiresAt`, which keeps its framework meaning ([SPEC §4.2](/SPEC.md#42-top-level-members)): after it the responder does not prompt for the request.
3. **MUST** sign the request, and **MUST** set `recipient` to the DID of the party it intends to ask.
4. **MUST** treat the answer as valid only if every one of these holds, and **MUST** otherwise treat the exchange as unanswered:
   1. the response's `proof` verifies, with `proofPurpose` `authentication`, and its `verificationMethod` is listed under the `authentication` relationship of the DID document of the party the request was addressed to;
   2. the response's `issuer` equals that request `recipient`, and its `threadId` pairs it with the request ([SPEC §4.9](/SPEC.md#49-the-threadid-member));
   3. `payload.challenge` equals the challenge the asker issued for this request, and the asker has not already accepted an answer for it;
   4. `payload.requestDigest` reproduces over the request document the asker sent, per [SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names), comparing decoded multihash bytes;
   5. the answer arrived before the request's `payload.expiresAt`, by the asker's own clock; and
   6. the signing key was valid for the responder's DID **at the time of the check**. The asker resolves the DID document when it verifies, not from a copy cached when the exchange began, so a key revoked or rotated away in between is refused.
5. **MUST** consume the challenge when it accepts or rejects an answer, so that a second answer for the same challenge is refused.
6. **MUST NOT** treat `confirmed` as proof that the person it is talking to is who they claim to be. See [Security & Privacy](#security--privacy).

A conforming **responder** (the request's `recipient`, the response's producer):

1. **MUST** verify the request's `proof` and that neither `payload.expiresAt` nor, where present, the envelope `expiresAt` has passed before showing anything to its user, and **MUST NOT** prompt for a request it cannot verify.
2. **MUST** show `question` to its user attributed to the asker's verified identity, and **MUST** show `note`, where present, as plain text visually distinct from `question`, never as markup, a link or an instruction.
3. **SHOULD** require a deliberate user action to answer, gated by a user-presence check such as device unlock or biometrics, so that an answer cannot be produced by software acting without the user. A responder that cannot do so SHOULD say so in its own interface; this task has no member for declaring it, because such a declaration would be unverifiable.
4. **MUST** answer with the exact `challenge` and the task digest of the request document as it received it, and **MUST** sign the response with a key listed under the `authentication` relationship of its own DID.
5. **MUST NOT** answer `confirmed` on a timeout, a default, or any path that did not involve the user's explicit choice. An unanswered prompt is not an answer.
6. A request whose prompt is open and waiting on the user is **held**, and a hold is a next step ([SPEC §8.6](/SPEC.md#86-reserved-response-type-slugs)): the responder **MUST** answer the request with `trust-task-next-step` carrying `continuation: resubmit`, and **MUST NOT** answer with an error response. It answers a resubmission in the same thread with the decision once it is known, or with a further next step while the prompt is still open. On an asynchronous transport where the asker sees nothing until the decision exists, silence is all the asker sees until the decision arrives. If `payload.expiresAt` passes with the prompt unanswered, the question lapses, the responder sends no `confirmed` answer, and the asker treats the exchange as unanswered.
7. **MAY** stay silent instead of sending `peer-check/ask:askerNotAccepted` (see [Responder disclosure](#responder-disclosure)).

## Definitions

The asker is the `issuer` of the request, the responder is its `recipient`, and the response is issued by the responder to the asker.

- **`challenge`** — a fresh, unguessable, base64url value of at least 128 bits, chosen by the asker for this one request. It ties an answer to one request and one moment.
- **`question`** — up to 200 characters. Asker-authored; the responder's device shows it, attributed to the asker.
- **`note`** — OPTIONAL, up to 500 characters. Asker-authored and **untrusted**: it is context the user may find useful and nothing the device vouches for.
- **`payload.expiresAt`** — REQUIRED. The answer deadline: after it the question lapses, the responder does not prompt and the asker accepts no answer. This is the task's own deadline ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 16), in the style of `task-consent/request/0.1`.
- **`expiresAt`** (envelope) — keeps its framework meaning ([SPEC §4.2](/SPEC.md#42-top-level-members)): after it the responder does not prompt for a stale request. A consumer that has begun work does not abandon it because the envelope `expiresAt` has since passed, and the asker does not read it as the answer deadline.
- **`requestDigest`** — the task digest of the request document, computed as [SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names) defines it, so the answer commits to every member of the request, the parties and the deadline.
- **`decision`** — `confirmed` or `notConfirmed`. It answers the question asked; it is not an authorisation.
- **`respondedAt`** — when the user answered. The envelope `issuedAt` is when the responder signed the document, which can be later; both are kept so the asker can tell the user's answer time from the signing time.
- **`deniedReason`** — OPTIONAL, only with `notConfirmed`: `notRecognised`, `declined` or `other`. The schema enforces the pairing with `dependentSchemas`: a document carrying it beside `confirmed` is invalid.

## Request

The asker is the producer and the responder the recipient. The request payload is described by [`payload.schema.json`](payload.schema.json). The document MUST carry `issuedAt`, `payload.expiresAt` and a `proof`; the proof below is abbreviated for the example.

### The asker puts a question to the responder

```json
{
  "id": "urn:uuid:5b0c7a1e-3d52-4f0e-9a6b-8e41c2d97f10",
  "type": "https://trusttasks.org/spec/peer-check/ask/0.1#request",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:bob.example",
  "issuedAt": "2026-10-28T14:00:00Z",
  "expiresAt": "2026-10-28T14:05:00Z",
  "payload": {
    "challenge": "q7Xk2mRt9LwB4vNf8ZpYhA",
    "question": "Did you just ask me to send you a one-time code?",
    "note": "I am on the phone with someone who sounds like you and wants a code.",
    "expiresAt": "2026-10-28T14:03:00Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-28T14:00:00Z",
    "verificationMethod": "did:web:alice.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z3FXQjecWufY46yg5abdVZsXqLhxhueuSoZgNRBCx7tRzqXRDgZ3wM1u8TRBo2s7YC5TrRTUq5J9pi9pTMpDfdyMg"
  }
}
```

## Response

The responder is the producer, the asker the recipient. The response payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, with the codes declared in front matter.

### The responder says yes (`confirmed`)

```json
{
  "id": "urn:uuid:8d2e6f43-91a7-4c1b-b7d0-2f6a93c15e88",
  "type": "https://trusttasks.org/spec/peer-check/ask/0.1#response",
  "threadId": "urn:uuid:5b0c7a1e-3d52-4f0e-9a6b-8e41c2d97f10",
  "issuer": "did:web:bob.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-10-28T14:00:20Z",
  "payload": {
    "challenge": "q7Xk2mRt9LwB4vNf8ZpYhA",
    "requestDigest": "zQmaoDjqe8muePtetx54p69mGtegPHFcqwQAWz6SsTh2gUD",
    "decision": "confirmed",
    "respondedAt": "2026-10-28T14:00:19Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-28T14:00:20Z",
    "verificationMethod": "did:web:bob.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z4oey5q2M3XKaxup7gH9dCtMhJEHgjqQkWtM2ohbSiDJ7jPLXsCjGrfG5Yp8XiL1aVZUgQjSBUUk3sQCm2VXrY8qk"
  }
}
```

### The responder says no (`notConfirmed`)

```json
{
  "id": "urn:uuid:1a9c4b07-6e33-4d2a-8f15-b0c7d3e29a64",
  "type": "https://trusttasks.org/spec/peer-check/ask/0.1#response",
  "threadId": "urn:uuid:5b0c7a1e-3d52-4f0e-9a6b-8e41c2d97f10",
  "issuer": "did:web:bob.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-10-28T14:00:25Z",
  "payload": {
    "challenge": "q7Xk2mRt9LwB4vNf8ZpYhA",
    "requestDigest": "zQmaoDjqe8muePtetx54p69mGtegPHFcqwQAWz6SsTh2gUD",
    "decision": "notConfirmed",
    "respondedAt": "2026-10-28T14:00:24Z",
    "deniedReason": "notRecognised"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-28T14:00:25Z",
    "verificationMethod": "did:web:bob.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z2mDfC3vL8nQe7sYxKp4WuTb1RhA9GjZc6VoNiXdE5yUtMqFkS3aPrHw8LJgB2zCn4TvQe6YsDxKu7MhRf9AiWp3o"
  }
}
```

### The responder holds while the user decides

While the prompt is open, the responder answers with a next step, not an error. The asker resubmits in the same thread (a new document with a fresh `id`, same `threadId`) and receives the decision, or a further next step, in reply.

```json
{
  "id": "urn:uuid:c6e1d9a2-4b70-4f38-a5d3-72e8f0b41c9d",
  "type": "https://trusttasks.org/spec/trust-task-next-step/0.1",
  "threadId": "urn:uuid:5b0c7a1e-3d52-4f0e-9a6b-8e41c2d97f10",
  "issuer": "did:web:bob.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-10-28T14:00:05Z",
  "payload": {
    "expects": [
      {
        "typeUri": "https://trusttasks.org/spec/peer-check/ask/0.1#request",
        "reason": "The question is showing on the responder's device and has not been answered yet."
      }
    ],
    "continuation": "resubmit",
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/peer-check/ask/0.1#request",
      "id": "urn:uuid:5b0c7a1e-3d52-4f0e-9a6b-8e41c2d97f10"
    }
  }
}
```

## Relationship to other tasks

- **`confirm/request` and `confirm/response` 0.1 (retired).** Those asked a *relying party* to obtain a user's consent to an action. They were retired in favour of `task-consent` ([#156](https://github.com/trustoverip/dtgwg-trust-tasks-tf/pull/156)) because a requester-authored `reason` is the least trustworthy basis for a consent decision. This task does not ask for consent to an action and does not authorise anything; it asks a peer a question about what the *peer* did, and treats the asker's text as untrusted throughout. It reuses none of the retired members' semantics for `reason`, `actionType` or `actionDetails`.
- **`task-consent/*`.** An executor asks an enrolled approver device to authorise one pending privileged task, with effects computed by the executor. Here there is no executor, no pending task, no approver set and no enrolment: two peers who already share a channel.
- **`auth/step-up/*`.** Raises the assurance level of a session. This task has no session.
- **`consent/approve-request`, `vrc/relationships/propose`.** Different parties and purposes; neither asks a peer whether they just did something.

## Security & Privacy

### Data carried

The request carries the asker's DID, the responder's DID, a random challenge, a question of at most 200 characters and an optional note of at most 500. The response carries an echoed challenge, a digest, a decision and a time. A producer **MUST NOT** put secrets, one-time codes, account numbers or other credentials in `question` or `note`: the point of the task is to check a person, not to transport anything sensitive, and the text can be seen by every intermediary the transport has. The smallest useful question is the best one.

`question` and `note` are asker-authored and **untrusted**. They can be wrong, manipulative, or written by someone who has taken over the asker's account. A responder's device renders them as plain text, attributed to the asker, never as markup or a link, and never in a way that could be mistaken for the device's own message.

### Correlation

A responder's DID, a stable `threadId`, and request timing are visible to whoever carries the documents. The asker and responder already know each other's identifiers; an observer of the channel learns that these two DIDs exchanged a check at this time. A producer **SHOULD** use a fresh challenge and a fresh document `id` for every request, and **SHOULD** use pairwise DIDs where the two parties have them.

### Retention

Both sides need the documents only until the answer has been checked or the question's deadline has passed. An asker that keeps a `confirmed` answer as a record is making its own choice; the answer is signed and is evidence of what it states, which is less than it may seem (see below). Deleting it after the exchange is the default this task assumes.

### Consent/purpose

The data is collected so the asker can ask one question of one peer and check the answer. It is not a general channel for requesting data, and a responder's agreement to be asked is a matter for the responder's own policy, which is why `askerNotAccepted` exists.

### What a signed yes proves, and what it does not

A valid `confirmed` answer proves that the holder of a key listed in the responder's DID document, using the device that holds it, made a deliberate choice, when the responder's device enforces one, to answer yes to this exact request at about this time. It does **not** prove that the person the asker can currently see or hear is the genuine responder, that the question was understood, or that the responder's device was in the responder's hands. A person who has the responder's unlocked device, or who is coercing the responder, can answer. A synthetic voice or face on a call is not defeated by this task, and nothing in it claims to defeat one: it adds one independent signal that must come from the responder's own key.

Responders **SHOULD** gate the answer behind a user-presence check (device unlock or biometrics). An asker can only rely on this where it knows the responder's software does it; the task cannot make it visible.

### Responder disclosure

Each refusal tells the asker something about the responder, in the manner of [SPEC §12.6](/SPEC.md#126-error-response-identity-leakage). `askerNotAccepted` tells an asker that the responder exists, is reachable, and has a policy about that asker; `noSurface` tells it that the responder's device is locked, headless or otherwise unable to prompt now. A party probing for reachability or device state can use either as an oracle. A responder that does not want to give that answer **MAY** stay silent instead of sending `askerNotAccepted` (and `noSurface`), at the cost of the asker not learning that no question was put; the asker treats silence as unanswered.

### Out-of-band channel

The value of the check comes from the answer travelling a different path from the suspicious contact. If the person who is being checked supplies the means of answering, for example a link or a number in the same call, an attacker who controls that conversation controls the check. The asker **SHOULD** deliver the request to the responder through a channel it already trusted before the contact in question, such as a DID relationship or a messaging channel established earlier, and **SHOULD NOT** accept an answer relayed by the party being checked.

### Replay and binding

The challenge makes each request unique; the request digest binds the answer to the exact text, parties and deadline that the responder saw; `payload.expiresAt` and the asker's freshness check bound the window; consuming the challenge refuses a second use. An answer is therefore of no use against any other request, to any other asker, or after the question's deadline has passed. A yes to one question does not become a yes to a different question phrased under the same challenge, because the digest differs.

### Key state

The asker resolves the responder's DID when it verifies the answer, and refuses a key that is not valid at that moment. A key rotated or revoked since the request was sent fails. The check is independent of the DID method: any method that can resolve a DID document and report key validity at a time, including `did:web`, `did:webvh` and `did:webs`, works.
