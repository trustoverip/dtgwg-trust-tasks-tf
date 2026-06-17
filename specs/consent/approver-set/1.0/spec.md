---
slug: consent/approver-set
version: "1.0"
title: Consent — Set Approver
summary: An admin binds the operator who approves inbound-messaging consent for a platform within a context, and how the prompt reaches them.
status: draft
targetFrameworkVersion: "0.2"
category: consent
keywords:
  - consent
  - approver
  - registry
  - routing
  - admin
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Context admin (operator)
    requirement: REQUIRED
    member: issuer
  - role: Verifiable-Trust Agent (consent authority)
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The binding decides who may approve a conversation reaching an agent; it must be bound to an admin so a third party cannot install themselves as the approver.
errorCodes:
  - code: consent/approver-set:notAuthorized
    meaning: The issuer is not an admin of the named context.
    retryable: false
  - code: consent/approver-set:invalidBinding
    meaning: The binding is malformed, or names an unknown context/approver.
    retryable: false
related:
  - consent/approver-list
  - consent/request
  - consent/decision
---

## Abstract

The **Consent — Set Approver** Trust Task records (upserts) an
[`ApproverBinding`](../../_shared/0.1/consent.schema.json): *who* approves
inbound-messaging consent for a given platform within a VTA context, and *how*
the consent prompt reaches them (`wake` → the approver's device signs a
DID-signed decision; `bridge-relay` → an enrolled bridge renders it, e.g. a card
in the operator's messaging app). [`consent/request`](../../request/1.0/spec.md)
resolves the binding to route a prompt to the right operator.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (an admin) **MUST** emit a document of type
`https://trusttasks.org/spec/consent/approver-set/1.0` with `payload.platform`,
`payload.context`, and `payload.approver` set, and a verified `proof`.

A conforming **consumer** (the VTA) **MUST** verify the proof and that the issuer
is an admin of `payload.context` (else `notAuthorized`), validate the binding
(else `invalidBinding`), upsert it keyed by `(platform, context)`, and return a
`#response`.

## Payload

`payload.platform` (REQUIRED) — messaging-platform tag.
`payload.context` (REQUIRED) — the VTA context path.
`payload.approver` (REQUIRED) — VID of the operator authorized to decide.
`payload.route` (OPTIONAL) — [`Route`](../../_shared/0.1/consent.schema.json) (`wake` / `bridge-relay`); defaults to `bridge-relay`.
`payload.routeHint` (OPTIONAL) — routing detail (e.g. the operator's opaque conversationRef).
`payload.ext` — extension slot per [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member).

## Examples

### Bind the Signal approver for a context

```json
{
  "id": "urn:uuid:consent-appset-0001",
  "type": "https://trusttasks.org/spec/consent/approver-set/1.0",
  "issuer": "did:web:operator.example",
  "recipient": "did:webvh:example:vta",
  "issuedAt": "2026-06-18T12:00:00Z",
  "payload": {
    "platform": "signal",
    "context": "vti-message-bridge",
    "approver": "did:web:operator.example",
    "route": "bridge-relay",
    "routeHint": "sig-0a1b2c3d"
  },
  "proof": { "…": "…" }
}
```

## Response

```json
{
  "id": "urn:uuid:consent-appset-resp-0001",
  "type": "https://trusttasks.org/spec/consent/approver-set/1.0#response",
  "threadId": "urn:uuid:consent-appset-0001",
  "issuer": "did:webvh:example:vta",
  "recipient": "did:web:operator.example",
  "issuedAt": "2026-06-18T12:00:01Z",
  "payload": {
    "status": "set"
  }
}
```

## Security & Privacy

**Admin-gated.** Only a context admin may set an approver — otherwise an attacker
could route consent prompts to themselves. **No raw addresses:** `routeHint`
carries an opaque conversationRef, not a phone number. The optional `ext`
extension is part of the signed surface.
