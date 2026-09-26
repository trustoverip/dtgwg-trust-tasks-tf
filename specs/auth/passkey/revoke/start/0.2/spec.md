---
slug: auth/passkey/revoke/start
version: "0.2"
title: Auth — Passkey Revoke (start)
summary: A subject — or an administrator, for them — asks the auth service to begin removing a passkey of either purpose; the response is a fresh user-verification challenge for the person acting, satisfied before anything is removed.
status: draft
targetFrameworkVersion: "0.6.0"
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
  - role: Producer
    requirement: REQUIRED
    member: issuer
  - role: Auth service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "Revocation removes an authentication capability permanently, and an attacker who revokes a subject's authenticators locks them out, or strips the gesture a step-up relies on. The proof identifies who is acting — the credential's owner, or the administrator acting for them; the user-verification ceremony this task begins then establishes that that person is present right now."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "Placing the request in time bounds how long a captured start can be replayed to begin another ceremony, independent of the revocationId's own expiry."
sideEffects:
  level: none
  rationale: "Begins a user-verification ceremony and returns options. The credential is not removed until the matching finish."
exposure:
  discloses: none
  actsAsSubject: false
  rationale: "The response carries only the ceremony: a handle and request options over the producer's own credentials."
errorCodes:
  - code: auth/passkey/revoke/start:credentialNotFound
    meaning: "No credential with this id is bound to the subject (the producer, or `payload.subject`). Consumers MUST return this for a credential belonging to a different subject as well, so the code cannot be used to probe whether an id exists elsewhere."
    retryable: false
  - code: auth/passkey/revoke/start:lastCredential
    meaning: "This is the subject's only remaining session passkey and the consumer refuses to leave them with none. Never returned for a step-up credential. `details.remaining` MAY carry the count."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        remaining: { type: integer, minimum: 0 }
  - code: auth/passkey/revoke/start:reauthUnavailable
    meaning: "The consumer cannot mount a user-verification ceremony for the producer — for example the producer holds no usable credential of a purpose that may authorise this revocation. Recovery is an administrator revoking for them, or out of band."
    retryable: false
  - code: auth/passkey/revoke/start:notAuthorized
    meaning: "`payload.subject` names someone other than the producer, and the producer's authority does not extend to revoking that subject's credentials. Returned before any credential lookup, so it reveals nothing about the subject's credentials."
    retryable: false
related:
  - auth/passkey/revoke/finish
  - auth/passkey/list
  - auth/passkey/enroll/invite
  - auth/passkey/enroll/redeem/finish
  - auth/step-up/approve-request
---

## Abstract

The **Auth — Passkey Revoke (start)** Trust Task is the first leg of removing a passkey. The producer names a credential; the auth service returns `PublicKeyCredentialRequestOptions` for a **fresh user-verification ceremony by the producer**. Nothing is removed until [`auth/passkey/revoke/finish/0.2`](../../finish/0.2/spec.md) presents the resulting assertion.

0.2 exists for the **step-up credentials** of [`auth/passkey/enroll/invite/0.2`](../../../enroll/invite/0.2/spec.md) (`purpose: stepUp`): passkeys that never open a session and are accepted only as the user-verification gesture of an operation-bound step-up. Their holders commonly have no session and only one such credential, so 0.1's shape — the owner revokes, over their own credentials, never their last — would leave a lost step-up authenticator irrevocable. 0.2 adds an **administrator revoking for the subject**, and lifts the last-credential refusal for step-up credentials.

## Changes from 0.1

