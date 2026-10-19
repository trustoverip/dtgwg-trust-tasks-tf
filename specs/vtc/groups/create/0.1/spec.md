---
slug: vtc/groups/create
version: '0.1'
title: VTC Groups — Create
summary: 'An administrator creates a community group: the community''s VTA mints the group''s own DID and, where it issues credentials, the community authorises it to issue membership credentials; the group is recorded only afterwards.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
parties:
- role: group administrator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: Creates an identity the community answers for and, where the group issues credentials, an authorization credential in the community's name. It must be attributable to its requester after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed create would mint a second group and a second DID.
sideEffects:
  level: mutating
  rationale: Has the community's VTA mint a DID, has the community issue the group an authorization credential, and stores the group. Recoverable with vtc/groups/delete.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: The request carries a name, description, kind, rule and policies; the response is the group record, never a key or a credential.
retention:
  class: durable
  rationale: The group persists until deleted, and its DID and authorization credential are the root its members' credentials chain to.
errorCodes:
- code: vtc/groups/create:ruleInvalid
  meaning: The rule is malformed, reads a fact that is not verified (a `member`-sourced field), or is present on a static group or absent on a dynamic one.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      field:
        type: string
        maxLength: 128
      reason:
        type: string
        maxLength: 500
- code: vtc/groups/create:kindNotServed
  meaning: The community serves only static groups. A dynamic group is not created; nothing was stored.
  retryable: false
- code: vtc/groups/create:settingsInvalid
  meaning: '`joinPolicy` on a dynamic group, or `showMembers` wider than `visibility`.'
  retryable: false
- code: vtc/groups/create:policyRefused
  meaning: The community's `groups` policy does not let this caller create this group.
  retryable: false
- code: vtc/groups/create:noKeyHolder
  meaning: The community has no link to a VTA that can hold the group's keys. Its host's operator configures one.
  retryable: false
- code: vtc/groups/create:keyHolderUnavailable
  meaning: The community's VTA did not answer. Nothing was stored; a DID it may have minted is named in `details.groupDid` so an operator can retire it.
  retryable: true
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      groupDid:
        type: string
        maxLength: 2048
- code: vtc/groups/create:keyHolderRefused
  meaning: The community's VTA refused a step. Its own code and message are carried in `details`. Nothing was stored.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    properties:
      code:
        type: string
        maxLength: 128
      message:
        type: string
        maxLength: 500
related:
- vtc/groups/show
- vtc/groups/update
- vtc/groups/delete
- vtc/groups/members/add
- vtc/rooms/community/create
- keys/sign-document
---

## Abstract

The **VTC Groups — Create** Trust Task creates a community group — a working group, a chapter, a cohort. Every group has **its own DID**, held by the community's own VTA as community rooms are, so that the group can issue its members a membership credential that rooms, peers and other communities can verify.

## The steps, and their order

The community, on a valid request:

1. Has its VTA mint the group's DID from the `vtc-group` DID template, in a context the community may act in. The template is `did:webvh` with one signing key and one key-agreement key, **no `service`** and no label or name variable: nothing serves messages sent to a group yet, and when group conversations exist a messaging service is added by a DID update. The DID document therefore carries no name or description. A serverless group DID is served by the community at `/group-dids/<groupId>/did.jsonl`.
2. Where `issuesCredentials` is true (the default), issues the group an authorization credential — a DTG `AuthorityCredential` (VAC) from the community to the group's DID — authorising the group to issue membership credentials, and keeps its expiry as `credentialAuthorityValidUntil`. Its VTA signs it **as the community's DID** through the generic `keys/sign-document/0.1` Trust Task (specified separately).
3. **Then**, and only then, stores the group.

