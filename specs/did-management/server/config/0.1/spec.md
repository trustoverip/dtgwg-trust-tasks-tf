---
slug: did-management/server/config
version: "0.1"
title: DID Management — Server Config
summary: An administrator reads a DID hosting service's effective, non-secret configuration — its identity, transports, DID methods, session lifetimes, registry and storage settings.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did-hosting, config, admin, operations, read]
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
  rationale: "The configuration discloses internal topology — the mediator, the agent holding the service's keys, listen addresses and storage paths — to administrators only. The service authorises on the document's proven issuer, so the proof is what makes the administrator the caller on every transport."
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: "A read with no effect to replay, but an issue time dates the request for a service that audits administrative reads."
sideEffects:
  level: none
  rationale: "Reads the running configuration; the service's own DID document may be resolved to report what it advertises, and nothing is written."
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: "Returns non-secret operational settings. Secrets — keys, tokens, credentials, passwords — are never part of the response."
retention:
  class: transient
  rationale: "The request carries nothing but its envelope."
errorCodes: []
related:
  - did-management/server/info
  - did-management/registry/list
  - did-management/stats/get
  - did-management/identity/list
---

## Abstract

**Server Config** is the administrator's read of how a hosting service is running: the settings an operator console's configuration page shows, and the control plane's own half of its dashboard (the fleet half is [`did-management/registry/list`](../../../registry/list/0.1/spec.md), the counters [`did-management/stats/get`](../../../stats/get/0.1/spec.md)).

The response states both what the configuration *enables* (`transports`) and what the service's own DID document *advertises* (`advertisedServices`). They can differ — a transport enabled after the document was last published is not yet reachable — and an operator needs to see both to tell why a peer cannot reach the service.

What an unauthenticated client needs before it has a session is [`did-management/server/info`](../../info/0.1/spec.md), a different, public task.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). A conforming hosting service:

1. Refuses a document with no verified `proof` bound to its `issuer` with `proofRequired` (or `proofInvalid`).
2. Refuses a caller without administrator standing with `permissionDenied`.
3. Returns its effective configuration — after every file, environment and default has been applied — omitting the members it does not configure.
4. **MUST NOT** include a secret in any member, including `ext`: no private key, seed, token, credential, password or connection string with credentials in it. A member whose configured value would carry one (a URL with a password in it, say) is reported with the secret removed, or not at all.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.* Not consequential, and declared for clarity.

The entitlement is **administrator standing on this hosting service**, read from its own access-control records at execution time; the `proof` establishes who asked, not that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

**Effective configuration** — the values the running service is using, as opposed to what any one configuration source says.

## Request

```json
{
  "id": "urn:uuid:b06c8fde-7d9b-4cef-9e8f-6b7c8d9e0f01",
  "type": "https://trusttasks.org/spec/did-management/server/config/0.1",
  "issuer": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "recipient": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:00Z",
  "payload": {}
}
```

## Response

```json
{
  "id": "urn:uuid:b06c8fde-7d9b-4cef-9e8f-6b7c8d9e0f02",
  "type": "https://trusttasks.org/spec/did-management/server/config/0.1#response",
  "threadId": "urn:uuid:b06c8fde-7d9b-4cef-9e8f-6b7c8d9e0f01",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "recipient": "did:webvh:QmAdminScid1:admin.example.com:alice",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": {
    "serviceDid": "did:webvh:QmControlScid2:control.example.com",
    "softwareVersion": "0.9.0",
    "deploymentMode": "control-plane",
    "publicUrl": "https://control.example.com",
    "didHostingUrl": "https://did.example.com",
    "mediatorDid": "did:webvh:QmMediatorScid6:mediator.example.com",
    "transports": { "tsp": true, "didcomm": true },
    "advertisedServices": ["TSPTransport", "DIDCommMessaging"],
    "enabledMethods": ["webvh", "web"],
    "agentNames": true,
    "listenAddress": "0.0.0.0:8530",
    "vta": { "did": "did:webvh:QmVtaScid5:vta.example.com", "url": "https://vta.example.com" },
    "registry": { "healthCheckIntervalSeconds": 60, "configuredInstances": 0 },
    "sessions": { "accessTokenSeconds": 900, "refreshTokenSeconds": 86400, "adminIdleTimeoutSeconds": 1800, "passkeyEnrollmentSeconds": 86400 },
    "storage": { "dataDir": "/var/lib/did-hosting" },
    "logging": { "level": "info", "format": "json" }
  }
}
```

## Security & Privacy

### Data carried

The response carries operational settings. Several are internal topology an attacker would value — the mediator and agent the service depends on, where it listens, where it stores data — which is why the read is administrator-only and why consumers **SHOULD** require a confidential transport. Secrets are excluded by rule 4, not by convention: the response is the kind of document that gets pasted into a support ticket.

### Correlation

The response links the service's DID to its mediator and its agent. Both are already visible to anyone who resolves the service's DID document or watches its traffic; the listen address and storage path are not, and are disclosed only to administrators. Both parties declare `identifierScope: public`, and must: the service's DID is the recipient every administrator addresses and the key its replies are verified against, and the administrator's DID is what the access-control entry is keyed on.

### Retention

Nothing of the request needs keeping beyond the exchange. A consumer that audits administrative reads **MAY** record the administrator.

### Consent/purpose

The configuration is disclosed so an administrator can operate and diagnose the service. It **MUST NOT** be disclosed to hosted-DID owners, who have no business with the service's internals.
