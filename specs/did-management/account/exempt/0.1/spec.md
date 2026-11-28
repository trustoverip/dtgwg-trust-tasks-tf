---
slug: did-management/account/exempt
version: "0.1"
title: "DID Management — Account Exempt"
summary: "An administrator exempts a self-registered account from its domain's terms — path prefixes, the agent-name and transfer bars, the inactivity lifecycle — or holds it to them again."
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did, did-hosting, account, self-registration, exempt, administration, expiry]
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
  - role: DID hosting service
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    Exempting an account lifts limits a stranger was admitted under, and that is an administrator's decision; the
    proof is what ties it to the administrator accountable for it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A consequential change; a replayed exemption could lift terms an administrator has since restored. The
    duplicate-execution record of SPEC §7.2 item 11 is bounded by this timestamp.
sideEffects:
  level: mutating
  rationale: >-
    Changes which rules an account is held to, and restores the DIDs of an account the lifecycle had disabled.
    Reversible with `exempt: false`.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: >-
    Returns the account, as did-management/account/status shows it to its holder.
retention:
  class: durable
  rationale: >-
    The exemption is part of the account's standing for as long as the account exists.
errorCodes:
  - code: did-management/account/exempt:noAccount
    meaning: The subject holds no self-registered account here. An account an administrator created is never held to self-registration terms, so it has nothing to be exempted from.
    retryable: false
related:
  - did-management/account/register
  - did-management/account/policy/set
  - did-management/account/status
---

## Abstract

A self-registered account (did-management/account/register) is an ordinary DID owner held to its domain's terms: its DIDs sit under the domain's path prefixes, it holds no agent names and transfers no DID, and it expires when unused. Those terms exist because the service knew nothing of the registrant but its admission evidence. Once an operator knows more — the registrant is a partner, a long-term customer, a service of its own — **Account Exempt** lifts them, leaving an ordinary owner. Setting `exempt: false` holds the account to the terms again.

Quotas are not part of the exemption: they sit on the account's access-control entry, where an administrator changes them as for any owner.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the hosting service):

1. **MUST** refuse with `permissionDenied` unless the issuer is an administrator of the account's domain, and with `noAccount` when the subject holds no self-registered account.
2. With `exempt: true`, **MUST** stop applying the domain's path prefixes, agent-name and transfer bars and inactivity lifecycle to the account, and **MUST** restore to service any DID the lifecycle had disabled for inactivity (never one its owner or an administrator disabled).
3. With `exempt: false`, **MUST** apply the terms again, and **MUST** restart the account's inactivity period from the request, so that an account is never disabled for time it spent exempt.
4. **MUST NOT** change the account's quotas, role or DIDs otherwise.

Exempting is a *setting a state* operation ([SPEC §7.2](/SPEC.md#72-consumer-requirements)): a request for the state that already holds succeeds, with `changed: false`.

## Authorization

The entitlement is **administrative authority over the account's domain**, as the service's access-control list records it. The proof establishes who is asking; the consumer checks that party's ACL entry. The holder of the account cannot exempt itself.

## Definitions

* **Terms.** The domain's [SelfRegistrationPolicy](../../../_shared/0.4/self-registration-policy.schema.json), as they apply to an account.
* `subject`, `exempt` — as their schema descriptions say.

## Request

An administrator sends the request to the hosting service; the payload validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Exempting a partner's agent

```json
{
  "id": "urn:uuid:e1f2a3b4-c5d6-4e7f-8a9b-0c1d2e3f4a5b",
  "type": "https://trusttasks.org/spec/did-management/account/exempt/0.1",
  "issuer": "did:webvh:QmAdminScid:admin.example.com",
  "recipient": "did:webvh:QmHostScid:did.example.com",
  "issuedAt": "2026-12-05T10:00:00Z",
  "payload": { "subject": "did:scid:vh:1:QmScidExample", "exempt": true },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-12-05T10:00:00Z",
    "verificationMethod": "did:webvh:QmAdminScid:admin.example.com#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z4AqPoX3xdRbW"
  }
}
```

## Response

The service answers with the account after the request and whether it changed, validating against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error`, not a `#response` document.

### The account is exempt

```json
{
  "id": "urn:uuid:f2a3b4c5-d6e7-4f8a-9b0c-1d2e3f4a5b6c",
  "type": "https://trusttasks.org/spec/did-management/account/exempt/0.1#response",
  "threadId": "urn:uuid:e1f2a3b4-c5d6-4e7f-8a9b-0c1d2e3f4a5b",
  "issuer": "did:webvh:QmHostScid:did.example.com",
  "recipient": "did:webvh:QmAdminScid:admin.example.com",
  "issuedAt": "2026-12-05T10:00:01Z",
  "payload": {
    "account": {
      "subject": "did:scid:vh:1:QmScidExample",
      "role": "owner",
      "domain": "did.example.com",
      "state": "active",
      "admission": "invite",
      "registeredAt": "2026-12-01T09:00:01Z",
      "lastActiveAt": "2026-12-04T18:00:00Z",
      "exempt": true
    },
    "changed": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-12-05T10:00:01Z",
    "verificationMethod": "did:webvh:QmHostScid:did.example.com#key-0",
    "proofPurpose": "authentication",
    "proofValue": "z5BqPoX3xdRbW"
  }
}
```

## Security & Privacy

### Data carried

The request names an account by its public DID; the response carries the account as its holder would see it.

### Correlation

The task links the administrator to the account it exempts. That is the audit record of the decision, and is not disclosed to anyone else.

### Retention

The exemption is kept with the account for its life. The service **SHOULD** keep an audit record of each change, naming the administrator.

### Consent/purpose

The exemption exists to lift limits an operator no longer needs. It carries no data collected for any other purpose.
