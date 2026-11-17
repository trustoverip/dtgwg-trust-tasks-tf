---
slug: vtc/community/logo/show
version: "0.1"
title: "VTC Community — Logo — Show"
summary: Read the logo a community holds — its digest, media type, size, dimensions and the URL it is served at, and on request the bytes themselves.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [logo, branding, image, appearance]
parties:
  - role: anyone drawing the community's mark
    requirement: OPTIONAL
    member: issuer
    identifierScope: any
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: OPTIONAL
  rationale: >-
    The logo is drawn on the community's sign-in and join pages, which a visitor loads before they hold any identity the community knows. It is the community's public mark, shown to everyone, so there is nothing a proof would protect, and requiring one would leave those pages without a logo.
maxDocumentBytes:
  response: 360448
  rationale: >-
    With `includeData`, the response carries the stored logo, at most 262144 bytes, which base64url-encodes to 349526 characters; the remaining 10 KiB covers the envelope, the reference and a proof — the same bound as vtc/community/logo/update's request and vtc/directory/images/get's response.
sideEffects:
  level: none
  rationale: >-
    Reads the stored logo. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses community configuration — the image its administrators chose as its mark, and where it is served. No personal data about any member, applicant or administrator.
retention:
  class: transient
  rationale: >-
    A read. The community keeps nothing, and the caller's copy is stale at the next update.
errorCodes: []
related:
  - vtc/community/logo/update
  - vtc/community/branding/show
  - vtc/community/profile/show
---

## Abstract

The **VTC Community — Logo — Show** Trust Task returns the logo a community holds, as stored by [`vtc/community/logo/update`](../../update/0.1/spec.md): its digest, media type, size and pixel dimensions, and the `url` the community serves it at. A page drops the `url` into an `<img>`; a client that cannot fetch over HTTP, or that wants the bytes over the same channel it already speaks, sets `includeData: true` and gets them inline.

It is how any client — not only the pages the community ships — finds the community's logo without knowing in advance where the community serves it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MAY** answer a caller it cannot identify. The logo is drawn on pages the community serves to anyone, so withholding it from an unidentified caller protects nothing.
2. **MUST** answer the `LogoRef` of the logo it holds — the same members, `url` included, that [`vtc/community/logo/update`](../../update/0.1/spec.md) answered when it stored it — and **MUST** answer `logo: null`, not an error, when it holds none.
3. **MUST**, when the request sets `includeData: true` and it holds a logo, return the stored bytes in `data`, exactly the bytes whose SHA-256 is `logo.sha256`; and **MUST NOT** return `data` otherwise.
4. **MUST** include `url` whenever it serves the logo over HTTP ([`vtc/community/logo/update`](../../update/0.1/spec.md#conformance) item 9).

A conforming **client** — a page or application that draws the logo:

1. **SHOULD** draw the logo from `url` where it can fetch over HTTP, and ask for `includeData` only where it cannot: the URL is cacheable and the inline bytes are not.
2. **SHOULD** compare `sha256`, not `url`, to tell whether the logo changed since it last drew it: the community **SHOULD** keep the URL the same across updates.
3. **MUST**, when it takes the bytes from `data`, check that their SHA-256 is `logo.sha256` and that they decode as `logo.mediaType`, and draw nothing when either fails.
4. **MUST** draw the logo as an image only — never as a document, and never in a way that lets it run anything.

## Definitions

- **`includeData`** — whether to return the stored bytes inline as well as the reference.
- **`logo`** — the `LogoRef` the community holds, or `null`: the SHA-256, media type, size and pixel dimensions of the stored re-encoding, and the `url` it is served at.
- **`url`** — the absolute `https` URL at which the community serves the stored bytes to anyone, without a credential. Present whenever it serves the logo over HTTP.
- **`data`** — the stored bytes, base64url without padding. Present only when asked for.
- **`updatedAt`** — when the stored logo was last replaced or removed. Absent when the community has never held one.

## Request

Anyone drawing the community's mark (`issuer`, OPTIONAL) sends the community (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A sign-in page asks where the logo is

No issuer and no proof: the visitor has no identity yet.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000401",
  "type": "https://trusttasks.org/spec/vtc/community/logo/show/0.1#request",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-16T12:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000401ff",
  "payload": {}
}
```

### A mobile wallet asks for the bytes over the channel it already has

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000403",
  "type": "https://trusttasks.org/spec/vtc/community/logo/show/0.1#request",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-16T12:01:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000403ff",
  "payload": {
    "includeData": true
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/community/logo/show/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). A refusal is a `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)).

### The stored logo

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000402",
  "type": "https://trusttasks.org/spec/vtc/community/logo/show/0.1#response",
  "issuer": "did:example:community",
  "issuedAt": "2026-11-16T12:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000401ff",
  "payload": {
    "logo": {
      "sha256": "3a7bd3e2360a3d29eea436fcfb7e44c735d117c42d1c1835420b6b9942dd4f1b",
      "mediaType": "image/png",
      "size": 18342,
      "width": 512,
      "height": 256,
      "url": "https://community.example/logo"
    },
    "updatedAt": "2026-11-16T10:00:01Z"
  }
}
```

### The stored logo with its bytes

The bytes are abbreviated; real ones are up to 349526 characters.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000404",
  "type": "https://trusttasks.org/spec/vtc/community/logo/show/0.1#response",
  "issuer": "did:example:community",
  "issuedAt": "2026-11-16T12:01:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000403ff",
  "payload": {
    "logo": {
      "sha256": "3a7bd3e2360a3d29eea436fcfb7e44c735d117c42d1c1835420b6b9942dd4f1b",
      "mediaType": "image/png",
      "size": 18342,
      "width": 512,
      "height": 256,
      "url": "https://community.example/logo"
    },
    "data": "iVBORw0KGgoAAAANSUhEUgAAAgAAAAEACAYAAAA",
    "updatedAt": "2026-11-16T10:00:01Z"
  }
}
```

### A community that holds no logo

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000405",
  "type": "https://trusttasks.org/spec/vtc/community/logo/show/0.1#response",
  "issuer": "did:example:community",
  "issuedAt": "2026-11-16T12:02:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000405ff",
  "payload": {
    "logo": null
  }
}
```

## Security & Privacy

### Data carried

Community configuration only: the image an administrator chose as the community's mark, already re-encoded and stripped of metadata by the community when it was stored ([`vtc/community/logo/update`](../../update/0.1/spec.md#data-carried)), and where it is served. The bytes are still untrusted input to a client, which is why a client checks them against the digest and media type before drawing them, and draws them only as an image (client items 3 and 4).

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the logo is its own public mark. A caller need not identify itself, and an unidentified read reveals only that someone is drawing the community's mark, which fetching `url` reveals anyway.

### Retention

Nothing is kept by the community. A client **MAY** cache the logo for as long as its `sha256` is unchanged; a stale logo is cosmetic.

### Consent/purpose

The purpose is to draw the community's mark wherever the community is shown — on its own pages and in any client that represents it.
