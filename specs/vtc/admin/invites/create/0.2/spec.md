---
slug: vtc/admin/invites/create
version: "0.2"
title: VTC Admin Invites — Create
summary: Invite an additional administrator to a Verifiable Trust Community — either a named DID, claimed by signing as it, or whoever registers a passkey, whose DID is derived from it — and receive the one-shot install URL and claim code.
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - vtc
  - admin
  - invite
  - passkey
  - wallet
  - enrolment
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Mints a credential-bearing URL that grants administrator access. The caller must be attributable.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: The invite carries secret material that admits its bearer as an administrator. Replayed, it mints a second such invite, and the material is out of the community’s hands the moment it is returned.
sideEffects:
  level: mutating
  rationale: "Persists an invite with the authority it confers. A wallet invitation for a DID with no entry also writes that DID's administrator entry; a passkey invitation writes none until it is claimed."
exposure:
  discloses: secret
  actsAsSubject: false
  rationale: Returns a claim code and install URL that are, until redeemed or expired, sufficient to become an administrator.
errorCodes:
  - code: vtc/admin/invites/create:ttlTooLong
    meaning: The requested `ttlSeconds` exceeds the 24-hour maximum.
    retryable: false
related:
  - vtc/admin/invites/list
  - vtc/admin/invites/revoke
  - vtc/install/claim/passkey/start
  - vtc/install/claim/passkey/finish
  - vtc/install/claim/start
  - vtc/install/claim/finish
---

## Abstract

The **VTC Admin Invites — Create** Trust Task invites one additional administrator. It returns a single-use `installUrl`, a `claimCode` to be delivered on a second channel, the invite's `jti` — the handle [`vtc/admin/invites/revoke`](../../revoke/0.1/spec.md) acts on — and the invitation's `kind`. There are two kinds, chosen by whether the request names a DID:

- **Wallet invitation** — `did` present. The invitation names the DID that will be the administrator. The invitee claims it by signing as that DID, from a wallet that holds it, with [`vtc/install/claim/start`](../../../../install/claim/start/0.3/spec.md) and [`finish`](../../../../install/claim/finish/0.3/spec.md) `0.3`, binding a step-up approver as they do. If the DID holds no entry, this task writes its administrator entry now; `aclEntryCreated` reports whether it did.
- **Passkey invitation** — `did` absent. The invitation names no DID. The invitee registers a passkey with [`vtc/install/claim/passkey/start`](../../../../install/claim/passkey/start/0.1/spec.md) and [`finish`](../../../../install/claim/passkey/finish/0.1/spec.md) `0.1`, and the administrator's DID is that passkey's own `did:key`. Nobody knows the DID until then, so this task writes no entry (`aclEntryCreated` is `false`); it records the authority the entry will carry, and finish writes the entry with it.

### Changes from 0.1

- `did` is optional. Absent, the invitation is a passkey invitation. In `0.1` every invitation named a DID, so an administrator could invite only someone who already held a DID in a wallet; inviting someone with nothing but a passkey needed the operator at the host.
- `did`, when present, must begin `did:`.
- The response gains a required `kind`, `wallet` or `passkey`.
- `label` is now stated to label the administrator entry the invitation writes, whichever kind it is.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **producer** supplies `did` for a wallet invitation and omits it for a passkey invitation. `ttlSeconds` is optional and **MUST NOT** exceed 86400; `label` is an operator-facing note.

A conforming **consumer**:

1. **MUST** verify that the caller may make an administrator with the authority the invitation confers (item 3): inviting an administrator is granting one, bounded by the caller's own authority like any grant.
2. **MUST** refuse a `ttlSeconds` over the 24-hour maximum with `ttlTooLong` rather than issue a longer-lived credential.
3. **MUST** confer the same authority on either kind: an administrator entry with the community's full administrative authority, delegated by the caller, labelled with `label` when one is given. For a **wallet** invitation the consumer writes it for `did` now, unless `did` already holds an administrator entry, which it leaves untouched; a `did` holding a non-administrator entry is refused rather than changed. For a **passkey** invitation it **MUST** record that authority, and the caller as its delegator, with the invitation, and write nothing to the access list.
4. **MUST** require for a passkey invitation whatever it requires before writing such an entry for a wallet invitation — for example, the caller's step-up and another administrator's consent. A passkey invitation always confers that authority, and when it is claimed there is no administrator present to authorise it: issuing the invitation is the grant.
5. **MUST** mint a single-use install token — naming `did` for a wallet invitation, and no DID for a passkey invitation — and a claim code, store only a verifier of the code, and return `installUrl` and `claimCode` **once**. They are not retrievable afterwards, and [`list`](../../list/0.2/spec.md) **MUST NOT** disclose them.
6. **MUST** refuse a claim of a wallet invitation by the passkey ceremony, and of a passkey invitation by the wallet ceremony, as those tasks require.

