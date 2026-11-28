---
slug: did-management/account/expiry-notice
version: "0.1"
title: "DID Management — Account Expiry Notice"
summary: "A DID hosting service tells an account holder that its account is about to be disabled for inactivity, has been disabled, or is about to be purged, so the holder can renew in time."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did, did-hosting, account, expiry, notification, decommission, inactivity, warning]
parties:
  - role: DID hosting service
    requirement: REQUIRED
    member: issuer
  - role: Account holder
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    A holder acts on the notice — surfacing it to a person, or renewing — and a forged one could be used to alarm a
    holder or to probe whether it is listening. The service's signature, checked against the service's DID document,
    is what lets the holder trust that its own hosting service sent it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A notice reports a state at a moment; a holder must be able to tell a current notice from a replayed old one, which
    could otherwise report a disablement the holder has since reversed.
sideEffects:
  level: none
  rationale: >-
    Informs only. The holder changes nothing by receiving it; whether it renews is its own decision, made through
    did-management/account/renew.
exposure:
  discloses: none
  actsAsSubject: false
  ingests: metadata
  rationale: >-
    The notice carries the holder's own account to the holder; nothing goes back.
retention:
  class: transient
  rationale: >-
    The notice is superseded by the account's state, which the holder can read at any time with
    did-management/account/status.
errorCodes: []
related:
  - did-management/account/renew
  - did-management/account/status
  - did-management/account/register
---

## Abstract

A hosting service that reclaims inactive accounts (did-management/account/renew) disables an account's DIDs and later purges them. Both are visible to the holder only after the fact — its DIDs stop resolving — unless the service says so first. **Account Expiry Notice** is how it does: a signed notice sent to the holder's DID, ahead of disablement, at disablement, and ahead of purge.

The notice is best-effort and never the only way to learn. A holder whose DID document advertises no messaging transport cannot be reached, and one that is offline will not receive it; did-management/account/status answers the same question on demand.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the hosting service):

1. **SHOULD** send `disablePending` at least once before `disableAt`, `disabled` when it disables the account, and `purgePending` before `purgeAt`.
2. **MUST** send a notice only to the account's `subject`, over a transport that subject's DID document advertises, and **MUST NOT** fall back to any other channel.
3. **SHOULD** send each stage once per occurrence, and retry delivery of a notice it could not deliver, but **MUST NOT** report a failed delivery as an error on the account: delivery is best-effort.
4. **MUST** carry the account as it stands when the notice is sent.

A conforming **consumer** (the account holder):

1. **MUST** verify the notice's proof against the hosting service's DID document and refuse one that is not from the service it holds an account with.
2. **MUST NOT** treat a notice as the account's current state where it can ask (did-management/account/status), and **MUST NOT** treat the absence of a notice as evidence that its account is in good standing ([SPEC §11.6](/SPEC.md#116-notifications-and-the-meaning-of-silence)).
3. **MAY** acknowledge with an empty `#response`; the producer does not require one.

## Authorization

The notice confers and exercises no authority: it describes the recipient's own account to the recipient, and the recipient does nothing it was not already entitled to do. The proof establishes only that the notice comes from the hosting service, so the holder can rely on what it says. Acting on it — renewing — is authorized by the holder's own account, through did-management/account/renew.

## Definitions

* **Stage.** `stage`: `disablePending`, `disabled` or `purgePending`, as the schema says.
* **Account holder.** The DID an account was registered to (`subject`), and the recipient of the notice.

## Request

The hosting service sends the notice to the account holder; the payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### The account will be disabled in fourteen days

```json
{
  "id": "urn:uuid:8e7d6c5b-4a3f-4e2d-1c0b-9a8f7e6d5c4b",
  "type": "https://trusttasks.org/spec/did-management/account/expiry-notice/0.1",
  "issuer": "did:webvh:QmHostScid:did.example.com",
  "recipient": "did:scid:vh:1:QmScidExample",
  "issuedAt": "2027-03-01T18:00:00Z",
  "payload": {
    "stage": "disablePending",
    "account": {
      "subject": "did:scid:vh:1:QmScidExample",
      "role": "owner",
      "domain": "did.example.com",
      "state": "warned",
      "admission": "invite",
      "registeredAt": "2026-12-01T09:00:01Z",
      "lastActiveAt": "2026-12-15T18:00:00Z",
      "disableAt": "2027-03-15T18:00:00Z"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-03-01T18:00:00Z",
    "verificationMethod": "did:webvh:QmHostScid:did.example.com#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z1NqPoX3xdRbW"
  }
}
```

### The account has been disabled

```json
{
  "id": "urn:uuid:9f8e7d6c-5b4a-4f3e-2d1c-0b9a8f7e6d5c",
  "type": "https://trusttasks.org/spec/did-management/account/expiry-notice/0.1",
  "issuer": "did:webvh:QmHostScid:did.example.com",
  "recipient": "did:scid:vh:1:QmScidExample",
  "issuedAt": "2027-03-15T18:00:05Z",
  "payload": {
    "stage": "disabled",
    "account": {
      "subject": "did:scid:vh:1:QmScidExample",
      "role": "owner",
      "domain": "did.example.com",
      "state": "disabled",
      "admission": "invite",
      "registeredAt": "2026-12-01T09:00:01Z",
      "lastActiveAt": "2026-12-15T18:00:00Z",
      "purgeAt": "2027-04-14T18:00:00Z"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-03-15T18:00:05Z",
    "verificationMethod": "did:webvh:QmHostScid:did.example.com#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z2PqPoX3xdRbW"
  }
}
```

## Security & Privacy

### Data carried

The notice carries the holder's own account and nothing else. A producer **MUST NOT** put anything about other accounts, or about the service's capacity, in `ext`.

### Correlation

The recipient is the holder's public DID (`identifierScope: public`), the identity the account belongs to; the notice is about that account, and a pairwise identifier would reach no one. A notice reveals to the holder's mediator, where one relays it, that the holder has an account with this service; the notice's content is protected by the transport's encryption where the transport provides it.

### Retention

Neither party needs to keep the notice: the account's state is authoritative and readable at any time. A holder **MAY** keep the latest notice to show a person.

### Consent/purpose

The notice exists to give the holder time to keep its account. The service sends it only to the account's own subject and uses the delivery for nothing else.
