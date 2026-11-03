---
slug: vtn/groups/register
version: "0.1"
title: "VTN Groups — Register"
summary: "A listed party or member registers one of its groups to serve a network role; the network accepts it with a revocable groupRegistered credential bound to the signed request."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, roles, groups, broadcast, nested-membership]
parties:
  - role: listed party
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: network
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  request: REQUIRED
  response: RECOMMENDED
  rationale: >-
    The signed request is the member's registration, which a room-pass chain or a relaying network relies on long after the exchange, so its proof is required. The acceptance is a credential carrying its own proof and status, so the response document's proof is not the evidence and is only RECOMMENDED.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed registration would restore a group the member has since replaced, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Records the group as serving the role for the member and issues a groupRegistered credential accepting it, revoking the acceptance of any group the member registered for that role before. Undone by registering another group, or by the party repudiating, departing or being removed, which drops its groups and revokes their acceptances.
consequences:
  - "Broadcasts this network sends or relays to the role are delivered to the registered endpoint."
  - "A group registered earlier by the same member for the same role is replaced, and its groupRegistered credential is revoked."
  - "Consumes one slot on the network's revocation status list."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a role name, a group identifier and a delivery endpoint, all of an organisation. Out: the registration as recorded, with the network's acceptance credential. No personal data.
retention:
  class: durable
  rationale: >-
    The network keeps the registration and its acceptance for as long as the group serves the role: relays are routed by it and room passes are issued on it. Afterwards the audit trail records it, and the acceptance's status-list entry stays revoked.
errorCodes:
  - code: vtn/groups/register:notListed
    meaning: "The network has no entry for the issuer in state `listed` or `acknowledged`."
    retryable: false
  - code: vtn/groups/register:unknownRole
    meaning: "The network defines no role named `role`."
    retryable: false
  - code: vtn/groups/register:groupTaken
    meaning: "Another member has already registered `group`."
    retryable: false
related:
  - vtn/roles/list
  - vtn/groups/list
  - vtn/membership/acknowledge
  - vtn/relay/deliver
  - vtn/rooms/pass
  - vtn/memberships/groups/add
---

## Abstract

A Verifiable Trust Network defines **roles** — `admins`, `notifications`, `security-contacts` — and reaches each member through a **group** the member registers to serve the role. A community registers one of its own groups; a network registers its **network group** for the role, `<networkDid>#role-<localRole>`, the set of groups its own members registered, so that deliveries cascade one network at a time and no network ever holds another's roster ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-VTN-032, VTI-VTN-033).

**VTN Groups — Register** is that registration. The member's signed request *is* the registration. The network accepts it by issuing a **groupRegistered credential** — a statement whose subject is the group and whose object is the digest of the signed request — with a status-list entry the network revokes when the registration ends. The pair is self-describing evidence that any third party can check, and that stops checking out once the network withdraws it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **listed party** (`issuer`):

1. **MUST** sign the request with a key of its own DID, so that the request stands as its registration without reference to any delegation.
2. **MUST** register as `group` either one of its own groups (a DID it controls) or, for a network, its own network group `<issuer>#role-<localRole>`; it **MUST NOT** register another network's network group.
3. **MUST** register a group for each role the network marks required ([`vtn/roles/list`](../../../roles/list/0.1/spec.md)) before acknowledging its membership ([`vtn/membership/acknowledge`](../../../membership/acknowledge/0.1/spec.md)).
4. **MUST** verify the response's `group` as a verifier does (below), with `registration` byte-for-byte the request it sent, and **MUST** keep the registration and acceptance together as the registration's evidence — a community hands the pair to its group's members for [`vtn/rooms/pass`](../../../rooms/pass/0.1/spec.md).

A conforming **network** (`recipient`):

