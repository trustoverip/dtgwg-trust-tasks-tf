---
slug: persona/contact/get
version: "1.1"
title: Persona Contact — Get
summary: Read one contact at its current or a named earlier revision, with the holder's own star, petname, lists and directory link — annotations only the holder's agent ever holds.
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords: [persona, contact, revision, history, petname, star, address-book]
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
  rationale: The response returns a third party's personal data and the holder's private annotations about them, so the agent must attribute the request to a key and an audit record must be able to name which application read it.
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
  rationale: The response returns what a counterparty disclosed to the holder and the holder's own annotations about that counterparty — back to a client of the holder's own, in the context the relationship lives in. Nothing is disclosed to the counterparty or to any community.
retention:
  class: transient
  rationale: A read is a point-in-time view; the stored contact is the record, and a caller keeps the response no longer than the rendering it serves.
errorCodes:
  - code: persona/contact/get:notFound
    meaning: No contact exists at that identifier in this context.
    retryable: false
  - code: persona/contact/get:revisionReaped
    meaning: The named revision existed and has been reaped under the retention policy. Distinct from notFound on purpose — a caller comparing against history must be able to tell "never existed" from "no longer kept", because only the second means their comparison is unsound rather than mistaken.
    retryable: false
related:
  - persona/contact/put
  - persona/contact/list
  - persona/contact/lists/show
---

## Abstract

**Persona Contact — Get** reads what a peer disclosed, and what the holder keeps
about them.

`includeHistory` returns revision **metadata without documents**, so a producer
can render a timeline cheaply and fetch only the revision the holder opens. A
reaped revision is reported as `revisionReaped`, distinct from `notFound`: a
caller comparing a current value against history needs to tell *never existed*
from *no longer kept*.

### What 1.1 adds

