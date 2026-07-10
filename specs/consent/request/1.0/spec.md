---
slug: consent/request
version: "1.0"
title: Consent — Request
summary: A messaging bridge asks a Verifiable-Trust Agent whether an inbound conversation may proceed to an AI agent, prompting operator consent on first contact.
status: draft
targetFrameworkVersion: "0.2"
category: consent
keywords:
  - consent
  - messaging
  - bridge
  - gate
  - approval
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
  requirement: REQUIRED
  rationale: The request asserts which conversation, on which platform, would reach which agent — the basis of the operator's consent decision. A proof binds the request to the enrolled bridge so the VTA can trust the subject it is being asked to gate.
sideEffects:
  level: none
  rationale: "Asks whether an inbound conversation may proceed; prompts the operator, persists no decision itself."
subjectPath: /subject
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: consent/request:noApprover
    meaning: No approver is configured for this platform/context, so consent cannot be routed to a human.
    retryable: false
  - code: consent/request:subjectInvalid
    meaning: The subject is malformed, or names an agent the VTA does not recognise.
    retryable: false
  - code: consent/request:rateLimited
    meaning: The bridge has exceeded the VTA's pending-consent budget.
    retryable: true
related:
  - consent/decision
  - consent/revoke
  - consent/list
  - auth/step-up/approve-request
---

## Abstract

The **Consent — Request** Trust Task is how a messaging bridge (acting as a
Policy Enforcement Point) asks a Verifiable-Trust Agent to gate an inbound
conversation **before** it reaches an AI agent. It is the first half of the
consent flow; the operator's decision returns as a
[`consent/decision/1.0`](../../decision/1.0/spec.md).

The model is **default-deny**: a conversation with no recorded
[`ConsentGrant`](../../_shared/0.1/consent.schema.json) is denied, and the bridge
raises this request. The VTA mints a pending consent, resolves the **approver**
for the conversation's platform/context, and routes an approval prompt to that
operator (over a push/wake channel, or relayed back through the bridge to the
operator's own conversation). When the operator decides, a `consent/decision`
records the grant and the bridge releases (or drops) the held message.

This mirrors the [`auth/step-up/approve-request`](../../../auth/step-up/approve-request/0.2/spec.md)
pattern, specialized to "may this conversation reach this agent."

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the bridge) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/consent/request/1.0`, with itself as `issuer` and the VTA as `recipient`.
2. Populate `payload.subject` with the platform-agnostic [`ConsentSubject`](../../_shared/0.1/consent.schema.json) — using the OPAQUE `conversationRef`, never a raw platform address.
3. Populate `payload.scope` with the access it seeks (`receive` or `converse`).
4. Generate `payload.challenge` with ≥128 bits of entropy; the matching `consent/decision` echoes it so the bridge can correlate the decision to this request.
5. Hold the triggering inbound message (and any subsequent ones for the same subject) and NOT deliver them to the agent until an `allow` decision is recorded.
6. Include a verified `proof`.

A conforming **consumer** (the VTA) **MUST**:

1. Verify the document's `proof` against the `issuer`, and that the issuer is an enrolled bridge permitted to gate the named agent.
2. If a non-expired `ConsentGrant` already exists for `payload.subject`, return it (see `consent/list`) rather than re-prompting.
3. Otherwise mint a pending consent bound to `(subject, challenge, expiresAt)`, resolve the approver for the subject's platform/context, and route an approval prompt. If no approver is configured → `noApprover`.
4. Return a `#response` with `status: accepted` (a prompt was routed; a decision will follow) or `status: refused` (with a `reason`).

The decision arrives out-of-band as a `consent/decision`; the bridge MAY also poll `consent/list`.

## Payload

`payload.subject` (REQUIRED) — the [`ConsentSubject`](../../_shared/0.1/consent.schema.json).
`payload.scope` (REQUIRED) — requested [`Scope`](../../_shared/0.1/consent.schema.json).
`payload.challenge` (REQUIRED) — ≥128-bit nonce echoed by the decision.
`payload.displayHint` (OPTIONAL) — a redactable human label shown to the operator (e.g. "Signal group 'Family'"); MUST NOT carry a raw address.
`payload.firstMessageDigest` (OPTIONAL) — multihash digest of the held first message, binding the request to concrete content.
`payload.contextHint` (OPTIONAL) — the VTA context path the bridge runs under.
`payload.ext` — extension slot per [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member).

## Examples

### Bridge asks the VTA to gate a new Signal group

```json
{
  "id": "urn:uuid:consent-req-0001-0000-0000-000000000001",
  "type": "https://trusttasks.org/spec/consent/request/1.0",
  "issuer": "did:webvh:example:bridge",
  "recipient": "did:webvh:example:vta",
  "issuedAt": "2026-06-17T15:00:00Z",
  "payload": {
    "subject": {
      "platform": "signal",
      "conversationRef": "sig-1a2b3c4d",
      "kind": "group",
      "agent": "did:key:z6MkAgentExample"
    },
    "scope": "converse",
    "challenge": "Q29uc2VudENoYWxsZW5nZU5vbmNlWFla",
    "displayHint": "Signal group 'Family'"
  },
  "proof": { "…": "…" }
}
```

## Response

The `#response` acknowledges receipt; it is NOT the decision.

```json
{
  "id": "urn:uuid:consent-req-resp-0001",
  "type": "https://trusttasks.org/spec/consent/request/1.0#response",
  "threadId": "urn:uuid:consent-req-0001-0000-0000-000000000001",
  "issuer": "did:webvh:example:vta",
  "recipient": "did:webvh:example:bridge",
  "issuedAt": "2026-06-17T15:00:01Z",
  "payload": {
    "status": "accepted",
    "requestId": "consent-pending-7f3a"
  }
}
```

## Security & Privacy

**Opaque subjects.** `conversationRef` is opaque; the raw platform address never
reaches the VTA. `displayHint` is operator-facing and MUST be free of raw
identifiers.

**Default-deny.** Absence of a grant is a deny. The bridge MUST hold inbound for
an unknown subject rather than fail open.

**Challenge binding.** The VTA binds the challenge to the subject server-side so
a `consent/decision` cannot be replayed for a different conversation.

**Proof before prompt.** The VTA MUST verify the bridge's proof before routing an
approval prompt, so an unenrolled party cannot spam an operator.

The optional `ext` extension is part of the signed surface.
