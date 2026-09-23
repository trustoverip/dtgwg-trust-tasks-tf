---
slug: persona/world/delete
version: "1.0"
title: "Persona — World Delete"
summary: A holder removes one of the parts of their life they had named, leaving every face and attribute that belonged to it exactly where it was.
status: draft
targetFrameworkVersion: "0.5.0"
category: identity
keywords:
  - persona
  - world
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
  rationale: >-
    A delete is irreversible for the record it names, and the record sits above
    every trust context. Attribution must survive the transport so an audit
    record names the key that removed the arrangement rather than the session it
    arrived on.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Required of any consequential task by SPEC §7.3 item 17. A replayed delete
    against a worldId the holder has since reused would remove an arrangement
    they had just rebuilt, and §7.2 item 11 can only absorb the duplicate inside
    a bounded window.
sideEffects:
  level: destructive
  rationale: >-
    Removes one world permanently. Declared destructive because the record cannot
    be recovered — though what it named can: no profile and no attribute is
    touched, so the loss is the arrangement and nothing else.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries one identifier. The response carries a boolean and a
    count of the profiles that no longer belong to any world.
errorCodes:
  - code: persona/world/delete:versionConflict
    meaning: The `expectedVersion` precondition failed — the world changed after the caller read it. Details carry the maintainer's current version. Nothing is deleted.
    retryable: false
related:
  - persona/world/put
  - persona/world/list
---

## Abstract

**Persona World Delete** removes one of the parts of a holder's life they had
named.

The whole of its design is one sentence: **a world is an arrangement, not a
container.** Deleting *Work* deletes the word *Work* and the statement about
what belonged to it. Every profile and every attribute that belonged survives
untouched, and a holder who deletes a world by mistake has lost a few minutes of
tidying rather than any part of their identity.

That is worth stating in a specification because the alternative reading is the
intuitive one. A grouping that looked like a folder, and behaved like one,
would make this the most dangerous task in the family — and a holder cannot tell
which kind they have from the button.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

The authority is **ownership of the world**, exactly as for `persona/world/put`:
the issuer must be the holder whose record it is, as the maintainer identifies
them. An administrator scoped to a single trust context is **not** authorized.

A consumer **MUST** establish that entitlement independently of the proof
([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). This task is not open
to any caller.

## Definitions

**`worldId`** — the world to remove.

**`expectedVersion`** — OPTIONAL precondition. Supply the version a prior read
returned to refuse a delete against a world that has changed since it was looked
at — the case where someone confirmed a screen describing a different
arrangement than the one about to go.

**`existed`** — whether there was anything to delete. A maintainer **MUST**
return `existed: false` as a **success**, not an error: a producer retrying
after a lost response has to be able to reach the state it wanted without having
to tell a second delete apart from a first.

**`releasedFaces`** — how many profiles now belong to no world. Nothing was
deleted and nothing about them changed; this is the count a consumer needs in
order to say what the screen will look like afterwards, which is the honest end
of a sentence that would otherwise read only *deleted*.

### What a maintainer MUST NOT do

A maintainer **MUST NOT** delete, unbind, modify or mark any profile or
attribute named by the world's membership. Deleting a world is not a cascade and
has no cascading form: there is deliberately no `cascade` member on this task,
because an arrangement that could take its members with it is a folder, and a
holder who reads it as a folder is right to be afraid of it.

## Request

The **Holder** issues the request to their **Agent**. The payload is described
by the top-level schema in [`payload.schema.json`](payload.schema.json).

### Deleting a world the caller has just read

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/persona/world/delete/1.0#request",
  "issuer": "did:example:holder",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "worldId": "01J8XR3QK9V0000000000000C4",
    "expectedVersion": 4
  }
}
```

## Response

The **Agent** reports whether there was anything to remove, and what no longer
belongs anywhere. The payload is described by the sub-schema reachable via
`$anchor: "response"`. Failures use `trust-task-error` rather than a `#response`
document.

### It existed, and two faces now belong to no part of a life

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/persona/world/delete/1.0#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:holder",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "existed": true,
    "releasedFaces": 2
  }
}
```

## Security & Privacy

### Data carried

The request carries one identifier and an optional version. The response carries
a boolean and a count. Neither carries a world name, a value, or anything about
the holder — a delete does not need to echo what it removed, and echoing the
name would put the most sensitive member of this family into a response for no
purpose the caller has.

### Correlation

Nothing here correlates: the identifiers are opaque and local to the holder's
own store. The world whose deletion this records was itself agent-scoped and
never crossed into any trust context, so its removal is invisible everywhere a
verifier can see.

### Retention

The world is removed. A maintainer **SHOULD NOT** retain a deleted world — it
has no evidentiary value, since nothing about a world is ever presented to a
counterparty — and any audit record of the deletion **MUST NOT** include the
name.

### Consent/purpose

The purpose is the holder tidying their own arrangement. There is nothing to
reuse: the response carries no personal data.

This document is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare that consent, approval or a step-up is required. That policy belongs to the consumer.
