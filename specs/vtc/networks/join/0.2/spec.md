---
slug: vtc/networks/join
version: "0.2"
title: "VTC Networks — Join"
summary: "An administrator accepts a Verifiable Trust Network's listing of the community: the community registers its groups for the network's required roles and signs the membership acknowledgement."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtc, networks, vtn, membership, acknowledge, curation]
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The task makes the community sign a statement to a third party — a network — on the authority of the administrator who asked, so the asker has to be attributable on every transport and afterwards in the audit trail.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replay would repeat a statement the community makes to a network, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Creates the community's membership record, signs registrations and the acknowledgement as the community and sends them to the network. Undone by vtc/networks/leave.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a network's DID and URL, and community group identifiers. Out: the membership record — the network's and the community's public identifiers, the community's group DIDs and the credentials between them. No personal data.
retention:
  class: durable
  rationale: >-
    The membership record is the community's evidence of what it agreed to with the network, kept while it is a member and afterwards; the change is in the audit trail.
errorCodes:
  - code: vtc/networks/join:notListed
    meaning: "The network does not list the community, so there is nothing to acknowledge."
    retryable: false
  - code: vtc/networks/join:networkMismatch
    meaning: "The network at `url` answers as a DID other than `network`."
    retryable: false
  - code: vtc/networks/join:roleUnassigned
    meaning: "The network requires a role the request assigns no group to. `details.role` names it."
    retryable: false
  - code: vtc/networks/join:unknownGroup
    meaning: "A `groupId` names no group of the community's that has a DID of its own."
    retryable: false
  - code: vtc/networks/join:networkRefused
    meaning: "The network refused a registration or the acknowledgement, or answered with a grant that does not match it. `details.message` carries the network's reason when it gave one."
    retryable: true
related:
  - vtc/networks/leave
  - vtc/networks/repudiate
  - vtc/networks/groups/register
  - vtc/networks/show
  - vtc/networks/deliver
  - vtn/groups/register
  - vtn/membership/acknowledge
---

## Abstract

The **VTC Networks — Join** Trust Task is how a community accepts a Verifiable Trust Network's listing of it. A network may list a community without asking ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/) (VTI-VTN-001 – VTI-VTN-063), VTI-VTN-010); the listing only becomes membership when the community acknowledges it (VTI-VTN-013). An administrator names the network — by its URL and the DID they expect to find there — and the community's groups that will serve the network's roles. The community then talks to the network with the network's own Trust Tasks, each signed as the community and addressed to that DID:

1. reads its entry at the network ([`vtn/entries/show`](../../../../vtn/entries/show/0.1/spec.md)) and verifies the listing is the network's;
2. registers each named group for its role with [`vtn/groups/register`](../../../../vtn/groups/register/0.1/spec.md) — the signed request names the role, the group's DID, and the community's Trust Task endpoint, where the network delivers the role's broadcasts with [`vtc/networks/deliver`](../../deliver/0.1/spec.md) — and verifies the network's `groupRegistered` acceptance of it;
3. reads the network's membership offer ([`vtn/membership/offer`](../../../../vtn/membership/offer/0.1/spec.md)) and acknowledges it ([`vtn/membership/acknowledge`](../../../../vtn/membership/acknowledge/0.1/spec.md)) with a membership credential (VMC) carrying the digest of the offered grant, and verifies the grant the network then signs.

The network may require groups for some roles before it accepts an acknowledgement, so that it can reach, say, every member's administrators from the moment the member joins. The task assigns those roles in the same request. A network joins a parent network the same way with [`vtn/memberships/join`](../../../../vtn/memberships/join/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.1

- **The community reaches the network only through `vtn/*` Trust Tasks** — `vtn/entries/show`, `vtn/roles/list`, `vtn/groups/register`, `vtn/membership/offer` and `vtn/membership/acknowledge` — each signed as the community for `authentication` and addressed to `network`, and it verifies the network's proof on every answer (items 3 to 8).
- **A group's registration is the community's signed `vtn/groups/register` request**, kept exactly as sent. The `servesRole` statement of `0.1` is retired; the network's `groupRegistered/1` acceptance binds the request by its digest. `membership` follows [`_shared/0.7`](../../../_shared/0.7/network-membership.schema.json).
- **A registered group's `endpoint` is the community's Trust Task endpoint**, at which the network delivers broadcasts for the group with `vtc/networks/deliver`, rather than a relay path per group.

