---
slug: vtc/groups/bindings/list
version: '0.1'
title: VTC Groups — Bindings — List
summary: 'A group manager or administrator lists what a group is bound to: the rooms its members are admitted to and the grants it confers.'
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
  requirement: RECOMMENDED
  rationale: Read by managers only.
sideEffects:
  level: none
  rationale: Reads bindings; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses the rooms and capabilities bound to a group.
retention:
  class: transient
  rationale: A read.
errorCodes:
- code: vtc/groups/bindings/list:notFound
  meaning: No group the caller may manage has this identifier.
  retryable: false
related:
- vtc/groups/bindings/add
- vtc/groups/bindings/remove
---

## Abstract

The **VTC Groups — Bindings — List** Trust Task returns a group's bindings: **room** bindings, which admit the group's members to a community-owned room, and **grants** and **role** bindings, which confer capabilities through the access-control list.

**Why there is no `show` and no paging.** A group has at most 64 bindings, always read whole; a binding is addressed by identifier only to remove it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST** return a group's bindings only to holders of `vtc.groups.manage` for the group, and answer `notFound` alike for a group that does not exist and one the caller may not manage.

## Authorization

**Authority: `vtc.groups.manage`** for the group, read from the signer's access-control entry at execution time ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `GroupBinding` — see [`group.schema.json`](../../../_shared/0.1/group.schema.json).

## Request

The group manager (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A group's bindings

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000066",
  "type": "https://trusttasks.org/spec/vtc/groups/bindings/list/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000065",
  "payload": {
    "groupId": "grp_W5k2N8xRb3Qz7TpL"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/groups/bindings/list/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### A group's bindings — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000067",
  "type": "https://trusttasks.org/spec/vtc/groups/bindings/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000065",
  "payload": {
    "items": [
      {
        "bindingId": "gb_P2k8W5qYc6Vd1MsK",
        "binding": {
          "kind": "room",
          "roomId": "did:webvh:QmRoom:community.example:rooms:wg-identity",
          "actions": [
            "read",
            "write"
          ],
          "validFor": "P365D"
        },
        "state": "active",
        "createdAt": "2026-10-02T09:00:00Z",
        "createdBy": "did:example:administrator"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Room identifiers and capability references bound to a group.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Members are named by their community-scoped `entryId`, never by DID. The group's own DID is minted for that group alone, so the community's groups cannot be linked to one another through their identifiers; its DID document carries no name or description. A group's membership credential, where a member presents it, links that member's membership DID to the group — which is what the member presents it for.

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

**A group's membership credential is evidence for others, never for the community.** Rooms, peers and other communities may rely on it. The community **MUST NOT** consult it, or the group's record, to decide any authorization of its own: authority at the community comes only from its access-control list, into which a group carrying authority provisions resource grants whose source is the group.
