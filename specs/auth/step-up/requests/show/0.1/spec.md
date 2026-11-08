---
slug: auth/step-up/requests/show
version: "0.1"
title: "Auth — Step-up Request Show"
summary: "The subject's approver agent fetches, by a single-use handle, a pending step-up — the relying party's signed approve-request and the operation it binds — so a step-up refused to a delegated producer can be answered on the subject's own device."
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords: [step-up, approve-request, hand-off, wallet, delegated-key, trigger-link, out-of-band]
parties:
  - role: Approver's agent
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: Relying party
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The handle is spent by the first request that names it, so the relying party must be able to tell which signer spent it and refuse a replay of the same document carried by anyone else; a proof by the agent's key, over a document with a unique `id`, is what makes the spend attributable and the answer bound to one requester.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The request spends a handle that lives only as long as the pending step-up it names — minutes — so a request that cannot be placed in time cannot be told apart from a replay of one sent earlier.
sideEffects:
  level: mutating
  rationale: >-
    Answering spends the handle: the relying party answers a handle at most once and every later request naming it is `notFound`. Nothing else changes — the pending step-up is neither approved nor withdrawn by being read.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: >-
    The response discloses the pending step-up — the subject's DID, the relying party's challenge, the accepted evidence kinds and approvers — and the operation it is bound to, whose payload may name other parties. It discloses no secret and exercises no authority of the subject's: approving still takes the subject's own factor and own signature on an approve-response.
retention:
  class: exchange
  rationale: >-
    The relying party keeps the handle, and the operation it names, only while the pending step-up lives; the agent keeps what it fetched only until the person decides.
errorCodes:
  - code: auth/step-up/requests/show:notFound
    meaning: No pending step-up answers to this handle — unknown, already read, expired, or its step-up already approved or withdrawn. One code for all of them, so a handle reveals nothing about which.
    retryable: false
related:
  - auth/step-up/approve-request
  - auth/step-up/approve-response
  - auth/step-up/approver/attest
---

## Abstract

A relying party that refuses an operation until the person it acts for steps up can carry the step-up request **inline**, in the refusal ([`approve-request/0.5`](../../../approve-request/0.5/spec.md), *Inline delivery*). That works when the operation's producer is the person's own agent — the browser holding their passkey, the console beside their wallet — because the producer surfaces the request and gathers the evidence itself. It fails when the producer cannot: a process acting with a key the person **delegated** to it, an automation, an assistant. A delegated key can sign the operation and can redeem a recorded step-up, but it can never make one, and an inline request has no proof of its own, so the producer cannot simply forward it to the person's wallet for display.

This task closes that gap with a **hand-off by reference**. Beside the inline request the relying party gives the producer a single-use **handle**. The producer shows the person a way to open it on the device that holds their factor — a link or a code. The person's approver agent sends this task with the handle and receives the relying party's own **signed** `approve-request` document, addressed to the person, together with the operation it is bound to, exactly as the relying party received it. The agent verifies the relying party's proof, shows the person the operation itself and the relying party's reason, and — if they approve — answers with [`approve-response/0.7`](../../../approve-response/0.7/spec.md) signed by the person's own DID, carrying their own evidence. The relying party records the approval; the producer sends the identical operation again and the recorded step-up is redeemed.

Nothing new is trusted: the request the agent shows is signed by the relying party, the answer is signed by the person with their own factor, and the producer still only redeems.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### The relying party offering a hand-off

A relying party that refuses an operation with an inline `details.stepUpRequest` ([`approve-request/0.5`](../../../approve-request/0.5/spec.md), *Inline delivery*):