The request, the response's shape and the error codes are unchanged. Because `membership.groups[].registration` now carries a different document, `0.2` does not declare `extends: "0.1"`. Released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`.
2. **MUST** read the network's profile at `url` (`/.well-known/vtn`) and refuse with `vtc/networks/join:networkMismatch`, before signing anything, when it names a DID other than `network`. A URL is where a network is reached, not who it is.
3. **MUST** send each request below to the network as a Trust Task signed as the community for `authentication`, with `recipient` `network` — never as a group, nor as the administrator — and **MUST** verify that each answer carries a proof made for `authentication` by a key of `network`. An answer without such a proof is treated as the network's refusal.
4. **MUST** read its own entry ([`vtn/entries/show`](../../../../vtn/entries/show/0.1/spec.md)) and verify the listing: a credential signed by `network` for `assertionMethod`, whose subject is the community's DID, whose predicate is `https://registry.trustoverip.org/dtg/vsc/lists/1`, and whose `object.value.nodeType` is `VTC`. It **MUST** refuse with `vtc/networks/join:notListed` when there is no such listing.
5. **MUST** read the network's roles ([`vtn/roles/list`](../../../../vtn/roles/list/0.1/spec.md)) and refuse with `vtc/networks/join:roleUnassigned` when a role is `required` and `groups` assigns none, and with `vtc/networks/join:unknownGroup` when a `groupId` is not one of its groups with a DID of its own. Both checks run before any registration is sent.
6. **MUST** register each assigned group before acknowledging, by sending [`vtn/groups/register`](../../../../vtn/groups/register/0.1/spec.md) with `role` the network's role, `group` the group's DID, and `endpoint` the community's Trust Task endpoint, at which it accepts [`vtc/networks/deliver`](../../deliver/0.1/spec.md). It **MUST** verify the answer's `group.acceptance` as a groupRegistered credential (`https://openvtc.org/vtn/vsc/groupRegistered/1`) issued by `network` and signed for `assertionMethod`, whose `credentialSubject.id` is the group's DID, whose `object.digestMultibase` equals the registration digest of the request exactly as sent (a SHA-256 multihash over the RFC 8785 (JCS) canonicalization of the signed request, `proof` included), and whose status on `network`'s revocation list is valid; and **MUST** keep the request and that credential unchanged as the group's `registration` and `acceptance`.
7. **MUST** fetch the network's offer ([`vtn/membership/offer`](../../../../vtn/membership/offer/0.1/spec.md)), check that it is a membership credential issued by `network` naming the community, and acknowledge it ([`vtn/membership/acknowledge`](../../../../vtn/membership/acknowledge/0.1/spec.md)) with a membership credential the community issues, carrying the offer's `digestMultibase` and signed for `assertionMethod`.
8. **MUST** verify that the grant the network returns is signed by `network` for `assertionMethod`, names the community, and is the credential the acknowledgement's digest binds. On this, or on any refusal or unverifiable answer from the network in items 4 to 8, it **MUST** refuse with `vtc/networks/join:networkRefused` and record no membership.
9. **MUST** record the membership in state `acknowledged`, with the listing, grant, acknowledgement and groups, in its audit trail naming the administrator, and answer with it.
10. Joining a network it is already an acknowledged member of **MUST** fail with `alreadyExists`.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry, read at execution time. For a signer that is a delegated signing key ([`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md)), it is the standing of the identity the key acts for. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's, and so is any further approval it requires before acting: a network membership commits the community to a curator's governance framework and discloses its groups' identifiers to that curator.

The network's authority over its own list is separate: it decides whom to list and whether to accept the acknowledgement. Joining grants the network nothing over the community's members (VTI-VTN-062).

## Definitions

- **`url`** — the network's API base, where its profile and Trust Task endpoint are found.
- **`network`** — the DID the administrator expects the network at `url` to be.
- **`groups`** — [`RoleAssignment`](../../../_shared/0.7/network-membership.schema.json)s: the community group to serve each role.
- **Registration** — the community's signed `vtn/groups/register` request for a group, exactly as sent; its **registration digest** is the one `vtn/groups/register` defines.
- **Trust Task endpoint** — where the community accepts Trust Tasks. A network delivers broadcasts for every group the community registered with it there, with `vtc/networks/deliver`.
- **`membership`** — the resulting record ([`NetworkMembership`](../../../_shared/0.7/network-membership.schema.json)).

## Request

A community administrator (`issuer`) asks the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Join, serving the network's admins with the community's admins group

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000321",
  "type": "https://trusttasks.org/spec/vtc/networks/join/0.2#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-03T10:00:21Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "url": "https://whn.example.org",
    "network": "did:example:whn",
    "groups": [
      {
        "role": "admins",
        "groupId": "admins"
      }
    ]
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### The membership

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000322",
  "type": "https://trusttasks.org/spec/vtc/networks/join/0.2#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-03T10:00:22Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "membership": {
      "network": "did:example:whn",
      "url": "https://whn.example.org",
      "name": "World Health Network",
      "state": "acknowledged",
      "listing": {
        "@context": [
          "https://www.w3.org/ns/credentials/v2",
          "https://registry.trustoverip.org/dtg/context/v1"
        ],
        "id": "urn:uuid:00000000-0000-4000-8000-0000000c0011",
        "type": [
          "VerifiableCredential",
          "DTGCredential",
          "StatementCredential"
        ],
        "issuer": "did:example:whn",
        "issuerScope": "public",
        "validFrom": "2026-11-01T09:00:00Z",
        "credentialSubject": {
          "id": "did:example:community",
          "predicate": "https://registry.trustoverip.org/dtg/vsc/lists/1",
          "object": {
            "value": {
              "nodeType": "VTC",
              "framework": {
                "iri": "https://whn.example.org/governance",
                "version": "1.0"
              }
            }
          }
        },
        "proof": {
          "type": "DataIntegrityProof",
          "cryptosuite": "eddsa-jcs-2022",
          "proofPurpose": "assertionMethod",
          "verificationMethod": "did:example:whn#key-0",
          "proofValue": "z3abc"
        }
      },
      "groups": [
        {
          "role": "admins",
          "groupId": "admins",
          "groupDid": "did:example:community-admins-group",
          "endpoint": "https://community.example.org/v1/trust-tasks",
          "registration": {
            "id": "urn:uuid:00000000-0000-4000-8000-0000000c0014",
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
            "id": "urn:uuid:00000000-0000-4000-8000-0000000c0015",
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
          "registeredAt": "2026-11-03T10:00:01Z"
        }
      ],
      "updatedAt": "2026-11-03T10:00:01Z",
      "grant": {
        "@context": [
          "https://www.w3.org/ns/credentials/v2",
          "https://registry.trustoverip.org/dtg/context/v1"
        ],
        "id": "urn:uuid:00000000-0000-4000-8000-0000000c0012",
        "type": [
          "VerifiableCredential",
          "DTGCredential",
          "MembershipCredential"
        ],
        "issuer": "did:example:whn",
        "issuerScope": "public",
        "validFrom": "2026-11-01T09:00:00Z",
        "credentialSubject": {
          "id": "did:example:community"
        },
        "proof": {
          "type": "DataIntegrityProof",
          "cryptosuite": "eddsa-jcs-2022",
          "proofPurpose": "assertionMethod",
          "verificationMethod": "did:example:whn#key-0",
          "proofValue": "z3abc"
        }
      },
      "acknowledgement": {
        "@context": [
          "https://www.w3.org/ns/credentials/v2",
          "https://registry.trustoverip.org/dtg/context/v1"
        ],
        "id": "urn:uuid:00000000-0000-4000-8000-0000000c0013",
        "type": [
          "VerifiableCredential",
          "DTGCredential",
          "MembershipCredential"
        ],
        "issuer": "did:example:community",
        "issuerScope": "public",
        "validFrom": "2026-11-01T09:00:00Z",
        "credentialSubject": {
          "id": "did:example:whn",
          "digestMultibase": "zQmGrantDigest"
        },
        "proof": {
          "type": "DataIntegrityProof",
          "cryptosuite": "eddsa-jcs-2022",
          "proofPurpose": "assertionMethod",
          "verificationMethod": "did:example:community#key-0",
          "proofValue": "z3abc"
        }
      },
      "joinedAt": "2026-11-03T10:00:01Z"
    }
  }
}
```

## Security & Privacy

### Data carried

Network DIDs and URLs, the community's group identifiers and group DIDs, and the DTG credentials that evidence the membership (listing, grant, acknowledgement, repudiation, group registrations and acceptances). These are the community's and the network's public identifiers and the statements they made about each other. No member's personal data is carried: a group is named by its identifier and DID, never by who is in it.

### Correlation

The credentials name the community's and the network's public identifiers by design (VTI-VTN-011): a membership the network publishes is meant to be correlatable. The community is addressed by its public identifier (`identifierScope: public`) because that is the identifier the network lists and the one its credentials name: a membership is a public relationship between two public parties, and a pairwise identifier would be one the network's listing, grant and published index could not refer to. A group's DID is disclosed to the network when it is registered; a community that does not want a group linkable across networks registers a different group with each.

### Retention

The community keeps the membership record, and the credentials in it, for as long as it is a member and afterwards as evidence of what it agreed to and when. Each change is recorded in the community's audit trail against the administrator who made it.

### Consent/purpose

The records exist to evidence the community's membership of networks and the groups it nominated to serve them; they are not reused for any other purpose.
