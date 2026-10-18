---
slug: persona/contact/lists/put
version: "1.0"
title: Persona Contact Lists — Put
summary: Create, rename, recolour or reorder one of the holder's own contact lists — kept by the holder's agent, never visible to the people on it or to any community.
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords: [persona, contact, list, favourites, address-book, close-friends]
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
  rationale: A list's name is the holder's own words about a group of people. A request whose origin depended on the transport would let a compromised channel rename "Family" to something else in every picker the holder trusts.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Mutating, so SPEC §7.3 item 17 sets REQUIRED as the floor. Details are replaced wholesale, so a replayed earlier request would put back a name or colour the holder has since changed.
sideEffects:
  level: mutating
  rationale: Creates a list, or replaces or removes its name, colour or order, in the agent's own store. Reversible by another request; nothing reaches any other party, and no contact changes.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: A list name is free text the holder wrote about a group of people ("Mentors", "Owes me money") — personal data about the holder and, by implication, about whoever they put on it. The response returns the list record to the client that wrote it.
retention:
  class: durable
  rationale: A list is worth having only if it is there next time. It persists until the holder deletes it.
errorCodes:
  - code: persona/contact/lists/put:notFound
    meaning: "`listId` names no list in this context. Answered the same whether it was never assigned or belongs to another context."
    retryable: false
  - code: persona/contact/lists/put:nameTaken
    meaning: Another list in this context already has this name, compared exactly. Two lists a holder cannot tell apart in a picker are a mistake waiting to put someone on the wrong one.
    retryable: false
  - code: persona/contact/lists/put:tooManyLists
    meaning: The context already holds as many lists as the agent allows. The limit is returned so a producer can say so.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: [limit]
      properties:
        limit: { type: integer, minimum: 1 }
related:
  - persona/contact/lists/list
  - persona/contact/lists/show
  - persona/contact/lists/delete
  - persona/contact/put
  - persona/contact/list
---

## Abstract

A holder keeps their own groupings of the people they know — "Mentors", "Met at
the spring meetup", "Family" — across every community they belong to.
**Persona Contact Lists — Put** creates one, or renames, recolours or reorders it.

A list is the holder's, **kept by the holder's agent and nowhere else.** The
people on it do not learn they are on it, and neither does any community they
were met through. This is the "Close Friends" rule: being on someone's list must
not be inferable by anyone else, including the operator of the community where
the two met. A list kept by a community would hand its operator the graph of who
watches whom; a list kept by the holder's agent can hold people from two
communities while neither community learns it exists.

A list is **context-scoped**, like the contacts it holds. It may hold contacts
known by any persona in its context and from any community, but never a contact
of another context: a list spanning contexts would join exactly what the
context boundary exists to keep apart.

