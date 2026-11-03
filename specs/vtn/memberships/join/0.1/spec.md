---
slug: vtn/memberships/join
version: "0.1"
title: "VTN Memberships — Join"
summary: "An administrator accepts a parent network's listing of this network: the network registers its network groups for the parent's roles and acknowledges the parent's membership offer, signing as itself."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, nested-membership, vmc, curation, acknowledge]
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
    The task makes the network sign statements to a third party — its parent — on the authority of the administrator who asked, so the asker has to be attributable on every transport and afterwards in the audit trail.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replay after the network left its parent would re-join it, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Signs group registrations and a membership acknowledgement as the network, sends them to the parent, and records the membership. Undone by vtn/memberships/leave.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: the parent's DID and URL, and role names. Out: the membership record — the parent's and this network's public identifiers, this network's group identifiers and the credentials and signed requests between them. No personal data.
retention:
  class: durable
  rationale: >-
    The membership record is the network's evidence of what it agreed to with its parent, kept while it is a member and afterwards; the change is in the audit trail.
errorCodes:
  - code: vtn/memberships/join:notListed
    meaning: "The parent does not list this network — no listing signed by the parent names it with `nodeType` `VTN` under the DTG `lists/1` predicate — so there is nothing to acknowledge."
    retryable: false
  - code: vtn/memberships/join:networkMismatch
    meaning: "The network at `url` answers as a DID other than `network`."
    retryable: false
  - code: vtn/memberships/join:roleUnassigned
    meaning: "A parent role — mapped in `roles`, or required by the parent and left unmapped — would be served by a role this network does not define. `details.role` names the parent role."
    retryable: false
    detailsSchema:
      type: object
      required: [role]
      properties:
        role:
          type: string
  - code: vtn/memberships/join:networkRefused
    meaning: "The parent could not be reached, refused a registration or the acknowledgement, or answered with an acceptance or grant that does not match what was sent. No membership is recorded. `details.code` and `details.message` carry the parent's error when it gave one."
    retryable: true
    detailsSchema:
      type: object
      properties:
        code:
          type: string
        message:
          type: string
related:
  - vtn/memberships/list
  - vtn/memberships/repudiate
  - vtn/memberships/leave
  - vtn/memberships/groups/add
  - vtn/membership/offer
  - vtn/membership/acknowledge
  - vtn/groups/register
  - vtc/networks/join
---

## Abstract

A Verifiable Trust Network may be listed by another network — its **parent** — as a community is ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-VTN-030). The listing is the parent's one-sided assertion (VTI-VTN-010); it becomes membership only when this network acknowledges it (VTI-VTN-013). **VTN Memberships — Join** is that acknowledgement, asked for by one of this network's administrators. It is the exact mirror of [`vtc/networks/join`](../../../../vtc/networks/join/0.1/spec.md): the administrator names the parent — by its URL and the DID they expect to find there — and which of this network's roles serves each parent role. The network then, signing as itself:

1. checks the parent at `url` is `network`, and that its listing names this network as a network;
2. registers, for each parent role, its own **network group** for the mapped local role (`<thisNetworkDid>#role-<localRole>`) through [`vtn/groups/register`](../../../groups/register/0.1/spec.md), and keeps the signed request and the parent's groupRegistered credential accepting it as the evidence;
3. acknowledges the parent's membership offer ([`vtn/membership/offer`](../../../membership/offer/0.1/spec.md), [`vtn/membership/acknowledge`](../../../membership/acknowledge/0.1/spec.md)) and verifies the grant the parent signs.

Broadcasts the parent sends to a registered role then arrive at this network, which passes them on to the groups its own members registered for the local role.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **administrator** (`issuer`):

1. **MUST** take `network` from a source other than the network at `url` — the parent's administrators, its published governance — so that the check in network item 2 means something.
2. **MUST NOT** treat this network as a member of the parent unless the response's `membership` is in state `acknowledged`.

