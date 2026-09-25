---
slug: sender-delegation
version: "0.1"
title: Sender Delegation
summary: A document's signer names the one transport sender that may carry specific Trust Task documents to one consumer, for at most five minutes, so a consumer that binds signer to sender can accept them without relaxing that rule.
status: draft
targetFrameworkVersion: "0.5.0"
category: authentication
keywords:
  - delegation
  - transport
  - sender
  - didcomm
  - tsp
  - persona
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Signer
    requirement: REQUIRED
    member: issuer
  - role: Consumer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The delegation is the only evidence that the signer, and not the sender,
    chose this sender. An unsigned delegation is a sender vouching for itself,
    which is the claim a consumer that binds signer to sender refuses to take.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A delegation is valid for at most 300 seconds from its issue time. Without
    an issue time there is no window to bound, and a captured delegation would
    let its sender carry the listed documents indefinitely.
sideEffects:
  level: none
  rationale: "Carried beside the documents it covers; never dispatched as a task, and creates no state at the consumer."
exposure:
  discloses: metadata
  actsAsSubject: false
  rationale: >-
    Discloses to the consumer that the signer and the sender are linked: the
    consumer learns which transport identity carries the signer's documents.
    Where the signer is a pairwise identifier, that link is exactly the
    correlation the pairwise identifier exists to avoid, and a signer that cannot
    accept it does not delegate.
errorCodes: []
related:
  - auth/authenticate
  - vault/sign-trust-task
---

## Abstract

A **Sender Delegation** lets the party that signed a *Trust Task document* name a different party to deliver it. It exists for consumers that bind the two: over an authenticated transport such as [DIDComm](../../../bindings/didcomm/0.2/spec.md) or [TSP](../../../bindings/tsp/0.1/spec.md), such a consumer acts on a document only when its proven signer **is** the transport-authenticated sender, and refuses anything else as `identityMismatch`.

That rule is right, and this specification does not weaken it. It covers one case the rule leaves no route for: a signer that cannot send. The motivating example is a wallet signing in to a relying party as a per-site persona whose keys, key agreement included, are held by the persona's maintainer and never released. The maintainer can sign a document as the persona (for example with [`vault/sign-trust-task`](../../vault/sign-trust-task/0.2/spec.md)); it cannot be the DIDComm or TSP sender of the wallet's message. The wallet can send, but only as itself.

A Sender Delegation is signed by the document's signer and names exactly one sender, one consumer, the documents covered, and a lifetime of at most five minutes. A consumer that accepts it treats the covered documents as sent by their signer. The sender gains no authority from it.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

### The delegation document

A Sender Delegation is a *Trust Task document* whose `type` is `https://trusttasks.org/spec/sender-delegation/0.1`, with:

