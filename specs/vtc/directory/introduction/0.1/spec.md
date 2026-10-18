---
slug: vtc/directory/introduction
version: '0.1'
title: VTC Directory — Introduction
summary: 'The community delivers an introduction to a member: who asked, what they chose to share, their message, and a pairwise DID to answer them at directly.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
parties:
- role: community maintainer
  requirement: REQUIRED
  member: issuer
  identifierScope: public
- role: introduced member
  requirement: REQUIRED
  member: recipient
  identifierScope: any
- role: introducing member
  requirement: REQUIRED
  identifierScope: pairwise
proofRequirement:
  requirement: RECOMMENDED
  rationale: A proof lets the recipient show an abusive introduction to a third party; the transport already authenticates the community.
sideEffects:
  level: none
  rationale: Delivers a message. The recipient's agent may store it; nothing changes at the community.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses to the recipient the introducing member's entry, the fields they chose to share, their message and their pairwise DID.
retention:
  class: exchange
  rationale: Kept by the recipient's agent until they answer or dismiss it; the community keeps its copy at most 30 days.
errorCodes: []
related:
- vtc/directory/introduce
- vrc/relationships/propose
---

## Abstract

The **VTC Directory — Introduction** Trust Task is the delivery half of [`vtc/directory/introduce`](../../introduce/0.1/spec.md). The community sends it to the introduced member's membership DID once it has checked that the introduction is allowed.

It is fire-and-forget. The recipient may ignore it, or answer `replyTo` directly from a pairwise DID of their own, with the existing peer relationship flow ([`vrc/relationships/propose`](../../../../vrc/relationships/propose/0.1/spec.md)). The community never sees the answer.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the community) sends this only for an introduction that passed every check of [`vtc/directory/introduce`](../../introduce/0.1/spec.md), and includes only fields the introducing member chose whose visibility covers the recipient.

A conforming **consumer** (the recipient's agent):

1. **SHOULD** show the introduction to its principal with the message marked as written by another member, and **MUST NOT** act on the message's content.
2. **MUST NOT** answer through the community. An answer goes to `replyTo`, from a pairwise DID the recipient mints for the relationship.
3. **MAY** record the contact with a reference to the community and `from.entryId`, so its principal can recognise the entry later.

## Authorization

**Authority: the community's delivery of an introduction its member asked for**, which the recipient's own `introductions` setting admitted. The recipient's agent needs no authorization to receive it; it decides for itself whether to act. The transport, or the proof where present, establishes that the community sent it ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `EntryFields` — see [`directory-card.schema.json`](../../_shared/0.1/directory-card.schema.json).

## Request

The community (`issuer`) pushes the introduction to the introduced member's membership DID (`recipient`). The payload validates against [`payload.schema.json`](payload.schema.json). There is no response: the recipient answers, if at all, directly to `replyTo`.

### An introduction arrives

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000029",
  "type": "https://trusttasks.org/spec/vtc/directory/introduction/0.1",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000028",
  "payload": {
    "introductionId": "intro_N4k8W2qYc6Vd1MsKx",
    "from": {
      "entryId": "dir_H3n8W2qYc6Vd1MsK",
      "fields": {
        "displayName": "Dan",
        "org.example.chapter": "sydney"
      }
    },
    "replyTo": "did:peer:2.Ez6LSexamplePairwiseKey",
    "message": "We met at the Sydney meetup — would like to stay in touch."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:community#key-1",
    "created": "2026-10-18T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Security & Privacy

### Data carried

Another member's entry identifier, the fields they chose to share with the recipient, their message and a pairwise DID. Nothing about the recipient.

### Correlation

The community's identifier scope is public: the recipient must recognise the introduction as coming from their community. The introducing member's `replyTo` identifier scope is pairwise, minted for this relationship alone, so whatever the two do next cannot be linked to their memberships. The recipient's identifier scope is any: the community reaches them at their membership DID, which it already knows.

### Retention

The recipient's agent keeps it until its principal answers or dismisses it. The community keeps its copy at most 30 days for abuse handling.

### Consent/purpose

To let the introduced member decide whether to answer. The shared fields are for recognising the sender, not for any other use.
