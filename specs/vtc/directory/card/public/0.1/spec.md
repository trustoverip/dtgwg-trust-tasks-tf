---
slug: vtc/directory/card/public
version: '0.1'
title: VTC Directory — Card — Public
summary: Read a member's external card exactly as they signed it — open to anyone, by entry, while the member keeps it published.
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- directory
- external card
- jscontact
parties:
- role: reader
  requirement: REQUIRED
  member: issuer
  identifierScope: pairwise
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: RECOMMENDED
  rationale: The card is public by its member's choice and the same for every reader. A proof is not required to read it, so a reader need not identify themselves; it is recommended so a community can rate-limit or attribute reads.
sideEffects:
  level: none
  rationale: Reads a published card; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Returns the card a member published so that anyone may read it — personal data they chose to make public, as they signed it.
retention:
  class: transient
  rationale: The request names an entry and nothing about the reader; there is nothing to keep beyond whatever attribution the optional proof carries.
errorCodes:
- code: vtc/directory/card/public:notFound
  meaning: No card is published for this entry — or not the one `cardDigest` names. Conflates an unknown entry, a member who never published, a card withdrawn or awaiting the member's signature, and a member who has left.
  retryable: false
related:
- vtc/directory/card/publish
- vtc/directory/images/get
- registry/record/query
---

## Abstract

The **VTC Directory — Card — Public** Trust Task returns a member's external card: the JSContact card holding their public directory values, signed by them, exactly as they published it with [`vtc/directory/card/publish`](../../publish/0.1/spec.md). Anyone may ask — a reader typically arrives from the pointer in the member's recognition record in the community's trust registry, which names the entry and the card's digest.

The answer is the member's own statement, carrying the member's own proof. A reader verifies that proof against the member's DID, and relies on nothing the community says about the card's contents: the community serves it, it does not vouch for it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`, §5.3).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **reader** (`issuer`):

1. **SHOULD** name the `cardDigest` it was pointed to, and **MUST** check that the digest of the returned `card` ([`card/publish`](../../publish/0.1/spec.md), *Computing `cardDigest`*) equals the returned `cardDigest`.
2. **MUST** verify `card.proof` over the card without it, by a verification method of the DID that signed it authorized for `assertionMethod`, before relying on any member; and **MUST** check that `uid` names the entry it asked for and `issuer` the DID that signed it.
3. **MUST NOT** treat the card as the community's statement. It is the member's. That they are a member is what the recognition record establishes, under its own rules.

A conforming **community** (`recipient`) **MUST**:

1. Answer any reader, with or without a proof. It **MAY** rate-limit reads.
2. Answer only a card the member published, whose `externalCard` state is `published` now, and — where `cardDigest` is present — only the card it names. It **MUST** answer the card exactly as the member signed it, with its `cardDigest` and `publishedAt`.
3. Answer every refusal with `notFound`, so a reader cannot tell a member who never published from one who withdrew, is awaiting their own signature, or left.

## Authorization

This task is not consequential ([SPEC §3](/SPEC.md#3-terminology)): it changes nothing and returns only what its member made public. It needs no authorization. A community that requires a `proof` does so to rate-limit or attribute reads, not to authorize them.

## Definitions

- `PublishedCard`, `EntryId` — see [`directory-card.schema.json`](../../../_shared/0.1/directory-card.schema.json).

## Request

The reader (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A reader follows a registry pointer

No proof: the reader has not identified themselves, and need not.

```json
{
  "id": "urn:uuid:9b4e6f2a-1c3d-4e5f-8a7b-000000000031",
  "type": "https://trusttasks.org/spec/vtc/directory/card/public/0.1",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-19T10:00:00Z",
  "payload": {
    "entryId": "dir_Q2xhcmFRMmxoY21GUk1t",
    "cardDigest": "zQmcjRKZd1TExLaEuMVv5xe2tTx4a1YkRQ58zwVrnByxYS4"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/directory/card/public/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)) with the code declared above.

### Carol's card, as she signed it

```json
{
  "id": "urn:uuid:9b4e6f2a-1c3d-4e5f-8a7b-000000000032",
  "type": "https://trusttasks.org/spec/vtc/directory/card/public/0.1#response",
  "issuer": "did:example:community",
  "issuedAt": "2026-10-19T10:00:01Z",
  "threadId": "urn:uuid:9b4e6f2a-1c3d-4e5f-8a7b-000000000031",
  "payload": {
    "card": {
      "@type": "Card",
      "version": "1.0",
      "uid": "urn:vtc-directory:dir_Q2xhcmFRMmxoY21GUk1t",
      "issuer": "did:example:carol",
      "name": { "full": "Carol Example" },
      "notes": { "n1": { "note": "Rust, decentralised identity, and long walks." } },
      "proof": {
        "type": "DataIntegrityProof",
        "cryptosuite": "eddsa-jcs-2022",
        "verificationMethod": "did:example:carol#key-1",
        "created": "2026-10-19T09:10:00Z",
        "proofPurpose": "assertionMethod",
        "proofValue": "z58DAdFfa9SkqZMVPxAQpic7ndSayn1PzZs6ZjWp1CktyGesjuTSwRdoWhAfGFCF5bppETSTojQCrfFPP2oumHKtz"
      }
    },
    "cardDigest": "zQmcjRKZd1TExLaEuMVv5xe2tTx4a1YkRQ58zwVrnByxYS4",
    "publishedAt": "2026-10-19T09:10:02Z"
  }
}
```

## Security & Privacy

### Data carried

The request names an entry and, optionally, a digest. The response is a card its member chose to make public and signed. A picture on it, where the member made one public, is read with [`vtc/directory/images/get`](../../../images/get/0.1/spec.md), which answers the public for exactly the images on a published card.

### Correlation

The card is a correlator by the member's choice ([`card/publish`](../../publish/0.1/spec.md)). The reader's scope is pairwise and, without a proof, there is no reader identifier at all: reading a card tells the community nothing about who read it beyond, at most, the attribution a proof carries.

The community maintainer's identifier scope is public: a reader finds it from the authority of a recognition record in its trust registry, and must be able to address it by that same DID. A pairwise identifier would break the pointer at its first link.

### Retention

Nothing beyond whatever the community keeps of a read it can attribute. A community **SHOULD NOT** keep anything of an unattributed read beyond what rate limiting needs.

### Consent/purpose

The card exists to be read by anyone, which is what its member signed it for. It is answered only while it is exactly what they signed and they keep it published; withdrawn, it is `notFound` at once.
