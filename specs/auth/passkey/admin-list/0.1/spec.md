---
slug: auth/passkey/admin-list
version: "0.1"
title: Auth — Passkey Admin List
summary: An administrator lists one member's passkeys of one purpose — typically their step-up passkeys — as non-secret metadata, so they can tell which to revoke. Never key material.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - auth
  - passkey
  - webauthn
  - list
  - step-up
  - administrator
  - credential-management
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: Auth service
    requirement: REQUIRED
    member: recipient
  - role: Subject
    requirement: REQUIRED
proofRequirement:
  requirement: REQUIRED
  rationale: The answer is an inventory of what can authenticate as somebody other than the asker — how many authenticators they hold, which have gone unused, which to revoke to strip their step-up gesture. The consumer authorises the administrator over the subject, and the proof is what makes the administrator the issuer rather than whoever holds a bearer token.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: The task is a read with no effect to replay, but an issue time dates the request for a consumer that audits administrative reads.
sideEffects:
  level: none
  rationale: "Reads the credential records the auth service already keeps. Nothing is bound, revoked, counted or touched — not even a credential's last-used time."
subjectPath: /subject
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "Returns identifiers, labels, instants and signature counters of the subject's credentials. No private key, no public key, no attestation, no ceremony state."
retention:
  class: transient
  rationale: "Nothing about the request is kept beyond the exchange, unless the consumer audits administrative reads; see Security & Privacy."
errorCodes:
  - code: auth/passkey/admin-list:notAdministrator
    meaning: "The producer holds no administrator standing at the consumer. Returned before `payload.subject` is looked at, so it reveals nothing about the subject."
    retryable: false
  - code: auth/passkey/admin-list:subjectNotMember
    meaning: "The subject is within the administrator's authority and known to the consumer, but is not a current member — they have left, been removed, or not yet been admitted. Their credentials answer nothing, so none are listed."
    retryable: false
  - code: auth/passkey/admin-list:subjectUnknown
    meaning: "No subject by that identifier is within the administrator's authority. Returned identically for a subject the consumer has never heard of and for one outside the administrator's authority, so the code cannot be used to probe who is a member elsewhere."
    retryable: false
  - code: auth/passkey/admin-list:purposeNotSupported
    meaning: "The consumer does not disclose credentials of this purpose to administrators — for example it issues no step-up credentials, or keeps session credentials to their owner's own auth/passkey/list."
    retryable: false
related:
  - auth/passkey/list
  - auth/passkey/revoke/start
  - auth/passkey/revoke/finish
  - auth/passkey/enroll/invite
  - auth/passkey/enroll/redeem/finish
---

## Abstract

The **Auth — Passkey Admin List** Trust Task lets an administrator see one member's passkeys of one purpose. Its main use is the **step-up credentials** of [`auth/passkey/enroll/invite/0.2`](../../enroll/invite/0.2/spec.md) (`purpose: stepUp`): an administrator invites a member to enrol one, and may revoke it for them with [`auth/passkey/revoke/start/0.2`](../../revoke/start/0.2/spec.md), but until now had no published way to see which credentials the member holds — and so which `credentialId` to revoke.

[`auth/passkey/list/0.1`](../../list/0.1/spec.md) does not cover this, on purpose: it lists the **signer's own** credentials and rejects a subject in the payload, so a filter parameter and an authorisation check can never disagree. This task is the other half: the subject is named, and the consumer authorises the administrator over that subject from its own state before it reads anything.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the administrator) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/auth/passkey/admin-list/0.1`, with itself as `issuer` and the auth service as `recipient`, carrying a verified `proof`.
2. Populate `payload.subject` with the member whose credentials it wants, and `payload.purpose` with the purpose to list.

A producer listing its **own** credentials **SHOULD** use [`auth/passkey/list`](../../list/0.1/spec.md) instead.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authorisation evidence is **the producer's verified proof** and **the producer's standing, in the consumer's own state, as an administrator with authority over the subject**. Nothing in the payload grants anything.

A conforming **consumer** **MUST**, in this order:

1. Validate the document and verify the `proof`, and identify the producer's VID from it. An unsigned document is refused with `proofRequired`.
2. Establish from its own state that the producer is an administrator. Refuse with `notAdministrator` otherwise, **before** looking the subject up.
3. Establish that the administrator's authority extends to `payload.subject` — for example that the subject lies within the contexts or communities the administrator administers. Refuse with `subjectUnknown` otherwise, identically to a subject it has never heard of.
4. Refuse a `payload.purpose` it does not disclose to administrators with `purposeNotSupported`.
5. Refuse a subject it knows, within the administrator's authority, who is not a current member with `subjectNotMember`.
6. Return `{ subject, purpose, credentials }`, where `credentials` holds every credential of that purpose bound to the subject and to **no one else**, each as a `ListedCredential`, sorted by `registeredAt` descending. Return `credentials: []` — not an error — when the subject holds none.

A conforming consumer **MUST NOT**:

- include private key material, the credential public key, attestation statements, AAGUIDs, user handles, or any ceremony state (a challenge, an `enrollmentId`, a `revocationId`) in the response — `ListedCredential` has no member for any of them and `additionalProperties` is `false`;
- change anything on this task: not a credential, not a counter, not a last-used time, not an invite;
- list credentials of a purpose other than the one asked for.

Step 3 comes before steps 4 and 5 so that neither `purposeNotSupported` nor `subjectNotMember` is ever an answer about a subject outside the administrator's authority.

