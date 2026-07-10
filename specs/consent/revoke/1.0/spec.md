---
slug: consent/revoke
version: "1.0"
title: Consent — Revoke
summary: An operator revokes a standing consent grant so a previously-allowed messaging conversation no longer reaches the AI agent.
status: draft
targetFrameworkVersion: "0.2"
category: consent
keywords:
  - consent
  - revoke
  - withdraw
  - operator
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Approver (operator, or an enrolled bridge relaying the operator's choice)
    requirement: REQUIRED
    member: issuer
  - role: Verifiable-Trust Agent (consent authority)
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: Revocation withdraws a previously-granted authorization; it must be bound to an approver so a third party cannot silently cut off a conversation.
sideEffects:
  level: mutating
  rationale: "Revokes a standing consent grant; recoverable via consent/decision."
subjectPath: /subject
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: consent/revoke:notAuthorized
    meaning: The issuer is not an approver for this subject's platform/context.
    retryable: false
  - code: consent/revoke:notFound
    meaning: No grant exists for the subject.
    retryable: false
related:
  - consent/decision
  - consent/request
  - consent/list
---

## Abstract

The **Consent — Revoke** Trust Task withdraws a standing
[`ConsentGrant`](../../_shared/0.1/consent.schema.json). After revocation the
subject reverts to **default-deny**: the bridge stops delivering that
conversation to the agent and, on the next inbound, MAY raise a fresh
[`consent/request`](../../request/1.0/spec.md).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** **MUST** emit a document of type
`https://trusttasks.org/spec/consent/revoke/1.0` with `payload.subject` set to
the grant being revoked, and a verified `proof`.

A conforming **consumer** (the VTA) **MUST** verify the proof and the issuer's
authority over the subject (else `notAuthorized`), delete the grant (else
`notFound`), signal any bridge caching it, and return a `#response`.

## Payload

`payload.subject` (REQUIRED) — the [`ConsentSubject`](../../_shared/0.1/consent.schema.json) to revoke.
`payload.reason` (OPTIONAL) — operator note recorded in the audit trail.
`payload.ext` — extension slot per [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member).

## Examples

```json
{
  "id": "urn:uuid:consent-rev-0001",
  "type": "https://trusttasks.org/spec/consent/revoke/1.0",
  "issuer": "did:web:operator.example",
  "recipient": "did:webvh:example:vta",
  "issuedAt": "2026-06-18T09:00:00Z",
  "payload": {
    "subject": {
      "platform": "signal",
      "conversationRef": "sig-1a2b3c4d",
      "kind": "group",
      "agent": "did:key:z6MkAgentExample"
    },
    "reason": "Left the group."
  },
  "proof": { "…": "…" }
}
```

## Response

```json
{
  "id": "urn:uuid:consent-rev-resp-0001",
  "type": "https://trusttasks.org/spec/consent/revoke/1.0#response",
  "threadId": "urn:uuid:consent-rev-0001",
  "issuer": "did:webvh:example:vta",
  "recipient": "did:web:operator.example",
  "issuedAt": "2026-06-18T09:00:01Z",
  "payload": {
    "status": "revoked"
  }
}
```

## Security & Privacy

**Fail-closed.** Revocation moves the subject to default-deny; an ambiguous or
failed revoke MUST NOT leave a conversation open. The optional `ext` extension is
part of the signed surface.
