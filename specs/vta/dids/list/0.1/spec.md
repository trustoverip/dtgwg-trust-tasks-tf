---
slug: vta/dids/list
version: "0.1"
title: "VTA DIDs — List"
summary: "Enumerate every identity a VTA holds that the caller may see — whatever its DID method or key assurance level — as one summary each, optionally narrowed to a context, methods, a hosting server or a DID class."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords:
  - vta
  - did
  - list
  - did method
  - did:webvh
  - did:webs
  - did:key
parties:
  - role: Caller
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: OPTIONAL
  rationale: "A read with no state change; the transport's authenticated sender is what the VTA resolves the caller's scope against."
sideEffects:
  level: none
  rationale: "Enumeration only."
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: "Returns the VTA's summary of each identity the caller may see: identifiers, context, class, level, the VTA's key ids and hosting server. No key material."
retention:
  class: transient
  rationale: "A read; nothing in it needs keeping beyond the caller's use of the answer."
errorCodes: []
related:
  - vta/dids/get
  - vta/dids/methods/list
  - vta/contexts/update-did
---

## Abstract

**VTA DIDs — List** answers *which identities does this VTA hold?* once, for every DID
method, instead of once per method. Each item is a `DidSummary`: what every identity has —
its identifier, method, context, class, key assurance level, whether it is its context's
DID, the VTA's ids for its current keys — with what only its method has under
`methodDetails`. What an identity *does* stays with its method's own tasks. The family's
rules are [CONVENTIONS.md §20](../../_shared/0.2/CONVENTIONS.md#20-every-identity-list-get-and-methods).

This task supersedes `vta/webvh/dids/list/1.0` and `vta/webs/dids/list/0.1`, which asked
the same question one method at a time.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the VTA) **MUST** summarise every identity
[CONVENTIONS.md §20.1](../../_shared/0.2/CONVENTIONS.md#201-which-identities-are-summarised)
names, of every method it lists, and **MUST** return only those whose context the caller
may access, without signalling the existence of any it filtered out: a `contextId` the
caller cannot reach answers an empty page, not an error. It **MUST** leave out deactivated
identities unless `includeDeactivated` is true. Every filter narrows; filters combine as a
conjunction. A `methods` entry the VTA holds no profile for matches nothing.

An empty array is a **successful** answer, and does not mean the VTA holds none. A caller
that needs to know whether one identity exists asks [`vta/dids/get`](../../get/0.1/spec.md),
which answers `notFound`.

Pages follow the registry's convention: `cursor` and `limit` in, `nextCursor` out, absent
on the last page. A consumer **MUST NOT** accept a cursor it did not issue, or one issued
for other filters.

A conforming **producer** that needs every identity **MUST** follow `nextCursor` until it
is absent; a short page is not the end. It **MUST** ignore a `methodDetails` member it does
not recognise.

## Definitions

- `contextId` — only identities in this context.
- `methods` — only identities of these methods, named as their profiles are
  (`DidMethodName`: `webvh`, `scid:vh`, `webs`, `key`).
- `serverId` — only identities published through this hosting server.
- `class` — only identities created in this DID class. An identity with no recorded class
  never matches.
- `includeDeactivated` — include identities that have been ended. Absent means no.
- `cursor`, `limit` — paging.

## Authorization

Authority is **access to each identity's context**, applied per identity
([CONVENTIONS.md §20.2](../../_shared/0.2/CONVENTIONS.md#202-who-sees-what)): the answer is
the subset of the VTA's identities the caller may see. Listing needs no authority beyond
that.

Verifying the producer's VID or `proof` establishes *who is asking*, never *what they may do* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The context check follows, and is the authorization.

## Request

A caller asks the VTA; the payload is [`payload.schema.json`](payload.schema.json).

### Every identity in one context

```json
{
  "id": "urn:uuid:5b0c2e1a-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/dids/list/0.1",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-17T10:00:00Z",
  "payload": {
    "contextId": "alice",
    "limit": 50
  }
}
```

## Response

The VTA answers with the summaries the caller may see, under `$anchor: "response"`.
Failures use `trust-task-error`.

### Three methods, one list

```json
{
  "id": "urn:uuid:5b0c2e1a-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/dids/list/0.1#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkOperator",
  "issuedAt": "2026-11-17T10:00:01Z",
  "threadId": "urn:uuid:5b0c2e1a-0000-4000-8000-000000000001",
  "payload": {
    "dids": [
      {
        "did": "did:webs:example.com:people:alice:ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
        "method": "webs",
        "contextId": "alice",
        "class": "persona",
        "level": 1,
        "primary": true,
        "deactivated": false,
        "serverId": "prod",
        "signingKeyId": "webs-alice-sign-0",
        "keyAgreementKeyId": "webs-alice-ka-0",
        "alsoKnownAs": [
          "did:web:example.com:people:alice:ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
          "did:keri:ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe"
        ],
        "createdAt": "2026-11-11T10:00:02Z",
        "updatedAt": "2026-11-11T10:00:02Z",
        "methodDetails": {
          "webs": {
            "aid": "ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
            "slot": "people/alice/ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
            "domain": "example.com",
            "sequenceNumber": 0,
            "keriCesrUrl": "https://example.com/people/alice/ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe/keri.cesr",
            "didJsonUrl": "https://example.com/people/alice/ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe/did.json"
          }
        }
      },
      {
        "did": "did:webvh:QmAliceScid:example.com:alice",
        "method": "webvh",
        "contextId": "alice",
        "level": 1,
        "primary": false,
        "deactivated": false,
        "serverId": "prod",
        "signingKeyId": "did:webvh:QmAliceScid:example.com:alice#key-0",
        "keyAgreementKeyId": "did:webvh:QmAliceScid:example.com:alice#key-1",
        "alsoKnownAs": [],
        "createdAt": "2026-10-02T09:12:00Z",
        "updatedAt": "2026-10-02T09:12:00Z",
        "methodDetails": {
          "webvh": {
            "scid": "QmAliceScid",
            "mnemonic": "alice",
            "portable": false,
            "logEntryCount": 1,
            "preRotationCount": 1
          }
        }
      },
      {
        "did": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "method": "key",
        "contextId": "alice",
        "label": "wallet holder",
        "level": 1,
        "primary": false,
        "deactivated": false,
        "signingKeyId": "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK#z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK",
        "alsoKnownAs": [],
        "createdAt": "2026-11-01T08:00:00Z",
        "updatedAt": "2026-11-01T08:00:00Z"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries filters and a page position. The response carries one summary per
identity the caller may see: identifiers, the VTA's bookkeeping (context, class, level,
label, key ids, hosting server) and the method's public details. No key material.

### Correlation

A list is a join no resolver can make: it shows which identities the operator keeps
together, across methods, which is exactly what separate persona and relationship DIDs
exist to hide from their counterparties. Listing every method at once makes the join
wider, not new — each method's list already disclosed its part of it to the same caller.
The task returns it only to callers who may see each context, and an operator **SHOULD**
give list access to a context only to callers entitled to see every identity in it.

### Retention

A read. The caller needs the answer only for as long as it acts on it.

### Consent/purpose

The list is returned to manage and act as the VTA's identities, and **SHOULD NOT** be kept
or exported as a directory of the subject's identities.
