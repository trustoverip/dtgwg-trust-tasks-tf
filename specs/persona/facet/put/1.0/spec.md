---
slug: persona/facet/put
version: "1.0"
title: "Persona — Facet Put"
summary: A holder names a part of their life — Work, Home, Play — and says which faces and attributes belong to it, so a wallet can arrange a pool it has outgrown and can warn when one part of a life shares a value with another.
status: retired
supersededBy: persona/world/put/1.0
targetFrameworkVersion: "0.5.0"
category: identity
keywords:
  - persona
  - facet
  - grouping
  - arrangement
parties:
  - role: Holder
    requirement: REQUIRED
    member: issuer
  - role: Agent
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: A facet names records that sit above every trust context, so a document that could compose one without attribution would let anything holding a session rearrange the holder's own account of their life. Attribution must survive the transport for the same reason it must for the profiles a facet groups.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A put replaces a facet's membership wholesale, so two copies applied out of order leave the older arrangement in place — and the older arrangement may place a face somewhere the holder has since moved it from.
sideEffects:
  level: mutating
  rationale: >-
    Creates or replaces one facet. Recoverable, and unusually so — no profile
    and no attribute is touched by any outcome of this task, so the worst case
    is an arrangement the holder must redo rather than anything they must
    recover.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: "`name` is the holder's own word for a part of their own life — \"Work\", \"the divorce\" — which is personal even though it is not a value about them. The membership lists carry identifiers only. The response returns identifiers, a version and timestamps."
errorCodes:
  - code: persona/facet/put:faceAlreadyPlaced
    meaning: One or more of the listed `faceIds` already belongs to a different facet. The details name each offending profile and the facet currently holding it, so a producer can offer to move it rather than guessing. The facet is not written.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required: ["placed"]
      properties:
        placed:
          type: array
          maxItems: 256
          items:
            type: object
            additionalProperties: false
            required: ["faceId", "facetId"]
            properties:
              faceId:
                type: string
              facetId:
                type: string
  - code: persona/facet/put:unresolvedReference
    meaning: A listed `faceId` or `attributeId` names a record the holder does not hold. The details name them. The facet is not written — an arrangement referring to something that never existed is a typo, and accepting it silently makes the typo permanent.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      properties:
        faceIds:
          type: array
          maxItems: 256
          items:
            type: string
        attributeIds:
          type: array
          maxItems: 1024
          items:
            type: string
  - code: persona/facet/put:versionConflict
    meaning: The `expectedVersion` precondition failed. Details carry the maintainer's current version.
    retryable: false
related:
  - persona/facet/list
  - persona/facet/delete
  - persona/profile/put
---

## Abstract

> **Retired in favour of [`persona/world/put/1.0`](../../../world/put/1.0).** Same
> task, same members, one word changed: what this family calls an arrangement of faces is
> a **world**, and `facet` shared a stem with `face` while naming something different.
> `facetId` is `worldId` there; nothing else moved. This specification stays readable so
> documents already issued against it remain verifiable.

**Persona Facet Put** names a part of a holder's life and says what belongs to
it.

A holder who uses this model for a while does not end up with three profiles.
They end up with twenty — one per site, one per counterparty, one they made once
and cannot now remember the purpose of — and a flat list of twenty is a list
nobody reads. A facet is the arrangement over them: *Work*, *Home*, *Play*, and
whatever else a particular life actually has in it.

This is a Trust Task rather than a client-side preference for one reason:
**a holder's arrangement of their own identity has to be the same arrangement on
every device they use.** A grouping kept in a browser's local storage is a
grouping their phone does not have, and the arrangement is worth more than the
device it was made on. It also gives the maintainer something it cannot
otherwise know — that two profiles are, to the holder, parts of the same life —
which is what turns a linkage report from a list of every shared value into a
list of the ones that cross a boundary the holder drew.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

The authority is **ownership of the pool being arranged**. A facet groups
profiles and attributes that are agent-scoped — they sit above every trust
context — so the entitlement is the same one `persona/profile/put` and
`persona/attribute/put` require: the issuer must be the holder whose records
these are, as the maintainer identifies them.

A consumer **MUST** establish that entitlement independently of the proof.
Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID,
`issuer`, `recipient`, transport identity or `proof` establishes *who* and
*unaltered*, never *authorized*. In particular an administrator scoped to one
trust context is **not** authorized here: a facet names records belonging to
every context, so an authority bounded by one of them is bounded by less than
this task reaches.

This task is not open to any caller.

## Definitions

**Facet** — a named part of the holder's life, and the profiles and attributes
that belong to it. It is an *arrangement*, not a container: nothing is stored
inside a facet, and deleting one deletes nothing but the arrangement.

**`facetId`** — the record's identity, assigned by the maintainer on create.
A producer supplies one to replace, or to make a create idempotent under retry.

**`name`** — the holder's own word, 1–64 characters. Never disclosed to a
verifier, never interpreted by the maintainer: it is not a scope, not a policy
input, and not a name any counterparty sees.

