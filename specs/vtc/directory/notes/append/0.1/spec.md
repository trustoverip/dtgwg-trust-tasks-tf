---
slug: vtc/directory/notes/append
version: '0.1'
title: VTC Directory — Notes — Append
summary: An administrator adds a note about a member to the community's administrative notes, or redacts an earlier note with a later one. Notes are never edited.
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
  rationale: A note is a statement by an administrator about a member, and must stay attributable to its author after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed append would duplicate a note.
sideEffects:
  level: mutating
  rationale: Appends a note. A note is withdrawn only by a later redacting note; nothing is edited.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: Carries free text about a member, which may be sensitive. Returns the stored note.
retention:
  class: durable
  rationale: Notes are kept until the member's departure disposition purges them or the retention period ends.
errorCodes:
- code: vtc/directory/notes/append:notFound
  meaning: No member has this entry.
  retryable: false
- code: vtc/directory/notes/append:noteNotFound
  meaning: '`redacts` names no note about this entry.'
  retryable: false
- code: vtc/directory/notes/append:alreadyRedacted
  meaning: The note named by `redacts` is already redacted.
  retryable: false
related:
- vtc/directory/notes/list
- vtc/directory/notes/export
---

## Abstract

The **VTC Directory — Notes — Append** Trust Task adds an entry to a member's administrative notes: free text up to 2000 characters, optionally with a follow-up date. Notes are append-only. A mistaken or outdated note is withdrawn by appending a note that `redacts` it, so the history of what was recorded, and that it was withdrawn, stays intact.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller without `vtc.directory.notes` with `permissionDenied`.
2. Refuse an unknown entry with `notFound`, a `redacts` naming no note about the entry with `noteNotFound`, and one naming a redacted note with `alreadyRedacted`.
3. Store the note with the signer as `author`; on a redaction, mark the redacted note `redactedBy` and withhold its text from every read but [`vtc/directory/notes/export`](../../export/0.1/spec.md).
4. Store notes apart from cards, so that erasure and export handle them on their own, and never place one in a projection.
5. Audit the append by reference — author, entry, note identifier — never the text.

## Authorization

**Authority: `vtc.directory.notes`** at the community, read from the signer's access-control entry at execution time. The proof establishes who wrote the note, which is recorded as its author; it is not the authorization ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `AdminNote`, `NoteId` — see [`admin-note.schema.json`](../../../_shared/0.1/admin-note.schema.json).

## Request

The notes administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Add a follow-up note

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000002e",
  "type": "https://trusttasks.org/spec/vtc/directory/notes/append/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000002d",
  "payload": {
    "entryId": "dir_Q7m2K9xVb4Tz8LpR",
    "text": "Asked to be put in touch with the Sydney organisers.",
    "followUp": "2026-10-24"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/directory/notes/append/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Add a follow-up note — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000002f",
  "type": "https://trusttasks.org/spec/vtc/directory/notes/append/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000002d",
  "payload": {
    "note": {
      "noteId": "note_B8k2N4xRb3Qz7TpLm",
      "entryId": "dir_Q7m2K9xVb4Tz8LpR",
      "author": "did:example:administrator",
      "createdAt": "2026-10-18T09:00:01Z",
      "text": "Asked to be put in touch with the Sydney organisers.",
      "followUp": "2026-10-24"
    }
  }
}
```

## Security & Privacy

### Data carried

Free text about a member. An author **SHOULD** record only what the community needs, **MUST NOT** record secrets, and **SHOULD** avoid special-category data unless the community's purpose requires it.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. The member is named by `entryId`; the note's author by DID, visible only to other holders of `vtc.directory.notes`.

### Retention

**Durable** until the member's departure disposition: purged with a purge, kept for the community's retention period otherwise. A redaction withholds text from reads but not from a subject-access export, because the member is entitled to know what was recorded.

### Consent/purpose

Administrative continuity. That the community keeps notes is part of its published directory disclosure statement, so members know before they join.
