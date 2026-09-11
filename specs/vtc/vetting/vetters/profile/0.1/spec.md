---
slug: vtc/vetting/vetters/profile
version: "0.1"
title: VTC Vetting — Vetter Profile
summary: A vetter publishes, replaces or unlists the profile applicants use to find them — languages, a coarse location, methods, accepted documentation and upcoming events. The community keeps it only while the vetter is an active member with a vetter grant.
status: draft
targetFrameworkVersion: "0.5"
category: governance
keywords:
  - vtc
  - vetting
  - vetter
  - profile
  - directory
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: vetter
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: The community acts on the authenticated sender, which it checks against its own member and grant records, so an authenticated transport identifies the vetter well enough to store a profile the vetter can overwrite at any time. A proof is recommended so a published profile stays attributable to its vetter after the transport has closed — which matters if a listed contact hint turns out to mislead applicants.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A profile replaces the stored one. An older document replayed after a newer one would re-list a vetter who had unlisted, or restore a location or event they had removed. Placing every document in a window is what lets the community refuse the stale copy.
sideEffects:
  level: mutating
  rationale: "Replaces the vetter's stored profile. Recoverable: the vetter sends another."
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: "The request carries personal data a vetter chooses to publish about themselves: a display name, languages, a coarse location, free-text availability and contact hint, and events they will attend. With `listed: true` the community discloses that, with the vetter's DID and grant expiry, to authenticated callers of vtc/vetting/vetters/list. The response returns only the stored `listed` flag and a timestamp."
retention:
  class: durable
  rationale: "The community keeps the profile until the vetter replaces it, and deletes it when the vetter's grant is revoked or the vetter leaves. Nothing about a profile is evidence of anything once the vetter can no longer vet, so it is kept no longer than that."
errorCodes:
  - code: vtc/vetting/vetters/profile:notEligible
    meaning: The sender is not an active member of this community holding a live vetter grant.
    retryable: false
related:
  - vtc/vetting/vetters/list
  - vtc/vetting/vetters/grant
  - vtc/vetting/vetters/resend
  - vetting/request
---

## Abstract

A community that admits people on peer identity vetting needs its applicants to find vetters. [`vtc/vetting/vetters/grant`](../../grant/0.1/spec.md) makes a member a vetter and publishes nothing about it. This task lets a vetter decide to be found.

The vetter sends the community a **profile**: the languages they can vet in, where they can meet, which methods they offer, what documentation they rely on, which events they will be at, and how to get a ticket from them. The vetter also says whether the profile is **listed**. Applicants and members search listed profiles with [`vtc/vetting/vetters/list`](../../list/0.1/spec.md).

A listing is not a way to reach a vetter. A request still goes through [`vetting/request`](../../../../../vetting/request/0.1/spec.md) with a ticket or an introduction the vetter accepts, and `contactHint` tells an applicant how to get one.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5 and may change in place while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **vetter** (`issuer`):

1. Sends the document from the member DID its vetter grant names.
2. **MUST** send the whole profile. A member it omits is absent from the stored profile afterwards; there is no partial update.
3. **MUST NOT** put a ticket code, a ticket secret, or any detail of an identity document in `displayName`, `availability`, `contactHint`, an event, or `ext`. A ticket published in a listing is no longer a gate.

A conforming **community** (`recipient`):

1. Applies the [SPEC §7.2](/SPEC.md#72-consumer-requirements) pipeline. It refuses with `malformedRequest` a payload whose event has an `endDate` before its `startDate`, or more than 31 days after it. JSON Schema cannot compare two members, so the schema does not catch either.
2. **MUST** refuse with `vtc/vetting/vetters/profile:notEligible` a sender that is not an active member of the community, or that holds no **live vetter grant**. That is a vetter role credential this community issued to the sender, neither expired nor revoked, as defined in [`vtc/vetting/vetters/grant`](../../grant/0.1/spec.md).
3. Otherwise **MUST** replace any profile it holds for the sender with this one, record when it did so, and return the stored `listed` and that time as `updatedAt`. Repeated execution is safe and intended ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 11): applying the same document again leaves the same profile stored.
4. **MAY** refuse, with `malformedRequest`, a document whose `issuedAt` is earlier than that of the document that produced the profile it holds.
5. **MUST** delete the profile when the member's vetter grant is revoked, and when the member leaves or is removed. While the member holds no live grant for any other reason, such as expiry, the profile is not listed.
6. **MUST NOT** disclose a profile except through [`vtc/vetting/vetters/list`](../../list/0.1/spec.md) under that task's rules, and **MUST NOT** list a profile stored with `listed: false`.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authority this task presupposes is a **live vetter grant held by an active member**: the community's own decision, made through [`vtc/vetting/vetters/grant`](../../grant/0.1/spec.md), that this member may vet. A profile is the vetter describing that role, and it confers nothing. Listing does not make a vetter eligible, and unlisting does not make them ineligible.

