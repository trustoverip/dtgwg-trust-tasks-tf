---
slug: vtc/badges/images/get
version: '0.1'
title: VTC Badges — Images — Get
summary: Read the bytes of a badge's picture, answered only to an active member who may see the badge.
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- badges
- gamification
- image
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
  rationale: The picture of a badge that may not be shown outside is shown to active members only (VTI-BDG-050), and which badges a member may see depends on who asks. Without a proof the caller is nobody, and nothing is answered.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: A read with no side effects; a replay discloses only what the caller was already entitled to see.
sideEffects:
  level: none
  rationale: Reads an image; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Returns the picture of a badge, to a member who may see the badge.
retention:
  class: transient
  rationale: A read; nothing is kept beyond the audit record of it.
maxDocumentBytes:
  response: 360448
  rationale: An image is stored at no more than 262144 bytes (vtc/badges/images/upload), which base64url-encodes to 349526 characters; the remaining 10 KiB covers the envelope, the reference and a proof.
errorCodes:
- code: vtc/badges/images/get:notFound
  meaning: No badge the caller may see has this identifier — or its picture is not the one `sha256` names. Conflates an unknown badge, one the caller may not see, badges being off, a caller who is not an active member, and a picture since replaced.
  retryable: false
related:
- vtc/badges/images/upload
- vtc/badges/show
- vtc/badges/wall/show
- vtc/directory/images/get
---

## Abstract

The **VTC Badges — Images — Get** Trust Task returns the bytes of a badge's picture. A badge carries its picture as a `CardBlobRef` — a digest, a media type and a size — and a member that wants to show it, on a wall, in the badge list or behind a holder list, asks for it here, naming the badge it saw it on.

The community answers exactly when the caller is an active member who may see the badge, by the one rule every badge read uses: an active badge that is `listed`, or that the caller holds, owns, or has an open offer of; every badge, retired ones included, to a holder of `vtc.badges.admin`; a retired badge to a holder of it. A picture is therefore never readable by a stranger, and there is no URL for it that could leak (VTI-BDG-050). Images are never read by digest alone: a digest is not a capability. A badge whose holders may show it outside also has its picture at a stable public location its credential names (VTI-BDG-051); that location is served outside this task, and this task still answers members only.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC badges design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-badges.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **reader** (`issuer`) names the badge it was shown the picture on, and **SHOULD** name the `sha256` it was shown, so it never displays a picture under a badge that no longer holds it. A reader **MUST** check that the SHA-256 of the decoded `data` equals `image.sha256` before showing it.

A conforming **consumer** (the community) **MUST**:

1. Answer as if no badge existed while its badge settings have `enabled` false (VTI-BDG-004).
2. Answer only an active member, signing with their membership DID or with a key they delegated for this community under [`auth/oob/grant/0.3`](../../../../../auth/oob/grant/0.3/spec.md) whose `signingScope` covers `vtc/badges`, read as its member; and only for a badge that member may see — an active badge that is `listed`, or that they hold, own or have an open offer of; any badge to a holder of `vtc.badges.admin`; a retired badge to a holder of it. Anyone and anything else is `notFound`.
3. Where `sha256` is present, answer only when the badge's picture is that image.
4. Answer every refusal with `notFound`, whatever its reason, so a reader cannot tell a withheld picture from a missing one.
5. Return the stored bytes and their `CardBlobRef`, without `url` or `urlExpiresAt`.
6. **SHOULD** bound the rate at which one caller reads images.
7. Audit the read by reference — the caller and the badge, never the image (VTI-BDG-091).

## Authorization

**Authority: the badge's visibility to the caller, decided now.** The caller's identity matters only as an input to that rule — whether they are an active member, what they hold, own and are offered, whether they administer badges. Nothing here grants access beyond what [`vtc/badges/show`](../../../show/0.1/spec.md) would show the same caller. The proof identifies the caller ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

A read is an act of the member (VTI-BDG-095): a member portal holding a key the member authorized for `vtc/badges` **MAY** sign it.

## Definitions

- `BadgeId`, `Badge` — see [`badge.schema.json`](../../../_shared/0.1/badge.schema.json).
- `CardBlobRef` — see the directory family's [`directory-card.schema.json`](../../../../directory/_shared/0.1/directory-card.schema.json).

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member reads a badge's picture to show a wall

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000301",
  "type": "https://trusttasks.org/spec/vtc/badges/images/get/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-06T10:00:00Z",
  "payload": {
    "badgeId": "bdg_W5k2N8xRb3Qz7TpL",
    "sha256": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:carol#key-1",
    "created": "2026-11-06T10:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/badges/images/get/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### The picture

The bytes are abbreviated.

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000302",
  "type": "https://trusttasks.org/spec/vtc/badges/images/get/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-11-06T10:00:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000301",
  "payload": {
    "image": {
      "sha256": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
      "mediaType": "image/png",
      "size": 61440
    },
    "data": "iVBORw0KGgoAAAANSUhEUgAABAAAAAQA"
  }
}
```

### Refused: a badge the caller may not see

```json
{
  "id": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000303",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-11-06T10:00:01Z",
  "threadId": "urn:uuid:3b9d6c1e-2f4a-4b8c-9d0e-000000000301",
  "payload": {
    "code": "vtc/badges/images/get:notFound",
    "message": "No badge bdg_W5k2N8xRb3Qz7TpL.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request names a badge and, optionally, a digest; the response carries the picture. A badge's picture is artwork, not personal data; the badge identifier reveals nothing about who holds it. A producer **MUST NOT** place personal data in `ext`.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the badge's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The digest is of the community's re-encoding, so it links nothing across communities ([`images/upload`](../../upload/0.1/spec.md)). Reads go to the community, never to a third-party host, so showing a picture tells no one outside the community who looked; the audit record of a read names the caller and the badge, which the community already knows the caller may see.

### Retention

**Transient.** Nothing beyond the audit record of a read, which names the caller and the badge, never the image.

### Consent/purpose

A badge's picture is shown to the members who may see the badge, and only to them, so that walls, lists and holder lists can show it. Serving it through this task, rather than from a URL anyone holding it can fetch, is what keeps an unlisted badge, and every badge of a community that does not let its badges out, inside the community (VTI-BDG-050).
