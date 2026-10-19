---
slug: vtc/directory/disclosure
version: '0.1'
title: VTC Directory — Disclosure
summary: Read the statement, generated from the community's configuration, of what its directory discloses about members and to whom — readable by anyone, applicants above all, before they apply.
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- directory
- disclosure
- privacy
- onboarding
parties:
- role: reader
  requirement: REQUIRED
  member: issuer
  identifierScope: pairwise
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: The statement is public, and the same for every reader. A proof is not required to read it, so that a prospective applicant can learn what the directory would disclose about them without identifying themselves; it is recommended so a community can rate-limit or attribute reads.
sideEffects:
  level: none
  rationale: Generates the statement from the configuration in force; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: 'The response describes the directory''s configuration — fields, ceilings, defaults and which features exist — never any member''s value, and never which members there are. The request carries nothing.'
retention:
  class: transient
  rationale: The request has no members, so there is nothing for the community to keep from it beyond whatever attribution the optional proof carries.
errorCodes: []
related:
- vtc/join-requests/manifest
- vtc/directory/fields/list
- vtc/directory/card/update
- vtc/directory/notes/list
---

## Abstract

The **VTC Directory — Disclosure** Trust Task returns a community's **disclosure statement**: what its member directory would disclose about a member, and to whom. It answers, for someone deciding whether to join, the questions a privacy notice should answer and usually does not answer precisely:

- which fields a card can hold, who writes each one — the member, the community, or a credential — and the widest audience each can ever reach (its **ceiling**);
- what a new card shows, to whom, before its member changes anything;
- whether the DID a member joins with can reach other members, or the public;
- whether administrators keep notes about members;
- whether the community has groups, whose members hold credentials that evidence their membership;
- whether a card can be published beyond the community.

The community **generates** the statement from its configuration each time it is read, so it describes the directory that runs, not the one someone intended, and cannot fall behind a change to it (VTI-DIR-130).

The same statement is carried in [`vtc/join-requests/manifest/0.4`](../../../join-requests/manifest/0.4/spec.md), which every applicant reads before applying. This task is how anyone else reads it — a member reviewing the terms they joined under, a prospective applicant comparing communities, a wallet showing it beside a join invitation — and how a reader fetches it without also fetching the join criteria.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **reader** (`issuer`) sends the request with no parameters.

A conforming **community** (`recipient`) that offers a directory **MUST**:

1. Answer every reader it answers [`vtc/join-requests/manifest`](../../../join-requests/manifest/0.4/spec.md) for, member or not. A community that publishes its join requirements to anyone answers this to anyone, with or without a `proof`; one that publishes them only to readers it can identify **MAY** do the same here, and **MUST NOT** be more restrictive than that, since an applicant who can read the criteria must be able to read the statement. A community **MAY** rate-limit reads, and **MUST NOT** tailor the statement to the reader: every reader at a given moment is given the same statement.
2. Generate the statement from the configuration in force when the request is answered, never from text an administrator wrote, as follows:
   - `fields`: every field whose state is `active`, base and community-defined, in display order, each with its `key`, `label`, `description` where it has one, `type`, `source`, `ceiling`, `defaultVisibility`, `searchable`, and `facet` where true. A disabled or retired field is not listed;
   - `newCard`: the listing, search setting and, where the community delivers introductions, the introductions setting a card has when the community creates it;
   - `membershipIdentifiers`: the ceiling of the base field `did`, or `admins` where that field is not active;
   - `adminNotes`: true where administrators can keep notes about members ([`vtc/directory/notes/append`](../../notes/append/0.1/spec.md) is served), false otherwise;
   - `groups`: present exactly where the community has groups, with the validity it gives each group membership credential;
   - `externalCards`: true where a member can turn on external publication, false where the community refuses it;
   - `statementDigest`: as defined below.
3. Never state an audience narrower than one a value can reach. A ceiling is the widest audience the field can reach whatever the directory policy decides, because the policy only narrows (VTI-DIR-021); a community whose policy narrows further **MAY** say so in its own documentation, but **MUST NOT** narrow the statement to match.