The payload names no DID. A vetter can only write their own profile, and the sender is the one the community checks. Per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10, a verified `proof` or an authenticated transport establishes which member sent the document, never that it holds a grant; that is checked against the community's own member and grant records.

This task gives an administrator no authority over a vetter's profile. A profile is the vetter's own statement, and an administrator does not write one for them. An administrator who needs a vetter removed from the listing revokes the grant, which deletes the profile.

## Definitions

**Vetter profile** — the object this task stores for a vetter: `listed`, `displayName`, `languages`, `location`, `methods`, `acceptsDocumentation`, `availability`, `contactHint`, `events` and `ext`, exactly as last sent.

**Listed** — a profile stored with `listed: true`, whose vetter is an active member with a live vetter grant. Only listed profiles appear in [`vtc/vetting/vetters/list`](../../list/0.1/spec.md).

**Live vetter grant** — as defined in [`vtc/vetting/vetters/grant`](../../grant/0.1/spec.md): a vetter role credential issued by this community to the member, not expired and not revoked.

| Member | Rule |
|---|---|
| `listed` | Required. `false` keeps the profile stored and out of listings |
| `displayName` | Optional, 1–128 characters. Self-asserted, not a legal name |
| `languages` | 0–16 BCP 47 tags, each 2–35 characters |
| `location` | Optional. `country` (ISO 3166-1 alpha-2, upper case) required; `region` and `city` optional, 1–128 characters each |
| `methods` | 1–3 of `inPerson`, `video`, `priorAcquaintance` |
| `acceptsDocumentation` | 0–16 documentation classes, in the tokens of `vetting/request`'s `acceptsDocumentation` |
| `availability` | Optional, 1–500 characters of free text |
| `contactHint` | Optional, 1–300 characters of free text on how to get a ticket |
| `events` | 0–32 events: `name` (1–200), `startDate` and `endDate` (`YYYY-MM-DD`, `endDate` 0–31 days after `startDate`), optional `location`, optional https `url` (≤ 2048) |
| `ext` | Optional ([SPEC §4.5.1](/SPEC.md#451-the-ext-extension-member)) |

## Request

The vetter sends the request to the community. See the top-level schema in [`payload.schema.json`](payload.schema.json); the shared member definitions are in [`vetter-profile.schema.json`](../../../../_shared/0.1/vetter-profile.schema.json).

### Carol lists herself for in-person and video vetting

```json
{
  "id": "urn:uuid:3a7c9e1f-2b4d-4f6a-8c0e-9d1b3f5a7c01",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/profile/0.1",
  "threadId": "urn:uuid:3a7c9e1f-2b4d-4f6a-8c0e-9d1b3f5a7c01",
  "issuer": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
  "recipient": "did:webvh:QmVtcScid:kernel-vtc.example",
  "issuedAt": "2026-09-15T08:30:00Z",
  "payload": {
    "listed": true,
    "displayName": "Carol M.",
    "languages": ["en", "de-AT"],
    "location": { "country": "AT", "city": "Vienna" },
    "methods": ["inPerson", "video"],
    "acceptsDocumentation": ["passport", "nationalId", "none"],
    "availability": "Weekday evenings, Central European Time.",
    "contactHint": "Ask for a ticket at the kernel-vtc table at the meetup, or message me in the community chat.",
    "events": [
      {
        "name": "Kernel Maintainers Meetup 2026",
        "startDate": "2026-10-05",
        "endDate": "2026-10-07",
        "location": { "country": "AT", "city": "Vienna" },
        "url": "https://kernel-vtc.example/events/maintainers-meetup-2026"
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmCarolScid1:kernel-vtc.example:carol#key-1",
    "created": "2026-09-15T08:30:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z2RA8945kouBqzqifZqkbB8ZSrj1sfVLZPvr6wz4RvHSaYqXySHQoep9vM1fRYit6tNfmaTDThA2ibMPhBMFh8w3N"
  }
}
```

### Carol unlists herself while travelling

The whole profile is sent again with `listed: false`. The community keeps it, and the listing stops showing it.

```json
{
  "id": "urn:uuid:3a7c9e1f-2b4d-4f6a-8c0e-9d1b3f5a7c03",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/profile/0.1",
  "threadId": "urn:uuid:3a7c9e1f-2b4d-4f6a-8c0e-9d1b3f5a7c03",
  "issuer": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
  "recipient": "did:webvh:QmVtcScid:kernel-vtc.example",
  "issuedAt": "2026-10-20T07:00:00Z",
  "payload": {
    "listed": false,
    "displayName": "Carol M.",
    "languages": ["en", "de-AT"],
    "location": { "country": "AT", "city": "Vienna" },
    "methods": ["inPerson", "video"],
    "acceptsDocumentation": ["passport", "nationalId", "none"],
    "events": []
  }
}
```

## Response

The community, now responding, confirms what it stored, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). `listed` is the stored flag and `updatedAt` is when the profile was stored; [`vtc/vetting/vetters/list`](../../list/0.1/spec.md) returns the same `updatedAt`. Refusals use `trust-task-error` with `malformedRequest` or this specification's code, never a `#response`.

### Stored and listed

```json
{
  "id": "urn:uuid:3a7c9e1f-2b4d-4f6a-8c0e-9d1b3f5a7c02",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/profile/0.1#response",
  "threadId": "urn:uuid:3a7c9e1f-2b4d-4f6a-8c0e-9d1b3f5a7c01",
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "recipient": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
  "issuedAt": "2026-09-15T08:30:01Z",
  "payload": {
    "listed": true,
    "updatedAt": "2026-09-15T08:30:01Z"
  }
}
```

### Refused: the sender holds no live vetter grant

```json
{
  "id": "urn:uuid:3a7c9e1f-2b4d-4f6a-8c0e-9d1b3f5a7c05",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:3a7c9e1f-2b4d-4f6a-8c0e-9d1b3f5a7c04",
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "recipient": "did:webvh:QmDavidScid1:kernel-vtc.example:david",
  "issuedAt": "2026-09-15T09:00:01Z",
  "payload": {
    "code": "vtc/vetting/vetters/profile:notEligible",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

Everything in a profile is published by the vetter's choice, and most of it is personal data about the vetter: the name they go by, the languages they speak, where they can be met, and when they will be at an event. The smallest profile is `listed`, one method, and three empty lists. Anything more is the vetter trading privacy for being easier to find.

`location` is coarse by construction. It has a country, optionally a region and a city, and no member for a street, a postcode or coordinates. Events are the most sensitive part, because an event with dates places a person at a place at a known time. A vetter's agent **SHOULD** make that plain before sending one, and a vetter **SHOULD** list only events where they are content to be approached.

`displayName`, `availability`, `contactHint` and event names are free text nobody has checked. A client rendering them **MUST** attribute them to the vetter. Tickets and identity-document details never belong in them (Conformance, vetter item 3).

### Correlation

The vetter declares `identifierScope: public`. A listing is useful only if it names the DID an applicant will address in [`vetting/request`](../../../../../vetting/request/0.1/spec.md), the DID the vetter's role credential names. A pairwise DID per applicant would match no credential. The consequence is that a listed profile links the vetter's member DID to the vetter role, a location and upcoming whereabouts, for every caller the community answers. That is why listing is the vetter's decision, a member with a grant and no profile is listed nowhere, and `listed: false` withdraws a profile without deleting it.

The community declares `identifierScope: public` for the reason every task in this family does: the vetter must be able to name the community the grant came from.

### Retention

The community keeps the current profile until the vetter replaces it, and deletes it when the grant is revoked or the vetter leaves. It **SHOULD NOT** keep superseded profiles beyond what its audit policy requires: an old profile says where a person used to be, and no decision depends on it. Events that have ended stay in the stored profile until the vetter removes them, but are never listed.

### Consent/purpose

The purpose is to let applicants find a vetter for this community. A community **MUST NOT** use a profile for anything else: not a page anyone can read without authenticating, not an export to another community, and not a way to contact vetters about something other than vetting. Whether a vetter's agent asks before publishing a profile, and whether a community requires anything before listing one, are the agent's and the community's policy; per [SPEC §7.3](/SPEC.md#73-specification-requirements) item 13 this specification takes no position on them.
