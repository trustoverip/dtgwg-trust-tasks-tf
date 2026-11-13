---
slug: messaging/mux/admin-revoke
version: "0.1"
title: "Messaging — Mux Admin Revoke"
summary: "An administrator revokes stored mux grants of other accounts — every grant of a principal DID, of a member DID, or between the two, or named grants of a member — detaching affected members from every connection, with reason admin, before the mediator answers."
status: draft
targetFrameworkVersion: "0.6.0"
category: messaging
keywords: [messaging, mediator, mux, multiplexing, grant, revoke, revocation, admin, operator]
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: Mediator
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    Revocation is irreversible — a revoked grant can only be replaced by a fresh member signature — and here it is made by neither party to the grant, so the document proof is the transport-independent record of which administrator ended the delegation. The response is the administrator's evidence that it took effect.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task ends stored delegations, which makes it consequential (SPEC §7.3 item 17). A delayed copy naming a principal or member DID could otherwise revoke grants signed after it.
sideEffects:
  level: destructive
  rationale: >-
    Deletes grants, irreversibly: a replay floor keeps their statements from ever being stored again, and only a new grant signed by the member DID, with a later `iat`, restores the delegation. Affected members are detached synchronously.
consequences:
  - Each affected principal stops receiving the member DID's messages on every connection, before this request is answered; messages stay in the member's inbox.
  - Each affected principal receives a durable grantsRevoked notice with reason admin.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The response names the revoked grants and their two parties to the administrator. The administrator acts on its own standing at the mediator, not on the authority of either party.
retention:
  class: durable
  rationale: >-
    A revoked grant is deleted at once and is not listed afterwards; what survives is the principal's replay floor for the member DID, until no grant at or before it could still be unexpired, and the mediator's audit record of the revocation.
errorCodes:
  - code: messaging/mux/admin-revoke:protectedAccount
    meaning: A grant the request selects has a protected party — its member DID or its principal DID is a rootAdmin account — and the requester is not a rootAdmin. Nothing was revoked.
    retryable: false
  - code: messaging/mux/admin-revoke:contended
    meaning: A member DID's record changed under concurrent writes too many times for the revocation to be committed, and no grant on it was revoked. Send the request again.
    retryable: true
related:
  - messaging/mux/admin-list
  - messaging/mux/admin-revoke-all
  - messaging/mux/revoke
  - messaging/mux/connections
  - messaging/mux/notice
  - messaging/mux/event
---

## Abstract

The **Messaging — Mux Admin Revoke** Trust Task lets a mediator's operator end stored grants ([`messaging/mux/grant`](../../grant/0.1/spec.md)) that belong to other accounts — for a principal DID reported lost or compromised, a member DID whose controller asks the operator for help, or a delegation that breaks the operator's policy. [`messaging/mux/revoke`](../../revoke/0.1/spec.md) does the same for a party to the grant; this task does it with neither party's cooperation, on the administrator's own standing.

The administrator selects grants by **principal DID** (every grant delegated to it), by **member DID** (every grant on it), by both (the grants between the two), or by **member DID and grant identifiers**, after finding them with [`messaging/mux/admin-list`](../../admin-list/0.1/spec.md). Revocation is synchronous: each affected member is detached from every connection carrying it through a revoked grant, with reason `admin`, and frames already queued for it are dropped, **before** the mediator answers. Each affected principal is told by a durable `grantsRevoked` notice.

