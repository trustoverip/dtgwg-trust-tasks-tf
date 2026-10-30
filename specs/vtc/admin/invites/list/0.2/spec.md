---
slug: vtc/admin/invites/list
version: "0.2"
title: VTC Admin Invites — List
summary: Enumerate a Verifiable Trust Community's administrator invites, outstanding and historical — wallet and passkey alike — with their redemption state.
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords:
  - vtc
  - admin
  - invite
  - passkey
  - wallet
  - list
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
  requirement: RECOMMENDED
  rationale: Read-only inventory of administrator invites. Recommended for attribution.
sideEffects:
  level: none
  rationale: "Reads the invite registry; persists nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related:
  - vtc/admin/invites/create
  - vtc/admin/invites/revoke
---

## Abstract

The **VTC Admin Invites — List** Trust Task enumerates the community's administrator invites as [`InviteSummary`](../../../../_shared/0.2/invite.schema.json) entries — outstanding, consumed and expired alike, of both kinds [`vtc/admin/invites/create`](../../create/0.2/spec.md) `0.2` issues. [`vtc/admin/invites/revoke`](../../revoke/0.1/spec.md) acts on an entry's `jti`.

### Changes from 0.1

Each entry gains a required `kind`, `wallet` or `passkey`. A passkey invitation names no DID until it is claimed, so its `targetDid` is absent while it is outstanding and, once consumed, is the DID derived from the passkey that claimed it. Without `kind`, an outstanding passkey invitation was indistinguishable from a row too old to record its DID.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

Producer: no payload members are required.

Consumer: verify the caller may manage administrator invites. Return every invite the community holds, including terminal ones — the consumed rows are the audit trail of who was granted administrator access and when — each with its `kind`, and its `targetDid` whenever the DID is known: a wallet invitation's always, a passkey invitation's once it is consumed.

## Response

### One passkey invitation outstanding, one wallet invitation consumed

```json
{
  "id": "urn:uuid:9a8b7c6d-5e4f-4a3b-8c2d-1e0f9a8b7c61",
  "type": "https://trusttasks.org/spec/vtc/admin/invites/list/0.2#response",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmAdminScid:alice.example",
  "issuedAt": "2026-10-30T09:05:00Z",
  "payload": {
    "invites": [
      { "jti": "0b6f2a9e-4c1d-4e8f-9a2b-3c4d5e6f7a8b", "kind": "passkey", "status": "issued", "expiresAt": "2026-10-30T10:00:01Z" },
      { "jti": "7c3e1a2b-9d8f-4e6a-b5c4-d3e2f1a0b9c8", "kind": "wallet", "status": "consumed", "targetDid": "did:webvh:QmBobScid:bob.example", "consumedAt": "2026-10-29T16:12:44Z" }
    ]
  }
}
```

## Security & Privacy

Invite records name the DIDs granted administrator access, so the list is admin-class metadata. It deliberately does **not** disclose the claim code or install URL: those are minted once by `create` and never retrievable afterwards, so a leaked list cannot be replayed into an enrolment.