1. **MUST** refuse with `vtn/groups/register:notListed` unless it has an entry for `issuer` in state `listed` or `acknowledged`.
2. **MUST** refuse with `vtn/groups/register:unknownRole` when it defines no role named `role`.
3. **MUST** refuse with `permissionDenied` a `group` of the network-group form (`<did>#role-<name>`) whose DID is not `issuer`: a network group can be registered only by its own network.
4. **MUST** refuse with `vtn/groups/register:groupTaken` a `group` another member has registered.
5. **MUST** issue and sign for `assertionMethod` an acceptance: a DTG Verifiable Statement Credential (`StatementCredential`) with `issuer` the network, `credentialSubject.id` the group, `credentialSubject.predicate` `https://openvtc.org/vtn/vsc/groupRegistered/1`, `credentialSubject.object.digestMultibase` the registration digest (below), and a `credentialStatus` entry on its revocation status list.
6. **MUST** record the group as serving `role` for `issuer` with the request, exactly as received, as its `registration`, replacing any group `issuer` registered for `role` before and revoking that group's acceptance; and **MUST** record the change in its audit trail with `issuer` as the actor.
7. **MUST** revoke the acceptance of every group a party registered when it repudiates, departs or is removed, and drop those groups.
8. **MUST** deliver broadcasts for `role` only to the endpoint of a group registered by a member whose entry is `acknowledged`.
9. **MUST NOT** treat the registration as giving it any say over the group's membership, and **MUST NOT** derive keys or create trust contexts for it (VTI-VTN-060 – VTI-VTN-063): the network delivers to the group; the group's owner decides who is in it.

A **verifier** of a registration and its acceptance (a network issuing a room pass, a member keeping its evidence, or any third party) **MUST** check:

1. the registration's proof, and that it is a `vtn/groups/register` request whose `issuer` is the member and whose `recipient` is the accepting network;
2. the acceptance's proof, for `assertionMethod` by a key of its `issuer`, which **MUST** be the registration's `recipient`; its predicate; its validity period; and that its `credentialSubject.id` equals the registration's `group`;
3. that its `digestMultibase` equals the registration digest of the registration it holds, compared as decoded multihash bytes; and
4. its status against its issuer's revocation status list: revoked or suspended means the registration has ended, and a status that cannot be determined **MUST NOT** be treated as valid (VTI-REG-012).

## Authorization

The authority this task presupposes is **being a listed party or member** of the network: the network accepts a registration only from the DID one of its entries names, for that entry, and only for a group that party may speak for (network items 1 and 3). The request's proof establishes who registers. The network's administrators decided earlier, by listing the party, that it may take part, and define the roles in the network's configuration; neither decision is revisited here. A consumer **MAY** require an operation-bound step-up of the party's operator before accepting a registration, as policy of its own.

## Definitions

