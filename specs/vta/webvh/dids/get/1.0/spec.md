---
slug: vta/webvh/dids/get
version: "1.0"
title: "VTA WebVH DIDs — Get"
summary: "Fetch one did:webvh record the VTA holds, optionally with its full log."
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - webvh
  - did
  - get
  - log
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
  rationale: "A read with no durable state change; the transport's authenticated sender is what the VTA resolves scope against."
sideEffects:
  level: none
  rationale: "Single-record read."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: "vta/webvh/dids/get:notFound"
    meaning: "No such DID is held by this VTA, or the caller cannot reach its context."
    retryable: false
related:
  - vta/webvh/dids/list
  - vta/webvh/dids/create
  - vta/webvh/dids/update
---

## Abstract

**VTA WebVH DIDs — Get** returns what the VTA holds about one `did:webvh`, and
on request the DID's log.

The record is the VTA's bookkeeping. The **log** is the DID itself: the
append-only history a resolver replays to arrive at the current document, and
the only thing against which that document can be checked.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. It documents a task already deployed at this version, written down after the fact.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST** return the log only when
`includeLog` is true, and **MUST NOT** treat its absence from a response as
information about the DID's history.

A conforming **consumer** **MUST** answer `vta/webvh/dids/get:notFound` for a
DID the caller cannot reach, whether or not the VTA holds it.

## Authorization

Authority is **access to the DID's context** — the same resolution that gates
the keys the DID is signed with.

Verifying the producer's VID or `proof` establishes *who is asking*, never *what they may do* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The role check follows, and is the authorization.

## Request

```json
{
  "id": "1a2b3c4d-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/get/1.0",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T11:00:00Z",
  "payload": {
    "did": "did:webvh:QmScidAbCdEfGh:example.com:alice",
    "includeLog": true
  }
}
```

## Response

```json
{
  "id": "2b3c4d5e-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webvh/dids/get/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOperator",
  "issuedAt": "2026-08-19T11:00:01Z",
  "threadId": "1a2b3c4d-0000-4000-8000-000000000001",
  "payload": {
    "record": {
      "did": "did:webvh:QmScidAbCdEfGh:example.com:alice",
      "serverId": "prod",
      "mnemonic": "alice",
      "scid": "QmScidAbCdEfGh",
      "contextId": "personal",
      "portable": true,
      "logEntryCount": 4,
      "preRotationCount": 3,
      "nextFragmentId": 5,
      "createdAt": "2026-08-19T11:00:01Z",
      "updatedAt": "2026-08-19T12:30:00Z"
    },
    "log": "{\"versionId\":\"1-QmScid…\"}\n{\"versionId\":\"2-…\"}\n"
  }
}
```

## Security & Privacy

The log is a complete history: every key this DID has ever published, every
service it has advertised, and when each changed. It is public information by
design — a resolver fetches it — but returning it here hands a caller the whole
record in one response, which is why it is opt-in rather than always present.