A principal can hold thousands of grants, so one request revokes at most `limit` of them and reports how many still match; repeating the request finishes the job. To void every grant on a member DID at once, under its epoch, the administrator uses [`messaging/mux/admin-revoke-all`](../../admin-revoke-all/0.1/spec.md). Session attaches are not grants and are not touched here; an attach lapses at its signed expiry, and ends with the epoch advance of `admin-revoke-all`.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is the mediator DID multiplexing design (VTI design docs, `mediator-did-multiplexing-design.md`, §4.8, §7.3 and §7.8).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the administrator) **MUST** emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/messaging/mux/admin-revoke/0.1`, with itself as `issuer` and the mediator as `recipient`, carrying `issuedAt` and a `proof`. It **MUST** name at least one of `principal` and `member`, and **MUST NOT** send `grantIds` without `member`.

A conforming **consumer** (the mediator) **MUST**:

1. Answer `unsupportedType` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) when mux is not enabled, and `malformedRequest` when neither `principal` nor `member` is present, or `grantIds` is present without `member`.
2. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), verify the `proof`, and reject a request outside its freshness window.
3. Respond with `permissionDenied` unless the requester's account type is `admin` or `rootAdmin`. The mediator's own `mediator` account may read grants with [`messaging/mux/admin-list`](../../admin-list/0.1/spec.md) but not revoke them.
4. Where the requester is not a `rootAdmin`, and any grant the selection matches — not only the first `limit` of them — has a protected party, that is, its member DID or its principal DID is a `rootAdmin` account, refuse and respond with `messaging/mux/admin-revoke:protectedAccount`, before revoking anything. A `rootAdmin` requester may revoke any grant.
5. Select the stored grants, in any state, whose principal is `principal` and whose member DID is `member`, for each of the two that is present, and — where `grantIds` is present — whose identifier is listed. Take them in order of member DID, then principal DID, then grant identifier, and act on at most `limit` of them (default 500).
6. Delete each grant acted on, raise its principal's replay floor for that member DID to the moment of revocation ([`messaging/mux/grant`](../../grant/0.1/spec.md#replay-floors)), in one atomic write per member DID's record, and detach its member from every connection that carries it through that grant, synchronously, exactly as [`messaging/mux/revoke`](../../revoke/0.1/spec.md#conformance) consumer rule 4 requires: remove the streaming entry, wait for the streaming path to confirm, drop any frame for the member still queued on the connection. Only then respond. On a clustered mediator, publish the detach to every node, with the same bounded window as `messaging/mux/revoke`. A frame for the member **MUST NOT** leave a local connection through a revoked grant after the response.
7. Emit a `detached` [`messaging/mux/event`](../../event/0.1/spec.md) with reason `admin` to each affected principal that is connected, and store a `grantsRevoked` [`messaging/mux/notice`](../../notice/0.1/spec.md) with reason `admin` in each affected principal's inbox — whether or not it was connected — once its grants are revoked and their members detached.
8. Respond with every grant this request revoked, `remaining` — how many stored grants still match the selection after it — and, in `unmatchedGrantIds`, the listed identifiers that matched no stored grant. Revoking what is already gone is not an error: the requested state already holds, and a request that matches nothing succeeds with an empty `revoked` and `remaining: 0`.
9. Write an audit record of the revocation naming the administrator, the selection and the grants revoked. A revoked grant is not listed afterwards.
10. Where the write to a member DID's record cannot complete under concurrent writes after a bounded number of attempts, revoke nothing on that record and respond `messaging/mux/admin-revoke:contended`; grants already revoked from other records stay revoked, their members detached and their principals notified, and a repeated request finds them gone.

An administrator's client that receives `remaining` above zero **SHOULD** repeat the request until it is `0`, and **MUST NOT** report the selection as revoked before then.

## Authorization

The entitlement is **administrative standing at the mediator**: the requester's account holds the `admin` or `rootAdmin` role (Consumer rule 3). Unlike the read-only [`messaging/mux/connections`](../../connections/0.1/spec.md) and [`messaging/mux/admin-list`](../../admin-list/0.1/spec.md), the mediator's own `mediator` account is not entitled: revocation is destructive, and an operator, not the service, decides it. A `rootAdmin` account is protected from an `admin` (Consumer rule 4), as [`messaging/account/remove`](../../../account/remove/0.1/spec.md) protects it: only a `rootAdmin` may revoke a grant whose member or principal is a `rootAdmin`, so that an `admin` cannot cut a root administrator's delegations. It is the operator's authority over what the mediator carries, not either party's authority over the delegation: the member DID never consented to this revocation and the principal never asked for it, which is why both learn of it — the principal by a `detached` event and a durable notice with reason `admin`, the member DID through [`messaging/mux/list`](../../list/0.1/spec.md), where the grant no longer appears. A party revoking its own grant uses [`messaging/mux/revoke`](../../revoke/0.1/spec.md) instead, and needs no administrator.

The `proof` establishes *who* asked and that the request is unaltered, never that they may ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); the mediator resolves the verified issuer to its account and decides from the role it finds there. Revocation only takes away: it cannot widen a delegation or create one, so an administrator cannot use it to carry a member's traffic anywhere.

## Definitions

- **`principal`** — OPTIONAL. Only grants delegated to this principal DID ([`MuxDid`](../../_shared/0.1/mux.schema.json#/$defs/MuxDid)).
- **`member`** — OPTIONAL. Only grants on this member DID. At least one of `principal` and `member` is REQUIRED; with both, only the grants between the two.
- **`grantIds`** — OPTIONAL, with `member` only. Only these grants of the member DID ([`GrantId`](../../_shared/0.1/mux.schema.json#/$defs/GrantId)). A grant identifier is chosen by its member's signer and is unique only on that member's record, which is why it never stands alone.
- **`limit`** — OPTIONAL. The most grants one request revokes (1–500; default 500).
- **`revoked`** (response) — [`GrantRef`](../../_shared/0.1/mux.schema.json#/$defs/GrantRef)s of every grant this request deleted.
- **`remaining`** (response) — how many grants still match; `0` when the selection is done.
- **`unmatchedGrantIds`** (response) — listed identifiers that matched nothing.

`remaining` is not a cursor: the request is its own continuation, because every grant it revoked is gone and a repeat selects only what is left. A count rather than a flag lets a console show progress, and an exact number is what the mediator can give, since it has just read the selection.

## Request

The administrator sends the request to the mediator. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Cut off a lost device

```json
{
  "id": "urn:uuid:8b2c3d4e-5f60-4b7c-9d8e-0f1a2b3c4d01",
  "type": "https://trusttasks.org/spec/messaging/mux/admin-revoke/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-11-13T09:05:00Z",
  "payload": {
    "principal": "did:webvh:QmPrinc1pal:phone.example"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-11-13T09:05:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

### Revoke one grant of a member

```json
{
  "id": "urn:uuid:8b2c3d4e-5f60-4b7c-9d8e-0f1a2b3c4d03",
  "type": "https://trusttasks.org/spec/messaging/mux/admin-revoke/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-11-13T09:06:00Z",
  "payload": {
    "member": "did:webvh:QmA1ice:alice.example:work",
    "grantIds": ["5d6e7f80-9a1b-4c2d-8e3f-4a5b6c7d8e9f"]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:admin.example#key-1",
    "created": "2026-11-13T09:06:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z4ExampleProofValueNotReal"
  }
}
```

## Response

The mediator responds after every affected member has been detached, in a payload that validates against the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)) rather than a `#response` document.

