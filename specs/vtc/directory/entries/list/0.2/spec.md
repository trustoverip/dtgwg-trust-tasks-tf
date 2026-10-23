---
slug: vtc/directory/entries/list
version: '0.2'
title: VTC Directory — Entries — List
summary: 'Browse, search and filter a community''s member directory. Each entry is projected for the caller: only fields the member, the community''s ceiling and its policy all allow.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
parties:
- role: member or administrator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: The projection depends entirely on who asks, and the continuation cursor is bound to the asker; an attributable caller gets a correctly scoped page.
sideEffects:
  level: none
  rationale: Evaluates the directory projection; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses personal data about other members — names, avatars, roles and whatever fields each member chose to show this caller.
retention:
  class: transient
  rationale: A read; nothing is kept beyond the audit record of the question.
errorCodes:
- code: vtc/directory/entries/list:queryTooShort
  meaning: '`query` is shorter than three characters and is not an exact nickname search.'
  retryable: false
- code: vtc/directory/entries/list:cursorInvalid
  meaning: '`cursor` was not issued for this caller and this question (query, filters and group).'
  retryable: false
- code: vtc/directory/entries/list:notFound
  meaning: '`group` names no community group the caller may see.'
  retryable: false
- code: vtc/directory/entries/list:rateLimited
  meaning: The caller has read the directory too often. Counted over the caller's own requests only.
  retryable: true
related:
- vtc/directory/entries/show
- vtc/directory/fields/list
- vtc/groups/members/list
---

## Abstract

The **VTC Directory — Entries — List** Trust Task is the directory's browse, search and filter. With no `query` it browses: it returns the entries that are `listed`. With a `query` it searches: it also returns `findable` entries that match under their member's search setting. `filters` narrow by facet; `group` narrows to one community group.

Every entry is a **projection** for the caller. Two callers asking the same question can receive different entries and different fields, because each member decides who sees what, the community caps it and its policy may narrow it further. A member who chose `hidden` is never found here by another member.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Changes from 0.1

The response's **`nextCursor`** changes type from `["string", "null"]` to **`"string"`** (still at most 1024 characters). It is present when more items remain and **absent** on the last page; a `null` value now fails validation. A producer that detected the last page by `nextCursor === null` **MUST** detect it by absence instead.

This is the registry-wide pagination convention: a continuation token is an optional string whose absence marks the last page, so there is one spelling of "no more pages" instead of two.

The request's `cursor` and `limit` already followed the convention and are unchanged on the wire — same types and bounds; only their schema descriptions are reworded to the registry's paging text. Nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is neither an active member nor an administrator with `permissionDenied`.
2. **Browsing** (no `query`): return only entries whose listing is `listed` and whose required fields are complete. **Searching** (`query`): return `listed` and `findable` entries whose member's `search` setting admits the match — `byName` matches `displayName` or `nickname`, `exactNickname` only an exact nickname, `off` nothing — and searchable fields the caller may see. A `hidden` entry is never returned to a non-administrator, except through `group` where the group shows its members to the caller.
3. Refuse a `query` shorter than three characters with `queryTooShort`, unless it is matched only as an exact nickname. Return at most 50 matches to any one search, across all its pages.
4. **MUST NOT** match a DID for a non-administrator, whatever the query looks like. An administrator's search **MAY** match the derived `did` field.
5. Return administrators every entry, at every listing, with every field.
6. Bind `nextCursor` to the caller, the query, the filters and the group under a key of its own, and refuse a cursor presented with any of them changed with `cursorInvalid` (VTI-ACL-066).
7. Offer and count a facet only for a field that is `facet: true` **and** visible to the caller; count only entries the caller could see; and A facet count below the community's threshold *k* (default 5) is returned as `count` absent and `fewerThan: k`, so that filtering cannot single out a hidden value.
8. Count requests per caller and refuse with `rateLimited` beyond the community's limit, counted over the caller's own requests regardless of what they found.
9. Return `notFound` for a `group` the caller may not see, exactly as for one that does not exist.

