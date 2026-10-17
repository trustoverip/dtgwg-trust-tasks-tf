---
slug: rooms/keys/key-package
version: "0.2"
title: Rooms Keys — Key-Package
summary: "An invited party asks the agent that will hold its room keys for the MLS KeyPackage the room's owner adds — the one it already holds for that room, member and invitation, or a new one — so an acceptance begun in one surface can be finished in another."
status: draft
targetFrameworkVersion: "0.6.0"
category: ai-agents
keywords:
  - room
  - mls
  - keypackage
  - membership
  - unlinkability
  - idempotency
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: KeyHolder
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "The mint retains a private key on the recipient's behalf; a request whose origin depended on the transport would let a compromised channel fill a key-holder with unusable key material."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A request that can mint retains key material; `issuedAt` bounds how old a request the recipient will act on."
sideEffects:
  level: mutating
  rationale: "Mints an MLS KeyPackage and retains its private half against a Welcome that may never arrive — unless one is already outstanding for the room, member and invitation, which is returned and nothing changes."
subjectPath: /roomId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries the room's invitation credential, which names its subject and is not a secret. The KeyPackage carries a signature key, an HPKE init key and the leaf's credential. It opens nothing; what it discloses is that this party is preparing to join something."
retention:
  class: durable
  rationale: "The private half must outlive the request — it is what processes the Welcome — but only until the KeyPackage is used or expires."
errorCodes:
  - code: rooms/keys/key-package:notInvited
    meaning: "The invitation does not verify, was not issued by the room `roomId` names, or has expired. The recipient mints nothing without a valid invitation to the room."
    retryable: false
related:
  - rooms/keys/welcome
  - rooms/keys/commit
  - rooms/keys/act
  - vtc/rooms/offers/accept
  - vtc/rooms/memberships/list
---

## Abstract

The **Rooms Keys — Key-Package** Trust Task obtains the MLS KeyPackage a room's owner needs in order to add a member to the group.

It is the first of the three steps that put a group into a key-holding agent — obtain a KeyPackage, be [welcomed](../../welcome/0.1/spec.md), then keep applying [commits](../../commit/0.1/spec.md) — and the only one the joining side initiates.

From this version a KeyPackage is **outstanding** from the moment it is minted until it is used or expires, and a request for the same room, member and invitation while it is outstanding returns it rather than minting another. That makes the whole of joining repeatable: every step answers a repeat with what it answered the first time, so any surface the member uses can finish an acceptance any other began.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

### What changed from 0.1

`0.1` minted a new KeyPackage for every request (*Single use*: "a second request mints a second package"). Joining a room the community owns is three exchanges — this mint, the acceptance [`vtc/rooms/offers/accept`](../../../../vtc/rooms/offers/accept/0.1/spec.md), and the Welcome — and the member makes them from a surface: a member portal through their wallet, a terminal client, a phone. When a surface stopped after the acceptance and before the Welcome, the community had admitted the KeyPackage and kept the Welcome for it; but any other surface, and the same one after losing its state, could only mint a **different** KeyPackage, which the community answers `alreadyAccepted`. The member was admitted to a room no surface could finish joining, and only an administrator could repair it.

`0.2`:

- **Returns the outstanding KeyPackage** for the same room, member and invitation (*One outstanding KeyPackage per room, member and invitation*), with its original `expiresAt`, and says so in the new `alreadyMinted`.
- **Requires `invitation`**, which `0.1` left optional with a SHOULD: it authorizes the mint, names the member, and is part of what identifies the outstanding KeyPackage.
- **Requires `expiresAt`** in the response, which `0.1` left optional.

