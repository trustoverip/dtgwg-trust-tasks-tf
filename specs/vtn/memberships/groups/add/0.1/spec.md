---
slug: vtn/memberships/groups/add
version: "0.1"
title: "VTN Memberships — Groups — Add"
summary: "An administrator registers this network's network group for one of its roles to serve a role at a parent network it is a member of, replacing any group registered for that parent role."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, nested-membership, groups, roles, broadcast]
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
    The task makes the network sign a registration to a third party — the parent — on the authority of the administrator who asked, so the asker has to be attributable on every transport and afterwards in the audit trail.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replay could restore a registration the network has since replaced, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Signs a vtn/groups/register request as the network and sends it to the parent, replacing the group registered for the parent role if there is one, and records the exchange in the membership. Undone by registering another group for the role, or by leaving.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: the parent's DID and two role names. Out: the membership record — the parent's and this network's public identifiers, this network's group identifiers and the credentials and signed requests between them. No personal data.
retention:
  class: durable
  rationale: >-
    The registration and the parent's acceptance are the evidence that the parent's deliveries for the role are meant for this network; they are kept in the membership record and the change is in the audit trail.
errorCodes:
  - code: vtn/memberships/groups/add:notMember
    meaning: "This network is not an acknowledged member of `network`."
    retryable: false
  - code: vtn/memberships/groups/add:unknownRole
    meaning: "`localRole` is not a role this network defines, so it has no network group to register."
    retryable: false
  - code: vtn/memberships/groups/add:networkRefused
    meaning: "The parent could not be reached, refused the registration — for example, it defines no role `parentRole` — or answered with an acceptance that does not match it; the membership is unchanged. `details.code` and `details.message` carry the parent's error when it gave one."
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
  - vtn/memberships/list
  - vtn/groups/register
  - vtn/relay/deliver
  - vtc/networks/groups/register
---

## Abstract

A parent network defines roles — `admins`, `notifications` — and reaches the parties serving them with broadcasts that cascade through its member networks ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-VTN-030). **VTN Memberships — Groups — Add** nominates this network to serve one of those roles after it has joined: it registers its own **network group** for one of its roles (`<thisNetworkDid>#role-<localRole>`) with the parent through [`vtn/groups/register`](../../../../groups/register/0.1/spec.md), signed as itself, and keeps the signed request and the parent's groupRegistered credential accepting it as the evidence. The parent's deliveries for the role then arrive at this network, which passes them on to the groups its own members registered for the local role. It is the mirror of [`vtc/networks/groups/register`](../../../../../vtc/networks/groups/register/0.1/spec.md); [`vtn/memberships/join`](../../../join/0.1/spec.md) does the same for the roles named when joining.

This network, not any of its members, is the registrant: it is the parent's member, and the group is its own. The parent learns the group's identifier, never who is in it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **network** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`, with `vtn/memberships/groups/add:notMember` when it holds no membership of `network` in state `acknowledged`, and with `vtn/memberships/groups/add:unknownRole` when `localRole` is not one of its roles. These checks run before anything is sent.
2. **MUST** send `vtn/groups/register` to the parent at the `url` recorded for the membership, signed as this network — never as the administrator — with `role` set to `parentRole`, `group` its network group `<thisNetworkDid>#role-<localRole>`, and `endpoint` the endpoint at which it accepts [`vtn/relay/deliver`](../../../../relay/deliver/0.1/spec.md).
3. **MUST** verify the response's `group.acceptance` as a groupRegistered credential (`https://openvtc.org/vtn/vsc/groupRegistered/1`) issued by `network` and signed for `assertionMethod`, whose `credentialSubject.id` is the network group, whose `object.digestMultibase` equals the digest of the request exactly as sent (SHA-256 multihash over the JCS canonicalization of the signed request, `proof` included), and whose status on `network`'s revocation list is valid, and **MUST** otherwise refuse with `vtn/memberships/groups/add:networkRefused` and leave the membership unchanged. A refusal by the parent is reported the same way.
4. **MUST** keep the request and the groupRegistered credential unchanged as the group's `registration` and `acceptance`, replacing any group recorded for the same `parentRole`, and record the change in its audit trail naming the administrator.
5. **MUST** accept `vtn/relay/deliver` from `network` for a group only while it is an acknowledged member and that group's registration is the one recorded, and **MUST** pass a delivery on only to the groups its own members registered for `localRole`. Serving a parent's role grants the parent nothing over those members (VTI-VTN-060 – VTI-VTN-063).

## Authorization

The authority this task presupposes is **administrator standing at this network**: the signer is one of the administrators in the network's configuration, read at execution time, or a key one of them delegated through [`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.2/spec.md), in which case it is the standing of the administrator the key acts for. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). A network **MAY** require a further, operation-bound step-up before registering a group with a parent; that is its policy, not this specification's.

The parent decides whether to accept the registration; it may refuse a role it does not define.

## Definitions

- **`network`** — the parent network's DID.
- **`parentRole`** — the parent's role to serve.
- **`localRole`** — this network's role whose network group serves it.
- **Network group** — the DID URL `<thisNetworkDid>#role-<localRole>`.
- **`membership`** — the record ([`UpstreamMembership`](../../../../_shared/0.1/membership.schema.json)), with the new [`UpstreamGroup`](../../../../_shared/0.1/membership.schema.json).

## Request

A network administrator (`issuer`) asks this network (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Serve the parent's notifications role with this network's announcements role

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000d51",
  "type": "https://trusttasks.org/spec/vtn/memberships/groups/add/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:regional",
  "issuedAt": "2026-11-03T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000dff",
  "payload": {
    "network": "did:example:whn",
    "parentRole": "notifications",
    "localRole": "announcements"
  }
}
```

## Response

This network answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### Registered

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000d52",
  "type": "https://trusttasks.org/spec/vtn/memberships/groups/add/0.1#response",
  "issuer": "did:example:regional",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-03T10:00:02Z",
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
        },
        {
          "parentRole": "notifications",
          "localRole": "announcements",
          "group": "did:example:regional#role-announcements",
          "registration": { "id": "urn:uuid:00000000-0000-4000-8000-0000000d0051" },
          "acceptance": { "id": "urn:uuid:00000000-0000-4000-8000-0000000d0052" }
        }
      ],
      "updatedAt": "2026-11-03T10:00:01Z"
    }
  }
}
```

The credentials, and the signed `vtn/groups/register` requests and responses, are abbreviated to their `id`; the network returns them in full.

## Security & Privacy

### Data carried

The request carries the parent's DID and two role names. The response carries the membership record: the two networks' public identifiers, this network's network-group identifiers and relay endpoint, and the credentials and signed requests between them. No personal data is carried: a network group is a DID URL on this network's DID, never a list of who is in it.

### Correlation

The registration names this network's public DID, because the parent can only accept a registration from the member it lists; this network therefore declares `identifierScope: public`. A network group is derived from that DID and the local role, so it adds no new correlation handle beyond the role name. The administrator declares `identifierScope: any`: the parent sees only this network's signature.

### Retention

The network keeps the registration and acceptance in the membership record for as long as it is a member and afterwards, as the evidence that the parent's deliveries for the role are meant for it. The change is recorded in its audit trail against the administrator who made it.

### Consent/purpose

The registration is this network's consent to receive the parent's broadcasts for `parentRole` and pass them on to its own `localRole` groups; it is used for nothing else. It is withdrawn by registering another group for the role or by leaving the parent.