The holder's **annotations** — `starred`, `petname`, `lists` and `directoryRef`
— beside the 1.0 `notes`, as [`persona/contact/put`](../../put/1.1/spec.md) 1.1
writes them. They are **contact-level**: whichever revision is asked for, the
annotations returned are the contact's current ones, because they are the
holder's present view of the relationship, not part of what the counterparty
disclosed at that revision. `starred` is always present; the others are absent
when the holder set none.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues). Version 1.0 remains in force; 1.1 is a backwards-compatible MINOR increment ([SPEC.md §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a *Trust Task document* whose `type` is
`https://trusttasks.org/spec/persona/contact/get/1.1`, populate `contextId` and
`contactId`, and include a `proof` per [SPEC.md §4.7](/SPEC.md#47-proof).

A conforming **maintainer** **MUST** confine the caller to its own context;
**MUST** return the current revision when `rev` is omitted; **MUST** emit
`persona/contact/get:revisionReaped` rather than falling back to the nearest
surviving revision; **MUST NOT** include documents in `history`; **MUST** return
the contact's current annotations with any revision; and **MUST** return a
`lists` member naming only lists that exist in this context, dropping none the
contact is on.

A conforming **producer** that renders a petname **SHOULD** mark it as the
holder's own ("your name for them") and keep the contact's disclosed display name
reachable beside it, so that a holder who named a contact long ago can still see
what the contact calls themselves — and so that a contact's display name can never
pass for a petname the holder wrote.

## Authorization

**Context-scoped**, confined to the caller's own context. The entitlement is
being authorised for that context; the `proof` identifies the caller so the agent
can look that up ([SPEC.md §7.2 item 10](/SPEC.md#72-consumer-requirements)), and
is not itself the entitlement.

## Definitions

- `Petname`, `ListIds`, `DirectoryRef` — see [`persona/_shared/0.1/contact-annotations.schema.json`](../../../_shared/0.1/contact-annotations.schema.json).
- **Annotations** — `starred`, `petname`, `lists`, `directoryRef` and `notes`.

## Request

The contact, optionally a revision, optionally a timeline. The payload validates
against the top-level schema in [`payload.schema.json`](payload.schema.json).

```json
{
  "id": "urn:uuid:5b7e0c2a-3f1d-4c6b-8e9a-1d2c3b4a5f01",
  "type": "https://trusttasks.org/spec/persona/contact/get/1.1",
  "issuer": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "recipient": "did:web:agent.alice.example",
  "issuedAt": "2026-10-18T09:10:00Z",
  "payload": {
    "contextId": "ctx-personal",
    "contactId": "01JA8Z3Q4R5S6T7V8W9X0Y1Z2A",
    "includeHistory": false
  }
}
```

## Response

The contact at the requested revision, its annotations, and when asked a timeline
of revision metadata, in a payload that validates against the sub-schema reachable
via `$anchor: "response"`. A refusal is a `trust-task-error`.

```json
{
  "id": "urn:uuid:5b7e0c2a-3f1d-4c6b-8e9a-1d2c3b4a5f02",
  "type": "https://trusttasks.org/spec/persona/contact/get/1.1#response",
  "threadId": "urn:uuid:5b7e0c2a-3f1d-4c6b-8e9a-1d2c3b4a5f01",
  "issuer": "did:web:agent.alice.example",
  "recipient": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
  "issuedAt": "2026-10-18T09:10:01Z",
  "payload": {
    "contactId": "01JA8Z3Q4R5S6T7V8W9X0Y1Z2A",
    "subjectDid": "did:peer:2.Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc",
    "knownByPersona": "did:peer:2.Ez6LSghwSE437wnDE1pt3X6hVDUQzSjsHzinpX3XFvMjRAm7y",
    "rev": 1,
    "document": {
      "claims": [
        { "type": "name.display", "valueType": "string", "value": "Sam Okafor" }
      ]
    },
    "starred": true,
    "petname": "Sam (belay partner)",
    "lists": ["01JA8Z5M2N3P4Q5R6S7T8V9W0X"],
    "directoryRef": { "vtcDid": "did:web:climbing.example", "entryId": "e_7Hq2mZp9Lx4v" }
  }
}
```

## Security & Privacy

### Data carried

The response carries **a third party's personal data** and the holder's
**annotations** about them: a star, a petname, list memberships, a directory link
and a private note. The annotations are the most sensitive members: their subject
cannot see them and did not consent to them.

These annotations **never leave the holder's agent** except in this response, to
a client the holder authorised for this context. A maintainer **MUST NOT** return
them to any other party, and a producer **MUST NOT** forward them — to the
contact, to the community a `directoryRef` names, or into anything it sends on
the holder's behalf. A maintainer **MAY** omit `notes` or `petname` for a caller it
judges should not see them, and a producer **MUST NOT** treat their absence as an
error.

### Correlation

**Being on a holder's list, starred or given a petname MUST NOT be inferable by
the person annotated or by any community.** Reading a contact reveals nothing to
either: the read stays between the holder's client and the holder's agent. A
producer that then reads a community's directory to show this contact's entry
**SHOULD** do so as it would for any entry, so that the community cannot tell a
starred or listed member from any other by what the holder's client fetches.

`directoryRef` links a pairwise relationship to a community-scoped entry. It is
a correlator held privately and returned only to the holder's own client.
Reading history reveals when a counterparty changed what they present; a producer
**SHOULD** request it only when the holder is inspecting a change.

### Retention

A response is a point-in-time view with no evidentiary value of its own — the
stored contact is the evidence. It **SHOULD NOT** be retained by the caller beyond
the rendering it serves; a cached copy of a petname or a list membership is the
same disclosure of the holder's private judgement, outside the store that
protects it.

### Consent/purpose

The purpose is recall and organisation: the holder checks what a counterparty told
them, and sees how they have filed them. Reading it under a persona other than the
one it was disclosed to would present the counterparty with an identity they never
disclosed to; the record's `knownByPersona` exists so that boundary is checkable.
What gate a producer applies is the consumer's policy and is deliberately not
specified here.
