---
slug: vtn/groups/list
version: "0.1"
title: "VTN Groups — List"
summary: "An administrator lists the groups members have registered to serve the network's roles, each with the signed exchange that registered it."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, administration, roles, groups, broadcast]
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: network
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The proof is the administrator's authentication: the network answers only a signer in its administrator list or a key one of them delegated, and the list includes the endpoints where members receive broadcasts.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read with no side effects; a replay discloses only what the administrator was already entitled to see. A network can still bound how long a signed request is honoured.
sideEffects:
  level: none
  rationale: >-
    Reads the network's group registrations; changes nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Out: each registration — the member's and group's identifiers, the delivery endpoint, and the signed request and response that registered it. In: at most a role name.
retention:
  class: transient
  rationale: >-
    Nothing in the request is worth keeping; the registrations are the network's own records.
errorCodes: []
related:
  - vtn/groups/register
  - vtn/roles/list
  - vtn/broadcasts/send
---

## Abstract

A Verifiable Trust Network defines roles, and each member registers a group to serve a role through [`vtn/groups/register`](../../register/0.1/spec.md): the member's signed request and the network's revocable groupRegistered credential accepting it together are the evidence of the registration. The network delivers broadcasts for a role to the endpoints of the groups registered for it. **VTN Groups — List** lets an administrator see those registrations, for every role or one.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **administrator** (`issuer`):

1. **MUST** sign the request as an administrator, or with a key an administrator delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md).
2. **MUST**, before relying on a registration as evidence outside the network, verify its `registration` as a `vtn/groups/register` request signed by `member` and its `acceptance` as the network's groupRegistered credential over that request's digest, with a status that resolves as valid, as [`vtn/groups/register`](../../register/0.1/spec.md) states for a verifier.

A conforming **network** (`recipient`):

1. **MUST** refuse with `permissionDenied` unless the signer is an administrator — in the administrator list of its configuration, read at execution time — or a key an administrator delegated and has not revoked.
2. **MUST** return every group registration it holds; with `role`, only those for that role. A `role` the network does not define yields an empty list.
3. **MUST** return each registration with the member's signed request exactly as received and the groupRegistered credential it issued for it ([`RegisteredGroup`](../../../_shared/0.1/groups.schema.json)), so that the pair stays verifiable.
4. **MUST NOT** register, change or remove a group in answering; a group is the member's, registered by the member (VTI-VTN-060).

## Authorization

The authority this task presupposes is **administrator standing**: being named in the network's administrator list when the request is executed, or holding a signing key such an administrator delegated through `auth/signing-key/enroll` and has not revoked. The proof establishes who signed; the match against the administrator list is what entitles the requester (network item 1).

## Definitions

- **`role`** — when given, only groups registered for this role.
- **`groups`** — the registrations ([`RegisteredGroup`](../../../_shared/0.1/groups.schema.json)): the member, the role, the group, the endpoint broadcasts for it are delivered to, the signed `registration`, the `acceptance` credential, and when it was accepted.

This family has no `show` task for a single registration: a registration is identified by its member and role, at most one per pair, and an administrator reads one by filtering on `role` and finding the member; its existence is never load-bearing for a caller, because the member, not the administrator, is the one who registers or replaces it.

## Request

An administrator (`issuer`) asks the network (`recipient`) for its group registrations. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### The groups serving one role

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000581",
  "type": "https://trusttasks.org/spec/vtn/groups/list/0.1#request",
  "issuer": "did:example:admin",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T10:00:00Z",
  "payload": {
    "role": "clinical-alerts"
  }
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### One registration

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000582",
  "type": "https://trusttasks.org/spec/vtn/groups/list/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:admin",
  "issuedAt": "2026-11-03T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000581",
  "payload": {
    "groups": [
      {
        "member": "did:example:community",
        "role": "clinical-alerts",
        "group": "did:example:community-alerts",
        "endpoint": "https://stvincents.example/v1/trust-tasks",
        "registration": { "id": "urn:uuid:00000000-0000-4000-8000-000000000301" },
        "acceptance": { "id": "urn:uuid:00000000-0000-4000-8000-000000000302" },
        "registeredAt": "2026-10-21T14:00:00Z"
      }
    ]
  }
}
```

The signed request and response are abbreviated to their `id`; the network returns them in full.

## Security & Privacy

### Data carried

The response carries members' and groups' identifiers, the endpoints broadcasts are delivered to, and the signed documents that registered them. The request carries at most a role name.

### Correlation

The network declares `identifierScope: public` because it is an organisation whose DID is how readers find and verify it, and members are organisations whose DIDs are public for the same reason. The administrator declares `identifierScope: any`: the network recognises its administrators by its own configuration, so an administrator may use an identifier it uses for nothing else. Endpoints are not published in the directory; the list shows them to administrators only, because they are where the network delivers.

### Retention

Nothing in the request is retained beyond the network's ordinary request log.

### Consent/purpose

Each registration was made by the member itself, for the purpose of receiving the role's broadcasts; the list lets administrators see who will receive them, and is used for nothing else.