1. **MAY** carry `details.stepUpHandoff`, an object valid against this specification's `$defs.Handoff`, beside it. It **MUST NOT** carry one without an inline request, nor for a step-up that elevates a session.
2. **MUST** mint the handle with at least 128 bits of entropy, bind it to that one pending step-up and to the operation it is bound to, and set `expiresAt` no later than the pending step-up's own expiry.
3. **MUST** keep the operation — its Type URI and its payload, member for member as received — for as long as the handle can be answered.
4. **MAY** answer a later sending of the same operation with a new inline request and a new handle. An approval of any open step-up for that operation authorizes it; a producer that sends the operation again before the person has decided is not thereby refused.

### The consumer (the relying party answering this task)

A conforming **consumer**:

1. **MUST** answer a handle at most once, atomically: the first request that names a live handle receives the response, and every later one `notFound`, whoever sends it.
2. **MUST** answer with `notFound` — and nothing more specific — for a handle it never minted, one already answered, one past its `expiresAt`, and one whose step-up was approved, withdrawn or expired in the meantime.
3. **MUST** return as `request` a complete [`approve-request/0.5`](../../../approve-request/0.5/spec.md) document it issues and signs, whose `recipient` is the step-up's `subject` and whose payload equals the inline request it gave the producer — the same `subject`, `challenge`, `boundTo`, `accepts`, `approvers` and `reason`.
4. **MUST** return as `operation` the operation the step-up is bound to, exactly as it received it.
5. **MUST NOT** treat the request, or the reading of the handle, as any part of an approval. A step-up is recorded only from an [`approve-response/0.7`](../../../approve-response/0.7/spec.md) meeting that specification, as for any other delivery.

### The producer (the approver's agent)

A conforming **producer** of this task — the agent that holds, or can reach, the subject's evidence:

