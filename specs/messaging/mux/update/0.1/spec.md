---
slug: messaging/mux/update
version: "0.1"
title: "Messaging — Mux Update"
summary: "Narrow a stored grant — a smaller scope or an earlier expiry — without a new member signature, from either side; anything that widens is refused and must be a fresh grant."
status: draft
targetFrameworkVersion: "0.6.0"
category: messaging
keywords: [messaging, mediator, mux, multiplexing, grant, narrow, scope]
parties:
  - role: Requester (the grant's principal, or its member DID or a principal holding a session attach of it)
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
    A narrowing changes a stored delegation without a new member signature, so the document proof is the only transport-independent record of who changed it. The mediator keeps the narrowed values beside the original signed statement, and an audit of the grant must be able to attribute both.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task mutates a stored grant, which makes it consequential (SPEC §7.3 item 17). A narrowing can only reduce what a grant allows, but a delayed copy arriving after a fresh grant replaced the old one names a grant that no longer exists, and freshness keeps that refusal unambiguous.
sideEffects:
  level: mutating
  rationale: >-
    Narrows a stored grant's effective scope or expiry. It cannot be undone by this task — widening again needs a fresh grant signed by the member — but nothing is deleted and the original signed statement is kept.
consequences:
  - Removing `receive` stops the member's live delivery on the principal's connection before the response; removing `delete` turns delete-on-ack off for the member.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The response returns the grant as it now stands to one of its two parties.
retention:
  class: durable
  rationale: >-
    The narrowed values are stored next to the grant's original signed statement for the grant's life, so both the delegation and its narrowing remain on record.
errorCodes:
  - code: messaging/mux/update:unknownGrant
    meaning: No grant with this identifier is held for which the requester is the principal, or the member (directly or through a live session attach). Conflates a grant that never existed, one that was removed, and one belonging to others.
    retryable: false
  - code: messaging/mux/update:widenRequiresGrant
    meaning: The requested scope is not a subset of the grant's effective scope, or the requested expiry is later than its effective expiry. Widening needs a fresh grant signed by the member DID (messaging/mux/grant), which replaces this one.
    retryable: false
  - code: messaging/mux/update:notActive
    meaning: The grant is `suspended`, `staleKey` or `epochRevoked` and cannot be narrowed. Recreate it with messaging/mux/grant.
    retryable: false
  - code: messaging/mux/update:grantExpired
    meaning: The grant is past its effective expiry and cannot be narrowed. Recreate it with messaging/mux/grant.
    retryable: false
  - code: messaging/mux/update:expiryNotInFuture
    meaning: The requested expiry is not in the future. To end a grant now, revoke it with messaging/mux/revoke.
    retryable: false
  - code: messaging/mux/update:contended
    meaning: The member DID's record changed under concurrent writes too many times for the narrowing to be committed. Nothing changed; send the request again.
    retryable: true
related:
  - messaging/mux/grant
  - messaging/mux/list
  - messaging/mux/revoke
---

## Abstract

The **Messaging — Mux Update** Trust Task **narrows** a stored grant made with [`messaging/mux/grant`](../../grant/0.1/spec.md): a smaller scope, an earlier expiry, or both. Either party may narrow — the principal declining part of what it was given, or the member DID taking part of it back — and no new member signature is needed, because narrowing only ever takes authority away. The mediator stores the narrowed values next to the original signed statement, which it keeps unchanged.

Anything that **widens** — a capability the grant does not have, a later expiry — is refused with `messaging/mux/update:widenRequiresGrant`. Widening always needs a fresh grant signed by the member, which replaces the old one.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is the mediator DID multiplexing design (VTI design docs, `mediator-did-multiplexing-design.md`, §4.7).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** **MUST** name at least one of `scope` and `expiresAt`.

A conforming **consumer** (the mediator) **MUST**:

1. Answer `unsupportedType` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) when mux is not enabled, and `malformedRequest` when neither `scope` nor `expiresAt` is present.
2. Respond `messaging/mux/update:unknownGrant` unless the grant exists and the requester is its principal, its member DID, or holds a live session attach of its member DID on the connection that carried the request.
3. Respond `messaging/mux/update:notActive` when the grant's stored state is not `active`, and `messaging/mux/update:grantExpired` when it is active but past its effective expiry.
4. Respond `messaging/mux/update:expiryNotInFuture` when `expiresAt` is not in the future.
5. Respond `messaging/mux/update:widenRequiresGrant` when `scope` is not a subset of the grant's **effective** scope, or `expiresAt` is later than its **effective** expiry. A value equal to the one in force is not widening.
6. Otherwise store the narrowed values beside the original statement, leaving the statement unchanged, in one atomic write to the member DID's record; where that write cannot complete under concurrent writes after a bounded number of attempts, change nothing and respond `messaging/mux/update:contended`. Where the principal is connected and the narrowing removes a capability, apply it before responding: removing `receive` stops live delivery for the member and drops any frame for it already queued on the connection (the message stays in its inbox); removing `delete` stops delete-on-ack for the member. Emit an `attached` [`messaging/mux/event`](../../event/0.1/spec.md) carrying the member's new scope.
7. Respond with the grant as it now stands and `changed`, which is false when the request named only values already in force. A narrowing that changes nothing succeeds: the requested state already holds.

