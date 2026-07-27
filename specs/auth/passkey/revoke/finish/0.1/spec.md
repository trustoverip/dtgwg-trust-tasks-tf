---
slug: auth/passkey/revoke/finish
version: "0.1"
title: Auth — Passkey Revoke (finish)
summary: A subject submits the user-verification assertion that completes a passkey revocation; on success the credential is unbound permanently.
status: draft
targetFrameworkVersion: "0.2"
category: authentication
keywords:
  - auth
  - passkey
  - webauthn
  - revoke
  - credential-management
  - step-up
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
  rationale: This leg destroys an authentication capability. The proof identifies the owner and the assertion proves presence; requiring both means neither a stolen token alone nor a captured assertion alone is sufficient.
sideEffects:
  level: destructive
  rationale: "The credential is unbound. A revoked passkey cannot be restored — the authenticator must be enrolled afresh, which requires it to be physically present."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: auth/passkey/revoke/finish:revocation_not_found
    meaning: No pending revocation with this id, or it belongs to a different subject.
    retryable: false
  - code: auth/passkey/revoke/finish:revocation_expired
    meaning: The revocationId outlived its window. Start a new ceremony.
    retryable: true
  - code: auth/passkey/revoke/finish:user_verification_failed
    meaning: The assertion did not verify, did not match the challenge bound at start, or did not carry the UV flag. Deliberately one code for all three — see Security & Privacy.
    retryable: true
  - code: auth/passkey/revoke/finish:last_credential
    meaning: Re-checked at commit time and the credential is now the subject's last, because another revocation completed in between. `details.remaining` MAY carry the count.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        remaining: { type: integer, minimum: 0 }
related:
  - auth/passkey/revoke/start
  - auth/passkey/list
  - auth/passkey/enroll/finish
  - auth/revoke-session
---

## Abstract

The **Auth — Passkey Revoke (finish)** Trust Task completes the ceremony begun by [`auth/passkey/revoke/start/0.1`](../../start/0.1/spec.md). The producer submits the WebAuthn assertion obtained over the `uvOptions` from start; on success the auth service unbinds the credential that start recorded against the `revocationId`.

