---
slug: consent/list
version: "1.1"
title: Consent — List
summary: A bridge fetches the consent grants it should enforce, so the steady-state inbound path is a local lookup with no per-message round-trip.
status: draft
targetFrameworkVersion: "0.5.0"
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
sideEffects:
  level: none
  rationale: "Read-only fetch of consent grants to enforce."
subjectPath: /subject
exposure:
  discloses: metadata
  actsAsSubject: false
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

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Changes from 1.0

The incremental-sync token gets names that say what it is. `since` is reserved
for an RFC 3339 lower-bound time filter, and `cursor`/`nextCursor` for page
continuation; this token is neither — it marks a position in the grant change
stream, and is present whether or not anything changed:

- Request **`since`** (optional opaque string) is renamed **`syncToken`**, with the same type and meaning: return only grants changed after it. A consumer **MUST NOT** accept a token it did not issue.
- Response **`cursor`** (optional string) is renamed **`nextSyncToken`**, with the same type and meaning: the token a bridge passes back as the request's `syncToken` on its next incremental fetch. It is omitted, never `null`, when the consumer has no token to offer.

No `limit` is added: the task does not page by size. Nothing else in the payload moves.

Breaking on the wire, released as a `MINOR` increment under [§5.2](/SPEC.md#52-compatibility-rules)'s `draft` allowance. `1.0` remains published.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the bridge) emits a document of type
`https://trusttasks.org/spec/consent/list/1.1`, optionally narrowing by `agent`,
`platform`, a full `subject`, or a `syncToken` (a previous response's `nextSyncToken`) for deltas.

A conforming **consumer** (the VTA) returns the matching grants the issuer is
authorized to see (else `notAuthorized`), newest decisions included, with an
optional `nextSyncToken` for incremental follow-up.

## Payload

`payload.agent` (OPTIONAL) — restrict to grants for this agent VID.
`payload.platform` (OPTIONAL) — restrict to this platform tag.
`payload.subject` (OPTIONAL) — a full [`ConsentSubject`](../../_shared/0.1/consent.schema.json) for a point-check.
`payload.syncToken` (OPTIONAL) — opaque token from a previous response's `nextSyncToken`; return only grants changed after it. The consumer MUST NOT accept a sync token it did not issue.
`payload.ext` — extension slot per [SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member).

## Examples

### Bridge syncs all grants for its agent

```json
{
  "id": "urn:uuid:consent-list-0001",
  "type": "https://trusttasks.org/spec/consent/list/1.1",
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
  "type": "https://trusttasks.org/spec/consent/list/1.1#response",
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
    "nextSyncToken": "c-2026-06-17T15:02:00Z"
  }
}
```

## Security & Privacy

### Data carried

The request carries optional filters — an agent VID, a platform tag, a full
`ConsentSubject` for a point-check — and an opaque `syncToken`. The response
carries [`ConsentGrant`](../../_shared/0.1/consent.schema.json) records: each
names a subject (platform, conversation reference, kind, agent), the effect and
scope, who granted it (`grantedBy`) and when, an optional expiry, and the
evidence kind. Grants carry opaque `conversationRef`s only. The optional `ext`
extension is part of the signed surface. The specification imposes no further
minimisation rule beyond the scoping below.

**Scoped reads.** The VTA returns only grants the requesting bridge is entitled
to enforce (its own agents).

### Correlation

The response joins an agent to the conversations it may (or may not) take part
in, and to the party that decided each grant and when. `threadId` correlates
request and response, and `nextSyncToken`/`syncToken` link a bridge's
successive incremental fetches. A consumer **MUST NOT** accept a sync token it
did not issue. The specification imposes no further rule on correlation.

### Retention

**Cache is advisory.** The bridge caches the grants locally (and persists the
cache) so the steady-state inbound path is a local lookup. The local cache is a
performance optimization; on an explicit point-check or a stale sync token the
bridge MUST defer to the VTA. The specification sets no retention period for the
cache beyond keeping it fresh after a `consent/decision` or `consent/revoke`.

### Consent/purpose

The grants are fetched so the bridge can enforce them — an Allow/Deny decision
on inbound traffic for its own agents. The specification states no other
purpose for them and imposes no further rule on their reuse.
