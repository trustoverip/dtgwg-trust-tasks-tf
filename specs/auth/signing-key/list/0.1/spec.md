---
slug: auth/signing-key/list
version: "0.1"
title: "Auth — Signing Key List"
summary: An identity lists the signing keys delegated to act for it — which devices can sign in its name, when each was last used, and which were revoked — so it can tell them apart before revoking one.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - signing-key
  - delegation
  - console
parties:
  - role: identity, or a key delegated to act for it
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: consumer holding the delegations
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The list is an inventory of every device that can sign as the identity, which ones have gone quiet, and which were disowned — reconnaissance for anyone deciding which key to steal or which to revoke to lock the identity out. Enumeration is therefore tied to a signature the consumer resolves to the identity, rather than to a bearer token, the same strength auth/passkey/list asks for.
sideEffects:
  level: none
  rationale: >-
    Reads the caller's delegation records. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses the caller's own delegations — public keys, labels, timestamps — to the caller. No key material and no other identity's delegations.
retention:
  class: transient
  rationale: >-
    A read. The consumer needs nothing from it after answering, and the caller's copy is a point-in-time view that goes stale at the next enrolment or revocation.
errorCodes: []
related:
  - auth/signing-key/enroll
  - auth/signing-key/revoke
  - auth/passkey/list
---

## Abstract

The **Auth — Signing Key List** Trust Task returns every signing-key delegation enrolled for the caller's identity through [`auth/signing-key/enroll`](../../enroll/0.1/spec.md): which `did:key`s may sign in its name, with the label each was given, when each was enrolled, when it was last used, when it expires, and whether it was revoked. It is the input to [`auth/signing-key/revoke`](../../revoke/0.1/spec.md), and the "this browser / other browsers" list an administration console shows beside its passkeys.

Revoked delegations are included so a holder can see that a device was disowned rather than never enrolled.

**Why there is no `show` sibling.** The registry's default for a collection is a list-and-show pair, because a filtered list cannot say `notFound`. This collection is the caller's own and is small — a handful of device profiles — so the whole of it is returned every time, and a caller wanting one entry finds it by `signingKeyDid`. Whether a given key exists is not load-bearing for anybody but its own identity, which already receives the complete list; a per-key lookup would add a probe surface (is this key enrolled here?) and answer nothing the list does not.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer**:

1. Resolves the proof's signer to an identity — the signer itself, or, for a signer that is an active delegated key, the identity that delegation acts for — and lists the delegations enrolled for **that** identity. It **MUST NOT** list delegations of any other identity, and the payload carries no member that could ask it to.
2. **MUST** answer `signingKeys: []`, not an error, for an identity with none: an identity that has never enrolled a key is the first state a console renders.
3. **MUST** include revoked and expired delegations, **MUST** report each one's `scope` and the `expiresAt` it set at enrolment, and **MUST** compute `active` with the same predicate its verifier applies to a signed document.
4. **SHOULD** order the list newest first, so a key enrolled a moment ago by somebody else is at the top rather than buried.
5. **MUST NOT** answer a caller whose signer resolves to no identity it recognises — it refuses with `permissionDenied`.

## Definitions

- **`signingKeys`** — every delegation for the caller's identity, each a `SigningKey` from [`auth/_shared/0.1/signing-key.schema.json`](../../../_shared/0.1/signing-key.schema.json).
- **`active`** — not revoked and not past `expiresAt`, at the moment the consumer answered.

## Request

The identity, or one of its delegated keys (`issuer`), sends an empty payload to the consumer (`recipient`).

### A console lists the keys that can sign for its administrator

```json
{
  "id": "urn:uuid:8d1b6a3c-2e7f-4c1a-9b8e-3a5d7c9e1f01",
  "type": "https://trusttasks.org/spec/auth/signing-key/list/0.1#request",
  "issuer": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-09-28T11:00:00Z",
  "threadId": "urn:uuid:8d1b6a3c-2e7f-4c1a-9b8e-3a5d7c9e1fff",
  "payload": {}
}
```

## Response

The consumer answers with the sub-schema reachable via `$anchor: "response"`: `signingKeys`, newest first. A refusal is a `trust-task-error`.

### One key in use, one disowned

```json
{
  "id": "urn:uuid:8d1b6a3c-2e7f-4c1a-9b8e-3a5d7c9e1f02",
  "type": "https://trusttasks.org/spec/auth/signing-key/list/0.1#response",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
  "issuedAt": "2026-09-28T11:00:01Z",
  "threadId": "urn:uuid:8d1b6a3c-2e7f-4c1a-9b8e-3a5d7c9e1fff",
  "payload": {
    "signingKeys": [
      {
        "signingKeyDid": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
        "identityDid": "did:web:alice.example",
        "scope": "console",
        "deviceLabel": "Work laptop — Chrome",
        "createdAt": "2026-09-28T10:00:31Z",
        "expiresAt": "2026-10-28T10:00:31Z",
        "lastUsedAt": "2026-09-28T10:58:12Z",
        "active": true
      },
      {
        "signingKeyDid": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "identityDid": "did:web:alice.example",
        "scope": "console",
        "deviceLabel": "Old desktop — Firefox",
        "createdAt": "2026-09-02T08:14:00Z",
        "expiresAt": "2026-10-02T08:14:00Z",
        "revokedAt": "2026-09-20T17:40:00Z",
        "active": false
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Public keys, labels and timestamps of the caller's own delegations. No private key material, and no other identity's records. `deviceLabel` is free text the identity itself wrote, returned to that identity.

### Correlation

The consumer is declared with identifier scope `public`: it is the one party every holder of a delegation addresses, and the delegation is meaningful only at the consumer that recorded it, so a caller has to be able to recognise it. A pairwise identifier for it would leave a holder unable to tell which consumer a key is enrolled at.

The response ties several `did:key`s to one identity — but only for the identity itself, which already knows. `lastUsedAt` reveals activity timing to the same party. Nothing here is disclosed to a third party.

### Retention

A read. The consumer keeps nothing; the caller's copy is stale at the next enrolment or revocation and **SHOULD NOT** be treated as a durable record of which keys are live.

### Consent/purpose

The purpose is to let an identity see, and choose among, the keys that can sign in its name. Using the listing to decide anything about another identity is outside it and not possible: the list is always the caller's own.