The payload names **no credential**. That is the security property this leg is built around: the target was fixed when the subject was shown what they were about to remove, so the assertion authorizes that specific removal and nothing else.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/passkey/revoke/finish/0.1`, with itself as `issuer` and the auth service as `recipient`.
2. Echo `payload.revocationId` verbatim from the start response.
3. Populate `payload.uvCredential` with the assertion from `navigator.credentials.get`.
4. Include a verified `proof`.

A conforming **consumer** **MUST**, in this order:

1. Verify the `proof` and identify the producer's VID.
2. Resolve `revocationId`. Unknown, expired, already-consumed, or bound to a different VID → `revocation_not_found` / `revocation_expired`.
3. Verify the assertion: signature valid against an enrolled credential of *this* subject, `clientDataJSON.challenge` equal to the challenge bound at start, origin and `rpId` as expected, and the **UV flag set** in the authenticator data. Any failure → `user_verification_failed`.
4. Re-check the last-credential guard **at commit time**, under the same per-subject serialization as start. Still the last → `last_credential`.
5. Unbind the credential recorded against the `revocationId`, consume the handle, and emit an audit record.
6. Return `{ credentialId, revokedAt, remaining }`.

A conforming consumer **MUST NOT** accept a target credential from this payload, **MUST NOT** accept an assertion whose UV flag is clear even if the signature verifies, and **MUST NOT** allow a `revocationId` to be redeemed twice.

### Why the guard is re-checked here

Start's refusal is a courtesy — it fails fast, before walking the subject through a ceremony. It is not the enforcement point, because an unbounded amount of time passes between the two legs and another ceremony may complete in the gap. The check that actually holds the invariant is this one, inside the same critical section as the removal.

A consumer that checks only at start has a race with a window as long as the ceremony timeout: two revocations start when three credentials exist, both pass, and the subject ends with one fewer than the guard promised. Checking only at finish would be *sufficient* but hostile — the subject completes a biometric prompt to be told it was never going to work. Both checks, with only the second load-bearing, is the shape that is correct and humane at once.

### Sessions established by the revoked credential

Revoking a credential does not, by itself, end sessions it authenticated. A consumer **SHOULD** also invalidate them — the usual reason for revoking is that the authenticator is in someone else's hands, and leaving its sessions live defeats the point. Consumers that do so **SHOULD** exclude the producer's current session, so a subject pruning an old key is not signed out mid-task. This is RECOMMENDED rather than REQUIRED because session lifetime is the consumer's policy; consumers that decline **SHOULD** document it, since subjects reasonably assume revocation is immediate and total.

## Definitions

* **UV flag.** The `UV` bit in WebAuthn authenticator data, asserting that the authenticator verified the user (biometric, PIN, …) during this ceremony. Distinct from `UP` (user *presence*), which a mere touch satisfies.
* **revocationId.** The single-use handle from start, bound server-side to the subject and the target credential.

## Payload

`payload.revocationId` — REQUIRED, echoed from start.

`payload.uvCredential` — REQUIRED, the assertion.

`payload.ext` — extension slot per [SPEC.md §4.5.1](../../../../../../SPEC.md#451-the-ext-extension-member).

## Examples

### Subject completes the revocation

```json
{
  "id": "pk-revf-1111-2222-3333-444444444444",
  "type": "https://trusttasks.org/spec/auth/passkey/revoke/finish/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-07-27T10:00:20Z",
  "payload": {
    "revocationId": "rev_9f8e7d6c5b4a3210",
    "uvCredential": {
      "id": "q1w2e3r4t5y6u7i8o9p0",
      "rawId": "q1w2e3r4t5y6u7i8o9p0",
      "type": "public-key",
      "response": {
        "clientDataJSON": "eyJ0eXBlIjoid2ViYXV0aG4uZ2V0IiwiY2hhbGxlbmdlIjoiY21WMmIydGxMV05vWVd4c1pXNW5aUSJ9",
        "authenticatorData": "SZYN5YgOjGh0NBcPZHZgW4_krrmihjLHmVzzuoMdl2MFAAAAAQ",
        "signature": "MEUCIQDxK8bTfnE1oJ2sLmA9Qw",
        "userHandle": "dXNyXzhmMmMxZDRlOWE3YjMwNTY"
      },
      "authenticatorAttachment": "platform"
    }
  },
  "proof": { "…": "…" }
}
```

The assertion comes from the MacBook (`q1w2…`), while the credential being revoked is the YubiKey named at start. The subject proves presence with an authenticator they still hold in order to eject one they no longer do.

## Response

A success *response* document carries `type: https://trusttasks.org/spec/auth/passkey/revoke/finish/0.1#response`.

### Successful revocation

```json
{
  "id": "pk-revf-resp-5555-6666-7777-888888888888",
  "type": "https://trusttasks.org/spec/auth/passkey/revoke/finish/0.1#response",
  "threadId": "pk-revf-1111-2222-3333-444444444444",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-07-27T10:00:21Z",
  "payload": {
    "credentialId": "z9x8c7v6b5n4m3k2j1h0",
    "revokedAt": "2026-07-27T10:00:21Z",
    "remaining": 1
  }
}
```

## Security & Privacy

**One code for every verification failure.** `user_verification_failed` covers a bad signature, a mismatched challenge, a wrong origin, and a clear UV flag alike. Separating them would tell an attacker probing a captured assertion precisely which control stopped them — which is the whole map they need. The consumer's own logs **SHOULD** record the specific cause; the wire **MUST NOT**.

**UV, not UP.** A consumer that accepts a bare user-presence touch has built a control that a stolen-and-still-plugged-in security key satisfies by itself. The UV flag is what distinguishes "somebody touched a key" from "somebody who can unlock this key authorized this".

**Single-use handles.** A `revocationId` **MUST** be consumed on first successful redemption. Otherwise a replayed finish is a standing right to re-run the removal, and — for consumers that reuse ids — to remove whatever now occupies that binding.

**Irreversibility is the point, and the risk.** `sideEffects: destructive` is not a formality: there is no undo, and restoring the credential requires the authenticator in hand. Producers **SHOULD** show the subject exactly which credential is about to go, using the `deviceLabel` from `auth/passkey/list`, before starting the ceremony rather than after it.

**Audit.** Consumers **MUST** record the revocation — subject, credential id, timestamp, and the credential that satisfied the verification. When a subject later reports being locked out, the identity of the authenticator that authorized the removal is the single most useful fact available.

The optional `ext` extension is part of the producer's signed surface.