- **Role** — a name the network defines ([`RoleName`](../../../_shared/0.1/groups.schema.json)); a role may be *required*, meaning a party must register a group for it before its acknowledgement is accepted.
- **`group`** (request) — the group that serves the role: a DID of the member's, or for a network its network group `<networkDid>#role-<localRole>`, where `localRole` is the member network's own role whose groups the deliveries fan out to.
- **`endpoint`** — the HTTPS URL at which the network delivers broadcasts for the group. A member network receives them through [`vtn/relay/deliver`](../../../relay/deliver/0.1/spec.md) at its Trust Task endpoint.
- **Registration** — this request, signed by the member, exactly as received.
- **Registration digest** — a SHA-256 multihash, multibase-encoded, over the RFC 8785 (JCS) canonicalization of the registration **including** its top-level `proof`. This is deliberately the bytes-as-received sense ([SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names), *stepDigest*), not the task digest: the acceptance binds the signed registration, so a re-signed or unsigned copy is a different registration.
- **Acceptance** — the network's groupRegistered credential. `https://openvtc.org/vtn/vsc/groupRegistered/1` is an interim predicate in the OpenVTC namespace, pending its registration in the DTG VSC registry.
- **`group`** (response) — the registration as recorded ([`RegisteredGroup`](../../../_shared/0.1/groups.schema.json)): `member` (the request's `issuer`), `role`, `group`, `endpoint`, the `registration`, the `acceptance`, and `registeredAt`.

## Request

The listed party (`issuer`) registers a group with the network (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member network registers its network group for `admins`

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000801",
  "type": "https://trusttasks.org/spec/vtn/groups/register/0.1#request",
  "issuer": "did:example:health-au",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T09:00:00Z",
  "payload": {
    "role": "admins",
    "group": "did:example:health-au#role-admins",
    "endpoint": "https://health-au.example/v1/trust-tasks"
  }
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`: the registration as recorded, with its acceptance. A refusal is a `trust-task-error`, and accepts nothing.

### The network accepts the registration

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000802",
  "type": "https://trusttasks.org/spec/vtn/groups/register/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:health-au",
  "issuedAt": "2026-11-03T09:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000801",
  "payload": {
    "group": {
      "member": "did:example:health-au",
      "role": "admins",
      "group": "did:example:health-au#role-admins",
      "endpoint": "https://health-au.example/v1/trust-tasks",
      "registration": { "id": "urn:uuid:00000000-0000-4000-8000-000000000801" },
      "acceptance": {
        "@context": [
          "https://www.w3.org/ns/credentials/v2",
          "https://registry.trustoverip.org/dtg/context/v1"
        ],
        "id": "urn:uuid:00000000-0000-4000-8000-0000000c0081",
        "type": ["VerifiableCredential", "DTGCredential", "StatementCredential"],
        "issuer": "did:example:whn",
        "issuerScope": "public",
        "validFrom": "2026-11-03T09:00:01Z",
        "credentialSubject": {
          "id": "did:example:health-au#role-admins",
          "predicate": "https://openvtc.org/vtn/vsc/groupRegistered/1",
          "object": {
            "digestMultibase": "zQmUXBBKmqKHhXshFGeMh43p8HWPbmmPLSgvSAQkkRqLsHM"
          }
        },
        "credentialStatus": {
          "id": "https://whn.example/v1/status/revocation#31",
          "type": "BitstringStatusListEntry",
          "statusPurpose": "revocation",
          "statusListIndex": "31",
          "statusListCredential": "https://whn.example/v1/status/revocation"
        },
        "proof": {
          "type": "DataIntegrityProof",
          "cryptosuite": "eddsa-jcs-2022",
          "proofPurpose": "assertionMethod",
          "verificationMethod": "did:example:whn#key-0",
          "proofValue": "z3ghi"
        }
      },
      "registeredAt": "2026-11-03T09:00:01Z"
    }
  }
}
```

The registration is abbreviated to its `id`; the network returns the signed request in full.

## Security & Privacy

### Data carried

The request carries a role name, a group identifier and an endpoint; the response carries the registration as recorded and the acceptance credential. All belong to organisations; neither carries personal data. A group is identified, never enumerated: the network learns who serves a role, not who is in the group.

### Correlation

A registration ties the member's DID to a group and an endpoint, and the pair is handed on — to the group's members, and through them to any network a person asks for a room pass. Both parties declare `identifierScope: public` because the pair is evidence for third parties: a verifier can only check it against a member and a network it can name the same way every time, the acceptance's status is published on the network's status list, and the group identifier of a network group is built on the network's public DID. A community that wants its groups less linkable can register a group DID used for nothing else. Checking an acceptance's status fetches the whole status list, so the network does not learn which registration was checked.

### Retention

The network keeps the current registration and acceptance for each member and role while the member takes part, because relays are routed and room passes are issued on them; a replaced or dropped registration is kept only in the audit trail, and its acceptance stays revoked. A member keeps its pairs for as long as its groups' members may need them.

### Consent/purpose

Registering a group is the member's consent to receive the network's broadcasts for that role at that endpoint, and to the pair being used as evidence of it. The network uses the registration for routing and for room-pass eligibility, and for nothing else; the member withdraws it by registering another group, or by repudiating or departing, and the network then revokes the acceptance.
