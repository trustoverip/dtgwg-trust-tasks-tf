---
slug: vta/webs/dids/list
version: "0.1"
title: "VTA did:webs DIDs — List"
summary: "Enumerate the did:webs DIDs a VTA holds that the caller may see, optionally narrowed to a context or hosting server. Alpha."
status: draft
targetFrameworkVersion: "0.5.0"
category: did-management
keywords:
  - vta
  - did-webs
  - keri
  - did
  - list
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Caller
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: OPTIONAL
  rationale: "A read with no state change."
sideEffects:
  level: none
  rationale: "Enumeration only."
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: "Returns the VTA's records of the DIDs the caller may see. No key material."
retention:
  class: transient
  rationale: "A read; nothing in it needs keeping beyond the caller's use of the answer."
errorCodes: []
related:
  - vta/webs/dids/get
  - vta/webs/dids/create
  - vta/webvh/servers/list
---

## Abstract

**VTA did:webs DIDs — List** enumerates the `did:webs` DIDs a VTA holds, filtered to the
contexts the caller can reach, optionally narrowed to one context or one hosting server,
and paged. Each item is the record [`vta/webs/dids/get`](../../get/0.1/spec.md) returns,
without the stream. The family's model is in
[CONVENTIONS.md](../../../_shared/0.1/CONVENTIONS.md).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. The `did:webs` integration is **alpha**.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST** return only DIDs whose context the caller
may access, and **MUST NOT** signal the existence of those it filtered out: a `contextId`
the caller cannot reach answers an empty page, not an error. It **MUST NOT** list a
key-role identity, which this family does not manage, and **MUST** leave out deactivated
DIDs unless `includeDeactivated` is true.

An empty array is a **successful** answer. A caller that needs to know whether one DID
exists asks [`vta/webs/dids/get`](../../get/0.1/spec.md), which answers `notFound`: the
family ships the split pair the registry defaults to, because only `get` can say "no such
DID" as a determination rather than as an empty page.

Pages follow the registry's convention: `cursor` and `limit` in, `nextCursor` out, absent
on the last page.

## Authorization

Authority is **access to each listed DID's context**, applied per record: the answer is
the subset of the VTA's `did:webs` DIDs the caller may see. Listing needs no authority
beyond that.

Verifying the producer's VID or `proof` establishes *who is asking*, never *what they may do* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The context check follows, and is the authorization.

## Request

```json
{
  "id": "91403e62-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webs/dids/list/0.1",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-11T11:05:00Z",
  "payload": {
    "contextId": "alice",
    "limit": 50
  }
}
```

## Response

```json
{
  "id": "a2514f73-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webs/dids/list/0.1#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkOperator",
  "issuedAt": "2026-11-11T11:05:01Z",
  "threadId": "91403e62-0000-4000-8000-000000000001",
  "payload": {
    "dids": [
      {
        "did": "did:webs:example.com:people:alice:ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
        "aid": "ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
        "contextId": "alice",
        "serverId": "prod",
        "domain": "example.com",
        "slot": "people/alice/ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
        "keriCesrUrl": "https://example.com/people/alice/ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe/keri.cesr",
        "didJsonUrl": "https://example.com/people/alice/ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe/did.json",
        "sequenceNumber": 1,
        "versionId": "1:EHmLpA0OPWQbm6Ua2A8m2c4XUbJkfR7zTG4MBxwdXNbk",
        "signingKeyId": "webs-alice-sign-0",
        "signingKey": "DKxy2sgzfplyr-tgwIxS19f2OchFHtLwPWD3v4oYimBx",
        "nextKeyId": "webs-alice-sign-1",
        "keyAgreementKeyId": "webs-alice-ka-0",
        "keyAgreementKey": "CBwO9bU8Qk3n2YQm0xTNu0R1tq2Hk7f1pWx0aJc3Kd5e",
        "alsoKnownAs": [
          "did:webs:example.com:people:alice:ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe"
        ],
        "services": [],
        "deactivated": false,
        "createdAt": "2026-11-11T10:00:02Z",
        "updatedAt": "2026-11-11T10:00:02Z"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries filters and a page position. The response carries one record per DID
the caller may see, as [`get`](../../get/0.1/spec.md) describes: public keys and
designations, and the VTA's own bookkeeping — key ids, context, server id, label. No key
material.

### Correlation

A list is a join no resolver can make: it shows which DIDs the operator keeps together,
which is exactly what separate persona and relationship DIDs exist to hide from their
counterparties. The task returns it only to callers who may see each context, and an
operator **SHOULD** give list access to a context only to callers entitled to see every
identity in it.

### Retention

A read. The caller needs the answer only for as long as it acts on it.

### Consent/purpose

The list is returned to manage the VTA's `did:webs` DIDs, and **SHOULD NOT** be kept or
exported as a directory of the subject's identities.
