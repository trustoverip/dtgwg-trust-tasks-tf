---
slug: messaging/mux/admin-revoke-all
version: "0.1"
title: "Messaging — Mux Admin Revoke All"
summary: "An administrator advances a member DID's epoch at the mediator without the member's signature, voiding every grant on it, ending its session attaches, and detaching it from every principal, with reason admin, before the mediator answers."
status: draft
targetFrameworkVersion: "0.6.0"
category: messaging
keywords: [messaging, mediator, mux, multiplexing, revoke, epoch, revocation, admin, operator]
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: Mediator
    requirement: REQUIRED
    member: recipient
    identifierScope: public
  - role: Member DID (whose epoch is advanced)
    requirement: REQUIRED
    identifierScope: any
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The epoch advance is irreversible — every grant on the member DID dies and none can be resumed — and is made without the member's signature, so the document proof is the only transport-independent record of which administrator made it. The response is the administrator's evidence of the epoch now in force.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task is destructive, which makes it consequential (SPEC §7.3 item 17). The request's epoch already makes a replay inert once the epoch has advanced; issuedAt bounds the submission as well.
sideEffects:
  level: destructive
  rationale: >-
    Advances the member DID's epoch, so every grant signed under that epoch or an earlier one becomes `epochRevoked` and can never be resumed, and ends every session attach of the DID admitted before it. Every principal carrying the DID, through a grant or an attach, is detached before the response.
consequences:
  - Every principal carrying this DID, through a grant or a session attach, stops receiving its messages, on every connection, before this request is answered.
  - A principal that carried the DID through a session attach must attach again with a new attach statement.
  - Each principal whose grant was voided receives a durable grantsRevoked notice with reason admin; new grants must be signed by the member DID under the new epoch.
subjectPath: /did
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The administrator acts on its own standing at the mediator, not on the member DID's authority. The response discloses the member DID's new epoch and how many grants were voided.
retention:
  class: durable
  rationale: >-
    The epoch counter is kept for the life of the member's account, and the voided grants stay listable, in state `epochRevoked`, for a retention window after their effective expiries. The epoch advance is audited.
errorCodes:
  - code: messaging/mux/admin-revoke-all:protectedAccount
    meaning: The member DID is a protected account — a rootAdmin account — and the requester is not a rootAdmin. Nothing changed.
    retryable: false
  - code: messaging/mux/admin-revoke-all:unknownAccount
    meaning: The member DID has no account at this mediator, so it has no epoch to advance.
    retryable: false
  - code: messaging/mux/admin-revoke-all:epochMismatch
    meaning: "The request names an epoch later than the member DID's current one. `details.currentEpoch` carries the epoch in force; send it."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [currentEpoch]
      properties:
        currentEpoch:
          type: integer
          minimum: 0
  - code: messaging/mux/admin-revoke-all:contended
    meaning: "The member DID's record changed under concurrent writes too many times for the epoch advance to be committed. Nothing changed; send the request again."
    retryable: true
related:
  - messaging/mux/admin-list
  - messaging/mux/admin-revoke
  - messaging/mux/revoke-all
  - messaging/mux/connections
  - messaging/mux/notice
  - messaging/mux/event
---

## Abstract

The **Messaging — Mux Admin Revoke All** Trust Task is the operator's form of [`messaging/mux/revoke-all`](../../revoke-all/0.1/spec.md). Every stored grant ([`messaging/mux/grant`](../../grant/0.1/spec.md)) embeds the member DID's **epoch** at the mediator, and a grant whose epoch is not the current one is dead. `revoke-all` advances the epoch on the member's own signed statement; this task advances it on an administrator's standing, for a member DID whose controller has lost its keys, whose keys are known to be compromised, or whose delegations the operator must end at once.

