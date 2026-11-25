---
slug: hcp/grants/revoked
version: "0.1"
title: "HCP — Revocation Notice"
summary: A Human Context Protocol provider tells a Context Client that a grant it held has been revoked, and the client acknowledges; carries the HCP 2.0-draft-3 revocation notification.
status: draft
targetFrameworkVersion: "0.6.0"
carries:
  protocol: HCP
  version: "2.0-draft-3"
  reference: https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md
  operation: Revocation notification
category: access-control
keywords: [hcp, human-context-protocol, grant, revocation, delegation, notification, security-event-token]
parties:
  - role: HCP Provider
    requirement: REQUIRED
    member: issuer
  - role: Context Client (HCP) that held the grant
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    HCP requires the revocation notice to be signed and the client to verify it (HCP 9.9): an unsigned notice would
    let anyone make a client stop using its context and a live grant. The proof is REQUIRED on every binding.
    The acknowledgement's proof is RECOMMENDED, so the provider can show the client received the notice.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The notice obliges the client to stop using disclosed context, so it is consequential; it carries the Security
    Event Token's `iat`. Every delivery attempt is a fresh document whose `issuedAt` is the time of that attempt,
    so HCP's 300-second freshness check applies to each attempt.
sideEffects:
  level: mutating
  rationale: >-
    The client must stop using every copy of context disclosed under the grant and must pass none of them on (HCP
    6.2.2), except a copy kept under a legal obligation it declared (HCP 6.2.5).
exposure:
  discloses: none
  actsAsSubject: false
  ingests: metadata
  rationale: >-
    The notice carries only a notice identifier, a grant identifier, an instant and whether the grant was revoked
    or ended with its delegation; the acknowledgement carries nothing.
retention:
  class: durable
  rationale: >-
    The provider records the first acknowledgement, or the abandonment of delivery, as a `revocation.notified` audit
    event (HCP 6.2.3), and the client keeps the notice as its record of why it stopped using the context.
errorCodes: []
related:
  - hcp/grants/revoke
  - hcp/entries/query
---

## Abstract

