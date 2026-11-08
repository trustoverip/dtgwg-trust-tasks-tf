---
slug: vtc/badges/mine/accept
version: '0.1'
title: VTC Badges — Mine — Accept
summary: 'A member accepts a badge offered to them: for a community badge the badge group issues their badge credential first and admits them after; for a kudos badge the giving is counted. Nothing shows on a card until this.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- award
- accept
- open-badges
parties:
- role: member
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: Accepting puts a statement on the member's card in their name and, for a community badge, a credential naming them as subject into their wallet. The acceptance must be attributable to the member after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed accept of an already-accepted award changes nothing, but bounding it keeps a captured document from acting after the member changed their mind.
sideEffects:
  level: mutating
  rationale: Marks the award accepted. For a community badge, has the badge group issue (or re-issue) the member's badge credential and records their badge-group membership. Reversed by vtc/badges/awards/revoke.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: true
  rationale: The member exercises their own choice to hold the badge; the response is the award and their view of the badge, never the credential itself, which reaches their agent as an offer.
retention:
  class: durable
  rationale: The accepted award is the holder's and the badge's history (VTI-BDG-005).
errorCodes:
- code: vtc/badges/mine/accept:badgesOff
  meaning: The community has badges turned off (VTI-BDG-004). The offer stands; nothing was recorded.
  retryable: true
- code: vtc/badges/mine/accept:alreadyDecided
  meaning: The award is not `offered` — it was accepted, declined, withdrawn, expired or revoked. The current award is in `details.state`.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      state:
        type: string
        maxLength: 32
- code: vtc/badges/mine/accept:badgeRetired
  meaning: The badge was retired after the offer was made. The offer is closed.
  retryable: false
- code: vtc/badges/mine/accept:keyHolderUnavailable
  meaning: The community's VTA did not answer the signing request for the badge credential. The offer stays `offered` and nothing was recorded; accept again.
  retryable: true
- code: vtc/badges/mine/accept:keyHolderRefused
  meaning: The community's VTA refused to sign the badge credential. Its own code and message are carried in `details`. The offer stays `offered`.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      code:
        type: string
        maxLength: 128
      message:
        type: string
        maxLength: 500
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/badges/awards/offer
- vtc/badges/mine/decline
- vtc/badges/mine/list
- vtc/badges/mine/hide
- vtc/badges/kudos/give
- vtc/groups/members/add
- keys/sign-document
---

## Abstract

The **VTC Badges — Mine — Accept** Trust Task is the member's side of an award. An offer made by a badge owner ([`vtc/badges/awards/offer`](../../../awards/offer/0.1/spec.md)) or a kudos giving ([`vtc/badges/kudos/give`](../../../kudos/give/0.1/spec.md)) waits in state `offered` until its recipient accepts it here or declines it with [`vtc/badges/mine/decline`](../../decline/0.1/spec.md). **Nothing is admitted, issued or shown before acceptance** (VTI-BDG-020, VTI-BDG-021).

For a **community badge**, acceptance is the admission: the community has its VTA sign, as the badge group's DID, the member's **badge credential**, offers it to the member's agent, and only then records their membership of the badge group — the order [`vtc/groups/members/add`](../../../../groups/members/add/0.1/spec.md) fixes for every group that issues credentials. For a **kudos badge** the giving is counted and nothing is issued. A repeatable badge accepted again appends an award: the credential is re-issued with every accepted award as evidence (VTI-BDG-022).

## The badge credential

Where the badge is a community badge, admitting the member includes this step **before** the membership is recorded (VTI-BDG-060):

- The community has its VTA sign, **as the badge group's DID** and through the generic `keys/sign-document/0.1` Trust Task (specified separately), an **Open Badges 3.0** `OpenBadgeCredential` — a profile of the W3C Verifiable Credentials Data Model v2.0 — in place of a DTG group membership credential:
  - `@context` `["https://www.w3.org/ns/credentials/v2", "https://purl.imsglobal.org/spec/ob/v3p0/context-3.0.3.json"]`, `type` `["VerifiableCredential", "OpenBadgeCredential"]`;
  - `issuer` a `Profile` whose `id` is the badge group's DID;
  - `credentialSubject` an `AchievementSubject` whose `id` is the member's membership DID and whose `achievement` is the badge: `id` a stable identifier of the badge the community serves, `name`, `description`, `criteria.narrative` from the badge's criteria, `image` the badge's picture, `creator` a `Profile` whose `id` is the community's DID, and `endorsement` carrying the community's `EndorsementCredential` of the Achievement;
  - one `evidence` entry per accepted award — its `narrative` the award's note, where there is one, and its date — so a repeatable badge's count is the length of `evidence`;
  - `validFrom` now and `validUntil` by the badge's regime (below);
  - the proof suite the community uses for every credential it issues.
