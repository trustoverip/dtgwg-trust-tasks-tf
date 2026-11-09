---
slug: messaging/mux/notice
version: "0.1"
title: "Messaging — Mux Notice"
summary: "Durable notices a mediator stores in a principal's or member DID's inbox, delivered at least once and acknowledged by deletion: grants suspended (recreate them), grants revoked, grants expiring, grants present on a DID."
status: draft
targetFrameworkVersion: "0.6.0"
category: messaging
keywords: [messaging, mediator, mux, multiplexing, notice, grant, suspension, hand-over]
parties:
  - role: Mediator
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: Recipient (a principal, or — for grantsPresent — a member DID)
    requirement: REQUIRED
    member: recipient
    identifierScope: any
proofRequirement:
  request: REQUIRED
  rationale: >-
    A notice is stored in an inbox and acted on later, after the connection that delivered it is gone — a grantsSuspended notice tells the client to discard grants and seek new member signatures. Its authenticity must not depend on the transport that happened to deliver it, so the mediator signs it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A notice may wait in an inbox for a long time and there is no response; issuedAt lets the recipient order notices against its own state and against messaging/mux/list, and recognise a re-delivery.
sideEffects:
  level: none
  rationale: >-
    Receiving a notice changes nothing at the mediator; the recipient acknowledges it by deleting it from its inbox, as for any stored message.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Notices carry delegation metadata into the recipient: member DIDs, grant identifiers, reasons and instants, and for grantsPresent how many delegations of the recipient's DID exist. No principal DID is disclosed to a member, and never any message content.
retention:
  class: transient
  rationale: >-
    The mediator keeps a notice in the recipient's inbox only until the recipient deletes it; the recipient acts on it and discards it. messaging/mux/list remains the source of truth.
errorCodes: []
related:
  - messaging/mux/grant
  - messaging/mux/revoke
  - messaging/mux/revoke-all
  - messaging/mux/list
  - messaging/mux/event
---

## Abstract

The **Messaging — Mux Notice** Trust Task carries what a mux party **must act on even if it was offline when it happened**. Where a live [`messaging/mux/event`](../../event/0.1/spec.md) describes one connection and is never stored, a notice is stored as an ordinary message in the recipient's inbox, delivered at least once, and acknowledged by deleting it. The live `detached` event may also have been sent; the notice is what a client relies on. The `ready` event's `pendingNotices` tells a reconnecting principal how many are waiting.

