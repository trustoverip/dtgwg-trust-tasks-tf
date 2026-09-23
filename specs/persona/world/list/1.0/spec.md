---
slug: persona/world/list
version: "1.0"
title: "Persona — World List"
summary: A holder reads back the parts of their life they have named, with the faces and attributes belonging to each, cursor-paginated so a client that follows the cursor sees all of them.
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
    The response enumerates how the holder has arranged their own identity across
    every trust context. A proofless request would let anything holding a session
    read that arrangement, which is a map of the holder's life even where it
    carries no value about them.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A read of the holder's own arrangement is worth replaying, so its documents
    must be placeable in time for §7.2 item 11 to absorb a duplicate.
sideEffects:
  level: none
  rationale: A read. Nothing is created, changed or removed.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Identifiers, versions and the holder's own labels, returned to the holder —
    `metadata`, on the same reading `persona/profile/list` takes of a profile
    name. The membership lists nonetheless say which identities the holder
    considers parts of one life, which is the most correlating statement in this
    family; that is a matter for §Correlation rather than for this level, and it
    is why a world never leaves the agent scope.
errorCodes:
  - code: persona/world/list:invalidCursor
    meaning: The supplied cursor was not one this maintainer issued, or has expired. A producer restarts the listing from the beginning rather than guessing a replacement.
    retryable: false
related:
  - persona/world/put
  - persona/world/delete
  - persona/profile/list
---

## Abstract

**Persona World List** reads back the parts of a holder's life they have named,
with the profiles and attributes belonging to each.

It is the read half of `persona/world/put`, and it exists as its own task rather
than as a member on `persona/profile/list` for the reason the two are separate
records: a world groups attributes as well as profiles, and a listing that
returned it as a member of one of them would have to leave out the other half.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

The authority is **ownership of the records being read**, exactly as for
`persona/world/put`: the issuer must be the holder whose worlds these are, as
the maintainer identifies them. An administrator scoped to a single trust
context is **not** authorized — a world spans every context, so an authority
bounded by one of them is bounded by less than this task returns.

A consumer **MUST** establish that entitlement independently of the proof
([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). This task is not open
to any caller.

## Definitions

**`limit`** — the page size to ask for, 1–250, defaulting to 100. **It is not a
cap on the result.** A producer wanting every world follows `nextCursor` to the
end; one that treats `limit` as a maximum draws a partial picture and has no way
to discover it did.

**`cursor`** — opaque continuation, echoed from a prior response. A producer
**MUST NOT** construct one.

**`nextCursor`** — present when more worlds remain. **Its absence is the only
signal that a listing is complete.** A producer **MUST NOT** infer exhaustion
from a short page: a maintainer may return fewer than `limit` for reasons of its
own, and a short array is indistinguishable from a complete one.

**`World`** — one named part of the holder's life. See
[`persona/world/put`](../../put/1.0/spec.md) for the meaning of each member.

### Dangling membership is returned, not pruned

A maintainer **MUST NOT** silently drop a `faceId` or `attributeId` whose record
has since been deleted. A dangling member is how a consumer can offer to tidy;
removing it quietly turns a deletion the holder may not have intended into one
they can never see. A consumer renders a dangling id as an arrangement to repair
rather than as a record that exists.

## Request

The **Holder** issues the request to their **Agent**. The payload is described
by the top-level schema in [`payload.schema.json`](payload.schema.json); every
member is OPTIONAL, so an empty payload is a valid first page.

### The first page

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/persona/world/list/1.0#request",
  "issuer": "did:example:holder",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "limit": 100
  }
}
```

## Response

The **Agent** returns the page and, when more remain, a cursor. The payload is
described by the sub-schema reachable via `$anchor: "response"`. Failures use
`trust-task-error` rather than a `#response` document.

### Two worlds, and no more to come

`nextCursor` is absent, which is what says the listing is complete.

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/persona/world/list/1.0#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:holder",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "worlds": [
      {
        "worldId": "01J8XR3QK9V0000000000000C4",
        "name": "Work",
        "colour": "teal",
        "icon": "💼",
        "faceIds": ["01J8XR3QK9V0000000000000A1"],
        "attributeIds": ["01J8XR3QK9V0000000000000B7"],
        "version": 4,
        "createdAt": "2026-01-01T00:00:01Z",
        "updatedAt": "2026-01-01T00:00:01Z"
      },
      {
        "worldId": "01J8XR3QK9V0000000000000C5",
        "name": "Home",
        "colour": "moss",
        "faceIds": [],
        "attributeIds": ["01J8XR3QK9V0000000000000B7"],
        "version": 5,
        "updatedAt": "2026-01-01T00:00:02Z"
      }
    ]
  }
}
```

Attribute `…B7` appears in both, which is legal and is the case `attributeIds`
exists to carry: a mobile number belongs to a working life and a home one at the
same time.

## Security & Privacy

### Data carried

The request carries a page size and an opaque cursor — nothing about the holder.
The response carries every world `name` the holder has chosen and the membership
lists. `name` is the sensitive member: *Work* discloses nothing, and *Recovery*
or *the divorce* disclose a great deal. A maintainer **MUST NOT** write it to an
operational log, a metric label, or an audit record readable by anyone but the
holder.

A consumer needing only the names — to render a picker — still receives the
membership, because a world is small and a second shape would be a second thing
to keep correct. A consumer **SHOULD NOT** retain the membership beyond the
screen it drew.

### Correlation

This response is the most correlating document in the persona family, and it
correlates nothing outside the agent. It states, in the holder's own words,
which of their identities belong to the same part of one life — precisely the
join that the multiple-persona model exists to deny a verifier. It is therefore
**agent-scoped and stays there**: a world **MUST NOT** appear in a materialised
projection, a disclosure, or any document a verifier receives, and a
context-scoped caller **MUST NOT** be able to read one.

### Retention

A read. A maintainer retains nothing on account of it beyond whatever audit
record its own policy keeps, and such a record **MUST NOT** include world names.

### Consent/purpose

The purpose is the holder reading their own arrangement. Reusing the response —
to infer a policy scope, to group audit records, to treat membership as
authority — is outside it. A world is not a permission.

This document is descriptive: per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) it does not declare that consent, approval or a step-up is required. That policy belongs to the consumer.
