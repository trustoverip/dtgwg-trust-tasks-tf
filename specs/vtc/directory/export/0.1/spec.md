---
slug: vtc/directory/export
version: '0.1'
title: VTC Directory — Export
summary: A directory administrator exports the whole member directory as JSContact cards, a page at a time. Members cannot export.
status: draft
targetFrameworkVersion: 0.6.0
category: governance
parties:
- role: directory administrator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: A bulk export of members' personal data must be attributable to the administrator who took it, after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Placing the document in time lets the community bind the export to a fresh confirmation by the requester and refuse a replay.
sideEffects:
  level: none
  rationale: Reads the directory; persists nothing but the audit record of the export.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses every member's directory card as JSContact — bulk personal data.
maxDocumentBytes:
  response: 8388608
  rationale: 500 cards of up to 16 KiB each, plus envelope and proof headroom.
retention:
  class: transient
  rationale: A read; the community keeps only the audit record.
errorCodes:
- code: vtc/directory/export:cursorInvalid
  meaning: '`cursor` was not issued for this caller and this export.'
  retryable: false
related:
- vtc/directory/entries/list
- vtc/directory/notes/export
---

## Abstract

The **VTC Directory — Export** Trust Task exports the community's member directory as JSContact cards (RFC 9553), so it can be moved to any address book or archived. It returns every active member's card as administrators see it, a page at a time.

**Bulk export is an administrator's act only.** A member can browse and search, within limits designed to stop enumeration; nothing lets a member take the directory away whole.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller without `vtc.directory.admin` with `permissionDenied`. No member capability reaches this task.
2. Return each active member's card as a JSContact Card, with `uid` `urn:vtc-directory:<entryId>`, mapping base fields as their definitions' `jsContactPath` says and other fields as vendor-specific properties. It **MUST NOT** include administrative notes.
3. Bind `nextCursor` to the caller and this export, and refuse it otherwise with `cursorInvalid`.
4. Audit the export at its highest severity, by reference: the exporter, the `reason`, the time and the number of cards.

A community **SHOULD** bind the export to a fresh confirmation by the requester bound to this one document — an operation-bound step-up. Whether it requires one is the community's policy, which this specification does not declare.

## Authorization

**Authority: `vtc.directory.admin`** at the community, read from the signer's access-control entry at execution time. The proof establishes who exported, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `JSContactCard` — see [`directory-card.schema.json`](../../_shared/0.1/directory-card.schema.json).

## Request

The directory administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Export the first page

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000034",
  "type": "https://trusttasks.org/spec/vtc/directory/export/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000033",
  "payload": {
    "format": "jscontact",
    "reason": "Annual archive.",
    "limit": 200
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:administrator#key-1",
    "created": "2026-10-18T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/directory/export/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Export the first page — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000035",
  "type": "https://trusttasks.org/spec/vtc/directory/export/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000033",
  "payload": {
    "items": [
      {
        "@type": "Card",
        "version": "1.0",
        "uid": "urn:vtc-directory:dir_Q7m2K9xVb4Tz8LpR",
        "name": {
          "full": "Carol"
        },
        "speakToAs": {
          "pronouns": {
            "p1": {
              "pronouns": "she/her"
            }
          }
        }
      }
    ],
    "exportedAt": "2026-10-18T09:00:01Z"
  }
}
```

## Security & Privacy

### Data carried

Every member's card, at every visibility, because administrators see every field. The exporter **MUST** protect the export at least as the community protects the directory.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. Cards are identified by `entryId`; a member's DID is in a card only as its derived `did` field, which administrators see.

### Retention

The community keeps the audit record; what the exporter keeps is governed by the community's own policy for exports.

### Consent/purpose

Archive, migration and continuity of the community's own directory. An export **MUST NOT** be used to disclose members' data beyond the audiences they chose.
