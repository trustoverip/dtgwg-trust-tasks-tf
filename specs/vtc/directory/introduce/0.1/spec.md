---
slug: vtc/directory/introduce
version: '0.1'
title: VTC Directory — Introduce
summary: A member asks the community to introduce them to another member's entry, offering a fresh pairwise DID to reply to. The answer is always `accepted`, so the request reveals nothing about the other member.
status: draft
targetFrameworkVersion: 0.6.0
category: governance
parties:
- role: member
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
- role: introduced member
  requirement: REQUIRED
  identifierScope: pairwise
proofRequirement:
  requirement: REQUIRED
  rationale: The community delivers the introduction in the sender's name and keeps it for abuse handling; it must be attributable to the sender after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed introduction would re-deliver a message its sender sent once.
sideEffects:
  level: mutating
  rationale: May push an introduction to another member and records it for 30 days for abuse handling. Nothing is undone; the record lapses.
exposure:
  discloses: none
  ingests: personal
  actsAsSubject: true
  rationale: 'Carries a short message, a pairwise DID and the sender''s choice of their own fields to share. Discloses nothing: the answer is the same whatever happened.'
retention:
  class: exchange
  rationale: The introduction is kept 30 days, for abuse handling only, then deleted.
errorCodes:
- code: vtc/directory/introduce:notMember
  meaning: The signer is not an active member of this community. (Concerns only the sender.)
  retryable: false
- code: vtc/directory/introduce:rateLimited
  meaning: The sender has asked for too many introductions, counted over their own requests whatever became of them.
  retryable: true
related:
- vtc/directory/introduction
- vtc/directory/entries/show
- vrc/relationships/propose
---

## Abstract

The **VTC Directory — Introduce** Trust Task lets a member reach another member **without either learning the other's DID**. The sender's agent mints a fresh pairwise DID for the relationship; the sender asks the community to introduce them to an entry, with a short message and the fields of their own card they want to share; the community checks the request and pushes [`vtc/directory/introduction`](../../introduction/0.1/spec.md) to the other member.

The recipient may ignore it, or answer the pairwise DID **directly** — with a pairwise DID of their own, through the existing peer relationship flow — so the community never learns whether the two met.

**The answer is always `accepted`.** The sender cannot tell delivery from refusal, so introductions cannot be used to probe who is a member or who has hidden themselves.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** mints `replyTo` for this introduction alone; it **MUST NOT** be the sender's membership DID, or a DID used for any other relationship.

A conforming **consumer** (the community) **MUST**:

1. Refuse with `notMember` only when the **sender** is not an active member, and with `rateLimited` only over the sender's own request count. Neither depends on the recipient.
2. Deliver the introduction only when all of these hold: the recipient is an active member; the sender may find the recipient's entry under the recipient's discoverability; the recipient's `introductions` setting admits the sender (`anyone`, a shared community group for `groups`, an attributed in-force verified relationship for `connections`, nobody for `off`); and neither the per-sender nor the per-sender-and-recipient limit is exceeded.
3. Fill the shared fields' values from the sender's own card, dropping any field whose visibility does not cover the recipient. It **MUST NOT** deliver a value the sender supplied.
4. Answer `status: "accepted"` in **every** other case — delivered, refused for any reason in item 2, recipient unknown — with the same response and no timing difference a caller can measure. It **MUST NOT** tell the sender later.
5. Deliver to the recipient's membership DID over the community's preferred transport, and keep the introduction, and the record that the sender asked, for no more than 30 days, for abuse handling only.
6. Audit the request by reference: sender, recipient entry and time, never the message.

## Authorization

**Authority: standing in the community, bounded by the recipient's own setting.** Any active member may ask; whether the introduction is delivered is decided by the recipient's `introductions` and discoverability choices, which the community enforces silently. The proof identifies the sender for the rate limit, the audit and the delivered introduction ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). Nothing in this task authorises the sender to learn anything about the recipient.

## Definitions

- **Pairwise DID** — a DID used for one relationship only, so that the two parties' other relationships cannot be linked through it.

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Ask to be introduced

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000026",
  "type": "https://trusttasks.org/spec/vtc/directory/introduce/0.1#request",
  "issuer": "did:example:carol",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000025",
  "payload": {
    "entryId": "dir_H3n8W2qYc6Vd1MsK",
    "replyTo": "did:peer:2.Ez6LSexamplePairwiseKey",
    "message": "We met at the Sydney meetup — would like to stay in touch.",
    "share": [
      "displayName",
      "org.example.chapter"
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:carol#key-1",
    "created": "2026-10-18T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/directory/introduce/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Ask to be introduced — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000027",
  "type": "https://trusttasks.org/spec/vtc/directory/introduce/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:carol",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000025",
  "payload": {
    "status": "accepted"
  }
}
```

## Security & Privacy

### Data carried

A message of at most 280 characters, a pairwise DID, and the sender's choice of their own fields. The message **MUST NOT** carry the sender's membership DID or any other correlating identifier the sender did not intend to share; the community delivers it unread.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. The `replyTo` party's identifier scope is pairwise: the DID is minted for this relationship alone, so the two members' later exchange cannot be linked to any other relationship of either, or to their memberships, by anyone but themselves. The community learns that the sender asked to meet the recipient. It does not learn whether they met: the reply goes directly to the pairwise DID.

### Retention

The introduction and the record of the request are kept at most 30 days, for abuse handling only, then deleted.

### Consent/purpose

To let members reach each other without exposing their DIDs. The community uses the request only to deliver it and to handle abuse; it **MUST NOT** use introductions to build a picture of who wants to meet whom.
