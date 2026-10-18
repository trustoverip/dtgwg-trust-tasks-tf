---
slug: persona/contact/lists/delete
version: "1.0"
title: Persona Contact Lists — Delete
summary: Delete one of the holder's own contact lists and take every contact off it, leaving the contacts themselves untouched; a repeated delete converges.
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
  rationale: A delete destroys the holder's own grouping of people and every membership in it. Attribution must survive the transport so a list that vanished can be traced to the key that removed it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A delete and a later re-creation of a list with the same name are distinguishable only by order, and SPEC §7.3 item 17 sets REQUIRED for a destructive task.
sideEffects:
  level: destructive
  rationale: "Removes the list and takes every contact off it. Not recoverable: a new list with the same name is a new list, and the memberships are gone."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: The request carries an identifier; the response carries counts. Neither carries the list's name or any contact.
retention:
  class: transient
  rationale: Nothing is kept of the list once deleted; the agent MAY record that a list was deleted, and when.
errorCodes: []
related:
  - persona/contact/lists/put
  - persona/contact/lists/list
  - persona/contact/delete
---

## Abstract

**Persona Contact Lists — Delete** removes one of the holder's lists and takes
every contact off it. The contacts themselves are untouched — their revisions,
their other lists and their other annotations stay — because deleting a grouping
is not deleting the people in it.

A repeated delete converges: `existed: false` and `contactsUpdated: 0`, so a
retry after a lost response is harmless and nothing watching the store sees a
change that did not happen.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/contact/lists/delete/1.0`, populate
`contextId` and `listId`, and include a `proof` per
[SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST**:

1. Confine the caller to its own context, and treat a `listId` held in another context exactly as one that does not exist: `existed: false`, nothing changed.
2. Remove the list and remove its `listId` from every contact's `lists` together, or do neither. A contact left naming a deleted list would make [`persona/contact/get`](../../../get/1.1/spec.md) return an identifier nothing resolves.
3. **MUST NOT** delete, or append a revision to, any contact.
4. **MUST NOT** echo the list's name or its contacts in the response, and **MUST NOT** write them to a log or an audit record.

## Authorization

**Context-scoped**, confined to the caller's own context — the same standing every
`persona/contact/*` task asks for. The `proof` identifies the caller so the agent
can look up which contexts it serves ([SPEC.md §7.2 item 10](/SPEC.md#72-consumer-requirements));
it is not itself the entitlement. What confirmation a producer obtains before
destroying a list is the consumer's policy, which this specification does not
declare.

## Definitions

- `ListId` — see [`persona/_shared/0.1/contact-annotations.schema.json`](../../../../_shared/0.1/contact-annotations.schema.json).

## Request

```json
{
  "id": "urn:uuid:5d6e7f8a-9b0c-4d1e-8f2a-3b4c5d6e7f01",
  "type": "https://trusttasks.org/spec/persona/contact/lists/delete/1.0",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:agent.alice.example",
  "issuedAt": "2026-10-18T10:00:00Z",
  "payload": { "contextId": "ctx-personal", "listId": "01JA8Z5M2N3P4Q5R6S7T8V9W0X" }
}
```

## Response

Whether the list existed and how many contacts were taken off it, in a payload
that validates against the sub-schema reachable via `$anchor: "response"`. A
refusal is a `trust-task-error`.

```json
{
  "id": "urn:uuid:5d6e7f8a-9b0c-4d1e-8f2a-3b4c5d6e7f02",
  "type": "https://trusttasks.org/spec/persona/contact/lists/delete/1.0#response",
  "threadId": "urn:uuid:5d6e7f8a-9b0c-4d1e-8f2a-3b4c5d6e7f01",
  "issuer": "did:web:agent.alice.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-18T10:00:01Z",
  "payload": { "listId": "01JA8Z5M2N3P4Q5R6S7T8V9W0X", "existed": true, "contactsUpdated": 3 }
}
```

## Security & Privacy

### Data carried

The request carries identifiers. The response carries counts. Neither carries the
list's name or its members, and a maintainer **MUST NOT** echo them — returning
what was just deleted would put the holder's private grouping of people into logs
that outlive the record.

### Correlation

Nothing is sent to a third party. **Being on a holder's list MUST NOT be inferable
by the person listed or by any community**, and that holds for having been taken
off one: a deletion changes nothing a contact or a community can observe. The
counts describe the holder's own store.

### Retention

This task is the retention control for a list. Once deleted, nothing of it
remains but, at most, a record that a list was deleted and when. Unlike a
contact's revisions, no list is kept for a disclosure reference: a list evidences
nothing anyone asserted.

### Consent/purpose

The purpose is to let the holder stop keeping a grouping they no longer want. The
people on the list are not told, which is correct: they were never told they were
on it. What confirmation a producer requires before destroying a list is the
consumer's policy and is deliberately not specified here.
