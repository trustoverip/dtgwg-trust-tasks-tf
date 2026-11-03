---
slug: vtc/networks/list
version: "0.1"
title: "VTC Networks — List"
summary: "An administrator lists the Verifiable Trust Networks the community belongs to, has repudiated or has left."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtc, networks, vtn, membership, curation]
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
    The answer discloses which curators the community has accepted and the group identifiers it gave them, and only administrators may read it, so the reader has to be identified by more than the transport.
sideEffects:
  level: none
  rationale: "Reads the community's membership records; changes nothing."
exposure:
  discloses: metadata
  actsAsSubject: false
  rationale: >-
    Out: network DIDs and URLs, the community's group DIDs, and the credentials between the community and each network. No personal data.
errorCodes: []
related:
  - vtc/networks/show
  - vtc/networks/join
  - vtc/networks/leave
---

## Abstract

The **VTC Networks — List** Trust Task answers which Verifiable Trust Networks (VTNs) the community has dealt with: the ones it is a member of, the ones whose listing of it it repudiated, and the ones it left — each with the credentials that evidence the state and the groups the community registered for the network's roles. A VTN is a curated set of communities and networks ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/) (VTI-VTN-001 – VTI-VTN-063)); listing is one-sided, and membership is the community's acknowledgement of a listing. These tasks are the community's side of that relationship.

Every record is the whole [`NetworkMembership`](../../../_shared/0.6/network-membership.schema.json), so a list answers what [`vtc/networks/show`](../../show/0.1/spec.md) would for each network. Show exists for the caller that holds one network's DID and wants that membership alone.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`.
2. **MUST** answer only with memberships it holds a record of — networks it joined, repudiated or left through these tasks — never with a network that merely lists it. A listing the community has not acted on is the network's assertion, not a fact about the community (VTI-VTN-011).
3. **MUST** order the items by network DID, and page them with `cursor` and `limit` (default 50), returning `nextCursor` when more remain.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry, read at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- **`state`** — only memberships in this state: `acknowledged`, `repudiated` or `departed`.
- **`cursor`**, **`limit`** — paging; `nextCursor` in the response continues it.
- **`items`** — the memberships ([`NetworkMembership`](../../../_shared/0.6/network-membership.schema.json)).

## Request

A community administrator (`issuer`) asks the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### The networks the community belongs to

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000301",
  "type": "https://trusttasks.org/spec/vtc/networks/list/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-03T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "state": "acknowledged"
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### One membership

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000302",
  "type": "https://trusttasks.org/spec/vtc/networks/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-03T10:00:02Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "items": [
      {
        "network": "did:example:whn",
        "url": "https://whn.example.org",
        "name": "World Health Network",
        "state": "acknowledged",
        "listing": {
          "@context": [
            "https://www.w3.org/ns/credentials/v2",
            "https://registry.trustoverip.org/dtg/context/v1"
          ],
          "id": "urn:uuid:00000000-0000-4000-8000-0000000c0001",
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
              "id": "urn:uuid:00000000-0000-4000-8000-0000000c0004",
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
              "id": "urn:uuid:00000000-0000-4000-8000-0000000c0005",
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
          "id": "urn:uuid:00000000-0000-4000-8000-0000000c0002",
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
          "id": "urn:uuid:00000000-0000-4000-8000-0000000c0003",
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
    ]
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