With one message the administrator voids every delegation of the DID, ends every session attach of it, and detaches it from every principal carrying it **before** the mediator answers, with reason `admin`. The administrator names the epoch to void, as [`messaging/mux/admin-list`](../../admin-list/0.1/spec.md) with `member` reported it, so a delayed or repeated request never advances the epoch twice. Revoking some of a DID's grants rather than all is [`messaging/mux/admin-revoke`](../../admin-revoke/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is the mediator DID multiplexing design (VTI design docs, `mediator-did-multiplexing-design.md`, §5.2, §7.3 and §7.8).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the administrator) **MUST** emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/mux/admin-revoke-all/0.1`, with itself as `issuer` and the mediator as `recipient`, carrying `issuedAt` and a `proof`, and naming in `epoch` the member DID's current epoch.

A conforming **consumer** (the mediator) **MUST**:

1. Answer `unsupportedType` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) when mux is not enabled.
2. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), verify the `proof`, and reject a request outside its freshness window.
3. Respond with `permissionDenied` unless the requester's account type is `admin` or `rootAdmin`. The mediator's own `mediator` account may read epochs with [`messaging/mux/admin-list`](../../admin-list/0.1/spec.md) but not advance them.
4. Where the requester is not a `rootAdmin` and `did` is a `rootAdmin` account, refuse and respond with `messaging/mux/admin-revoke-all:protectedAccount`. A `rootAdmin` requester may advance any member DID's epoch. Respond `messaging/mux/admin-revoke-all:unknownAccount` when `did` has no account at the mediator, and `messaging/mux/admin-revoke-all:epochMismatch`, with `currentEpoch` in `details`, when `epoch` is later than the member DID's current epoch.
5. Where `epoch` equals the current epoch, advance it exactly as [`messaging/mux/revoke-all`](../../revoke-all/0.1/spec.md#conformance) consumer rule 3 requires: by one, in one atomic write to the member DID's record (where that write cannot complete under concurrent writes after a bounded number of attempts, change nothing and respond `messaging/mux/admin-revoke-all:contended`); move every grant on the member DID signed under an earlier epoch and still `active` to `epochRevoked`; and detach the member from every connection carrying it through such a grant, or through a session attach admitted before this request, synchronously, before responding, with the same cluster rules and bounded window. A frame for the member **MUST NOT** leave a local connection through a voided grant or an ended attach after the response.
6. Where `epoch` is earlier than the current epoch, change nothing and succeed with `epochAdvanced: false`: every grant under that epoch is already void, so the requested state holds.
7. Emit a `detached` [`messaging/mux/event`](../../event/0.1/spec.md) with reason `admin` to each affected principal that is connected, store a `grantsRevoked` [`messaging/mux/notice`](../../notice/0.1/spec.md) with reason `admin` in the inbox of each principal whose grant was voided, and write an audit record naming the administrator, the member DID and the epochs before and after. A principal that carried the DID only through a session attach was connected when it ended, so the `detached` event is its report; it gets no notice.
8. Respond with the epoch now in force, whether it advanced, and how many grants were voided. Session attaches ended under rule 5 are not counted in `revokedGrants`.

The epoch advance is the same state change `messaging/mux/revoke-all` makes, so everything that rests on the epoch rests on it identically: voided grants are kept, in state `epochRevoked`, for the mediator's retention window after their effective expiries, a member DID's epoch is never forgotten while its account exists, and a grant the member signs under a voided epoch is refused. Only the reason differs — `admin` where the member's own advance reports `epochBumped` — so that a principal can tell an operator's action from the member's.

## Authorization

The entitlement is **administrative standing at the mediator**: the requester's account holds the `admin` or `rootAdmin` role (Consumer rule 3). Unlike the read-only [`messaging/mux/connections`](../../connections/0.1/spec.md) and [`messaging/mux/admin-list`](../../admin-list/0.1/spec.md), the mediator's own `mediator` account is not entitled: an epoch advance is destructive, and an operator, not the service, decides it. A `rootAdmin` account is protected from an `admin` (Consumer rule 4), as [`messaging/account/remove`](../../../account/remove/0.1/spec.md) protects it: only a `rootAdmin` may advance a `rootAdmin`'s epoch. It is the operator's authority over what the mediator carries, and it is distinct from the authority `messaging/mux/revoke-all` rests on — current control of the member DID, proven by its own signature. Here no member signature is involved and the `did` in the request proves nothing about who controls it; the member DID never consented, which is why every affected principal learns of it with reason `admin`, and the DID's controller sees the new epoch the next time it lists or grants.

The `proof` establishes *who* asked and that the request is unaltered, never that they may ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); the mediator resolves the verified issuer to its account and decides from the role it finds there. An epoch advance only takes away: it cannot create a grant or move a member's traffic to a principal, and the next grant still needs the member's own signature under the new epoch.

