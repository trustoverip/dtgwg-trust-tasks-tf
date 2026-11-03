---
slug: vtn/membership/repudiate
version: "0.1"
title: "VTN Membership — Repudiate"
summary: "A listed party repudiates its listing, or a member departs by repudiating its grant."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, vmc, curation, nested-membership, departure]
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
    The proof is the party's authentication: only the party a network lists can reject the listing or leave, so an unattributed repudiation could end a membership in someone else's name.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replay after the party was re-listed and rejoined would end the new membership, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: destructive
  rationale: >-
    Moves the entry to `repudiated` or `departed`, revokes the network's offer (and, on departure, the listing and the grant) on its status list, and drops every group the party registered. Status-list revocation is permanent; only a fresh listing by the network, and a fresh acknowledgement, restores membership.
consequences:
  - "Repudiating the listing: the entry stays published as `repudiated`, with the repudiation beside the listing; the offer is revoked."
  - "Repudiating the grant (departure): the membership ends, the listing, offer and grant are revoked, and the entry is no longer published."
  - "Either way, every group the party registered with the network is dropped, so it receives no further broadcasts for the network's roles."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: the party's repudiation credential. Out: the party's entry as it now stands. Public identifiers of organisations only; no personal data.
retention:
  class: durable
  rationale: >-
    The repudiation is the evidence that the party, not the network, ended the relationship; it is published beside the listing and kept in the audit trail after the entry stops being published.
errorCodes:
  - code: vtn/membership/repudiate:notListed
    meaning: "The network has no entry for the issuer, or none in a state that can be repudiated (`listed`, `acknowledged` or `suspended`)."
    retryable: false
  - code: vtn/membership/repudiate:repudiationInvalid
    meaning: "The repudiation is not a `repudiates/1` statement issued by the requester about this network, does not verify, or names neither the entry's listing nor — for a member — its grant. A member that names its listing is refused too: a member departs by naming the grant. `details.message` says which."
    retryable: false
    detailsSchema:
      type: object
      required: [message]
      properties:
        message:
          type: string
related:
  - vtn/membership/offer
  - vtn/membership/acknowledge
  - vtn/entries/show
  - vtn/groups/register
---

## Abstract

A Verifiable Trust Network may list a party on its own authority (VTI-VTN-010), and may never make it a member without its agreement (VTI-VTN-013) — so the party must also be able to say *no*, and later to *leave* ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-VTN-012). **VTN Membership — Repudiate** carries both. The party issues a `repudiates/1` statement whose object is the digest of a credential the network issued it: the **listing**, to reject being listed, or the **grant**, to depart from a membership it acknowledged. The network publishes the repudiation and moves the entry to `repudiated` or `departed` accordingly.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **listed party** (`issuer`):

1. **MUST** issue the repudiation as a DTG Verifiable Statement Credential (`StatementCredential`) with itself as `issuer`, `credentialSubject.id` the network's DID, `credentialSubject.predicate` `https://registry.trustoverip.org/dtg/vsc/repudiates/1`, and `credentialSubject.object.digestMultibase` the digest of the credential it repudiates, and **MUST** sign it for `assertionMethod`.
2. **MUST** name the listing to reject a listing it never acknowledged, and the grant to depart from a membership; it **MUST** compute the digest as a SHA-256 multihash, multibase-encoded, over the JCS canonicalization of that credential with its `proof` removed.
3. **MUST** check that the entry in the response is in the state it asked for (`repudiated` or `departed`) and carries its repudiation.

A conforming **network** (`recipient`):

1. **MUST** refuse with `vtn/membership/repudiate:notListed` unless it has an entry for `issuer` in state `listed`, `acknowledged` or `suspended`.
2. **MUST** verify the repudiation — issued by `issuer`, signed for `assertionMethod` by a key of `issuer`, within its validity period, a statement under the `repudiates/1` predicate, naming the network as its subject — and **MUST** decode its `digestMultibase` and compare the digest bytes against the listing and, where it exists, the grant. It **MUST** refuse with `vtn/membership/repudiate:repudiationInvalid` on any failure, when the digest matches neither, and when it names the listing of an entry that is `acknowledged` or `suspended`.
3. On a repudiated **listing** (entry `listed`): **MUST** revoke the offer on its status list and drop it, keep the listing published with the repudiation beside it, and move the entry to `repudiated`.
4. On a repudiated **grant** (entry `acknowledged` or `suspended`): **MUST** revoke the listing, the offer if any, and the grant on its status list, and move the entry to `departed`, after which it is no longer published.
5. In both cases **MUST** drop every group `issuer` registered ([`vtn/groups/register`](../../../groups/register/0.1/spec.md)) so that no broadcast reaches it through them, publish the repudiation with the entry, and record the change in its audit trail with `issuer` as the actor.
6. **MUST NOT** refuse a valid repudiation for any reason of its own policy: the network cannot hold a party it lists, or a member, against its will (VTI-VTN-013), and it **MUST NOT** change the party's own membership, trust contexts or keys in response (VTI-VTN-060 – VTI-VTN-063).