When a passkey invitation is claimed, [`vtc/install/claim/passkey/finish`](../../../../install/claim/passkey/finish/0.1/spec.md) `0.1` item 9 writes the derived DID's entry with exactly the authority item 3 recorded. Revoking the invitation before then, with [`revoke`](../../revoke/0.1/spec.md), withdraws that authority; nothing else does.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The evidence is the caller's proof and their own administrator entry, read at execution. An invitation of either kind grants a community administrator, so the consumer authorises it as it authorises that grant. The authorization decision is the *consumer*'s alone; per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10, verifying the `proof` establishes who asked, never that they may.

## Request

### A wallet invitation

```json
{
  "id": "urn:uuid:5b1d8c2e-3f4a-4b6c-9d7e-8f9a0b1c2d31",
  "type": "https://trusttasks.org/spec/vtc/admin/invites/create/0.2",
  "issuer": "did:webvh:QmAdminScid:alice.example",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-30T09:00:00Z",
  "payload": {
    "did": "did:webvh:QmBobScid:bob.example",
    "ttlSeconds": 3600,
    "label": "Bob — operations"
  }
}
```

### A passkey invitation

```json
{
  "id": "urn:uuid:5b1d8c2e-3f4a-4b6c-9d7e-8f9a0b1c2d32",
  "type": "https://trusttasks.org/spec/vtc/admin/invites/create/0.2",
  "issuer": "did:webvh:QmAdminScid:alice.example",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-30T09:00:00Z",
  "payload": {
    "ttlSeconds": 3600,
    "label": "Carol — finance"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. Refusals use `trust-task-error`.

### A passkey invitation is issued

```json
{
  "id": "urn:uuid:5b1d8c2e-3f4a-4b6c-9d7e-8f9a0b1c2d33",
  "type": "https://trusttasks.org/spec/vtc/admin/invites/create/0.2#response",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmAdminScid:alice.example",
  "issuedAt": "2026-10-30T09:00:01Z",
  "threadId": "urn:uuid:5b1d8c2e-3f4a-4b6c-9d7e-8f9a0b1c2d32",
  "payload": {
    "jti": "0b6f2a9e-4c1d-4e8f-9a2b-3c4d5e6f7a8b",
    "kind": "passkey",
    "installUrl": "https://acme-vtc.example/admin/install?token=eyJhbGciOiJFZERTQSJ9.eyJhdWQiOiJ2dGMtaW5zdGFsbCJ9.c2ln",
    "claimCode": "K7QF-2M9D-XR4T",
    "expiresAt": "2026-10-30T10:00:01Z",
    "aclEntryCreated": false
  }
}
```

## Security & Privacy

**The response is the credential.** Until it is redeemed or expires, whoever holds the `installUrl` and `claimCode` can become an administrator — under the named DID for a wallet invitation, and under a DID of their own making for a passkey invitation. That is why `exposure.discloses` is `secret`, why the 24-hour ceiling is a hard bound rather than a default, and why the two values go to the invitee on separate channels.

**A passkey invitation is a bearer grant.** A wallet invitation can be claimed only by whoever controls the named DID; a passkey invitation by whoever holds the URL and the code. Its authority is therefore decided, and authorised, when it is issued (items 3 and 4), and the only way to withdraw it before it is claimed is to revoke it.

That the values are returned exactly once, and never again by `list`, keeps the window of exposure to the single response rather than to every later read of the invite registry.

### Data carried

The request carries an optional DID, a lifetime and an optional label. The response carries the install URL, the claim code, the invite's identifier, its kind and expiry, and whether an entry was written.

### Correlation

A wallet invitation links the community to the DID it names, which the inviter chose; a passkey invitation names no DID, and the DID its claim derives is new and specific to that passkey. The install URL and claim code are random and link nothing.

### Retention

The invite is kept, with the authority it confers, until it is revoked; once consumed or expired it is kept as audit history, as [`list`](../../list/0.2/spec.md) describes.

### Consent/purpose

The install URL and claim code exist to admit one administrator, once, and **MUST NOT** be accepted for anything else. The invitee consents by claiming; until then the invitation grants nothing to anyone.
