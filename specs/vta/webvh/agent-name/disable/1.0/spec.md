---
slug: vta/webvh/agent-name/disable
version: "1.0"
title: "VTA WebVH Agent-Name — Disable"
summary: "An administrator stops an agent name resolving while continuing to hold it."
status: draft
targetFrameworkVersion: "0.2"
category: did-management
keywords:
  - vta
  - webvh
  - agent-name
  - disable
  - did
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: "A name speaks for a DID to anyone who resolves it; the VTA must attribute a change to a specific administrator independently of the transport."
sideEffects:
  level: mutating
  rationale: "Stops the name resolving. The VTA keeps holding it."
subjectPath: /did
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: "vta/webvh/agent-name/disable:notFound"
    meaning: "The DID is unknown to this VTA, unreachable by the caller, or does not hold this name."
    retryable: false
related:
  - vta/webvh/agent-name/list
  - vta/webvh/agent-name/check
  - vta/webvh/agent-name/set
  - vta/webvh/agent-name/remove
---

## Abstract

**VTA WebVH Agent-Name — Disable** stops a name resolving without releasing it.

An agent name is a human-readable handle — `example.com/@alice` — that resolves
to a DID. The direction matters: name → DID is an HTTP redirect served by the
name's own host and proves nothing on its own, because whoever controls a domain
can point a name at somebody else's DID. Only the DID's controller can add the
matching `alsoKnownAs` entry, so **DID → name is the authoritative direction**.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. It documents a task already deployed at this version, written down after the fact.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) apply.

A conforming **consumer** (the VTA) **MUST** continue to hold a disabled name —
it is not available to other callers, and a `check` for it **MUST** report it as
unavailable.

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
  "type": "https://trusttasks.org/spec/vta/webvh/agent-name/disable/1.0",
  "issuer": "did:key:z6MkAdmin",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T11:00:00Z",
  "payload": {
    "did": "did:webvh:QmScidAbCdEfGh:example.com:alice",
    "name": "alice"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-19T11:00:00Z",
    "verificationMethod": "did:key:z6MkAdmin#z6MkAdmin",
    "proofPurpose": "assertionMethod",
    "proofValue": "z3FXQ..."
  }
}
```

## Response

```json
{
  "id": "2b3c4d5e-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vta/webvh/agent-name/disable/1.0#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:web:vta.example",
  "issuedAt": "2026-08-19T11:00:01Z",
  "threadId": "1a2b3c4d-0000-4000-8000-000000000001",
  "payload": {
    "did": "did:webvh:QmScidAbCdEfGh:example.com:alice",
    "name": "alice",
    "enabled": false
  }
}
```

## Security & Privacy

Disabling is the safe counterpart to removal: the name stops working and
nobody else can take it, so a link that used to resolve fails rather than
reaching a stranger. Prefer it wherever the name has been published.
