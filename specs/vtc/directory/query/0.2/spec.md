---
slug: vtc/directory/query
version: "0.2"
title: VTC Directory — Query
summary: Read the fields a community member lets this caller see, addressed by the member's DID and projected through the community's directory policy, with the member's directory entry identifier.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - directory
  - consent
  - projection
  - privacy
authors:
  - Glenn Gore (https://github.com/stormer78)
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
  requirement: RECOMMENDED
  rationale: The projection depends on who is asking, so an attributable caller gets a correctly scoped answer.
sideEffects:
  level: none
  rationale: "Evaluates the directory policy and projects fields; persists nothing."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses personal data about one member — the fields they chose to show this caller — and the identifier of their directory entry.
retention:
  class: transient
  rationale: A read.
errorCodes:
  - code: vtc/directory/query:notFound
    meaning: No member with that DID, or none whose directory projection is visible to this caller.
    retryable: false
related:
  - vtc/directory/entries/show
  - vtc/directory/entries/list
  - vtc/directory/images/get
  - vtc/directory/card/update
---

## Abstract

The **VTC Directory — Query** Trust Task returns the subset of a member's record that the community's directory policy permits this caller to see, addressed by the member's DID, together with the member's directory **entry identifier**. `fields` narrows the request; the policy decides what is actually returned.

The projection is a **policy decision, not a database read**. Two callers asking for the same subject and the same fields can legitimately receive different answers, because the directory ceremony evaluates who is asking against what the subject consented to publish.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.1

The response carries `entryId` ([`EntryId`](../../_shared/0.1/directory-card.schema.json)), the identifier of the member's directory entry, on every answer.

A caller that meets a member by their DID — the author of a record in a data room, the signer of a document — can read their projection here, but every other directory read is addressed by entry: the member's picture ([`vtc/directory/images/get`](../../images/get/0.1/spec.md)), their entry with the groups the caller shares ([`vtc/directory/entries/show`](../../entries/show/0.1/spec.md)), and the page a member surface shows for them. Without the entry identifier such a caller could name the member but not show their picture or open their card.

The specification also now targets framework 0.6.0. The request is unchanged; the response gains a required member, so released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Relationship to the directory family

Its successor for browsing is [`vtc/directory/entries/show`](../../entries/show/0.1/spec.md), which addresses a member by entry identifier and returns the full projection. A member addresses `entries/show` by DID only as an administrator, because a lookup by DID is the membership oracle the directory is designed not to be. This task is answered under the rule below instead, which makes it no such oracle: a community that serves both **MUST** answer it through the same projection, so the two never disagree about what a caller may see.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** names the `subject` DID. `fields` is an optional comma-separated narrowing hint.

A conforming **consumer** (the community):

1. **MUST** run the directory projection rather than reading the membership record directly, and return only fields the projection permits for this caller. A requested field the projection withholds **MUST** be **absent** from the response, not present-and-null — an explicit null discloses that the field exists and is being withheld, which is itself information the subject did not consent to share.
2. **MUST** answer `notFound` — the same code and message — when there is no such member, when the member exists but nothing is visible to this caller, and when the caller may not address members by DID and the projection does not show them the member's `did` field. Only a caller the projection would show the member's DID to anyway, the member themselves, or an administrator holding `vtc.directory.admin` or `vtc.members.manage` gets an answer.
3. **MUST**, on every answer, return `entryId`: the identifier of the member's entry, the same one [`vtc/directory/entries/list`](../../entries/list/0.2/spec.md) and [`entries/show`](../../entries/show/0.1/spec.md) answer for them. It **MUST NOT** answer an entry identifier on a `notFound`, or in any refusal's `details`.

## Request

A member (`issuer`) sends the community (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/directory/query/0.2",
  "issuer": "did:example:member",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-26T09:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "subject": "did:example:alberto"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`: the projection and the member's entry. A refusal is a `trust-task-error`.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/directory/query/0.2#response",
  "issuer": "did:example:community",
  "recipient": "did:example:member",
  "issuedAt": "2026-11-26T09:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "subject": "did:example:alberto",
    "entryId": "dir_3pQkV7wXyZ1aB2cD4eF6gH",
    "fields": {
      "displayName": "Alberto Leon",
      "role": "member",
      "did": "did:example:alberto"
    }
  }
}
```

## Security & Privacy

Conflating "absent because unset" with "absent because withheld" is the whole privacy property here, and it is why the two indistinguishable-outcomes rules above are normative rather than advisory. A directory that answers "that member exists but you may see nothing" is a membership oracle; one that answers `notFound` is not.

`fields` is a hint, never an authorisation. A caller asking for a field they may not see gets it withheld, not an error — an error would confirm the field exists.

### What the entry identifier adds

Nothing the caller could not already reach. An entry identifier is not derived from, and does not reveal, any DID of the member ([`EntryId`](../../_shared/0.1/directory-card.schema.json)), and it addresses nothing the projection would not show the same caller: `entries/show` and `images/get` answer it through the same projection. It is answered only where the caller is already shown the member — by item 2, a caller the projection shows the member's DID to, the member, or an administrator — and so links the DID the caller named to an entry that caller could open by browsing. A caller to whom the member is hidden gets `notFound`, with no identifier.

The identifier is stable for the life of the membership and survives a rotation of the member's DID. A caller who keeps it can follow the member across a rotation; that is the same property every other directory read gives a caller who found the entry by browsing.

### Data carried

A member's DID and an optional field hint in; that member's projection for the caller — only the fields the member's visibility, the field's ceiling and the community's policy all admit — and their entry identifier out. Nothing out for a member the caller may not see.

### Correlation

The community is addressed by its public DID, the identifier members already know it by. The answer names the member by the DID the caller sent and by the community-scoped entry identifier; neither links the member to another community, since an entry identifier is random per community (*What the entry identifier adds*).

### Retention

The community keeps nothing for this read beyond what its directory already holds; a request is recorded only as its framework replay protection and audit require.

### Consent/purpose

The member's own visibility choices decide what any caller sees, and the community's policy can only narrow them. The purpose is to name and introduce a member to someone the member already lets see them — for example, the author of a message in a room both share.
