---
slug: auth/passkey/revoke/start
version: "0.1"
title: Auth — Passkey Revoke (start)
summary: A subject asks the auth service to begin removing one of their passkeys; the response is a fresh user-verification challenge that must be satisfied before anything is removed.
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
  rationale: Revocation removes an authentication capability permanently, and an attacker who revokes a subject's authenticators locks them out of their own account. The proof identifies the credential's owner; the user-verification ceremony this task begins then establishes that the owner is present right now, rather than that a token they once held is being replayed.
sideEffects:
  level: none
  rationale: "Begins a user-verification ceremony and returns options. The credential is not removed until the matching finish."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: auth/passkey/revoke/start:credential_not_found
    meaning: No credential with this id is bound to the subject. Consumers MUST return this for a credential belonging to a different subject as well, so the code cannot be used to probe whether an id exists elsewhere.
    retryable: false
  - code: auth/passkey/revoke/start:last_credential
    meaning: This is the subject's only remaining passkey and the consumer refuses to leave them with none. `details.remaining` MAY carry the count.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        remaining: { type: integer, minimum: 0 }
  - code: auth/passkey/revoke/start:reauth_unavailable
    meaning: The consumer requires user verification to revoke but cannot mount a ceremony — for example every enrolled authenticator is itself unusable. Recovery is out of band.
    retryable: false
related:
  - auth/passkey/revoke/finish
  - auth/passkey/list
  - auth/passkey/enroll/start
  - auth/step-up/approve-request
---

## Abstract

The **Auth — Passkey Revoke (start)** Trust Task is the first leg of removing a passkey. The subject names a credential from [`auth/passkey/list/0.1`](../../../list/0.1/spec.md); the auth service returns `PublicKeyCredentialRequestOptions` for a **fresh user-verification ceremony**. Nothing is removed until [`auth/passkey/revoke/finish/0.1`](../../finish/0.1/spec.md) presents the resulting assertion.

The two-leg shape mirrors [`auth/passkey/enroll/{start,finish}`](../../../enroll/start/0.1/spec.md), and exists for the same reason in reverse. Enrollment splits so the authenticator can generate a credential against a server challenge; revocation splits so the server can demand proof of presence *before* destroying one.

### Why user verification rather than a delegated confirmation

A `proof` establishes that the request came from the subject's key. It does not establish that a human is at the other end of it — a key held by malware satisfies it perfectly. Revocation is exactly where that gap matters: the attacker's goal is not to read anything but to remove the subject's ability to come back.

The [`confirm/request`](../../../../../confirm/request/0.1/spec.md) pair addresses the same gap by asking a *separate* approval agent, and returns its answer out of band on the approver's own transport. That is the right shape when the approver is a different party or a different device. It is the wrong shape here, because the party being asked is the subject who is already in the middle of a synchronous ceremony, and because it makes revocation depend on a second transport being reachable at the moment somebody is trying to eject a stolen authenticator.