**`colour`** — a colour *name* from a closed set, resolved by each consumer
against its own palette. See [Definitions of colour](#why-a-name-and-not-a-value).

**`icon`** — one or two emoji, at most 8 bytes, carrying no meaning of its own.
A consumer that cannot render it omits it and shows the name.

**`faceIds`** — the profiles belonging to this facet. **A profile belongs to at
most one facet.** A maintainer **MUST** refuse a document placing a profile that
another facet already holds, with `persona/facet/put:faceAlreadyPlaced`.

**`attributeIds`** — the attributes belonging to this facet. **An attribute MAY
belong to several**, and a maintainer **MUST NOT** enforce exclusivity over
them: a mobile number is genuinely part of both a working life and a home one,
and a model that made the holder choose would be asking them to answer a
question about their phone that has no answer.

### Why membership lives on the facet

The obvious alternative is a `facetId` member on `persona/attribute/put` and
`persona/profile/put`. It is the wrong shape, and the reason is
mechanical rather than aesthetic.

`persona/attribute/put` **replaces** the attribute. A consumer that wants to
place forty attributes into a facet would have to send forty puts, each of which
must carry the attribute's `value` — and a well-behaved consumer does not have
those values: `persona/attribute/list` withholds the plaintext of anything
resolving to `sensitivity: high` unless it is asked for by name, which is the
point of that member. So the consumer either asks for every sensitive value it
holds in order to perform an arrangement that has nothing to do with values, or
it sends a put without one and silently destroys them.

Membership on the facet has neither problem. One record is written, no value
crosses the wire, and the worst outcome of any error is an arrangement to redo.

### Why a name and not a value

`colour` carries a name — `teal`, `plum` — and never a hex value or any other
literal. Two reasons, both about the consumer rather than the holder.

A literal cannot be legible in a terminal, in a light theme and in a dark one at
the same time, so a stored `#8B0000` is a colour that is wrong somewhere and the
holder has no way to know where. And a consumer that reserves colours to carry
meaning — an error, a warning, an irreversible act — has to be able to keep a
decorative choice out of that channel. It cannot do that with an arbitrary
value. It can do it trivially with a closed set it maps itself.

The eight members are chosen to be distinguishable from one another and to carry
no status connotation: none is named for success, warning or danger.

## Request

The **Holder** issues the request to their **Agent**. The payload is described
by the top-level schema in [`payload.schema.json`](payload.schema.json).

Both membership lists are **replaced**, not merged. Omitting one means an empty
list rather than "leave it as it was" — a member whose absence meant *keep*
would make it impossible to empty one.

### Creating a facet with two profiles and a shared attribute

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/persona/facet/put/1.0#request",
  "issuer": "did:example:holder",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "name": "Work",
    "colour": "teal",
    "icon": "💼",
    "faceIds": ["01J8XR3QK9V0000000000000A1", "01J8XR3QK9V0000000000000A2"],
    "attributeIds": ["01J8XR3QK9V0000000000000B7"],
    "expectedVersion": 0
  }
}
```

## Response

The **Agent** — the recipient of the request, now responding — returns the
record's identity and its new version. The payload is described by the
sub-schema reachable via `$anchor: "response"`.

`created` distinguishes a create from a replacement, which a producer that
supplied its own `facetId` cannot otherwise tell. Failures use `trust-task-error`
carrying one of the codes declared in this document's front matter, never a
`#response` document.

### The facet was created

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/persona/facet/put/1.0#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:holder",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "facetId": "01J8XR3QK9V0000000000000C4",
    "version": 1,
    "created": true,
    "createdAt": "2026-01-01T00:00:01Z",
    "updatedAt": "2026-01-01T00:00:01Z"
  }
}
```

## Security & Privacy

### Data carried

The request carries the holder's own word for a part of their own life, and
identifiers. `name` is the sensitive member and it is sensitive in a way that is
easy to miss: it is not a value *about* the holder, and it can still be the most
revealing thing in the store. *Work* discloses nothing. *Recovery*, *the
divorce*, *the second family* each disclose a great deal, and a holder naming a
facet is not thinking about a maintainer's logs. A maintainer **MUST NOT** write
`name` to an operational log, an audit record readable by anyone but the holder,
or a metric label; and a producer **MUST NOT** put a value about the holder in
`name` or `icon` — those members exist to find a facet again, and any other use
is a value stored somewhere the pool's protections do not reach.

The smallest payload that answers the task is `name` and `colour`. Membership
may be added later, and a producer building a facet before the holder has
decided what goes in it **SHOULD** send exactly that.

### Correlation

A facet is the holder's own statement that two identities belong to the same
part of one life, which is precisely the kind of link the rest of this family
exists to help them avoid making by accident. It is therefore **agent-scoped and
stays there**. A facet, its name, its colour and its membership **MUST NOT**
cross into a trust context: they **MUST NOT** appear in a materialised
projection (`persona/binding/set`), in a disclosure, or in any document a
verifier receives. A context that could read them would learn how the holder
organises every *other* context, which is the one thing the direction of this
model is arranged to prevent.

Within the agent, a facet makes an existing correlation report sharper rather
than adding a new exposure: a maintainer that already indexes shared values
(`persona/correlation/analyze`) can distinguish sharing *inside* a facet, which
the holder intended, from sharing *across* facets, which is the finding worth
raising. That refinement is a matter for that specification and is named here
only because it is the reason this one is worth implementing.

### Retention

A facet is durable by nature: it is an arrangement the holder made and expects
to find again. A maintainer keeps it until the holder deletes it
(`persona/facet/delete`). It has no evidentiary value — nothing about a facet is
ever presented to a counterparty — so there is no reason to retain a deleted one
and a maintainer **SHOULD NOT**.

### Consent/purpose

The purpose is the holder's own navigation of their own records, and the derived
purpose of telling them when two parts of their life share a value. Reusing a
facet for anything else — inferring a policy scope from it, grouping audit
records by it, treating membership as authority — is outside the purpose it was
collected for. A facet is not a permission and a maintainer **MUST NOT** read it
as one.

This document is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare that consent, approval or a step-up is required. That policy belongs to the consumer.
