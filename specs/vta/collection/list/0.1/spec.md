---
slug: vta/collection/list
version: "0.1"
title: "VTA Collection — List"
summary: "A client lists who collects the messages queued at the mediator for each of the principal's persona DIDs in one trust context — the VTA, a claiming client and until when, or nobody and why."
status: draft
targetFrameworkVersion: "0.6.0"
category: notifications
keywords:
  - collection
  - mediator
  - persona
  - vta
parties:
  - role: Client acting for the principal
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read that changes nothing, normally carried by an authenticated transport between a client
    and its own VTA.
sideEffects:
  level: none
  rationale: "Read-only enumeration of the collection state of a context's identifiers."
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Out: the context's persona DIDs, and which of the principal's clients collects for each and
    until when. In: a context and filters.
retention:
  class: exchange
  rationale: >-
    The VTA remains the record; the page is the client's working copy for the view it renders.
errorCodes: []
related:
  - vta/collection/claim
  - vta/collection/release
  - vta/collection/settings/show
---

## Abstract

The **VTA Collection — List** Trust Task shows the principal who is collecting the messages
addressed to each of its persona DIDs in one trust context (VTI-NTF-091): the VTA, which collects
for every such DID no client has claimed (VTI-NTF-080); a client holding a claim made with
[`vta/collection/claim`](../../claim/0.1/spec.md), with the claim's end and whether it can still be
renewed; or nobody, because the VTA is at the bound it states on the identifiers it collects for, or
because another connection kept evicting its own and it stopped (VTI-NTF-085, VTI-NTF-088), or
because collection by default is disabled for the context — by its own setting, or by the VTA's
operator suspending it for every context (VTI-NTF-097, VTI-NTF-098), which
[`vta/collection/settings/show`](../../settings/show/0.1/spec.md) tells apart. The last
is the one a principal needs to see: a persona nobody collects for is one whose notifications never
arrive, and nothing else reports it.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

A filter suffices for this collection without a single-record twin: an identifier's existence is
not load-bearing here, because a client that asks about one identifier the VTA does not collect for
learns that definitively from [`vta/collection/claim`](../../claim/0.1/spec.md) or
[`vta/collection/release`](../../release/0.1/spec.md), which refuse it with `notCollectable`, and
an empty page for an `identifier` filter means the same.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` a caller without standing under [Authorization](#authorization),
   and answer nothing about any other context (VTI-NTF-091).
3. Return every identifier of `contextId` that VTI-NTF-080 names, whether or not collection by
   default is enabled for it (VTI-NTF-094), matching
   every filter given: `identifier` only, where given; of `collector` only, where given.
4. For each, state its `collector`: `client` with its `claim` while a claim is in force, including
   one that is no longer renewable; `vta` while the VTA collects for it; and `none` with its
   `notCollectedReason` otherwise.
5. Page with `cursor` and `limit`, refusing with `malformedRequest` a `cursor` it did not issue for
   the same filters.

**Retry class** (VTI-OPS-060): the first — no durable effect.

## Authorization

**Standing in the trust context** named by `contextId`, as for
[`vta/collection/claim`](../../claim/0.1/spec.md#authorization).

## Definitions

- **`contextId`** — the trust context to list.
- **`identifier`**, **`collector`** — optional filters.
- **`cursor`**, **`limit`** — paging.
- **`items`** (response) — one
  [`CollectionEntry`](../../_shared/0.1/collection.schema.json) per identifier.
- **`nextCursor`** (response) — present when more remain.

## Request

The client sends its VTA the top-level schema in [`payload.schema.json`](payload.schema.json).

### The personas of one context

```json
{
  "id": "urn:uuid:91a2b3c4-d5e6-4f7a-8b8c-9d0e1f2a3b4c",
  "type": "https://trusttasks.org/spec/vta/collection/list/0.1",
  "issuer": "did:key:z6MkWalletClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2027-03-01T09:00:00Z",
  "payload": {
    "contextId": "personal"
  }
}
```

## Response

The VTA answers with the sub-schema reachable via `$anchor: "response"`.

### One persona claimed, one collected by the VTA, one by nobody

```json
{
  "id": "urn:uuid:a2b3c4d5-e6f7-4a8b-9c9d-0e1f2a3b4c5d",
  "type": "https://trusttasks.org/spec/vta/collection/list/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkWalletClient",
  "issuedAt": "2027-03-01T09:00:01Z",
  "threadId": "urn:uuid:91a2b3c4-d5e6-4f7a-8b8c-9d0e1f2a3b4c",
  "payload": {
    "items": [
      {
        "identifier": "did:webvh:QmPersona:persona.example",
        "collector": "client",
        "claim": {
          "claimant": "did:key:z6MkDesktopClient",
          "endsAt": "2027-03-02T09:00:01Z",
          "renewable": true
        }
      },
      {
        "identifier": "did:webvh:QmSecond:persona.example",
        "collector": "vta"
      },
      {
        "identifier": "did:webvh:QmThird:persona.example",
        "collector": "none",
        "notCollectedReason": "conflict"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The context's persona DIDs, which of the principal's clients holds a claim for each and until when,
and why one is collected by nobody. Nothing about the messages queued for any of them.

### Correlation

The listing joins the principal's personas in one context, which a party with standing there can
already see. It is answered to nobody else (VTI-NTF-091).

### Retention

The VTA remains the record. A client **SHOULD NOT** keep a page beyond the view it renders.

### Consent/purpose

The listing is how a principal sees that a persona is collected for by nobody, and settles a
conflict between two collectors (VTI-NTF-085), which only the principal can do.
