---
slug: vtc/directory/notes/list
version: '0.2'
title: VTC Directory — Notes — List
summary: An administrator holding the notes capability reads the community's administrative notes about one member, newest first.
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
  rationale: Every read of administrative notes is audited against its reader; the reader must be attributable after the transport has closed.
sideEffects:
  level: none
  rationale: Reads notes; persists nothing but the audit record of the read.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses what administrators wrote about a member — possibly sensitive personal data.
retention:
  class: transient
  rationale: A read; the audit record of it is kept, by reference.
errorCodes:
- code: vtc/directory/notes/list:notFound
  meaning: No member, past or present, has this entry, or its notes have been purged.
  retryable: false
- code: vtc/directory/notes/list:cursorInvalid
  meaning: '`cursor` was not issued for this caller and entry.'
  retryable: false
related:
- vtc/directory/notes/append
- vtc/directory/notes/export
---

## Abstract

The **VTC Directory — Notes — List** Trust Task reads the administrative notes a community keeps about one member: what administrators recorded, when and by whom, and which notes later notes redacted.

Notes are a separate record from the card. They are never on a card, never in any projection, and never read by a member — including the member they are about, whose route to them is a subject-access request answered by [`vtc/directory/notes/export`](../../export/0.1/spec.md).

**Why there is no `show`.** Notes are only ever read in the context of one member, a page at a time; no caller addresses a single note except to redact it, which [`vtc/directory/notes/append`](../../append/0.1/spec.md) does by identifier. A per-note read would add a second path to sensitive text with nothing gained.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC member-directory design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-member-directory.md`).

## Changes from 0.1

The response's **`nextCursor`** changes type from `["string", "null"]` to **`"string"`** (still at most 1024 characters). It is present when more items remain and **absent** on the last page; a `null` value now fails validation. A producer that detected the last page by `nextCursor === null` **MUST** detect it by absence instead.

This is the registry-wide pagination convention: a continuation token is an optional string whose absence marks the last page, so there is one spelling of "no more pages" instead of two.

The request's `cursor` and `limit` already followed the convention and are unchanged on the wire — same types and bounds; only their schema descriptions are reworded to the registry's paging text. Nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `0.1` remains published.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller without `vtc.directory.notes` with `permissionDenied`.
2. Return the notes for the entry newest first, with the text of a redacted note withheld.
3. Bind `nextCursor` to the caller and the entry.
4. Audit the read by reference: the reader, the entry and the time — never the text.
5. **MUST NOT** return notes through any other task, projection or policy outcome.

## Authorization

**Authority: `vtc.directory.notes`** at the community, read from the signer's access-control entry at execution time. It is separate from `vtc.directory.admin` so a community can let moderators keep notes without editing the directory schema, and the built-in administrative roles carry it. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `AdminNote` — see [`admin-note.schema.json`](../../../_shared/0.1/admin-note.schema.json).

## Request

The notes administrator (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Read a member's notes

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000002b",
  "type": "https://trusttasks.org/spec/vtc/directory/notes/list/0.2#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-18T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000002a",
  "payload": {
    "entryId": "dir_Q7m2K9xVb4Tz8LpR"
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/directory/notes/list/0.2#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Read a member's notes — answered

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000002c",
  "type": "https://trusttasks.org/spec/vtc/directory/notes/list/0.2#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-18T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-00000000002a",
  "payload": {
    "items": [
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

Administrative notes about one member: free text written by administrators, which may be sensitive. A note **MUST NOT** carry secrets or a third party's personal data beyond what the note needs.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members, its administrators and the registry already know it by, and a pairwise identifier per caller would make the directory's owner differ by who asks. The member is named by their community-scoped `entryId`.

### Retention

Nothing is kept from the read but its audit record, by reference. The notes themselves follow the member's departure disposition: purged with a purge, kept for the community's retention period otherwise.

### Consent/purpose

Notes exist so administrators can do their work — follow-ups, context, moderation history. The community discloses in its published directory statement that it keeps them. They **MUST NOT** be used to decide anything about the member that a member-visible record should decide.
