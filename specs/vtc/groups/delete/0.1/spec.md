---
slug: vtc/groups/delete
version: '0.1'
title: VTC Groups — Delete
summary: 'A group manager deletes a group: its bindings are withdrawn first, its members'' credential renewal stops and its authorization credential is revoked, and only then is the group removed.'
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
  rationale: Withdraws every member's admissions and grants that came through the group; must be attributable after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. Placing the document in time bounds the duplicate-execution window.
sideEffects:
  level: destructive
  rationale: Removes the group and its membership record, withdraws every room admission and access-control grant that only it provided, and revokes the group's authorization credential. None of it is restored by creating the group again.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: Names a group and a reason; returns counts.
retention:
  class: durable
  rationale: The deletion is why members lost what the group provided; the community keeps the record of it.
errorCodes:
- code: vtc/groups/delete:notFound
  meaning: No group the caller may manage has this identifier.
  retryable: false
- code: vtc/groups/delete:keyHolderUnavailable
  meaning: The community's VTA did not answer. The group still exists; the community retries the withdrawal, and the request may be sent again.
  retryable: true
related:
- vtc/groups/create
- vtc/groups/bindings/remove
---

## Abstract

The **VTC Groups — Delete** Trust Task deletes a group. It is remote-first: what the group provided is withdrawn before the group is forgotten, so that no member is left holding an admission or a grant whose source no longer exists.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who does not manage the group with `notFound` when they may not see it, and `permissionDenied` when they may.
2. Stop admitting anyone to the group at once.
3. Withdraw every binding first, as [`vtc/groups/bindings/remove`](../../bindings/remove/0.1/spec.md) does — room admissions and access-control grants whose only source is this group — then stop renewing every member's group credential, revoke the group's authorization credential through the community's own status list, and only then delete the group and its membership record.
4. Answer `deleting` when a withdrawal is still in progress, and complete it with its reconciler, the one owner of its retries.
5. Treat the withdrawal of grants from a group carrying authority as a reduction of each affected member's authority, on the terms it applies to any reduction.

## Authorization

**Authority: `vtc.groups.manage`** for the group, read from the signer's access-control entry at execution time, under the community's `groups` policy purpose. A community **SHOULD** treat deleting a group carrying authority as it treats the reductions it causes. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- **Reconciler** — the community's process that brings members' room admissions, access-control grants and credentials into line with groups and bindings; the one owner of retries for them.

## Request

The group manager (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Delete a group

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000045",
  "type": "https://trusttasks.org/spec/vtc/groups/delete/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000044",
  "payload": {
    "groupId": "grp_W5k2N8xRb3Qz7TpL",
    "reason": "Working group concluded."
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/groups/delete/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Delete a group — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000046",
  "type": "https://trusttasks.org/spec/vtc/groups/delete/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000044",
  "payload": {
    "groupId": "grp_W5k2N8xRb3Qz7TpL",
    "state": "deleted",
    "bindingsRemoved": 1,
    "membersRemoved": 14
  }
}
```

## Security & Privacy

### Data carried

A group identifier and a reason.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The group's own DID is minted for that group alone, so the community's groups cannot be linked to one another through their identifiers; its DID document carries no name or description. A group's membership credential, where a member presents it, links that member's membership DID to the group — which is what the member presents it for.

### Retention

**Durable** for the record of the deletion. The membership record is deleted with the group.

### Consent/purpose

Ending a group the community no longer needs. **A group's membership credential is evidence for others, never for the community.** Rooms, peers and other communities may rely on it. The community **MUST NOT** consult it, or the group's record, to decide any authorization of its own: authority at the community comes only from its access-control list, into which a group carrying authority provisions resource grants whose source is the group.