The **HCP — Revocation Notice** Trust Task carries the revocation notification of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 6.2.3 and 9.9). When a grant is revoked, the *HCP Provider* tells the *Context Client* that held it, where that client holds disclosed entries and has registered a notification endpoint (HCP 6.2.2), and the client acknowledges. HCP sends this notice as a signed Security Event Token (RFC 8417) delivered by push (RFC 8935); this task carries the same event as a signed Trust Task document, so the provider can reach the client over any Trust Task binding. The same notice also announces a grant that ended because the delegation it was authorized under ended (HCP 17.6), *pending HCP confirmation* ([question 17](#pending-hcp-confirmation)).

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)). The revocation itself is [`hcp/grants/revoke`](../../revoke/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider and client that use this task **MUST** also meet every requirement HCP 2.0-draft-3 places on revocation notices; the ones that bear on the document are restated below, unchanged. Conformance to HCP is HCP's ([SPEC §9.4](/SPEC.md#94-carried-protocols)): a provider **MUST NOT** list a conformance identifier for this carriage in its HCP `conformsTo`, and advertises the task through [`trust-task-discovery`](../../../../trust-task-discovery/0.4/spec.md), *pending HCP confirmation* ([question 1](#pending-hcp-confirmation)).

This specification makes one declared departure from HCP: every delivery attempt is a fresh Trust Task document, with its own `id`, `issuedAt` and `proof`, where HCP 9.9 sends the same token on every attempt. The token's `jti`, which HCP keeps the same on every attempt so that the client can deduplicate, is carried as the payload's `noticeId` (see [Delivery rules carried from HCP](#delivery-rules-carried-from-hcp)). It is *pending HCP confirmation* ([question 16](#pending-hcp-confirmation)).

## Authorization

The authority is **the provider's position as the holder of the grant**: only the provider that issued a grant can say it has ended. The client accepts the notice only when its `proof` verifies under a key of that provider. Here the proof is the authorization in the sense of [CONTRIBUTING-SPECS](../../../../../CONTRIBUTING-SPECS.md#what-to-write): a verified assertion by the one party entitled to make it, for the one purpose of ending use of that grant.

HCP requires the notice to be signed with an asymmetric algorithm, with a key the provider publishes at its `jwks_uri`, and the client to verify it (HCP 9.9). A Trust Task proof is made with a key of the `issuer` VID. To meet both, the provider **MUST** sign the notice with a key that its VID's DID document lists **and** that its HCP JWKS also publishes, so that a client verifying under either HCP's rule or this framework's finds the same key. A client **MUST** check that the `issuer` VID is bound to the provider that issued the grant (see [Parties](#parties)) and **MUST** refuse a notice whose key it cannot find in that provider's JWKS.

The client does not need to hold any authority of its own to acknowledge.

## Definitions

- **HCP Provider**, **Context Client** and **grant** have their HCP 2.0-draft-3 meanings.
- The **grant-revoked event** is HCP's event `https://hcp.example/spec/events/grant-revoked`, with members `grantId` and `revokedAt`.
- A **notice** is the announcement of one grant's ending. Its `noticeId` identifies it.
- A **delivery attempt** is one sending of the notice. Each attempt is a new Trust Task document, with its own `id`, `issuedAt` and `proof`. Every attempt for one notice carries the same `threadId` and the same payload, and so the same `noticeId`.

### HCP field mapping

The Security Event Token's claims map to the Trust Task envelope, and the event's members to the payload. The SET's `events` wrapper is replaced by the Type URI, which names the one event this task carries.

| HCP / SET field | Member | Notes |
|---|---|---|
| `iss` | `issuer` | The provider's VID. |
| `aud` | `recipient` | The client's VID. |
| `jti` | `noticeId` | **Identical on every delivery attempt**, as HCP 9.9 requires of `jti`; the client deduplicates on it. *Pending HCP confirmation* ([question 16](#pending-hcp-confirmation)). |
| none | `id` | **Fresh on every delivery attempt**, as for any new document ([SPEC §8.4](/SPEC.md#84-retry-semantics)). It identifies the attempt, not the notice. |
| `iat` | `issuedAt` | Whole seconds, as SET instants already are. **Fresh on every delivery attempt**: the time that attempt is sent. *Pending HCP confirmation* ([question 16](#pending-hcp-confirmation)). |
| The same token on every attempt (HCP 9.9) | a fresh document on every attempt, with a stable `noticeId` | The `id`, `issuedAt` and `proof` are new on every attempt; the `threadId` and payload are the same. A declared departure from HCP 9.9, *pending HCP confirmation* ([question 16](#pending-hcp-confirmation)). |
| `events` key `https://hcp.example/spec/events/grant-revoked` | the Type URI | |
| event `grantId` | `grantId` | Unchanged. |
| event `revokedAt` | `revokedAt` | Unchanged for a revocation. For a grant ended with its delegation, when the delegation ended (HCP 17.6). |
| `grant.expired` audit `reason` `delegation-ended` (HCP 8.2, 17.6) | `reason` | OPTIONAL. `revoked` or `delegation-ended`; absent means `revoked`. HCP's event has no such member; it is added, *pending HCP confirmation* ([question 17](#pending-hcp-confirmation)). |
| JWS signature | `proof` | See [Authorization](#authorization) for the key. Made afresh on every delivery attempt. |
| RFC 8935 `202 Accepted` | the empty `#response` payload | |
| RFC 8935 `400` error codes | `trust-task-error` codes | See [Errors](#errors). |

### Parties

| HCP role | Trust Task member |
|---|---|
| HCP Provider | `issuer` of the notice, `recipient` of the acknowledgement |
| Context Client | `recipient` of the notice, `issuer` of the acknowledgement |

HCP sends the notice to the notification endpoint the client registered. Here the client's VID stands in for that endpoint: at registration, the client **MUST** give the provider the VID it will receive notices at, and the provider **MUST** send the notice to that VID and no other. The provider's VID is bound to its HCP identity as [`hcp/entries/query`](../../../entries/query/0.1/spec.md#parties) describes.

## Request

The provider sends the notice to the client. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json): `noticeId`, which identifies the notice, `grantId`, the grant that ended, and `revokedAt`, when. All three are REQUIRED. `noticeId` carries the Security Event Token's `jti` (HCP 9.9), in the `urn:uuid` form HCP's own example gives it: an identifier the provider **MUST** mint fresh for each notice and **MUST** keep the same on every delivery attempt of that notice, *pending HCP confirmation* ([question 16](#pending-hcp-confirmation)). `reason` is OPTIONAL: `revoked` for a revocation (HCP 6.2), which is also what its absence means, or `delegation-ended` for a grant that ended because the delegation it was authorized under ended (HCP 17.6), *pending HCP confirmation* ([question 17](#pending-hcp-confirmation)). A provider **MUST** send `reason: "delegation-ended"` for such a grant, with `revokedAt` the instant the delegation ended, and a client **MUST** act on the notice the same way whichever `reason` it carries.

### A grant was revoked

```json
{
  "id": "urn:uuid:2c8e4a61-5d3b-4f9e-b7a2-0e1d9c8b7a63",
  "type": "https://trusttasks.org/spec/hcp/grants/revoked/0.1#request",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:booking-agent.example",
  "issuedAt": "2026-11-24T11:00:02Z",
  "threadId": "urn:uuid:e5a3c1b9-8d7f-4e2a-9b6c-3d2e1f0a9b74",
  "payload": {
    "noticeId": "urn:uuid:f1e2d3c4-b5a6-4978-8a6b-5c4d3e2f1a09",
    "grantId": "urn:uuid:c4d2e1f0-9a8b-4c7d-8e6f-5a4b3c2d1e0f",
    "revokedAt": "2026-11-24T11:00:01Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-24T11:00:02Z",
    "verificationMethod": "did:web:context.provider.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z2pQf8bW4nLxY6vT1kR9sJ3dM7hA5cE2gZ8uN4oP6iB3yX1wV9tK7mL5qD2fS8jH4rC6eG1aU3zO9nT5bW7kM2xQ"
  }
}
```

### A later delivery attempt of the same notice

The provider had no acknowledgement for the first attempt, so it sends the notice again. This attempt is a new document: a new `id`, the time of this attempt as `issuedAt`, and a new `proof`. The `threadId` and the payload, `noticeId` included, are those of the first attempt.

```json
{
  "id": "urn:uuid:3a9c7e5b-2d1f-4b8a-9e6c-4f2a1b0c9d86",
  "type": "https://trusttasks.org/spec/hcp/grants/revoked/0.1#request",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:booking-agent.example",
  "issuedAt": "2026-11-24T11:08:02Z",
  "threadId": "urn:uuid:e5a3c1b9-8d7f-4e2a-9b6c-3d2e1f0a9b74",
  "payload": {
    "noticeId": "urn:uuid:f1e2d3c4-b5a6-4978-8a6b-5c4d3e2f1a09",
    "grantId": "urn:uuid:c4d2e1f0-9a8b-4c7d-8e6f-5a4b3c2d1e0f",
    "revokedAt": "2026-11-24T11:00:01Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-11-24T11:08:02Z",
    "verificationMethod": "did:web:context.provider.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z5tW3nQ8kR1vY6pB4cX9mL2sD7hF5jG3aZ8uE1oP6iT4yK2wV9nM7bQ5xS3fC8rH1eA6gJ4dU2zO7lT9kN5wB3"
  }
}
```

### A grant ended with its delegation

The delegation under which a delegate authorized the grant has ended, so the grant ended at the same instant (HCP 17.6).

```json
{
  "id": "urn:uuid:7d1f9b3e-4a6c-4e2d-8f5b-1c0a9e8d7f64",
  "type": "https://trusttasks.org/spec/hcp/grants/revoked/0.1#request",
  "issuer": "did:web:context.provider.example",
  "recipient": "did:web:booking-agent.example",
  "issuedAt": "2027-03-01T00:00:05Z",
  "threadId": "urn:uuid:0b9a8c7d-6e5f-4a3b-9c2d-1e0f9a8b7c65",
  "payload": {
    "noticeId": "urn:uuid:6c5b4a39-2817-4f06-9e5d-4c3b2a190f8e",
    "grantId": "urn:uuid:9e8d7c6b-5a4f-4e3d-8c2b-1a0f9e8d7c6b",
    "revokedAt": "2027-03-01T00:00:00Z",
    "reason": "delegation-ended"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-03-01T00:00:05Z",
    "verificationMethod": "did:web:context.provider.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z4kR8sN2pW6tY1vB9cX3mL7qD5hF2jG8aZ4uE6oP1iT3yK9wV7nM5bQ2xS8fC4rH6eA1gJ3dU9zO5lT7kN2wB4"
  }
}
```

## Response

The client acknowledges the notice. The payload is the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json): the empty object, which carries RFC 8935's `202 Accepted`. It is declared because the provider relies on it: the first acknowledgement is what HCP's `revocation.notified` event records ([SPEC §4.4.2](/SPEC.md#442-acknowledging-a-fire-and-forget-task)). A refusal is a `trust-task-error` (see [Errors](#errors)).

### The client accepted the notice

```json
{
  "id": "urn:uuid:8b6d4f2a-1e9c-4a7b-8d5e-6f4a3b2c1d85",
  "type": "https://trusttasks.org/spec/hcp/grants/revoked/0.1#response",
  "issuer": "did:web:booking-agent.example",
  "recipient": "did:web:context.provider.example",
  "issuedAt": "2026-11-24T11:00:03Z",
  "threadId": "urn:uuid:e5a3c1b9-8d7f-4e2a-9b6c-3d2e1f0a9b74",
  "payload": {}
}
```

A client answers a later delivery attempt of a notice it already processed, recognized by its `noticeId`, with this same empty acknowledgement, and does not act on the notice again (see [Delivery rules carried from HCP](#delivery-rules-carried-from-hcp)). A bit-for-bit identical copy of one attempt, as a transport may redeliver, is absorbed by its `id` ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) and acknowledged the same way.

## Errors

RFC 8935's `400` error codes map to Trust Task errors as follows.

| RFC 8935 code | Trust Task error |
|---|---|
| `invalid_request` | `malformedRequest` |
| `invalid_key`, `authentication_failed` | `proofInvalid` |
| `invalid_audience` | `wrongRecipient` |
| `invalid_issuer`, `access_denied` | `permissionDenied` |

Every one of these is `retryable: false`. A non-retryable error ends delivery, and the provider records it as abandoned. A client that is temporarily unable to process the notice answers `unavailable`, and the provider retries.

A later delivery attempt of a notice the client already processed is not an error: the client answers it with the empty acknowledgement (see [Response](#response)). Because every attempt carries a fresh `id`, the framework's `idConflict` ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) has its ordinary meaning here and no other.

## Delivery rules carried from HCP

- **When a notice is sent.** Only if the client holds entries disclosed under the grant and has registered a notification endpoint, which here is its VID (see [Parties](#parties)) (HCP 6.2.2, 6.2.3). A client with no endpoint is not notified; its copies were capped at one hour of use (HCP 7.6).
- **Retries.** Until it receives an acknowledgement or a non-retryable error, the provider **MUST** make a further delivery attempt on a failed delivery and on any other answer, with exponential backoff, for at least 5 attempts over at least one hour (HCP 9.9). This is a carriage rule, a declared departure from HCP 9.9's "same token" wording, *pending HCP confirmation* ([question 16](#pending-hcp-confirmation)): every attempt **MUST** be a new Trust Task document, with a fresh `id`, a fresh `issuedAt`, the time of that attempt, and a fresh `proof` over it, and **MUST** carry the notice's `threadId` and payload unchanged, so the same `noticeId`. The framework's rules on repeated documents ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements), [SPEC §8.4](/SPEC.md#84-retry-semantics)) apply to each attempt as to any other document. A provider **MUST NOT** reuse an attempt's `id` for a document that differs from it. A provider **MUST NOT** make a delivery attempt more than 24 hours after the notice's first attempt: delivery not acknowledged by then is abandoned (HCP 6.2.4). HCP sets only the minimum; the 24-hour limit is a carriage bound, not an HCP requirement, and it is what lets a client size its deduplication record.
- **Deduplication.** The client deduplicates on `noticeId`, as HCP 9.9 has it deduplicate on `jti`. A client that receives an attempt whose `noticeId` it has already processed from that provider **MUST** answer it with the empty acknowledgement (see [Response](#response)) and **MUST NOT** act on the notice a second time. A client **MUST** keep its record of a processed `noticeId` for at least 25 hours after it processed the notice, so that every later attempt is recognized: no attempt comes later than 24 hours after the first, and the extra hour covers the 300-second clock tolerance and attempts still in transit.
- **Audit.** The provider treats any acknowledgement, the first attempt's or a later one's, as successful delivery. It records `revocation.notified` on the first acknowledgement, or when it abandons delivery (HCP 6.2.3), and meets HCP's duties on abandonment (HCP 6.2.4).
- **Freshness.** HCP's rule that a client **SHOULD** reject a SET whose `iat` differs from local time by more than 300 seconds (HCP 9.9) applies to each attempt's `issuedAt`, and so does the client's acceptance window for any Trust Task document. Because every attempt carries a fresh `issuedAt`, a client that missed the early attempts can still accept a later one; under HCP's own text every attempt carries the first one's `iat`, so any attempt after 300 seconds would be stale.
- **No notice after relinquishment.** The provider sends no notice to a client that relinquished the grant itself (HCP 5.2.7).
- **Independence.** The revocation took effect at `revokedAt` whether or not this notice is ever delivered (HCP 6.2.3). A client that learns of the revocation another way, such as `hcp:invalidToken` on a query (HCP 6.2.2), acts on it at once.
- **On receipt.** The client **MUST** stop using every copy of context disclosed under the grant, whatever its `useUntil`, and **MUST NOT** pass any of them on (HCP 6.2.2, 7.6). The one exception is a copy kept in a record under a legal obligation the client declared at registration, which it keeps only to meet that obligation (HCP 6.2.5). It also stops using the grant's token. HCP does not require deletion, and the provider cannot enforce it (HCP 6.2.2).
- **A delegation that ends.** When a delegation ends, every grant authorized under it ends with status `expired`, not `revoked`, and "invalidation and notification follow 6.2.2–6.2.4, as for a revocation" (HCP 17.6). HCP names only the grant-revoked event. This specification announces such a grant with this notice, carrying `reason: "delegation-ended"` (the reason HCP's `grant.expired` audit event records, HCP 8.2) and `revokedAt` the instant the delegation ended, *pending HCP confirmation* ([question 17](#pending-hcp-confirmation)). The delivery rules above apply to it unchanged. A grant that reaches its own `expiresAt` is not announced.

Over TSP without a return path, the acknowledgement is delivered best-effort ([TSP binding 0.2 §6](../../../../../bindings/tsp/0.2/spec.md)), so the provider may see no acknowledgement for a notice the client accepted. It then retries, and abandons as HCP describes.

## Pending HCP confirmation

HCP 2.0-draft-3 leaves the following points open. This specification takes the reading stated for each, and will follow the HCP editors' answer when they give one. The question numbers are those of the list of questions put to the editors.

- **Question 1 — conformance claims.** HCP defines no conformance identifier for Trust Task carriage, so a provider **MUST NOT** claim this carriage in its HCP `conformsTo`; it advertises the task with `trust-task-discovery` instead. A conformance identifier for the carriage is to be requested from the HCP editors.
- **Question 16 — freshness versus retries.** Every delivery attempt is a fresh Trust Task document, with a new `id`, `issuedAt` and `proof`, and the client applies HCP's 300-second check to that `issuedAt`. The SET's `jti` is carried as the payload's `noticeId`, the same on every attempt; the client deduplicates on it and answers a repeat with the empty acknowledgement. This is a declared departure from HCP 9.9, which sends the same token on every attempt (see [Delivery rules carried from HCP](#delivery-rules-carried-from-hcp)).
- **Question 17 — a grant ended with its delegation.** It is announced with this notice, with `reason: "delegation-ended"` and `revokedAt` the instant the delegation ended (HCP 17.6). The `reason` member is an addition HCP's event does not have (see [Request](#request)).

## What HCP requires that a document cannot express

- The retry schedule and the abandonment duty (HCP 9.9, 6.2.4).
- The client's end of use of disclosed copies, and its passing none on (HCP 6.2.2), which no acknowledgement can prove.
- The key-publication rule: the signing key's presence in the provider's JWKS (HCP 9.9) is checked outside the document.

## Security & Privacy

### Data carried

The notice carries a notice identifier, a grant identifier, the instant the grant ended and, optionally, `reason`; the acknowledgement carries nothing. `noticeId` is a random identifier and **MUST NOT** be derived from anything about the person or the grant. Neither carries context or tokens. `reason` says only whether the grant was revoked or ended with its delegation. It **MUST NOT** carry why: a provider **MUST NOT** send the delegation's own reason for ending (HCP 17.6), such as `majority`, which would disclose the person's age, and **MUST NOT** put that, the revocation's reason, or anything about the person, in `ext`. A reason for a revocation could disclose why the person withdrew.

### Correlation

The `grantId` joins this notice to the revocation request and to the client's earlier queries under the grant; that join is the purpose of the notice. The client learns only that, and when, the grant ended. The timing of the notice can reveal that the person acted at that moment; HCP accepts that cost. `noticeId` and `threadId` join the delivery attempts of one notice to each other and to nothing else: `noticeId` **MUST** be fresh per notice, and `threadId` **SHOULD** be fresh per notice; both are the same on every delivery attempt.

### Retention

The provider keeps the `revocation.notified` audit event (HCP 6.2.3) and **SHOULD** keep the acknowledgement with it as evidence of delivery. The client **SHOULD** keep the signed notice as its record of why it stopped using the context, for as long as it might be asked to account for that, and keeps each processed `noticeId` for at least 25 hours, longer than the provider's 24-hour delivery limit (see [Delivery rules carried from HCP](#delivery-rules-carried-from-hcp)). HCP requires the client to stop using the copies and pass none on, not to delete them (HCP 6.2.2); a copy kept under a declared legal obligation is kept only to meet that obligation (HCP 6.2.5).

### Consent/purpose

The notice exists to carry out the person's withdrawal of permission. Its data is used only to identify the grant whose context the client must stop using. Nothing in this task asks the person anything, and whether the client tells its own user is its policy ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).
