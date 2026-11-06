---
slug: vtc/badges/images/upload
version: '0.1'
title: VTC Badges — Images — Upload
summary: Hand the community the picture for a badge, which it re-encodes, strips of metadata and stores, answering the reference a badge then holds.
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- gamification
- image
parties:
- role: badge administrator, owner or member
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: Bytes are stored inside the community on the strength of who sent them, and only that signer may put the picture on a badge. The writer must be attributable on every upload.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed upload stores nothing new, but bounding it keeps a captured document from renewing an image's erasure deadline.
sideEffects:
  level: mutating
  rationale: Stores one image. No badge changes; an image no badge holds is erased at `expiresAt`.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: A badge picture is artwork the community will show under the badge's name, not a picture of the signer. The response is a digest, a media type and a size.
retention:
  class: durable
  rationale: Kept while a badge holds it, for the badge's life. An image no badge holds is erased at `expiresAt`, at most 24 hours after upload.
maxDocumentBytes:
  request: 360448
  rationale: An image is at most 262144 bytes, which base64url-encodes to 349526 characters; the remaining 10 KiB covers the envelope, the media type and a proof. 262144 is the chunk ceiling vtc/website/upload derives for a document to survive a 1 MiB mediator message after DIDComm encoding and two forward wrappers, so no image ever needs a chunked transfer.
errorCodes:
- code: vtc/badges/images/upload:badgesOff
  meaning: The community has badges turned off (VTI-BDG-004). Nothing was stored.
  retryable: false
- code: vtc/badges/images/upload:notMember
  meaning: The signer is neither an active member who may make a kudos badge or who owns a badge, nor a holder of `vtc.badges.admin`. Nothing was stored.
  retryable: false
- code: vtc/badges/images/upload:tooLarge
  meaning: '`data` decodes to more than 262144 bytes, or to a picture larger than the community decodes — more pixels, or more memory to decode, than its bound. Nothing was stored.'
  retryable: false
- code: vtc/badges/images/upload:notAnImage
  meaning: '`data` does not decode as an image of `mediaType`, or as an image the community can re-encode. Nothing was stored.'
  retryable: false
- code: vtc/badges/images/upload:rateLimited
  meaning: The signer has uploaded as many images as the community allows in its current window. Nothing was stored.
  retryable: true
related:
- vtc/badges/images/get
- vtc/badges/create
- vtc/badges/update
- vtc/badges/promote
- vtc/directory/images/upload
---

## Abstract

The **VTC Badges — Images — Upload** Trust Task is how a picture gets onto a badge. An administrator sends the picture for a community badge they are about to define; a member sends the one for a kudos badge they are about to make; an owner or maker sends a new one for a badge they are changing. The community **re-encodes** it — dropping every metadata block (EXIF, XMP, ICC profiles, comments), capping its dimensions, and choosing the stored format — and answers a `CardBlobRef`: the SHA-256 of the stored bytes, their media type and size (VTI-BDG-050).

The upload names no badge. An image is **yours until a badge holds it**: the signer then names its digest in [`vtc/badges/create`](../../../create/0.1/spec.md), [`vtc/badges/update`](../../../update/0.1/spec.md) or [`vtc/badges/promote`](../../../promote/0.1/spec.md), each of which accepts a digest only from the same signer's upload that no badge has yet taken up. A picture a badge holds is kept while the badge exists and read by members with [`vtc/badges/images/get`](../../get/0.1/spec.md); one no badge takes up is erased.

Re-encoding is what makes the digest safe to show. The same artwork uploaded to two communities, re-encoded by each, has two digests, so a digest a reader sees links nothing across communities; and an image stripped of metadata carries no device serial, location or editing history into the community. Where a badge may be shown outside, its picture is also published at a stable public location its credential names (VTI-BDG-051); that location is served outside this task.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** is a holder of `vtc.badges.admin`, or an active member who may make a kudos badge or who owns a badge, signing with their membership DID or, for a member, with a key they delegated for this community under [`auth/oob/grant/0.3`](../../../../../auth/oob/grant/0.3/spec.md) whose `signingScope` covers `vtc/badges`. A producer **SHOULD** scale an image down before sending it: the community keeps no more than it needs to show.

A conforming **consumer** (the community) **MUST**:

