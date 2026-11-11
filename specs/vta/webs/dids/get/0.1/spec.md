---
slug: vta/webs/dids/get
version: "0.1"
title: "VTA did:webs DIDs — Get"
summary: "Fetch one did:webs DID the VTA holds — its keys, designations, host and state — optionally with its keri.cesr stream. Alpha."
status: draft
targetFrameworkVersion: "0.5.0"
category: did-management
keywords:
  - vta
  - did-webs
  - keri
  - did
  - get
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
  rationale: "A read with no state change; the transport's authenticated sender is what the VTA resolves the caller's scope against."
sideEffects:
  level: none
  rationale: "Single-record read."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: "Returns the VTA's record of the DID — public keys, the VTA's key ids, the host and slot — and on request the published stream. No key material."
retention:
  class: transient
  rationale: "A read; nothing in it needs keeping beyond the caller's use of the answer."
errorCodes:
  - code: "vta/webs/dids/get:notFound"
    meaning: "No such did:webs DID is held by this VTA, or the caller cannot reach its context."
    retryable: false
related:
  - vta/webs/dids/list
  - vta/webs/dids/create
  - vta/webs/dids/update
---

## Abstract

**VTA did:webs DIDs — Get** returns what the VTA holds about one `did:webs` DID: its
AID, context, host and slot, the current signing key and its committed successor, the
designated key agreement key, aliases and services, its latest event, and whether it is
deactivated. On request it also returns the `keri.cesr` stream, which *is* the DID: the
history a resolver verifies to arrive at the document the host serves. The family's
model is in [CONVENTIONS.md](../../../_shared/0.1/CONVENTIONS.md).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. The `did:webs` integration is **alpha**.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST**:

1. Answer `notFound` alike for a DID it does not hold, one in a context the caller cannot
   reach, and a key-role identity under `did:webs`, which this family does not manage.
2. Answer from its own record, without resolving the DID or reading the host. The record
   is the controller's state, and the host's copy is what it published from it.
3. Return `keriCesr` only when `includeKeriCesr` is true.
4. Report `rotationStaged` when a rotation is staged and not committed, since the host may
   then already serve a key state the record does not yet name
   ([CONVENTIONS.md §6](../../../_shared/0.1/CONVENTIONS.md#6-remote-first)).

A deactivated DID is answered like any other, with `deactivated` true: the record and its
final stream are kept as evidence.

## Authorization

Authority is **access to the DID's context**: the same resolution that gates the keys the
DID is signed with. A caller that may see a context may read its DIDs.

Verifying the producer's VID or `proof` establishes *who is asking*, never *what they may do* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The context check follows, and is the authorization.

## Request

```json
{
  "id": "7f2e1c40-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webs/dids/get/0.1",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-11T11:00:00Z",
  "payload": {
    "did": "did:webs:example.com:people:alice:ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe"
  }
}
```

## Response

```json
{
  "id": "803f2d51-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webs/dids/get/0.1#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkOperator",
  "issuedAt": "2026-11-11T11:00:01Z",
  "threadId": "7f2e1c40-0000-4000-8000-000000000001",
  "payload": {
    "record": {
      "did": "did:webs:example.com:people:alice:ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
      "aid": "ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
      "contextId": "alice",
      "serverId": "prod",
      "domain": "example.com",
      "slot": "people/alice/ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
      "keriCesrUrl": "https://example.com/people/alice/ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe/keri.cesr",
      "didJsonUrl": "https://example.com/people/alice/ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe/did.json",
      "label": "Alice — persona",
      "sequenceNumber": 1,
      "versionId": "1:EHmLpA0OPWQbm6Ua2A8m2c4XUbJkfR7zTG4MBxwdXNbk",
      "signingKeyId": "webs-alice-sign-0",
      "signingKey": "DKxy2sgzfplyr-tgwIxS19f2OchFHtLwPWD3v4oYimBx",
      "nextKeyId": "webs-alice-sign-1",
      "keyAgreementKeyId": "webs-alice-ka-0",
      "keyAgreementKey": "CBwO9bU8Qk3n2YQm0xTNu0R1tq2Hk7f1pWx0aJc3Kd5e",
      "alsoKnownAs": [
        "did:webs:example.com:people:alice:ENro7uf0ePmiK3jdTo2YCdXLqW7z7xoP6qhhBou6gBLe",
        "did:webvh:QmAliceScid:example.com:alice"
      ],
      "services": [
        {
          "id": "#didcomm",
          "type": "DIDCommMessaging",
          "serviceEndpoint": { "uri": "did:webvh:QmMediatorScid:mediator.example", "accept": ["didcomm/v2"] }
        },
        {
          "id": "#tsp",
          "type": "TSPTransport",
          "serviceEndpoint": "did:webvh:QmMediatorScid:mediator.example"
        }
      ],
      "deactivated": false,
      "createdAt": "2026-11-11T10:00:02Z",
      "updatedAt": "2026-11-11T10:00:02Z"
    }
  }
}
```

## Security & Privacy

### Data carried

The request names one DID. The response carries the VTA's record of it — public keys,
the VTA's ids for its keys, the host, domain and slot, the designations — and, on
request, the published stream. No key material is returned. Everything in the record but
the key ids, the context, the server id and the label is public, since a resolver fetches
it from the host; those four are the VTA's own bookkeeping and say how the operator has
organised its identities.

### Correlation

The record names the context a DID belongs to, so a caller who can read two DIDs' records
learns whether one context holds both, which a resolver of the two DIDs cannot. That is
why the task answers only for contexts the caller may reach, and answers `notFound`
identically otherwise.

### Retention

A read. The caller needs the answer only for as long as it acts on it; the stream it may
carry is public, and is republished on every change, so a kept copy goes stale.

### Consent/purpose

The record is returned to manage the DID: to show an operator its state, to drive an
update or a rotation (`versionId` is the precondition they take), or to audit what was
published. It is not a resolution service, and a caller **SHOULD NOT** use it in place of
resolving a DID it does not manage.
