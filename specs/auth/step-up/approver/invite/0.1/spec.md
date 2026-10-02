---
slug: auth/step-up/approver/invite
version: "0.1"
title: "Auth — Step-up Approver Invite"
summary: An administrator issues a single-use invite, redeemed by the invited subject with a separately delivered claim code, to bind a step-up approver to a subject who holds no step-up factor from which to authorize one themselves.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - step-up
  - approver
  - invite
  - enrollment
  - second-factor
  - bootstrap
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: Relying party
    requirement: REQUIRED
    member: recipient
  - role: Invited subject
    requirement: REQUIRED
subjectPath: /subject
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    An invite is the anchor a subject's first step-up approver is bound on, so it must be attributable to the administrator who issued it on every transport, and remain so in the audit record after the transport is gone. Without the proof, anyone able to reach the relying party could mint invites for the subjects whose step-ups they want to answer. The response carries the invite's secret halves back, so it is signed by the relying party too: an administrator can then tell a genuine invite from one an intermediary substituted.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    An invite is an offer to bind a factor, and an offer with no issue time never lapses. Requiring it gives the relying party an outer bound on how long a captured request stays executable, independent of `ttl`.
sideEffects:
  level: mutating
  rationale: >-
    Writes a single-use, expiring invite. Nothing is bound until the invited subject redeems it with auth/step-up/approver/redeem/start and finish; an unredeemed invite lapses on its own.
exposure:
  discloses: secret
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The response carries the invite URL, whose token is one half of the redemption, and the claim code, the other half. Together — with the invited subject's own signature, which redemption also requires — they let a step-up approver be bound to the subject until the invite expires or is redeemed.
retention:
  class: durable
  rationale: >-
    The relying party keeps who invited whom, when, and whether and when the invite was redeemed, as the audit record of the anchor the subject's approver was bound on. The token and code are kept only as hashes, and only until redemption or expiry.
errorCodes:
  - code: auth/step-up/approver/invite:selfInvite
    meaning: "`subject` is the administrator issuing the invite. A subject's step-up factor is never bound on the strength of their own signing key — that would let whoever stole the key mint a factor for it."
    retryable: false
  - code: auth/step-up/approver/invite:subjectUnknown
    meaning: "The relying party does not recognise the subject — it holds no standing a step-up could ever be asked of."
    retryable: false
  - code: auth/step-up/approver/invite:ttlTooLong
    meaning: "`ttl` exceeds 24 hours or the relying party's own lower cap. `details.maxTtl` MAY state the cap in seconds."
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        maxTtl:
          type: integer
          minimum: 1
related:
  - auth/step-up/approver/redeem/start
  - auth/step-up/approver/redeem/finish
  - auth/step-up/approver/enroll
  - auth/step-up/approver/list
  - auth/step-up/approver/revoke
  - auth/step-up/approver/attest
  - auth/passkey/enroll/invite
---

## Abstract

A step-up approver ([`auth/step-up/approver/attest/0.1`](../../attest/0.1/spec.md)) is a subject's own additional factor only if it was bound on an anchor **independent of the subject's signing key**: a second factor bound on the strength of the first adds nothing. A subject who already holds a factor adds another with [`enroll`](../../enroll/0.1/spec.md). A subject who holds none — a wallet administrator who has never had a passkey at this relying party, or one who has lost every factor — needs someone else's authority as the anchor.

This task is that anchor. An administrator asks the relying party for a single-use invite for one subject; the relying party returns a URL and a separate **claim code**; the administrator delivers the two over different channels; the invited subject opens the URL, types the code and signs [`redeem/start`](../../redeem/start/0.1/spec.md) and [`redeem/finish`](../../redeem/finish/0.1/spec.md) with their own DID, binding an approver whose possession the finish proves.

It mirrors [`auth/passkey/enroll/invite/0.2`](../../../../passkey/enroll/invite/0.2/spec.md) with `purpose: stepUp`, but as its own family: an invite that binds no passkey would be misnamed as a passkey invite, and the redemption's evidence (a signed statement, not a WebAuthn attestation) is different.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the administrator) **MUST**:

1. Set itself as `issuer` and the relying party as `recipient`, and sign the document.
2. Populate `payload.subject` with the DID being invited — never its own.
3. Deliver `url` and `claimCode` to the invited subject over **different** channels.

A conforming **consumer** (the relying party) **MUST**:

