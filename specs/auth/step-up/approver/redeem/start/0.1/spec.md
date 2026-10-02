---
slug: auth/step-up/approver/redeem/start
version: "0.1"
title: "Auth — Step-up Approver Redeem (start)"
summary: The invited subject, signing as their own DID, presents a step-up approver invite's token and its separately delivered claim code, and the relying party opens an enrolment ceremony with a challenge for the approver to sign.
status: draft
targetFrameworkVersion: "0.6.0"
category: authentication
keywords:
  - step-up
  - approver
  - invite
  - redeem
  - enrollment
parties:
  - role: Invited subject
    requirement: REQUIRED
    member: issuer
  - role: Relying party
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The proof is what makes the invited subject, and no one else, the redeemer. An invite's token and code are bearer halves that can be intercepted; requiring the subject's own signature means whoever intercepts both still cannot bind their device to someone else's DID, and means only the subject's attempts count toward voiding the invite.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. Placing the request in time also bounds how long a captured redemption request stays replayable, independent of the invite's own expiry.
sideEffects:
  level: mutating
  rationale: >-
    Opens an enrolment ceremony with a fresh challenge and counts wrong claim codes against the invite; the fifth wrong code voids the invite. Nothing is bound, and the invite is not consumed, until the matching finish.
exposure:
  discloses: metadata
  ingests: secret
  actsAsSubject: false
  rationale: >-
    The request carries the invite token and claim code, which with the subject's signature authorise a binding. The response returns a ceremony id, a challenge, the relying party's DID and an expiry.
retention:
  class: exchange
  rationale: >-
    The ceremony lives only until the finish or its expiry. Wrong-code attempts are counted against the invite for the invite's lifetime only.
errorCodes:
  - code: auth/step-up/approver/redeem/start:inviteNotFound
    meaning: "The token names no invite this relying party holds, or one already redeemed."
    retryable: false
  - code: auth/step-up/approver/redeem/start:inviteExpired
    meaning: "The invite's lifetime has elapsed. Ask the administrator for a new one."
    retryable: false
  - code: auth/step-up/approver/redeem/start:inviteVoided
    meaning: "Five wrong claim codes were presented for this invite, and it has been voided — by this request or an earlier one. Ask the administrator for a new one."
    retryable: false
  - code: auth/step-up/approver/redeem/start:notInvitedSubject
    meaning: "The document is not signed by the subject the invite was issued for. Checked before the claim code, and not counted as a wrong code."
    retryable: false
  - code: auth/step-up/approver/redeem/start:codeMismatch
    meaning: "The claim code is wrong. `details.attemptsRemaining` MAY state how many wrong codes remain before the invite is voided."
    retryable: true
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        attemptsRemaining:
          type: integer
          minimum: 0
          maximum: 4
related:
  - auth/step-up/approver/invite
  - auth/step-up/approver/redeem/finish
  - auth/step-up/approver/attest
---

## Abstract

The first leg of redeeming an [`auth/step-up/approver/invite`](../../../invite/0.1/spec.md). The invited subject — typically through their wallet, from the page the invite URL opened — signs this document **as their own DID**, carrying the token from the URL and the claim code they were given separately. The relying party answers with an `enrollmentId`, a fresh `challenge` and its own DID as `audience`: the values the approver will sign an enrolment statement over in [`redeem/finish`](../../finish/0.1/spec.md).

Unlike [`auth/passkey/enroll/redeem/start`](../../../../../passkey/enroll/redeem/start/0.1/spec.md), which a browser with no trusted key sends unsigned and which therefore answers every failure with one code, this request is signed by the invited subject. The relying party knows who is asking before it looks at the code, so it can say precisely what went wrong without becoming an oracle to anyone but the subject the invite is for.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the invited subject) **MUST** set `issuer` to its own DID and sign the document with a verification method of that DID — not with a delegated key such as a console signing key.

A conforming **consumer** (the relying party), in this order:

1. **MUST** verify the proof, and require its verification method to belong to the `issuer` DID's own document. A proof by a delegated key acting for the subject → `notInvitedSubject`.
2. **MUST** look the invite up by a hash of `token`. None, or already redeemed → `inviteNotFound`; voided → `inviteVoided`; past its expiry → `inviteExpired`.
3. **MUST** refuse with `notInvitedSubject` when `issuer` is not the invite's subject, **before** examining `claimCode`, and **MUST NOT** count that attempt. Only the invited subject can spend the invite's attempts, so nobody holding a captured URL can void an invite by guessing.
4. **MUST** verify `claimCode` against its stored hash in constant time. On a mismatch, count it; on the fifth wrong code for the invite, void the invite and refuse with `inviteVoided`; otherwise refuse with `codeMismatch`.
5. **MUST** generate a fresh `enrollmentId` and a `challenge` of at least 128 bits, bound to the invite, the subject and an expiry no later than the invite's (RECOMMENDED: 5 minutes).
6. **MUST** leave the invite unconsumed: a ceremony the subject abandons may be restarted until the invite expires.
7. **SHOULD** rate-limit this task per source and per invite.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authority this task presupposes is **an unexpired, unconsumed, unvoided invite for the signer, together with its claim code**. The invite carries the issuing administrator's authority over the subject; the claim code shows the second channel reached its holder; the signature shows the holder is the subject the invite names. None is sufficient alone, and the subject's signature confers nothing on its own — it is a key the subject already holds, which is exactly why it cannot be the anchor of a step-up factor and why the invite is.

