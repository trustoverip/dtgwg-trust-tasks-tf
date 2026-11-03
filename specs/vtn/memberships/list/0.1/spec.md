---
slug: vtn/memberships/list
version: "0.1"
title: "VTN Memberships — List"
summary: "An administrator lists the parent networks this network belongs to, has repudiated the listing of, or has left."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, nested-membership, vmc, curation]
parties:
  - role: network administrator
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
    Only administrators may read the answer, which discloses which parents the network has accepted and the group identifiers it gave them, so the reader has to be identified by more than the transport.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read changes nothing, so a replay discloses only what the original disclosed; a timestamp still lets the network bound how long a captured request stays usable.
sideEffects:
  level: none
  rationale: "Reads the network's upstream membership records; changes nothing and contacts no parent."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Out: parent network DIDs and URLs, this network's group identifiers, and the credentials and signed exchanges between this network and each parent. No personal data.
retention:
  class: transient
  rationale: >-
    The request carries nothing to keep; the records it reads are kept by the tasks that create them.
errorCodes: []
related:
  - vtn/memberships/join
  - vtn/memberships/repudiate
  - vtn/memberships/leave
  - vtn/memberships/groups/add
  - vtc/networks/list
---

## Abstract

A Verifiable Trust Network can itself be a member of other networks — its **parents** ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-VTN-030). **VTN Memberships — List** answers which parents this network has dealt with: those it is a member of, those whose listing it repudiated, and those it left, each with the credentials that evidence the state and the groups it registered for the parent's roles. It is the mirror of [`vtc/networks/list`](../../../../vtc/networks/list/0.1/spec.md) for a network that is a member.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **network** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`.
2. **MUST** answer only with memberships it holds a record of — parents it joined, repudiated or left through [`vtn/memberships/join`](../../join/0.1/spec.md), [`vtn/memberships/repudiate`](../../repudiate/0.1/spec.md) and [`vtn/memberships/leave`](../../leave/0.1/spec.md) — never with a parent that merely lists it. A listing this network has not acted on is the parent's assertion, not membership (VTI-VTN-010, VTI-VTN-013).
3. **MUST** return each membership in full, with its `state`, and **SHOULD** order them by parent DID.

## Authorization

The authority this task presupposes is **administrator standing at this network**: the signer is one of the administrators in the network's configuration, read at execution time, or a key one of them delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md). Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The task is not open to listed parties, peer networks or the public, whatever the network's directory visibility.

## Definitions

- **`memberships`** — this network's memberships of parent networks ([`UpstreamMembership`](../../../_shared/0.1/membership.schema.json)). `state` is `acknowledged` (a member), `repudiated` (it rejected the parent's listing) or `departed` (it left).

## Request

A network administrator (`issuer`) asks this network (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json); it carries no members.

### This network's parents

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000d01",
  "type": "https://trusttasks.org/spec/vtn/memberships/list/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:regional",
  "issuedAt": "2026-11-03T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000dff",
  "payload": {}
}
```

## Response

This network answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### A member of one parent, departed from another

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000d02",
  "type": "https://trusttasks.org/spec/vtn/memberships/list/0.1#response",
  "issuer": "did:example:regional",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-03T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000dff",
  "payload": {
    "memberships": [
      {
        "network": "did:example:global",
        "url": "https://global.example.org",
        "state": "departed",
        "listing": { "id": "urn:uuid:00000000-0000-4000-8000-0000000d0001" },
        "grant": { "id": "urn:uuid:00000000-0000-4000-8000-0000000d0002" },
        "acknowledgement": { "id": "urn:uuid:00000000-0000-4000-8000-0000000d0003" },
        "repudiation": { "id": "urn:uuid:00000000-0000-4000-8000-0000000d0004" },
        "groups": [],
        "updatedAt": "2026-10-20T09:00:00Z"
      },
      {
        "network": "did:example:whn",
        "url": "https://whn.example.org",
        "name": "World Health Network",
        "state": "acknowledged",
        "listing": { "id": "urn:uuid:00000000-0000-4000-8000-0000000d0011" },
        "grant": { "id": "urn:uuid:00000000-0000-4000-8000-0000000d0012" },
        "acknowledgement": { "id": "urn:uuid:00000000-0000-4000-8000-0000000d0013" },
        "groups": [
          {
            "parentRole": "admins",
            "localRole": "admins",
            "group": "did:example:regional#role-admins",
            "registration": { "id": "urn:uuid:00000000-0000-4000-8000-0000000d0014" },
            "acceptance": { "id": "urn:uuid:00000000-0000-4000-8000-0000000d0015" }
          }
        ],
        "updatedAt": "2026-11-03T10:00:02Z"
      }
    ]
  }
}
```

The credentials, and the signed `vtn/groups/register` request and response, are abbreviated to their `id`; the network returns them in full.

## Security & Privacy

### Data carried

The request carries nothing. The response carries the parents' DIDs, URLs and names, this network's network-group identifiers, and the credentials and signed exchanges between this network and each parent (listing, grant, acknowledgement, repudiation, group registrations and acceptances). No personal data is carried.

### Correlation

The answer names public identifiers of organisations — the parents and this network — whose relationships the parents publish anyway, except for a parent this network has repudiated or left, which the answer still discloses to the administrator. This network declares `identifierScope: public` because it is the network its parents list and its credentials name; a pairwise identifier would be one those credentials could not refer to. The administrator declares `identifierScope: any`: their identifier stays between them and the network.

### Retention

Nothing in the request is kept beyond the exchange. The records it reads are durable, kept by the tasks that create them.

### Consent/purpose

The answer exists so administrators can see and manage this network's memberships of parent networks; it is not reused for any other purpose.
