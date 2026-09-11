---
slug: vtc/vetting/vetters/list
version: "0.1"
title: VTC Vetting — List Vetters
summary: Find a community's listed vetters by language, location, method or event. Only vetters who chose to be listed and hold a live grant appear, and only with what they published.
status: draft
targetFrameworkVersion: "0.5"
category: governance
keywords:
  - vtc
  - vetting
  - vetter
  - directory
  - discovery
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: applicant or member
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: The community answers only a caller it can identify, and an authenticated transport identifies one. A proof is recommended so a community that rate-limits or audits listing reads can attribute them on every transport, relayed ones included.
sideEffects:
  level: none
  rationale: "Reads stored vetter profiles and grant records; persists nothing."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The response discloses listed vetters' self-published profiles, with each vetter's member DID and grant expiry — personal data about the vetters, which each chose to publish to authenticated callers. The request carries optional filters, which reveal where, when and in what language the caller is looking for a vetter."
retention:
  class: transient
  rationale: "A read changes nothing and needs no record. What a community may keep is that an identified caller searched, for rate limiting. The caller's copy goes stale as soon as a vetter updates their profile or loses their grant."
errorCodes: []
related:
  - vtc/vetting/vetters/profile
  - vtc/vetting/vetters/grant
  - vetting/request
  - vtc/join-requests/manifest
---

## Abstract

An applicant to a community that admits people on peer identity vetting has to find vetters, usually people they have never met. This task returns the community's **listed vetters**: members who hold a live vetter grant and have published a profile with [`vtc/vetting/vetters/profile`](../../profile/0.1/spec.md) saying they want to be found. A caller can narrow the list by language, country, region, city, method, or events the vetter will attend.

Each entry is what the vetter published, plus their DID and when their grant expires. The applicant then gets a ticket as the entry's `contactHint` describes and sends [`vetting/request`](../../../../../vetting/request/0.1/spec.md), whose `eligibilityVp` proves the vetter's eligibility for that request. The listing is a directory, not proof of eligibility.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5 and may change in place while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **caller** (`issuer`) **MAY** send any combination of filters, and **MUST** treat `nextCursor` as opaque, sending it back unchanged with the filters of the request that returned it.

A conforming **community** (`recipient`):

