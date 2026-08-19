---
slug: vta/webvh/agent-name/check
version: "1.0"
title: "VTA WebVH Agent-Name — Check"
summary: "Ask whether an agent name is free on a host before trying to take it."
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - webvh
  - agent-name
  - check
  - availability
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
  rationale: "A read with no durable state change. It reserves nothing."
sideEffects:
  level: none
  rationale: "Availability query; nothing is held or reserved."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes: []
related:
  - vta/webvh/agent-name/set
  - vta/webvh/agent-name/list
---

## Abstract

**VTA WebVH Agent-Name — Check** reports whether a name can be taken on the
host, and whether it is withheld by the host's own policy.

An agent name is a human-readable handle — `example.com/@alice` — that resolves
to a DID. The direction matters: name → DID is an HTTP redirect served by the
name's own host and proves nothing on its own, because whoever controls a domain
can point a name at somebody else's DID. Only the DID's controller can add the
matching `alsoKnownAs` entry, so **DID → name is the authoritative direction**.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. It documents a task already deployed at this version, written down after the fact.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST NOT** reserve, hold or otherwise
affect the name in the course of answering. The answer is a snapshot: another
caller may take the name between this response and an attempt to set it.

`reserved` **MUST** be distinguished from merely unavailable — a reserved name is
withheld by host policy and will not become free by waiting.

## Authorization

Any authenticated caller with access to the DID's context may ask. The task
discloses only whether a name is free on a host, which the host's own redirect
already reveals to anyone who tries it.

## Request

```json
{
  "id": "1a2b3c4d-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webvh/agent-name/check/1.0",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T11:00:00Z",
  "payload": {
    "did": "did:webvh:QmScidAbCdEfGh:example.com:alice",
    "name": "support"
  }
}
```

## Response

```json
{
  "id": "2b3c4d5e-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webvh/agent-name/check/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOperator",
  "issuedAt": "2026-08-19T11:00:01Z",
  "threadId": "1a2b3c4d-0000-4000-8000-000000000001",
  "payload": {
    "name": "support",
    "domain": "example.com",
    "available": false,
    "reserved": true
  }
}
```

## Security & Privacy

Because the answer reserves nothing, a caller that checks and then sets can lose
the name in between. A consumer **MUST NOT** present a successful check as a
guarantee, and a caller that needs the name should be prepared for the set to
fail.
