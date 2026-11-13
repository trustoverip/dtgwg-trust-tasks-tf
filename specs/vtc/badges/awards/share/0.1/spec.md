---
slug: vtc/badges/awards/share
version: '0.1'
title: VTC Badges — Awards — Share
summary: "Show an accepted award outside the community: the holder asks, and the community answers the public locations at which it serves the award's credential, a verification of it, and the badge's picture with the credential baked in."
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
  rationale: Showing a badge outside the community is the holder's own choice about their own award (VTI-BDG-070); a public location that names them must be attributable to them after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed share would undo a stop the holder made since.
sideEffects:
  level: mutating
  rationale: Starts serving one award at public locations, and records that the holder chose to. Reversed with vtc/badges/awards/unshare, after which the locations answer nothing.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: true
  rationale: The request names an award; the response is the award record with the public locations the community now serves it at. The credential itself is not returned here — the holder already holds it — but from then on it is served to anyone holding the location, by the holder's choice.
retention:
  class: durable
  rationale: The holder's choice and the location's token are kept until the holder stops sharing, the award is taken back, the badge is retired or the holder departs.
errorCodes:
- code: vtc/badges/awards/share:badgesOff
  meaning: The community has badges turned off (VTI-BDG-004). Nothing changed.
  retryable: false
- code: vtc/badges/awards/share:notFound
  meaning: No accepted award of the signer's has this identifier. Conflates an unknown award, another member's award, and an award of the signer's that is not `accepted` — so a share does not tell a stranger that an award exists.
  retryable: false
- code: vtc/badges/awards/share:notShareable
  meaning: The award is the signer's and accepted, but its badge is not one the community marked as shown outside (`shareableOutside` false, or a kudos badge), or the badge is retired. `details.reason` says which.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      reason:
        type: string
        enum:
        - insideOnly
        - kudos
        - retired
- code: vtc/badges/awards/share:publicLocationUnavailable
  meaning: The community serves no public location for badges — it has no public origin configured — so nothing can be shared outside it yet. Try again once the community does.
  retryable: true
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/badges/awards/unshare
- vtc/badges/mine/list
- vtc/badges/mine/accept
- vtc/badges/create
- vtc/directory/card/publish
---

## Abstract

The **VTC Badges — Awards — Share** Trust Task is how a badge's holder shows one of their awards outside the community (VTI Community Badges chapter, VTI-BDG-070). Only a badge the community marked as one its holders may show outside (`shareableOutside`, fixed when the badge is defined or promoted) can be shared, and only by its holder, award by award. The community answers the public locations it serves the award at: a **verification location** that answers the badge credential exactly as issued, with its status in the community's own status set, for any Open Badges verifier to check; and, where the community serves one, a **baked image** — the badge's picture with the credential baked in — for a website or an email signature.

The locations are the community's to choose and are **unguessable**: a random token of at least 128 bits, never the award's identifier, so a location is known only to those the holder gives it to. Nothing served there discloses any other holder or how many hold the badge (VTI-BDG-071). Sharing is idempotent: a share of an award already shared answers the same locations. [`vtc/badges/awards/unshare`](../../unshare/0.1/spec.md) stops it, after which the locations answer nothing; stopping is not a revocation. The member's published external card may name the locations ([`vtc/directory/card/publish`](../../../../directory/card/publish/0.1/spec.md), VTI-BDG-072); the community never adds one the member did not choose.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse every share with `badgesOff` while its badge settings have `enabled` false (VTI-BDG-004).
2. Refuse with `notFound` an `awardId` that names no award, an award that is not the signer's — the signer is the recipient named by the award's `entryId` — and an award of the signer's whose `state` is not `accepted`. The three cases answer alike.
3. Refuse with `notShareable` an award of a badge that is not marked `shareableOutside` (`details.reason` `insideOnly`), of a kudos badge (`kudos`), or of a retired badge (`retired`). A badge's regime is fixed when it is defined or promoted (VTI-BDG-062); a share never changes it.
4. Refuse with `publicLocationUnavailable` when it has no public origin at which to serve the locations. It **MUST NOT** answer a location it will not serve.
5. Otherwise serve, at a location of its own choosing under its public origin, the holder's badge credential **exactly as issued** — the signed document the holder was offered, with its `credentialStatus` — together with a verification of it, to anyone who holds the location, over HTTPS, without authentication (VTI-BDG-070). The location **MUST** carry a token of at least 128 bits of entropy, generated for this share, and **MUST NOT** be derived from the award, the badge, the holder or the credential. A share of an award already shared **MUST** answer the locations in force, not new ones.
6. Where it serves a baked image, bake the credential it serves at the verification location into the badge's picture as the Open Badges baking specification describes for the picture's media type, at a location under the same token, and name it `bakedImageUrl`.
7. Serve, at the locations, nothing that names any other holder of the badge, nor how many hold it, nor any member the credential's evidence does not already name (VTI-BDG-071). The badge's Achievement document and its picture at their public locations (VTI-BDG-051) are the badge's, and name no holder.
8. Record the share on the award (`shared`: the locations and `since`) and answer the award. Audit the share by reference — the award, the badge and the holder's entry — never the token (VTI-BDG-091).
9. Stop serving the locations, and clear `shared`, when the holder stops sharing ([`vtc/badges/awards/unshare`](../../unshare/0.1/spec.md)), when the award is taken back ([`vtc/badges/awards/revoke`](../../revoke/0.1/spec.md)), when the badge is retired ([`vtc/badges/retire`](../../../retire/0.1/spec.md)), and when the holder departs (VTI-BDG-090). A location that no longer serves **MUST** answer as one that never existed.
10. Serve, after the holder's credential is re-issued or renewed in the same regime (a repeat award, a renewal), the credential now in force at the same locations: the share is of the award, and the locations outlive the credential's re-issue.

