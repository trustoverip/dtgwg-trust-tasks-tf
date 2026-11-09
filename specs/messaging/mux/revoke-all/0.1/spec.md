---
slug: messaging/mux/revoke-all
version: "0.1"
title: "Messaging — Mux Revoke All"
summary: "The hand-over switch: a member DID proves current control with a signed revoke-all statement and advances its epoch at the mediator, voiding every grant on it at once and detaching it from every principal before the mediator answers."
status: draft
targetFrameworkVersion: "0.6.0"
category: messaging
keywords: [messaging, mediator, mux, multiplexing, revoke, epoch, hand-over, revocation]
parties:
  - role: Requester (the member DID, or a party carrying its statement)
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: Mediator
    requirement: REQUIRED
    member: recipient
    identifierScope: public
  - role: Member DID (the revoke-all statement's issuer)
    requirement: REQUIRED
    identifierScope: any
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    Revoke-all is irreversible: every grant on the member DID dies, and none can be resumed. The member's own signed statement is the authority; the document proof attributes the submission, so the record of who triggered a hand-over does not rest on transport logs. The response is the requester's evidence of the epoch now in force.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task is destructive, which makes it consequential (SPEC §7.3 item 17). The statement's epoch already makes a replay inert once the epoch has advanced; issuedAt bounds the submission as well.
sideEffects:
  level: destructive
  rationale: >-
    Advances the member DID's epoch, so every grant signed under that epoch or an earlier one becomes `epochRevoked` and can never be resumed. Every principal carrying the DID through a grant is detached before the response.
consequences:
  - Every principal holding a grant on this DID stops receiving its messages, on every connection, before this request is answered — including principals the requester does not know about.
  - Each affected principal receives a durable grantsRevoked notice; new grants must be signed under the new epoch.
subjectPath: /revocation/issuer
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: >-
    The task exercises the member DID's own authority over every delegation of it, proven by a statement signed with a key currently in its authentication relationship. The response discloses the new epoch and how many grants were voided.
retention:
  class: durable
  rationale: >-
    The epoch counter is kept for the life of the member's account, and the voided grants stay listable, in state `epochRevoked`, for a retention window after their effective expiries. The revocation is audited.
errorCodes:
  - code: messaging/mux:proofPurposeInvalid
    meaning: "The embedded proof's `proofPurpose` is not `authentication`."
    retryable: false
  - code: messaging/mux:statementProofInvalid
    meaning: "The embedded Data Integrity proof does not verify \u2014 a bad signature, a verification method not controlled by the statement's `issuer`, or one not in the issuer's current `authentication` relationship."
    retryable: false
  - code: messaging/mux:didUnresolvable
    meaning: "The member DID could not be resolved now, so the statement could not be checked. Retry later."
    retryable: true
  - code: messaging/mux:audienceMismatch
    meaning: "The statement's `audience` is not this mediator's DID."
    retryable: false
  - code: messaging/mux:notYetValid
    meaning: "The statement's `iat` is more than 60 seconds in the mediator's future."
    retryable: false
  - code: messaging/mux:statementExpired
    meaning: "The statement's `exp` has passed."
    retryable: false
  - code: messaging/mux:lifetimeTooLong
    meaning: "The statement's `exp` lies further after `iat` than the mediator accepts for a revoke-all statement."
    retryable: false
  - code: messaging/mux/revoke-all:epochMismatch
    meaning: "The statement names an epoch later than the member DID's current one. `details.currentEpoch` carries the epoch in force; re-sign under it."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [currentEpoch]
      properties:
        currentEpoch:
          type: integer
          minimum: 0
  - code: messaging/mux/revoke-all:contended
    meaning: "The member DID's record changed under concurrent writes too many times for the epoch advance to be committed. Nothing changed; send the request again."
    retryable: true
related:
  - messaging/mux/revoke
  - messaging/mux/grant
  - messaging/mux/list
  - messaging/mux/notice
---

## Abstract

The **Messaging — Mux Revoke All** Trust Task is the **hand-over switch**. Every stored grant ([`messaging/mux/grant`](../../grant/0.1/spec.md)) embeds the member DID's **epoch** at the mediator, and a grant whose epoch is not the current one is dead. This task advances the epoch. With one message, whoever controls a DID *now* voids every delegation of it, including delegations to principals it has never heard of, and detaches the DID from every principal carrying it **before** the mediator answers.

The authority is a statement signed, with an embedded Data Integrity proof, by a key currently in the member DID's `authentication` relationship, naming the epoch to void. It is typically sent by the DID's new controller, after [`messaging/mux/list`](../../list/0.1/spec.md) as `member` has shown it what existed. Revoking one principal's grants, rather than all of them, is [`messaging/mux/revoke`](../../revoke/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is the mediator DID multiplexing design (VTI design docs, `mediator-did-multiplexing-design.md`, §4.8, §5.2, §5.3 and §7.3).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### The revoke-all statement

The statement is a closed JSON object, [`RevokeAllStatement`](../../_shared/0.1/mux.schema.json#/$defs/RevokeAllStatement), signed by the member DID with an embedded W3C Data Integrity proof ([`MuxDataIntegrityProof`](../../_shared/0.1/mux.schema.json#/$defs/MuxDataIntegrityProof); cryptosuite `eddsa-jcs-2022` or `eddsa-rdfc-2022`) whose `proofPurpose` **MUST** be `authentication`:

```json
{
  "type": "muxRevokeAll",
  "issuer": "<member DID>",
  "audience": "<mediator DID>",
  "epoch": 3,
  "iat": 1794222000,
  "exp": 1794222300,
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "<member DID>#key-2",
    "proofPurpose": "authentication",
    "proofValue": "z…"
  }
}
```

`epoch` is the epoch to void — the current one, as [`messaging/mux/list`](../../list/0.1/spec.md) as `member` reports it. Binding the statement to an epoch is what makes it single-use: once the epoch has advanced, the same statement voids nothing more. `iat` and `exp` are JWT NumericDate values ([RFC 7519 §2](https://www.rfc-editor.org/rfc/rfc7519#section-2)); a short `exp`, minutes rather than hours, is RECOMMENDED.

A conforming **producer** **MUST** sign the statement with a key in the member DID's **current** `authentication` relationship and name the member DID's current epoch.

A conforming **consumer** (the mediator) **MUST**:

1. Answer `unsupportedType` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) when mux is not enabled.
2. Check the statement's **shape** and **type** as part of validating the payload ([SPEC §7.2](/SPEC.md#72-consumer-requirements)) — a closed `RevokeAllStatement` whose `type` is `muxRevokeAll`, else `malformedRequest` — and then, **in this order**, respond with the first failure's code: **proof purpose** is `authentication` (`messaging/mux:proofPurposeInvalid`); **Data Integrity verification** — the proof verifies, its verification method is controlled by `issuer` and listed in the issuer's **current** `authentication` relationship (`messaging/mux:statementProofInvalid`, or `messaging/mux:didUnresolvable`); **audience** is this mediator's DID (`messaging/mux:audienceMismatch`); **validity window** — `iat` no more than 60 seconds in the future (`messaging/mux:notYetValid`), `exp` in the future (`messaging/mux:statementExpired`), `exp` within the mediator's bound for such statements (`messaging/mux:lifetimeTooLong`); **epoch** not later than the current epoch (`messaging/mux/revoke-all:epochMismatch`, with `currentEpoch` in `details`).
3. Where `epoch` equals the current epoch: advance it by one, in one atomic write to the member DID's record (where that write cannot complete under concurrent writes after a bounded number of attempts, change nothing and respond `messaging/mux/revoke-all:contended`); move every grant on the member DID signed under an earlier epoch and still `active` to `epochRevoked`; and detach the member from every connection carrying it through such a grant, synchronously — remove the streaming entry, wait for the streaming path to confirm, drop any frame for the member still queued on the connection. Only then respond. On a clustered mediator, publish the detach to every node; correctness on another node comes from the epoch comparison every pickup and delete makes, and the remaining window is bounded to live pushes already in flight there. A frame for the member **MUST NOT** leave a local connection through a voided grant after the response.
4. Where `epoch` is earlier than the current epoch, change nothing and succeed with `epochAdvanced: false`: every grant under that epoch is already void, so the requested state holds.
5. Emit a `detached` [`messaging/mux/event`](../../event/0.1/spec.md) with reason `epochBumped` to each affected principal that is connected, store a `grantsRevoked` [`messaging/mux/notice`](../../notice/0.1/spec.md) with that reason in each one's inbox, and write an audit record.
6. Respond with the epoch now in force, whether it advanced, and how many grants were voided. Session attaches are unaffected: each is bound to its own connection and lapses with it.

Correctness rests on the epoch comparison, not on deleting grants: voided grants are kept, in state `epochRevoked`, for the mediator's retention window after their effective expiries, and are then pruned. A member DID's epoch is never forgotten while its account exists, since forgetting it would revive the grants it voided.

## Authorization

**Authority: current control of the member DID, proven by its own signature.** Whoever can sign with a key in the DID's current `authentication` relationship controls it now, and the controller of a DID is entitled to end every delegation of it — that is the whole point of the hand-over switch. The statement's embedded proof is the authority, and only that proof: the document `issuer`, its `proof` and the transport identify who submitted it ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)) and confer nothing. The member DID or anyone it hands the statement to may submit it. No principal is consulted.

The limit is the DID method. For a DID whose keys cannot rotate — did:key, did:peer — a previous owner who kept the private key can sign this statement too, and so can race the new owner; no protocol can tell two holders of one key apart. DIDs meant to change hands should use a method with key rotation, and rotate on hand-over, after which the old key fails consumer rule 2.

## Definitions

- **`revocation`** — REQUIRED. The [`RevokeAllStatement`](../../_shared/0.1/mux.schema.json#/$defs/RevokeAllStatement); the member DID is its `issuer`.
- **Epoch** — a per-member-DID counter at the mediator, starting at 0, embedded in every grant. Only this task advances it; the unknown-key rule ([`messaging/mux/grant`](../../grant/0.1/spec.md#the-unknown-key-rule)) suspends grants without touching it, which is why a new controller follows a suspension with revoke-all to make a hand-over final.
- **`epoch`** (response) — the epoch now in force; any new grant must carry it.
- **`epochAdvanced`** (response) — false when the named epoch was already void.
- **`revokedGrants`** (response) — how many active grants this request voided.

## Request

The member DID sends the request to the mediator. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A new owner voids every delegation of its DID

```json
{
  "id": "urn:uuid:1e2f3a4b-5c6d-4e7f-8a9b-0c1d2e3f4a01",
  "type": "https://trusttasks.org/spec/messaging/mux/revoke-all/0.1",
  "issuer": "did:webvh:QmA1ice:alice.example:work",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-11-09T11:00:30Z",
  "payload": {
    "revocation": {
      "type": "muxRevokeAll",
      "issuer": "did:webvh:QmA1ice:alice.example:work",
      "audience": "did:web:mediator.example",
      "epoch": 3,
      "iat": 1794222000,
      "exp": 1794222300,
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:webvh:QmA1ice:alice.example:work#key-2",
        "proofPurpose": "authentication",
        "proofValue": "z58DAdFfa9SkqZMVPxAQpic7ndSayn1PzZs6ZjWp1CktyGesjuTSwRdoWhAfGFCF5bppETSTojQCrfFPP2oumHKtz"
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmA1ice:alice.example:work#key-2",
    "created": "2026-11-09T11:00:30Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

The statement, signed by the new owner's `#key-2`, voids epoch 3 and is accepted for five minutes from 2026-11-09T11:00:00Z. The proof values are placeholders.

## Response

The mediator responds after every affected principal has been detached, in a payload that validates against the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` rather than a `#response` document.

### Epoch advanced

```json
{
  "id": "urn:uuid:1e2f3a4b-5c6d-4e7f-8a9b-0c1d2e3f4a02",
  "type": "https://trusttasks.org/spec/messaging/mux/revoke-all/0.1#response",
  "threadId": "urn:uuid:1e2f3a4b-5c6d-4e7f-8a9b-0c1d2e3f4a01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:webvh:QmA1ice:alice.example:work",
  "issuedAt": "2026-11-09T11:00:30Z",
  "payload": {
    "did": "did:webvh:QmA1ice:alice.example:work",
    "epoch": 4,
    "epochAdvanced": true,
    "revokedGrants": 0
  }
}
```

No grant was still active — the unknown-key rule had already suspended the old principal's — so none was voided; the epoch advance still guarantees that a grant the old owner signs under epoch 3 — which suspension alone would not have prevented — is refused.

### Refused: an epoch from the future

```json
{
  "id": "urn:uuid:1e2f3a4b-5c6d-4e7f-8a9b-0c1d2e3f4a03",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "threadId": "urn:uuid:1e2f3a4b-5c6d-4e7f-8a9b-0c1d2e3f4a01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:webvh:QmA1ice:alice.example:work",
  "issuedAt": "2026-11-09T11:00:30Z",
  "payload": {
    "code": "messaging/mux/revoke-all:epochMismatch",
    "message": "The proof names epoch 9; the current epoch is 3.",
    "retryable": false,
    "details": { "currentEpoch": 3 }
  }
}
```

## Security & Privacy

### Data carried

The member DID's statement — the mediator, the DID, an epoch and a short validity window, with its Data Integrity proof — in; the new epoch and a count out. No grant details, no principal DIDs and no message content. A producer **MUST NOT** place anything but extension data its ecosystem defines in `ext`.

### Correlation

The request links nothing new: it names one DID. It tells the mediator, and through the grantsRevoked notices every affected principal, that the DID's controller chose to end all delegations at this moment — which, after a hand-over, is the signal the old owner's devices need. The count in the response tells the new owner how many delegations were still live without naming them. The mediator's identifier scope is public by design: it is the address every principal, member and peer already uses to reach it, and the `aud` every mux proof names.

### Retention

**Durable.** The epoch is kept for the life of the account; voided grants stay listed as `epochRevoked` for a retention window after their effective expiries. The revocation is audited with the proof, which is the evidence that the DID's controller made it.

### Consent/purpose

The purpose is hand-over and compromise recovery: whoever controls a DID now can end every delegation of it, in one message, without knowing or contacting the principals. It needs no principal's cooperation by design.
