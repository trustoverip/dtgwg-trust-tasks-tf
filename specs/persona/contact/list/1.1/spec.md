---
slug: persona/contact/list
version: "1.1"
title: Persona Contact — List
summary: Enumerate a context's contacts as summaries with the holder's own stars, petnames, lists and directory links, narrowed by persona, star, list, community or what changed — with no claim values in the listing.
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords: [persona, contact, privacy, petname, star, address-book]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Application
    requirement: REQUIRED
    member: issuer
  - role: Agent
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The response enumerates the holder's counterparties and how the holder has marked them, which is relationship data about third parties and the holder's private judgement of them, so the agent must attribute the request to a key.
issuedAtRequirement:
  requirement: OPTIONAL
  rationale: A list has no durable effect and no ordering hazard.
sideEffects:
  level: none
  rationale: Reads only.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: The response returns identifiers and display names of the holder's counterparties and the holder's own annotations on them, to a client of the holder's own. No claim values, no notes, and nothing to the counterparties or to any community.
retention:
  class: transient
  rationale: A point-in-time view with no evidentiary value; cursors live only as long as their enumeration.
errorCodes:
  - code: persona/contact/list:cursorInvalid
    meaning: The supplied cursor is unrecognised, expired, or was minted for a different set of filters. A producer restarts the enumeration.
    retryable: false
  - code: persona/contact/list:listUnknown
    meaning: "`listId` names no list in this context. Distinct from an empty page on purpose: a list is the holder's own, and \"no such list\" is a definite answer a client should act on (it was deleted elsewhere) rather than render as an empty list."
    retryable: false
related:
  - persona/contact/get
  - persona/contact/put
  - persona/contact/lists/list
---

## Abstract

**Persona Contact — List** finds a contact. It returns summaries and no claim
values, because finding one contact does not require disclosing the details of
every contact.

`changedSince` answers *what changed while I was away* cheaply, and
`hasUnreviewedChange` lets a producer badge the contact whose payment address
moved. Omitting `knownByPersona` lists a context's contacts across every persona
in it, which a producer **SHOULD** offer deliberately rather than by default.

### What 1.1 adds

Each summary carries the holder's **annotations** that a client sorts, badges and
filters by — `starred` (always present), `petname`, `lists` and `directoryRef` —
as [`persona/contact/put`](../../put/1.1/spec.md) 1.1 writes them. The private
`notes` stay on [`persona/contact/get`](../../get/1.1/spec.md): a listing is
rendered in places a single contact's detail is not.

