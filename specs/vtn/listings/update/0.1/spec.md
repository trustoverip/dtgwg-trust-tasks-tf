---
slug: vtn/listings/update
version: "0.1"
title: "VTN Listings — Update"
summary: "An administrator edits the label and directory profile of an entry; no credential is reissued and the entry's state does not change."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, curation, directory, profile]
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
    The proof is the administrator's authentication: the profile is what the network publishes about a party, so an unattributed edit would let anyone rewrite how the network describes its members.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed edit would silently restore an old description, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Changes the members of the entry's label and directory profile that the request names, and records an audit event. Reversible by another update. Nothing signed changes.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: an entry's DID and the profile an administrator writes for it. Out: the entry. Public identifiers and descriptions of organisations only; no personal data.
retention:
  class: durable
  rationale: >-
    The profile is published for as long as the entry is; the audit trail records who changed it and when.
errorCodes:
  - code: vtn/listings/update:invalidTransition
    meaning: "The entry is `removed` or `departed`, so it is no longer published and has no profile to edit. `details` names the state and the event."
    retryable: false
    detailsSchema:
      type: object
      required: [from, event]
      properties:
        from:
          type: string
          enum: [listed, acknowledged, repudiated, suspended, removed, departed]
        event:
          type: string
related:
  - vtn/listings/create
  - vtn/listings/show
  - vtn/directory/search
  - auth/signing-key/enroll
---

## Abstract

A network describes each party it lists: a label and a profile (a description, a public address and tags) that the directory shows and searches. **VTN Listings — Update** lets an administrator change that description. The profile is the network's own description of the party. It is in no credential, so no credential is reissued, and the entry's state, history and credentials are unchanged.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **administrator** (`issuer`):

1. **MUST** send only the members it changes. A member left out is kept; to clear `description` or `url` send an empty string, and to clear `tags` send an empty array.

A conforming **network** (`recipient`):

1. **MUST** refuse with `permissionDenied` unless `issuer` is one of its configured administrators, or a key delegated by one through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md), read from its configuration when it executes the request.
2. **MUST** refuse with `notFound` when it has no entry for `did`.
3. **MUST** refuse with `vtn/listings/update:invalidTransition` when the entry is `removed` or `departed`.
4. **MUST** replace each of `label`, `description`, `url` and `tags` that the request carries, **MUST** keep each one it does not, and **MUST** treat an empty string (or, for `tags`, an empty array) as clearing that member.
5. **MUST NOT** reissue, re-sign or revoke any credential, and **MUST NOT** change the entry's state or history. It **MUST** set `updatedAt` and record an `update` event in its audit trail naming `issuer` as the actor.
6. **MUST** answer with the entry. A request whose profile matches the entry's succeeds the same way: this is a setting operation ([SPEC §7.2](/SPEC.md#72-consumer-requirements)).

## Authorization

The authority this task presupposes is **administrator standing on the network**. The `issuer` is one of the administrators named in the network's configuration, or a key that one of them delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md). The network reads that standing when it executes the request (network item 1), so an administrator removed from the configuration, or a delegation revoked, loses the authority at once. The proof establishes who signed and that the document is unaltered ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The match between the signer and the configured administrators is what entitles it. The listed party has no say over the profile, because it is the network's description and asserts nothing in the party's name. A consumer **MAY** require an operation-bound step-up; that is its own policy.

## Definitions

- **`did`**: the entry to edit.
- **`label`**: the name the directory shows. Kept when absent; cleared by an empty string.
- **`description`**, **`url`**, **`tags`**: members of the entry's profile ([`EntryProfile`](../../../_shared/0.1/entry.schema.json)). Each one absent is kept; an empty string or empty array clears it.
- **`entry`**: the administrator's view of the entry ([`Entry`](../../../_shared/0.1/entry.schema.json)).

## Request

An administrator (`issuer`) sends the new profile to the network (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### An administrator rewrites a community's profile

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000c11",
  "type": "https://trusttasks.org/spec/vtn/listings/update/0.1#request",
  "issuer": "did:example:admin",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T11:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000c1f",
  "payload": {
    "did": "did:example:community",
    "label": "St Vincent's Health",
    "description": "A teaching hospital network in Sydney.",
    "url": "https://community.example",
    "tags": ["hospital", "nsw", "teaching"]
  }
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`: the entry with its new label and profile. A refusal is a `trust-task-error`.

### One profile member changes; nothing else does

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000c12",
  "type": "https://trusttasks.org/spec/vtn/listings/update/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:admin",
  "issuedAt": "2026-11-03T11:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000c1f",
  "payload": {
    "entry": {
      "did": "did:example:community",
      "kind": "community",
      "label": "St Vincent's Health",
      "profile": {
        "description": "A teaching hospital network in Sydney.",
        "url": "https://community.example",
        "tags": ["hospital", "nsw", "teaching"]
      },
      "state": "acknowledged",
      "createdAt": "2026-11-03T10:00:01Z",
      "updatedAt": "2026-11-03T11:00:01Z",
      "listing": {
        "credential": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0011" },
        "statusIndex": 41
      },
      "grant": {
        "credential": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0012" },
        "statusIndex": 42
      },
      "ack": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0021" },
      "history": [
        { "changedAt": "2026-11-03T10:00:01Z", "to": "listed" },
        { "changedAt": "2026-11-03T10:30:00Z", "from": "listed", "to": "acknowledged" }
      ]
    }
  }
}
```

The credentials are abbreviated to their `id`; the network returns them in full.

## Security & Privacy

### Data carried

The request carries an organisation's DID and the profile the directory will publish for it; the response carries the entry. None of it is personal data. The profile is published, so an administrator **MUST NOT** put anything in `label`, `description`, `url`, `tags` or `ext` that the network would not publish, such as a person's contact details or internal notes.

### Correlation

The profile is published by design, so the edit is visible to anyone who may read the directory, and the change of description can be seen by comparing reads over time. Both parties declare `identifierScope: public`. The entry is named by the public DID that the listing credential and verifiers use, and the network by the DID it publishes under, so a pairwise identifier could not name the entry. The administrator's DID is recorded in the audit trail as the actor.

### Retention

The network keeps the current profile for as long as the entry exists. The audit trail keeps the fact and time of each edit and who made it; the network need not keep earlier profiles.

### Consent/purpose

The profile is collected to describe the party in the directory and is used for nothing else. It is the network's description, not the party's statement, which is why the party's agreement is not part of this task. A party that objects to how it is described has the same remedy as for the listing itself.