1. Refuse every upload with `badgesOff` while its badge settings have `enabled` false (VTI-BDG-004).
2. Refuse with `notMember` a signer who is not a holder of `vtc.badges.admin`, nor an active member the settings' `kudosMakers` admit as a maker (within `limits.minTenureDays`), nor an active member who made a kudos badge or owns a community badge (manages its badge group).
3. Refuse `data` that decodes to more than 262144 bytes with `tooLarge`, before decoding it as an image; and bound what it decodes — the pixels a picture may claim and the memory its decoding may take — refusing a picture beyond either with `tooLarge`. A few kilobytes can claim tens of thousands of pixels across, and the byte bound alone does not stop them. The payload schema already bounds `data` to 262144 bytes, so in practice it is the decoding bound that refuses.
4. Decode `data` as an image of `mediaType`, refusing with `notAnImage` what does not decode, and re-encode it: every metadata block removed, its dimensions capped at a bound of the community's choosing, no larger than 1024 pixels on the longer side, and stored as one of the `ImageMediaType`s. It **MUST NOT** store or serve the bytes it received (VTI-BDG-050).
5. Answer the `CardBlobRef` of the stored bytes — `sha256` of exactly those bytes, their `mediaType` and `size` — and the `expiresAt` by which a badge must hold it, no later than 24 hours after the upload.
6. Record who uploaded the image. [`vtc/badges/create`](../../../create/0.1/spec.md), [`vtc/badges/update`](../../../update/0.1/spec.md) and [`vtc/badges/promote`](../../../promote/0.1/spec.md) **MUST** accept an `image.sha256` only for an image the same signer uploaded, not erased and not yet taken up by a badge, or the one that badge already holds. Anything else is that task's `imageNotFound`. A digest seen on someone else's badge is never a way to put their picture on your own.
7. Keep an image while a badge holds it, for the badge's life, retired included (VTI-BDG-005); erase one when no badge holds it any longer — at `expiresAt` for one never taken up, and when the badge's picture is replaced.
8. Where a badge that may be shown outside holds the image, publish it at the stable public location the badge's credential names (VTI-BDG-051), through a surface outside this task, and **MUST NOT** publish the picture of any other badge.
9. Bound how many images one signer may upload in a window, refusing beyond it with `rateLimited`.
10. Audit the upload by reference — the signer and the digest, never the image (VTI-BDG-091).

## Authorization

**Authority: standing in the community, read now.** A holder of `vtc.badges.admin` may upload, because administrators define community badges; a member the settings admit as a kudos maker may, because they are about to make one; a badge's owner or a kudos badge's maker may, because they change its picture. An upload confers no right to any badge: which badge may hold the image is decided at [`create`](../../../create/0.1/spec.md), [`update`](../../../update/0.1/spec.md) or [`promote`](../../../promote/0.1/spec.md), by item 6. The proof establishes who signed ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

A member's upload is an act of the member (VTI-BDG-095): a member portal holding a key the member authorized for `vtc/badges` **MAY** sign it. An administrator's upload, for a community badge, **MUST NOT** be accepted from such a key.

## Definitions

- `CardBlobRef`, `ImageMediaType` — see the directory family's [`directory-card.schema.json`](../../../../directory/_shared/0.1/directory-card.schema.json).
- `Badge` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).

## Request

The badge administrator, owner or member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member uploads the picture for a kudos badge

The image is abbreviated; a real one is up to 349526 characters.

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000201",
  "type": "https://trusttasks.org/spec/vtc/badges/images/upload/0.1#request",
  "issuer": "did:example:tomas",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T08:55:00Z",
  "payload": {
    "mediaType": "image/png",
    "data": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAAB"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:tomas#key-1",
    "created": "2026-11-06T08:55:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/images/upload/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Stored, re-encoded as WebP

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000202",
  "type": "https://trusttasks.org/spec/vtc/badges/images/upload/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:tomas",
  "issuedAt": "2026-11-06T08:55:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000201",
  "payload": {
    "image": {
      "sha256": "2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae",
      "mediaType": "image/webp",
      "size": 18342
    },
    "expiresAt": "2026-11-07T08:55:01Z"
  }
}
```

The member then names it in [`vtc/badges/create`](../../../create/0.1/spec.md), as `"image": { "sha256": "2c26…e7ae" }`.

### Refused: a member the settings do not admit as a maker

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000203",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:tomas",
  "issuedAt": "2026-11-06T08:55:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000201",
  "payload": {
    "code": "vtc/badges/images/upload:notMember",
    "message": "Kudos badges are made by members of the crew group; you are not in it.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request carries an image: artwork for a badge, chosen by the writer. The response carries a digest, a media type and a size. A producer **MUST NOT** upload a picture of a person as a badge: a badge's picture is shown to every member who may see the badge and, where it may be shown outside, to the world, and nothing in this family treats it as personal data. A producer **MUST NOT** place personal data in `ext`.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. Re-encoding makes the stored bytes, and so the digest, the community's own: an image uploaded to two communities yields two unrelated digests, and the metadata a camera or editor embeds — device serials, coordinates, timestamps — is gone before anything is stored. A community **MUST NOT** keep the bytes it received, or a digest of them, after re-encoding, since either would link the upload to the same image elsewhere. Where the badge may be shown outside, its picture is public (VTI-BDG-051); who uploaded it, and who holds the badge, are not.

### Retention

**Durable.** An image lasts as long as a badge holds it — for the badge's life, retired included — and no longer than 24 hours when none does. The record of who uploaded it is kept with the audit of the upload.

### Consent/purpose

An image is uploaded to be the picture of a badge the writer is about to define, make or change, shown to the members who may see that badge and, only where the community marked the badge as one its holders may show outside, published. It is not used for anything else: not matched across badges or members, not scanned, not shown for a badge the writer did not put it on.