A conforming **producer** (the holder's portal or wallet) **MUST** present the locations to the holder as theirs to give out, and **MUST NOT** publish them anywhere on the holder's behalf without the holder's choice.

## Authorization

**Authority: holding the award — read now.** The signer shares only an award of their own that they accepted; the community reads the award's recipient at execution time and refuses every other signer as `notFound`. Sharing confers nothing and changes nothing the community recorded about the holding (VTI-BDG-002): it starts serving, to the public, what the holder already holds. The proof establishes who asked ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

This is an act of the member (VTI-BDG-095): a member portal holding a key the member authorized for `vtc/badges` under [`auth/oob/grant/0.3`](../../../../../auth/oob/grant/0.3/spec.md) (`signingScope`) **MAY** sign it, for the session that authorization arms.

## Definitions

- `AwardId`, `Award`, `AwardShare` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).

## Request

The holder (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A holder shows their Workshop host award on their website

```json
{
  "id": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000501",
  "type": "https://trusttasks.org/spec/vtc/badges/awards/share/0.1#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T12:00:00Z",
  "payload": {
    "awardId": "awd_K3mT9pQx2Lw8Rv5N"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:priya#key-1",
    "created": "2026-11-06T12:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/awards/share/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Shared: the locations the holder may give out

```json
{
  "id": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000502",
  "type": "https://trusttasks.org/spec/vtc/badges/awards/share/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-06T12:00:01Z",
  "threadId": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000501",
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
      "credentialId": "urn:uuid:9f1b2c3d-4e5f-4a6b-8c7d-000000000777",
      "shared": {
        "verificationUrl": "https://harbourside.example/badges/shared/Qm7xV2kR9pLw4Nt8Hs3Fz6Bd1Yc5Je0Ua",
        "bakedImageUrl": "https://harbourside.example/badges/shared/Qm7xV2kR9pLw4Nt8Hs3Fz6Bd1Yc5Je0Ua/image.png",
        "since": "2026-11-06T12:00:01Z"
      }
    }
  }
}
```

### Refused: the badge is kept inside the community

```json
{
  "id": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000503",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-06T12:00:01Z",
  "threadId": "urn:uuid:7c2e4a9b-1d3f-4b6e-8a5c-000000000501",
  "payload": {
    "code": "vtc/badges/awards/share:notShareable",
    "message": "This badge is shown inside the community only.",
    "retryable": false,
    "details": { "reason": "insideOnly" }
  }
}
```

## Security & Privacy

### Data carried

An award identifier in; the award with its public locations out. From the moment the response is sent, the holder's badge credential is served to anyone who holds the verification location: it names the holder by their membership identifier, the badge, the community as the badge's creator, and each award's evidence, including the giver's note where the giver wrote one. A holder who does not want a note shown outside should not share that award; a community **SHOULD** say so where it offers the share.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. The holder's membership identifier is in the credential, as it is in every credential the community issues them; sharing makes it readable to whoever holds the location, which is the holder's choice. The location's token is random and names nothing: two shares of two awards do not share a prefix a reader could walk, and a location that stopped serving answers as one that never existed, so a reader cannot tell a stopped share from a wrong guess. The status set the credential names is the whole community's (VTI-CRD-014), so checking the award's status singles out no small group.

### Retention

**Durable.** The holder's choice and the token are kept until the holder stops sharing, the award is taken back, the badge is retired or the holder departs; then the locations answer nothing and `shared` is cleared. The credential served is the one the community already keeps for the holder; the share keeps no copy of its own.

### Consent/purpose

The holder choosing, award by award, to show a badge they earned to people outside the community, on a website or in an email signature, in a form those people can verify. Only a badge the community marked as one its holders may show outside can be shared (VTI-BDG-070), and only the holder shares it: never the community, an owner or an administrator on the holder's behalf. **A badge confers nothing**: the community **MUST NOT** read a badge or an award in any authorization decision of its own (VTI-BDG-002).
