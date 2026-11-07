---
slug: vtc/badges/awards/unshare
version: '0.1'
title: VTC Badges — Awards — Unshare
summary: "Stop showing an award outside the community: its public locations answer nothing from now on, as if they never existed. Stopping is not a revocation; the holder keeps the award and its credential."
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- gamification
- portal
- open-badges
parties:
- role: holder
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: Stopping a share is the holder's own choice about their own award (VTI-BDG-070); it must be attributable to them after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed stop would undo a share the holder made since.
sideEffects:
  level: mutating
  rationale: Stops serving one award at its public locations and clears the record of the share. Idempotent; reversed by sharing again, which answers new locations.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: The request names an award; the response is the award as the holder holds it, without public locations.
retention:
  class: durable
  rationale: The award stands; only the share's token goes. Nothing of the share is kept once stopped.
errorCodes:
- code: vtc/badges/awards/unshare:badgesOff
  meaning: The community has badges turned off (VTI-BDG-004). Nothing changed — though with badges off every public location already answers nothing.
  retryable: false
- code: vtc/badges/awards/unshare:notFound
  meaning: No award of the signer's has this identifier. Conflates an unknown award and another member's award.
  retryable: false
related:
- vtc/badges/awards/share
- vtc/badges/mine/list
---

## Abstract

The **VTC Badges — Awards — Unshare** Trust Task stops a share the holder made with [`vtc/badges/awards/share`](../../share/0.1/spec.md) (VTI Community Badges chapter, VTI-BDG-070). From the response on, the award's public locations answer as locations that never existed; the token is discarded, and sharing again answers new locations. Stopping is not a revocation: the holder keeps the award, their membership of the badge group and the credential, and the badge stays on their card as before.

The task is idempotent. An award of the signer's that is not shared is answered as it is.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse every stop with `badgesOff` while its badge settings have `enabled` false (VTI-BDG-004).
2. Refuse with `notFound` an `awardId` that names no award, or an award that is not the signer's — the signer is the recipient named by the award's `entryId`. The two cases answer alike.
3. Stop serving the award's public locations, discard the token, and clear `shared` on the award. A location that no longer serves **MUST** answer as one that never existed (VTI-BDG-070), so a reader cannot tell a stopped share from a wrong guess.
4. Answer the award as it now stands. An award that was not shared is answered unchanged.
5. Revoke nothing and change nothing else: the award, the holder's membership of the badge group, the credential and its status stand as they were. Stopping is not a revocation.
6. Audit the stop by reference — the award, the badge and the holder's entry (VTI-BDG-091).

## Authorization

**Authority: holding the award — read now.** The signer stops only a share of an award of their own; the community reads the award's recipient at execution time and refuses every other signer as `notFound`. Stopping confers and removes nothing (VTI-BDG-002). The proof establishes who asked ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

This is an act of the member (VTI-BDG-095): a member portal holding a key the member authorized for `vtc/badges` under [`auth/oob/grant/0.3`](../../../../../auth/oob/grant/0.3/spec.md) (`signingScope`) **MAY** sign it, for the session that authorization arms.

## Definitions

- `AwardId`, `Award` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).

## Request

The holder (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A holder takes their award off their website

```json
{
  "id": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000511",
  "type": "https://trusttasks.org/spec/vtc/badges/awards/unshare/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-20T09:00:00Z",
  "payload": {
    "awardId": "awd_K3mT9pQx2Lw8Rv5N"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:priya#key-1",
    "created": "2026-11-20T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/awards/unshare/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Stopped

```json
{
  "id": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000512",
  "type": "https://trusttasks.org/spec/vtc/badges/awards/unshare/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000511",
  "payload": {
    "award": {
      "awardId": "awd_K3mT9pQx2Lw8Rv5N",
      "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
      "entryId": "dir_Pr1yaNatarajan00",
      "givenBy": "dir_BenOkafor0000000",
      "note": "For the Saturday joinery workshop.",
      "state": "accepted",
      "offeredAt": "2026-11-05T18:00:00Z",
      "decidedAt": "2026-11-05T19:12:40Z",
      "credentialId": "urn:uuid:9f1b2c3d-4e5f-4a6b-8c7d-000000000777"
    }
  }
}
```

### Refused: not the signer's award

```json
{
  "id": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000513",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-20T09:00:01Z",
  "threadId": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000511",
  "payload": {
    "code": "vtc/badges/awards/unshare:notFound",
    "message": "No award of yours has this identifier.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

An award identifier in; the award as the holder holds it out, without public locations. Nothing about the share — the token, who read it — is answered or kept.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. The stopped locations answer as ones that never existed, so a reader holding an old location learns nothing from its silence that a wrong guess would not also tell them.

### Retention

**Durable** for the award, which stands; nothing of the share is kept once stopped. Sharing again mints a new token, so an old location never comes back to life.

### Consent/purpose

The holder withdrawing their own choice to show an award outside the community (VTI-BDG-070). Nobody but the holder stops a share this way; the community stops one itself only when the award is taken back, the badge is retired or the holder departs. **A badge confers nothing**: the community **MUST NOT** read a badge or an award in any authorization decision of its own (VTI-BDG-002).
