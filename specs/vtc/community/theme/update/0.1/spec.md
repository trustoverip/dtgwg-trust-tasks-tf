---
slug: vtc/community/theme/update
version: "0.1"
title: "VTC Community — Theme — Update"
summary: An administrator replaces the stylesheet the community's console and member portal are drawn with — or checks one without storing it.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [theme, stylesheet, css, branding, appearance]
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
    The write changes how the community's pages look to every administrator, member and visitor, and is authorized only by the signer's administrator standing — so the signer has to be attributable on every transport, and the change has to be attributable afterwards in the audit trail.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed update would silently put back a theme an administrator has since replaced, so a duplicate must be placeable in a bounded window.
maxDocumentBytes:
  request: 49152
  response: 49152
  rationale: >-
    A stylesheet of up to 32768 bytes, JSON-escaped (newlines and quotes roughly add a third in a typical stylesheet), plus envelope and proof headroom. Both variants carry the stylesheet.
sideEffects:
  level: mutating
  rationale: >-
    Replaces one configuration row, or — on a dry run — nothing. Recoverable by another update; nothing is issued, revoked or deleted beyond the row itself.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries a stylesheet an administrator wrote; the response echoes what was, or would be, stored. No personal data.
retention:
  class: durable
  rationale: >-
    The stored stylesheet is the community's configuration until the next update, and the change is recorded in the audit trail against the administrator who made it. A dry run retains nothing.
errorCodes:
  - code: vtc/community/theme/update:outsideProfile
    meaning: The stylesheet uses a construct the theme profile does not admit. `details` names the first one found — `line` and `column` (1-based, in Unicode code points) and `rule`, the profile item it breaks — so an editor can point at it.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["line", "column", "rule"]
      properties:
        line:
          type: integer
          minimum: 1
        column:
          type: integer
          minimum: 1
        rule:
          description: The profile item broken, `P1` to `P8`.
          type: string
          pattern: "^P[1-8]$"
related:
  - vtc/community/theme/show
  - vtc/community/branding/update
---

## Abstract

The **VTC Community — Theme — Update** Trust Task replaces the community's theme — one stylesheet that its administration console and member portal apply over their built-in look, read back by [`vtc/community/theme/show`](../../show/0.1/spec.md). The stylesheet is restricted to the [theme profile](#the-theme-profile): design tokens, and a small set of cosmetic properties on ordinary elements. Nothing in the profile can fetch a resource or add content, and no element rule can move, hide or overlay an element. A token can change whatever a page draws from it — sizes and spacing included — which is why the surfaces where a person decides something are isolated from the theme altogether ([show, client item 2](../../show/0.1/spec.md#conformance)).

With `dryRun: true` the community checks the stylesheet and answers with what it would store, storing nothing. A client uses that to preview a theme exactly as it would be stored before an administrator commits it.