- **`payload.subject`** (optional): whose credential it is, when the producer is not its owner. Absent, the producer's own, as in 0.1. Present and not the producer, the consumer **MUST** authorise the producer as an administrator over that subject (`notAuthorized` otherwise). 0.1 rejected `subject` outright; that rule is kept for the owner's case by treating an absent `subject` as the producer and nothing else.
- **The ceremony is always the producer's.** `uvOptions` covers credentials of the **producer**, whoever owns the target: the person acting proves they are present.
- **Purpose-aware last-credential refusal.** `lastCredential` protects a subject's last **session** credential only. A step-up credential may be revoked down to none: losing one costs the subject a gesture, not their account.
- `reauthUnavailable` now also covers a producer with no credential that may authorise this revocation; `notAuthorized` is new.
- The framework target moves to 0.6.0. Released as a `MINOR` increment under the `draft` allowance of [SPEC §5.2](/SPEC.md#52-compatibility-rules).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/passkey/revoke/start/0.2`, with itself as `issuer` and the auth service as `recipient`, carrying a verified `proof`.
2. Populate `payload.credentialId`, and `payload.subject` only when revoking another subject's credential.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authorisation evidence is **the producer's verified proof**, plus — when `payload.subject` names someone else — **the producer's standing as an administrator over that subject in the consumer's own state**, and, at finish, the producer's user-verified assertion.

A conforming **consumer** **MUST**, in this order:

1. Verify the `proof` and identify the producer's VID. The target subject is `payload.subject` when present, the producer otherwise.
2. When the target subject is not the producer, authorise the producer from the consumer's own state — never from the document — as an administrator who may revoke that subject's credentials. Refuse with `notAuthorized` otherwise, **before** looking the credential up.
3. Resolve `credentialId` **within the target subject's credentials only**, of both purposes. A credential bound to anybody else **MUST** yield `credentialNotFound`, identically to an id that exists nowhere.
4. For a **session** credential, refuse with `lastCredential` if removing it would leave the subject with no session credential — before issuing any challenge. Never refuse a step-up credential on that ground.
5. Choose the credentials the producer may verify with: for an administrator acting for another subject, the producer's **session** credentials; for the owner revoking a session credential, their session credentials; for the owner revoking a step-up credential, their **step-up** credentials, the target included. None usable → `reauthUnavailable`.
6. Generate a fresh `revocationId` and bind it server-side to: the producer's VID, the target subject, the target `credentialId` and its purpose, the challenge in `uvOptions.challenge`, the credentials chosen in step 5, and an expiry (RECOMMENDED 5 minutes).
7. Return `uvOptions` whose `allowCredentials` covers exactly the credentials chosen in step 5 and whose `userVerification` is `"required"`.

A conforming consumer **MUST NOT** remove the credential on this leg, and **MUST NOT** treat a start that is never finished as consent to remove anything.

### Why an owner's step-up credential verifies its own revocation

A step-up credential is accepted only as a gesture bound to one operation of its own subject ([`enroll/invite` 0.2](../../../enroll/invite/0.2/spec.md#a-stepup-credential), rule 2), and revoking it is such an operation. Including the target lets a subject who still holds the authenticator, but no longer trusts it, remove it. A subject who has lost it has nothing to verify with, and that is the administrator's case in step 2 — which is why 0.2 adds it.

### The last-credential refusal

Unchanged from [0.1](../0.1/spec.md#the-last-credential-refusal) for session credentials, including its serialization requirement: the count in step 4 and the removal at finish **MUST** be serialized per subject.

## Definitions

* **Producer.** The party acting; identified by `issuer` and confirmed by the `proof`. The credential's owner, or an administrator acting for them.
* **Target subject.** Whose credential is revoked: `payload.subject`, or the producer.
* **Purpose.** `session` or `stepUp`, as fixed when the credential was bound ([`enroll/invite` 0.2](../../../enroll/invite/0.2/spec.md)).

## Payload

`payload.credentialId` — REQUIRED, the credential to remove.

`payload.subject` — OPTIONAL, the target subject when it is not the producer.

`payload.ext` — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

## Examples

### A community administrator begins revoking a member's lost step-up passkey

```json
{
  "id": "urn:uuid:3b0c9a4e-8f6d-4c2b-a1e7-5d9f0b2c6e11",
  "type": "https://trusttasks.org/spec/auth/passkey/revoke/start/0.2",
  "issuer": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-25T12:00:00Z",
  "payload": {
    "credentialId": "c3RlcHVwLWNyZWQtY2Fyb2w",
    "subject": "did:webvh:QmCarolScid3:acme-vtc.example:carol"
  },
  "proof": { "…": "…" }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/auth/passkey/revoke/start/0.2#response`. Payload: `{ revocationId, uvOptions }`.

```json
{
  "id": "urn:uuid:3b0c9a4e-8f6d-4c2b-a1e7-5d9f0b2c6e12",
  "type": "https://trusttasks.org/spec/auth/passkey/revoke/start/0.2#response",
  "threadId": "urn:uuid:3b0c9a4e-8f6d-4c2b-a1e7-5d9f0b2c6e11",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "issuedAt": "2026-09-25T12:00:01Z",
  "payload": {
    "revocationId": "rev_4c1d7e9a0b3f5268",
    "uvOptions": {
      "challenge": "cmV2b2tlLWZvci1jYXJvbA",
      "rpId": "acme-vtc.example",
      "timeout": 60000,
      "userVerification": "required",
      "allowCredentials": [
        { "type": "public-key", "id": "ZGFuYS1zZXNzaW9uLWtleQ", "transports": ["internal", "hybrid"] }
      ]
    }
  }
}
```

`allowCredentials` offers Dana's own session passkey: she is the one acting.

## Security & Privacy

**Administrator authority comes from the consumer's state.** A `subject` in the payload is a request, never a grant. Without step 2, any producer could name any subject and — having a working authenticator of their own — strip another subject's credentials.

**No oracle.** `notAuthorized` is returned before the credential is looked up, and `credentialNotFound` covers both "no such id" and "someone else's id", so neither code reveals what credentials another subject holds.

**The person acting verifies.** An administrator revoking for a member proves *their own* presence. The member's absence is the usual reason for the administrator acting — a lost authenticator — so their verification cannot be required.

**Notice.** A consumer **SHOULD** tell the subject, over a channel it already has with them, when an administrator revoked one of their credentials.

**Ceremony expiry**, **why the target is bound at start** and **non-revocation is also a risk**: as in [0.1](../0.1/spec.md#security--privacy).

### Data carried

The request carries a credential id and, for an administrator, the target subject. The response carries a challenge and the ids of the producer's own credentials.

### Correlation

An administrator's revocation links them to the subject in the auth service's audit log, which is its purpose. The response reveals nothing about the target subject's credentials.

### Retention

The revocation handle lives until the finish or its expiry.

### Consent/purpose

The handle authorises removing the one credential named at start, and nothing else.

The optional `ext` extension is part of the producer's signed surface.
