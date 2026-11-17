---
slug: vta/dids/methods/list
version: "0.1"
title: "VTA DIDs — Methods"
summary: "Ask a VTA which DID methods it holds, lists and will create now, which DID classes each creation may state, and which tasks serve each method's identities."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords:
  - vta
  - did
  - did method
  - capabilities
parties:
  - role: Caller
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: OPTIONAL
  rationale: "A read of the VTA's own capabilities, with no state change."
sideEffects:
  level: none
  rationale: "A read of configuration."
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: none
  rationale: "Returns which DID methods the VTA supports and how. It names no identity."
retention:
  class: transient
  rationale: "A read; a caller may cache it for the life of its session."
errorCodes: []
related:
  - vta/dids/list
  - vta/dids/get
---

## Abstract

**VTA DIDs — Methods** tells a caller what the VTA does with each DID method it holds a
profile for: whether `vta/dids/list` summarises its identities, whether the VTA will
create one now, the DID classes a creation may state, and the task types that serve its
identities. A client decides what to offer from this answer rather than from a DID
string's prefix, so a method added to the VTA reaches it without a client release
([CONVENTIONS.md §20.4](../../../_shared/0.2/CONVENTIONS.md#204-methods)).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the VTA) **MUST** report every method it holds a profile for,
one entry each, and **MUST** report `creatable` as it stands now — the build and the
configuration together — so a method compiled in but switched off is reported `false`.
`operations` **MUST** name only task types the VTA serves for that method's identities;
this family's own tasks are not repeated there.

A conforming **producer** **SHOULD** treat the answer as current for its session, and
**MUST NOT** infer from an absent method that identities of it do not exist — only that the
VTA does not list them.

## Definitions

The request has no members beyond `ext`.

## Authorization

Any authenticated caller. The answer describes the VTA's configuration, not any context's
contents.

Verifying the producer's VID or `proof` establishes *who is asking*, never *what they may do* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)).

## Request

A caller asks the VTA; the payload is [`payload.schema.json`](payload.schema.json).

### The VTA's methods

```json
{
  "id": "urn:uuid:7d2e4a3c-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/dids/methods/list/0.1",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:webvh:QmVtaScid:vta.example",
  "issuedAt": "2026-11-17T10:00:00Z",
  "payload": {}
}
```

## Response

The VTA answers under `$anchor: "response"`. Failures use `trust-task-error`.

### did:webvh and did:key, with did:webs switched off

```json
{
  "id": "urn:uuid:7d2e4a3c-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/dids/methods/list/0.1#response",
  "issuer": "did:webvh:QmVtaScid:vta.example",
  "recipient": "did:key:z6MkOperator",
  "issuedAt": "2026-11-17T10:00:01Z",
  "threadId": "urn:uuid:7d2e4a3c-0000-4000-8000-000000000001",
  "payload": {
    "methods": [
      {
        "method": "webvh",
        "listed": true,
        "creatable": true,
        "classes": [],
        "operations": [
          "https://trusttasks.org/spec/vta/webvh/dids/create/1.2",
          "https://trusttasks.org/spec/vta/webvh/dids/get/1.0",
          "https://trusttasks.org/spec/vta/webvh/dids/update/1.1",
          "https://trusttasks.org/spec/vta/webvh/dids/rotate-keys/1.0",
          "https://trusttasks.org/spec/vta/webvh/dids/delete/1.0"
        ]
      },
      {
        "method": "scid:vh",
        "listed": true,
        "creatable": true,
        "classes": [],
        "operations": [
          "https://trusttasks.org/spec/vta/webvh/dids/create/1.2",
          "https://trusttasks.org/spec/vta/webvh/dids/get/1.0"
        ]
      },
      {
        "method": "webs",
        "listed": true,
        "creatable": false,
        "classes": [],
        "operations": [
          "https://trusttasks.org/spec/vta/webs/dids/get/0.1",
          "https://trusttasks.org/spec/vta/webs/dids/delete/0.1"
        ]
      },
      {
        "method": "key",
        "listed": true,
        "creatable": false,
        "classes": [],
        "operations": []
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The response carries the VTA's DID-method configuration. No identity, key or context.

### Correlation

The configuration is the same for every caller and says nothing about any holder.

### Retention

A caller may keep the answer for its session and **SHOULD** ask again in a new one, since
the configuration can change.

### Consent/purpose

The answer is for deciding what a client offers. It carries no personal data.