So this pair keeps the confirmation **in-band and synchronous**: the same WebAuthn stack that authenticates is reused to prove presence. Consumers whose approver genuinely is a separate party **SHOULD** use `confirm/*` and carry the resulting reference under `ext`.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/passkey/revoke/start/0.1`, with itself as `issuer` and the auth service as `recipient`.
2. Populate `payload.credentialId` with an id returned by `auth/passkey/list`.
3. Include a verified `proof`.

A conforming **consumer** **MUST**:

1. Verify the `proof` and identify the producer's VID.
2. Resolve `credentialId` **within that subject's credentials only**. A credential bound to anybody else **MUST** yield `credential_not_found`, identically to an id that exists nowhere.
3. Refuse with `last_credential` if removing it would leave the subject with no passkey — **before** issuing any challenge, so the subject is not walked through a ceremony that was always going to fail.
4. Generate a fresh `revocationId` and bind it server-side to: the producer's VID, the target `credentialId`, the challenge in `uvOptions.challenge`, and an expiry (RECOMMENDED 5 minutes).
5. Return `uvOptions` whose `allowCredentials` covers the subject's enrolled credentials and whose `userVerification` is `"required"`.

A conforming consumer **MUST NOT** remove the credential on this leg, and **MUST NOT** treat a start that is never finished as consent to remove anything.

### The last-credential refusal

A consumer that permits a subject to revoke their final passkey converts one compromised session into permanent account loss. The refusal is therefore **normative**, not a deployment preference.

It has a real cost: a subject whose only authenticator is genuinely lost cannot use this task to clean up, because they cannot satisfy the user-verification ceremony either. That case is account recovery, and it is deliberately out of scope — a task that could remove the last credential without user verification would be exactly the lockout primitive this refusal exists to prevent. Consumers **SHOULD** direct such subjects to an out-of-band recovery path, and **SHOULD** encourage a second authenticator at enrollment so the situation does not arise.

### Serialization

The check in step 3 and the removal in the matching finish **MUST** be serialized per subject. Two concurrent revocations that each observe two remaining credentials will each conclude they are not the last, and both will succeed — leaving zero. A consumer that performs the count and the removal without a shared lock has the guard in name only.

## Definitions

* **Subject.** The credential's owner; identified by `issuer` and confirmed by the `proof`.
* **Auth service.** The WebAuthn relying party; identified by `recipient`.
* **revocationId.** Opaque correlation handle between start and finish; produced by the consumer, bound to the target credential.
* **User verification.** A WebAuthn ceremony asserting that a human authorized this operation on an enrolled authenticator now — the `UV` flag in the authenticator data.

## Payload

`payload.credentialId` — REQUIRED, the credential to remove.

`payload.ext` — extension slot per [SPEC.md §4.5.1](../../../../../../SPEC.md#451-the-ext-extension-member).

## Examples

### Subject begins revoking a lost YubiKey

```json
{
  "id": "pk-rev-1111-2222-3333-444444444444",
  "type": "https://trusttasks.org/spec/auth/passkey/revoke/start/0.1",
  "issuer": "did:web:alice.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-07-27T10:00:00Z",
  "payload": {
    "credentialId": "z9x8c7v6b5n4m3k2j1h0"
  },
  "proof": { "…": "…" }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/auth/passkey/revoke/start/0.1#response`. Payload: `{ revocationId, uvOptions }`.

### Successful start

```json
{
  "id": "pk-rev-resp-5555-6666-7777-888888888888",
  "type": "https://trusttasks.org/spec/auth/passkey/revoke/start/0.1#response",
  "threadId": "pk-rev-1111-2222-3333-444444444444",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-07-27T10:00:01Z",
  "payload": {
    "revocationId": "rev_9f8e7d6c5b4a3210",
    "uvOptions": {
      "challenge": "cmV2b2tlLWNoYWxsZW5nZQ",
      "rpId": "auth.example",
      "timeout": 60000,
      "userVerification": "required",
      "allowCredentials": [
        { "type": "public-key", "id": "q1w2e3r4t5y6u7i8o9p0", "transports": ["internal", "hybrid"] }
      ]
    }
  }
}
```

Note that `allowCredentials` here offers the MacBook — not the YubiKey being revoked. A consumer **MAY** exclude the target credential so a subject holding only the authenticator they are trying to remove cannot authorize its own removal; a consumer **MAY** equally include it, since a subject who still holds a working authenticator they no longer trust is a legitimate case. Either is conformant, but the choice **MUST** be consistent, because a subject who is offered the target on one attempt and not the next cannot tell a policy from a fault.

### Refusing the last credential

```json
{
  "id": "pk-rev-resp-9999-aaaa-bbbb-cccccccccccc",
  "type": "https://trusttasks.org/spec/trust-task-error/0.1",
  "threadId": "pk-rev-1111-2222-3333-444444444444",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:alice.example",
  "issuedAt": "2026-07-27T10:00:01Z",
  "payload": {
    "code": "auth/passkey/revoke/start:last_credential",
    "message": "Refusing to remove your only passkey — you would not be able to sign in again. Enroll another authenticator first.",
    "details": { "remaining": 1 }
  }
}
```

## Security & Privacy

**Why the target is bound at start.** The finish carries only the `revocationId` and the assertion. If it carried the target too, a consumer that trusted the finish's copy would let an attacker who intercepted a legitimate ceremony redirect the removal to a different credential — the user verifies one thing and a different one is destroyed. Binding the target to the handle server-side means the user-verification ceremony authorizes precisely what the subject saw.

**Enumeration.** `credential_not_found` covers both "no such id" and "that id belongs to someone else". Distinguishing them would turn this task into an oracle for credential ownership.

**Ceremony expiry.** A `revocationId` **MUST** expire (RECOMMENDED 5 minutes) and **MUST** be single-use. An unexpiring handle is a standing authorization to destroy a credential, redeemable by whoever obtains it.

**Non-revocation is also a risk.** The controls above make revocation hard to perform maliciously. Consumers **MUST NOT** extend that reasoning to making it slow or obscure: the common case is a subject who has genuinely lost a device and needs it gone now. A revocation path buried behind support tickets leaves live credentials in the field, which is the larger risk in practice.

The optional `ext` extension is part of the producer's signed surface.
