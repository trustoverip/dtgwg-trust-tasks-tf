---
slug: persona/contact/lists/show
version: "1.0"
title: Persona Contact Lists — Show
summary: Read one of the holder's own contact lists by its identifier, with a definite notFound when the context does not hold it.
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords: [persona, contact, list, address-book]
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
  rationale: The response returns one of the holder's own groupings of people, which is their private judgement of others, so the agent must attribute the request to a key.
issuedAtRequirement:
  requirement: OPTIONAL
  rationale: A read has no durable effect and no ordering hazard.
sideEffects:
  level: none
  rationale: Reads only.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: The response returns one list's name, colour, order and size to a client of the holder's own. It names no contact.
retention:
  class: transient
  rationale: A point-in-time view of a record the agent keeps.
errorCodes:
  - code: persona/contact/lists/show:notFound
    meaning: "`listId` names no list in this context. Answered the same whether it was never assigned, was deleted, or belongs to another context."
    retryable: false
related:
  - persona/contact/lists/list
  - persona/contact/lists/put
  - persona/contact/get
---

## Abstract

**Persona Contact Lists — Show** reads one of the holder's contact lists. It is
the single-record half of the split pair with
[`persona/contact/lists/list`](../../list/1.0/spec.md): a client that holds a
`listId` — from a contact's `lists`, or a filter it saved — learns definitely
whether that list still exists, and its current name, colour and size.

A contact names its lists by `listId`, never by name, so renaming a list touches
no contact; this is how a client resolves the identifier to what it shows.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/contact/lists/show/1.0`, populate
`contextId` and `listId`, and include a `proof` per
[SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST** confine the caller to its own context and
**MUST** answer `notFound` for a list it holds in another context exactly as for
one it never assigned.

## Authorization

**Context-scoped**, confined to the caller's own context. The `proof` identifies
the caller so the agent can look up which contexts it serves
([SPEC.md §7.2 item 10](/SPEC.md#72-consumer-requirements)); it is not itself the
entitlement.

## Definitions

- `ContactList`, `ListId` — see [`persona/_shared/0.1/contact-annotations.schema.json`](../../../../_shared/0.1/contact-annotations.schema.json).

## Request

```json
{
  "id": "urn:uuid:4c5d6e7f-8a9b-4c0d-9e1f-2a3b4c5d6e01",
  "type": "https://trusttasks.org/spec/persona/contact/lists/show/1.0",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:agent.alice.example",
  "issuedAt": "2026-10-18T09:31:00Z",
  "payload": { "contextId": "ctx-personal", "listId": "01JA8Z5M2N3P4Q5R6S7T8V9W0X" }
}
```

## Response

The list, in a payload that validates against the sub-schema reachable via
`$anchor: "response"`. A refusal is a `trust-task-error`.

```json
{
  "id": "urn:uuid:4c5d6e7f-8a9b-4c0d-9e1f-2a3b4c5d6e02",
  "type": "https://trusttasks.org/spec/persona/contact/lists/show/1.0#response",
  "threadId": "urn:uuid:4c5d6e7f-8a9b-4c0d-9e1f-2a3b4c5d6e01",
  "issuer": "did:web:agent.alice.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-18T09:31:01Z",
  "payload": {
    "list": {
      "listId": "01JA8Z5M2N3P4Q5R6S7T8V9W0X",
      "name": "Mentors",
      "colour": "#1f8a70",
      "order": 10,
      "contactCount": 3,
      "createdAt": "2026-10-18T08:00:01Z",
      "updatedAt": "2026-10-18T08:00:01Z"
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries a context and a list identifier. The response carries one
list's name, colour, order and size — the holder's own words about a group of
people, and no contact.

**A list never leaves the holder's agent** except in this response, to a client
the holder authorised for the context, and a producer **MUST NOT** forward it.

### Correlation

The response names no third party. **Being on a holder's list MUST NOT be
inferable by the person listed or by any community**; this read is invisible to
both. A uniform `notFound` keeps the task from saying whether a list identifier
exists in some other context.

### Retention

A point-in-time view; it **SHOULD NOT** be retained beyond the rendering it
serves.

### Consent/purpose

The purpose is to show the holder one of their own lists. Nothing here is
collected from anyone but the holder. What gate a producer applies is the
consumer's policy and is deliberately not specified here.