1. **MUST** obtain the handle from the person, through a channel the deployment defines (a trigger link, a scanned code), and **MUST** send this task only to the relying party that channel names, verified as that deployment requires.
2. On a response, **MUST** verify `request` as a Trust Task document in its own right ([SPEC §7.2](/SPEC.md#72-consumer-requirements)) — its proof by the relying party it sent this task to, its schema, its `recipient` — and then act as the consumer of [`approve-request/0.5`](../../../approve-request/0.5/spec.md), refusing as that specification states when it does not speak for `subject`.
3. **MUST** show the person the `operation` — what will be done — and not only the relying party's `reason`. Where it knows how the relying party derives `boundTo` from an operation, it **MUST** recompute it from `operation` and `challenge`, and refuse to proceed on a mismatch.
4. **MUST** send the [`approve-response/0.7`](../../../approve-response/0.7/spec.md) to the relying party itself, as the subject, on its own transport. It **MUST NOT** pass the evidence to the producer of the refused operation.

### The producer of the refused operation

A producer that receives `details.stepUpHandoff` and cannot obtain the evidence itself:

1. **SHOULD** present the handle to the person as the deployment's channel defines, and **MUST NOT** present it to anyone else.
2. **MUST NOT** surface `details.stepUpRequest` as something the person approves: it carries no proof. What the person approves is what their agent fetched.
3. **MAY** send the identical operation again once the person has approved; a step-up the relying party recorded is redeemed as for any other delivery.

## Authorization

**Possession of a live handle** is the whole authority for this task, and it authorizes nothing but the read. The relying party gives the handle only to the producer of the refused operation, in its own reply, and that producer gives it only to the person; the handle names one pending step-up and is spent by the first request that uses it (consumer item 1).

The proof on the request establishes which key spent the handle and binds the answer to that document; it is not the authority, and the issuer is typically a fresh key the agent generated for this exchange. Neither the proof nor the handle confers any part of the step-up: an approval still takes an [`approve-response/0.7`](../../../approve-response/0.7/spec.md) signed by the subject with the subject's own factor, verified by the relying party against the challenge it issued. A party that obtains a handle — by watching a screen — learns the operation and can deny the person their read by spending it first, which the person sees as a refusal, but it cannot approve.

## Definitions

- **Hand-off** — a relying party's offer, carried as `details.stepUpHandoff` in an inline step-up refusal, of a handle by which the subject's approver agent fetches the pending step-up.
- **Handle** — the single-use reference `$defs.Handle` describes.
- **Approver's agent** — the wallet, or other agent, that holds or can reach the subject's evidence for a step-up: their passkey, or one of their bound step-up approvers.

## Request

The approver's agent sends the handle to the relying party. The payload is the top-level schema of `payload.schema.json`.

### The person's wallet fetches a step-up their assistant was refused

```json
{
  "id": "urn:uuid:2b7c1f0e-4a3d-4c1b-9e6f-0d8a5b2c7e41",
  "type": "https://trusttasks.org/spec/auth/step-up/requests/show/0.1",
  "issuer": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "recipient": "did:webvh:QmCommunity:community.example",
  "issuedAt": "2026-11-08T09:14:05Z",
  "parentThreadId": "Hk2pQ9xV4mT7rW1sZ8yN3A",
  "payload": {
    "handle": "Hk2pQ9xV4mT7rW1sZ8yN3A"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-08T09:14:05Z",
    "verificationMethod": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH#z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
    "proofPurpose": "authentication",
    "proofValue": "z3FXQjecWufY46yg5abdVZsXqLhxhueuSoZgNSARiKBk9czhSePTFehP8c3PGfb6a22gkfUKhG5tlpjngvqpAMvHH"
  }
}
```

## Response

The relying party answers with the pending step-up and the operation it is bound to. The payload is the sub-schema reachable via `$anchor: "response"` in `payload.schema.json`. A handle that answers nothing is refused with `trust-task-error` and the code `auth/step-up/requests/show:notFound`.

### The pending grant, signed by the community and addressed to Alice

```json
{
  "id": "urn:uuid:5e0d9c2a-1f4b-4d8e-a7c3-6b2e9f1d0a58",
  "type": "https://trusttasks.org/spec/auth/step-up/requests/show/0.1#response",
  "threadId": "urn:uuid:2b7c1f0e-4a3d-4c1b-9e6f-0d8a5b2c7e41",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "issuedAt": "2026-11-08T09:14:06Z",
  "payload": {
    "request": {
      "id": "urn:uuid:9c4e2a71-0b8d-4f36-a5e2-1d7c3b9f8e20",
      "type": "https://trusttasks.org/spec/auth/step-up/approve-request/0.5",
      "issuer": "did:webvh:QmCommunity:community.example",
      "recipient": "did:webvh:QmAlice:people.example:alice",
      "issuedAt": "2026-11-08T09:14:06Z",
      "expiresAt": "2026-11-08T09:18:51Z",
      "payload": {
        "subject": "did:webvh:QmAlice:people.example:alice",
        "boundTo": "zQmTQ8NqgyH4hZxEb4cR5vX9fP2kW7yL3mJ6nB1sD8tG0aV",
        "challenge": "q3XKtZJ0p8m4Ud2vWcH7Rs1aLyN6bEf9",
        "reason": "Grant moderator to did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "accepts": ["approverSigned"],
        "approvers": ["did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2"]
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "created": "2026-11-08T09:14:06Z",
        "verificationMethod": "did:webvh:QmCommunity:community.example#key-0",
        "proofPurpose": "authentication",
        "proofValue": "z4oey5q2M3XKaDup3tmzN1DZyPNqtHmvPwJcXFbQGbVmE6Bn6x1o8PdnGVxXw5mX7tUUpLu7f2gvDQ2HNg2UZn1Ah"
      }
    },
    "operation": {
      "type": "https://trusttasks.org/spec/acl/grant/0.2",
      "payload": {
        "entry": {
          "subject": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
          "role": "moderator",
          "act": { "scope": "all" },
          "capabilities": { "scope": "ceiling" },
          "keys": { "scope": "none" }
        },
        "reason": "Moderation help for the autumn intake"
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-08T09:14:06Z",
    "verificationMethod": "did:webvh:QmCommunity:community.example#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z5Vb4MRxQrmyQUnqF7zNx8vH3a2s1CkpWdYtLe9jG6oBu8iN4rT2cXhKfDqP7wS1mE3vA9bZ6nUyJ5gL0tR8kH4xQ"
  }
}
```

### The handle was already spent

```json
{
  "id": "urn:uuid:7a1f3e9b-2c5d-4e60-b8a4-0f9d6c3b1e72",
  "type": "https://trusttasks.org/spec/trust-task-error/0.3",
  "threadId": "urn:uuid:2b7c1f0e-4a3d-4c1b-9e6f-0d8a5b2c7e41",
  "issuer": "did:webvh:QmCommunity:community.example",
  "recipient": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
  "issuedAt": "2026-11-08T09:14:06Z",
  "payload": {
    "code": "auth/step-up/requests/show:notFound",
    "message": "no pending step-up answers to this handle"
  }
}
```

### The hand-off a refusal carries

Not a document of this task: the `details` of the refusal the relying party sent the producer of the operation — here an assistant whose key Alice delegated — beside the inline request. The `details.stepUpHandoff` object is `$defs.Handoff`.

```json
{
  "code": "permissionDenied",
  "message": "this operation needs a step-up from did:webvh:QmAlice:people.example:alice",
  "details": {
    "stepUpRequest": {
      "subject": "did:webvh:QmAlice:people.example:alice",
      "boundTo": "zQmTQ8NqgyH4hZxEb4cR5vX9fP2kW7yL3mJ6nB1sD8tG0aV",
      "challenge": "q3XKtZJ0p8m4Ud2vWcH7Rs1aLyN6bEf9",
      "reason": "Grant moderator to did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
      "accepts": ["approverSigned"],
      "approvers": ["did:key:z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2"]
    },
    "stepUpHandoff": {
      "handle": "Hk2pQ9xV4mT7rW1sZ8yN3A",
      "expiresAt": "2026-11-08T09:18:51Z",
      "show": "https://trusttasks.org/spec/auth/step-up/requests/show/0.1"
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries a handle and nothing else. The response carries the subject's DID, the relying party's challenge and binding, the evidence kinds and approvers it accepts, its reason, and the operation — whose payload can name other people, such as the subject of a grant. That is the least an agent can show a person and still let them decide what they are approving; the reason alone is the relying party's description of the act, and the operation is the act. A relying party **MUST NOT** add to the response anything the inline request did not carry other than the operation.

### Correlation

The agent's issuer is typically a fresh key generated for this exchange, so the request links to nothing else the person does. The relying party learns, from the approve-response that follows, that this agent speaks for the subject — which it already knows from the subject's DID. The handle links the refused operation's producer to the person's agent for the length of one step-up, and to nothing after it.

The relying party's identifier scope is public because the agent must check two things against it: that the deployment's channel named this relying party, and that the `approve-request` it receives is signed by the same party it sent the handle to. A pairwise identifier would leave the agent nothing to compare the request's issuer with, and the person nothing recognisable to see as the party asking. The relying party already uses this identifier for the operation and for the approve-response, so the task adds no correlation through it.

### Retention

The relying party keeps the handle and the operation only while the pending step-up lives (`retention: exchange`), and deletes them when it is answered, approved, withdrawn or expires. The agent keeps what it fetched until the person decides and **SHOULD** discard it then. The approval itself is recorded and audited as [`approve-response/0.7`](../../../approve-response/0.7/spec.md) states.

### Consent/purpose

The person consents to one operation, shown to them by their own agent from the relying party's signed request and the operation itself — not from anything the producer of the operation said. Reading the request is not consent: nothing is approved until the person gives their own evidence and the relying party verifies it. Because the handle is single-use, a party that spends it first denies the person their read, which the person sees and can recover from by having the operation sent again; it cannot make the approval, which takes the person's own factor.