## Definitions

- **`did`** — REQUIRED. The member DID ([`MuxDid`](../../_shared/0.1/mux.schema.json#/$defs/MuxDid)) whose epoch is advanced.
- **`epoch`** — REQUIRED. The epoch to void: the member DID's current one. Naming it makes the request single-use, exactly as the epoch in a [`RevokeAllStatement`](../../_shared/0.1/mux.schema.json#/$defs/RevokeAllStatement) does.
- **`epoch`** (response) — the epoch now in force.
- **`epochAdvanced`** (response) — false when the named epoch was already void.
- **`revokedGrants`** (response) — how many active grants this request voided.

## Request

The administrator sends the request to the mediator. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Void every delegation of a compromised DID

```json
{
  "id": "urn:uuid:9c3d4e5f-6071-4c8d-8e9f-1a2b3c4d5e01",
  "type": "https://trusttasks.org/spec/messaging/mux/admin-revoke-all/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-11-13T09:10:00Z",
  "payload": {
    "did": "did:webvh:QmA1ice:alice.example:work",
    "epoch": 3
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-11-13T09:10:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The mediator responds after every affected principal has been detached, in a payload that validates against the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)) rather than a `#response` document.

### Epoch advanced

```json
{
  "id": "urn:uuid:9c3d4e5f-6071-4c8d-8e9f-1a2b3c4d5e02",
  "type": "https://trusttasks.org/spec/messaging/mux/admin-revoke-all/0.1#response",
  "threadId": "urn:uuid:9c3d4e5f-6071-4c8d-8e9f-1a2b3c4d5e01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-11-13T09:10:00Z",
  "payload": {
    "did": "did:webvh:QmA1ice:alice.example:work",
    "epoch": 4,
    "epochAdvanced": true,
    "revokedGrants": 2
  }
}
```

### Refused: an epoch from the future

```json
{
  "id": "urn:uuid:9c3d4e5f-6071-4c8d-8e9f-1a2b3c4d5e03",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "threadId": "urn:uuid:9c3d4e5f-6071-4c8d-8e9f-1a2b3c4d5e01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-11-13T09:10:00Z",
  "payload": {
    "code": "messaging/mux/admin-revoke-all:epochMismatch",
    "message": "The request names epoch 9; the current epoch is 3.",
    "retryable": false,
    "details": { "currentEpoch": 3 }
  }
}
```

## Security & Privacy

### Data carried

A member DID and an epoch in; the new epoch and a count out. No grant details, no principal DIDs, no signed statement and no message content. A producer **MUST NOT** place anything but extension data its ecosystem defines in `ext`.

### Correlation

The request names one DID the mediator already holds. Through the `grantsRevoked` notices it tells every principal whose grant was voided that an operator ended the DID's delegations at this moment — not which administrator. The count in the response tells the administrator how many delegations were still live without naming them; [`messaging/mux/admin-list`](../../admin-list/0.1/spec.md) names them. The mediator's identifier scope is public by design: it is the `audience` every grant its members sign names.

### Retention

**Durable.** The epoch is kept for the life of the account; voided grants stay listed as `epochRevoked` for a retention window after their effective expiries. The epoch advance is audited with the request's proof, which is the evidence of which administrator made it.

### Consent/purpose

The purpose is compromise recovery and operating the mediator when the member DID's controller cannot act: a controller who has lost every key that could sign a `revoke-all` statement, a DID whose keys are known to be in the wrong hands, or an operator's policy that requires ending a DID's delegations at once. The member DID does not consent, by design; the mediator tells every affected principal with reason `admin`. For a DID whose controller can still sign, `messaging/mux/revoke-all` is the right task, and an operator **SHOULD** prefer it.
