---
slug: auth/passkey/list
version: "0.1"
title: Auth — Passkey List
summary: A subject lists every passkey the auth service holds for them, so they can tell their authenticators apart before revoking one.
status: draft
targetFrameworkVersion: "0.2"
category: authentication
keywords:
  - auth
  - passkey
  - webauthn
  - list
  - credential-management
  - fido2
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Subject
    requirement: REQUIRED
    member: issuer
  - role: Auth service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The credential list is an inventory of exactly what can authenticate as the subject — how many authenticators exist, which are platform-bound, and which have gone unused. That is reconnaissance for anyone deciding which credential to attack or which to revoke to lock the subject out, so enumeration is tied to the subject's own signing key rather than to a bearer token.
sideEffects:
  level: none
  rationale: "Read-only listing of the subject's registered credentials."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: auth/passkey/list:passkeys_not_supported
    meaning: This auth service does not manage passkeys, so there is no inventory to return. Distinct from an empty list, which asserts that passkeys ARE supported and this subject has none.
    retryable: false
related:
  - auth/passkey/revoke/start
  - auth/passkey/revoke/finish
  - auth/passkey/enroll/start
  - auth/sessions/list
---

## Abstract

The **Auth — Passkey List** Trust Task enumerates the passkeys bound to a subject's VID.

It is the credential-management counterpart to [`auth/sessions/list/0.1`](../../../sessions/list/0.1/spec.md). Sessions answers *"where am I signed in?"*; this answers *"what can sign me in?"* — a longer-lived and more consequential question, because revoking a session ends an episode whereas revoking a credential removes a capability permanently.

Its output is the input to revocation: [`auth/passkey/revoke/start/0.1`](../../revoke/start/0.1/spec.md) targets a `credentialId` returned here.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/passkey/list/0.1`, with itself as `issuer` and the auth service as `recipient`.
2. Provide an empty payload (only the optional `ext` slot).
3. Include a verified `proof`.

A conforming **consumer** **MUST**:

1. Validate the document and verify the `proof`.
2. Enumerate every credential bound to the VID identified by the `proof` — **never** to a subject named in the payload, which is why the payload carries no subject member.
3. Return a `#response` document carrying `{ credentials }`, each entry a `RegisteredCredential`.
4. Return `credentials: []` — not an error — when the subject has no passkeys.

A conforming consumer **MUST NOT** return any credential bound to a different subject, and **MUST NOT** include private key material, the credential public key, or the signature counter. None of those help a human choose which authenticator to revoke, and the counter in particular is a cloning-detection signal that belongs in the consumer's own telemetry.

Consumers **SHOULD** sort by `registeredAt` descending, so a credential enrolled by an attacker moments ago appears first rather than buried beneath legitimate ones.

## Definitions

* **Subject.** The party whose credentials are listed; identified by `issuer` and confirmed by the `proof`.
* **Auth service.** The WebAuthn relying party; identified by `recipient`.
* **RegisteredCredential.** The management view of one passkey; see [`_shared/0.1/webauthn.schema.json#RegisteredCredential`](../../../_shared/0.1/webauthn.schema.json).

## Payload

The request payload carries no required members. The proof is the request.

`payload.ext` (optional) — extension slot per [SPEC.md §4.5.1](../../../../../SPEC.md#451-the-ext-extension-member).

## Examples

### Subject lists their passkeys

```json
{
  "id": "pk-list-1111-2222-3333-444444444444",
  "type": "https://trusttasks.org/spec/auth/passkey/list/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-07-27T09:00:00Z",
  "payload": {},
  "proof": { "…": "…" }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/auth/passkey/list/0.1#response`.

### Two credentials, one never used

```json
{
  "id": "pk-list-resp-5555-6666-7777-888888888888",
  "type": "https://trusttasks.org/spec/auth/passkey/list/0.1#response",
  "threadId": "pk-list-1111-2222-3333-444444444444",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-07-27T09:00:01Z",
  "payload": {
    "credentials": [
      {
        "credentialId": "q1w2e3r4t5y6u7i8o9p0",
        "deviceLabel": "Alice's MacBook Pro",
        "transports": ["internal", "hybrid"],
        "registeredAt": "2026-07-20T11:04:00Z",
        "lastUsedAt": "2026-07-27T08:55:12Z"
      },
      {
        "credentialId": "z9x8c7v6b5n4m3k2j1h0",
        "deviceLabel": "YubiKey 5C (backup, in safe)",
        "transports": ["usb", "nfc"],
        "registeredAt": "2026-07-20T11:09:30Z"
      }
    ]
  }
}
```

The backup key has no `lastUsedAt`: it has never completed an assertion. For a key deliberately kept in a safe that is reassuring; for a credential the subject does not recognize, a recent one is the strongest available signal that somebody else is using it.

### No passkeys

```json
{
  "id": "pk-list-resp-9999-aaaa-bbbb-cccccccccccc",
  "type": "https://trusttasks.org/spec/auth/passkey/list/0.1#response",
  "threadId": "pk-list-1111-2222-3333-444444444444",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-07-27T09:00:01Z",
  "payload": {
    "credentials": []
  }
}
```

## Security & Privacy

**Why proof-required, and why there is no subject filter.** The list is an inventory of everything that can authenticate as the subject. Its value to an attacker is not the identifiers — those are useless without the authenticator — but the *shape*: how many credentials exist, whether any is a lone platform credential whose loss would lock the subject out, and which have gone unused long enough that revoking them would go unnoticed. Binding enumeration to the subject's signing key, and taking the subject from the `proof` rather than from the payload, removes the class of bug where a filter parameter and an authorization check disagree.

**Empty list versus unsupported.** A consumer that does not manage passkeys **MUST** return `auth/passkey/list:passkeys_not_supported` rather than an empty array. Conflating them tells a subject auditing their own security that they have no passkeys, when the truth is that this service would not know.

**What is deliberately absent.** No public key, no signature counter, no AAGUID. A subject choosing which authenticator to revoke needs a label, a date, and a transport hint; the cryptographic material serves only an attacker fingerprinting the authenticator estate. Consumers needing an attestation-grade inventory for regulatory reasons **SHOULD** carry it under `ext` rather than widening the default surface.

**Confidentiality.** The response is privacy-sensitive. Consumers **MUST** require transport-level confidentiality.

The optional `ext` extension is part of the producer's signed surface.
