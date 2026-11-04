---
slug: vtn/listings/create
version: "0.1"
title: "VTN Listings — Create"
summary: "An administrator lists a community or network: the network signs a listing credential and offers an unsigned membership grant, and the entry starts `listed`, which is never membership."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, curation, listing, vsc]
parties:
  - role: administrator
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
    The proof is the administrator's authentication: the network signs a credential on its own authority because of who asked, so an unattributed request would let anyone make the network vouch for a party.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed listing after the entry was removed would re-list the party, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Creates an entry, signs and publishes a listing credential, and publishes an unsigned membership offer. Undone by vtn/listings/remove, which revokes what the network issued.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: an organisation's DID and the directory profile an administrator writes for it. Out: the new entry with the listing credential and the offer. Public identifiers of organisations only; no personal data.
retention:
  class: durable
  rationale: >-
    The entry and its listing credential are what the network publishes and verifiers check; they are kept for as long as the entry is, and in the audit trail after it.
errorCodes: []
related:
  - vtn/listings/update
  - vtn/listings/remove
  - vtn/listings/show
  - vtn/membership/offer
  - vtn/membership/acknowledge
  - auth/signing-key/enroll
---

## Abstract

A Verifiable Trust Network curates a set of communities and networks. **VTN Listings — Create** is how its administrators add one. The network signs a listing credential about the party on its own authority, which needs nothing from the party ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-VTN-010) and establishes nothing about it (VTI-VTN-020). It also prepares the membership grant it would give the party, unsigned, as an offer. The entry starts `listed`. A listed party is not a member until it acknowledges the offer itself ([`vtn/membership/acknowledge`](../../../membership/acknowledge/0.1/spec.md)); unacknowledged is never membership (VTI-VTN-013).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **administrator** (`issuer`):

1. **MUST** choose `kind` for what the party is: `community` for a Verifiable Trust Community, `network` for another network.
2. **MUST NOT** present the response as the party's membership. The entry is `listed` until the party acknowledges.

A conforming **network** (`recipient`):

