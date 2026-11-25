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
keywords: [hcp, human-context-protocol, grant, revocation, notification, security-event-token]
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
    Event Token's `iat`, and the client's duplicate-execution record absorbs the provider's retries only inside a
    bounded window.
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
    The notice carries only a grant identifier and an instant; the acknowledgement carries nothing.
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

The **HCP — Revocation Notice** Trust Task carries the revocation notification of the [Human Context Protocol (HCP) 2.0-draft-3](https://github.com/loyalagents/hcp-spec/blob/b790561/spec/2.0-draft-3/specification.md) (HCP 6.2.3 and 9.9). When a grant is revoked, the *HCP Provider* tells the *Context Client* that held it, where that client holds disclosed entries and has registered a notification endpoint (HCP 6.2.2), and the client acknowledges. HCP sends this notice as a signed Security Event Token (RFC 8417) delivered by push (RFC 8935); this task carries the same event as a signed Trust Task document, so the provider can reach the client over any Trust Task binding.

It is a member of the HCP *protocol family* ([SPEC §9.4](/SPEC.md#94-carried-protocols)). The revocation itself is [`hcp/grants/revoke`](../../revoke/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

HCP 2.0-draft-3 is itself a draft, and its editors have not published or endorsed this carriage ([SPEC §9.4](/SPEC.md#94-carried-protocols), [SPEC §6.5](/SPEC.md#65-private-and-unpublished-trust-task-specifications)). This family is a proposal to them. HCP's text is © its authors under CC BY 4.0; this specification was written from that text, with attribution, and where it differs from the HCP text, the HCP text governs.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here. A provider and client that use this task **MUST** also meet every requirement HCP 2.0-draft-3 places on revocation notices; the ones that bear on the document are restated below, unchanged. Conformance to HCP is HCP's ([SPEC §9.4](/SPEC.md#94-carried-protocols)).

## Authorization

The authority is **the provider's position as the holder of the grant**: only the provider that issued a grant can say it has ended. The client accepts the notice only when its `proof` verifies under a key of that provider. Here the proof is the authorization in the sense of [CONTRIBUTING-SPECS](../../../../../CONTRIBUTING-SPECS.md#what-to-write): a verified assertion by the one party entitled to make it, for the one purpose of ending use of that grant.

HCP requires the notice to be signed with an asymmetric algorithm, with a key the provider publishes at its `jwks_uri`, and the client to verify it (HCP 9.9). A Trust Task proof is made with a key of the `issuer` VID. To meet both, the provider **MUST** sign the notice with a key that its VID's DID document lists **and** that its HCP JWKS also publishes, so that a client verifying under either HCP's rule or this framework's finds the same key. A client **MUST** check that the `issuer` VID is bound to the provider that issued the grant (see [Parties](#parties)) and **MUST** refuse a notice whose key it cannot find in that provider's JWKS.

The client does not need to hold any authority of its own to acknowledge.

## Definitions

- **HCP Provider**, **Context Client** and **grant** have their HCP 2.0-draft-3 meanings.
- The **grant-revoked event** is HCP's event `https://hcp.example/spec/events/grant-revoked`, with members `grantId` and `revokedAt`.

### HCP field mapping

The Security Event Token's claims map to the Trust Task envelope, and the event's members to the payload. The SET's `events` wrapper is replaced by the Type URI, which names the one event this task carries.

| HCP / SET field | Member | Notes |
|---|---|---|
| `iss` | `issuer` | The provider's VID. |
| `aud` | `recipient` | The client's VID. |
| `jti` | `id` | **Identical on every delivery attempt**, so the client's duplicate-execution record ([SPEC §7.2 item 11](/SPEC.md#72-consumer-requirements)) absorbs retries. |
| `iat` | `issuedAt` | Whole seconds, as SET instants already are. Identical on every attempt. |
| `events` key `https://hcp.example/spec/events/grant-revoked` | the Type URI | |
| event `grantId` | `grantId` | Unchanged. |
| event `revokedAt` | `revokedAt` | Unchanged. |
| JWS signature | `proof` | See [Authorization](#authorization) for the key. |
| RFC 8935 `202 Accepted` | the empty `#response` payload | |
| RFC 8935 `400` error codes | `trust-task-error` codes | See [Errors](#errors). |

### Parties

| HCP role | Trust Task member |
|---|---|
| HCP Provider | `issuer` of the notice, `recipient` of the acknowledgement |
| Context Client | `recipient` of the notice, `issuer` of the acknowledgement |

HCP sends the notice to the notification endpoint the client registered. Here the client's VID stands in for that endpoint: at registration, the client **MUST** give the provider the VID it will receive notices at, and the provider **MUST** send the notice to that VID and no other. The provider's VID is bound to its HCP identity as [`hcp/entries/query`](../../../entries/query/0.1/spec.md#parties) describes.

## Request

The provider sends the notice to the client. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json): `grantId`, the grant that was revoked, and `revokedAt`, when. Both are REQUIRED.

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

A client acknowledges a re-delivered notice the same way, from its duplicate-execution record.

## Errors

RFC 8935's `400` error codes map to Trust Task errors as follows.

| RFC 8935 code | Trust Task error |
|---|---|
| `invalid_request` | `malformedRequest` |
| `invalid_key`, `authentication_failed` | `proofInvalid` |
| `invalid_audience` | `wrongRecipient` |
| `invalid_issuer`, `access_denied` | `permissionDenied` |

Every one of these is `retryable: false`. A non-retryable error ends delivery, and the provider records it as abandoned. A client that is temporarily unable to process the notice answers `unavailable`, and the provider retries.

## Delivery rules carried from HCP

- **When a notice is sent.** Only if the client holds entries disclosed under the grant and has registered a notification endpoint, which here is its VID (see [Parties](#parties)) (HCP 6.2.2, 6.2.3). A client with no endpoint is not notified; its copies were capped at one hour of use (HCP 7.6).
- **Retries.** Until it receives an acknowledgement or a non-retryable error, the provider **MUST** retry on a failed delivery and on any answer other than an acknowledgement or a non-retryable error, with exponential backoff, for at least 5 attempts over at least one hour, with the same document: the same `id`, `issuedAt` and `proof` (HCP 9.9).
- **Audit.** The provider records `revocation.notified` on the first acknowledgement, or when it abandons delivery (HCP 6.2.3), and meets HCP's duties on abandonment (HCP 6.2.4).
- **Freshness.** HCP's rule that a client **SHOULD** reject a SET whose `iat` differs from local time by more than 300 seconds (HCP 9.9) applies to `issuedAt` unchanged. HCP's own text is in tension here: every retry over at least one hour carries the same token, so a retry after 300 seconds is stale by that rule. This specification does not resolve it; it is an open question for the HCP editors. A client that acknowledged the first delivery is unaffected, because its duplicate-execution record answers a retry before freshness matters: that record **MUST** cover at least the provider's whole retry period, so a retry inside it is acknowledged, not executed again. A client that never received an earlier attempt and rejects a late one as stale leaves the provider to abandon delivery (HCP 6.2.4).
- **No notice after relinquishment.** The provider sends no notice to a client that relinquished the grant itself (HCP 5.2.7).
- **Independence.** The revocation took effect at `revokedAt` whether or not this notice is ever delivered (HCP 6.2.3). A client that learns of the revocation another way, such as `hcp:invalidToken` on a query (HCP 6.2.2), acts on it at once.
- **On receipt.** The client **MUST** stop using every copy of context disclosed under the grant, whatever its `useUntil`, and **MUST NOT** pass any of them on (HCP 6.2.2, 7.6). The one exception is a copy kept in a record under a legal obligation the client declared at registration, which it keeps only to meet that obligation (HCP 6.2.5). It also stops using the grant's token. HCP does not require deletion, and the provider cannot enforce it (HCP 6.2.2).
- **A delegation that ends.** When a delegation ends, every grant authorized under it ends with status `expired`, not `revoked`, and "invalidation and notification follow 6.2.2–6.2.4, as for a revocation" (HCP 17.6). HCP names only the grant-revoked event, so whether such a grant is announced with this notice is an open question for the HCP editors. This version of the task is defined for revocation.

Over TSP without a return path, the acknowledgement is delivered best-effort ([TSP binding 0.2 §6](../../../../../bindings/tsp/0.2/spec.md)), so the provider may see no acknowledgement for a notice the client accepted. It then retries, and abandons as HCP describes.

## What HCP requires that a document cannot express

- The retry schedule and the abandonment duty (HCP 9.9, 6.2.4).
- The client's end of use of disclosed copies, and its passing none on (HCP 6.2.2), which no acknowledgement can prove.
- The key-publication rule: the signing key's presence in the provider's JWKS (HCP 9.9) is checked outside the document.

## Security & Privacy

### Data carried

The notice carries a grant identifier and the instant of revocation; the acknowledgement carries nothing. Neither carries context or tokens. A producer **MUST NOT** put the revocation's reason, or anything about the person, in `ext`: HCP's event carries no reason, and a reason could disclose why the person withdrew.

### Correlation

The `grantId` joins this notice to the revocation request and to the client's earlier queries under the grant; that join is the purpose of the notice. The client learns only that, and when, the grant ended. The timing of the notice can reveal that the person acted at that moment; HCP accepts that cost. `threadId` **SHOULD** be fresh per revocation and the same on every retry.

### Retention

The provider keeps the `revocation.notified` audit event (HCP 6.2.3) and **SHOULD** keep the acknowledgement with it as evidence of delivery. The client **SHOULD** keep the signed notice as its record of why it stopped using the context, for as long as it might be asked to account for that. HCP requires the client to stop using the copies and pass none on, not to delete them (HCP 6.2.2); a copy kept under a declared legal obligation is kept only to meet that obligation (HCP 6.2.5).

### Consent/purpose

The notice exists to carry out the person's withdrawal of permission. Its data is used only to identify the grant whose context the client must stop using. Nothing in this task asks the person anything, and whether the client tells its own user is its policy ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)).
