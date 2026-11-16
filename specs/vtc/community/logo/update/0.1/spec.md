---
slug: vtc/community/logo/update
version: "0.1"
title: "VTC Community — Logo — Update"
summary: An administrator hands the community a new logo, which it re-encodes, strips of metadata and serves from its own origin — or removes the one it holds.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [logo, branding, image, appearance]
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The write changes the image the community shows every administrator, member and visitor, and is authorized only by the signer's administrator standing — so the signer has to be attributable on every transport, and the change has to be attributable afterwards in the audit trail.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed update would silently put back a logo an administrator has since replaced or removed, so a duplicate must be placeable in a bounded window.
maxDocumentBytes:
  request: 360448
  rationale: >-
    An image is at most 262144 bytes, which base64url-encodes to 349526 characters; the remaining 10 KiB covers the envelope, the media type and a proof — the same derivation as vtc/directory/images/upload, so a logo never needs a chunked transfer over a 1 MiB mediator message. The response carries a digest, four small members and a URL, and stays within the framework default.
sideEffects:
  level: mutating
  rationale: >-
    Replaces or removes one stored image. Recoverable by another update; nothing is issued, revoked or deleted beyond the image itself.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries an image the administrator chose to represent the community; the response is its digest, media type, size, dimensions and the URL the community serves it at. No personal data is expected — a logo is the community's own mark.
retention:
  class: durable
  rationale: >-
    The stored image is the community's configuration until the next update, and the change is recorded in the audit trail against the administrator who made it.
errorCodes:
  - code: vtc/community/logo/update:tooLarge
    meaning: '`data` decodes to more than 262144 bytes, or to a picture larger than the community decodes — more pixels, or more memory to decode, than its bound. Nothing was stored.'
    retryable: false
  - code: vtc/community/logo/update:notAnImage
    meaning: '`data` does not decode as an image of `mediaType`, or as an image the community can re-encode. Nothing was stored.'
    retryable: false
  - code: vtc/community/logo/update:rateLimited
    meaning: The administrator the signer resolves to has sent as many updates as the community allows in its current window. Nothing was stored.
    retryable: true
related:
  - vtc/community/logo/show
  - vtc/community/branding/update
  - vtc/community/profile/update
  - vtc/community/theme/update
  - vtc/directory/images/upload
---

## Abstract

The **VTC Community — Logo — Update** Trust Task is how a community gets a logo it serves itself. An administrator sends an image; the community **re-encodes** it — dropping every metadata block, capping its dimensions and choosing the stored format — stores it in place of the logo it held, and answers the stored image's digest, media type, size, dimensions and the URL it serves it at. Sending `remove: true` instead of an image removes the stored logo.

A community's pages are commonly served under a content security policy that admits images from their own origin only. A logo held at some other address then cannot be drawn on them at all, and a community that has no other place on its own origin to put a file has no logo. The community serving the logo it stored is what this task exists for. Which path a community serves it at is its own choice, but it is never something a client has to know in advance: the community names it as the logo's `url` in this task's response and in [`vtc/community/logo/show`](../../show/0.1/spec.md), which anyone may send to learn the logo the community holds. The requirements on serving are in [item 9](#conformance).

