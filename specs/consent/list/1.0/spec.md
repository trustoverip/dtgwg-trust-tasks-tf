---
slug: consent/list
version: "1.0"
title: Consent — List
summary: A bridge fetches the consent grants it should enforce, so the steady-state inbound path is a local lookup with no per-message round-trip.
status: draft
targetFrameworkVersion: "0.2"
category: consent
keywords:
  - consent
  - list
  - sync
  - grants
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Messaging bridge (Policy Enforcement Point)
    requirement: REQUIRED
    member: issuer
  - role: Verifiable-Trust Agent (consent authority)
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: A read-only sync. A proof lets the VTA scope the returned grants to the requesting bridge's agents, but the grants are not themselves mutated.
errorCodes:
  - code: consent/list:notAuthorized
    meaning: The issuer may not read grants for the requested agent/platform.
    retryable: false
related:
  - consent/request
  - consent/decision
  - consent/revoke
---

## Abstract

The **Consent — List** Trust Task lets a bridge fetch the
[`ConsentGrant`](../../_shared/0.1/consent.schema.json) records it must enforce.
The bridge caches them locally (and persists the cache) so the steady-state
inbound path is a local Allow/Deny lookup; an out-of-band signal (or a periodic
`consent/list`) keeps the cache fresh after a `consent/decision` or
`consent/revoke`. With a full `subject` filter it also serves as a point-check
for a single conversation.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the bridge) emits a document of type
`https://trusttasks.org/spec/consent/list/1.0`, optionally narrowing by `agent`,
`platform`, a full `subject`, or a `since` cursor for deltas.

A conforming **consumer** (the VTA) returns the matching grants the issuer is
authorized to see (else `notAuthorized`), newest decisions included, with an
optional `cursor` for incremental follow-up.

## Payload

`payload.agent` (OPTIONAL) — restrict to grants for this agent VID.
`payload.platform` (OPTIONAL) — restrict to this platform tag.
`payload.subject` (OPTIONAL) — a full [`ConsentSubject`](../../_shared/0.1/consent.schema.json) for a point-check.
`payload.since` (OPTIONAL) — opaque cursor; return only grants changed after it.
`payload.ext` — extension slot per [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member).

## Examples

### Bridge syncs all grants for its agent

```json
{
  "id": "urn:uuid:consent-list-0001",
  "type": "https://trusttasks.org/spec/consent/list/1.0",
  "issuer": "did:webvh:example:bridge",
  "recipient": "did:webvh:example:vta",
  "issuedAt": "2026-06-18T10:00:00Z",
  "payload": {
    "agent": "did:key:z6MkAgentExample"
  },
  "proof": { "…": "…" }
}
```

## Response

```json
{
  "id": "urn:uuid:consent-list-resp-0001",
  "type": "https://trusttasks.org/spec/consent/list/1.0#response",
  "threadId": "urn:uuid:consent-list-0001",
  "issuer": "did:webvh:example:vta",
  "recipient": "did:webvh:example:bridge",
  "issuedAt": "2026-06-18T10:00:01Z",
  "payload": {
    "grants": [
      {
        "subject": {
          "platform": "signal",
          "conversationRef": "sig-1a2b3c4d",
          "kind": "group",
          "agent": "did:key:z6MkAgentExample"
        },
        "effect": "allow",
        "scope": "converse",
        "grantedBy": "did:web:operator.example",
        "grantedAt": "2026-06-17T15:02:00Z",
        "evidence": "bridge-attested"
      }
    ],
    "cursor": "c-2026-06-17T15:02:00Z"
  }
}
```

## Security & Privacy

**Scoped reads.** The VTA returns only grants the requesting bridge is entitled
to enforce (its own agents). Grants carry opaque `conversationRef`s only.

**Cache is advisory.** The local cache is a performance optimization; on an
explicit point-check or a stale cursor the bridge MUST defer to the VTA. The
optional `ext` extension is part of the signed surface.
