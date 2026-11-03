---
slug: vtn/membership/offer
version: "0.1"
title: "VTN Membership — Offer"
summary: "A listed party fetches the network's unsigned membership grant, so it can acknowledge it."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, vmc, curation, nested-membership]
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
  requirement: REQUIRED
  rationale: >-
    The offer names the party and is handed only to it; the proof is the party's authentication, so that probing DIDs does not reveal who a restricted network lists.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read that changes nothing, so a replay discloses only what the original did; the network still wants to place a proved request in its acceptance window.
sideEffects:
  level: none
  rationale: >-
    Returns the offer the network created when it listed the party; creates and changes nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    In: nothing beyond the requester's own identity. Out: an unsigned membership credential naming the party and the network, both organisations with public identifiers.
retention:
  class: transient
  rationale: >-
    The request carries nothing to keep; the offer itself is the network's own record, kept with the entry.
errorCodes:
  - code: vtn/membership/offer:notListed
    meaning: "The network has no entry for the issuer, or none in a state that holds an offer: the party is not listed, repudiated the listing, departed or was removed."
    retryable: false
  - code: vtn/membership/offer:alreadyMember
    meaning: "The issuer's entry is `acknowledged` or `suspended`: the grant is already signed, and there is no offer left to acknowledge."
    retryable: false
related:
  - vtn/membership/acknowledge
  - vtn/membership/repudiate
  - vtn/groups/register
  - vtn/roles/list
---

## Abstract

A Verifiable Trust Network lists a community or network on its own authority (VTI-VTN-010), and membership needs the party's agreement on top ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-VTN-013). The agreement is an acknowledgement over the digest of the network's membership grant — so the party has to see the grant before the network signs it. **VTN Membership — Offer** hands the listed party that grant, unsigned: the *offer*. The party then acknowledges it with [`vtn/membership/acknowledge`](../../acknowledge/0.1/spec.md), and the network signs the very credential the acknowledgement's digest names.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **listed party** (`issuer`):

1. **MUST** sign the request as itself, or with a key it delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md).
2. **MUST** check that the offer is a membership credential issued by the network, naming the party as its subject, before acknowledging it, and **MUST** compute its acknowledgement's `digestMultibase` over the offer exactly as received.
3. **MUST NOT** present the offer as membership, or as evidence of anything: it is unsigned, and the network has not granted it (VTI-VTN-013).

A conforming **network** (`recipient`):

1. **MUST** answer only the party its entry names: the offer for `issuer`'s own entry, never another's. A request carries no DID to look up, so it cannot be used to probe who is listed.
2. **MUST** refuse with `vtn/membership/offer:notListed` when it has no entry for `issuer`, or the entry is `repudiated`, `removed` or `departed`, and with `vtn/membership/offer:alreadyMember` when the entry is `acknowledged` or `suspended`.
3. **MUST** return, for an entry in state `listed`, the membership grant it created when it listed the party: a membership credential issued by the network, subject the party, carrying its `credentialStatus`, and without a `proof`.
4. **MUST** return the same offer, unchanged, on every request until the entry changes state, so that a digest the party computed stays the digest of the grant the network will sign.
5. **MUST NOT** sign the offer here, or treat the request as the party's agreement to anything.

## Definitions

- **Offer** — the network's membership grant for the party, without its `proof`. Its `id`, `validFrom` and `credentialStatus` are fixed when the network lists the party, so it is the credential the network signs on acknowledgement, byte for byte under JCS.
- **`offer`** — that credential ([`Credential`](../../../_shared/0.1/entry.schema.json)).

The request payload is empty: the party asks about itself, and `issuer` says who it is.

## Request

The listed party (`issuer`) asks the network (`recipient`) for its offer. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A listed community asks for its offer

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000601",
  "type": "https://trusttasks.org/spec/vtn/membership/offer/0.1#request",
  "issuer": "did:example:community",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T09:55:00Z",
  "payload": {}
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`: the unsigned grant. A refusal is a `trust-task-error`.

### The offer

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000602",
  "type": "https://trusttasks.org/spec/vtn/membership/offer/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-03T09:55:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000601",
  "payload": {
    "offer": {
      "@context": [
        "https://www.w3.org/ns/credentials/v2",
        "https://registry.trustoverip.org/dtg/context/v1"
      ],
      "id": "urn:uuid:00000000-0000-4000-8000-0000000c0012",
      "type": ["VerifiableCredential", "DTGCredential", "MembershipCredential"],
      "issuer": "did:example:whn",
      "issuerScope": "public",
      "validFrom": "2026-11-02T08:00:00Z",
      "credentialSubject": {
        "id": "did:example:community"
      },
      "credentialStatus": {
        "id": "https://whn.example/v1/status/revocation#12",
        "type": "BitstringStatusListEntry",
        "statusPurpose": "revocation",
        "statusListIndex": "12",
        "statusListCredential": "https://whn.example/v1/status/revocation"
      }
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries nothing but the requester's identity. The response carries one unsigned credential between two organisations' public identifiers. Neither carries personal data.

### Correlation

The offer names the party and the network, which is what the membership will publish once acknowledged. Both parties declare `identifierScope: public` because they are organisations whose identifiers are public by design: the offer is bound to the DID the network listed, and only that DID can acknowledge it. Answering only the party itself (network item 1) keeps a restricted network's listings from being enumerated through this task.

### Retention

The network keeps nothing from the request. The offer is part of the entry and is dropped once it is signed (on acknowledgement) or revoked (on repudiation or removal).

### Consent/purpose

Fetching the offer is not consent to anything: the party consents by acknowledging it, and may instead repudiate the listing ([`vtn/membership/repudiate`](../../repudiate/0.1/spec.md)). The offer is handed to the party for that decision only.
