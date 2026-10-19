---
slug: vtc/groups/managers/remove
version: '0.1'
title: VTC Groups — Managers — Remove
summary: A manager steps down, or an administrator who manages every group withdraws another member's appointment as a group's manager.
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
  rationale: Withdraws a member's authority over a group. Must be attributable after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed removal would remove a manager since appointed again.
sideEffects:
  level: mutating
  rationale: Removes a resource grant from an access-control entry. Recoverable with vtc/groups/managers/add.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: Names the manager by entry and returns the group's managers by entry, never by DID.
retention:
  class: durable
  rationale: The removal is why the member no longer manages the group.
errorCodes:
- code: vtc/groups/managers/remove:notFound
  meaning: No group the caller may manage has this identifier.
  retryable: false
- code: vtc/groups/managers/remove:notManager
  meaning: The entry holds no appointment to manage this group.
  retryable: false
related:
- vtc/groups/managers/add
- vtc/groups/show
---

## Abstract

The **VTC Groups — Managers — Remove** Trust Task ends a member's appointment as a group's manager: the resource grant `vtc.groups.manage` qualified to the group is removed from their access-control entry. A manager may step down; only an administrator who manages every group removes somebody else.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who does not manage the group with `notFound` when they may not see it, and `permissionDenied` when they may.
2. Refuse with `permissionDenied` a caller removing an entry other than their own unless they hold `vtc.groups.manage` unqualified. Managers of one group do not remove one another.
3. Refuse an entry that holds no grant of `vtc.groups.manage` qualified to this group with `notManager`. Holding the capability unqualified is not an appointment this task removes.
4. Remove that grant from the entry, leaving every other grant and the entry's role untouched, and audit the change by reference.

The removal ends the member's authority over the group's membership and settings. Members they added, and requests they decided, stand.

## Authorization

**Authority: `vtc.groups.manage`** — qualified `group:<groupId>` to step down from that group, unqualified to remove another member's appointment — read from the signer's access-control entry at execution time. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `GroupId` — see [`group.schema.json`](../../../_shared/0.1/group.schema.json).
- `EntryId` — see [`directory-card.schema.json`](../../../../directory/_shared/0.1/directory-card.schema.json).

## Request

The group manager (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Step down

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-0000000000ab",
  "type": "https://trusttasks.org/spec/vtc/groups/managers/remove/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-19T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-0000000000aa",
  "payload": {
    "groupId": "grp_W5k2N8xRb3Qz7TpL",
    "entryId": "dir_Q7m2K9xVb4Tz8LpR"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:carol#key-1",
    "created": "2026-10-19T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/groups/managers/remove/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Step down — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-0000000000ac",
  "type": "https://trusttasks.org/spec/vtc/groups/managers/remove/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-19T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-0000000000aa",
  "payload": {
    "groupId": "grp_W5k2N8xRb3Qz7TpL",
    "managers": []
  }
}
```

## Security & Privacy

### Data carried

A group and an entry; the group's managers by entry.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Managers are named by their community-scoped `entryId`, never by DID.

### Retention

**Durable**: the audit row records why the appointment ended.

### Consent/purpose

Ending a delegation of a group's running. Nothing the manager did while appointed is undone.
