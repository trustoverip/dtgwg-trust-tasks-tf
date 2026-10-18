---
slug: persona/contact/lists/list
version: "1.0"
title: Persona Contact Lists — List
summary: Enumerate the holder's own contact lists in one context, in the order they chose, with how many contacts each holds.
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords: [persona, contact, list, favourites, address-book]
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
  rationale: The response returns the holder's own groupings of the people they know, which is their private judgement of others, so the agent must attribute the request to a key.
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
  rationale: The response returns list names, colours, order and sizes to a client of the holder's own. It names no contact.
retention:
  class: transient
  rationale: A point-in-time view; cursors live only as long as their enumeration.
errorCodes:
  - code: persona/contact/lists/list:cursorInvalid
    meaning: The supplied cursor is unrecognised or expired. A producer restarts the enumeration.
    retryable: false
related:
  - persona/contact/lists/show
  - persona/contact/lists/put
  - persona/contact/lists/delete
  - persona/contact/list
---

## Abstract

**Persona Contact Lists — List** returns the holder's contact lists in one
context, ordered by the holder's `order` and then by name, each with how many
contacts are on it. It is what a client renders as "My lists".

It returns list records, not their members. A list's contacts are read with
[`persona/contact/list`](../../../list/1.1/spec.md) 1.1 and its `listId` filter,
paged like any other listing; inlining them here would make every render of the
list picker a read of the holder's whole address book.

This task and [`persona/contact/lists/show`](../../show/1.0/spec.md) are the
registry's split pair for this collection: a list addressed by an identifier the
context does not hold is `notFound` on `show`, a definite answer, where this
listing can only return a page without it.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/contact/lists/list/1.0`, populate
`contextId`, and include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof). It
**MUST NOT** construct or parse a cursor, and **MUST NOT** infer exhaustion from a
short page.

A conforming **maintainer** **MUST** confine the caller to its own context;
**MUST** return lists ordered by `order` ascending, then `name`, stably across
the pages of one enumeration; **MUST** report each list's `contactCount` as of the
read; and **MUST NOT** return lists of any other context.

## Authorization

**Context-scoped**, confined to the caller's own context. The `proof` identifies
the caller so the agent can look up which contexts it serves
([SPEC.md §7.2 item 10](/SPEC.md#72-consumer-requirements)); it is not itself the
entitlement.

## Definitions

- `ContactList` — see [`persona/_shared/0.1/contact-annotations.schema.json`](../../../../_shared/0.1/contact-annotations.schema.json).

## Request

```json
{
  "id": "urn:uuid:3b4c5d6e-7f8a-4b9c-8d0e-1f2a3b4c5d01",
  "type": "https://trusttasks.org/spec/persona/contact/lists/list/1.0",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:agent.alice.example",
  "issuedAt": "2026-10-18T09:30:00Z",
  "payload": { "contextId": "ctx-personal", "limit": 100 }
}
```

## Response

The lists, in a payload that validates against the sub-schema reachable via
`$anchor: "response"`. A refusal is a `trust-task-error`.

```json
{
  "id": "urn:uuid:3b4c5d6e-7f8a-4b9c-8d0e-1f2a3b4c5d02",
  "type": "https://trusttasks.org/spec/persona/contact/lists/list/1.0#response",
  "threadId": "urn:uuid:3b4c5d6e-7f8a-4b9c-8d0e-1f2a3b4c5d01",
  "issuer": "did:web:agent.alice.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-18T09:30:01Z",
  "payload": {
    "lists": [
      {
        "listId": "01JA8Z5M2N3P4Q5R6S7T8V9W0X",
        "name": "Mentors",
        "colour": "#1f8a70",
        "order": 10,
        "contactCount": 3,
        "createdAt": "2026-10-18T08:00:01Z",
        "updatedAt": "2026-10-18T08:00:01Z"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries a context and pagination state. The response carries the
holder's list names, colours, order and sizes — the holder's own words about
groups of people, but no contact, no DID and no claim value.

**Lists never leave the holder's agent** except in this response, to a client the
holder authorised for the context, and a producer **MUST NOT** forward them to a
contact, a community or anyone else.

### Correlation

The response names no third party. **Being on a holder's list MUST NOT be
inferable by the person listed or by any community**, and nothing here is visible
to either: the read stays between the holder's client and the holder's agent.
`contactCount` describes the holder's own store. `listId` is meaningful inside one
context of one agent only, and links nothing across communities.

### Retention

A point-in-time view with no evidentiary value; it **SHOULD NOT** be retained
beyond the rendering it serves. Cursors live only as long as their enumeration.

### Consent/purpose

The purpose is to show the holder their own lists, so they can browse, filter and
manage them. Nothing here is collected from anyone but the holder, and nothing is
used for anything else. What gate a producer applies is the consumer's policy and
is deliberately not specified here.