What a repeated request returns changes, a request member becomes required, and so do two response members: changes with wire effect, so a new version per [SPEC §5.2](/SPEC.md#52-compatibility-rules) rather than an in-place edit to `0.1` — a `MINOR` increment.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

### Recipient

A conforming **recipient** (the key holder):

1. **MUST** refuse with `notInvited` a request whose invitation does not verify, was not issued by the room `roomId` names, or has expired.
2. **MUST** take the member from the invitation's subject, and **MUST** refuse with the standard `permissionDenied` ([SPEC §8.3](/SPEC.md#83-standard-error-codes)) when its caller may not act as that member. A caller to which the member has delegated narrower standing does not act as the member.
3. **MUST**, while it holds an outstanding KeyPackage it minted for the same room, the same member and the same invitation, return that KeyPackage — the same bytes, and so the same `KeyPackageRef` ([RFC 9420 §5.2](https://www.rfc-editor.org/rfc/rfc9420#section-5.2)) — with the `expiresAt` it gave when it minted it and `alreadyMinted: true`, and **MUST NOT** mint another.
4. **MUST** otherwise mint a new KeyPackage for the room whose leaf credential identifies the member, and answer it with `alreadyMinted: false`.
5. **MUST** treat a KeyPackage as outstanding from when it is minted until the first of: it processes a Welcome for it ([`rooms/keys/welcome`](../../welcome/0.1/spec.md)), or `expiresAt` passes. It **MUST** discard the private half at whichever comes first, and **MUST NOT** return a KeyPackage that is no longer outstanding.
6. **MUST** identify an invitation by the credential itself — its identifier where the governing profile gives one, its digest otherwise — and never by what it says about its subject or room alone. A second invitation to the same room for the same member, issued after a removal and re-admission, is a different invitation and is given a different KeyPackage: its predecessor's leaf was in the group and was removed from it.
7. **MUST** answer concurrent requests for the same room, member and invitation with the same KeyPackage.

### Producer

A conforming **producer** (a member's surface):

1. **MUST** send the KeyPackage it is given, unaltered, to the room's owner — in an offer of a community-owned room, as the `keyPackage` of [`vtc/rooms/offers/accept`](../../../../vtc/rooms/offers/accept/0.1/spec.md), completed as the member with [`rooms/keys/act`](../../act/0.5/spec.md).
2. **SHOULD**, when `alreadyMinted` is `true`, expect that another surface may have sent it already, and that the owner's answer to it is a repeat of the one it gave that surface.

## Behaviour

### One KeyPackage per room, never reused across rooms

A KeyPackage is a stable, public identifier for whoever presents it. The same one offered to two rooms tells anyone who sees both that one party is in both — which is precisely the linkage a `private` room exists to deny, arriving through the door rather than the wall.

So `roomId` is required and a recipient **MUST** mint per room. Reuse is not an optimisation worth having: a KeyPackage is small, minting is cheap, and correlation is forever.

### One outstanding KeyPackage per room, member and invitation

Returning the outstanding KeyPackage is not reuse in that sense. It goes to the same room's owner, for the same member, against the same invitation — the one party that already saw it, or would have. Nothing is linked that was not linked by the invitation.

What it buys is that the three steps of joining each answer a repeat with their first answer:

1. the key holder returns the KeyPackage it already holds (Recipient item 3);
2. the room's owner, asked again to admit the same KeyPackage, answers with the Welcome it already made ([`vtc/rooms/offers/accept`](../../../../vtc/rooms/offers/accept/0.1/spec.md), *Repeating an accept*; [`rooms/owner/group/add`](../../../owner/group/add/0.2/spec.md) item 8);
3. the key holder, handed a Welcome for a group it already joined for the member, answers `alreadyJoined` ([`rooms/keys/welcome`](../../welcome/0.1/spec.md)).

A surface that runs all three, from the start, finishes whatever any surface began — and cannot tell, and need not, whether one did. [`vtc/rooms/memberships/list`](../../../../vtc/rooms/memberships/list/0.1/spec.md) sets out when a surface does so.

### The private half is retained, so the mint is not free

The recipient keeps the private key that will process the Welcome. A caller that mints and never joins has left key material behind. In `0.1` a caller that minted repeatedly left a pile of it; here a repeat returns the outstanding one, so a recipient holds at most one private half per room, member and invitation.

A recipient **SHOULD** bound how long it retains an unused KeyPackage — `expiresAt` says what it chose — and that bound is also how long an acceptance begun elsewhere can be finished. A community keeps its Welcome for a bounded time too; a recipient **SHOULD NOT** expire a KeyPackage sooner than the communities it serves keep theirs, or the Welcome outlives the key that opens it.

### Single use

MLS consumes a KeyPackage on add. A recipient **MUST NOT** return a KeyPackage once it has processed a Welcome for it (Recipient item 5): the next request for the room — after a removal, under a new invitation — mints a new one.

## Request

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/rooms/keys/key-package/0.2#request",
  "issuer": "did:example:client",
  "recipient": "did:example:keyholder",
  "issuedAt": "2026-10-17T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "roomId": "did:webvh:example.com:rooms:board",
    "invitation": "eyJhbGciOi…"
  }
}
```

## Response

### The KeyPackage another surface already obtained

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/rooms/keys/key-package/0.2#response",
  "issuer": "did:example:keyholder",
  "recipient": "did:example:client",
  "issuedAt": "2026-10-17T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "keyPackage": "AAEAAgADAAQ",
    "expiresAt": "2026-10-24T09:57:00Z",
    "alreadyMinted": true
  }
}
```

## Security & Privacy

**Nothing here opens anything.** A KeyPackage is public by construction — signature key, HPKE init key, credential. It is safe to hand to the owner, and safe for the owner's transport to see. What it is not safe to do is *reuse* it across rooms, which is a privacy property rather than a confidentiality one and so is easy to get wrong while everything appears to work.

**Returning it again opens nothing either.** A caller that may act as the member could have obtained the KeyPackage the first time; obtaining it again gives it the same public value. Only the key holder can process the Welcome sealed to it, so the surface that receives the KeyPackage the second time gains no more than the one that received it first.

**Minting is not joining.** A party that mints has consented to nothing. Consent is the invitation, and it becomes load-bearing at the Welcome. An implementation that treated a mint as acceptance would have collapsed the two-party act the room family is built on into a one-party one.

### Data carried

A room identifier and an invitation credential. The response carries a public KeyPackage, its expiry, and whether it was minted for this request.

### Correlation

A KeyPackage is a correlator by nature: it is a stable value that identifies its holder to anyone who sees it twice. Per-room minting confines that to one room, and returning the outstanding one keeps it there: it goes again only to the owner of the room it was minted for, against the invitation that owner issued.

### Retention

The private half is retained until the KeyPackage is used or expires, and **MUST** be discarded at whichever comes first. Retaining it past expiry keeps a key alive for a join that will never happen. The recipient also retains which room, member and invitation each outstanding KeyPackage was minted for, for as long as it retains the private half, and no longer.

### Consent/purpose

Minting is not joining, and the separation is the point. The purpose here is to produce something the owner can add; a party that mints has agreed to nothing beyond being addable. Consent to be in the room is the invitation, and it becomes load-bearing at the Welcome.

The expiry is a purpose bound rather than housekeeping. A KeyPackage minted so one room's owner can add one member is spent when that happens or when the window closes, and the private half goes with it.
