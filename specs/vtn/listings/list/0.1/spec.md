---
slug: vtn/listings/list
version: "0.1"
title: "VTN Listings — List"
summary: "An administrator lists every entry the network holds, in the administrators' view: its credentials, offer and full lifecycle history, removed and departed entries included."
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
    The proof is the administrator's authentication: the network answers only a signer in its administrator list or a key one of them delegated, and the admin view includes entries the published set does not show.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read with no side effects; a replay discloses only what the administrator was already entitled to see. A network can still bound how long a signed request is honoured.
sideEffects:
  level: none
  rationale: >-
    Reads the network's entries; changes nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Out: every entry with its credentials and history — organisations' public identifiers. In: at most a state filter.
retention:
  class: transient
  rationale: >-
    Nothing in the request is worth keeping; the entries are the network's own records.
errorCodes: []
related:
  - vtn/listings/show
  - vtn/listings/create
  - vtn/audit/list
  - vtn/directory/search
---

## Abstract

A Verifiable Trust Network lists communities and networks on its own authority ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-VTN-010), and its administrators need to see the whole of what it holds — not only the published set, but the offers not yet acknowledged, the full lifecycle history of each entry, and the entries that were removed or departed. **VTN Listings — List** returns that administrators' view of every entry, optionally filtered by state.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **administrator** (`issuer`):

1. **MUST** sign the request as an administrator, or with a key an administrator delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md).
2. **MUST NOT** treat an entry as a member unless its state is `acknowledged` (VTI-VTN-013); an entry is a listing, and a listing establishes nothing on its own (VTI-VTN-020).

A conforming **network** (`recipient`):

1. **MUST** refuse with `permissionDenied` unless the signer is an administrator — in the administrator list of its configuration, read at execution time — or a key an administrator delegated and has not revoked.
2. **MUST** return every entry it holds, in every state, `removed` and `departed` included; with `state`, only entries in that state.
3. **MUST** return each entry in full ([`Entry`](../../../_shared/0.1/entry.schema.json)): its listing, its offer while it has one, its grant and the party's acknowledgement or repudiation where they exist, and its history oldest first.

## Authorization

The authority this task presupposes is **administrator standing**: being named in the network's administrator list when the request is executed, or holding a signing key such an administrator delegated through `auth/signing-key/enroll` and has not revoked. The proof establishes who signed; the match against the administrator list, read from the network's configuration at execution time, is what entitles the requester (network item 1). Directory visibility does not apply: the admin view is never public, whatever the published set's visibility.

## Definitions

- **`state`** — when given, only entries in this state. Absent means every state.
- **`entries`** — the administrators' view of each entry ([`Entry`](../../../_shared/0.1/entry.schema.json)), which differs from the published view ([`IndexEntry`](../../../_shared/0.1/entry.schema.json)) in carrying the offer, the status-list index of each credential the network issued, timestamps and the full history.

This task is the read-many half of a split pair with [`vtn/listings/show`](../../show/0.1/spec.md), which reads one entry by DID and answers `notFound` for an absent one.

## Request

An administrator (`issuer`) asks the network (`recipient`) for its entries. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### The entries still waiting for acknowledgement

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000551",
  "type": "https://trusttasks.org/spec/vtn/listings/list/0.1#request",
  "issuer": "did:example:admin",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T10:00:00Z",
  "payload": {
    "state": "listed"
  }
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### One listed entry

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000552",
  "type": "https://trusttasks.org/spec/vtn/listings/list/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:admin",
  "issuedAt": "2026-11-03T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000551",
  "payload": {
    "entries": [
      {
        "did": "did:example:clinic",
        "kind": "community",
        "label": "Eastside Clinic",
        "profile": { "tags": ["clinic"] },
        "state": "listed",
        "createdAt": "2026-11-02T09:00:00Z",
        "updatedAt": "2026-11-02T09:00:00Z",
        "listing": {
          "credential": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0041" },
          "statusIndex": 7
        },
        "offer": {
          "credential": { "id": "urn:uuid:00000000-0000-4000-8000-0000000c0042" },
          "statusIndex": 8
        },
        "history": [
          { "changedAt": "2026-11-02T09:00:00Z", "to": "listed" }
        ]
      }
    ]
  }
}
```

The credentials are abbreviated to their `id`; the network returns them in full.

## Security & Privacy

### Data carried

The response carries organisations' public identifiers, the profiles administrators gave them, the credentials between them and the network, and their history, including the reasons administrators recorded. The request carries at most a state. Administrators **SHOULD NOT** record personal data in profiles or reasons: entries describe organisations.

### Correlation

The network declares `identifierScope: public` because it is an organisation whose DID is how readers find and verify it. The administrator declares `identifierScope: any`: the network recognises its administrators by its own configuration, so an administrator may use an identifier it uses for nothing else. The response reveals no more about any party than the network already holds of it.

### Retention

Nothing in the request is retained beyond the network's ordinary request log.

### Consent/purpose

The admin view exists so the network's administrators can curate the set they are accountable for. It includes parties that have left; their records are kept as the evidence of what was published and when, and are shown to administrators only.