The update replaces the whole value. The empty string removes the theme.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller that does not hold administrator standing with `permissionDenied`, after resolving the proof's signer — or, for a signer that is a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the identity it acts for — to its access-control entry at execution time. This applies to a dry run too: the check runs the community's own parser, and is not a service for anyone who asks.
2. **MUST** refuse a stylesheet longer than 32768 bytes in UTF-8 with `malformedRequest`. The schema's `maxLength` counts code points; this bound counts bytes, and both apply.
3. **MUST** check the stylesheet against the [theme profile](#the-theme-profile) and **MUST** refuse, with `vtc/community/theme/update:outsideProfile`, one that uses any construct the profile does not admit. A check that cannot parse the stylesheet refuses it.
4. **MUST** store, and answer with, the stylesheet with its comments removed and otherwise unchanged. Removing comments is the only normalisation; it is what a client compares, so the stored text is what the dry run returned.
5. On `dryRun: true`, **MUST** run items 1–4, **MUST NOT** store anything or write an audit record, and **MUST** answer `stored: false` with no `updatedAt`.
6. Otherwise **MUST** record the change in its audit trail, naming the administrator and the digest of the stylesheet stored (never its text), and **MUST** refuse with `unavailable` rather than make an unaudited change when it cannot write the audit record. An update that stores the stylesheet already stored changes nothing, is not audited, and answers `stored: false`.
7. **MUST** answer with what it stored, `stored: true` when the stored theme changed.

A conforming **client** — an editor that sends this task:

1. **SHOULD** offer a preview: the stylesheet the dry run returned, applied to the community's pages drawn from sample data, in which every write does nothing. An administrator then sees the theme as it would be stored before storing it, and nothing they try in the preview reaches the community.
2. **MUST** apply the theme only under the client requirements of [`vtc/community/theme/show`](../../show/0.1/spec.md#conformance) — in a preview as everywhere else.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry, read at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's.

A theme confers no authority and needs no other administrator's consent. That is a consequence of the profile and of client item 2 of the show task together: a stylesheet that could restyle an approval prompt would let one administrator decide what another sees when asked to approve, and would then need the same second-party consent as the things those prompts protect. Keeping the profile and the isolation is what keeps this task an ordinary configuration write.

## The theme profile

A stylesheet is in the theme profile when every item below holds. The items are numbered so that `outsideProfile` can name the one broken (`details.rule`, e.g. `"P4"`).

- **P1 — Tokens.** It is CSS text ([CSS Syntax Level 3](https://www.w3.org/TR/css-syntax-3/)) of style rules and `@media` rules, separated by whitespace and comments.
- **P2 — No escapes, no markup.** It contains no `\` and no `<`. Escapes are how a banned name is spelled so that a simple scan misses it; `<` is how text leaves a `<style>` element.
- **P3 — At-rules.** The only at-rule is `@media` with the prelude `(prefers-color-scheme: light)` or `(prefers-color-scheme: dark)`, whose block holds style rules only. `@import`, `@font-face`, `@namespace`, `@layer`, `@keyframes`, `@property`, `@container`, `@supports` and every other at-rule are outside the profile.
- **P4 — No resources, no functions that read the page.** No value contains `url(`, `src(`, `image(`, `image-set(`, `cross-fade(`, `element(`, `attr(`, `paint(` or `env(`, matched case-insensitively. A theme loads nothing; drawing it discloses nothing about the visitor.
- **P5 — No `!important`.** A theme never outranks the page's own styles.
- **P6 — Token rules.** A style rule whose selector is exactly `:root`, `:root[data-theme="light"]` or `:root[data-theme="dark"]` declares only custom properties (`--` followed by `[a-z][a-z0-9-]*`), each value at most 512 characters. A custom property whose name begins `--protected-` is outside the profile: the name prefix is reserved for the surfaces the theme cannot reach.
- **P7 — Element selectors.** Any other style rule's selector list is built only from type, class, ID and attribute selectors, the combinators (descendant, `>`, `+`, `~`), the pseudo-classes `:hover`, `:focus`, `:focus-visible`, `:focus-within`, `:active`, `:disabled`, `:checked`, `:first-child`, `:last-child`, `:nth-child()` and `:not()` (over selectors in this list), and the pseudo-elements `::placeholder` and `::selection`. `:has()` and every other pseudo-class or pseudo-element are outside the profile.
- **P8 — Element properties.** Such a rule declares only custom properties as in P6, and these properties:
  - `color`, `background-color`, `accent-color`, `caret-color`, `outline-color`, `text-decoration-color`;
  - `background-image`, whose value is one or more of `linear-gradient()`, `radial-gradient()`, `conic-gradient()` and their `repeating-` forms, or `none`;
  - `border-color`, `border-style`, `border-width`, `border-radius` and their per-side and per-corner longhands, a width at most `4px` and a radius at most `32px`;
  - `font-family`, `font-weight`, `font-style`, `letter-spacing` (at most `0.2em` either way), `text-decoration-line`, `text-transform`.

  Every property that can move, size, hide, overlay, clip or add content — `display`, `visibility`, `opacity`, `position`, `inset`, `transform`, `z-index`, `content`, `width`, `height`, `margin`, `padding`, `font-size`, `box-shadow`, `filter`, `clip-path`, `mask`, `pointer-events` among them — is outside it.

The profile is deliberately small. Every admitted element property paints only within the element it is declared on, so no rule can draw over another element. What the profile cannot bound is a token: a page that sizes its controls from `--control-h` will draw them at whatever height a theme gives it, and a page that colours its text from `--text` can be made to draw text in the colour of its ground. The profile does not try to bound token values, because which tokens a page has and what it draws from them are the page's own. That is what client item 2 of the show task is for: the surfaces where a person decides, approves, consents or is warned take nothing from the theme, tokens included, so nothing a theme does can change what a person is asked to decide.

## Definitions

- **`stylesheet`** — CSS text in the theme profile, at most 32768 bytes. The empty string removes the theme.
- **`dryRun`** — check and answer, store nothing. Absent means `false`.
- **`stored`** — whether this request changed the stored theme.

## Request

A community administrator (`issuer`) sends the whole new stylesheet to the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Check a theme before storing it

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000201",
  "type": "https://trusttasks.org/spec/vtc/community/theme/update/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-17T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000201ff",
  "payload": {
    "stylesheet": "/* Forest */\n:root { --brand: #0a7f5c; --radius-md: 10px; }\n@media (prefers-color-scheme: dark) { :root { --brand: #3fc29a; } }\n.card { border-radius: 14px; }",
    "dryRun": true
  }
}
```

### Store it

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000203",
  "type": "https://trusttasks.org/spec/vtc/community/theme/update/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-17T10:02:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000201fe",
  "payload": {
    "stylesheet": ":root { --brand: #0a7f5c; --radius-md: 10px; }\n@media (prefers-color-scheme: dark) { :root { --brand: #3fc29a; } }\n.card { border-radius: 14px; }"
  }
}
```

### Remove the theme

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000205",
  "type": "https://trusttasks.org/spec/vtc/community/theme/update/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-17T10:05:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000201fd",
  "payload": {
    "stylesheet": ""
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`: what it stored, or would store. A refusal is a `trust-task-error`.

### The dry run — comment removed, nothing stored

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000202",
  "type": "https://trusttasks.org/spec/vtc/community/theme/update/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-17T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000201ff",
  "payload": {
    "stylesheet": "\n:root { --brand: #0a7f5c; --radius-md: 10px; }\n@media (prefers-color-scheme: dark) { :root { --brand: #3fc29a; } }\n.card { border-radius: 14px; }",
    "stored": false
  }
}
```

### Stored

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000204",
  "type": "https://trusttasks.org/spec/vtc/community/theme/update/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-17T10:02:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000201fe",
  "payload": {
    "stylesheet": ":root { --brand: #0a7f5c; --radius-md: 10px; }\n@media (prefers-color-scheme: dark) { :root { --brand: #3fc29a; } }\n.card { border-radius: 14px; }",
    "stored": true,
    "updatedAt": "2026-10-17T10:02:01Z"
  }
}
```

### Refused — a stylesheet that hides an element

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000206",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-17T10:03:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000201fc",
  "payload": {
    "code": "vtc/community/theme/update:outsideProfile",
    "message": "display is not a theme property",
    "retryable": false,
    "details": {
      "line": 3,
      "column": 18,
      "rule": "P8"
    }
  }
}
```

## Security & Privacy

### Data carried

A stylesheet an administrator wrote, drawn by every client that loads the community's pages. It is untrusted input there, and the profile is what makes it safe to draw: it can fetch nothing (P4), so drawing it discloses nothing about the visitor; it cannot outrank the page (P5) or move, hide or overlay anything (P8); and the client keeps it off every surface where a person decides something (show, client item 2). The audit record carries the stylesheet's digest, not its text, so the audit trail does not grow by 32 KiB an edit and still identifies exactly what was stored.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the theme is its own. The audit trail ties each change to the administrator who made it; that attribution is the point of requiring a proof.

### Retention

The stored stylesheet lives until the next update. The audit record of a change is durable and names the administrator and the digest. A dry run retains nothing.

### Consent/purpose

The purpose is to choose how the community's own pages look. Using a theme to change what a person is asked to decide is outside it, and the profile with the show task's client requirements is how the specification keeps it out.
