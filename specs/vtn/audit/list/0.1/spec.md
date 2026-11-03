---
slug: vtn/audit/list
version: "0.1"
title: "VTN Audit — List"
summary: "An administrator reads the network's audit trail: every change to its entries, when, to what state, by whom and why."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, curation, administration, audit]
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
    The proof is the administrator's authentication: the network answers only a signer in its administrator list or a key one of them delegated, and the trail names the administrators who acted.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read with no side effects; a replay discloses only what the administrator was already entitled to see. A network can still bound how long a signed request is honoured.
sideEffects:
  level: none
  rationale: >-
    Reads the audit trail; changes nothing.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Out: audit events — the DIDs of the entries changed and of the administrators or parties who changed them, with the reasons recorded. In: at most an entry's DID and a count.
retention:
  class: transient
  rationale: >-
    Nothing in the request is worth keeping; the trail is the network's own durable record.
errorCodes: []
related:
  - vtn/listings/list
  - vtn/listings/show
---

## Abstract

Every change to a Verifiable Trust Network's entries — a listing, an acknowledgement, a repudiation or departure, a suspension, reinstatement or removal, a profile update — is recorded in the network's audit trail with who made it ([VTI specification, *The Verifiable Trust Network*](https://trustoverip.github.io/dtgwg-vti-spec/), VTI-VTN-010). **VTN Audit — List** lets an administrator read that trail, for the whole network or one entry.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **administrator** (`issuer`):

1. **MUST** sign the request as an administrator, or with a key an administrator delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md).

A conforming **network** (`recipient`):

1. **MUST** refuse with `permissionDenied` unless the signer is an administrator — in the administrator list of its configuration, read at execution time — or a key an administrator delegated and has not revoked.
2. **MUST** return the events it recorded, oldest first; with `did`, only events for that entry. A `did` it holds no events for yields an empty list, not `notFound`: the trail of an entry is a filter over the network's trail, not a record of its own.
3. **MUST**, with `limit`, return only the most recent `limit` matching events, still oldest first.
4. **MUST** record in each event the actor that caused it — the administrator who acted, whatever key they signed with, or the listed party for its own acknowledgement and repudiation — and **MUST NOT** alter or drop an event once recorded.

## Authorization

The authority this task presupposes is **administrator standing**: being named in the network's administrator list when the request is executed, or holding a signing key such an administrator delegated through `auth/signing-key/enroll` and has not revoked. The proof establishes who signed; the match against the administrator list is what entitles the requester (network item 1). Directory visibility does not apply: the trail is never public.

## Definitions

- **`did`** — when given, only events for this entry.
- **`limit`** — when given, the most recent this many events (1 to 1000). Absent means all.
- **`events`** — the matching events ([`AuditEvent`](../../../_shared/0.1/entry.schema.json)), oldest first: when, which entry, what happened, its state afterwards, who did it, and the reason given.

This family has no `show` task for a single event: an event has no identifier of its own and nothing refers to one, so it is only ever read in sequence; filtering by entry and bounding by `limit` is how a caller narrows it. In this version `limit` is a bound, not a page: there is no `cursor`, and a caller that wants an entry's full trail filters by `did`.

## Request

An administrator (`issuer`) asks the network (`recipient`) for its trail. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### The last two events for one entry

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000571",
  "type": "https://trusttasks.org/spec/vtn/audit/list/0.1#request",
  "issuer": "did:example:admin",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T10:00:00Z",
  "payload": {
    "did": "did:example:community",
    "limit": 2
  }
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`. A refusal is a `trust-task-error`.

### Acknowledged, then suspended

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000572",
  "type": "https://trusttasks.org/spec/vtn/audit/list/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:admin",
  "issuedAt": "2026-11-03T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000000571",
  "payload": {
    "events": [
      {
        "occurredAt": "2026-10-21T14:30:00Z",
        "did": "did:example:community",
        "event": "acknowledge",
        "to": "acknowledged",
        "actor": "did:example:community"
      },
      {
        "occurredAt": "2026-11-01T08:15:00Z",
        "did": "did:example:community",
        "event": "suspend",
        "to": "suspended",
        "actor": "did:example:admin",
        "reason": "Accreditation under review"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The response carries the DIDs of entries and of the actors who changed them — organisations, and the network's administrators — with the reasons recorded. The request carries at most a DID and a count. Administrators **SHOULD NOT** put personal data in reasons: they are kept for as long as the trail is.

### Correlation

The network declares `identifierScope: public` because it is an organisation whose DID is how readers find and verify it, and the listed parties' DIDs are public for the same reason. The administrator declares `identifierScope: any`: the network recognises its administrators by its own configuration, so an administrator may use an identifier it uses for nothing else. The trail does link each administrator's identifier to the actions they took; that accountability is its purpose, and it is shown to administrators only.

### Retention

Nothing in the request is retained beyond the network's ordinary request log. The trail itself is durable: it is the evidence of what the network published and when, and outlives the entries it describes.

### Consent/purpose

The trail exists so the network's administrators can account for the set they curate. It is used for that and nothing else, and never published.