This task stores an image. It does not change the profile's `logoUrl` ([`vtc/community/profile/update`](../../../profile/update/0.1/spec.md)), which remains an address a community may publish for readers that fetch from anywhere, nor the `logoUrl` in the join manifest ([`vtc/community/branding/update`](../../../branding/update/0.1/spec.md)), which a prospective applicant's client fetches. A community that holds a stored logo and either address decides which its own pages draw; a page served under a policy that admits images from its own origin only can draw the stored logo and no other.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC community-logo design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-community-logo.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller that does not hold administrator standing with `permissionDenied`, after resolving the proof's signer — or, for a signer that is a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.2/spec.md)), the identity it acts for — to its access-control entry at execution time. It **MUST** make this check before decoding `data`: decoding is the expensive step, and is not a service for anyone who asks.
2. **MUST** refuse `data` that decodes to more than 262144 bytes with `vtc/community/logo/update:tooLarge` before decoding it as an image; and **MUST** bound what it decodes — the pixels a picture may claim and the memory its decoding may take — refusing a picture beyond either with `tooLarge`. A few kilobytes can claim tens of thousands of pixels across, and the byte bound alone does not stop them. The payload schema already bounds `data` to 262144 bytes, so a longer `data` fails validation with `malformedRequest`, and in practice it is the decoding bound that refuses with `tooLarge`.
3. **MUST** decode `data` as an image of `mediaType` and no other type, never by sniffing its content, refusing with `vtc/community/logo/update:notAnImage` what does not decode as that type.
4. **MUST** re-encode the image: every metadata block removed (EXIF, XMP, ICC profiles, text chunks, comments) and anything after the image's end discarded, the longer side capped at a bound of the community's choosing no larger than 512 pixels, and stored as one of the `LogoMediaType`s. It **MUST** re-encode deterministically — the same decoded picture always yields the same stored bytes, with no timestamp or other varying content written into them — so that sending the same image twice is recognizably a no-op (item 7). It **MUST NOT** store or serve the bytes it received. A community **SHOULD** store a logo in a lossless format: logos are mostly flat color and sharp edges, which lossy compression visibly degrades.
5. **MUST** bound how many updates one administrator may send in a window, refusing beyond it with `vtc/community/logo/update:rateLimited`. The bound **MUST** be counted against the identity item 1 resolves the signer to, not the signing key, so that enrolling more delegated keys does not raise it; it counts every request that passes item 1, including one that changes nothing; and it **MUST** be applied before decoding `data`, for the reason item 1 gives.
6. **MUST** refuse a request that carries neither `image` nor `remove: true`, or both, as `malformedRequest` — the payload schema does — and **MUST NOT** read a missing or `null` image as removal.
7. **MUST** record each change in its audit trail, naming the administrator and the digest of the stored image (never the image), or that the logo was removed, and **MUST** refuse with `unavailable` rather than make an unaudited change when it cannot write the audit record. An update that stores the image already stored, or removes a logo the community does not hold, changes nothing, is not audited, and answers `stored: false`.
8. **MUST** answer with the logo it now holds — `stored: true` when this request changed it.
9. Where it serves the stored logo over HTTP, **MUST** name the address as the logo's `url` — an absolute `https` URL — in every `LogoRef` it answers, here and in `vtc/community/logo/show`; **SHOULD** keep that URL the same across updates, so a page can embed it once; **MUST** serve exactly the stored bytes there with the stored media type, **MUST** send `X-Content-Type-Options: nosniff`, and **MUST NOT** require a session or credential or set a cookie to do so: the logo is drawn on pages a visitor sees before signing in. It **SHOULD** send a validator (an `ETag` derived from the digest) and a content security policy that lets the response run nothing if opened directly.

A conforming **client** — an editor that sends this task:

1. **SHOULD** scale an image down before sending it: the community keeps no more than 512 pixels on the longer side.
2. **MUST** rasterize a vector image (SVG) itself before sending it, as one of the `LogoMediaType`s. It **SHOULD** draw it from a self-contained source in an environment where its script does not run and it fetches nothing: an image drawn into a canvas, rather than a document inserted into a page.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry, read at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

A logo confers no authority and needs no other administrator's consent. The same standing already sets the profile's `logoUrl`, which every client draws as the community's mark; storing the image rather than naming its address changes where the bytes live, not who may choose them. A logo is drawn inside a box its page fixes, so it cannot move, cover or restyle the surfaces where a person decides something.

## Definitions

- **`image`** — the new logo: a `LogoImage` of `mediaType` and `data`. Absent when the request removes the logo.
- **`remove`** — `true` to remove the stored logo. A request carries exactly one of `image` and `remove`; removal is never an absent or `null` image, so a request whose image was never filled in fails validation rather than deleting the logo.
- **`mediaType`** — the media type of `data` as sent, one of the `LogoMediaType`s. The community decodes `data` as this type only.
- **`data`** — the image, base64url without padding, at most 262144 bytes decoded.
- **`logo`** — the `LogoRef` the community holds after the request, or `null`: the SHA-256, media type, size and pixel dimensions of the stored re-encoding, and the `url` it serves it at.
- **`url`** — the absolute `https` URL at which the community serves the stored bytes to anyone, without a credential. Present whenever it serves the logo over HTTP. A client draws the logo from it and compares `sha256` to tell whether the logo changed.
- **`stored`** — whether this request changed the stored logo.
- **`updatedAt`** — when the stored logo was last replaced or removed.

