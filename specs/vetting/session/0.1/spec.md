---
slug: vetting/session
version: "0.1"
title: Vetting — Session
summary: A vetter opens a vetting session with an applicant it agreed to vet, while the two are together; the applicant answers with a signed Vetting Card bound to this session. The session document's id is what the resulting Vetting Statement cites as taskContext.
status: draft
targetFrameworkVersion: "0.5"
category: identity
keywords:
  - vetting
  - identity-vetting
  - session
  - vetting-card
  - liveness
  - match-code
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: vetter
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: applicant
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The request is what a Vetting Statement cites — its `id` as `taskContext`, its
    content as the task digest — and is relied on by a community long after the two
    people have left the call, so the challenge it issues must be attributable to the
    vetter. The response delivers the card; the card carries the applicant's own
    signature, but on a relayed path only the envelope proof attributes the delivery
    and ties it to this session's thread.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A session is opened for one sitting. Replayed, it would ask the applicant's agent for a second card against a challenge the vetter has already consumed, from a vetter who is no longer on the call.
sideEffects:
  level: mutating
  rationale: "The applicant's agent signs a Vetting Card with the join DID's key and records the disclosure; the vetter holds session state until a card arrives or the session lapses."
exposure:
  discloses: secret
  ingests: metadata
  actsAsSubject: true
  rationale: "The response is the Vetting Card: the applicant's legal name and whatever other claims they chose to show, signed with the join DID's assertion key and held by the vetter. That is identity material about a person, delivered under their own signature, and is graded at the top of the scale for that reason. Producing it exercises the applicant's own authority — the card is a signature in their name. The request carries only binding material and claim types."
retention:
  class: durable
  rationale: "This document's `id` is what a Vetting Statement carries as `taskContext`, and its content is what the statement's task digest is computed over, so an applicant retains it for as long as it holds the statement. The card inside the response is not durable: the vetter keeps it only as long as its own retention policy allows, and afterwards at most its digest, which the statement already carries."
errorCodes:
  - code: vetting/session:unknownRequest
    meaning: This vetter holds no accepted vetting request from this applicant with the named requestId.
    retryable: false
  - code: vetting/session:claimUnavailable
    meaning: The applicant cannot, or will not, present a claim the session requires.
    retryable: false
  - code: vetting/session:matchCodeMismatch
    meaning: The applicant's agent derives a different match code from the one the vetter reads out, so the session the applicant received is not the one the vetter opened. The vetter may open a fresh session.
    retryable: true
related:
  - vetting/request
  - vetting/decline
  - credential-exchange/issue
  - vtc/join-requests/manifest
---

## Abstract

