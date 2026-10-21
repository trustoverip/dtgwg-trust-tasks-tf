---
slug: vtc/config/did-hosting/list
version: '0.1'
title: VTC Config — DID Hosting — List
summary: 'An administrator lists the did:webvh hosting servers the community''s VTA can publish its identities through, to choose where its groups'' and rooms'' identities are published.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
parties:
- role: administrator
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: Answered to holders of `vtc.config.admin` only, read from the signer's access-control entry.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: The community asks its VTA on the caller's behalf; `issuedAt` bounds how old a request it answers.
sideEffects:
  level: none
  rationale: Reads the VTA's registrations through the community's link to it; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: The hosting servers' ids, DIDs and labels — operational topology the VTA answers any authenticated caller, carrying no credential.
retention:
  class: transient
  rationale: A read.
errorCodes:
- code: vtc/config/did-hosting/list:noKeyHolder
  meaning: The community has no link to a VTA. Its identities can only be served by the community itself; its host's operator configures a link to publish elsewhere.
  retryable: false
- code: vtc/config/did-hosting/list:keyHolderUnavailable
  meaning: The community's VTA did not answer.
  retryable: true
- code: vtc/config/did-hosting/list:keyHolderRefused
  meaning: The community's VTA refused the read. Its own code and message are carried in `details`.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    required:
    - code
    properties:
      code:
        type: string
        maxLength: 128
      message:
        type: string
        maxLength: 500
related:
- vtc/config/export
- vtc/config/import
- vta/webvh/servers/list
- vtc/groups/create
- vtc/rooms/community/create
---

## Abstract

**VTC Config — DID Hosting — List** answers an administrator choosing where the identities the community mints are published: each group's (`vtc/groups/create`) and each room's it owns (`vtc/rooms/community/create`). A community either serves an identity's `did:webvh` log itself, or has its VTA publish it on a hosting server registered there. This task lists those servers, as the community's VTA reports them ([`vta/webvh/servers/list`](../../../../../vta/webvh/servers/list/1.0/spec.md)), so the choice is made from what exists rather than typed.

The choice itself is configuration, written the way the community writes any of its configuration; this task only reads.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer only a caller holding `vtc.config.admin`, read from the signer's access-control entry at execution time, and refuse any other with `permissionDenied`.
2. Ask its own VTA, over the link it already keeps to it, as the community — never as the caller — and answer with the servers the VTA reported, each with its `id`, `did` and `label` where the VTA gave one, and nothing else of the VTA's record. A server whose id the `HostingServer` schema cannot carry is left out: the community's configuration could not name it either.
3. Refuse with `noKeyHolder` when it has no link to a VTA, with `keyHolderUnavailable` when the VTA does not answer, and with `keyHolderRefused`, carrying the VTA's own code and message, when it refuses.
4. **MUST NOT** return or log any credential the VTA holds for a server; the VTA's record carries none, and the community adds none.

## Authorization

**Authority: `vtc.config.admin`**, read from the signer's access-control entry at execution time ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- `HostingServer` — see [`payload.schema.json`](payload.schema.json).

## Request

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000101",
  "type": "https://trusttasks.org/spec/vtc/config/did-hosting/list/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-10-21T09:00:00Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000100",
  "payload": {}
}
```

## Response

```json
{
  "id": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000102",
  "type": "https://trusttasks.org/spec/vtc/config/did-hosting/list/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-10-21T09:00:01Z",
  "threadId": "urn:uuid:7d1e0c4a-5b2f-4c3d-8e9f-000000000100",
  "payload": {
    "servers": [
      {
        "id": "hosting-eu",
        "did": "did:webvh:QmHost:hosting.example.com",
        "label": "EU hosting"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Each hosting server's id at the VTA, its DID and its label. None is a secret: the VTA answers the same to any authenticated caller, and the DIDs it publishes name their host anyway.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its administrators and members already know it by.

### Retention

None; a read.

### Consent/purpose

An administrator choosing where the community's identities are published.