## Definitions

- **`token`** — the invite token, as carried in the invite URL.
- **`claimCode`** — the claim code the administrator delivered separately.
- **`enrollmentId`** — the ceremony handle the finish echoes, and the `boundTo` of the approver's enrolment statement.
- **`challenge`** — the value the approver signs over in its enrolment statement.
- **`audience`** — the relying party's DID, which the statement names.

## Request

The invited subject (`issuer`) sends the request to the relying party (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Alice redeems the invite from her wallet

```json
{
  "id": "urn:uuid:9c4e7a3b-2d5f-4b0c-8e1a-6f3a4b5c7d01",
  "type": "https://trusttasks.org/spec/auth/step-up/approver/redeem/start/0.1",
  "issuer": "did:webvh:QmAliceScid4:wallet.example:alice",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-01T15:03:00Z",
  "threadId": "urn:uuid:9c4e7a3b-2d5f-4b0c-8e1a-6f3a4b5c7dff",
  "payload": {
    "token": "sua_9c2e1f7a6b3d4c8e9a1b2d3e4f5a6b7c8d9e0f1a",
    "claimCode": "7KQ4-MX2P-9TDA"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmAliceScid4:wallet.example:alice#key-1",
    "created": "2026-10-01T15:03:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3kg…"
  }
}
```

## Response

The relying party answers with the sub-schema reachable via `$anchor: "response"`: the ceremony's id, the challenge, its own DID and the ceremony's expiry. Refusals use `trust-task-error`.

### The ceremony is open

```json
{
  "id": "urn:uuid:9c4e7a3b-2d5f-4b0c-8e1a-6f3a4b5c7d02",
  "type": "https://trusttasks.org/spec/auth/step-up/approver/redeem/start/0.1#response",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmAliceScid4:wallet.example:alice",
  "issuedAt": "2026-10-01T15:03:01Z",
  "threadId": "urn:uuid:9c4e7a3b-2d5f-4b0c-8e1a-6f3a4b5c7dff",
  "payload": {
    "enrollmentId": "enr_5b0e3a2e6f4c4b8f9d2a1f0c2b7e4a01",
    "challenge": "RW5yb2xDaGFsbGVuZ2VOb25jZTAxMjM0NTY",
    "audience": "did:webvh:QmVtcScid7:acme-vtc.example",
    "expiresAt": "2026-10-01T15:08:01Z"
  }
}
```

### A wrong claim code

```json
{
  "id": "urn:uuid:9c4e7a3b-2d5f-4b0c-8e1a-6f3a4b5c7d03",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmAliceScid4:wallet.example:alice",
  "issuedAt": "2026-10-01T15:03:01Z",
  "threadId": "urn:uuid:9c4e7a3b-2d5f-4b0c-8e1a-6f3a4b5c7dfe",
  "payload": {
    "code": "auth/step-up/approver/redeem/start:codeMismatch",
    "message": "the claim code is wrong",
    "retryable": true,
    "details": { "attemptsRemaining": 3 }
  }
}
```

## Security & Privacy

**Precise codes, and why they are safe here.** The passkey redemption answers every failure with one code because its sender is anonymous. This one is signed: the relying party reaches the claim code only after it has established that the signer is the invited subject (Conformance item 3), so `codeMismatch` and `attemptsRemaining` tell nothing to anyone but the subject who already holds the token and is entitled to try.

**Voiding.** Five wrong codes void the invite. Because only the invited subject's attempts count, a party that intercepted the URL cannot burn the subject's invite, and the subject — who has the code on another channel — has ample margin.

**The claim code is never logged.** It is compared with its stored hash; failures are recorded against the invite only as a count.

### Data carried

The request carries the invite token and the claim code, signed by the subject. The response carries a ceremony id, a challenge, the relying party's DID and an expiry — nothing about the invite's issuer and no other subject's data.

### Correlation

The relying party learns that the invited subject redeemed, from where and when, which is the purpose of the task. The `enrollmentId` and `challenge` are single-use and expire with the ceremony.

### Retention

The ceremony state is kept until the finish or its expiry. Wrong-code counts live with the invite and are discarded with it; the audit record notes a voiding.

### Consent/purpose

The token and code are used to open this one enrolment and for nothing else.
