---
slug: vtc/directory/images/upload
version: '0.1'
title: VTC Directory — Images — Upload
summary: Hand the community an image for a directory card — an avatar or an image field — which it re-encodes, strips of metadata and stores, answering the reference a card then holds.
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- directory
- avatar
- image
parties:
- role: member or directory administrator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: Bytes are stored inside the community on the strength of who sent them, and the image may later be shown on that writer's card only. The writer must be attributable on every upload.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed upload stores nothing new, but bounding it keeps a captured document from renewing an image's erasure deadline.
sideEffects:
  level: mutating
  rationale: Stores one image. Nothing on any card changes; an image no card holds is erased at `expiresAt`.
exposure:
  discloses: none
  ingests: personal
  actsAsSubject: true
  rationale: A picture is personal data the writer chooses to hand over. The response is a digest, a media type and a size.
retention:
  class: durable
  rationale: Kept while a card holds it, and erased with the card's value. An image no card holds is erased at `expiresAt`, at most 24 hours after upload.
maxDocumentBytes:
  request: 360448
  rationale: An image is at most 262144 bytes, which base64url-encodes to 349526 characters; the remaining 10 KiB covers the envelope, the media type and a proof. 262144 is the chunk ceiling vtc/website/upload derives for a document to survive a 1 MiB mediator message after DIDComm encoding and two forward wrappers, so no image ever needs a chunked transfer.
errorCodes:
- code: vtc/directory/images/upload:notMember
  meaning: The signer is neither an active member nor a holder of the directory administration capability. Nothing was stored.
  retryable: false
- code: vtc/directory/images/upload:tooLarge
  meaning: '`data` decodes to more than 262144 bytes. Nothing was stored.'
  retryable: false
- code: vtc/directory/images/upload:notAnImage
  meaning: '`data` does not decode as an image of `mediaType`, or as an image the community can re-encode. Nothing was stored.'
  retryable: false
- code: vtc/directory/images/upload:rateLimited
  meaning: The signer has uploaded as many images as the community allows in its current window. Nothing was stored.
  retryable: true
related:
- vtc/directory/images/get
- vtc/directory/card/update
- vtc/directory/card/assert
---

## Abstract

The **VTC Directory — Images — Upload** Trust Task is how a picture gets onto a directory card. A member sends the image for their avatar, or for a community field of type `image`; an administrator sends one for an `admin`-sourced image field. The community **re-encodes** it — dropping every metadata block (EXIF, XMP, ICC profiles, comments), capping its dimensions, and choosing the stored format — and answers a `CardBlobRef`: the SHA-256 of the stored bytes, their media type and size.

The upload names no card and no field. An image belongs to nobody until a card holds it: the member then sets the reference as the field's value with [`vtc/directory/card/update`](../../../card/update/0.1/spec.md), or an administrator with [`vtc/directory/card/assert`](../../../card/assert/0.1/spec.md), and it is shown as that field is — to the audience its member chose, up to the field's ceiling. An image no card takes up is erased.

Re-encoding is what makes the digest safe to show. The same photograph uploaded to two communities, re-encoded by each, has two digests, so a digest a reader sees links nothing across communities; and an image stripped of metadata carries no camera serial, location or editing history into the directory.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** is an active member, signing with their membership DID or with a key they delegated for this community under [`auth/oob/grant/0.2`](../../../../../auth/oob/grant/0.2/spec.md) whose `signingScope` covers `vtc/directory`; or a holder of the directory administration capability. A producer **SHOULD** scale an image down before sending it: the community keeps no more than it needs to show.

A conforming **consumer** (the community) **MUST**:

