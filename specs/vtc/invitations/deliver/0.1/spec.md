---
slug: vtc/invitations/deliver
version: "0.1"
title: "VTC Invitations — Deliver"
summary: "Deliver an issued invitation to the DID it admits — pushed to that DID as a credential offer, or returned as an offer small enough for a QR code — releasable only to that DID."
status: draft
targetFrameworkVersion: "0.6.0"
category: credentials
parties:
  - role: inviter
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Sends a community's invitation to a party or hands its offer to the inviter. The inviter
    who chose to deliver it must be attributable, as the one who issued it is.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A mutating task. A replayed delivery replaces the offer the inviter last handed over, so the
    code in that QR stops redeeming; placing each delivery in time is what lets the inviter
    tell which offer is live.
sideEffects:
  level: mutating
  rationale: >-
    Records a single-use offer for the invitation, replacing any earlier one, and on the
    `message` channel sends it to the invited DID.
exposure:
  discloses: metadata
  actsAsSubject: false
  rationale: >-
    The offer carries a pre-authorized code, but the code redeems only for a request whose
    key-binding proof is by the invited DID's key, so it admits no one else. The invitation
    credential itself is never in this task's response.
errorCodes:
  - code: vtc/invitations/deliver:notFound
    meaning: No invitation with that `id` exists.
    retryable: false
  - code: vtc/invitations/deliver:revoked
    meaning: The invitation has been revoked, so there is nothing to deliver.
    retryable: false
  - code: vtc/invitations/deliver:noRoute
    meaning: On the `message` channel, the invited DID advertises no transport this community can send over. The `offer` channel still works.
    retryable: false
related:
  - vtc/invitations/issue
  - vtc/invitations/list
  - vtc/invitations/revoke
  - credential-exchange/offer
  - credential-exchange/request
  - credential-exchange/issue
---

## Abstract

The **VTC Invitations — Deliver** Trust Task gets an issued invitation to the DID it admits. On the `message` channel the community sends that DID a [`credential-exchange/offer`](../../../../credential-exchange/offer/0.1/); on the `offer` channel it returns the offer to the inviter, to hand over out of band — typically as a QR code. Either way the invitee redeems it with [`credential-exchange/request`](../../../../credential-exchange/request/0.1/) and receives the credential in [`credential-exchange/issue`](../../../../credential-exchange/issue/0.1/).

[`vtc/invitations/issue`](../../issue/0.1/) returns the signed invitation to the inviter once, and nothing carried it further: an invitee was reached only by whatever the inviter improvised, and a credential large enough to be useful is too large for a QR code. This task closes both gaps with an offer: small, because it names the credential rather than containing it, and safe to photograph, because it redeems only for the invited DID.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

**Producer.** Name an issued invitation and a channel. Choose `offer` when the invitee has no messaging service, or when the inviter will be present to hand the offer over.

**Consumer.** Refuse an unknown `id` with `notFound`, a revoked invitation with `revoked`, and a lapsed one with the standard `expired` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)). Otherwise it MUST:

1. record a single-use offer for the invitation's credential, bound to the invitation's subject DID, lapsing no later than the invitation does. Any earlier offer for the same invitation MUST stop redeeming, so at most one offer per invitation is live;
2. on `message`, send a `credential-exchange/offer` carrying it to the subject DID over a transport that DID advertises, in the stack's order of preference, and refuse with `noRoute` if it advertises none the consumer can use; on `offer`, return it in `offer`;
3. release the credential only in answer to a `credential-exchange/request` carrying that offer's code, whose key-binding proof is by a key of the subject DID. A request proven by any other key MUST be refused, whoever sends it.

The consumer MUST NOT include the invitation credential in this task's response. It reaches only the invitee, through `credential-exchange/issue`.

## Authorization

The entitlement is the **inviter capability** — the same one [`vtc/invitations/issue`](../../issue/0.1/) requires (administrator, moderator or issuer), verified against the community's own access-control list. Verifying the `proof` establishes who asked; per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements) it does not establish that they may.

The invitee's entitlement to the credential is separate, and is not the inviter's to confer: it is the key-binding proof in `credential-exchange/request`, by a key of the DID the invitation names. So an inviter who delivers to the wrong place, or whose QR code is photographed, has disclosed nothing that admits anyone else.

## Definitions

- **`id`** (request, response) — the invitation's identifier, as `vtc/invitations/list` reports it.
- **`channel`** (request, response) — `message` to send the offer to the invited DID, `offer` to return it to the inviter.
- **`offer`** (response) — on the `offer` channel only: the OID4VCI Credential Offer, verbatim. It names the credential and carries a pre-authorized code; it does not contain the credential.
- **`expiresAt`** (response) — when this delivery's offer lapses.

## Request

Sent by an inviter to the community, carrying the payload defined by the top-level schema in [`payload.schema.json`](payload.schema.json).

### Returning an offer for a QR code

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/invitations/deliver/0.1#request",
  "issuer": "did:example:inviter",
  "recipient": "did:webvh:QmCommunityScid:community.example",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "id": "urn:uuid:5f1c0b7e-9a4e-4c2a-8d11-3b0e6f7a2c90",
    "channel": "offer"
  }
}
```

## Response

Sent by the community, carrying the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error`, not a `#response` document.

### The offer to render

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/invitations/deliver/0.1#response",
  "issuer": "did:webvh:QmCommunityScid:community.example",
  "recipient": "did:example:inviter",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "id": "urn:uuid:5f1c0b7e-9a4e-4c2a-8d11-3b0e6f7a2c90",
    "channel": "offer",
    "offer": {
      "credential_issuer": "did:webvh:QmCommunityScid:community.example",
      "credential_configuration_ids": ["VIC"],
      "grants": {
        "urn:ietf:params:oauth:grant-type:pre-authorized_code": {
          "pre-authorized_code": "pac_3f9d2c1a7b6e4d5c8a0b1e2f3a4b5c6d"
        }
      }
    },
    "expiresAt": "2026-01-08T00:00:01Z"
  }
}
```

On the `message` channel the response carries no `offer`: the offer went to the invitee.

## Security & Privacy

### Data carried

The request carries an invitation identifier and a channel. The response carries, on the `offer` channel, an offer naming the credential and a pre-authorized code. That code is the one member worth pausing on: it is a bearer value in OID4VCI generally, but here it redeems only for a request proven by the invited DID's key, so disclosing it — in a photographed QR code, a relayed message, a log — admits no one else. The invitation credential itself never appears in this task.

### Correlation

The offer names the community and, through its code, one invitation. An observer of a QR code learns that someone was invited to that community, which the invitee's presence usually discloses anyway. On the `message` channel the transport carries the offer to the invited DID; TSP's metadata-private routing hides that recipient from intermediaries, which is one reason the stack prefers it.

### Retention

The community keeps the live offer until it is redeemed, replaced by a later delivery, or lapses, and then discards it. The delivery request is audit evidence of who sent the invitation where, and when.

### Consent/purpose

The offer exists to let the invited DID collect its invitation, and is used for nothing else. Whether a delivery needs a further approval is the community's policy, not this specification's.
