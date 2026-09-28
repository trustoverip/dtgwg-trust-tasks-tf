---
slug: auth/signing-key/revoke
version: "0.1"
title: "Auth — Signing Key Revoke"
summary: Revoke one signing-key delegation, so the next document its key signs is refused — by the identity it acts for, or by an administrator doing incident response.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - signing-key
  - delegation
  - revocation
parties:
  - role: owning identity, one of its keys, or an administrator
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
    The consumer decides whether the caller may revoke by comparing the signer's resolved identity with the delegation's owner, so the signer has to be attributable on every transport. A revocation is also the record an operator points to after an incident, which a proof makes durable.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replay changes nothing — a revoked key stays revoked — but §7.2 item 11 still needs a bounded window to recognise the duplicate.
sideEffects:
  level: destructive
  rationale: >-
    Revocation is irreversible: the delegation becomes a tombstone, and the same key can never be enrolled again. It removes authority and confers none, which is why no gesture beyond the caller's own standing is presupposed — but it cannot be undone except by generating and enrolling a new key.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request names a key; the response reports when it was revoked and how many of its identity's keys remain active. No key material either way.
retention:
  class: durable
  rationale: >-
    The tombstone is kept so the key cannot be re-enrolled and so the identity's list shows that a device was disowned. The revocation is audited against the caller.
errorCodes:
  - code: auth/signing-key/revoke:notFound
    meaning: "No delegation for this key exists that the caller may revoke. Returned identically for a key that was never enrolled and for one belonging to an identity the caller has no authority over, so the code cannot be used to probe which keys are enrolled for whom."
    retryable: false
related:
  - auth/signing-key/enroll
  - auth/signing-key/list
  - auth/passkey/revoke/start
---

## Abstract

The **Auth — Signing Key Revoke** Trust Task withdraws one delegation enrolled by [`auth/signing-key/enroll`](../../enroll/0.1/spec.md). From the moment it executes, a document signed by the key is refused, and the key cannot be enrolled again.

Revocation is the lever an identity reaches for when a device is lost, a browser profile is suspect, or a key has simply outlived its use. It presupposes nothing beyond the caller's standing over the delegation: requiring a fresh gesture to *withdraw* a credential would be a gate that protects the attacker, since an operator who suspects a device should be able to disown it before they reach for their authenticator.

Revoking every key an identity has is a legitimate state. It costs the identity the delegated signing path and nothing else: its own standing, its passkeys and its own key are untouched.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer**:

1. Resolves the proof's signer to an identity — the signer itself, or the identity an active delegated key acts for. It **MUST** accept a revocation signed by the owning identity, and **MUST** accept one signed by the delegated key being revoked, so a console can revoke its own key when its user signs out. It **MAY** accept one signed by another active key of the same identity.
2. **MUST** revoke only a delegation owned by that identity, unless the caller holds the consumer's highest administrative standing (at a community, an unrestricted administrator), which **MAY** revoke any delegation for incident response. A caller with narrower administrative standing **MUST NOT** revoke a peer's delegation: revocation cannot escalate, but it can deny service.
3. **MUST** answer `notFound` both for a key it holds no delegation for and for a delegation the caller may not revoke.
4. **MUST** leave a tombstone rather than delete the record, so the key cannot be enrolled again.
5. **MUST** succeed, changing nothing, for a delegation that is already revoked, and report its original `revokedAt`.
6. **MUST** serialise the revocation with enrolment, so a revocation cannot interleave with a concurrent enrolment of the same key.
7. **MUST** report `remainingActive` — the owning identity's delegations still active after the revocation — and **MUST** audit the revocation against the caller, naming the key.

## Authorization

The authority this task presupposes is **ownership of the delegation** — being the identity it acts for, or being the delegated key itself — or the consumer's highest administrative standing, for incident response. Verifying the proof establishes which identity asked; the comparison with the delegation's owner is what authorizes ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The final decision is the consumer's.

## Definitions

- **`signingKeyDid`** — the delegated key to revoke, as returned by [`auth/signing-key/list`](../../list/0.1/spec.md).
- **Tombstone** — the revoked record, kept so the key stays burned.
- **`remainingActive`** — the owning identity's active delegations after the revocation.

## Request

The caller (`issuer`) names the key to revoke to the consumer (`recipient`).

### An administrator disowns a lost laptop from another device

```json
{
  "id": "urn:uuid:c3f7e2a1-9b4d-4e6f-8a2c-5d1e7b9f3a01",
  "type": "https://trusttasks.org/spec/auth/signing-key/revoke/0.1#request",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:community.example",
  "issuedAt": "2026-09-28T12:00:00Z",
  "threadId": "urn:uuid:c3f7e2a1-9b4d-4e6f-8a2c-5d1e7b9f3aff",
  "payload": {
    "signingKeyDid": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp"
  }
}
```

## Response

The consumer answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### Revoked; one key remains

```json
{
  "id": "urn:uuid:c3f7e2a1-9b4d-4e6f-8a2c-5d1e7b9f3a02",
  "type": "https://trusttasks.org/spec/auth/signing-key/revoke/0.1#response",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-09-28T12:00:01Z",
  "threadId": "urn:uuid:c3f7e2a1-9b4d-4e6f-8a2c-5d1e7b9f3aff",
  "payload": {
    "signingKeyDid": "did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp",
    "revokedAt": "2026-09-28T12:00:01Z",
    "remainingActive": 1
  }
}
```

## Security & Privacy

### Data carried

A key DID in, a timestamp and a count out. Nothing secret, and no free text.

### Correlation

The consumer is declared with identifier scope `public`: it is the one party every holder of a delegation addresses, and the delegation is meaningful only at the consumer that recorded it, so a caller has to be able to recognise it. A pairwise identifier for it would leave a holder unable to tell which consumer a key is enrolled at.

The consumer learns which caller revoked which key; that is the audit record. `notFound` is deliberately uniform, so a caller cannot learn whether a key it does not own is enrolled for somebody else.

### Retention

The tombstone and the audit row are durable. A caller need not keep the response beyond confirming the revocation took effect.

### Consent/purpose

The purpose is to withdraw a device's ability to sign in an identity's name. It is available to the identity itself and to the consumer's highest administrators for incident response, and to no one else.