A conforming **network** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`, and **MUST** refuse a `network` that is its own DID with `malformedRequest`: a network is not a member of itself.
2. **MUST** read the parent's profile at `url` (`/.well-known/vtn`) and refuse with `vtn/memberships/join:networkMismatch`, before signing anything, when it names a DID other than `network`. A URL is where a network is reached, not who it is.
3. **MUST** read its own entry at the parent ([`vtn/entries/show`](../../../entries/show/0.1/spec.md), signed as itself) and verify the listing: a credential signed by `network` for `assertionMethod`, whose subject is this network's DID, whose predicate is `https://registry.trustoverip.org/dtg/vsc/lists/1`, and whose `object.value.nodeType` is `VTN` (VTI-VTN-031). It **MUST** refuse with `vtn/memberships/join:notListed` when there is no such listing.
4. **MUST** resolve the role mappings before sending anything: each entry of `roles`, plus — for every role the parent's profile marks required and `roles` does not map — a mapping to this network's role of the same name. It **MUST** refuse with `vtn/memberships/join:roleUnassigned`, naming the parent role, when a mapping's `localRole` is not a role this network defines.
5. **MUST** register each mapping with the parent before acknowledging, by sending `vtn/groups/register` signed as this network, with `role` the parent role, `group` its network group `<thisNetworkDid>#role-<localRole>`, and `endpoint` the endpoint at which it accepts [`vtn/relay/deliver`](../../../relay/deliver/0.1/spec.md). It **MUST** verify the response's `group.acceptance` as a groupRegistered credential (`https://openvtc.org/vtn/vsc/groupRegistered/1`) issued by `network` and signed for `assertionMethod`, whose `credentialSubject.id` is the network group, whose `object.digestMultibase` equals the digest of the request exactly as sent (SHA-256 multihash over the JCS canonicalization of the signed request, `proof` included), and whose status on `network`'s revocation list is valid; and **MUST** keep the request and that credential unchanged as the group's `registration` and `acceptance`.
6. **MUST** fetch the parent's offer, check it is a membership credential issued by `network` naming this network, and acknowledge it with a membership credential this network issues, carrying the offer's `digestMultibase` and signed for `assertionMethod`.
7. **MUST** verify that the grant the parent returns is signed by `network` for `assertionMethod`, names this network, and is the credential the acknowledgement's digest binds. On this or any refusal by the parent in items 5–7 it **MUST** refuse with `vtn/memberships/join:networkRefused` and record no membership.
8. **MUST** sign every registration and the acknowledgement as this network — never as a group, nor as the administrator — and **MUST** record the membership in state `acknowledged`, with the listing, grant, acknowledgement and groups, in its audit trail naming the administrator, and answer with it.
9. Joining a parent it is already an acknowledged member of **MUST** fail with `alreadyExists`.
10. **MUST NOT** treat membership of the parent as extending to its own members: the parent does not thereby list, admit or grant anything to them, and they do not thereby become members of the parent (VTI-VTN-033). Joining gives the parent no authority over this network's members, trust contexts or keys (VTI-VTN-060 – VTI-VTN-063).

## Authorization

The authority this task presupposes is **administrator standing at this network**: the signer is one of the administrators in the network's configuration, read at execution time, or a key one of them delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md), in which case it is the standing of the administrator the key acts for. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A network **MAY** require a further, operation-bound step-up before acting, since joining commits it to a parent's governance framework and discloses its group identifiers to that parent; that is the network's policy, not this specification's.

The parent's authority over its own list is separate: it decides whom to list, and whether to accept the registrations and the acknowledgement.

## Definitions

- **`url`** — the parent network's base URL, where its profile and Trust Task endpoint are found.
- **`network`** — the DID the administrator expects the parent at `url` to be.
- **`roles`** — [`RoleMapping`](../../../_shared/0.1/membership.schema.json)s: which of this network's roles (`localRole`) serves each parent role (`parentRole`). May be empty, in which case each role the parent requires is served by the same-named local role.
- **Network group** — this network's group for one of its own roles, the DID URL `<thisNetworkDid>#role-<localRole>`. Deliveries to it reach the groups this network's members registered for that role.
- **`membership`** — the resulting record ([`UpstreamMembership`](../../../_shared/0.1/membership.schema.json)).

## Request

A network administrator (`issuer`) asks this network (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Join a parent, serving its admins with this network's admins

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000d21",
  "type": "https://trusttasks.org/spec/vtn/memberships/join/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:regional",
  "issuedAt": "2026-11-03T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000dff",
  "payload": {
    "url": "https://whn.example.org",
    "network": "did:example:whn",
    "roles": [
      { "parentRole": "admins", "localRole": "admins" }
    ]
  }
}
```

## Response

This network answers with the sub-schema reachable via `$anchor: "response"`: the membership as it now stands. A refusal is a `trust-task-error`.

### A member of the parent

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000d22",
  "type": "https://trusttasks.org/spec/vtn/memberships/join/0.1#response",
  "issuer": "did:example:regional",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-03T10:00:03Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000dff",
  "payload": {
    "membership": {
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
  }
}
```

The credentials, and the signed `vtn/groups/register` requests, are abbreviated to their `id`; the network returns them in full.

## Security & Privacy

### Data carried

The request carries the parent's URL and DID and role names. The response carries the membership record: the two networks' public identifiers, this network's network-group identifiers and relay endpoint, and the credentials and signed requests between the two networks (listing, grant, acknowledgement, group registrations and their acceptance credentials). No personal data is carried: a network group is named by a DID URL on this network's DID, never by who is in it.

### Correlation

Both networks are named by their public identifiers by design: the parent lists, grants to and publishes this network under one DID, and a verifier can only check a membership whose parties it can name the same way every time. This network therefore declares `identifierScope: public`. The administrator declares `identifierScope: any`: the task is between the administrator and their own network, and the administrator's identifier is never sent to the parent. Everything sent to the parent is signed by this network, so the parent learns which network joined, not which administrator asked.

### Retention

The network keeps the membership record, and the credentials and exchanges in it, for as long as it is a member and afterwards, as evidence of what it agreed to and when. The change is recorded in its audit trail against the administrator who made it.

### Consent/purpose

The acknowledgement is this network's consent to being published as the parent's member, and the registrations are its consent to receive the parent's broadcasts for the mapped roles. The record exists to evidence that membership and is not reused for any other purpose. This network withdraws its consent with [`vtn/memberships/leave`](../../leave/0.1/spec.md).
