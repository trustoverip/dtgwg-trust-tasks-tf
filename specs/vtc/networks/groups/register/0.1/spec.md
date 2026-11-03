---
slug: vtc/networks/groups/register
version: "0.1"
title: "VTC Networks — Groups — Register"
summary: "An administrator registers one of the community's groups to serve a role in a Verifiable Trust Network the community belongs to, such as the network's admins."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtc, networks, vtn, groups, roles, broadcast]
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
    Signs a registration as the community and sends it to the network, replacing the group registered for the role if there is one; the network revokes its acceptance of the replaced group.
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
  - code: vtc/networks/groups/register:notMember
    meaning: "The community is not an acknowledged member of the network."
    retryable: false
  - code: vtc/networks/groups/register:unknownGroup
    meaning: "`groupId` names no group of the community's that has a DID of its own."
    retryable: false
  - code: vtc/networks/groups/register:networkRefused
    meaning: "The network refused the registration \u2014 for example, it defines no such role. `details.message` carries its reason when it gave one."
    retryable: true
related:
  - vtc/networks/join
  - vtc/networks/notice
  - vtc/networks/evidence
---

## Abstract

The **VTC Networks — Groups — Register** Trust Task nominates one of the community's groups to serve a role in a Verifiable Trust Network the community belongs to. A network defines roles — `admins`, `notifications` — and reaches the parties serving them with broadcasts that cascade through its member networks ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/) (VTI-VTN-001 – VTI-VTN-063)). The community signs a `servesRole` statement: this group, with this DID, serves this role in this network, and deliveries for it go to the community's relay endpoint for the group. The network answers with a `groupRegistered` statement bound to the registration's digest.

The community, not the group, is the registrant: it is the member, and the group is its. The network learns the group's DID, never who is in it. Broadcasts arrive at the community, which passes them on to the group's members with [`vtc/networks/notice`](../../../notice/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`, with `vtc/networks/groups/register:notMember` when it is not an acknowledged member of `network`, and with `vtc/networks/groups/register:unknownGroup` when `groupId` is not one of its groups with a DID of its own.
2. **MUST** sign the registration as the community, naming the group's DID as subject and its relay endpoint for the group.
3. **MUST** verify the network's acceptance — signed by `network`, naming the group, bound to the registration's digest — before recording it, and refuse with `vtc/networks/groups/register:networkRefused` otherwise.
4. **MUST** relay broadcasts for the role only to the members of the group registered for it, and only while the network's acceptance of that group is in force.
5. **MUST** record the change in its audit trail.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry, read at execution time. For a signer that is a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.2/spec.md)), it is the standing of the identity the key acts for. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's, and so is any further approval it requires before acting: a network membership commits the community to a curator's governance framework and discloses its groups' identifiers to that curator.

## Definitions

- **`network`** — the network's DID.
- **`role`** — the network's role.
- **`groupId`** — the community group to serve it.

## Request

A community administrator (`issuer`) asks the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Serve the network's admins with the community's admins group

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000351",
  "type": "https://trusttasks.org/spec/vtc/networks/groups/register/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-03T10:00:51Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "network": "did:example:whn",
    "role": "admins",
    "groupId": "admins"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### Registered

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000352",
  "type": "https://trusttasks.org/spec/vtc/networks/groups/register/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-03T10:00:52Z",
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
        "id": "urn:uuid:00000000-0000-4000-8000-0000000c0022",
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
          "endpoint": "https://community.example.org/v1/networks/relay/admins",
          "registration": {
            "@context": [
              "https://www.w3.org/ns/credentials/v2",
              "https://registry.trustoverip.org/dtg/context/v1"
            ],
            "id": "urn:uuid:00000000-0000-4000-8000-0000000c0025",
            "type": [
              "VerifiableCredential",
              "DTGCredential",
              "StatementCredential"
            ],
            "issuer": "did:example:community",
            "issuerScope": "public",
            "validFrom": "2026-11-01T09:00:00Z",
            "credentialSubject": {
              "id": "did:example:community-admins-group",
              "predicate": "https://openvtc.org/vtn/vsc/servesRole/1",
              "object": {
                "value": {
                  "network": "did:example:whn",
                  "role": "admins",
                  "endpoint": "https://community.example.org/v1/networks/relay/admins"
                }
              }
            },
            "proof": {
              "type": "DataIntegrityProof",
              "cryptosuite": "eddsa-jcs-2022",
              "proofPurpose": "assertionMethod",
              "verificationMethod": "did:example:community#key-0",
              "proofValue": "z3abc"
            }
          },
          "acceptance": {
            "@context": [
              "https://www.w3.org/ns/credentials/v2",
              "https://registry.trustoverip.org/dtg/context/v1"
            ],
            "id": "urn:uuid:00000000-0000-4000-8000-0000000c0026",
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
        "id": "urn:uuid:00000000-0000-4000-8000-0000000c0023",
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
        "id": "urn:uuid:00000000-0000-4000-8000-0000000c0024",
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