## Request

A community administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### An administrator sends a new logo

The image is abbreviated; a real one is up to 349526 characters.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000301",
  "type": "https://trusttasks.org/spec/vtc/community/logo/update/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-16T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000301ff",
  "payload": {
    "image": {
      "mediaType": "image/png",
      "data": "iVBORw0KGgoAAAANSUhEUgAAAgAAAAEACAYAAAA"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:administrator#key-1",
    "created": "2026-11-16T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

### An administrator removes the logo

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000303",
  "type": "https://trusttasks.org/spec/vtc/community/logo/update/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-16T11:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000303ff",
  "payload": {
    "remove": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:administrator#key-1",
    "created": "2026-11-16T11:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/community/logo/update/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json): the logo it now holds and whether this request changed it. Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)) with the codes declared above.

### Stored, re-encoded as PNG at 512 × 256

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000302",
  "type": "https://trusttasks.org/spec/vtc/community/logo/update/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-16T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000301ff",
  "payload": {
    "logo": {
      "sha256": "3a7bd3e2360a3d29eea436fcfb7e44c735d117c42d1c1835420b6b9942dd4f1b",
      "mediaType": "image/png",
      "size": 18342,
      "width": 512,
      "height": 256,
      "url": "https://community.example/logo"
    },
    "stored": true,
    "updatedAt": "2026-11-16T10:00:01Z"
  }
}
```

### Removed

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000304",
  "type": "https://trusttasks.org/spec/vtc/community/logo/update/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-16T11:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000303ff",
  "payload": {
    "logo": null,
    "stored": true,
    "updatedAt": "2026-11-16T11:00:01Z"
  }
}
```

### Refused — a picture that claims more pixels than the community decodes

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000306",
  "type": "https://trusttasks.org/spec/trust-task-error/0.6",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-16T10:05:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000305ff",
  "payload": {
    "code": "vtc/community/logo/update:tooLarge",
    "message": "the picture is larger than the community decodes",
    "retryable": false,
    "inResponseTo": {
      "typeUri": "https://trusttasks.org/spec/vtc/community/logo/update/0.1#request",
      "id": "urn:uuid:00000000-0000-4000-8000-000000000305"
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries an image the administrator chose as the community's mark. It is untrusted input to the community, and three things make it safe to serve from the community's own origin, where whatever it serves runs with that origin's standing:

- **Only raster types.** SVG is excluded by the schema, not by a check a consumer might skip: an SVG is a document that can carry script, styles and references to other resources, and served from the community's origin and opened directly it would run there. A client rasterizes one before sending it (client item 2).
- **Decode bounds before decoding** (item 2). A small file can claim enormous dimensions, so the byte bound alone does not bound the memory decoding takes.
- **Re-encoding** (items 3 and 4). The stored bytes are the community's own output from decoded pixels, so a file that is valid as an image and as something else — a polyglot with HTML or script after the image data — does not survive, nor does metadata an editor or camera embedded. Decoding strictly as the declared type, and serving with the stored type and `nosniff` (item 9), keep any client from reading the bytes as something else.

The response carries a digest, a media type, a size and dimensions. The audit record carries the digest, not the image, so the audit trail does not grow by up to 256 KiB a change and still identifies exactly what was stored.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the logo is its own public mark, served to anyone. The audit trail ties each change to the administrator who made it; that attribution is the point of requiring a proof. Re-encoding makes the digest the community's own, so it does not match the file the administrator sent or the same file elsewhere.

### Retention

The stored image lives until the next update. The audit record of a change is durable and names the administrator and the digest. A community **MUST NOT** keep the bytes it received, or a digest of them, after re-encoding, since either would link the upload to the same file elsewhere — the property [Correlation](#correlation) relies on.

### Consent/purpose

The purpose is to show the community's mark on its own pages and wherever its public profile is read. A logo is public by design: nothing an administrator would not publish belongs in one.
