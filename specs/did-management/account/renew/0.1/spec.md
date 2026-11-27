---
slug: did-management/account/renew
version: "0.1"
title: "DID Management — Account Renew"
summary: "An account holder tells its DID hosting service the account is still in use, restarting its inactivity period, and restoring it if it was disabled for inactivity."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did, did-hosting, account, renew, heartbeat, liveness, expiry, inactivity]
parties:
  - role: Account holder
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: DID hosting service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The renewal keeps an account and its DIDs alive and can restore a disabled one, so it must come from the holder:
    only the holder's signature, checked against its DID document, shows that. A renewal anyone could send would let
    a third party hold an abandoned account open indefinitely.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A renewal restarts a clock. A replayed one would restart it again on the holder's behalf after the holder stopped,
    and the duplicate-execution record of SPEC §7.2 item 11 is bounded by this timestamp.
sideEffects:
  level: mutating
  rationale: >-
    Restarts the account's inactivity period, and restores a disabled account's DIDs to service. Repeating it changes
    nothing but the clock.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: >-
    The request carries nothing; the response returns the holder's own account, including when it would next be
    disabled.
retention:
  class: transient
  rationale: >-
    The only lasting effect is the account's liveness timestamp, which the service keeps as account state, not as a
    record of this document.
errorCodes:
  - code: did-management/account/renew:noAccount
    meaning: The issuer holds no account on this service, or held one that has been purged. A purged account is registered again with did-management/account/register.
    retryable: false
  - code: did-management/account/renew:suspended
    meaning: The account was disabled by an administrator, not for inactivity. Only an administrator restores it.
    retryable: false
related:
  - did-management/account/register
  - did-management/account/status
  - did-management/account/expiry-notice
---

## Abstract

A DID hosting service that admits accounts it has no other relationship with (did-management/account/register) needs a way to reclaim the ones nobody uses. It does so by inactivity: an account no one has shown to be in use for a period the service sets is disabled — its DIDs stop resolving — and, after a further grace period, purged.

**Account Renew** is the holder's way of saying "still in use". It is meant to be sent by software, not by a person: an agent that holds a hosted DID sends it when it starts and periodically while it runs, so an account in use never comes near expiry. Sent during the grace period, it restores a disabled account.

Renewal is not the only liveness signal. A service **MAY** also count any other task the holder sends, and resolutions of the account's DIDs (`lastResolvedAt`); this task is the one signal the holder controls directly.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the hosting service):

1. **MUST** act only on the account whose subject is the identity `issuer` names (a `did:scid` without its `src`), and answer `noAccount` when there is none.
2. **MUST** answer `suspended`, and change nothing, when an administrator disabled the account; renewal restores only an account disabled for inactivity.
3. Otherwise **MUST** record the holder as active now, set the account `active`, recompute `disableAt`, restore a `disabled` account's DIDs to service, and answer with the account and whether its state changed.

Renewal is a *setting a state* operation ([SPEC §7.2](/SPEC.md#72-consumer-requirements)): renewing an `active` account succeeds, with `changed: false`.

A conforming **producer** **SHOULD** renew when it starts and at least daily while it runs, and **SHOULD** read `disableAt` from the response rather than assume the service's periods.

## Authorization

The entitlement is **holding the account**: being the DID it was registered to. The consumer establishes it by verifying the document's proof against the issuer's DID document, for `authentication`, and comparing the issuer's identity with the account's `subject`. No other party may renew an account, administrators included — an administrator who wants an account kept exempts it from expiry through the service's own administration, which is not this task.

## Definitions

* **Account holder.** The DID an account was registered to (`subject`).
* **Inactivity period, grace period.** The service's two durations: how long an account may go without a liveness signal before it is disabled, and how long a disabled account is kept before it is purged. `disableAt` and `purgeAt` state them for one account.

## Request

The holder sends the request to its hosting service; the payload, which validates against the top-level schema in [`payload.schema.json`](payload.schema.json), is empty.

### A daily renewal

```json
{
  "id": "urn:uuid:3f2e1d0c-9b8a-4f7e-6d5c-4b3a2f1e0d9c",
  "type": "https://trusttasks.org/spec/did-management/account/renew/0.1",
  "issuer": "did:scid:vh:1:QmScidExample",
  "recipient": "did:webvh:QmHostScid:did.example.com",
  "issuedAt": "2027-01-15T08:00:00Z",
  "payload": {},
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-01-15T08:00:00Z",
    "verificationMethod": "did:scid:vh:1:QmScidExample#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z6JqPoX3xdRbW"
  }
}
```

## Response

The service answers with the account as it now stands and whether the renewal changed its state, validating against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### The account's inactivity period restarts

```json
{
  "id": "urn:uuid:4a3b2c1d-0e9f-4a8b-7c6d-5e4f3a2b1c0d",
  "type": "https://trusttasks.org/spec/did-management/account/renew/0.1#response",
  "threadId": "urn:uuid:3f2e1d0c-9b8a-4f7e-6d5c-4b3a2f1e0d9c",
  "issuer": "did:webvh:QmHostScid:did.example.com",
  "recipient": "did:scid:vh:1:QmScidExample",
  "issuedAt": "2027-01-15T08:00:01Z",
  "payload": {
    "account": {
      "subject": "did:scid:vh:1:QmScidExample",
      "role": "guest",
      "domain": "did.example.com",
      "state": "active",
      "admission": "invite",
      "registeredAt": "2026-12-01T09:00:01Z",
      "lastActiveAt": "2027-01-15T08:00:01Z",
      "disableAt": "2027-04-15T08:00:01Z"
    },
    "changed": false
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-01-15T08:00:01Z",
    "verificationMethod": "did:webvh:QmHostScid:did.example.com#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z7KqPoX3xdRbW"
  }
}
```

## Security & Privacy

### Data carried

The request carries nothing beyond the envelope. The response carries the holder's own account. A producer **MUST NOT** use `ext` to send anything about the agent's use of its DID.

### Correlation

`issuer` is the holder's public DID (`identifierScope: public`), the identity the account belongs to; a pairwise identifier would name no account. Renewal timing is observable to the service and shows when the holder's software runs. A producer that does not want that pattern visible **MAY** add jitter to its schedule; the service needs only that renewals arrive within the inactivity period.

### Retention

The service keeps only the time of the latest liveness signal, which it **MAY** record coarsely (for example at most once a day). It need not keep the document.

### Consent/purpose

The liveness timestamp exists to decide whether the account is still in use. A service **MUST NOT** use renewal timing for anything else, such as profiling when the holder is online.
