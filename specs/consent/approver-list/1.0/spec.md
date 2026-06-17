---
slug: consent/approver-list
version: "1.0"
title: Consent — List Approvers
summary: Fetch the approver bindings that decide inbound-messaging consent, optionally filtered by platform or context.
status: draft
targetFrameworkVersion: "0.2"
category: consent
keywords:
  - consent
  - approver
  - registry
  - list
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Admin or enrolled bridge
    requirement: REQUIRED
    member: issuer
  - role: Verifiable-Trust Agent (consent authority)
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: A read-only query. A proof lets the VTA scope the returned bindings to what the issuer may see, but they are not mutated.
errorCodes:
  - code: consent/approver-list:notAuthorized
    meaning: The issuer may not read approver bindings for the requested platform/context.
    retryable: false
related:
  - consent/approver-set
  - consent/request
---

## Abstract

The **Consent — List Approvers** Trust Task returns the
[`ApproverBinding`](../../_shared/0.1/consent.schema.json) records the VTA holds,
optionally narrowed by `platform` or `context`. Operator tooling uses it to show
the current routing; the VTA itself resolves bindings internally during
[`consent/request`](../../request/1.0/spec.md).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** emits a document of type
`https://trusttasks.org/spec/consent/approver-list/1.0`, optionally narrowing by
`platform` and/or `context`.

A conforming **consumer** (the VTA) returns the matching bindings the issuer is
authorized to see (else `notAuthorized`).

## Payload

`payload.platform` (OPTIONAL) — restrict to this platform tag.
`payload.context` (OPTIONAL) — restrict to this context path.
`payload.ext` — extension slot per [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member).

## Examples

```json
{
  "id": "urn:uuid:consent-applist-0001",
  "type": "https://trusttasks.org/spec/consent/approver-list/1.0",
  "issuer": "did:web:operator.example",
  "recipient": "did:webvh:example:vta",
  "issuedAt": "2026-06-18T12:05:00Z",
  "payload": {
    "context": "vti-message-bridge"
  },
  "proof": { "…": "…" }
}
```

## Response

```json
{
  "id": "urn:uuid:consent-applist-resp-0001",
  "type": "https://trusttasks.org/spec/consent/approver-list/1.0#response",
  "threadId": "urn:uuid:consent-applist-0001",
  "issuer": "did:webvh:example:vta",
  "recipient": "did:web:operator.example",
  "issuedAt": "2026-06-18T12:05:01Z",
  "payload": {
    "approvers": [
      {
        "platform": "signal",
        "context": "vti-message-bridge",
        "approver": "did:web:operator.example",
        "route": "bridge-relay",
        "routeHint": "sig-0a1b2c3d"
      }
    ]
  }
}
```

## Security & Privacy

**Scoped reads.** The VTA returns only bindings the requester is entitled to see.
Bindings carry an opaque `routeHint`, never a raw address. The optional `ext`
extension is part of the signed surface.
