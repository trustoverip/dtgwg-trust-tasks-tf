---
slug: vtc/directory/images/get
version: '0.1'
title: VTC Directory — Images — Get
summary: Read the bytes of an image on a directory entry — an avatar or an image field — answered only where the caller's projection of that entry shows it.
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- directory
- avatar
- image
- projection
parties:
- role: reader
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: Who may see an image is decided by the projection, which depends on who asks; an attributable caller gets the answer their standing allows. Without a proof the caller is anyone, and only an image on a published external card is answered.
sideEffects:
  level: none
  rationale: Reads an image; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Returns a picture a member put on their card, to a caller their visibility choice admits.
retention:
  class: transient
  rationale: A read; nothing is kept beyond the audit record of it.
maxDocumentBytes:
  response: 360448
  rationale: An image is stored at no more than 262144 bytes (vtc/directory/images/upload), which base64url-encodes to 349526 characters; the remaining 10 KiB covers the envelope, the reference and a proof.
errorCodes:
- code: vtc/directory/images/get:notFound
  meaning: No image the caller may see is at that entry and field — or not the one `sha256` names. Conflates an unknown entry, a field that holds no image, a value the caller's projection withholds, and an image since replaced.
  retryable: false
related:
- vtc/directory/images/upload
- vtc/directory/entries/show
- vtc/directory/card/public
---

## Abstract

The **VTC Directory — Images — Get** Trust Task returns the bytes of a picture on a directory entry. A projection carries an image as a `CardBlobRef` — a digest, a media type and a size — and a reader that wants to show it asks for it here, naming the entry and the field it saw it in.

The community answers exactly when the caller's projection of that entry shows that field, by the same rule every read uses: the member's chosen audience, the field's ceiling and the community's directory policy, intersected (VTI-DIR-020). A member-only avatar is therefore never readable by a stranger, and there is no URL for it that could leak. Images are never read by digest alone: a digest is not a capability.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **reader** (`issuer`) names the entry and field it was shown the image in, and **SHOULD** name the `sha256` it was shown, so it never displays a picture under a card that no longer holds it. A reader **MUST** check that the SHA-256 of the decoded `data` equals `image.sha256` before showing it.

A conforming **community** (`recipient`) **MUST**:

1. Project the entry for the caller exactly as [`vtc/directory/entries/show`](../../../entries/show/0.1/spec.md) does — a member's or administrator's standing read now, a key delegated under [`auth/oob/grant/0.2`](../../../../../auth/oob/grant/0.2/spec.md) read as its member — and answer only when that projection carries `field` with an image value.
2. Treat a caller with no proof, or one who is neither an active member nor an administrator, as the public: answer only an image that is a value of the member's published external card ([`vtc/directory/card/publish`](../../../card/publish/0.1/spec.md)).
3. Where `sha256` is present, answer only when the value is that image.
4. Answer every refusal with `notFound`, whatever its reason, so a reader cannot tell a withheld picture from a missing one (VTI-DIR-110).
5. Return the stored bytes and their `CardBlobRef`, without `url` or `urlExpiresAt`.
6. **SHOULD** bound the rate at which one caller reads images, as it bounds browsing (VTI-DIR-114).

## Authorization

**Authority: the projection, decided now.** The caller's identity matters only as an input to the projection — who they are, what groups and relationships they share with the subject. Nothing here grants access beyond what [`entries/show`](../../../entries/show/0.1/spec.md) would show the same caller. The proof, where present, identifies the caller ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `CardBlobRef`, `EntryId`, `FieldKey` — see [`directory-card.schema.json`](../../../_shared/0.1/directory-card.schema.json).

## Request

The reader (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member reads another member's avatar

```json
{
  "id": "urn:uuid:9b4e6f2a-1c3d-4e5f-8a7b-000000000011",
  "type": "https://trusttasks.org/spec/vtc/directory/images/get/0.1",
  "issuer": "did:example:dave",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-19T09:05:00Z",
  "payload": {
    "entryId": "dir_Q2xhcmFRMmxoY21GUk1t",
    "field": "avatar",
    "sha256": "3a7bd3e2360a3d29eea436fcfb7e44c735d117c42d1c1835420b6b9942dd4f1b"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:dave#key-1",
    "created": "2026-10-19T09:05:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/directory/images/get/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)) with the code declared above.

### The avatar

The bytes are abbreviated.

```json
{
  "id": "urn:uuid:9b4e6f2a-1c3d-4e5f-8a7b-000000000012",
  "type": "https://trusttasks.org/spec/vtc/directory/images/get/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:dave",
  "issuedAt": "2026-10-19T09:05:01Z",
  "threadId": "urn:uuid:9b4e6f2a-1c3d-4e5f-8a7b-000000000011",
  "payload": {
    "image": {
      "sha256": "3a7bd3e2360a3d29eea436fcfb7e44c735d117c42d1c1835420b6b9942dd4f1b",
      "mediaType": "image/webp",
      "size": 18342
    },
    "data": "UklGRm5HAABXRUJQVlA4IGJH"
  }
}
```

## Security & Privacy

### Data carried

The request names an entry and a field; the response carries the image. The entry identifier reveals nothing about the member's DID (VTI-DIR-050), and a reader can only have it from a projection that showed it, or from an external card's registry pointer.

### Correlation

The digest is of the community's re-encoding, so it links nothing across communities ([`images/upload`](../../upload/0.1/spec.md)). Reads go to the community, never to a third-party host, so showing a picture tells no one outside the community who looked.

The community maintainer's identifier scope is public: a reader addresses the community whose directory showed it the entry, by the DID it already knows it by; a pairwise identifier per reader would make the community differ by who asks.

### Retention

Nothing beyond the audit record of a read, which names the entry and field, never the image.

### Consent/purpose

A picture is shown to the audience its member chose for it, and only to them. Serving images through the projection, rather than from a URL anyone holding it can fetch, is what keeps that choice true after the picture has been shown once.