| Member | Requirement |
|---|---|
| `issuer` | REQUIRED. The signer of every document the delegation covers. |
| `recipient` | REQUIRED. The one consumer the covered documents are addressed to. |
| `issuedAt` | REQUIRED. |
| `expiresAt` | REQUIRED, and **MUST NOT** be more than 300 seconds after `issuedAt`. |
| `payload.delegate` | REQUIRED. The bare DID of the one transport sender permitted to carry the covered documents. |
| `payload.documents` | REQUIRED. The `id` of each covered document; one to sixteen entries. |
| `proof` | REQUIRED, with a `verificationMethod` controlled by `issuer` ([SPEC.md §4.7](/SPEC.md#47-proof)), `proofPurpose` **`authentication`**, and a `verificationMethod` the issuer's DID document lists under `authentication`. The delegation stands in for the transport authentication the signer cannot perform, so it is the signer authenticating, not attesting. |

A delegation is never dispatched. It travels beside the documents it covers, in the carriage the binding defines for it ([DIDComm §5.1](../../../bindings/didcomm/0.2/spec.md#51-sender-delegation), [TSP §5.5](../../../bindings/tsp/0.1/spec.md#55-sender-delegation)). A consumer that receives one as the dispatched document of a message **MUST** reject it with `malformedRequest`.

### Producer requirements

A conforming **producer** (the sender) **MUST**:

1. Obtain the delegation from the signer. The signer **MUST** sign it only for a sender it has chosen, and only for documents it has also signed.
2. Name as `payload.delegate` a transport identity used for no other signer and no other consumer (a pairwise transport identifier) whenever the signer is itself pairwise, such as a per-site persona. See [Correlation](#correlation).
3. List every carried document's `id` in `payload.documents`. A producer that needs to cover a sequence it has not yet signed (a challenge request and the authenticate that spends it) **MAY** assign the documents' ids in advance and list them all.
4. Carry the delegation in the same transport message as each covered document, using the binding's carriage.
5. Not carry a delegation where its own transport identity is the document's signer. A delegation is for the case the direct rule cannot express, not a second way to express the one it can.

### Consumer requirements

A consumer is never obliged to accept a delegation. A consumer that does accept one **MUST**, for each carried document whose proven signer is not the transport-authenticated sender, accept the document only when **all** of the following hold, and otherwise **MUST** reject it with `identityMismatch` ([SPEC.md §4.8.1](/SPEC.md#481-precedence-of-in-band-over-transport-derived-identity)):

1. The document itself passes the consumer's ordinary checks, including its own `proof` verifying as its `issuer`.
2. A delegation accompanies it in the binding's carriage, its `type` is this specification's, and its `proof` verifies as its `issuer` with `proofPurpose` `authentication`, by a `verificationMethod` listed under `authentication` in the issuer's DID document.
3. The delegation's `issuer` equals the document's `issuer`, by exact string equality.
4. The delegation's `payload.delegate` equals the transport-authenticated sender, compared as a bare DID by exact string equality.
5. The delegation's `recipient` equals the consumer's own identifier, and equals the document's `recipient`.
6. The consumer's clock is at or after the delegation's `issuedAt` (allowing the consumer's ordinary skew tolerance) and before its `expiresAt`, and `expiresAt` is no more than 300 seconds after `issuedAt`.
7. The document's `id` appears in the delegation's `payload.documents`.

A consumer that accepts a delegated document:

* **MUST** authorize it as its `issuer`, exactly as if the issuer had sent it. The delegate is a carrier and acquires no authority, standing, or ACL identity from the delegation.
* **MUST** apply its ordinary replay protection to the document, keyed on its `issuer` and `id`. A delegation does not make a covered document replayable, and a replayed delegation covers only documents that replay protection already refuses.
* **MUST** route any response, and any `trust-task-error` under `identityMismatch`, to the transport-authenticated sender, the only party able to open it. The response document's own `recipient` names the document's `issuer` as usual.
* **MAY** decline delegation for any task type, and **SHOULD** decline it for tasks whose purpose is to prove who is present, beyond signing in: a step-up approval or a consent decision is evidence that its signer acted, and a delegate carrying it adds nothing but a second party to trust.

### Signer requirements

A signer's maintainer (the party holding its keys, for example a VTA answering [`vault/sign-trust-task`](../../vault/sign-trust-task/0.2/spec.md)) that signs a delegation **MUST**:

1. Sign it only under the same authorization, consent and step-up gates it applies to signing the covered documents themselves. A delegation that is cheaper to obtain than the documents it covers is the weaker door.
2. Sign it only for a `payload.delegate` the requester is entitled to name, and **SHOULD** require that to be the requester's own transport identity, or one the requester has proven control of.
3. Not sign one with an `expiresAt` more than 300 seconds after its `issuedAt`.

## Definitions

* **Signer.** The party whose `proof` is on a carried document, identified by its `issuer`. It issues the delegation.
* **Sender** (the *delegate*). The transport-authenticated sender of the message that carries the documents.
* **Consumer.** The party the covered documents are addressed to; the delegation's `recipient`.

## Payload

`payload.delegate` (REQUIRED) is the bare DID of the one sender the delegation permits.

`payload.documents` (REQUIRED) lists the `id` of each covered document.

`payload.ext` (optional) is the extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

The full JSON Schema is in [`payload.schema.json`](payload.schema.json).

## Examples

### A persona signing in through its wallet

The persona `did:webvh:zPersona…:agent.example:contexts:shop` delegates carriage of its `auth/challenge` and `auth/authenticate` documents to a transport identity the wallet uses for this persona at this relying party only, `did:peer:2.Ez6LS….Vz6Mk…`, for sign-in at `did:webvh:zRp…:rp.example`.

```json
{
  "id": "urn:uuid:5c0e2a91-7b4d-4f3e-a1c8-9d2f6e0b7a35",
  "type": "https://trusttasks.org/spec/sender-delegation/0.1",
  "issuer": "did:webvh:zPersona:agent.example:contexts:shop",
  "recipient": "did:webvh:zRp:rp.example",
  "issuedAt": "2026-09-25T10:00:00Z",
  "expiresAt": "2026-09-25T10:02:00Z",
  "payload": {
    "delegate": "did:peer:2.Ez6LSexample.Vz6Mkexample",
    "documents": [
      "urn:uuid:3b1d7f0e-8a52-4c1e-9d3a-6f0b2c7e5a14",
      "urn:uuid:9e4a6c02-1f7b-4d8e-b3a5-0c2d8f6e1b97"
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:zPersona:agent.example:contexts:shop#key-1",
    "created": "2026-09-25T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z4sT…"
  }
}
```

## Security & Privacy

**What is delegated is carriage, and only carriage.** Every covered document still carries its signer's own proof, and the consumer still authorizes it as its signer. A consumer that let a delegation stand in for a document's proof would turn it into a bearer credential; the consumer requirements rule that out by checking the document first.

**Why it is this narrow.** The rule a delegation works around exists because a transport's report of its sender is not a proof, and a proof by one party paired with another party's envelope is the shape an attacker produces. A delegation keeps both halves signed by the signer: the documents, and the choice of who carries them. Naming the documents, the one sender, the one consumer and a five-minute ceiling means a leaked delegation lets its holder deliver only documents the signer already signed, to the one party they were addressed to, briefly, and replay protection refuses any second delivery.

### Data carried

The signer's and the sender's identifiers, the consumer's, and the `id`s of the covered documents. Nothing of the covered documents' content. A producer **MUST NOT** place anything in `ext` it would not show the consumer, which reads it in full.

### Correlation

A delegation tells the consumer which transport identity carries the signer's documents. For a pairwise signer carried by a long-lived sender (a wallet's own DID, shared by every persona it holds), that is the correlation the pairwise identifier exists to prevent: every consumer the wallet delegates to learns the same sender for otherwise unlinkable personas. That is why a producer names a pairwise delegate ([Producer requirements](#producer-requirements) item 2), and why the signer is the party to decide whether to accept the link. A signer that cannot accept it does not delegate, and uses a binding that has no transport sender to bind, such as HTTPS.

Only the sender can open a response carried by the transport, so every response to a delegated document is visible to the delegate. That is inherent in having one; a signer that must keep its responses from the delegate does not delegate.

### Retention

A delegation has no value past its `expiresAt`, and nothing about it needs keeping beyond the covered documents' own replay records. A consumer that logs it for audit keeps it as evidence of who carried what, not as authority.

### Consent/purpose

The signer consents to one sender carrying named documents to one consumer, and to nothing else. A consumer **MUST NOT** read a delegation as consent to any other sender, document or consumer, or as the signer's approval of what the covered documents ask for, which the documents' own proofs express.