1. **MUST** refuse with `permissionDenied` unless `issuer` is one of its configured administrators, or a key delegated by one through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md), read from its configuration when it executes the request.
2. **MUST** refuse with `malformedRequest` when `did` is the network's own DID. A network does not list itself.
3. **MUST** refuse with `alreadyExists` when it has an entry for `did` in state `listed`, `acknowledged` or `suspended`. This is a creating operation ([SPEC §7.2](/SPEC.md#72-consumer-requirements)): the existing entry is not touched.
4. **MAY** list a party whose entry is `repudiated`, `removed` or `departed`. The entry starts over at `listed` with a new listing credential and a new offer, and keeps its history. The network **MUST** revoke the superseded listing credential if it is not already revoked.
5. **MUST** sign, for `assertionMethod`, a listing credential: a DTG verifiable statement credential issued by the network about `did`, with predicate `https://registry.trustoverip.org/dtg/vsc/lists/1` and `object.value` `{ "nodeType": "VTC" | "VTN", "framework": { "iri", "version" } }`. `nodeType` is `VTC` for `kind` `community` and `VTN` for `network` (VTI-VTN-031). `framework` is the governance framework the network operates under. The credential **MUST** carry a `credentialStatus` entry in the network's status lists, and the network **MUST** serve it by its `id`.
6. **MUST** build the membership offer: the membership grant (VMC) it would issue to `did`, with its own `credentialStatus` entry, and **MUST NOT** sign it. It signs the grant only when the party acknowledges it (VTI-VTN-013).
7. **MUST** create the entry in state `listed`, record the transition in the entry's history and in its audit trail naming `issuer` as the actor, and answer with the entry.
8. **MUST NOT** publish a `listed` entry as a member. Every entry it publishes carries its acknowledgement state (VTI-VTN-013).
9. **MUST NOT** change the party's own membership, trust contexts or keys (VTI-VTN-060 – VTI-VTN-063). Listing is one-sided and grants nothing.

## Authorization

The authority this task presupposes is **administrator standing on the network**. The `issuer` is one of the administrators named in the network's configuration, or a key that one of them delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md). The network reads that standing when it executes the request (network item 1). It does not rely on when a key was enrolled or on an earlier answer, so an administrator removed from the configuration, or a delegation revoked, loses the authority at once. The proof establishes who signed and that the document is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The match between the signer and the configured administrators is what entitles it. Nothing is needed from the listed party, because listing is the network's own one-sided statement (VTI-VTN-010). A consumer **MAY** require an operation-bound step-up before it signs the listing; that is its own policy.

## Definitions

- **`did`**: the party to list, a community or another network.
- **`kind`**: `community` or `network` ([`EntryKind`](../../../_shared/0.1/entry.schema.json)).
- **`label`**, **`description`**, **`url`**, **`tags`**: the directory profile the administrator writes for the entry. They are the network's description and are in no credential.
- **Listing credential**: the `lists/1` statement credential, signed by the network. It says the network lists the party under its framework, and nothing more (VTI-VTN-020).
- **Offer**: the unsigned membership grant, which the party fetches through [`vtn/membership/offer`](../../../membership/offer/0.1/spec.md) and acknowledges.
- **`entry`**: the administrator's view of the entry ([`Entry`](../../../_shared/0.1/entry.schema.json)), with the listing and the offer as `Issued` records (credential plus status-list index).

## Request

An administrator (`issuer`) asks the network (`recipient`) to list a party. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### An administrator lists a community

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000c01",
  "type": "https://trusttasks.org/spec/vtn/listings/create/0.1#request",
  "issuer": "did:example:admin",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000cff",
  "payload": {
    "did": "did:example:community",
    "kind": "community",
    "label": "St Vincent's",
    "description": "A teaching hospital network.",
    "url": "https://community.example",
    "tags": ["hospital", "nsw"]
  }
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`: the new entry, in state `listed`. A refusal is a `trust-task-error`.

### The party is listed, not a member

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000c02",
  "type": "https://trusttasks.org/spec/vtn/listings/create/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:admin",
  "issuedAt": "2026-11-03T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000cff",
  "payload": {
    "entry": {
      "did": "did:example:community",
      "kind": "community",
      "label": "St Vincent's",
      "profile": {
        "description": "A teaching hospital network.",
        "url": "https://community.example",
        "tags": ["hospital", "nsw"]
      },
      "state": "listed",
      "createdAt": "2026-11-03T10:00:01Z",
      "updatedAt": "2026-11-03T10:00:01Z",
      "listing": {
        "credential": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0011" },
        "statusIndex": 41
      },
      "offer": {
        "credential": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0012" },
        "statusIndex": 42
      },
      "history": [
        { "changedAt": "2026-11-03T10:00:01Z", "to": "listed" }
      ]
    }
  }
}
```

The credentials are abbreviated to their `id`; the network returns them in full.

## Security & Privacy

### Data carried

The request carries an organisation's DID and a short directory profile; the response carries the entry, a signed listing credential and an unsigned offer. None of it is personal data. The profile is published in the directory, so an administrator **MUST NOT** put anything in `label`, `description`, `url` or `tags` that the network would not publish, such as contact details of a person or internal notes. `ext` is not a place for those either.

### Correlation

The listing is published by design. It is the network's public statement that it lists the party, so anyone who may read the directory learns the relationship. Both parties declare `identifierScope: public`. The listed party is named by the public DID that verifiers and other networks use for it, and the network by the DID its credentials are issued under, so a pairwise identifier would defeat the listing. The administrator's DID is recorded in the audit trail as the actor; an operator that does not want its administrators' identities joined across networks can give them network-specific DIDs.

### Retention

The network keeps the entry, its listing credential and its offer for as long as the entry exists. It keeps the history and audit trail after the entry is removed, because they record who listed whom and when. The request itself is kept as long as duplicate detection needs it.

### Consent/purpose

Listing does not need the party's consent; it is the network's one-sided statement, and it grants nothing (VTI-VTN-010, VTI-VTN-020). The party's agreement is asked for separately, by the offer, and only an acknowledgement makes it a member. A party that does not want to be listed can repudiate the listing ([`vtn/membership/repudiate`](../../../membership/repudiate/0.1/spec.md)). The profile is collected to describe the party in the directory and is used for nothing else.
