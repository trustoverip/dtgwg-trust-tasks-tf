---
slug: vtc/networks/evidence
version: "0.2"
title: "VTC Networks — Evidence"
summary: "A member asks the community for the credentials that link a group they belong to up to a Verifiable Trust Network, which a network asks for before it admits them to a room."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtc, networks, vtn, groups, rooms, evidence]
parties:
  - role: community member
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The answer is released only to members of the groups it concerns, so the asker has to be identified by more than the transport.
sideEffects:
  level: none
  rationale: "Reads the community's group registrations; changes nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
  rationale: >-
    Out: for each group the caller is in, the community's registration of it with a network and that network's acceptance — public identifiers and statements, no personal data.
errorCodes: []
related:
  - vtc/networks/groups/register
  - vtc/networks/notice
  - vtn/rooms/pass
---

## Abstract

The **VTC Networks — Evidence** Trust Task gives a member the credentials that connect a group they are in to a Verifiable Trust Network. A network that opens a room for one of its roles admits a person on evidence they carry, checked when they ask ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/) (VTI-VTN-001 – VTI-VTN-063)): their membership credential from their group, and for each network between that group and the room's network, the group's registration — the registering member's signed [`vtn/groups/register`](../../../../vtn/groups/register/0.1/spec.md) request — and that network's `groupRegistered` acceptance of it. A community holds the first link — its signed registration of the group with a network it belongs to, and the network's acceptance — and hands it to the group's members with this task, for them to present with [`vtn/rooms/pass`](../../../../vtn/rooms/pass/0.1/spec.md). A link beyond that, from a member network to its own parent, is that network's to give.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.1

- **`links[].registration` is the community's signed `vtn/groups/register` request**, exactly as the network accepted it, rather than a `servesRole` statement; the network's `groupRegistered/1` acceptance binds it by its digest, so the link is the `{ registration, acceptance }` pair a network checks for `vtn/rooms/pass`.

The request and the response's shape are unchanged. Released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** answer only with links for groups the caller is an active member of — never another group's, whatever `groupId` names — and only for networks it is an acknowledged member of.
2. **MUST** answer each link as it holds it: its own signed registration request and the network's acceptance, byte-for-byte unchanged, so that a network can verify both and recompute the registration's digest.
3. **MUST** answer an empty `links` rather than an error when the caller is in no registered group: whether a group is registered is not disclosed to non-members of it.

## Authorization

The authority is **active membership of the group** each link concerns, read at execution time from the community's group records. The proof establishes who asked ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10); the community decides which links that person may have, link by link.

## Definitions

- **`groupId`**, **`network`** — optional filters.
- **`links`** — for each group the caller is in and each network it is registered with: **`groupId`**, **`groupDid`**, **`network`**, **`role`**, the community's **`registration`** (its signed `vtn/groups/register` request, with its proof) and the network's **`acceptance`** (its `groupRegistered` credential, bound to the registration's digest).

## Request

A community member (`issuer`) asks the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Evidence for one network

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000371",
  "type": "https://trusttasks.org/spec/vtc/networks/evidence/0.2#request",
  "issuer": "did:example:member",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-03T10:00:11Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "network": "did:example:whn"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### The admins group's link

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000372",
  "type": "https://trusttasks.org/spec/vtc/networks/evidence/0.2#response",
  "issuer": "did:example:community",
  "recipient": "did:example:member",
  "issuedAt": "2026-11-03T10:00:12Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "links": [
      {
        "groupId": "admins",
        "groupDid": "did:example:community-admins-group",
        "role": "admins",
        "registration": {
          "id": "urn:uuid:00000000-0000-4000-8000-0000000c0033",
          "type": "https://trusttasks.org/spec/vtn/groups/register/0.1#request",
          "issuer": "did:example:community",
          "recipient": "did:example:whn",
          "issuedAt": "2026-11-03T10:00:01Z",
          "payload": {
            "role": "admins",
            "group": "did:example:community-admins-group",
            "endpoint": "https://community.example.org/v1/trust-tasks"
          },
          "proof": {
            "type": "DataIntegrityProof",
            "cryptosuite": "eddsa-jcs-2022",
            "verificationMethod": "did:example:community#key-0",
            "created": "2026-11-03T10:00:01Z",
            "proofPurpose": "authentication",
            "proofValue": "z3abc"
          }
        },
        "acceptance": {
          "@context": [
            "https://www.w3.org/ns/credentials/v2",
            "https://registry.trustoverip.org/dtg/context/v1"
          ],
          "id": "urn:uuid:00000000-0000-4000-8000-0000000c0036",
          "type": [
            "VerifiableCredential",
            "DTGCredential",
            "StatementCredential"
          ],
          "issuer": "did:example:whn",
          "issuerScope": "public",
          "validFrom": "2026-11-01T09:00:00Z",
          "credentialSubject": {
            "id": "did:example:community-admins-group",
            "predicate": "https://openvtc.org/vtn/vsc/groupRegistered/1",
            "object": {
              "digestMultibase": "zQmRegistrationDigest"
            }
          },
          "credentialStatus": {
            "id": "https://whn.example.org/v1/status/revocation#31",
            "type": "BitstringStatusListEntry",
            "statusPurpose": "revocation",
            "statusListIndex": "31",
            "statusListCredential": "https://whn.example.org/v1/status/revocation"
          },
          "proof": {
            "type": "DataIntegrityProof",
            "cryptosuite": "eddsa-jcs-2022",
            "proofPurpose": "assertionMethod",
            "verificationMethod": "did:example:whn#key-0",
            "proofValue": "z3abc"
          }
        },
        "network": "did:example:whn"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The community's signed registration of the caller's group with a network and the network's acceptance: public identifiers (the network's, the community's, the group's) and the statements between them. Nothing about other members.

### Correlation

Presenting a link to a network discloses to it that the presenter is in the group — which is the point of presenting it. The community is addressed by its public identifier (`identifierScope: public`) because that is the identifier its registrations are signed under and the one a network checks them against; a pairwise identifier would not verify. A member who does not want to be linkable to their group does not ask for a room pass.

### Retention

The member's agent keeps the links for as long as it wants to present them; the network's acceptance carries its own status, so a stale link fails when checked.

### Consent/purpose

The links exist to let a group's members show a network that they serve its role; the community releases them only to those members.
