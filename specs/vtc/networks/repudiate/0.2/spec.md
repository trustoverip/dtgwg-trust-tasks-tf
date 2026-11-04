---
slug: vtc/networks/repudiate
version: "0.2"
title: "VTC Networks — Repudiate"
summary: "An administrator rejects a Verifiable Trust Network's listing of the community, which the network must then publish beside the listing."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtc, networks, vtn, repudiation, listing]
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
    Creates the community's membership record in state repudiated, signs a repudiation as the community and sends it to the network.
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
  - code: vtc/networks/repudiate:notListed
    meaning: "The network does not list the community, so there is nothing to repudiate."
    retryable: false
  - code: vtc/networks/repudiate:networkMismatch
    meaning: "The network at `url` answers as a DID other than `network`."
    retryable: false
  - code: vtc/networks/repudiate:isMember
    meaning: "The community is a member of the network; it leaves with vtc/networks/leave."
    retryable: false
  - code: vtc/networks/repudiate:networkRefused
    meaning: "The network refused the repudiation. `details.message` carries its reason when it gave one."
    retryable: true
related:
  - vtc/networks/join
  - vtc/networks/leave
  - vtn/membership/repudiate
---

## Abstract

The **VTC Networks — Repudiate** Trust Task is how a community says publicly that it does not accept a Verifiable Trust Network's listing of it. A network may list a community without its agreement, and must publish a repudiation it receives alongside the entry ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/) (VTI-VTN-001 – VTI-VTN-063), VTI-VTN-012). The community reads its entry at the network ([`vtn/entries/show`](../../../../vtn/entries/show/0.1/spec.md)), signs a repudiation statement naming the listing by digest, and sends it to the network with [`vtn/membership/repudiate`](../../../../vtn/membership/repudiate/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Changes from 0.1

- **The community reaches the network only through `vtn/*` Trust Tasks**: it reads the listing with `vtn/entries/show` and sends the repudiation with `vtn/membership/repudiate`, each signed as the community for `authentication` and addressed to `network`, and it verifies the network's proof on each answer (items 3 and 4).
- `membership` follows [`_shared/0.7`](../../../_shared/0.7/network-membership.schema.json).

The request, the response's shape and the error codes are unchanged. Released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller without administrator standing with `permissionDenied`.
2. **MUST** check the network's DID at `url` as [`vtc/networks/join`](../../join/0.2/spec.md) item 2 does, and refuse with `vtc/networks/repudiate:networkMismatch` before signing anything.
3. **MUST** send each request to the network as `vtc/networks/join` item 3 does — signed as the community for `authentication`, with `recipient` `network`, verifying the network's proof on the answer — and **MUST** read its entry with [`vtn/entries/show`](../../../../vtn/entries/show/0.1/spec.md) and verify the listing as `vtc/networks/join` item 4 does (else `vtc/networks/repudiate:notListed`). It **MUST** refuse with `vtc/networks/repudiate:isMember` when the community is an acknowledged member of the network.
4. **MUST** sign the repudiation as the community, for `assertionMethod`, naming the listing by its digest, send it with [`vtn/membership/repudiate`](../../../../vtn/membership/repudiate/0.1/spec.md), and **MUST** record the membership in state `repudiated` only once the network has accepted it; otherwise it refuses with `vtc/networks/repudiate:networkRefused` and records nothing.
5. **MUST** record the change, with `reason`, in its audit trail. `reason` is not sent to the network.

## Authorization

The authority this task presupposes is **administrator standing at the community**: the signer's access-control entry, read at execution time. For a signer that is a delegated signing key ([`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md)), it is the standing of the identity the key acts for. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision is the community's, and so is any further approval it requires before acting: a network membership commits the community to a curator's governance framework and discloses its groups' identifiers to that curator.

## Definitions

- **`url`**, **`network`** — as in [`vtc/networks/join`](../../join/0.2/spec.md).
- **`reason`** — the administrator's reason, for the audit trail only.

## Request

A community administrator (`issuer`) asks the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Repudiate a listing

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000331",
  "type": "https://trusttasks.org/spec/vtc/networks/repudiate/0.2#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-03T10:00:31Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "url": "https://whn.example.org",
    "network": "did:example:whn",
    "reason": "We did not agree to be curated by this network."
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### The repudiated listing

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000332",
  "type": "https://trusttasks.org/spec/vtc/networks/repudiate/0.2#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-11-03T10:00:32Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "membership": {
      "network": "did:example:whn",
      "url": "https://whn.example.org",
      "name": "World Health Network",
      "state": "repudiated",
      "listing": {
        "@context": [
          "https://www.w3.org/ns/credentials/v2",
          "https://registry.trustoverip.org/dtg/context/v1"
        ],
        "id": "urn:uuid:00000000-0000-4000-8000-0000000c0016",
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
      "groups": [],
      "updatedAt": "2026-11-03T10:00:01Z",
      "repudiation": {
        "@context": [
          "https://www.w3.org/ns/credentials/v2",
          "https://registry.trustoverip.org/dtg/context/v1"
        ],
        "id": "urn:uuid:00000000-0000-4000-8000-0000000c0017",
        "type": [
          "VerifiableCredential",
          "DTGCredential",
          "StatementCredential"
        ],
        "issuer": "did:example:community",
        "issuerScope": "public",
        "validFrom": "2026-11-01T09:00:00Z",
        "credentialSubject": {
          "id": "did:example:whn",
          "predicate": "https://registry.trustoverip.org/dtg/vsc/repudiates/1",
          "object": {
            "digestMultibase": "zQmRepudiated"
          }
        },
        "proof": {
          "type": "DataIntegrityProof",
          "cryptosuite": "eddsa-jcs-2022",
          "proofPurpose": "assertionMethod",
          "verificationMethod": "did:example:community#key-0",
          "proofValue": "z3abc"
        }
      }
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
