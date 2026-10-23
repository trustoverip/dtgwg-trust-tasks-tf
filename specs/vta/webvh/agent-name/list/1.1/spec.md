---
slug: vta/webvh/agent-name/list
version: "1.1"
title: "VTA WebVH Agent-Name — List"
summary: "List every agent name a DID holds, enabled or not."
status: draft
targetFrameworkVersion: "0.5.0"
category: did-management
keywords:
  - vta
  - webvh
  - agent-name
  - list
  - did
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
  rationale: "A read with no durable state change."
sideEffects:
  level: none
  rationale: "Enumeration only."
subjectPath: /did
exposure:
  discloses: metadata
  actsAsSubject: false
errorCodes:
  - code: "vta/webvh/agent-name/list:notFound"
    meaning: "The DID is unknown to this VTA or unreachable by the caller."
    retryable: false
related:
  - vta/webvh/agent-name/set
  - vta/webvh/agent-name/check
  - vta/webvh/agent-name/disable
---

## Abstract

**VTA WebVH Agent-Name — List** returns every name a DID holds, including
disabled ones.

An agent name is a human-readable handle — `example.com/@alice` — that resolves
to a DID. The direction matters: name → DID is an HTTP redirect served by the
name's own host and proves nothing on its own, because whoever controls a domain
can point a name at somebody else's DID. Only the DID's controller can add the
matching `alsoKnownAs` entry, so **DID → name is the authoritative direction**.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Changes from 1.0

One member of each `names[]` item (`AgentNameEntry`) changes type:

* **`createdAt`**: integer Unix seconds (`minimum: 0`) → RFC 3339 string (`type: "string"`, `format: "date-time"`), e.g. `1767225600` → `"2026-01-01T00:00:00Z"`. It keeps its name and remains optional.

The registry-wide convention is that every instant on the wire is an RFC 3339 string in a member named `*At`; this was an integer instant in the shared entry.

The change arrives through the shared component: this version re-pins its `$ref` into `webvh-records.schema.json` from `vta/_shared/0.1` to `vta/_shared/0.7`. `name`, `enabled`, `ext` and the request payload are unchanged.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `1.0` remains published and pinned to `vta/_shared/0.1`.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST** include disabled names, marked as
such. A disabled name is still held and cannot be taken by another caller, so
omitting it would misrepresent what is available.

## Authorization

Authority is **access to the DID's context** for reads, and the **administrator
role over it** for changes: a name speaks for the DID, so the authority that
governs the DID governs its names.

The binding this task records is one half of a claim. It becomes authoritative
only when the DID document claims the name back through `alsoKnownAs`; a
consumer **MUST NOT** treat a name recorded here as proof of the binding.

Verifying the producer's VID or `proof` establishes *who is asking*, never *what they may do* ([SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements)). The role check follows, and is the authorization.

## Request

```json
{
  "id": "1a2b3c4d-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vta/webvh/agent-name/list/1.1",
  "issuer": "did:key:z6MkOperator",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T11:00:00Z",
  "payload": { "did": "did:webvh:QmScidAbCdEfGh:example.com:alice" }
}
```

## Response

```json
{
  "id": "2b3c4d5e-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webvh/agent-name/list/1.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkOperator",
  "issuedAt": "2026-08-19T11:00:01Z",
  "threadId": "1a2b3c4d-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmScidAbCdEfGh:example.com:alice",
    "names": [
      { "name": "alice", "enabled": true, "createdAt": "2026-01-01T00:00:00Z" },
      { "name": "alice-old", "enabled": false, "createdAt": "2025-07-01T00:00:00Z" }
    ]
  }
}
```

## Security & Privacy

### Data carried

The request carries one DID. The response carries that DID and every agent name
held for it, enabled or not, each with its `enabled` flag and, where recorded,
the instant it was created. No key material is carried.

### Correlation

The names a DID holds link its identities together: a caller who knows one name
learns the others, including ones deliberately retired. Where that correlation
matters, hold names under separate DIDs rather than separate names on one.

### Retention

This task is a read and changes no durable state. The names it reports are held
by the VTA until removed; a disabled name is still held and cannot be taken by
another caller, which is why it is listed.

### Consent/purpose

Authority to read is access to the DID's context — see
[Authorization](#authorization). A name recorded here is one half of a claim and
is not proof of the binding; that comes only from the DID document's
`alsoKnownAs`.