A narrowed expiry is enforced exactly as a signed one: when it passes, the grant is reported `expired` and its member is detached with reason `expired`.

## Authorization

**Authority: being a party to the grant.** Either the principal or the member DID may narrow, because narrowing only removes authority, and either party may always give up or take back part of a delegation without the other's cooperation. That is also why no member signature is needed: the only thing a signature would protect against is a widening, and widening is refused. The member DID is proven by authenticating as it, or by a live session attach of it on the request's connection. The document `proof` identifies the requester and records who narrowed ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)); the entitlement is the party relationship, read from the grant record.

## Definitions

- **`grantId`** — REQUIRED. The grant to narrow.
- **`scope`** — OPTIONAL. The new effective [`MuxScope`](../../_shared/0.1/mux.schema.json#/$defs/MuxScope); a subset of the current one.
- **`expiresAt`** — OPTIONAL. The new effective expiry, RFC 3339; in the future, and no later than the current one.
- **Effective scope / expiry** — the signed value, or the narrowed one where a previous update narrowed it.
- **`grant`** (response) — the [`GrantView`](../../_shared/0.1/mux.schema.json#/$defs/GrantView) as it now stands: the signed statement verbatim, with the effective scope and expiry beside it and `narrowed: true`.
- **`changed`** (response) — whether anything changed.

## Request

The requester sends the request to the mediator. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### The member takes `pickup` back and shortens the grant

```json
{
  "id": "urn:uuid:9c0d1e2f-3a4b-4c5d-8e6f-7a8b9c0d1e01",
  "type": "https://trusttasks.org/spec/messaging/mux/update/0.1",
  "issuer": "did:webvh:QmA1ice:alice.example:work",
  "recipient": "did:web:mediator.example",
  "issuedAt": "2026-11-09T12:00:00Z",
  "payload": {
    "grantId": "0b8f6c1e-3a2d-4f5e-9b7c-1d2e3f4a5b6c",
    "scope": ["receive"],
    "expiresAt": "2026-11-16T09:00:00Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmA1ice:alice.example:work#key-1",
    "created": "2026-11-09T12:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The mediator responds with the grant as it now stands, in a payload that validates against the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` rather than a `#response` document.

### Narrowed

```json
{
  "id": "urn:uuid:9c0d1e2f-3a4b-4c5d-8e6f-7a8b9c0d1e02",
  "type": "https://trusttasks.org/spec/messaging/mux/update/0.1#response",
  "threadId": "urn:uuid:9c0d1e2f-3a4b-4c5d-8e6f-7a8b9c0d1e01",
  "issuer": "did:web:mediator.example",
  "recipient": "did:webvh:QmA1ice:alice.example:work",
  "issuedAt": "2026-11-09T12:00:00Z",
  "payload": {
    "grant": {
      "statement": {
        "type": "muxGrant",
        "issuer": "did:webvh:QmA1ice:alice.example:work",
        "audience": "did:web:mediator.example",
        "grantId": "0b8f6c1e-3a2d-4f5e-9b7c-1d2e3f4a5b6c",
        "principal": "did:webvh:QmPrinc1pal:phone.example",
        "scope": [
          "receive",
          "pickup"
        ],
        "epoch": 3,
        "iat": 1794214800,
        "exp": 1796806800,
        "proof": {
          "type": "DataIntegrityProof",
          "cryptosuite": "eddsa-jcs-2022",
          "verificationMethod": "did:webvh:QmA1ice:alice.example:work#key-1",
          "proofPurpose": "authentication",
          "proofValue": "z58DAdFfa9SkqZMVPxAQpic7ndSayn1PzZs6ZjWp1CktyGesjuTSwRdoWhAfGFCF5bppETSTojQCrfFPP2oumHKtz"
        }
      },
      "state": "active",
      "scope": [
        "receive"
      ],
      "expiresAt": "2026-11-16T09:00:00Z",
      "narrowed": true
    },
    "changed": true
  }
}
```

### Refused: the principal tries to add `delete`

```json
{
  "id": "urn:uuid:9c0d1e2f-3a4b-4c5d-8e6f-7a8b9c0d1e03",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "threadId": "urn:uuid:9c0d1e2f-3a4b-4c5d-8e6f-7a8b9c0d1e10",
  "issuer": "did:web:mediator.example",
  "recipient": "did:webvh:QmPrinc1pal:phone.example",
  "issuedAt": "2026-11-09T12:05:00Z",
  "payload": {
    "code": "messaging/mux/update:widenRequiresGrant",
    "message": "Adding delete widens the grant; it needs a new grant signed by the member.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

A grant identifier and the narrowed values in; the grant's view out, to one of its parties. No signed statement goes in, and no message content. A producer **MUST NOT** place anything but extension data its ecosystem defines in `ext`.

### Correlation

The request names a grant the mediator already holds and the two parties already know, so it links nothing new. The mediator learns which side chose to narrow, and when. The mediator's identifier scope is public by design: it is the address every principal, member and peer already uses to reach it, and the `aud` every mux proof names.

### Retention

**Durable**, with the grant: the narrowed values are kept beside the original statement for as long as the grant is kept. The document proof is the record of who narrowed.

### Consent/purpose

The purpose is to reduce a delegation, at either party's choice, without involving the other: a member taking back part of what it gave, a principal declining what it no longer needs. It can never be used to extend a delegation.
