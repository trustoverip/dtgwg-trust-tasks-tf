---
slug: vtn/memberships/leave
version: "0.1"
title: "VTN Memberships — Leave"
summary: "An administrator withdraws this network from a parent network: the network repudiates the parent's membership grant, signing as itself."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, nested-membership, departure, repudiation]
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
    The task makes the network sign a public statement to a third party — the parent — on the authority of the administrator who asked, so the asker has to be attributable on every transport and afterwards in the audit trail.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replay would repeat a departure the network may since have reversed by re-joining, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Signs a repudiation of the parent's grant as the network and sends it to the parent, which records the departure and stops delivering to this network's groups; the membership moves to departed. Reversed only by joining afresh with vtn/memberships/join.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: the parent's DID. Out: the membership record — the parent's and this network's public identifiers, this network's group identifiers and the credentials and signed exchanges between them. No personal data.
retention:
  class: durable
  rationale: >-
    The record is the network's evidence of when and how it left the parent; the change is in the audit trail.
errorCodes:
  - code: vtn/memberships/leave:notMember
    meaning: "This network is not an acknowledged member of `network`."
    retryable: false
  - code: vtn/memberships/leave:networkRefused
    meaning: "The parent could not be reached or refused the repudiation; the membership is unchanged. `details.code` and `details.message` carry the parent's error when it gave one."
    retryable: true
    detailsSchema:
      type: object
      properties:
        code:
          type: string
        message:
          type: string
related:
  - vtn/memberships/join
  - vtn/memberships/repudiate
  - vtn/memberships/list
  - vtn/membership/repudiate
  - vtc/networks/leave
---

## Abstract

This network's membership of a parent is its acknowledgement of the parent's grant (VTI-VTN-013). **VTN Memberships — Leave** withdraws it: the network signs, as itself, a repudiation of the grant and sends it to the parent through [`vtn/membership/repudiate`](../../../membership/repudiate/0.1/spec.md); the parent then records the departure and revokes what it issued for this network ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-VTN-053). It is the mirror of [`vtc/networks/leave`](../../../../vtc/networks/leave/0.2/spec.md). Leaving changes nothing about this network's own members, which were never members of the parent (VTI-VTN-033, VTI-VTN-063).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **network** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`, and with `vtn/memberships/leave:notMember` when it holds no membership of `network` in state `acknowledged`.
2. **MUST** sign the repudiation as this network, for `assertionMethod`, naming the grant it holds by its digest, and send it with `vtn/membership/repudiate` to the parent at the `url` recorded for the membership.
3. **MUST** move the membership to `departed`, with the repudiation, only once the parent has accepted it, and **MUST** otherwise leave the membership unchanged and refuse with `vtn/memberships/leave:networkRefused`.
4. **MUST** then stop accepting [`vtn/relay/deliver`](../../../relay/deliver/0.1/spec.md) from `network` for the groups it had registered there, keeping the groups' registrations and acceptances in the record as evidence.
5. **MUST** record the change in its audit trail naming the administrator, and answer with the membership.

## Authorization

The authority this task presupposes is **administrator standing at this network**: the signer is one of the administrators in the network's configuration, read at execution time, or a key one of them delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md), in which case it is the standing of the administrator the key acts for. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A network **MAY** require a further, operation-bound step-up before leaving a parent; that is its policy, not this specification's.

## Definitions

- **`network`** — the parent network's DID, as recorded by [`vtn/memberships/join`](../../join/0.1/spec.md).
- **Repudiation** — a statement credential issued by this network, predicate `https://registry.trustoverip.org/dtg/vsc/repudiates/1`, whose subject is the parent and whose `object.digestMultibase` is the grant's digest. Repudiating a grant is departure.
- **`membership`** — the record ([`UpstreamMembership`](../../../_shared/0.1/membership.schema.json)), now in state `departed`.

## Request

A network administrator (`issuer`) asks this network (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Leave a parent

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000d41",
  "type": "https://trusttasks.org/spec/vtn/memberships/leave/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:regional",
  "issuedAt": "2026-11-03T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000dff",
  "payload": {
    "network": "did:example:whn"
  }
}
```

## Response

This network answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### Departed

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000d42",
  "type": "https://trusttasks.org/spec/vtn/memberships/leave/0.1#response",
  "issuer": "did:example:regional",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-03T10:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000dff",
  "payload": {
    "membership": {
      "network": "did:example:whn",
      "url": "https://whn.example.org",
      "name": "World Health Network",
      "state": "departed",
      "listing": { "id": "urn:uuid:00000000-0000-4000-8000-0000000d0011" },
      "grant": { "id": "urn:uuid:00000000-0000-4000-8000-0000000d0012" },
      "acknowledgement": { "id": "urn:uuid:00000000-0000-4000-8000-0000000d0013" },
      "repudiation": { "id": "urn:uuid:00000000-0000-4000-8000-0000000d0041" },
      "groups": [
        {
          "parentRole": "admins",
          "localRole": "admins",
          "group": "did:example:regional#role-admins",
          "registration": { "id": "urn:uuid:00000000-0000-4000-8000-0000000d0014" },
          "acceptance": { "id": "urn:uuid:00000000-0000-4000-8000-0000000d0015" }
        }
      ],
      "updatedAt": "2026-11-03T10:00:01Z"
    }
  }
}
```

The credentials, and the signed `vtn/groups/register` request and response, are abbreviated to their `id`; the network returns them in full.

## Security & Privacy

### Data carried

The request carries the parent's DID. The response carries the membership record: the two networks' public identifiers, this network's network-group identifiers, and the credentials and signed exchanges between them, now including the repudiation. No personal data is carried, and the repudiation carries no reason: the parent publishes it.

### Correlation

The parent publishes the departure, by design: a verifier must be able to see that the membership has ended. Both networks are named by their public identifiers, which is why this network declares `identifierScope: public`; a pairwise identifier would be one the parent's grant and index could not refer to. The administrator declares `identifierScope: any`: the parent sees only this network's signature.

### Retention

The network keeps the departed membership, with every credential and exchange in it, as evidence of what it agreed to and when it withdrew. The change is recorded in its audit trail against the administrator who made it.

### Consent/purpose

The repudiation withdraws the consent this network gave by acknowledging the grant, and is used for nothing else.
