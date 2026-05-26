---
slug: auth/passkey/enroll/invite
version: "0.1"
title: Auth — Passkey Enroll (invite)
summary: An administrator issues a single-use invite URL that an unenrolled subject can redeem to bind their first passkey, bridging the cold-start gap where the subject has no existing authentication factor.
status: draft
targetFrameworkVersion: "0.1"
category: authentication
keywords:
  - auth
  - passkey
  - webauthn
  - enrollment
  - invite
  - bootstrap
  - admin
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
  - role: Auth service
    requirement: REQUIRED
  - role: Invitee
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED
  rationale: An invite assigns a role + scopes to a VID the auth service has never seen authenticate. The administrator's signed proof is the entire trust chain — without it, a token-stealing attacker could mint admin-tier invites pointing at attacker-controlled VIDs.
errorCodes:
  - code: auth/passkey/enroll/invite:subject_already_enrolled
    meaning: The invitee VID already has a passkey credential on file. Use auth/passkey/enroll/start (with the existing session) instead.
    retryable: false
  - code: auth/passkey/enroll/invite:role_not_permitted
    meaning: The administrator's authority does not allow assigning the requested role.
    retryable: false
related:
  - auth/passkey/enroll/start
  - auth/passkey/enroll/finish
  - auth/passkey/login/start
  - acl/grant
---

## Abstract

The **Auth — Passkey Enroll (invite)** Trust Task closes the *cold-start* gap in passkey-based auth: a brand-new subject has no existing factor with which to start the standard [`auth/passkey/enroll/start/0.1`](../start/0.1/spec.md) ceremony, because that flow assumes a pre-authenticated session.

An administrator emits this task to ask the auth service to mint a single-use invite URL. The administrator shares the URL with the invitee out-of-band (email, secure messaging, in-person QR scan). The invitee opens the URL in a WebAuthn-capable browser; the auth service drives a standard [`auth/passkey/enroll/start`](../start/0.1/spec.md) + [`auth/passkey/enroll/finish`](../finish/0.1/spec.md) ceremony scoped to the invite, and on success binds the credential to the subject + applies the role and scopes the invite encoded.

Logically: a *passkey* is to the invite as a *grant* is to an `acl/grant` — the invite asserts the binding, the redemption demonstrates control.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/passkey/enroll/invite/0.1`, with itself as `issuer` and the auth service as `recipient`.
2. Populate `payload.subject` with the VID being invited.
3. Include a verified `proof`.
4. **MAY** specify `payload.role` and `payload.scopes`; consumer policy may default these.

A conforming **consumer** (the auth service) **MUST**:

1. Validate the document and verify the `proof`.
2. Authorize the administrator: their role MUST permit issuing invites for the requested role/scopes. Refuse with `role_not_permitted` otherwise.
3. Verify the invitee VID is not already enrolled. Refuse with `subject_already_enrolled`.
4. Generate a single-use `token` with ≥128 bits of entropy and a `url` containing it (typical: `${auth_base_url}/enroll?token=…`).
5. Bind the token server-side to: the invitee VID, the role/scopes, the administrator's VID (for audit), an expiry derived from `payload.ttl` (default 1 h if unspecified).
6. Return a `#response` document carrying `{ invite, subject, expiresAt }`.

On redemption (a separate flow that the consumer drives once the invitee opens the URL), the consumer MUST:

7. Look up the token; expire if past `expiresAt`.
8. Drive the standard `auth/passkey/enroll/{start,finish}` ceremony — the token replaces the producer-side proof requirement on those tasks for the invite-scoped variant.
9. On enrollment success: bind the credential to `subject`, attach the invite's role/scopes, mark the token consumed.
10. On enrollment failure: leave the token consumable (the invitee may retry within `expiresAt`). Token consumption MUST be atomic with credential persistence.

## Definitions

* **Administrator.** The party issuing the invite; identified by `issuer`. The framework deliberately doesn't constrain who counts as administrator — that's a consumer-policy concern.
* **Invitee.** The party receiving the invite. The framework verifies they can complete a WebAuthn ceremony; it does NOT verify they are the legitimate human behind the VID. That trust flows from the administrator.
* **Invite.** The `{ token, url }` artifact the administrator shares.

## Payload

`payload.subject` (REQUIRED) — invitee VID.

`payload.role`, `payload.scopes`, `payload.deviceLabel`, `payload.ttl` — optional invite metadata.

`payload.ext` — extension slot.

## Examples

### Admin invites a new user

```json
{
  "id": "invite-1234-5678-90ab-cdef12345678",
  "type": "https://trusttasks.org/spec/auth/passkey/enroll/invite/0.1",
  "issuer": "did:web:admin.example",
  "recipient": "did:web:auth.example",
  "issuedAt": "2026-05-23T17:00:00Z",
  "payload": {
    "subject": "did:web:bob.example",
    "role": "member",
    "deviceLabel": "Bob's first device",
    "ttl": 86400
  },
  "proof": { "…": "…" }
}
```

## Response

### Issued invite

```json
{
  "id": "invite-resp-2345-6789-01bc-def234567890",
  "type": "https://trusttasks.org/spec/auth/passkey/enroll/invite/0.1#response",
  "threadId": "invite-1234-5678-90ab-cdef12345678",
  "issuer": "did:web:auth.example",
  "recipient": "did:web:admin.example",
  "issuedAt": "2026-05-23T17:00:01Z",
  "payload": {
    "subject": "did:web:bob.example",
    "expiresAt": "2026-05-24T17:00:01Z",
    "invite": {
      "token": "inv_8f2c1d4e9a7b30568f2c1d4e9a7b3056",
      "url": "https://auth.example/enroll?token=inv_8f2c1d4e9a7b30568f2c1d4e9a7b3056"
    }
  }
}
```

## Security & Privacy

**Cold-start trust.** The whole flow assumes the administrator's authority is the trust root for the invitee's VID binding. Misuse — an attacker compromising an admin token — lets the attacker bind credentials to arbitrary VIDs. The DID-signed `proof` is the firewall: a stolen bearer token is insufficient.

**Token strength.** ≥128 bits is the floor; ≥192 bits RECOMMENDED. The token IS the entire authentication factor for the redemption ceremony — it must be unguessable.

**Out-of-band channel.** The administrator delivers the URL via a channel the consumer doesn't see. Consumers MAY surface a "send to email" convenience in the response (via ext), but the framework treats delivery as the administrator's concern.

**Replay.** Tokens are single-use. Consumers MUST atomically mark consumed-on-success.

**Audit.** Both the issuance and the redemption MUST log the administrator + invitee VIDs and the timestamps; a future incident review needs to reconstruct who admitted whom.

The optional `ext` extension is part of the producer's signed surface.
