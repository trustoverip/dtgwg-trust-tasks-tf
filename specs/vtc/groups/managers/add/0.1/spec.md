---
slug: vtc/groups/managers/add
version: '0.1'
title: VTC Groups — Managers — Add
summary: A group manager, or an administrator who manages every group, appoints a community member a manager of one group — an access-control resource grant of vtc.groups.manage qualified to that group.
status: draft
targetFrameworkVersion: 0.6.0
category: governance
parties:
- role: group manager
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: Gives a member authority over a group's membership. Must be attributable after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed appointment would restore a manager since removed.
sideEffects:
  level: mutating
  rationale: Writes a resource grant onto the appointee's access-control entry. Recoverable with vtc/groups/managers/remove.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: Names the appointee by entry and returns the group's managers by entry, never by DID.
retention:
  class: durable
  rationale: The grant is the appointee's authority over the group for as long as it stands.
errorCodes:
- code: vtc/groups/managers/add:notFound
  meaning: No group the caller may manage has this identifier.
  retryable: false
- code: vtc/groups/managers/add:entryNotFound
  meaning: No active member has this entry.
  retryable: false
- code: vtc/groups/managers/add:alreadyManager
  meaning: The member already manages the group, by this grant or by managing every group.
  retryable: false
related:
- vtc/groups/managers/remove
- vtc/groups/show
- vtc/groups/members/add
---

## Abstract

The **VTC Groups — Managers — Add** Trust Task appoints a community member a manager of one group. A manager adds and removes the group's members, decides its join requests, and changes its settings ([`vtc/groups/update`](../../../update/0.1/spec.md)).

The appointment is not a field of the group. It is an **access-control resource grant** on the appointee's entry — `vtc.groups.manage` qualified `group:<groupId>` — so the community's access-control list stays the one place its authority is recorded, and every gate reads it there.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who does not manage the group with `notFound` when they may not see it, and `permissionDenied` when they may.
2. Refuse an entry that is not an active member with `entryNotFound`, and one that already manages the group — by a grant qualified to it, or by holding `vtc.groups.manage` unqualified — with `alreadyManager`.
3. Write the appointment as a resource grant on the appointee's access-control entry: capability `vtc.groups.manage`, qualifier `group:<groupId>`, delegated by the signer. The grant sits beside the entry's administrative role, never inside its ceiling, so a member with no administrative role may hold it.
4. Bound the grant by the signer's own holding: the signer covers `vtc.groups.manage` for this group at the moment of writing, and a grant the signer does not cover is never written.
5. Treat the grant as any other delegated grant when its granter departs, or no longer covers it: it is reviewed on the community's terms for a departed granter's grants, and is not left standing silently.
6. Withdraw the grant when the group is deleted, and when the appointee leaves the community.
7. Audit the appointment by reference — the group, the appointee's entry and the granter — and **MUST NOT** treat a group's record as the source of who manages it.

A manager's authority is over the group's membership and settings. It confers nothing else: adding a member to a group that carries authority is still an authority-conferring grant, held for the consent such a grant requires ([`vtc/groups/members/add`](../../../members/add/0.1/spec.md)), whoever the manager is.

## Authorization

**Authority: `vtc.groups.manage`** for the group — unqualified, or qualified `group:<groupId>` — read from the signer's access-control entry at execution time. A manager may appoint a co-manager of the group they manage; only a holder of the unqualified capability appoints across groups. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `GroupId` — see [`group.schema.json`](../../../_shared/0.1/group.schema.json).
- `EntryId` — see [`directory-card.schema.json`](../../../../directory/_shared/0.1/directory-card.schema.json).

## Request

The group manager (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Appoint a manager

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000009b",
  "type": "https://trusttasks.org/spec/vtc/groups/managers/add/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-19T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000009a",
  "payload": {
    "groupId": "grp_W5k2N8xRb3Qz7TpL",
    "entryId": "dir_Q7m2K9xVb4Tz8LpR",
    "reason": "Carol convenes the working group."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:administrator#key-1",
    "created": "2026-10-19T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/groups/managers/add/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Appoint a manager — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000009c",
  "type": "https://trusttasks.org/spec/vtc/groups/managers/add/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-19T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000009a",
  "payload": {
    "groupId": "grp_W5k2N8xRb3Qz7TpL",
    "managers": ["dir_Q7m2K9xVb4Tz8LpR"]
  }
}
```

## Security & Privacy

### Data carried

A group and an entry; the group's managers by entry.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Managers are named by their community-scoped `entryId`, never by DID.

### Retention

**Durable** for the grant, until it is removed, its group is deleted, or its holder leaves the community.

### Consent/purpose

Delegating the running of a group to a member. The grant is authority at the community, so it lives in the access-control list and is reviewed with every other delegated grant; the group's record never decides who manages it.