A vetter who has accepted an applicant's [`vetting/request`](../../request/0.1/spec.md) opens a session when the two are together, in the same room or on a live call. The session issues a **challenge** and names the **claims** the applicant must show. The applicant's `#response` is a signed **Vetting Card** carrying those claims, bound to this vetter and this session. The vetter checks the card against the person in front of them and whatever documentation the vetter relies on. Then it either issues a Vetting Statement over [`credential-exchange/issue`](../../../credential-exchange/issue/0.1/spec.md) or declines with [`vetting/decline`](../../decline/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5 and may change in place while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## The session's name, and the statement it produces

A session is **its own exchange**, conducted inside the request exchange, and two rules follow. They mirror [`witness/session`](../../../witness/session/0.1/spec.md) for the same reasons.

1. **This document's `id` names the session.** Per [SPEC §4.9.1](/SPEC.md#491-naming-an-exchange-from-outside-the-framework), a citation naming an exchange as evidence names the innermost exchange that attests the event, by the `id` of the document that opened it. The vetting happened in *this* exchange, not in the request exchange, whose documents say only that a vetter agreed to meet. A Vetting Statement issued for this session **MUST** carry this document's `id` as `taskContext`, and **SHOULD** carry this document's *task digest* ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names)) as `taskDigestMultibase`, so the citation is bound to this document rather than to anything that reuses its `id`.
2. **Every document of the session carries `parentThreadId`.** A producer **MUST** set `parentThreadId` ([SPEC §4.9.2](/SPEC.md#492-the-parentthreadid-member)) to the request exchange's `threadId` on every document of this exchange, error responses included. A consumer **MUST NOT** reject a document solely for its absence.

The statement is a DTG `EndorsementCredential` whose `credentialSubject.endorsement` is the identity-vetting body defined in [`vetting/_shared/0.1/identity-vetting`](../../_shared/0.1/identity-vetting.schema.json). Its credential envelope belongs to DTG credentials. What this specification fixes is how the statement relates to the session. Its `issuer` is the session's `issuer`, and `credentialSubject.id` is the card's `publisher`. `endorsement.community` is the session's `domain` and `endorsement.method` its `method`. `endorsement.identityCommitment` is copied from the card, and `endorsement.cardDigestMultibase` is the card digest defined below. The statement **MUST** carry an `id`, which is what a vetter later names to withdraw it.

The statement below is the one Carol issues for the session in this document's examples. Its `taskDigestMultibase` and `cardDigestMultibase` are the real values for the documents as printed.

```json
{
  "@context": ["https://www.w3.org/ns/credentials/v2", "https://firstperson.network/credentials/dtg/v1"],
  "id": "urn:uuid:7e5d3c1b-9f8a-4b6c-a2d1-e0f9a8b7c601",
  "type": ["VerifiableCredential", "DTGCredential", "EndorsementCredential"],
  "issuer": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
  "validFrom": "2026-09-17T15:09:00Z",
  "validUntil": "2027-01-15T15:09:00Z",
  "taskContext": "urn:uuid:9a7e4c21-5b3d-4e8f-a1c2-3d4e5f6a7b01",
  "taskDigestMultibase": "zQmWAWEtpUqE3xd3LUpZ8GryGrZMCqH1A5D7bZcayvEfJTK",
  "credentialSubject": {
    "id": "did:webvh:QmAliceScid1:alice.example",
    "endorsement": {
      "type": "https://firstperson.network/endorsements/identity-vetting/0.1",
      "community": "did:webvh:QmVtcScid:kernel-vtc.example",
      "method": "video",
      "documentClasses": ["passport"],
      "claimsVerified": ["name.legal"],
      "livenessConfirmed": true,
      "identityCommitment": "zQmT7GFcSjCY7YwuK5RP3TNNF8wp7fnCfMMYjMeatbbWo7b",
      "cardDigestMultibase": "zQmYn4rU7vALWT8K9nC4vD8EcSZXFh8ZrCBDFUXgXo2pcH5",
      "declaredRelationship": "community-colleague",
      "attestationTextDigest": "zQmappH2ogZX2jVEmxitPLHKtA82EmxJjkPWo3szBErmNFj"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmCarolScid1:kernel-vtc.example:carol#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z5k2pxtz3XrdADnsNJ1XQiZ4oj7XHVqTamscwe3Wir6JjjNKp6mJZZTmynBW32NBmWCxjs5g8Xmjck9ZLfPKtU5G4"
  }
}
```

## The match code

A card proves that whoever controls the join DID signed it. It does not prove that this person is the one in front of the vetter. The **match code** closes that gap. Both agents derive the same eight characters from this document's `id`, and the two people read them to each other:

```
digest = SHA-256( "vetting-session-match/v1" || 0x00 || UTF-8(id) )
code   = the first 40 bits of digest, most significant first, as eight
         Crockford base32 characters (0123456789ABCDEFGHJKMNPQRSTVWXYZ),
         written XXXX-XXXX
```

`id` is this document's `id` exactly as it appears, and the domain tag is the 24 ASCII bytes shown followed by one zero byte. For the session in the examples below, the code is `DREC-TG1C`.

When the codes match, the person on the call is driving the agent that received this session, and that agent holds the join DID's key. A vetter **MUST NOT** set the statement's `livenessConfirmed` to `true` unless both people read the same code. An applicant's agent **SHOULD** display the code before the card leaves it, and answers `vetting/session:matchCodeMismatch` when its person reports a different one. Whether that agent also asks its person to confirm before sending the card is the agent's policy. Per [SPEC §7.3](/SPEC.md#73-specification-requirements) item 13, this specification does not require it.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **vetter** (`issuer`):

1. Opens a session only for a request it accepted, naming that request's `requestId`, addressed to that request's `joinDid`.
2. **MUST** generate `challenge` as 32 bytes from a cryptographically secure source, fresh for this session and never reused, and **MUST** set `domain` to the request's `community`.
3. **SHOULD** set `requiredClaims` to exactly the claim types the community's criterion requires. The identity commitment is computed over them, and a session that asks for different ones produces a commitment no other vetter's statement will match.
4. **SHOULD** set `expiresAt` close to `issuedAt`. No more than 15 minutes is **RECOMMENDED**, because the session exists for one sitting.
5. On receiving a card, **MUST** check all of the following before showing it to a person, and **MUST NOT** issue a statement on a card that fails any of them:
   - the card's `proof` verifies under an `assertionMethod` key of its `publisher`;
   - `publisher` equals the request's `joinDid` and the response's `issuer`, and `audience` is the vetter's own DID;
   - `community`, `challenge` and `domain` equal this session's values;
   - the card has not expired, and its `expiresAt` is no later than this session's;
   - every type in `requiredClaims` is present;
   - `identityCommitment` recomputes from `commitmentSalt` and the claims.
6. **MUST NOT** capture or store any detail of the documentation it inspects, and **MUST NOT** keep the card beyond its own published retention period.

A conforming **applicant** (`recipient`):

1. Answers only a session from the vetter that accepted `requestId`, and otherwise returns `vetting/session:unknownRequest`.
2. Builds the card as follows, then signs it with an `assertionMethod` key of the join DID:
   - `type` is `VerifiableDataStructure`, `RelationshipCard` and `VettingCard`;
   - `publisher` is its join DID, and `audience` is the session's `issuer`;
   - `community` and `domain` are the session's `domain`, and `challenge` is the session's `challenge`;
   - `expiresAt` is no later than the session's;
   - `claims` include every type in `requiredClaims`, and no type outside `requiredClaims` and `optionalClaims`;
   - `identityCommitment` is computed as defined below, with the application's salt.
3. **MUST NOT** put a document number, a document image, a portrait, or any other detail of identity documentation on the card.
4. Returns `vetting/session:claimUnavailable` rather than a card missing a required claim.

### Identity commitment

```
identityCommitment = multibase( multihash( SHA-256( JCS( { "salt": commitmentSalt, "claims": R } ) ) ) )
```

`R` holds `{ "type", "value" }` for each card claim whose type is in the session's `requiredClaims`, ordered by `type` and then by the JCS serialization of `value`. `commitmentSalt` is 32 random bytes, base64url-encoded, generated **once per application** and reused on every card of it. Every vetter of the application therefore sees, and attests, the same commitment. The community can then check that all the statements it counts are about the same claimed identity without learning that identity, because the salt never reaches it.

### Card digest

`cardDigestMultibase` is the SHA-256 multihash, multibase-encoded, of the RFC 8785 canonicalization of the card **exactly as the vetter received it, `proof` included**. It identifies one delivered card, so a re-signed card with the same claims is a different card.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authority to open a session is the **accepted request**. The vetter that accepted `requestId` from this applicant may open sessions under it, and nobody else may. An applicant refuses any other with `vetting/session:unknownRequest`. Nothing else in the request carries authority. Its REQUIRED `proof` makes the challenge attributable to the vetter, which matters for what the statement cites. It does not entitle the vetter to a card.

A card is the applicant's own act. It authorizes nothing: it is evidence, which the vetter weighs against a person and a document. The authority that makes a resulting statement *count* is not in this exchange at all. It is the community's, applied when it decides, to a statement from a vetter it made eligible.

Per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10, verifying either proof establishes who sent a document, never that its recipient must act on it.

## Definitions

**Vetting Card** — a profile of the relationship card: a signed data structure, published by the applicant's join DID and bound to one vetter and one session, carrying the claims the vetter checks. Its schema is [`vetting/_shared/0.1/vetting-card`](../../_shared/0.1/vetting-card.schema.json).

**Match code** — the eight characters both agents derive from this document's `id`, read aloud to establish that the person present drives the agent that holds the join DID.

**Identity commitment** — a salted digest over the required claims, equal across one application's cards and statements.

**Commitment salt** — the per-application random value that makes the commitment unguessable to anyone who has not seen a card.

## Request

The vetter sends the session to the applicant. See the top-level schema in [`payload.schema.json`](payload.schema.json).

### Opening a video session

```json
{
  "id": "urn:uuid:9a7e4c21-5b3d-4e8f-a1c2-3d4e5f6a7b01",
  "type": "https://trusttasks.org/spec/vetting/session/0.1",
  "threadId": "urn:uuid:9a7e4c21-5b3d-4e8f-a1c2-3d4e5f6a7b01",
  "parentThreadId": "urn:uuid:6f1c2b0a-3d4e-4f5a-8b6c-7d8e9f0a1b01",
  "issuer": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
  "recipient": "did:webvh:QmAliceScid1:alice.example",
  "issuedAt": "2026-09-17T15:02:00Z",
  "payload": {
    "requestId": "urn:uuid:4b2e8f10-7a6c-4d3b-9e21-0f5a6b7c8d01",
    "challenge": "Xq3v9bT0cN2mR8sLk4Jw7pYh1eZa6uGd5fQi0oVxWnE",
    "domain": "did:webvh:QmVtcScid:kernel-vtc.example",
    "method": "video",
    "requiredClaims": ["name.legal"],
    "optionalClaims": ["account.handle"],
    "expiresAt": "2026-09-17T15:17:00Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmCarolScid1:kernel-vtc.example:carol#key-1",
    "created": "2026-09-17T15:02:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z63jiSzsVJshBfyZwcr6nUopHo5M1QnBnWJHtwTpdNEFeD7KoX5rezJcGeoY8AVuTSo5Q3uH2KqMoEZk68qqGu3AR"
  }
}
```

## Response

The applicant, now responding, returns the signed card, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` with this specification's codes.

### The Vetting Card

The `identityCommitment` shown is the real value for the salt and the `name.legal` claim as printed.

```json
{
  "id": "urn:uuid:9a7e4c21-5b3d-4e8f-a1c2-3d4e5f6a7b02",
  "type": "https://trusttasks.org/spec/vetting/session/0.1#response",
  "threadId": "urn:uuid:9a7e4c21-5b3d-4e8f-a1c2-3d4e5f6a7b01",
  "parentThreadId": "urn:uuid:6f1c2b0a-3d4e-4f5a-8b6c-7d8e9f0a1b01",
  "issuer": "did:webvh:QmAliceScid1:alice.example",
  "recipient": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
  "issuedAt": "2026-09-17T15:04:06Z",
  "payload": {
    "card": {
      "type": ["VerifiableDataStructure", "RelationshipCard", "VettingCard"],
      "id": "urn:uuid:d2c4e6f8-1a3b-4c5d-8e7f-9a0b1c2d3e01",
      "publisher": "did:webvh:QmAliceScid1:alice.example",
      "cardVersion": 1,
      "audience": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
      "community": "did:webvh:QmVtcScid:kernel-vtc.example",
      "challenge": "Xq3v9bT0cN2mR8sLk4Jw7pYh1eZa6uGd5fQi0oVxWnE",
      "domain": "did:webvh:QmVtcScid:kernel-vtc.example",
      "issuedAt": "2026-09-17T15:04:05Z",
      "expiresAt": "2026-09-17T15:17:00Z",
      "claims": [
        { "type": "name.legal", "value": "Alice Example", "provenance": "selfAsserted" },
        { "type": "account.handle", "value": "alice@example.org", "provenance": "selfAsserted" }
      ],
      "identityCommitment": "zQmT7GFcSjCY7YwuK5RP3TNNF8wp7fnCfMMYjMeatbbWo7b",
      "commitmentSalt": "mT7rK2pWq9Zc4Vb8Nx1Ls6Hd3Fg0Jy5Ra7Ue2Io4PkA",
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:webvh:QmAliceScid1:alice.example#key-1",
        "proofPurpose": "assertionMethod",
        "proofValue": "z46F4qaaxCkXUkQQMGK1gk1efa54xT3GDw9GByQu4cHZmr86Vw1frjBcGALCAQjPTua6J3YtcQQbgDFCuJnhczu3F"
      }
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmAliceScid1:alice.example#key-1",
    "created": "2026-09-17T15:04:06Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "zuoTZbVpBUvRSPXq7An9rfxFFqnbWLSGZbLpkF3utZnGwyk6yNDqmuUQX2qQdre7PBBpHMWoatui2Y41nMCwuUqM"
  }
}
```

## Security & Privacy

### Data carried

The request carries binding material — a challenge and a domain — plus claim *types* and a method. Nothing in it describes the applicant.

The response carries the most personal data in the vetting exchange: the applicant's claimed legal name and any other claims they chose to show, under their own signature. It is the smallest disclosure that lets a vetter do the job. The required claims are what the community counts. Optional claims are the applicant's choice, and they never affect whether a statement counts.

What the card never carries is the documentation itself. A vetter looks at a passport; the passport does not travel. There is no member for a document number, an image or a portrait, and `person.portrait` is refused as a claim type. A vetter's agent that offers to photograph a document, or stores what the vetter saw, defeats the design.

`commitmentSalt` travels only to vetters, inside the card. A vetter can use it to test the commitment, but that vetter has already seen the claims it covers. The community receives statements, never cards, and never the salt.

### Correlation

The applicant and the vetter declare `identifierScope: public` for the reasons [`vetting/request`](../../request/0.1/spec.md) gives. One join DID has to connect every card and statement of an application, and statements have to be attributable to a member.

The identity commitment is the deliberate join key of this design. It is equal across every card and statement of one application, which is what lets a community see that its vetters checked the same claimed identity. It links nothing across applications, because each application draws a fresh salt. It reveals nothing to anyone without the salt, because the salt is 256 bits and the claim values are low-entropy exactly where an unsalted digest would be guessable.

The challenge and `id` are fresh per session, and the match code is derived from `id`, so none of them links two sessions. Two vetters of one application who compare notes can link their sessions. They hold the same claims about the same join DID anyway.

### Retention

Two lifetimes meet in this exchange.

**This document is durable.** A statement cites its `id` and its task digest, and a verifier who wants to check that pairing needs the document the digest was taken over. The applicant **SHOULD** retain it for as long as it holds the statement, and **SHOULD** submit it with the statement if the community asks.

**The card is not.** The vetter needs it only long enough to check it and issue or decline. It **SHOULD** delete the card promptly afterwards — within days, not months — and keep at most its digest, which the statement already carries. The applicant's agent records that it disclosed the card, to whom and when, which is the applicant's own evidence of where their identity went.

### Consent/purpose

The applicant discloses the card to one vetter, to have one claimed identity checked for one community's application. A vetter **MUST NOT** reuse the claims for anything else — a contact list, a note to the community, a record of who it has met. The community's decision rests on statements, and giving it a card would move the applicant's legal name somewhere the applicant did not send it. Whether an agent requires a step-up, or asks its person, before it signs and sends a card is that agent's policy. Per [SPEC §7.3](/SPEC.md#73-specification-requirements) item 13, this specification does not decide it.