### Revoked

```json
{
  "id": "urn:uuid:8b2c3d4e-5f60-4b7c-9d8e-0f1a2b3c4d02",
  "type": "https://trusttasks.org/spec/messaging/mux/admin-revoke/0.1#response",
  "threadId": "urn:uuid:8b2c3d4e-5f60-4b7c-9d8e-0f1a2b3c4d01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-11-13T09:05:00Z",
  "payload": {
    "revoked": [
      {
        "grantId": "0b8f6c1e-3a2d-4f5e-9b7c-1d2e3f4a5b6c",
        "did": "did:webvh:QmA1ice:alice.example:work",
        "principal": "did:webvh:QmPrinc1pal:phone.example"
      },
      {
        "grantId": "c4d5e6f7-0819-4a2b-8c3d-4e5f60718293",
        "did": "did:webvh:QmB0b:bob.example",
        "principal": "did:webvh:QmPrinc1pal:phone.example"
      }
    ],
    "remaining": 0
  }
}
```

### Refused: not an administrator

```json
{
  "id": "urn:uuid:8b2c3d4e-5f60-4b7c-9d8e-0f1a2b3c4d04",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "threadId": "urn:uuid:8b2c3d4e-5f60-4b7c-9d8e-0f1a2b3c4d01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-11-13T09:05:00Z",
  "payload": {
    "code": "permissionDenied",
    "message": "messaging/mux/admin-revoke requires an administrator account.",
    "retryable": false
  }
}
```

### Refused: a grant of a root administrator

An `admin` selected a principal that holds a grant from a `rootAdmin`'s DID. Nothing was revoked; a `rootAdmin` must make this request.

```json
{
  "id": "urn:uuid:8b2c3d4e-5f60-4b7c-9d8e-0f1a2b3c4d05",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "threadId": "urn:uuid:8b2c3d4e-5f60-4b7c-9d8e-0f1a2b3c4d01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-11-13T09:05:00Z",
  "payload": {
    "code": "messaging/mux/admin-revoke:protectedAccount",
    "message": "A selected grant's member is a rootAdmin account; only a rootAdmin may revoke it.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

Selectors — a principal DID, a member DID, grant identifiers — in; references to the revoked grants and a count out. No signed statement and no message content. A producer **MUST NOT** place anything but extension data its ecosystem defines in `ext`.

### Correlation

The request names DIDs and grants the mediator already links, and the response shows them to an administrator who could already read them with [`messaging/mux/admin-list`](../../admin-list/0.1/spec.md); it adds the fact and time of the revocation. The `grantsRevoked` notice tells each affected principal that an operator acted, not which administrator. The mediator's identifier scope is public by design: it is the `audience` every grant its members sign names.

### Retention

**Durable**, in effect: the grant is deleted at once and no longer listed, but the replay floor it leaves outlives it, so the revoked statement can never be stored again, and the revocation is audited. The document proof is the record of which administrator revoked.

### Consent/purpose

The purpose is operating the mediator safely: cutting off a principal reported lost or compromised, acting for a member DID's controller who cannot reach its own keys, or enforcing the operator's policy on what may be multiplexed. Neither party consents, by design, so the mediator tells the principal every time (Consumer rule 7). Revocation is not a way to read a DID's traffic or to move it to another principal: it only removes.