| `notice` | To | Meaning / required action |
|---|---|---|
| `grantsSuspended` | each affected principal | The unknown-key rule ([`messaging/mux/grant`](../../grant/0.1/spec.md#the-unknown-key-rule)) suspended these grants. **They must be recreated**: a fresh grant signed by the member DID under its current epoch. |
| `grantsRevoked` | each affected principal | The member revoked, the member advanced its epoch with revoke-all, an operator acted, or an account was removed. Informational: there is no recreate path unless the member grants again. |
| `grantsExpiring` | the principal | Grants past about three quarters of their lifetime, whose renewal the application did **not** opt into. They lapse unless the application acts. |
| `grantsPresent` | the member DID | The member DID authenticated directly while grants on it exist, so a new owner learns that delegations exist. |

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is the mediator DID multiplexing design (VTI design docs, `mediator-did-multiplexing-design.md`, §4.9, §5.3, §5.4 and §5.8).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### Members per kind

The payload is one object whose `notice` member says which kind it is. Each kind carries exactly these members besides `notice` (and optionally `ext`), and no others:

| `notice` | REQUIRED |
|---|---|
| `grantsSuspended` | `suspensions` — each entry `did`, `grantIds`, `reason` (`suspendedUnknownKey`), `suspendedAt` |
| `grantsRevoked` | `revocations` — each entry `did`, `grantIds`, `reason` (`revokedByMember`, `epochBumped`, `admin` or `accountRemoved`), `revokedAt` |
| `grantsExpiring` | `expiring` — each entry `grantId`, `did`, `expiresAt` |
| `grantsPresent` | `did`, `count`, `suspended`, `epoch` |

A consumer **MUST** treat a document that carries a member not listed for its kind, or lacks a required one, as malformed.

A conforming **producer** (the mediator) **MUST**:

1. Emit documents whose `type` is `https://trusttasks.org/spec/messaging/mux/notice/0.1`, with itself as `issuer`, the recipient as `recipient`, an `issuedAt` and a `proof`, packed to the recipient and **stored in the recipient's inbox** as an ordinary message, so that it is delivered at least once — live if the recipient is connected, otherwise on its next pickup or connect.
2. Commit the change a notice reports before storing the notice: a `grantsSuspended` or `grantsRevoked` notice is stored only once the grants are suspended or revoked and their members detached.
3. Store `grantsSuspended` for each principal whose grants the unknown-key rule suspended, and `grantsPresent` for the member DID whose new key triggered it, or whenever the member DID authenticates directly while grants on it exist and it has not been told since they last changed.
4. Store `grantsRevoked` for each affected principal when a member DID revokes (messaging/mux/revoke as `member`), advances its epoch (messaging/mux/revoke-all), or an operator revokes, or an account is removed. A principal that revoked its own grants is not sent one.
5. Store `grantsExpiring` once per grant, at about three quarters of its lifetime, only for grants whose renewal the application did not opt into. Batching several grants into one notice is RECOMMENDED.
6. Never disclose a principal DID in a notice to a member DID: `grantsPresent` carries counts only. The member lists principals with messaging/mux/list.

A conforming **consumer** **MUST**:

1. Verify the `proof` before acting on a notice.
2. Delete a notice from its inbox only after it has handled it; deletion is the acknowledgement.
3. Treat the document `id` as the notice's identity, and handle a re-delivered notice as already handled.
4. On `grantsSuspended`, stop relying on the listed grants, and surface the notice to its application. Recreate a grant only where the application opted into recreation for it and the device holds the member DID's key; recreating means a fresh messaging/mux/grant signed under the member's current epoch with a new `grantId` and an `iat` newer than the suspended grant's.
5. On `grantsExpiring`, surface the notice; renew only where the application opted in.
6. Not respond. There is no `#response` form for this task.

## Authorization

**Authority: none is exercised; the mediator informs a party of changes to delegations it is party to.** Every change a notice reports was authorised by its own task or rule — a member's signature, the unknown-key rule, an operator's action — and committed before the notice was stored. A notice goes only to a party to the delegation: a principal hears about its own grants, a member DID about delegations of itself. The `proof` establishes that the mediator issued it; it confers nothing on the recipient, and in particular it is not authority to resume a suspended grant, which only a fresh member signature can replace.

## Definitions

- **`notice`** — REQUIRED. `grantsSuspended`, `grantsRevoked`, `grantsExpiring` or `grantsPresent`.
- **`suspensions`**, **`revocations`** — per member DID, the grant identifiers concerned, a [`DetachReason`](../../_shared/0.1/mux.schema.json#/$defs/DetachReason), and when the change was committed.
- **`expiring`** — grants and their effective expiries.
- **`did`**, **`count`**, **`suspended`**, **`epoch`** — for `grantsPresent`: the member DID, how many grants on it the mediator holds, how many this authentication just suspended, and its current epoch, which a new grant must carry.

## Request

The mediator stores the notice in the recipient's inbox. The payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Grants suspended: recreate them

```json
{
  "id": "urn:uuid:5b6c7d8e-9f0a-4b1c-8d2e-3f4a5b6c7d01",
  "type": "https://trusttasks.org/spec/messaging/mux/notice/0.1",
  "issuer": "did:web:mediator.example",
  "recipient": "did:webvh:QmPrinc1pal:phone.example",
  "issuedAt": "2026-11-09T10:42:17Z",
  "payload": {
    "notice": "grantsSuspended",
    "suspensions": [
      {
        "did": "did:webvh:QmA1ice:alice.example:work",
        "grantIds": ["0b8f6c1e-3a2d-4f5e-9b7c-1d2e3f4a5b6c"],
        "reason": "suspendedUnknownKey",
        "suspendedAt": "2026-11-09T10:42:17Z"
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:mediator.example#key-1",
    "created": "2026-11-09T10:42:17Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

### Grants present on the DID a new owner just authenticated as

```json
{
  "id": "urn:uuid:5b6c7d8e-9f0a-4b1c-8d2e-3f4a5b6c7d02",
  "type": "https://trusttasks.org/spec/messaging/mux/notice/0.1",
  "issuer": "did:web:mediator.example",
  "recipient": "did:webvh:QmA1ice:alice.example:work",
  "issuedAt": "2026-11-09T10:42:17Z",
  "payload": {
    "notice": "grantsPresent",
    "did": "did:webvh:QmA1ice:alice.example:work",
    "count": 1,
    "suspended": 1,
    "epoch": 3
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:web:mediator.example#key-1",
    "created": "2026-11-09T10:42:17Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Security & Privacy

### Data carried

To a principal: member DIDs, grant identifiers, reasons, and instants. To a member DID: counts and its epoch, never a principal DID. No message content, no signed statement and no key material. A producer **MUST NOT** put anything but extension data its ecosystem defines in `ext`.

### Correlation

A notice to a principal restates delegations it already holds. A `grantsPresent` notice tells a DID's current controller that delegations of it exist, and how many — deliberately, because that is how a new owner learns of an old owner's principals — but not who holds them; the member lists them with messaging/mux/list. The mediator's identifier scope is public by design: it signs every notice, and a recipient must recognise it. Notices sit in inboxes, so their count and timing are visible to anyone who can observe the inbox's size.

### Retention

**Transient.** The mediator keeps a notice only until the recipient deletes it. A recipient **SHOULD** discard a notice once it has acted on it; messaging/mux/list is the record of what stands.

### Consent/purpose

The purpose is that no party loses track of a delegation it is party to because it was offline: a principal learns its grants were suspended, revoked or are lapsing, and a member DID learns delegations of it exist. Notices are not a log and are not for analytics.
