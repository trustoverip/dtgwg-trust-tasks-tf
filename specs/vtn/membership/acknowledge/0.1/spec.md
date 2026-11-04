---
slug: vtn/membership/acknowledge
version: "0.1"
title: "VTN Membership — Acknowledge"
summary: "A listed party acknowledges a network's membership offer, and the network signs the grant it binds: the party becomes a member."
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
    The proof is the party's authentication: the network admits exactly the DID it lists, so an unattributed acknowledgement could make anyone a member in a party's name.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replay after the party departed would re-admit it, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Signs and publishes the membership grant and moves the entry to `acknowledged`. Undone by vtn/membership/repudiate naming the grant (departure), or by the network removing the entry.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: the party's acknowledgement credential. Out: the party's published entry, with the network's signed grant. Public identifiers of organisations only; no personal data.
retention:
  class: durable
  rationale: >-
    The acknowledgement and the grant are the evidence of membership that the network publishes and verifiers rely on; they are kept for as long as the entry is, and in the audit trail after it.
errorCodes:
  - code: vtn/membership/acknowledge:notListed
    meaning: "The network has no entry for the issuer, or none in a state that can be acknowledged (`listed`)."
    retryable: false
  - code: vtn/membership/acknowledge:acknowledgementInvalid
    meaning: "The acknowledgement is not a membership credential issued by the requester, does not verify, or does not carry the digest of the grant the network offered. `details.message` says which."
    retryable: false
  - code: vtn/membership/acknowledge:rolesUnregistered
    meaning: "The network requires a group for one or more roles and the party has registered none. `details.roles` lists them."
    retryable: true
    detailsSchema:
      type: object
      required: [roles]
      properties:
        roles:
          type: array
          items:
            type: string
related:
  - vtn/membership/offer
  - vtn/membership/repudiate
  - vtn/groups/register
  - vtc/networks/join
---

## Abstract

A Verifiable Trust Network lists communities and networks on its own authority ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-VTN-010), but a listing is never membership: membership needs the listed party's agreement (VTI-VTN-013). **VTN Membership — Acknowledge** is that agreement. The party has fetched the network's unsigned membership offer ([`vtn/membership/offer`](../../offer/0.1/spec.md)) and signed an acknowledgement over its digest; it sends the acknowledgement here, and the network signs the very grant the digest names. Both credentials then exist together — a completed membership credential (VMC) pair — and the network publishes the party as a member.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **listed party** (`issuer`):

1. **MUST** build the acknowledgement over the offer it fetched through `vtn/membership/offer`, as a membership credential it issues whose `digestMultibase` is the digest of that offer, and **MUST** sign it for `assertionMethod`.
2. **MUST** register a group for each role the network's profile marks required ([`vtn/groups/register`](../../../groups/register/0.1/spec.md)) before acknowledging.
3. **MUST** verify that the grant in the response is signed by the network, names the party, and is the credential its acknowledgement's digest binds, and **MUST NOT** treat itself as a member otherwise.

A conforming **network** (`recipient`):

1. **MUST** refuse with `vtn/membership/acknowledge:notListed` unless it has an entry for `issuer` in state `listed`. An entry already `acknowledged` **MUST** be refused with `alreadyExists`.
2. **MUST** verify the acknowledgement as a membership credential issued by `issuer` — its proof for `assertionMethod`, its subject the network — whose `digestMultibase` equals the digest of the offer the network holds for the entry, and **MUST** refuse with `vtn/membership/acknowledge:acknowledgementInvalid` on any failure.
3. **MUST** refuse with `vtn/membership/acknowledge:rolesUnregistered` while a role its profile marks required has no group registered by `issuer`.
4. **MUST** then sign the offered grant unchanged — so the acknowledgement's digest binds it — publish it with the acknowledgement, move the entry to `acknowledged`, and record the change in its audit trail naming `issuer` as the actor.
5. **MUST NOT** sign a grant naming a party that has not acknowledged (VTI-VTN-013), and **MUST NOT** change the party's own membership, trust contexts or keys (VTI-VTN-060 – VTI-VTN-063).

## Authorization

The authority this task presupposes is **being the listed party**: the network accepts an acknowledgement only from the DID its entry names, and only for that entry. The proof establishes who sent the document and the acknowledgement's own proof establishes who agreed; it is the match between both and the entry's DID that entitles the requester (network item 2). The network's decision to list the party was made earlier, by its administrators, and is not revisited here; a network **MAY** additionally hold an acknowledgement for its own review before signing the grant.

## Definitions

- **Offer** — the network's membership grant for the party, published unsigned, so the party can acknowledge it before it exists in signed form.
- **`acknowledgement`** — the party's membership credential carrying the offer's `digestMultibase`.
- **Grant** — the offer, signed by the network. Because the digest excludes `proof`, the signed grant is byte-for-byte the credential the acknowledgement names.
- **`entry`** — the party's published entry ([`IndexEntry`](../../../_shared/0.1/entry.schema.json)), now carrying the grant and the acknowledgement.

## Request

The listed party (`issuer`) sends its acknowledgement to the network (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A community acknowledges a network's offer

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000401",
  "type": "https://trusttasks.org/spec/vtn/membership/acknowledge/0.1#request",
  "issuer": "did:example:community",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000004ff",
  "payload": {
    "acknowledgement": {
      "@context": [
        "https://www.w3.org/ns/credentials/v2",
        "https://registry.trustoverip.org/dtg/context/v1"
      ],
      "id": "urn:uuid:00000000-0000-4000-8000-0000000c0021",
      "type": ["VerifiableCredential", "DTGCredential", "MembershipCredential"],
      "issuer": "did:example:community",
      "issuerScope": "public",
      "validFrom": "2026-11-03T10:00:00Z",
      "credentialSubject": {
        "id": "did:example:whn",
        "digestMultibase": "zQmOfferDigest"
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
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`: the party's entry as it is now published. A refusal is a `trust-task-error`.

### The party is a member

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000402",
  "type": "https://trusttasks.org/spec/vtn/membership/acknowledge/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-03T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000004ff",
  "payload": {
    "entry": {
      "did": "did:example:community",
      "kind": "community",
      "state": "acknowledged",
      "acknowledged": true,
      "label": "St Vincent's",
      "profile": {},
      "listing": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0011" },
      "grant": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0012" },
      "acknowledgement": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0021" }
    }
  }
}
```

The credentials are abbreviated to their `id`; the network returns them in full.

## Security & Privacy

### Data carried

The request carries one credential between two organisations' public identifiers; the response carries the published entry. Neither carries personal data. A party **MUST NOT** put anything in the acknowledgement beyond what a membership credential requires: it is published.

### Correlation

The acknowledgement and the grant are published by design — they are what makes the membership verifiable — so the relationship between the party and the network is public to whoever may read the network's directory. Both parties declare `identifierScope: public` because they are organisations whose identifiers are public by design: a network can only list, and a verifier can only check, a party it can name the same way every time. Nothing here is more correlatable than the membership itself.

### Retention

The network keeps both credentials for as long as the entry exists, and in its audit trail afterwards, because they are the evidence of what the party agreed to and when.

### Consent/purpose

The acknowledgement is the party's consent to being published as a member, and it is used for nothing else. The party withdraws it by departing ([`vtn/membership/repudiate`](../../repudiate/0.1/spec.md) naming the grant).
