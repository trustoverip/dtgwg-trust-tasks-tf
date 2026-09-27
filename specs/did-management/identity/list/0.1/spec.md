---
slug: did-management/identity/list
version: "0.1"
title: DID Management — Identity List
summary: An administrator lists the generations of a DID hosting service's own identity that it still honours — the current one and each superseded one, with when it stops being honoured.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did-hosting, identity, key-rotation, generations, admin]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: DID hosting service
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "The answer tells an administrator which superseded keys the service will still accept messages under, and until when — what they need to decide whether to cut one off. It is disclosed to administrators only, and the service authorises on the document's proven issuer on every transport."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no effect to replay, but an issue time dates the request for a service that audits administrative reads."
sideEffects:
  level: none
  rationale: "Reads the service's identity state; nothing is rotated, retired or dropped."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns key identifiers — the public verification-method ids a DID document advertises — and lifecycle instants. Never key material."
retention:
  class: transient
  rationale: "The request carries nothing but its envelope."
errorCodes: []
related:
  - did-management/identity/retire
  - did-management/server/config
---

## Abstract

A hosting service rotates its own identity by publishing a new version of its DID document with new keys. Peers holding a cached copy of the old document keep encrypting to, and verifying against, the old keys until their cache expires, so the service keeps honouring the superseded *generation* for a grace period and then drops its key material.

**Identity List** shows the administrator every generation the service still honours: which is current, what each advertised, and when each superseded one lapses. It is the read that precedes [`did-management/identity/retire`](../../retire/0.1/spec.md), the kill switch for a generation whose key is compromised.

A list without a read-one sibling suffices here: a service honours a handful of generations at most, and the one task that acts on a generation by identifier (`retire`) returns its own `notFound`.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming hosting service:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`).
2. Refuses a caller without administrator standing with `permissionDenied`.
3. Returns every generation it still honours, newest first, as the running process holds them — not as a store on disk says, since the running process is what accepts messages. Exactly one generation is `current`; a service with no rotating identity returns an empty list.
4. **MUST NOT** return key material.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.* Not consequential, and declared for clarity.

The entitlement is **administrator standing on this hosting service**, read from its own access-control records at execution time; the `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

**Generation** — one set of the service's keys, together with the DID document version that advertised them and the transports it was reachable on.

## Request

```json
{
  "id": "urn:uuid:d28eab0f-9fbd-4e01-9a0b-8d9e0f1a2b01",
  "type": "https://trusttasks.org/spec/did-management/identity/list/0.1",
  "issuer": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": {}
}
```

## Response

```json
{
  "id": "urn:uuid:d28eab0f-9fbd-4e01-9a0b-8d9e0f1a2b02",
  "type": "https://trusttasks.org/spec/did-management/identity/list/0.1#response",
  "threadId": "urn:uuid:d28eab0f-9fbd-4e01-9a0b-8d9e0f1a2b01",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": {
    "generations": [
      {
        "generationId": 2,
        "did": "did:webvh:QmControlScid2:control.example.com",
        "current": true,
        "signingKeyId": "did:webvh:QmControlScid2:control.example.com#key-2",
        "keyAgreementKeyId": "did:webvh:QmControlScid2:control.example.com#key-agreement-2",
        "mediatorDid": "did:webvh:QmMediatorScid6:mediator.example.com",
        "transports": { "tsp": true, "didcomm": true },
        "createdAt": "2026-09-26T12:00:00Z"
      },
      {
        "generationId": 1,
        "did": "did:webvh:QmControlScid2:control.example.com",
        "current": false,
        "signingKeyId": "did:webvh:QmControlScid2:control.example.com#key-1",
        "keyAgreementKeyId": "did:webvh:QmControlScid2:control.example.com#key-agreement-1",
        "mediatorDid": "did:webvh:QmMediatorScid6:mediator.example.com",
        "transports": { "tsp": false, "didcomm": true },
        "createdAt": "2026-09-01T10:00:00Z",
        "retiredAt": "2026-09-26T12:00:00Z",
        "expiresAt": "2026-10-03T12:00:00Z"
      }
    ],
    "rotationGraceSeconds": 604800
  }
}
```

## Security & Privacy

### Data carried

The response carries public key identifiers, mediator DIDs, transport flags and instants. Each key identifier is one the service's own DID document advertised, so none is secret; the information an attacker would value is `expiresAt` — how long a stolen superseded key keeps working — which is why the read is administrator-only.

### Correlation

The response concerns only the service itself. Both parties declare `identifierScope: public`, and must: the service's DID is the recipient every administrator addresses and the key its replies are verified against, and the administrator's DID is what the access-control entry is keyed on.

### Retention

Nothing of the request needs keeping beyond the exchange.

### Consent/purpose

The list is disclosed so an administrator can see what the service still accepts and decide whether to retire a generation early.
