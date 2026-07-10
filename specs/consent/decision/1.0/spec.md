---
slug: consent/decision
version: "1.0"
title: Consent — Decision
summary: An approver allows or denies an AI agent's access to a messaging conversation, recording a consent grant at the Verifiable-Trust Agent.
status: draft
targetFrameworkVersion: "0.2"
category: consent
keywords:
  - consent
  - decision
  - grant
  - approval
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
  rationale: The decision IS the authorization that lets a conversation reach an agent. The proof binds it to the approver — the operator's DID for a directly-signed decision, or an enrolled bridge's DID when it attests the operator's out-of-band choice.
sideEffects:
  level: mutating
  rationale: "Records a consent grant at the agent; recoverable via consent/revoke."
subjectPath: /subject
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: consent/decision:notAuthorized
    meaning: The issuer is not an approver for this subject's platform/context.
    retryable: false
  - code: consent/decision:challengeMismatch
    meaning: The echoed challenge does not match any pending consent request for the subject.
    retryable: false
  - code: consent/decision:subjectInvalid
    meaning: The subject is malformed or unknown.
    retryable: false
related:
  - consent/request
  - consent/revoke
  - consent/list
  - auth/step-up/approve-response
---

## Abstract

The **Consent — Decision** Trust Task records an approver's `allow` or `deny`
over a [`ConsentSubject`](../../_shared/0.1/consent.schema.json) — the second
half of the flow started by [`consent/request/1.0`](../../request/1.0/spec.md).
On `allow`, the VTA stores a [`ConsentGrant`](../../_shared/0.1/consent.schema.json)
and signals the bridge to release the held conversation; on `deny`, the held
messages are dropped.

Two issuer modes share this one type (the `evidence` field on the resulting
grant records which was used):

- **Directly signed** — the operator's own DID issues and signs the decision (a
  verifiable presentation of "I consent"). Strongest; non-repudiable.
- **Bridge-attested** — an enrolled bridge issues the decision on the operator's
  behalf, attesting an out-of-band choice (e.g. the operator replied to a prompt
  in their own messaging app). The VTA trusts the bridge's enrollment; weaker
  than a directly-signed operator decision, and intended as a migration step
  toward direct signing.

This mirrors [`auth/step-up/approve-response`](../../../auth/step-up/approve-response/0.2/spec.md).

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the approver) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/consent/decision/1.0`, with the approver as `issuer` and the VTA as `recipient`.
2. Populate `payload.subject` with the exact subject being decided.
3. Set `payload.effect` to `allow` or `deny`. When `allow`, set `payload.scope`.
4. Echo `payload.challenge` from the `consent/request` it answers (omit only for an operator-initiated pre-authorization, which the VTA MAY accept per policy).
5. Include a verified `proof` (the operator's key for a directly-signed decision; the bridge's key for a bridge-attested one).

A conforming **consumer** (the VTA) **MUST**:

1. Verify the `proof` and that the `issuer` is an authorized approver for the subject's platform/context (else `notAuthorized`).
2. When a `challenge` is present, match it to a pending consent for the subject (else `challengeMismatch`); consume the pending entry (single-use).
3. Record a `ConsentGrant` (`evidence` reflecting the issuer mode) — or, for `deny`, a deny grant — and signal any waiting bridge.
4. Return a `#response` with `status: recorded` and the `grantId`, or `rejected` with a `reason`.

## Payload

`payload.subject` (REQUIRED) — the [`ConsentSubject`](../../_shared/0.1/consent.schema.json).
`payload.effect` (REQUIRED) — `allow` or `deny`.
`payload.scope` (REQUIRED when `effect` is `allow`) — granted [`Scope`](../../_shared/0.1/consent.schema.json).
`payload.challenge` (RECOMMENDED) — echoes the `consent/request` challenge; omit only for pre-authorization.
`payload.expiresAt` (OPTIONAL) — grant TTL; after it, the subject must be re-consented.
`payload.ext` — extension slot per [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member).

## Examples

### Operator allows the Signal group, converse scope

```json
{
  "id": "urn:uuid:consent-dec-0001",
  "type": "https://trusttasks.org/spec/consent/decision/1.0",
  "threadId": "urn:uuid:consent-req-0001-0000-0000-000000000001",
  "issuer": "did:web:operator.example",
  "recipient": "did:webvh:example:vta",
  "issuedAt": "2026-06-17T15:02:00Z",
  "payload": {
    "subject": {
      "platform": "signal",
      "conversationRef": "sig-1a2b3c4d",
      "kind": "group",
      "agent": "did:key:z6MkAgentExample"
    },
    "effect": "allow",
    "scope": "converse",
    "challenge": "Q29uc2VudENoYWxsZW5nZU5vbmNlWFla"
  },
  "proof": { "…": "…" }
}
```

## Response

```json
{
  "id": "urn:uuid:consent-dec-resp-0001",
  "type": "https://trusttasks.org/spec/consent/decision/1.0#response",
  "threadId": "urn:uuid:consent-dec-0001",
  "issuer": "did:webvh:example:vta",
  "recipient": "did:web:operator.example",
  "issuedAt": "2026-06-17T15:02:01Z",
  "payload": {
    "status": "recorded",
    "grantId": "consent-grant-9b1e"
  }
}
```

## Security & Privacy

**Approver authority.** The VTA MUST confirm the issuer may approve for the
subject's platform/context. A directly-signed operator decision is the strong
form; a bridge-attested decision relies on the bridge's enrollment and SHOULD be
recorded with `evidence: "bridge-attested"` so audits can tell them apart.

**Challenge single-use.** Matching and consuming the pending challenge prevents
replaying one approval onto a different conversation.

**Deny is durable.** A recorded `deny` is a grant too; the bridge keeps dropping
that subject until the decision is revoked or expires.

The optional `ext` extension is part of the signed surface.
