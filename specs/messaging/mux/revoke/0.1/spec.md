---
slug: messaging/mux/revoke
version: "0.1"
title: "Messaging — Mux Revoke"
summary: "Revoke stored grants by identifier or by counterparty DID, as their principal or as their member DID; affected members are detached from every connection before the mediator answers."
status: draft
targetFrameworkVersion: "0.6.0"
category: messaging
keywords: [messaging, mediator, mux, multiplexing, grant, revoke, revocation]
parties:
  - role: Requester (the grants' principal, or their member DID or a principal holding a session attach of it)
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
    Revocation is irreversible — a revoked grant can only be replaced by a fresh member signature — and is made without one, so the document proof is the transport-independent record of who ended the delegation. The response is the requester's evidence that it took effect.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task ends stored delegations, which makes it consequential (SPEC §7.3 item 17). A delayed copy naming counterparty DIDs could otherwise revoke a grant signed after it.
sideEffects:
  level: destructive
  rationale: >-
    Deletes grants, irreversibly: a replay floor keeps their statements from ever being stored again, and only a new grant signed by the member DID, with a later `iat`, restores the delegation. Affected members are detached synchronously.
consequences:
  - The principal stops receiving the member DID's messages on every connection, before this request is answered; messages stay in the member's inbox.
  - When a member revokes, each affected principal receives a durable grantsRevoked notice.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The response names the revoked grants and their two parties to the requester, who is one of them.
retention:
  class: durable
  rationale: >-
    A revoked grant is deleted at once and is not listed afterwards; what survives is the principal's replay floor for the member DID, until no grant at or before it could still be unexpired, and the mediator's audit record of the revocation.
errorCodes:
  - code: messaging/mux:memberNotProven
    meaning: The request revokes as `member` for a DID the requester neither is nor holds a live session attach of on the connection that carried the request.
    retryable: false
  - code: messaging/mux/revoke:contended
    meaning: A member DID's record changed under concurrent writes too many times for the revocation to be committed, and nothing was revoked. Send the request again.
    retryable: true
related:
  - messaging/mux/revoke-all
  - messaging/mux/grant
  - messaging/mux/list
  - messaging/mux/detach
  - messaging/mux/notice
---

## Abstract

The **Messaging — Mux Revoke** Trust Task ends stored grants made with [`messaging/mux/grant`](../../grant/0.1/spec.md). Either party may revoke without the other's cooperation:

- **As `principal`**, the requester drops grants delegated to it — named by grant identifier, or by member DID.
- **As `member`**, the requester removes grants on a member DID it controls — named by grant identifier, or by principal DID. This is how a member DID cuts off one principal without touching the rest.

Revocation is synchronous: each affected member is detached from every connection carrying it, and frames already queued for it are dropped, **before** the mediator answers. To void every grant on a member DID at once, including ones whose principals it does not know, the member uses [`messaging/mux/revoke-all`](../../revoke-all/0.1/spec.md). Session attaches are removed with [`messaging/mux/detach`](../../detach/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is the mediator DID multiplexing design (VTI design docs, `mediator-did-multiplexing-design.md`, §4.8, §7.3 and §7.4).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** **MUST** name at least one of `grantIds` and `dids`, and **MUST NOT** send `did` with `as: principal`.

A conforming **consumer** (the mediator) **MUST**:

1. Answer `unsupportedType` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) when mux is not enabled, and `malformedRequest` when neither `grantIds` nor `dids` is present, or `did` accompanies `as: principal`.
2. As `member`: the member DID is `did`, or the requester when `did` is absent. Respond `messaging/mux:memberNotProven` unless the member DID is the requester or the requester holds a live session attach of it on the connection that carried the request.
3. Select the stored grants, in any state, that both match a selector — a listed `grantIds` entry, or a listed DID as counterparty (as `principal`, the member DID; as `member`, the principal DID) — and belong to the requester in the role it named: as `principal`, grants whose principal is the requester; as `member`, grants on the member DID.
4. Delete each selected grant, raise the principal's replay floor for that member DID to the moment of revocation ([`messaging/mux/grant`](../../grant/0.1/spec.md#replay-floors)), in one atomic write to the member DID's record, and detach its member from every connection that carries it through that grant, synchronously: remove the streaming entry, wait for the streaming path to confirm, drop any frame for the member still queued on the connection. Only then respond. On a clustered mediator, publish the detach to every node; the response waits for the local detach, and every pickup or delete for the member is checked against the grant record, so the window on another node is bounded to live pushes already in flight there. A frame for the member **MUST NOT** leave a local connection through a revoked grant after the response.
5. Emit a `detached` [`messaging/mux/event`](../../event/0.1/spec.md) to each affected principal that is connected, with reason `revokedByPrincipal` or `revokedByMember`, and, when a member revoked, store a `grantsRevoked` [`messaging/mux/notice`](../../notice/0.1/spec.md) in each affected principal's inbox.
6. Respond with the grants revoked, and echo in `unmatchedGrantIds` and `unmatchedDids` the selectors that matched no stored grant the requester may revoke in that role. Revoking what is already gone is not an error: the requested state already holds. A selector matching another party's grant is reported unmatched, exactly as one matching nothing.
7. Write an audit record of the revocation. A revoked grant is not listed afterwards.
8. Where the write to a member DID's record cannot complete under concurrent writes after a bounded number of attempts, and no grant was revoked, respond `messaging/mux/revoke:contended`; grants already revoked from other records stay revoked and are reported.

## Authorization

**Authority: being a party to the grant.** A principal may always decline a delegation it holds; a member DID may always withdraw a delegation it made. Neither needs the other's cooperation, which is what lets a member cut off a compromised or lost principal. The member DID is proven by authenticating as it, or by a live session attach of it on the request's connection — itself proven by the member's signature. The document `proof` and the transport identify the requester ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); the entitlement is the party relationship, read from the grant records. Consumer rule 3 enforces it: a selector never reaches a grant the requester is not party to in the role it named.

## Definitions

- **`as`** — REQUIRED. [`MuxRole`](../../_shared/0.1/mux.schema.json#/$defs/MuxRole).
- **`did`** — OPTIONAL, `member` only. The member DID whose grants are revoked (default: the requester).
- **`grantIds`** — OPTIONAL. Grant identifiers.
- **`dids`** — OPTIONAL. Counterparty DIDs: member DIDs as `principal`, principal DIDs as `member`.
- **`revoked`** (response) — [`GrantRef`](../../_shared/0.1/mux.schema.json#/$defs/GrantRef)s of the grants this request deleted.
- **`unmatchedGrantIds`**, **`unmatchedDids`** (response) — selectors that matched nothing revocable.

## Request

The requester sends the request to the mediator. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member DID cuts off one principal

```json
{
  "id": "urn:uuid:0d1e2f3a-4b5c-4d6e-8f7a-8b9c0d1e2f01",
  "type": "https://trusttasks.org/spec/messaging/mux/revoke/0.1",
  "issuer": "did:webvh:QmA1ice:alice.example:work",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-11-09T13:00:00Z",
  "payload": {
    "as": "member",
    "dids": ["did:webvh:QmPrinc1pal:phone.example"]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmA1ice:alice.example:work#key-1",
    "created": "2026-11-09T13:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The mediator responds after every affected member has been detached, in a payload that validates against the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` rather than a `#response` document.

### Revoked

```json
{
  "id": "urn:uuid:0d1e2f3a-4b5c-4d6e-8f7a-8b9c0d1e2f02",
  "type": "https://trusttasks.org/spec/messaging/mux/revoke/0.1#response",
  "threadId": "urn:uuid:0d1e2f3a-4b5c-4d6e-8f7a-8b9c0d1e2f01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:webvh:QmA1ice:alice.example:work",
  "issuedAt": "2026-11-09T13:00:00Z",
  "payload": {
    "revoked": [
      {
        "grantId": "0b8f6c1e-3a2d-4f5e-9b7c-1d2e3f4a5b6c",
        "did": "did:webvh:QmA1ice:alice.example:work",
        "principal": "did:webvh:QmPrinc1pal:phone.example"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Selectors — grant identifiers and counterparty DIDs — in; references to the revoked grants out. No signed statement and no message content. A producer **MUST NOT** place anything but extension data its ecosystem defines in `ext`.

### Correlation

The request names DIDs and grants the mediator already links; it adds the fact and time of the revocation. Unmatched selectors are echoed rather than explained, so a requester cannot use this task to learn whether a DID it is not party with holds grants. The mediator's identifier scope is public by design: it is the address every principal, member and peer already uses to reach it, and the `aud` every mux proof names.

### Retention

**Durable**, in effect: the grant itself is deleted at once and no longer listed, but the replay floor it leaves behind outlives it, so the revoked statement can never be stored again, and the revocation is audited. The document proof is the record of who revoked.

### Consent/purpose

The purpose is to end a delegation, at either party's choice: a member withdrawing a principal's access to its messages, a principal giving up a member it no longer carries. Revocation needs nothing from the other side, so neither party can be held to a delegation it no longer wants.
