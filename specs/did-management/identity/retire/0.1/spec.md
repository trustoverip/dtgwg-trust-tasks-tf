---
slug: did-management/identity/retire
version: "0.1"
title: DID Management — Identity Retire
summary: An administrator makes a DID hosting service stop honouring a superseded generation of its own identity immediately, dropping that generation's key material ahead of its grace period — the kill switch for a compromised key.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did-hosting, identity, key-rotation, compromise, kill-switch, admin]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: DID hosting service
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "The task irreversibly destroys key material the service is using to receive messages. It must be attributable to an administrator on every transport, and it must not be satisfiable by the very key being retired — the service authorises on the proven issuer, never on its own identity."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A destructive task. A retire replayed after the operator has rolled back to that generation would cut the service off again; the issue time bounds how long a captured one can be spent."
sideEffects:
  level: destructive
  rationale: "Drops the generation's private keys from the running service and from its store. Messages still addressed to that generation's key-agreement key no longer decrypt, and nothing can restore the keys."
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
  rationale: "The response confirms the generation and when it was dropped."
retention:
  class: durable
  rationale: "The signed request is the record of an administrator destroying the service's key material, and belongs in the audit trail for as long as the service keeps one."
errorCodes:
  - code: did-management/identity/retire:notFound
    meaning: "The service honours no generation with this `generationId` — it never existed, or has already expired or been retired."
    retryable: false
  - code: did-management/identity/retire:current
    meaning: "The generation is the current one. Retiring it would leave the service unable to decrypt anything; rotate first (publish a new DID document version), then retire the generation that rotation superseded."
    retryable: false
related:
  - did-management/identity/list
---

## Abstract

When a hosting service rotates its identity it keeps honouring the superseded generation for a grace period, so peers with a cached DID document can still reach it. If the superseded key is compromised, that grace period is the attacker's window. **Identity Retire** closes it: the service drops the generation's key material from the running process — the secrets resolver and every listener profile — before it answers, and from its store.

Peers whose cached document still names the dropped key cannot reach the service until their cache expires. That breakage is the point.

The task retires only a *superseded* generation. To stop using the current key, the operator rotates — publishes a new DID document version — which makes it superseded; then retires it.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming hosting service:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`), and one with no `issuedAt` with `malformedRequest`.
2. Refuses a caller without administrator standing with `permissionDenied`. A document whose proven issuer is the service's own DID is refused the same way: the service is never its own administrator.
3. Answers `notFound` for a generation it does not honour and `current` for the current one.
4. Otherwise removes the generation's private keys from every in-process holder — so that no further message addressed to them decrypts and no further signature is made with them — and from its store, then answers with `droppedAt`. It **MUST NOT** answer before the in-process removal has completed, since the answer is what tells the administrator the window is closed.
5. Is idempotent in effect: a second retire of the same generation is `notFound`.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The entitlement is **administrator standing on this hosting service**, read from its own access-control records at execution time; the `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The retire is a local operator action on the service's own keys, not a delegable authority: no peer, hosted-DID owner or registered instance holds it.

## Definitions

**Generation** — as in [`did-management/identity/list`](../../list/0.1/spec.md).

## Request

```json
{
  "id": "urn:uuid:e39fbc10-a0ce-4f12-8b1c-9e0f1a2b3c01",
  "type": "https://trusttasks.org/spec/did-management/identity/retire/0.1",
  "issuer": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": { "generationId": 1 }
}
```

## Response

```json
{
  "id": "urn:uuid:e39fbc10-a0ce-4f12-8b1c-9e0f1a2b3c02",
  "type": "https://trusttasks.org/spec/did-management/identity/retire/0.1#response",
  "threadId": "urn:uuid:e39fbc10-a0ce-4f12-8b1c-9e0f1a2b3c01",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": { "generationId": 1, "droppedAt": "2026-09-27T09:00:01Z" }
}
```

## Security & Privacy

### Data carried

The request carries a generation number; the response, the same number and an instant. The effect is the destruction of private keys; the documents carry none.

### Correlation

The task concerns only the service. That an administrator retired a generation early is observable to peers as failures to reach the service under the old key — which is the intended effect. Both parties declare `identifierScope: public`, and must: the service's DID is the recipient every administrator addresses and the key its replies are verified against, and the administrator's DID is what the access-control entry is keyed on.

### Retention

The signed request is evidence that a named administrator destroyed the service's key material at a stated time, and **SHOULD** be retained with the service's audit trail.

### Consent/purpose

The task exists to end a compromised key's usefulness at once. It **MUST NOT** be used as a routine substitute for letting a superseded generation lapse, since every early retire strands peers with a cached document.
