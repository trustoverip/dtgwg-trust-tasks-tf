---
slug: vtc/directory/notes/export
version: '0.1'
title: VTC Directory — Notes — Export
summary: An administrator exports every administrative note about one member, redacted ones included, to answer that member's subject-access request.
status: draft
targetFrameworkVersion: 0.6.0
category: governance
parties:
- role: notes administrator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: An export of everything recorded about a person must be attributable to the administrator who took it, after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: The export is the most sensitive read in the family; placing the document in time lets the community bind it to a fresh confirmation by the requester and refuse a replay.
sideEffects:
  level: none
  rationale: Reads notes; persists nothing but the audit record of the export.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses every note about a member as written, including redacted text.
maxDocumentBytes:
  response: 4194304
  rationale: Up to several thousand notes of 2000 characters each, plus envelope and proof headroom.
retention:
  class: transient
  rationale: A read; the export is handed to the subject and the community keeps only the audit record.
errorCodes:
- code: vtc/directory/notes/export:notFound
  meaning: No member, past or present, has this entry, or its notes have been purged.
  retryable: false
related:
- vtc/directory/notes/list
- vtc/directory/export
---

## Abstract

The **VTC Directory — Notes — Export** Trust Task returns every administrative note about one member exactly as written, redacted notes included. It is how a community answers a member's subject-access request for what its administrators recorded about them; members never read notes directly.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller without `vtc.directory.notes` with `permissionDenied`.
2. Return every note about the entry, oldest first, with redacted text included.
3. Audit the export at its highest severity, by reference: the exporter, the entry, the `reason` and the time.

A community **SHOULD** bind the export to a fresh confirmation by the requester bound to this one document — an operation-bound step-up — and record it with the audit row. Whether it requires one is the community's policy, which this specification does not declare.

## Authorization

**Authority: `vtc.directory.notes`** at the community, read from the signer's access-control entry at execution time, for the purpose of answering a subject-access request. The proof establishes who exported, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `AdminNote` — see [`admin-note.schema.json`](../../../_shared/0.1/admin-note.schema.json).

## Request

The notes administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Export for a subject-access request

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000031",
  "type": "https://trusttasks.org/spec/vtc/directory/notes/export/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000030",
  "payload": {
    "entryId": "dir_Q7m2K9xVb4Tz8LpR",
    "reason": "Subject-access request received 2026-10-17."
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/directory/notes/export/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Export for a subject-access request — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000032",
  "type": "https://trusttasks.org/spec/vtc/directory/notes/export/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000030",
  "payload": {
    "entryId": "dir_Q7m2K9xVb4Tz8LpR",
    "exportedAt": "2026-10-18T09:00:01Z",
    "notes": [
      {
        "noteId": "note_B8k2N4xRb3Qz7TpLm",
        "entryId": "dir_Q7m2K9xVb4Tz8LpR",
        "author": "did:example:administrator",
        "createdAt": "2026-10-10T10:00:00Z",
        "text": "Asked to be put in touch with the Sydney organisers.",
        "followUp": "2026-10-24"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Everything administrators recorded about one member. The exporter **MUST** hand it only to the member, or to whom the member directs, and **SHOULD** deliver it over an end-to-end protected channel.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. The member is named by `entryId`.

### Retention

The community keeps the audit record of the export; the export itself is not stored by the community.

### Consent/purpose

Answering the member's right of access. The export **MUST NOT** be used for any other purpose, including disclosure to anyone other than the member.