## Authorization

The authority this task presupposes is **being the listed party**: only the DID an entry names can repudiate it. The request's proof shows who sent the document, and the repudiation's own proof shows who repudiated; it is the match between both and the entry's DID that entitles the requester (network items 1 and 2). No standing at the network is needed beyond that, and the network has no discretion to refuse a valid repudiation (network item 6). A consumer **MAY** require an operation-bound step-up of the party's own operator before acting on a departure, as policy of its own.

## Definitions

- **Listing** — the network's `lists/1` credential naming the party (VTI-VTN-010). Repudiating it rejects being listed; it does not delete the listing, which stays published with the repudiation beside it.
- **Grant** — the network's signed membership credential for the party, which exists once the party acknowledged. Repudiating it is **departure**.
- **`repudiation`** — the party's `repudiates/1` statement, whose `credentialSubject.object.digestMultibase` names the listing or the grant ([`Credential`](../../../_shared/0.1/entry.schema.json)).
- **`entry`** — the party's entry as it now stands ([`IndexEntry`](../../../_shared/0.1/entry.schema.json)), in state `repudiated` or `departed`, with `acknowledged` `false`.

## Request

The listed party (`issuer`) sends its repudiation to the network (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A member departs

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000701",
  "type": "https://trusttasks.org/spec/vtn/membership/repudiate/0.1#request",
  "issuer": "did:example:community",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T11:00:00Z",
  "payload": {
    "repudiation": {
      "@context": [
        "https://www.w3.org/ns/credentials/v2",
        "https://registry.trustoverip.org/dtg/context/v1"
      ],
      "id": "urn:uuid:00000000-0000-4000-8000-0000000c0051",
      "type": ["VerifiableCredential", "DTGCredential", "StatementCredential"],
      "issuer": "did:example:community",
      "issuerScope": "public",
      "validFrom": "2026-11-03T11:00:00Z",
      "credentialSubject": {
        "id": "did:example:whn",
        "predicate": "https://registry.trustoverip.org/dtg/vsc/repudiates/1",
        "object": {
          "digestMultibase": "zQmGrantDigest"
        }
      },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "proofPurpose": "assertionMethod",
        "verificationMethod": "did:example:community#key-0",
        "proofValue": "z3def"
      }
    }
  }
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`: the party's entry as it now stands. A refusal is a `trust-task-error`.

### The party has departed

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000702",
  "type": "https://trusttasks.org/spec/vtn/membership/repudiate/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-03T11:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000701",
  "payload": {
    "entry": {
      "did": "did:example:community",
      "kind": "community",
      "state": "departed",
      "acknowledged": false,
      "label": "St Vincent's",
      "profile": {},
      "listing": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0011" },
      "grant": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0012" },
      "acknowledgement": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0021" },
      "repudiation": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0051" }
    }
  }
}
```

The credentials are abbreviated to their `id`; the network returns them in full.

## Security & Privacy

### Data carried

The request carries one statement credential between two organisations' public identifiers; the response carries the party's entry. Neither carries personal data. A party **MUST NOT** put anything in the repudiation beyond what the `repudiates/1` predicate requires: it is published.

### Correlation

The repudiation is published by design, so that anyone reading the network can see that the party rejected the listing or left, and that the network did not decide it. Both parties declare `identifierScope: public` because they are organisations whose identifiers are public by design: the repudiation has to name the network, and be checkable against the party the network listed, the same way every time. Nothing here is more correlatable than the listing or membership it ends.

### Retention

A `repudiated` entry stays published, with its repudiation, for as long as the network keeps the listing. A `departed` entry is no longer published, but the network keeps the repudiation in its audit trail as the evidence of who ended the membership and when.

### Consent/purpose

The repudiation withdraws the party's consent — to being listed, or to being published as a member — and is used for nothing else. It cannot be withdrawn in turn: a party that wants back in is re-listed by the network and acknowledges afresh.