A community that offers no directory does not serve this task, and answers it as any unserved type ([SPEC §8](/SPEC.md#8-error-responses), `unsupportedType`).

A conforming **reader**:

1. **SHOULD** show the statement to a prospective applicant before they apply, in words rather than as the raw object — an applicant reading `"ceiling": "public"` has not been told that the value can be seen by anyone on the internet.
2. **SHOULD** record the `statementDigest` it showed, with the application it informed, so the applicant can later tell whether the terms changed after they joined.
3. **MUST NOT** treat the statement as describing any member's card: it states what can be disclosed, not what a member has chosen to disclose.

### Computing `statementDigest`

```
statementDigest = multibase( multihash( SHA-256( JCS( disclosure ∖ statementDigest ) ) ) )
```

`disclosure ∖ statementDigest` is the statement object with its `statementDigest` member removed and nothing else changed — `ext` included. `JCS` is [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785) serialized as UTF-8. The value is a `DigestMultibase`. SHA-256 and base58btc (`z`) are **RECOMMENDED**, and a reader compares decoded multihash bytes, not encoded strings. The statement carries no timestamp, so the digest changes exactly when what the statement says changes, and two reads of an unchanged configuration give the same digest.

## Authorization

This task is not consequential ([SPEC §3](/SPEC.md#3-terminology)): it changes nothing and discloses only the community's own configuration. It needs no authorization. A community that requires a `proof` does so to rate-limit or attribute reads, not to authorize them; see Security & Privacy.

## Definitions

- `DirectoryDisclosure`, `DisclosedField`, `NewCardDefaults`, `GroupDisclosure` — see [`disclosure-statement.schema.json`](../../_shared/0.1/disclosure-statement.schema.json).
- `Audience`, `Listing`, `Introductions` — see [`directory-card.schema.json`](../../_shared/0.1/directory-card.schema.json).
- **Ceiling** — the widest audience a field may ever reach in this community.

## Request

The reader (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json). The payload is empty.

### A prospective applicant reads the statement

No proof: the reader has not identified themselves, and need not.

```json
{
  "id": "urn:uuid:3c9a7e10-4d5b-4f6a-9b2c-000000000001",
  "type": "https://trusttasks.org/spec/vtc/directory/disclosure/0.1",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-19T09:00:00Z",
  "payload": {}
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/directory/disclosure/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). This task defines no extended error codes; failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### A working group's directory

This community lists a new member's name and role to other members and keeps everything else to administrators until the member chooses. A member may show their name and bio to anyone, and their DID to other members, but never further. The community asserts each member's chapter, which members see. Administrators keep notes, the community has groups whose credentials last 30 days, and nobody's card leaves the community. The `statementDigest` shown is the real value for the statement as printed.

```json
{
  "id": "urn:uuid:3c9a7e10-4d5b-4f6a-9b2c-000000000002",
  "type": "https://trusttasks.org/spec/vtc/directory/disclosure/0.1#response",
  "issuer": "did:example:community",
  "issuedAt": "2026-10-19T09:00:01Z",
  "threadId": "urn:uuid:3c9a7e10-4d5b-4f6a-9b2c-000000000001",
  "payload": {
    "communityDid": "did:example:community",
    "disclosure": {
      "fields": [
        {
          "key": "displayName",
          "label": "Name",
          "type": "text",
          "source": "member",
          "ceiling": "public",
          "defaultVisibility": "members",
          "searchable": true
        },
        {
          "key": "pronouns",
          "label": "Pronouns",
          "type": "text",
          "source": "member",
          "ceiling": "members",
          "defaultVisibility": "admins",
          "searchable": false
        },
        {
          "key": "bio",
          "label": "About",
          "type": "longText",
          "source": "member",
          "ceiling": "public",
          "defaultVisibility": "admins",
          "searchable": false
        },
        {
          "key": "role",
          "label": "Role",
          "type": "text",
          "source": "derived",
          "ceiling": "members",
          "defaultVisibility": "members",
          "searchable": false
        },
        {
          "key": "joinedAt",
          "label": "Member since",
          "type": "date",
          "source": "derived",
          "ceiling": "members",
          "defaultVisibility": "admins",
          "searchable": false
        },
        {
          "key": "did",
          "label": "Membership DID",
          "type": "did",
          "source": "derived",
          "ceiling": "members",
          "defaultVisibility": "admins",
          "searchable": false
        },
        {
          "key": "org.example.employer",
          "label": "Employer",
          "description": "Where you work, if you want members to know.",
          "type": "text",
          "source": "member",
          "ceiling": "members",
          "defaultVisibility": "admins",
          "searchable": true
        },
        {
          "key": "org.example.chapter",
          "label": "Chapter",
          "type": "enum",
          "source": "admin",
          "ceiling": "members",
          "defaultVisibility": "members",
          "searchable": false,
          "facet": true
        }
      ],
      "newCard": {
        "listing": "listed",
        "search": "byName",
        "introductions": "anyone"
      },
      "membershipIdentifiers": "members",
      "adminNotes": true,
      "groups": {
        "credentialValidity": "P30D"
      },
      "externalCards": false,
      "statementDigest": "zQmUeLYeywAWrRgAw5k8dtbYMxU8gq7zvZCh5K6ykjt8ZYw"
    }
  }
}```

Read as an applicant would be told it: *"Members will see your name, your role and your chapter. Everything else you add is seen only by administrators until you choose otherwise; you can show your name and bio to anyone, and your pronouns, employer, joining date and DID to members. Administrators may keep notes about you. If you join a group you will hold a credential, valid for 30 days at a time, that shows you are in it. Your card is never published outside the community."*

## Security & Privacy

### Data carried

The request carries nothing. The response carries the community's directory configuration: field definitions without their constraints, default settings and which features exist. No member's value, and nothing that reveals which members there are or how many.

### Correlation

The community maintainer's identifier scope is public: an applicant finds the community by its DID and asks it, and a pairwise identifier per reader would make the community differ by who asks. The reader's scope is pairwise and, without a proof, there is no reader identifier at all. The statement is the same for every reader, so reading it tells the community nothing about the reader beyond, at most, the attribution a proof carries.

### Retention

Nothing beyond whatever the community keeps of a read it can attribute. A community **SHOULD NOT** keep anything of an unattributed read beyond what rate limiting needs.

### Consent/purpose

The statement exists so that a person can decide whether to apply knowing what the directory would make of them, and so that members can see the terms the directory runs under. It is the directory's part of what a community issuing membership credentials states, under the Decentralized Trust Graph credential specifications, about whether and to whom it discloses its members' identifiers. Every audience in it is an upper bound: a member can narrow any of their own values, and the community's policy can narrow further, but nothing reaches a wider audience than the statement says.