## Definitions

* **Administrator.** The party asking; identified by `issuer` and confirmed by the `proof`. Who counts, and over whom, is the consumer's policy, taken from its own state.
* **Subject.** The member whose credentials are listed: `payload.subject`.
* **Purpose.** `session` or `stepUp`, as fixed when the credential was bound ([`enroll/invite` 0.2](../../enroll/invite/0.2/spec.md)).
* **ListedCredential.** The administrator's view of one credential: `credentialId`, `deviceLabel`, `registeredAt`, `lastUsedAt` and `signCount`, and nothing else.

## Payload

`payload.subject` — REQUIRED, the member's VID.

`payload.purpose` — REQUIRED, `session` or `stepUp`. There is no default.

`payload.ext` — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

## Examples

### A community administrator lists a member's step-up passkeys

```json
{
  "id": "urn:uuid:9e4b2c1a-7d3f-4a6e-8b05-1c2d3e4f5a61",
  "type": "https://trusttasks.org/spec/auth/passkey/admin-list/0.1",
  "issuer": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": {
    "subject": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
    "purpose": "stepUp"
  },
  "proof": { "…": "…" }
}
```

## Response

A success *response* document carries `type: https://trusttasks.org/spec/auth/passkey/admin-list/0.1#response`.

### Two step-up passkeys, one never used

```json
{
  "id": "urn:uuid:9e4b2c1a-7d3f-4a6e-8b05-1c2d3e4f5a62",
  "type": "https://trusttasks.org/spec/auth/passkey/admin-list/0.1#response",
  "threadId": "urn:uuid:9e4b2c1a-7d3f-4a6e-8b05-1c2d3e4f5a61",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": {
    "subject": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
    "purpose": "stepUp",
    "credentials": [
      {
        "credentialId": "c3RlcHVwLWNyZWQtY2Fyb2wtMg",
        "deviceLabel": "Carol's spare key",
        "registeredAt": "2026-09-26T16:20:00Z",
        "signCount": 0
      },
      {
        "credentialId": "c3RlcHVwLWNyZWQtY2Fyb2w",
        "deviceLabel": "Carol's laptop",
        "registeredAt": "2026-09-25T10:04:00Z",
        "lastUsedAt": "2026-09-26T21:47:12Z",
        "signCount": 14
      }
    ]
  }
}
```

### A member holding none

```json
{
  "id": "urn:uuid:9e4b2c1a-7d3f-4a6e-8b05-1c2d3e4f5a63",
  "type": "https://trusttasks.org/spec/auth/passkey/admin-list/0.1#response",
  "threadId": "urn:uuid:9e4b2c1a-7d3f-4a6e-8b05-1c2d3e4f5a61",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": {
    "subject": "did:webvh:QmCarolScid3:acme-vtc.example:carol",
    "purpose": "stepUp",
    "credentials": []
  }
}
```

## Security & Privacy

**Only administrators with authority over the subject.** The subject in the payload is a request, never a grant. Being an administrator somewhere is not enough: a consumer whose administrators are scoped (to contexts, namespaces, communities) **MUST** apply that scope in step 3. Without it, any administrator could enumerate every member's authenticator estate.

**No oracle.** `notAdministrator` is decided before the subject is looked at. A subject outside the administrator's authority is `subjectUnknown`, exactly like one that does not exist, and `subjectNotMember` and `purposeNotSupported` are only ever answers about a subject inside it. A non-administrator therefore learns nothing about anyone, and a scoped administrator learns nothing outside their scope.

**Never key material.** The response is what an administrator needs to choose a credential to revoke — an identifier, a label, when it was bound, when it was last used — and to notice a cloned authenticator: the **signature counter**. [`auth/passkey/list`](../../list/0.1/spec.md) withholds the counter from the subject because it is the consumer's own cloning signal; an administrator is the consumer's operator and the person who acts on that signal, so it is disclosed here. A counter that has gone backwards, or a `lastUsedAt` the member does not recognise, is a reason to revoke. Nothing else about the credential is disclosed, and the schema forbids adding it outside `ext`.

**No side effects.** Listing does not touch `lastUsedAt` or the counter, so an administrator reading the inventory cannot mask the use it is looking for.

**Confidentiality.** The response is privacy-sensitive. Consumers **MUST** require transport-level confidentiality, or a confidential transport binding.

**Free text.** `deviceLabel` is untrusted free text bounded at 256 characters, written at enrolment by the member or suggested by the inviting administrator. A surface **MUST NOT** present it as an attested property.

### Data carried

The request carries a subject VID and a purpose. The response carries, per credential, an identifier, an optional label, one or two instants and an optional counter.

### Correlation

The request links the administrator to the subject. The response reveals how many credentials of the purpose the subject holds and when they used them — to an administrator who already has authority over the subject.

### Retention

The consumer keeps nothing of the request beyond the exchange. The `auth/passkey` family records audit events for issuance, redemption and revocation and does not require them for reads; [`auth/passkey/list`](../../list/0.1/spec.md) records none. A consumer that audits administrative reads **SHOULD** include this task, naming the administrator, the subject and the purpose — never a credential id — because it is one party reading another's authenticator inventory.

### Consent/purpose

The listing exists so an administrator can manage the subject's credentials — to confirm an enrolment completed, or to choose which one to revoke. It **MUST NOT** be used to decide anything about the subject beyond that.

The optional `ext` extension is part of the producer's signed surface.