Every remote step comes before the one local write. A failure in steps 1–2 leaves no group behind; at most a DID the VTA minted and nobody uses, which `keyHolderUnavailable` names. The community never holds the group's keys and never falls back to holding them.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller without unqualified `vtc.groups.manage` with `permissionDenied`, and refuse with `policyRefused` where its `groups` policy refuses.
2. Refuse a rule on a static group, a missing rule on a dynamic group, or a rule that reads anything but verified facts — community role, `admin`- or `credential`-sourced fields, personhood, join date — with `ruleInvalid`. A rule **MUST NOT** read a `member`-sourced field.
3. Refuse `joinPolicy` on a dynamic group, and `showMembers` wider than `visibility`, with `settingsInvalid`. A community that serves only static groups **MAY** refuse a dynamic one with `kindNotServed` once its rule has been checked; it **MUST NOT** store a dynamic group it does not compute.
4. Perform the steps above in order, store the group only after they succeed, refuse with `noKeyHolder` when it has no VTA to hold the group's keys, and **MUST NOT** hold any group key itself.
5. Issue the group's authorization credential with an expiry, and renew it well before it lapses for as long as the group exists.
6. Create the group with no members and no bindings; a dynamic group's members are computed after it is stored.
7. **MUST NOT** return or log a key or a credential.

## Authorization

**Authority: unqualified `vtc.groups.manage`** at the community, read from the signer's access-control entry at execution time, under its `groups` policy purpose, which may refuse. A qualified holder (`group:<groupId>`) manages one existing group and cannot create one. Creating a group confers nothing on anyone: authority reaches a group only through a binding ([`vtc/groups/bindings/add`](../../bindings/add/0.1/spec.md)). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

**The community acts through its key holder.** The community's VTA mints and holds the group's DID; the community sends it the requests a key holder serves and the VTA applies its own gates to each.

## Definitions

- `Group`, `DynamicRule` — see [`group.schema.json`](../../_shared/0.1/group.schema.json).
- **The community's VTA** — the key holder the community reaches over its own messaging link, as it reaches it for its other VTA-held identities.

## Request

The group administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A working group members can ask to join

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000003d",
  "type": "https://trusttasks.org/spec/vtc/groups/create/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000003c",
  "payload": {
    "name": "Working Group: Identity",
    "description": "Drafts the community's identity profile.",
    "kind": "static",
    "joinPolicy": "request",
    "visibility": "members",
    "showMembers": "groupMembers",
    "reason": "Chartered at the October meeting."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:administrator#key-1",
    "created": "2026-10-18T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/groups/create/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### A working group members can ask to join — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000003e",
  "type": "https://trusttasks.org/spec/vtc/groups/create/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000003c",
  "payload": {
    "group": {
      "groupId": "grp_W5k2N8xRb3Qz7TpL",
      "groupDid": "did:webvh:QmGroupScid:community.example:group-dids:grp_W5k2N8xRb3Qz7TpL",
      "name": "Working Group: Identity",
      "description": "Drafts the community's identity profile.",
      "kind": "static",
      "joinPolicy": "request",
      "visibility": "members",
      "showMembers": "groupMembers",
      "issuesCredentials": true,
      "carriesAuthority": false,
      "memberCount": 0,
      "createdAt": "2026-10-01T08:00:00Z",
      "managers": [],
      "credentialAuthorityValidUntil": "2027-10-18T09:00:00Z"
    }
  }
}
```

### Refused: a dynamic rule reading a self-asserted field

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000003f",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000040",
  "payload": {
    "code": "vtc/groups/create:ruleInvalid",
    "message": "org.example.employer is member-sourced; a rule reads verified facts only.",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

A name, description, kind, rule and policies in; the group record out. The group's keys are created inside the community's VTA and never leave it. A producer **MUST NOT** place personal data in `name`, `description`, `reason` or `ext`.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The group's own DID is minted for that group alone, so the community's groups cannot be linked to one another through their identifiers; its DID document carries no name or description. A group's membership credential, where a member presents it, links that member's membership DID to the group — which is what the member presents it for.

### Retention

**Durable.** The group persists until deleted.

### Consent/purpose

The community organising its members. **A group's membership credential is evidence for others, never for the community.** Rooms, peers and other communities may rely on it. The community **MUST NOT** consult it, or the group's record, to decide any authorization of its own: authority at the community comes only from its access-control list, into which a group carrying authority provisions resource grants whose source is the group.
