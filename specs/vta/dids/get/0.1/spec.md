---
slug: vta/dids/get
version: "0.1"
title: "VTA DIDs — Get"
summary: "Fetch one identity a VTA holds, whatever its DID method or key assurance level, as the summary vta/dids/list returns."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords:
  - vta
  - did
  - get
  - did method
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
  rationale: "Returns the VTA's summary of one identity: identifiers, context, class, level, the VTA's key ids and hosting server. No key material."
retention:
  class: transient
  rationale: "A read; nothing in it needs keeping beyond the caller's use of the answer."
errorCodes:
  - code: "vta/dids/get:notFound"
    meaning: "The VTA holds no record of this identity, or the caller cannot reach its context."
    retryable: false
related:
  - vta/dids/list
  - vta/dids/methods/list
---

## Abstract

**VTA DIDs — Get** returns one identity's `DidSummary`, the record
[`vta/dids/list`](../../list/0.1/spec.md) returns, whatever the identity's method. It is
the half of the split pair that can say *no such identity* as a determination
([CONVENTIONS.md §20](../../_shared/0.2/CONVENTIONS.md#20-every-identity-list-get-and-methods)).
A method's own record — a webvh log, a did:webs key event log — is read with that method's
`get`.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the VTA) **MUST** answer with the identity's summary when it
holds a record of it ([CONVENTIONS.md §20.1](../../_shared/0.2/CONVENTIONS.md#201-which-identities-are-summarised))
and the caller may reach its context, a deactivated identity included. It **MUST** answer
`vta/dids/get:notFound` both when it holds no record and when the caller cannot reach the
context, and **MUST NOT** let the two be told apart by the answer.

`did` may name either form of a `did:scid:vh:1` identity; the summary presents the form the
identity is presented in.

A conforming **producer** **MUST** ignore a `methodDetails` member it does not recognise.

## Definitions

- `did` — the identity to summarise.

## Authorization

Authority is **access to the identity's context**
([CONVENTIONS.md §20.2](../../_shared/0.2/CONVENTIONS.md#202-who-sees-what)). Reading
needs nothing beyond it.

Verifying the producer's VID or `proof` establishes *who is asking*, never *what they may do* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The context check follows, and is the authorization.

## Request

A caller asks the VTA; the payload is [`payload.schema.json`](payload.schema.json).

### One identity

```json
{
  "id": "urn:uuid:6c1d3f2b-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/dids/get/0.1",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-17T10:00:00Z",
  "payload": {
    "did": "did:webvh:QmAliceScid:example.com:alice"
  }
}
```

## Response

The VTA answers with the summary under `$anchor: "response"`. Failures use
`trust-task-error`.

### A did:webvh identity

```json
{
  "id": "urn:uuid:6c1d3f2b-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/dids/get/0.1#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkOperator",
  "issuedAt": "2026-11-17T10:00:01Z",
  "threadId": "urn:uuid:6c1d3f2b-0000-4000-8000-000000000001",
  "payload": {
    "did": {
      "did": "did:webvh:QmAliceScid:example.com:alice",
      "method": "webvh",
      "contextId": "alice",
      "level": 1,
      "primary": true,
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
    }
  }
}
```

## Security & Privacy

### Data carried

The request carries one DID. The response carries its summary: identifiers, the VTA's
bookkeeping and the method's public details. No key material.

### Correlation

A summary ties an identity to its context and to the VTA's key ids, which no resolver
sees. It is returned only to callers who may reach that context, and `notFound` does not
distinguish an identity the caller may not see from one that does not exist.

### Retention

A read. The caller needs the answer only for as long as it acts on it.

### Consent/purpose

The summary is returned to manage and act as the VTA's identities.