**The projection.** For each entry, a field is returned to the viewer only where **all three** of the following hold, and the community **MUST** compute their intersection, never their union:

1. the visibility the **member** chose for the field (or, where they chose none, the field's `defaultVisibility`) covers the viewer;
2. the community's **ceiling** for the field covers the viewer;
3. the community's directory policy permits it.

An audience covers a viewer as [`Audience`](../../../_shared/0.1/directory-card.schema.json) defines: the viewer satisfies the audience chosen, or a wider one. The community — not its policy — computes which audiences a viewer satisfies, and policy may only narrow the result. A field that is not returned **MUST** be **absent** from `fields`, never present-and-null.

**Host invariants.** Whatever the directory policy returns, the community **MUST NOT**:

- project a departed, suspended or `application` member, or a member whose record is removed, to any viewer but an administrator;
- place an administrative note, or any part of one, in a projection;
- return a `public` field outside the community except through the member's own signed public card;
- return a `disabled` or `retired` field.

## Authorization

**Authority: standing in the community, and each member's own visibility choices.** Any active member may read the directory, and what they read is decided per field by the member it is about: the community's projection is the enforcement of those choices, not a discretion of its own. Administrators holding `vtc.directory.admin` read every entry whole. The `directory` policy purpose may narrow any projection and may never widen one. The proof, where present, identifies the caller so their standing, their groups and their relationships can be read at execution time; it is not the authorization ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `DirectoryEntry`, `Audience`, `Listing` — see [`directory-card.schema.json`](../../../_shared/0.1/directory-card.schema.json).
- *k* — the community's facet threshold, default 5.

## Request

The member or administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Search for a name

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000001e",
  "type": "https://trusttasks.org/spec/vtc/directory/entries/list/0.2#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000001d",
  "payload": {
    "query": "caro",
    "limit": 25
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:carol#key-1",
    "created": "2026-10-18T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/directory/entries/list/0.2#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Search for a name — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000001f",
  "type": "https://trusttasks.org/spec/vtc/directory/entries/list/0.2#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000001d",
  "payload": {
    "items": [
      {
        "entryId": "dir_Q7m2K9xVb4Tz8LpR",
        "fields": {
          "displayName": "Carol",
          "pronouns": "she/her",
          "role": "member"
        },
        "sharedGroups": [
          "grp_W5k2N8xRb3Qz7TpL"
        ]
      }
    ],
    "facets": [
      {
        "field": "org.example.chapter",
        "values": [
          {
            "value": "sydney",
            "count": 12
          },
          {
            "value": "berlin",
            "fewerThan": 5
          }
        ]
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The response carries personal data about other members: whatever directory fields each subject chose to show this viewer, and which the community and its policy allow. Nothing else is in it — no DID unless the subject made `did` visible, no administrative note, no field the intersection withholds. A producer **MUST NOT** put anything in `ext` that the community would not otherwise disclose. Facet counts below *k* are withheld because a filter combined with a count of one is a disclosure of a value the member did not show.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Entries are addressed by `EntryId`, which is random, scoped to this community and not derived from any DID, so the directory adds no correlator of its own across communities; a member's DID appears only where they chose to show it. A viewer can still link what they see to what they see elsewhere through values the subject chose to show (a name, an avatar). Avatars are re-encoded by the community so their digests do not link communities.

### Retention

Nothing is retained by the community beyond its audit record of the read, which names the viewer and the question, never the projected values. A viewer **SHOULD** treat a projection as current only until it next reads it: the subject may narrow it at any time.

### Consent/purpose

The projection exists so members can find, recognise and reach one another within the community. A viewer **MUST NOT** reuse it to build a list of the community's members outside the directory, and a community **MUST NOT** treat a member's visibility choice as consent to any other disclosure. Whether a read is rate-limited is the community's policy.