1. Refuse a signer who is neither an active member nor a directory administrator with `notMember`.
2. Refuse `data` that decodes to more than 262144 bytes with `tooLarge`, before decoding it as an image.
3. Decode `data` as an image of `mediaType`, refusing with `notAnImage` what does not decode, and re-encode it: every metadata block removed, its dimensions capped at a bound of the community's choosing, no larger than 1024 pixels on the longer side, and stored as one of the `ImageMediaType`s. It **MUST NOT** store or serve the bytes it received.
4. Answer the `CardBlobRef` of the stored bytes — `sha256` of exactly those bytes, their `mediaType` and `size` — and the `expiresAt` by which a card must hold it, no later than 24 hours after the upload.
5. Record who uploaded the image. [`vtc/directory/card/update`](../../../card/update/0.1/spec.md) **MUST** accept a `CardBlobRef` value only for an image the same member uploaded and that has not been erased, or the one that field already holds; [`vtc/directory/card/assert`](../../../card/assert/0.1/spec.md) only for one a directory administrator uploaded. Anything else is that task's `valueInvalid`. A digest seen on someone else's card is never a way to put their picture on your own.
6. Erase an image when no card holds it any longer — at `expiresAt` for one never taken up, and when the value holding it is replaced, cleared, retired or erased with its card.
7. Bound how many images one signer may upload in a window, refusing beyond it with `rateLimited`.
8. Audit the upload by reference — the signer and the digest, never the image.

## Authorization

**Authority: standing in the community, read now.** An active member may upload, because every member may put a picture on their own card; a directory administrator may, because administrators write `admin`-sourced image fields. An upload confers no right to any card: which card may hold the image is decided at [`card/update`](../../../card/update/0.1/spec.md) or [`card/assert`](../../../card/assert/0.1/spec.md), by item 5. The proof establishes who signed ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `CardBlobRef`, `ImageMediaType` — see [`directory-card.schema.json`](../../../_shared/0.1/directory-card.schema.json).

## Request

The member or administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member uploads an avatar

The image is abbreviated; a real one is up to 349526 characters.

```json
{
  "id": "urn:uuid:9b4e6f2a-1c3d-4e5f-8a7b-000000000001",
  "type": "https://trusttasks.org/spec/vtc/directory/images/upload/0.1",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-19T09:00:00Z",
  "payload": {
    "mediaType": "image/jpeg",
    "data": "_9j_4AAQSkZJRgABAQAAAQABAAD"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:carol#key-1",
    "created": "2026-10-19T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/directory/images/upload/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)) with the codes declared above.

### Stored, re-encoded as WebP

```json
{
  "id": "urn:uuid:9b4e6f2a-1c3d-4e5f-8a7b-000000000002",
  "type": "https://trusttasks.org/spec/vtc/directory/images/upload/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-19T09:00:01Z",
  "threadId": "urn:uuid:9b4e6f2a-1c3d-4e5f-8a7b-000000000001",
  "payload": {
    "image": {
      "sha256": "3a7bd3e2360a3d29eea436fcfb7e44c735d117c42d1c1835420b6b9942dd4f1b",
      "mediaType": "image/webp",
      "size": 18342
    },
    "expiresAt": "2026-10-20T09:00:01Z"
  }
}
```

The member then puts it on their card with [`vtc/directory/card/update`](../../../card/update/0.1/spec.md), as `"values": { "avatar": { "sha256": "3a7b…4f1b", "mediaType": "image/webp", "size": 18342 } }`.

## Security & Privacy

### Data carried

The request carries an image: personal data, possibly a face, chosen by the writer. The response carries a digest, a media type and a size. Nothing about who will see the image travels in either direction: that is the card's visibility, decided by its member.

### Correlation

Re-encoding makes the stored bytes, and so the digest, the community's own: an image uploaded to two communities yields two unrelated digests (VTI-DIR-051), and the metadata a camera or editor embeds — device serials, coordinates, timestamps — is gone before anything is stored. A community **MUST NOT** keep the bytes it received, or a digest of them, after re-encoding, since either would link the upload to the same image elsewhere.

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and its trust registry already know it by, and a pairwise identifier per caller would make the community differ by who asks.

### Retention

An image lasts as long as a card holds it, and no longer than 24 hours when none does. Departure erases a member's images with their card under the departure disposition (VTI-DIR-121).

### Consent/purpose

An image is uploaded to be shown on the writer's card, or for administrators on the card of the member it describes, to the audience that card's member chooses. It is not used for anything else: not matched across members, not scanned, not shown outside the projection.