Three filters narrow by annotation: `starred`, `listId` (one of the holder's
[lists](../../lists/list/1.0/spec.md)) and `directoryVtcDid` (contacts linked to
one community's directory). The last is what the holder's client asks for when it
joins that community's member directory with the holder's address book — stars
first, petnames beside display names, "My lists" as filters — **locally**, on the
holder's own device, so that neither the agent nor the community sees the other
half.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues). Version 1.0 remains in force; 1.1 is a backwards-compatible MINOR increment ([SPEC.md §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/contact/list/1.1`, populate
`payload.contextId`, and include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).
It **MUST NOT** construct or parse a cursor, and **MUST NOT** infer exhaustion
from a short page.

A conforming **maintainer** **MUST** confine the caller to its own context,
**MUST** apply every filter given (as AND), **MUST** return summaries in a stable
order across the pages of one enumeration, **MUST** bind a cursor to the filters
it was minted for, and **MUST NOT** include claim values or `notes` in a summary.
It **MUST** answer `listUnknown` for a `listId` this context does not hold.

A conforming maintainer **MUST NOT** substitute a DID — or a petname — for an
absent `displayName`. A producer that renders the holder's petname where the
contact's own name would go **SHOULD** mark it as the holder's, so a holder never
mistakes their own label for what the contact calls themselves, and a contact's
display name can never pass for a petname the holder wrote.

## Authorization

**Context-scoped**, confined to the caller's own context. The entitlement is
being authorised for that context; the `proof` identifies the caller so the agent
can look that up ([SPEC.md §7.2 item 10](/SPEC.md#72-consumer-requirements)).

## Definitions

- `Petname`, `ListId`, `ListIds`, `DirectoryRef` — see [`persona/_shared/0.1/contact-annotations.schema.json`](../../../_shared/0.1/contact-annotations.schema.json).

## Request

See the payload schema; every member carries its own rationale there.

### The holder's contacts at one community, for a local join with its directory

```json
{
  "id": "urn:uuid:7c1d2e3f-4a5b-4c6d-8e7f-9a0b1c2d3e01",
  "type": "https://trusttasks.org/spec/persona/contact/list/1.1",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:agent.alice.example",
  "issuedAt": "2026-10-18T09:20:00Z",
  "payload": {
    "contextId": "ctx-personal",
    "directoryVtcDid": "did:web:climbing.example",
    "limit": 200
  }
}
```

## Response

Summaries, in a payload that validates against the sub-schema reachable via
`$anchor: "response"`. A refusal is a `trust-task-error`.

```json
{
  "id": "urn:uuid:7c1d2e3f-4a5b-4c6d-8e7f-9a0b1c2d3e02",
  "type": "https://trusttasks.org/spec/persona/contact/list/1.1#response",
  "threadId": "urn:uuid:7c1d2e3f-4a5b-4c6d-8e7f-9a0b1c2d3e01",
  "issuer": "did:web:agent.alice.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-18T09:20:01Z",
  "payload": {
    "contacts": [
      {
        "contactId": "01JA8Z3Q4R5S6T7V8W9X0Y1Z2A",
        "subjectDid": "did:peer:2.Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc",
        "knownByPersona": "did:peer:2.Ez6LSghwSE437wnDE1pt3X6hVDUQzSjsHzinpX3XFvMjRAm7y",
        "rev": 1,
        "displayName": "Sam Okafor",
        "claimCount": 1,
        "receivedAt": "2026-10-18T09:00:00Z",
        "starred": true,
        "petname": "Sam (belay partner)",
        "lists": ["01JA8Z5M2N3P4Q5R6S7T8V9W0X"],
        "directoryRef": { "vtcDid": "did:web:climbing.example", "entryId": "e_7Hq2mZp9Lx4v" }
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries a context, optional filters and pagination state. The
response carries **identifiers and display names of third parties** and the
holder's **annotations** on them — stars, petnames, list memberships and directory
links — but no claim values and no notes.

The annotations are the holder's private judgement of other people, and **never
leave the holder's agent** except in this response, to a client the holder
authorised for this context. A producer **MUST NOT** forward them: not to a
contact, not to a community, not into anything it sends on the holder's behalf.

### Correlation

The unfiltered listing shows which of the holder's personas know which
counterparties, side by side. That is why the task is confined to a single
context, and a maintainer **MUST NOT** offer a cross-context listing on this task.

**Being on a holder's list, or starred, MUST NOT be inferable by the person listed
or by any community.** These filters run inside the holder's agent and reveal
nothing outward. The risk is in what a client does next: a client that used a
`starred` or `listId` page to decide which directory entries to fetch would show
the community whom the holder favours. A client joining a community's directory
**SHOULD** read the directory as it would without annotations and apply them
locally, which is what `directoryVtcDid` exists to make cheap.

`directoryRef` links a pairwise relationship to a community-scoped entry. It is a
correlator held privately, returned only to the holder's own client.

### Retention

A point-in-time view with no evidentiary value; it **SHOULD NOT** be retained
beyond the rendering it serves. A client that caches it to join with a directory
**SHOULD** hold it in memory for that page's life only: a persisted copy of who
the holder stars is the social graph this design keeps away from every server.

### Consent/purpose

The purpose is navigation and triage: the holder finds a counterparty, sees which
relationships have changed, and sees them in the order and groupings they chose.
Using annotations to rank, recommend or profile the people annotated, or to tell
anyone else about them, is outside that purpose. What gate a producer applies
before showing the unfiltered, cross-persona view is the consumer's policy and is
deliberately not specified here.
