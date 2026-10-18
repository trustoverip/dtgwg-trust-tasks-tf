---
slug: vtc/directory/entries/show
version: '0.1'
title: VTC Directory — Entries — Show
summary: Read one directory entry, projected for the caller. Members address it by entry identifier; administrators may also address it by DID.
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
  rationale: The projection depends on who asks; an attributable caller gets a correctly scoped answer.
sideEffects:
  level: none
  rationale: Evaluates the directory projection; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses personal data about one member — the fields they chose to show this caller.
retention:
  class: transient
  rationale: A read.
errorCodes:
- code: vtc/directory/entries/show:notFound
  meaning: 'No entry the caller may see: no such entry, or one with nothing visible to this caller. The two are indistinguishable by design.'
  retryable: false
related:
- vtc/directory/entries/list
- vtc/directory/query
- vtc/directory/introduce
---

## Abstract

The **VTC Directory — Entries — Show** Trust Task reads one entry, projected for the caller exactly as [`vtc/directory/entries/list`](../../list/0.1/spec.md) projects it, plus the groups the caller shares with the member. It succeeds the minimal [`vtc/directory/query/0.1`](../../../query/0.1/spec.md), which addresses a member by DID.

Members address an entry by its `entryId` — found by browsing or search, or given to them by its member. Only administrators may address an entry by DID: a lookup by DID is exactly the membership oracle the directory is designed not to be.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** names exactly one of `entryId` and `did`.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is neither an active member nor an administrator with `permissionDenied`.
2. Refuse `did` from a caller without `vtc.directory.admin` or `vtc.members.manage` with `permissionDenied`, **before** looking the DID up, so the refusal says nothing about the DID.
3. Return `notFound` — the same code, message and timing class — both when there is no such entry and when the caller may see nothing of it. A member who chose `hidden` is `notFound` to another member who does not share a group showing them, even by `entryId`.
4. Return `connected` only when the caller is a party to an attributed, in-force verified relationship with the member, and `sharedGroups` only for groups whose membership the caller may see.

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

**Authority: standing in the community, and the member's own visibility choices**, exactly as for [`vtc/directory/entries/list`](../../list/0.1/spec.md). Addressing by DID additionally needs `vtc.directory.admin` or `vtc.members.manage`, because it answers "is this DID a member?" — a question only those who administer membership may ask. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `DirectoryEntry`, `EntryId` — see [`directory-card.schema.json`](../../../_shared/0.1/directory-card.schema.json).

## Request

The member or administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member reads an entry

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000021",
  "type": "https://trusttasks.org/spec/vtc/directory/entries/show/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000020",
  "payload": {
    "entryId": "dir_Q7m2K9xVb4Tz8LpR"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/directory/entries/show/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### A member reads an entry — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000022",
  "type": "https://trusttasks.org/spec/vtc/directory/entries/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000020",
  "payload": {
    "entry": {
      "entryId": "dir_Q7m2K9xVb4Tz8LpR",
      "fields": {
        "displayName": "Carol",
        "role": "member",
        "avatar": {
          "sha256": "9f2c9f2c9f2c9f2c9f2c9f2c9f2c9f2c9f2c9f2c9f2c9f2c9f2c9f2c9f2c9f2c",
          "mediaType": "image/webp",
          "size": 18213,
          "url": "https://community.example/v1/directory/blobs/9f2c?sig=abc",
          "urlExpiresAt": "2026-10-18T09:05:00Z"
        }
      },
      "connected": true
    }
  }
}
```

### Not found — or nothing visible

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000023",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000024",
  "payload": {
    "code": "vtc/directory/entries/show:notFound",
    "message": "No such entry.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The response carries personal data about other members: whatever directory fields each subject chose to show this viewer, and which the community and its policy allow. Nothing else is in it — no DID unless the subject made `did` visible, no administrative note, no field the intersection withholds. A producer **MUST NOT** put anything in `ext` that the community would not otherwise disclose.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Entries are addressed by `EntryId`, which is random, scoped to this community and not derived from any DID, so the directory adds no correlator of its own across communities; a member's DID appears only where they chose to show it. A viewer can still link what they see to what they see elsewhere through values the subject chose to show (a name, an avatar). Avatars are re-encoded by the community so their digests do not link communities.

### Retention

Nothing is retained by the community beyond its audit record of the read, which names the viewer and the question, never the projected values. A viewer **SHOULD** treat a projection as current only until it next reads it: the subject may narrow it at any time.

### Consent/purpose

The projection exists so members can find, recognise and reach one another within the community. A viewer **MUST NOT** reuse it to build a list of the community's members outside the directory, and a community **MUST NOT** treat a member's visibility choice as consent to any other disclosure. Whether a read is rate-limited is the community's policy.
