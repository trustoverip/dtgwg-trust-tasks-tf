---
slug: vtc/community/theme/show
version: "0.1"
title: "VTC Community — Theme — Show"
summary: Read the stylesheet a community's own pages — its administration console and its member portal — are drawn with.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [theme, stylesheet, css, branding, appearance]
parties:
  - role: anyone loading the community's pages
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
    The theme is drawn on the community's sign-in pages, which a visitor loads before they hold any identity the community knows. It is presentation the community shows to everyone who loads those pages, so there is nothing a proof would protect, and requiring one would leave the sign-in page unthemed.
sideEffects:
  level: none
  rationale: >-
    Reads one configuration row of the community. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses community configuration — the stylesheet its administrators chose for its own pages. No personal data about any member, applicant or administrator.
retention:
  class: transient
  rationale: >-
    A read. The community keeps nothing, and the caller's copy is stale at the next update.
errorCodes: []
related:
  - vtc/community/theme/update
  - vtc/community/branding/show
---

## Abstract

The **VTC Community — Theme — Show** Trust Task returns the community's theme: one stylesheet, in the restricted theme profile defined by [`vtc/community/theme/update/0.1`](../../update/0.1/spec.md#the-theme-profile), that the community's own pages — its administration console and its member portal, sign-in pages included — apply over their built-in look.

A theme is presentation only. It cannot load anything or add content, and — by the client requirements below — cannot reach any surface on which someone decides, approves, consents or is warned, not even through a design token.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MAY** answer a caller it cannot identify. The theme is drawn on pages the community serves to anyone, so withholding it from an unidentified caller protects nothing.
2. **MUST** return the stored stylesheet exactly as [`vtc/community/theme/update`](../../update/0.1/spec.md) stored it, and **MUST** return `stylesheet: ""` — not an error — when none has been set.
3. **MUST NOT** return a stylesheet outside the theme profile. A community that finds a stored stylesheet it can no longer accept — after a change to its own checks, or a restore — **MUST** answer the empty string and **SHOULD** record that it did.

A conforming **client** — a page that applies the theme:

1. **MUST** check the stylesheet against the theme profile before applying it, and **MUST** apply none of it when any part fails. A community is trusted to have checked; a page that renders what it was given without checking lets a defect in either one reach every administrator.
2. **MUST** render every surface on which a person decides, approves, declines, consents, signs or is warned — a consent or approval prompt, a step-up, a destructive confirmation, a security or single-administrator banner — in isolation from the theme, so that no stylesheet in the profile can change how that surface is drawn. Isolating by structure (a shadow tree, a separate document) satisfies this, provided the isolated surface declares every custom property it reads inside the isolation — custom properties inherit into a shadow tree, so a surface that reads an inherited token is not isolated from it. Isolating by selector does not satisfy this, because no selector list can anticipate every rule that might match.
3. **MUST** apply the theme beneath its own styles for those surfaces and **MUST NOT** honour a theme on any page that is not the community's own.
4. **SHOULD** show, somewhere the theme cannot reach, that a custom theme is in force.

## Definitions

- **`stylesheet`** — CSS text in the [theme profile](../../update/0.1/spec.md#the-theme-profile). The empty string means no theme: the pages draw with their built-in look.
- **`updatedAt`** — when the stylesheet was last replaced. Absent when the community never set one.

## Request

Anyone loading the community's pages (`issuer`, OPTIONAL) sends an empty payload to the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A sign-in page loads the theme

No issuer and no proof: the visitor has no identity yet.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000101",
  "type": "https://trusttasks.org/spec/vtc/community/theme/show/0.1#request",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-17T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000101ff",
  "payload": {}
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### The stored theme

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000102",
  "type": "https://trusttasks.org/spec/vtc/community/theme/show/0.1#response",
  "issuer": "did:example:community",
  "issuedAt": "2026-10-17T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000101ff",
  "payload": {
    "stylesheet": ":root { --brand: #0a7f5c; --radius-md: 10px; }\n@media (prefers-color-scheme: dark) { :root { --brand: #3fc29a; } }\n.card { border-radius: 14px; }",
    "updatedAt": "2026-10-16T08:30:00Z"
  }
}
```

### A community that never set one

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000103",
  "type": "https://trusttasks.org/spec/vtc/community/theme/show/0.1#response",
  "issuer": "did:example:community",
  "issuedAt": "2026-10-17T10:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000101fe",
  "payload": {
    "stylesheet": ""
  }
}
```

## Security & Privacy

### Data carried

Community configuration only: CSS an administrator wrote. It is untrusted input to every client that draws it, which is why the client checks it again (client item 1) and keeps it away from every surface where a person is asked to decide something (client item 2). Without that isolation one administrator could restyle another's approval prompt — hide its Decline button, or the banner saying the community runs with a single administrator — and the second-party consent the prompt exists for would be decided by the stylesheet's author.

The profile admits no construct that fetches anything (`url()`, `@import`, `@font-face`), so drawing a theme discloses nothing about the visitor to anyone, the community included, beyond the read itself.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the theme is its own. A caller need not identify itself, and an unidentified read reveals only that someone loaded the community's pages, which loading them reveals anyway.

### Retention

Nothing is kept by the community. A client **MAY** cache the stylesheet for as long as it caches the pages it themes; a stale theme is cosmetic.

### Consent/purpose

The purpose is to draw the community's own pages the way its administrators chose. Applying the stylesheet to any other page is outside it (client item 3).