This task writes the list record — `name`, `colour`, `order`. **Membership is set
on the contact**, by the `lists` member of
[`persona/contact/put`](../../../put/1.1/spec.md) 1.1, so it has one writer.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/contact/lists/put/1.0`, populate `contextId`
and exactly one of the two forms — create (no `listId`, with `name`) or update
(`listId` and at least one change) — and include a `proof` per
[SPEC.md §4.7](/SPEC.md#47-proof). It **SHOULD** trim leading and trailing white
space from `name`; the agent stores what it receives.

A conforming **maintainer** (the holder's agent) **MUST**:

1. Confine the caller to its own context, and answer `notFound` for a `listId` it holds in any other context exactly as for one it never assigned.
2. On a create, assign the `listId`, and place a list given no `order` after every existing list.
3. On an update, apply every change together or not at all: each detail given replaces the stored one, `removeColour` removes the colour, each detail absent is unchanged.
4. Refuse with `nameTaken` a name another list in the context already has, compared exactly, without case-folding or normalising.
5. Answer `changed: false`, write nothing and leave `updatedAt` where it was, when every detail given already matches.
6. **MUST NOT** send a list, its name, or the fact that a contact is on it to anyone but a caller authorised for this context — not to a contact, not to a community, and not in any document it signs, presents or seals for the holder.
7. **MUST NOT** write a list's name to a log or an audit record. It **MAY** record that a list was created or changed, and when.

## Authorization

**Context-scoped**, confined to the caller's own context — the same standing every
`persona/contact/*` task asks for. The `proof` identifies the caller so the agent
can look up which contexts it serves ([SPEC.md §7.2 item 10](/SPEC.md#72-consumer-requirements));
it is not itself the entitlement. A list confers nothing on anyone: it is a
grouping in the holder's address book, and no authorization anywhere may rest on
it. Whether anything more is required before writing is the consumer's policy,
which this specification does not declare.

## Definitions

- `ContactList`, `ListId`, `ListName`, `ListColour`, `ListOrder` — see [`persona/_shared/0.1/contact-annotations.schema.json`](../../../../_shared/0.1/contact-annotations.schema.json).

## Request

An **Application** sends this to the holder's **Agent**. The payload validates
against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Create a list

```json
{
  "id": "urn:uuid:2a3b4c5d-6e7f-4a8b-9c0d-1e2f3a4b5c01",
  "type": "https://trusttasks.org/spec/persona/contact/lists/put/1.0",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:agent.alice.example",
  "issuedAt": "2026-10-18T08:00:00Z",
  "payload": {
    "contextId": "ctx-personal",
    "name": "Mentors",
    "colour": "#1f8a70",
    "order": 10
  }
}
```

## Response

The list as it now stands, whether it was created, and whether anything changed,
in a payload that validates against the sub-schema reachable via
`$anchor: "response"`. A refusal is a `trust-task-error`.

```json
{
  "id": "urn:uuid:2a3b4c5d-6e7f-4a8b-9c0d-1e2f3a4b5c02",
  "type": "https://trusttasks.org/spec/persona/contact/lists/put/1.0#response",
  "threadId": "urn:uuid:2a3b4c5d-6e7f-4a8b-9c0d-1e2f3a4b5c01",
  "issuer": "did:web:agent.alice.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-18T08:00:01Z",
  "payload": {
    "list": {
      "listId": "01JA8Z5M2N3P4Q5R6S7T8V9W0X",
      "name": "Mentors",
      "colour": "#1f8a70",
      "order": 10,
      "contactCount": 0,
      "createdAt": "2026-10-18T08:00:01Z",
      "updatedAt": "2026-10-18T08:00:01Z"
    },
    "created": true,
    "changed": true
  }
}
```

## Security & Privacy

### Data carried

The request carries a list's name, colour and order. The name is free text the
holder wrote about a group of people, and can be as revealing as anything in
their address book ("Owes me money", "Do not introduce"). It carries no contact
and no DID.

**A list never leaves the holder's agent.** Neither does the fact that anyone is
on one. The agent returns lists only to clients the holder authorised for the
context (Conformance item 6). A producer **MUST NOT** place a secret in a list
name or in `ext`.

### Correlation

This task names no third party, so there is nothing about anyone else to
correlate within it. The rule that matters is the one this family holds for every
annotation: **being on a holder's list MUST NOT be inferable by the person listed
or by any community.** No list, list identifier or membership is ever part of a
message to a counterparty or a community, and a maintainer **MUST NOT** let one
change anything they can observe — what is presented to them, when the agent
contacts them, or which directory entries it reads.

`listId` is meaningful only inside one context of one agent, and is never sent
outside it, so it cannot link the holder across communities.

### Retention

A list persists until the holder deletes it with
[`persona/contact/lists/delete`](../../delete/1.0/spec.md). It has no history: a
replaced name is gone, which is the smallest record a later breach could expose.
A list is backed up with the holder's contacts, and protected as they are.

### Consent/purpose

The purpose is the holder's own organisation of the people they know: finding
them, filtering by them, and — where the holder's agent holds a room's keys —
sharing a room with a group of them by the holder's own choice. Using a list to
rank, recommend or profile the people on it, or reporting it to anyone, is
outside that purpose. Whether anything more is required before creating or
changing a list is the consumer's policy and is deliberately not specified here.