1. Refuse with `selfInvite` when `payload.subject` is the administrator the proof establishes — the signer, or the identity a delegated key the signer used acts for.
2. Refuse with `subjectUnknown` a subject it holds no standing for.
3. Authorize the administrator for this subject (see [Authorization](#authorization)), refusing with the framework `permissionDenied` otherwise.
4. Refuse with `ttlTooLong` a `ttl` above 86 400 seconds (24 hours) or above its own lower cap. Apply 900 seconds (15 minutes) when `ttl` is absent.
5. Generate a single-use token with at least 128 bits of entropy, a `url` carrying it, and a `claimCode` with at least 40 bits of entropy. Store each only as a hash — the code as a slow, salted one, since it is short.
6. Bind the invite to the subject, the administrator, the optional label and the expiry, and give it an `inviteId`.
7. Record an audit event naming the administrator, the subject and the expiry — never the token or the code.
8. Return `{ inviteId, url, claimCode, expiresAt }`. `claimCode` is returned only here and is never carried in `url`.
9. **SHOULD** tell the subject, over a channel it already has with them, that an invite was issued for them.

A pending invite is superseded by a newer one for the same subject only if the consumer chooses; it **MAY** keep several, each single-use.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authority this task presupposes is **the administrator's standing over the invited subject** at the relying party — at a community, an administrator whose scope covers the subject's. The proof establishes who issued the invite, never that they may; the standing is read from the relying party's own state at execution time.

Because this invite is the anchor on which a factor is bound, the evidence for it is the *administrator's* authority, exercised independently of the subject's key. The reference implementation — a VTC — treats issuing one as an operation that confers authority, and gates it behind an operation-bound step-up satisfied by the administrator's own factor ([`approve-request/0.4`](../../../approve-request/0.4/spec.md) *Inline delivery*), so a single stolen administrator key cannot mint factors for others. That gate is the relying party's policy; this specification describes the evidence the task rests on and does not state which operations require a step-up.

The invite confers nothing by itself. The binding it leads to still requires the claim code, the invited subject's own signature over redemption, and proof of possession of the approver key.

## Definitions

- **`subject`** — the DID being invited to bind an approver. Must be a subject the relying party recognises, and not the administrator.
- **`label`** — a suggested label for the approver, which the subject MAY override at finish.
- **`ttl`** — seconds the invite stays redeemable; at most 24 hours, 15 minutes by default.
- **`inviteId`** — the relying party's identifier for the invite, for audit and for an administrator listing or withdrawing invites. Not a redemption factor.
- **`url`** — where the subject redeems; carries the token.
- **`claimCode`** — the second half, delivered separately.

## Request

The administrator (`issuer`) sends the request to the relying party (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A community administrator invites a wallet administrator to bind an approver

```json
{
  "id": "urn:uuid:8b3d6f2a-1c4e-4a9b-b7d0-5e2f3a4b6c01",
  "type": "https://trusttasks.org/spec/auth/step-up/approver/invite/0.1",
  "issuer": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-01T15:00:00Z",
  "threadId": "urn:uuid:8b3d6f2a-1c4e-4a9b-b7d0-5e2f3a4b6cff",
  "payload": {
    "subject": "did:webvh:QmAliceScid4:wallet.example:alice",
    "label": "Browser plugin",
    "ttl": 900
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmDanaScid8:acme-vtc.example:dana#key-1",
    "created": "2026-10-01T15:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## Response

The relying party answers with the sub-schema reachable via `$anchor: "response"`: the invite's id, its URL, the claim code (returned exactly once) and its expiry. Refusals use `trust-task-error`.

### Issued invite

```json
{
  "id": "urn:uuid:8b3d6f2a-1c4e-4a9b-b7d0-5e2f3a4b6c02",
  "type": "https://trusttasks.org/spec/auth/step-up/approver/invite/0.1#response",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "issuedAt": "2026-10-01T15:00:01Z",
  "threadId": "urn:uuid:8b3d6f2a-1c4e-4a9b-b7d0-5e2f3a4b6cff",
  "payload": {
    "inviteId": "sui_4f7a2c9e1b3d",
    "url": "https://acme-vtc.example/admin/enrol-approver?token=sua_9c2e1f7a6b3d4c8e9a1b2d3e4f5a6b7c8d9e0f1a",
    "claimCode": "7KQ4-MX2P-9TDA",
    "expiresAt": "2026-10-01T15:15:01Z"
  }
}
```

### An administrator invites themselves

```json
{
  "id": "urn:uuid:8b3d6f2a-1c4e-4a9b-b7d0-5e2f3a4b6c03",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmDanaScid8:acme-vtc.example:dana",
  "issuedAt": "2026-10-01T15:00:01Z",
  "threadId": "urn:uuid:8b3d6f2a-1c4e-4a9b-b7d0-5e2f3a4b6cfe",
  "payload": {
    "code": "auth/step-up/approver/invite:selfInvite",
    "message": "an administrator does not invite themselves; ask another administrator",
    "retryable": false
  }
}
```

## Security & Privacy

**Why not a self-invite.** A step-up factor exists to add something to possession of a signing key. If the key alone could bind the factor, whoever stole the key would bind their own and the step-up would add nothing. So an invite is the authority of a *different* administrator, and redemption additionally needs the claim code and the subject's own signature.

**Two channels.** Neither the URL nor the claim code alone redeems the invite. A surface that issues invites **SHOULD** present the two separately and say why.

**Token and code strength.** The token is at least 128 bits (192 RECOMMENDED). The claim code is short enough to type, which is why redemption voids the invite after five wrong codes ([`redeem/start`](../../redeem/start/0.1/spec.md)) and why only the invited subject's signed attempts count.

**Lifetime.** 15 minutes by default and 24 hours at most: long enough to read a message on another channel, short enough that an invite forgotten in an inbox is not a standing route to a factor.

**Free text.** `label` is bounded at 64 characters, authored by the administrator, and a suggestion the subject MAY override; a surface rendering it before redemption SHOULD attribute it to the administrator.

### Data carried

The request carries the subject's DID, an optional label and an optional lifetime. The response carries the invite id, the URL with its token, the claim code and the expiry — together, with the subject's signature, the means to bind a factor to the subject. The relying party keeps only hashes of the token and code. A producer **MUST NOT** put anything identifying beyond a device description in `label`.

### Correlation

The invite links the administrator to the subject and to the moment they chose to let the subject bind a factor. The URL names the relying party's origin and no subject, so a URL seen alone does not say who was invited.

### Retention

The token and code hashes are kept until the invite is redeemed, voided or expires, then discarded. The audit record of issuance — administrator, subject, expiry, never the token or code — is durable, because it is the anchor the binding rested on.

### Consent/purpose

The invite exists to bind one step-up approver to one subject. Its token and code **MUST NOT** be accepted for anything else — in particular not to sign in, open a session or bind any other kind of credential. Whether the relying party asks the administrator for a step-up before issuing one is its own policy.
