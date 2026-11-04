---
slug: vtn/listings/show
version: "0.1"
title: "VTN Listings — Show"
summary: "An administrator reads one entry the network holds, in the administrators' view, whatever its state."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, curation, administration, listing]
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: network
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The proof is the administrator's authentication: the network answers only a signer in its administrator list or a key one of them delegated, and the admin view includes entries and detail the published set does not show.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read with no side effects; a replay discloses only what the administrator was already entitled to see. A network can still bound how long a signed request is honoured.
sideEffects:
  level: none
  rationale: >-
    Reads one entry; changes nothing.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Out: one entry with its credentials and history — an organisation's public identifier. In: that organisation's DID.
retention:
  class: transient
  rationale: >-
    Nothing in the request is worth keeping; the entry is the network's own record.
errorCodes: []
related:
  - vtn/listings/list
  - vtn/listings/update
  - vtn/entries/show
  - vtn/audit/list
---

## Abstract

**VTN Listings — Show** is the read-one half of [`vtn/listings/list`](../../list/0.1/spec.md): an administrator of a Verifiable Trust Network reads one entry by the listed party's DID, in the administrators' view — its listing, offer, grant, the party's acknowledgement or repudiation, and its full history — whatever its state, `removed` and `departed` included. The published view of the same entry, for anyone the directory admits, is [`vtn/entries/show`](../../../entries/show/0.1/spec.md).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **administrator** (`issuer`):

1. **MUST** sign the request as an administrator, or with a key an administrator delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md).
2. **MUST NOT** treat the entry as a member unless its state is `acknowledged` (VTI-VTN-013).

A conforming **network** (`recipient`):

1. **MUST** refuse with `permissionDenied` unless the signer is an administrator — in the administrator list of its configuration, read at execution time — or a key an administrator delegated and has not revoked. It **MUST** decide this before looking up `did`, so a non-administrator learns nothing about which DIDs it holds.
2. **MUST** refuse with `notFound` when it holds no entry for `did`.
3. **MUST** otherwise return the entry in full ([`Entry`](../../../_shared/0.1/entry.schema.json)), in whatever state it is.

## Authorization

The authority this task presupposes is **administrator standing**: being named in the network's administrator list when the request is executed, or holding a signing key such an administrator delegated through `auth/signing-key/enroll` and has not revoked. The proof establishes who signed; the match against the administrator list is what entitles the requester (network item 1). Directory visibility does not apply: the admin view is never public.

## Definitions

- **`did`** — the listed party whose entry to read.
- **`entry`** — the administrators' view of the entry ([`Entry`](../../../_shared/0.1/entry.schema.json)).

## Request

An administrator (`issuer`) asks the network (`recipient`) for one entry. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Read a member's entry

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000561",
  "type": "https://trusttasks.org/spec/vtn/listings/show/0.1#request",
  "issuer": "did:example:admin",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T10:00:00Z",
  "payload": {
    "did": "did:example:community"
  }
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### An acknowledged entry

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000562",
  "type": "https://trusttasks.org/spec/vtn/listings/show/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:admin",
  "issuedAt": "2026-11-03T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000561",
  "payload": {
    "entry": {
      "did": "did:example:community",
      "kind": "community",
      "label": "St Vincent's Hospital",
      "profile": { "url": "https://stvincents.example", "tags": ["hospital"] },
      "state": "acknowledged",
      "createdAt": "2026-10-20T09:00:00Z",
      "updatedAt": "2026-10-21T14:30:00Z",
      "listing": {
        "credential": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0011" },
        "statusIndex": 1
      },
      "grant": {
        "credential": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0012" },
        "statusIndex": 2
      },
      "ack": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0021" },
      "history": [
        { "changedAt": "2026-10-20T09:00:00Z", "to": "listed" },
        { "changedAt": "2026-10-21T14:30:00Z", "from": "listed", "to": "acknowledged" }
      ]
    }
  }
}
```

The credentials are abbreviated to their `id`; the network returns them in full.

## Security & Privacy

### Data carried

The response carries one organisation's public identifier, the profile administrators gave it, the credentials between it and the network, and its history, including any reasons administrators recorded. The request carries a DID. Administrators **SHOULD NOT** record personal data in profiles or reasons.

### Correlation

The network declares `identifierScope: public` because it is an organisation whose DID is how readers find and verify it, and the listed party's DID is public for the same reason. The administrator declares `identifierScope: any`: the network recognises its administrators by its own configuration, so an administrator may use an identifier it uses for nothing else.

### Retention

Nothing in the request is retained beyond the network's ordinary request log.

### Consent/purpose

The admin view exists so the network's administrators can curate the set they are accountable for. A departed or removed party's record is kept as evidence of what was published and when, and is shown to administrators only.
