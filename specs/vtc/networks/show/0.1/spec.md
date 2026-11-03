---
slug: vtc/networks/show
version: "0.1"
title: "VTC Networks — Show"
summary: "An administrator reads one of the community's network memberships: its state, the credentials that evidence it, and the groups registered for the network's roles."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtc, networks, vtn, membership]
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
    Only administrators may read a membership, so the reader has to be identified by more than the transport.
sideEffects:
  level: none
  rationale: "Reads one membership record; changes nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
  rationale: >-
    Out: one network's DID and URL, the community's group DIDs registered with it, and the credentials between them. No personal data.
errorCodes: []
related:
  - vtc/networks/list
  - vtc/networks/groups/register
---

## Abstract

The **VTC Networks — Show** Trust Task reads one of the community's memberships of a Verifiable Trust Network by the network's DID: its state, the network's listing, grant and the community's acknowledgement or repudiation, and the groups the community registered for the network's roles with the network's acceptance of each ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/) (VTI-VTN-001 – VTI-VTN-063)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`.
2. **MUST** refuse with `notFound` a network it holds no membership record for.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry, read at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- **`network`** — the network's DID.
- **`membership`** — the record ([`NetworkMembership`](../../../_shared/0.6/network-membership.schema.json)).

## Request

A community administrator (`issuer`) asks the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### One network

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000311",
  "type": "https://trusttasks.org/spec/vtc/networks/show/0.1#request",
  "issuer": "did:example:administrator",
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

### A membership with its admins group registered

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000312",
  "type": "https://trusttasks.org/spec/vtc/networks/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-03T10:00:12Z",
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
        "id": "urn:uuid:00000000-0000-4000-8000-0000000c0006",
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
            "id": "urn:uuid:00000000-0000-4000-8000-0000000c0009",
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
            "id": "urn:uuid:00000000-0000-4000-8000-0000000c0010",
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
        "id": "urn:uuid:00000000-0000-4000-8000-0000000c0007",
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
        "id": "urn:uuid:00000000-0000-4000-8000-0000000c0008",
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