1. Applies the [SPEC §7.2](/SPEC.md#72-consumer-requirements) pipeline. It **MUST NOT** answer a caller it cannot identify — one with neither a verified `proof` nor a transport-authenticated sender — and refuses it with `permissionDenied`. It refuses with `malformedRequest` a request whose `eventFrom` is after its `eventTo`, and a `cursor` it did not issue for the same filters.
2. **MUST** return a vetter only when all of these hold: the vetter is an active member; the vetter holds a **live vetter grant** as defined in [`vtc/vetting/vetters/grant`](../../grant/0.1/spec.md), neither expired nor revoked; the community holds a profile for the vetter stored with `listed: true`; and the vetter matches every filter sent.
3. Applies the filters as follows. Text comparisons are case-insensitive.
   - `language` matches a listed tag equal to it, or beginning with it followed by `-`. `de` matches `de` and `de-AT`, and `de-AT` does not match `de`.
   - `country` matches `location.country`. `region` and `city` match `location.region` and `location.city`, exactly apart from case. A vetter without that member does not match.
   - `method` matches when it is among the vetter's `methods`.
   - `eventFrom`, `eventTo` and `eventName` are the **event filters**. An event matches when its `startDate`–`endDate` range, both ends included, overlaps `eventFrom`–`eventTo`, where an absent `eventFrom` or `eventTo` leaves that end open, and when its `name` contains `eventName`, where sent. Only events whose `endDate` is today (UTC) or later are considered, which are the events the entry returns. A vetter matches the event filters when at least one considered event matches. When no event filter is sent, events play no part, and vetters with no events match.
4. **MUST** order the entries this way. When an event filter is sent, by the earliest `startDate` among the vetter's matching events, earliest first. Otherwise, and among entries with the same such date, by `displayName`, with entries that have none after those that do, and then by `vetterDid`. Names and DIDs are compared by Unicode code point.
5. Returns at most `limit` entries, 50 where `limit` is absent. It includes `nextCursor` when more entries match and omits it on the last page. A cursor continues the same order.
6. Returns in each entry the vetter's member DID as `vetterDid`; the stored `displayName`, `languages`, `location`, `methods`, `acceptsDocumentation`, `availability` and `contactHint`, with optional members absent where the profile has none; the stored events whose `endDate` is today (UTC) or later; the live grant's `validUntil` as `grantValidUntil`; and the profile's `updatedAt`.
7. **MUST NOT** disclose anything else about a vetter or the community's vetters. That includes a grant's identifiers or issue date, the member record, an unlisted or ineligible profile, how many vetters are unlisted, and why a particular member does not appear.

Repeating a request is always safe: it reads and changes nothing.

### Why there is no `show`

This family deliberately has no single-vetter read to pair with this list ([CONTRIBUTING-SPECS](/CONTRIBUTING-SPECS.md#read-one-and-read-many-tasks)). A `show` by DID answers "is this member a vetter?", and its `notFound` would tell a caller whether a member is a vetter who has not listed themselves. That is the one determination a listing must not make. An applicant who already has a vetter's DID learns whether they are eligible from the `eligibilityVp` in [`vetting/request`](../../../../../vetting/request/0.1/spec.md), bound to that request. So there is no lookup by DID, here or anywhere else in the family. An empty page, by design, says nothing.

## Authorization

The authority this task presupposes is **being an identified caller**: an applicant or a member the community can authenticate. Nothing more is needed, because every entry was published by its vetter for exactly this audience. A community **MAY** restrict callers further under its own policy, for example to members and applicants with an open join request, or by rate limit.

This task is not consequential ([SPEC §2](/SPEC.md#2-terminology)). It changes nothing, and what it discloses is metadata the vetters chose to publish. Per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10, identifying the caller establishes who asked, not that the community must answer.

## Definitions

**Listed vetter** — an active member holding a live vetter grant whose profile is stored with `listed: true`.

**Event filters** — `eventFrom`, `eventTo` and `eventName`, applied together to one event at a time (Conformance item 3).

## Request

The caller sends the request to the community. Every member is optional, and `{}` lists every listed vetter, a page at a time. See the top-level schema in [`payload.schema.json`](payload.schema.json).

### Alice looks for a German-speaking vetter to meet in Austria

```json
{
  "id": "urn:uuid:9e2b4d6f-1a3c-4e5f-8b7d-0c2e4a6b8d01",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/list/0.1",
  "threadId": "urn:uuid:9e2b4d6f-1a3c-4e5f-8b7d-0c2e4a6b8d01",
  "issuer": "did:webvh:QmAliceScid1:alice.example",
  "recipient": "did:webvh:QmVtcScid:kernel-vtc.example",
  "issuedAt": "2026-09-16T10:00:00Z",
  "payload": {
    "language": "de",
    "country": "AT",
    "method": "inPerson",
    "limit": 20
  }
}
```

### Alice looks for vetters at a meetup in October

```json
{
  "id": "urn:uuid:9e2b4d6f-1a3c-4e5f-8b7d-0c2e4a6b8d03",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/list/0.1",
  "threadId": "urn:uuid:9e2b4d6f-1a3c-4e5f-8b7d-0c2e4a6b8d03",
  "issuer": "did:webvh:QmAliceScid1:alice.example",
  "recipient": "did:webvh:QmVtcScid:kernel-vtc.example",
  "issuedAt": "2026-09-16T10:05:00Z",
  "payload": {
    "eventFrom": "2026-10-01",
    "eventTo": "2026-10-31",
    "eventName": "meetup",
    "limit": 2
  }
}
```

## Response

The community, now responding, returns one page, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). An empty `vetters` array is a successful answer. This task defines no extended error codes; failures use `trust-task-error` with the framework's standard codes.

### Carol matches

Carol's listed languages include `de-AT`, which `de` matches.

```json
{
  "id": "urn:uuid:9e2b4d6f-1a3c-4e5f-8b7d-0c2e4a6b8d02",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/list/0.1#response",
  "threadId": "urn:uuid:9e2b4d6f-1a3c-4e5f-8b7d-0c2e4a6b8d01",
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "recipient": "did:webvh:QmAliceScid1:alice.example",
  "issuedAt": "2026-09-16T10:00:01Z",
  "payload": {
    "vetters": [
      {
        "vetterDid": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
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
        ],
        "grantValidUntil": "2027-09-13T10:00:01Z",
        "updatedAt": "2026-09-15T08:30:01Z"
      }
    ]
  }
}
```

### A page of two, with more to come

Both vetters' earliest matching event starts on 5 October, so Carol, who has a `displayName`, comes before Erin, who has none. `nextCursor` fetches the rest.

```json
{
  "id": "urn:uuid:9e2b4d6f-1a3c-4e5f-8b7d-0c2e4a6b8d04",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/list/0.1#response",
  "threadId": "urn:uuid:9e2b4d6f-1a3c-4e5f-8b7d-0c2e4a6b8d03",
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "recipient": "did:webvh:QmAliceScid1:alice.example",
  "issuedAt": "2026-09-16T10:05:01Z",
  "payload": {
    "vetters": [
      {
        "vetterDid": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
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
        ],
        "grantValidUntil": "2027-09-13T10:00:01Z",
        "updatedAt": "2026-09-15T08:30:01Z"
      },
      {
        "vetterDid": "did:webvh:QmErinScid1:kernel-vtc.example:erin",
        "languages": ["en"],
        "methods": ["inPerson"],
        "acceptsDocumentation": [],
        "events": [
          {
            "name": "Kernel Maintainers Meetup 2026",
            "startDate": "2026-10-05",
            "endDate": "2026-10-07"
          }
        ],
        "grantValidUntil": "2027-03-01T00:00:00Z",
        "updatedAt": "2026-09-10T17:12:44Z"
      }
    ],
    "nextCursor": "c2Vlay0yMDI2LTEwLTA1LWVyaW4"
  }
}
```

## Security & Privacy

### Data carried

The response carries personal data about vetters: names they go by, languages, coarse locations, free text, and events that place them somewhere at a known time. Every item was published by that vetter through [`vtc/vetting/vetters/profile`](../../profile/0.1/spec.md) with `listed: true`, and a vetter who has not done so is never returned. Beyond the profile, an entry adds only two things. `vetterDid` is the DID the applicant must address anyway. `grantValidUntil` tells an applicant whether the vetter will still be eligible when they meet, and the role credential the vetter presents in `vetting/request` shows it anyway. The community's records about the member, and the grant's identifiers, stay with the community (Conformance item 7).

Free text in an entry is the vetter's, unchecked. A client **MUST** attribute it to the vetter, and **SHOULD** render event URLs as links to leave the client, not as trusted community content.

The request carries only filters. They are optional, and the smallest request is `{}`. A filter still reveals something about the caller: where they are, where they will travel, which languages they speak.

### Correlation

The caller declares `identifierScope: any`, and the community must be able to identify it (Conformance item 1). The community therefore learns who searched for vetters where and when. For an applicant, that can include their location and travel plans before they have applied. If the caller is the join DID the applicant later applies with, the search is linked to the application. A community **SHOULD NOT** join listing reads to join requests for any purpose but rate limiting. An applicant who does not want the link **MAY** search under a different DID the community will authenticate.

The community declares `identifierScope: public`. An applicant has to name the community to ask it anything, and the vetters' role credentials name it too.

Listed vetters are public to every identified caller by design, and any caller can page through all of them. That is the price of being findable, and it is why listing is opt-in and revocable. A community **SHOULD** rate-limit listing reads so that copying the whole directory is slow, and a cursor **MUST NOT** encode anything about an entry the caller has not been sent.

### Retention

Transient. The community has nothing to keep from a read beyond counters for rate limiting. The caller **SHOULD** keep an entry only until it has contacted, or given up on, the vetter. Entries go stale when a vetter updates or unlists a profile, and when a grant expires or is revoked. A client that builds its own copy of the directory keeps publishing vetters who have since withdrawn.

### Consent/purpose

The purpose is to help applicants and members find a vetter for this community. A vetter's `listed: true` is their own decision to be found for that purpose, and neither the community nor a caller may widen it. A caller **MUST NOT** republish entries, merge them into a directory spanning communities, or contact vetters about anything other than vetting. Whether a community restricts who may list vetters beyond identifying them is its policy, and whether a client asks its user before searching is the client's; per [SPEC §7.3](/SPEC.md#73-specification-requirements) item 13 this specification takes no position on either.