- **Regime.** Where the badge's holders may not show it outside (`shareableOutside` false), the credential is short-lived and renewed through the community's VTA for as long as the holder stays, with no `credentialStatus` and no status set of the badge's or the group's own (VTI-BDG-061; the community's setting, defaulting to 30 days renewed when 10 remain, as a group membership credential). Where they may (`shareableOutside` true), its validity is the settings' `credentialValidDays` and it carries a `credentialStatus` entry in the **community's own status set** — the set its community credentials use, allocated as VTI-CRD-014 requires — which the community issues and alone changes (VTI-BDG-062). A re-issue under this regime revokes the previous credential's entry.
- The community offers the credential to the member through the existing credential-exchange offer path ([`credential-exchange/offer`](../../../../../credential-exchange/offer/0.1/spec.md)), as it offers any credential it issues, and keeps its `id` on the award as `credentialId`.

A verifier chains the badge credential to the community by its endorsement, by the community's authorization credential to the badge group, or by a trust registry (VTI-BDG-064). The community itself never reads it (VTI-BDG-002).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** is the member the award names, signing with their membership DID or with a key the portal holds for them (Authorization).

A conforming **consumer** (the community) **MUST**:

1. Refuse with `badgesOff` while its badge settings have `enabled` false, leaving the offer as it is.
2. Answer `notFound` for an award that does not name the signer's entry, `alreadyDecided` for one not in state `offered`, and `badgeRetired` where the badge was retired since (closing the offer as `withdrawn`).
3. Re-check, now, that the signer is still an active member and, for a community badge, that the badge group still exists and the badge is still `active`.
4. For a community badge: issue the badge credential as *The badge credential* describes — a first credential for a first award, a re-issue carrying every accepted award as evidence for a repeat — and offer it to the member's agent, **before** anything is recorded; answer `keyHolderUnavailable` or `keyHolderRefused`, recording nothing and leaving the award `offered`, when its VTA does not sign. Under the outside regime, revoke the previous credential's status entry after the new one is issued.
5. **Then** record the award `accepted` with `decidedAt` and `credentialId`, and, for a first award of a community badge, record the member's badge-group membership with source `added` (or `requestApproved` for an award that came from a request). The badge joins the member's wall unhidden, unpinned, last in their order.
6. For a kudos badge: record the award `accepted`; issue nothing (VTI-BDG-041).
7. Audit by reference — the award, the badge and the member — never the note.

## Authorization

**Authority: the award names the signer.** An award is accepted by its recipient and nobody else; no capability is involved. Accepting confers nothing: a badge is never read by an authorization decision (VTI-BDG-002). The proof establishes who asked ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

Accepting is an act of the member (VTI-BDG-095): a member portal that holds a key the member authorized to sign for them for `vtc/badges` (`auth/oob` `signingScope`) **MAY** sign it with that key, for the session that authorization arms. The credential itself never passes through the portal: it is offered to the member's own agent.

## Definitions

- `Award`, `AwardState`, `MyBadge` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).
- **Badge credential** — the `OpenBadgeCredential` a badge group issues its holders, above.

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Accepting a workshop badge

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000301",
  "type": "https://trusttasks.org/spec/vtc/badges/mine/accept/0.1#request",
  "issuer": "did:example:noor",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T19:10:00Z",
  "payload": {
    "awardId": "awd_Qz7TpLW5k2N8xRb3"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/mine/accept/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Accepted: the credential was issued and the badge is on the card

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000302",
  "type": "https://trusttasks.org/spec/vtc/badges/mine/accept/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:noor",
  "issuedAt": "2026-11-06T19:10:02Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000301",
  "payload": {
    "award": {
      "awardId": "awd_Qz7TpLW5k2N8xRb3",
      "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
      "entryId": "dir_NoorElSayed00000",
      "givenBy": "dir_Pr1yaNatarajan00",
      "note": "Resin pour workshop, 25 October 2026",
      "state": "accepted",
      "offeredAt": "2026-11-06T18:00:01Z",
      "decidedAt": "2026-11-06T19:10:02Z",
      "credentialId": "urn:uuid:9c1d2e3f-4a5b-4c6d-8e7f-000000000777"
    },
    "badge": {
      "badge": {
        "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
        "kind": "community",
        "name": "Workshop host",
        "description": "Ran a workshop for the community.",
        "image": { "sha256": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08", "mediaType": "image/png", "size": 61440 },
        "repeatable": true,
        "requestable": true,
        "listed": true,
        "shareableOutside": true,
        "groupId": "grp_Hs7Qm2Lk9Tn4Bv1X",
        "state": "active",
        "holderCount": 10,
        "createdAt": "2026-11-06T09:00:02Z"
      },
      "count": 1,
      "since": "2026-11-06T19:10:02Z",
      "latestAt": "2026-11-06T19:10:02Z",
      "pinned": false,
      "hidden": false,
      "position": 7,
      "credentialId": "urn:uuid:9c1d2e3f-4a5b-4c6d-8e7f-000000000777",
      "credentialValidUntil": "2028-11-06T19:10:02Z"
    }
  }
}
```

### Refused: the owner withdrew the offer first

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000303",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:noor",
  "issuedAt": "2026-11-06T19:10:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000304",
  "payload": {
    "code": "vtc/badges/mine/accept:alreadyDecided",
    "message": "This offer was withdrawn.",
    "retryable": false,
    "details": { "state": "withdrawn" }
  }
}
```

## Security & Privacy

### Data carried

An award identifier in; the award and the holder's view of the badge out. The credential is never in the response: it reaches the member's own agent as a credential offer.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The badge credential names the member's membership DID as subject, which is what they will present it with; under the outside regime its status entry sits in the community's whole status set, so a status check names neither the badge nor the member (VTI-CRD-013, VTI-CRD-014).

### Retention

**Durable.** The accepted award is kept as the holder's and the badge's history.

### Consent/purpose

The member's choice to hold a badge. **Acceptance is the consent** VTI-BDG-020 requires before anything is admitted, issued or shown. A badge confers nothing: the community **MUST NOT** read the award, the membership or the credential in any authorization decision of its own (VTI-BDG-002, VTI-DIR-087).
