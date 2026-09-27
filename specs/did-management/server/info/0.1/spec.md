---
slug: did-management/server/info
version: "0.1"
title: DID Management — Server Info
summary: Anyone asks a DID hosting service for the public facts a client needs before it has a session — the service's own DID, whether it serves agent names, and the names its own DID serves.
status: draft
targetFrameworkVersion: "0.6.0"
category: did-management
keywords: [did-hosting, discovery, bootstrap, public, agent-names]
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Client
    requirement: OPTIONAL
    member: issuer
    identifierScope: any
  - role: DID hosting service
    requirement: OPTIONAL
    member: recipient
    identifierScope: public
proofRequirement:
  request: OPTIONAL
  response: REQUIRED
  rationale: "The request is a public read: it asks for nothing but facts the service publishes, and a client that has no key yet must be able to send it. The response is REQUIRED because a client takes `serviceDid` from it as the recipient of every task it sends next; a signature by that DID over the response is what lets the client check the service controls the DID it names, instead of taking an unsigned claim from whatever answered."
issuedAtRequirement:
  requirement: OPTIONAL
  rationale: "A public read with no effect; nothing depends on placing it in time."
sideEffects:
  level: none
  rationale: "Reads configuration and the service's own record; nothing is written."
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
  rationale: "Every member of the response is already public: the service's DID and the names it serves are in its own published DID document, and whether it serves agent names is observable by anyone who has a name to try."
retention:
  class: transient
  rationale: "The request carries nothing to keep."
bearer: true
errorCodes: []
related:
  - did-management/server/config
  - trust-task-discovery
---

## Abstract

A client arriving at a hosting service's web surface — its login page, a browser extension, a script — has to know the service's DID before it can address a Trust Task to it, and has to know whether the service serves agent names before it offers them. **Server Info** answers those two questions, with no session and no key.

It is deliberately narrow. It says nothing about the DIDs the service hosts — the association between hosted DIDs and the service that controls them is the thing a hosting service never publishes — and nothing about its fleet. Operational settings are [`did-management/server/config`](../../config/0.1/spec.md), for administrators.

## Status of this Document

Draft ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements). This specification is a **bearer** specification ([SPEC §4.8.3](/SPEC.md#483-bearer-specifications)): its documents may omit `recipient`, because a client asks it precisely when it does not yet know the recipient's identifier, and because the response is a statement by the service about itself that is equally true for every reader.

A conforming hosting service:

1. Accepts the request with or without `issuer`, `recipient` and `proof`, and does not authorise it. A request that does carry a `proof` is verified like any other and refused if it does not verify; an absent one is not a defect.
2. Answers with `serviceDid` = its own DID, which **MUST** also be the response's `issuer`, and signs the response with a key that DID's document lists under `authentication`.
3. Sets `recipient` on the response to the request's `issuer` when the request named one, and omits it otherwise.
4. **MUST NOT** include in the response anything about a DID it hosts, about its fleet, or about its configuration beyond the members of the schema.

A conforming client **MUST** verify the response's proof, and that its `issuer` equals `serviceDid`, before addressing anything to `serviceDid`.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.* Not consequential, and declared for clarity.

None. The task is open to any caller, which is safe because it returns only facts the service already publishes and changes nothing.

## Definitions

**Community name** — the agent name of a service that owns its whole domain (`{domain}/@`), whose local part is empty.

## Request

```json
{
  "id": "urn:uuid:c17d9aef-8eac-4df0-8f9a-7c8d9e0f1a01",
  "type": "https://trusttasks.org/spec/did-management/server/info/0.1",
  "payload": {}
}
```

## Response

```json
{
  "id": "urn:uuid:c17d9aef-8eac-4df0-8f9a-7c8d9e0f1a02",
  "type": "https://trusttasks.org/spec/did-management/server/info/0.1#response",
  "threadId": "urn:uuid:c17d9aef-8eac-4df0-8f9a-7c8d9e0f1a01",
  "issuer": "did:webvh:QmControlScid2:control.example.com",
  "issuedAt": "2026-09-27T09:00:01Z",
  "payload": {
    "serviceDid": "did:webvh:QmControlScid2:control.example.com",
    "agentNames": true,
    "serviceNames": [""],
    "domainPurgeGraceSeconds": 604800
  }
}
```

## Security & Privacy

### Data carried

The request carries nothing. The response carries the service's DID, a flag, the service's own agent names and a duration. None of it concerns a hosted DID or a person.

### Correlation

A client that sends no `issuer` is not identified by the task at all; one that sends an `issuer` is identified to a service it was about to authenticate to anyway. The service declares `identifierScope: public`, and must: `serviceDid` is the one every client will address, and the response is only useful if it names it.

### Retention

Nothing needs keeping. A client **MAY** cache the response for the length of its visit and **SHOULD** re-read it when the service's signature on a later reply stops verifying against the cached DID.

### Consent/purpose

The response exists so a client can address the service and decide which features to offer. It is public by design; its narrowness is the privacy property, and rule 4 is what keeps it.
