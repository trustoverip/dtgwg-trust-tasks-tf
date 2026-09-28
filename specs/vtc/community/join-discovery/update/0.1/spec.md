---
slug: vtc/community/join-discovery/update
version: "0.1"
title: "VTC Community — Join Discovery — Update"
summary: An administrator replaces whether the community answers its join manifest to a caller it cannot identify.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The write changes what the community tells every prospective applicant, under the community's name, and is authorized only by the signer's administrator standing — so the signer has to be attributable on every transport, and the change has to be attributable afterwards in the audit trail.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed update would silently put back a configuration an administrator has since replaced, so a duplicate must be placeable in a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Replaces one configuration row. Recoverable by another update; nothing is issued, revoked or deleted beyond the row itself.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    The request carries community configuration an administrator wrote; the response echoes what was stored. No personal data.
retention:
  class: durable
  rationale: >-
    The stored value is the community's configuration until the next update, and the change is recorded in the audit trail against the administrator who made it.
errorCodes: []
related:
  - vtc/community/join-discovery/show
  - vtc/join-requests/manifest
---

## Abstract

The **VTC Community — Join Discovery — Update** Trust Task sets whether the community answers [`vtc/join-requests/manifest`](../../../../join-requests/manifest/0.2/spec.md) to a caller it cannot identify.

Turning it off does not refuse applicants: a signed request, or one from a transport-authenticated sender, is still answered, and refusing those would break the join ceremony rather than close anything. It makes the answer attributable.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. **MUST** refuse a caller that does not hold administrator standing with `permissionDenied`, resolving a delegated signing key to the identity it acts for.
2. **MUST** apply the stored setting to every manifest request it receives afterwards, and **MUST** keep answering identified callers whatever it is.
3. **MUST** record a change in its audit trail, naming the administrator and the new value, and **MUST** refuse with `unavailable` rather than make an unaudited change. An update that changes nothing is not audited.
4. **MUST** answer with what it stored.

## Authorization

The authority this task presupposes is **administrator standing at the community**, read at execution time. The decision is the community's ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

- **`joinDiscovery.public`** — the new setting.

## Request

A community administrator (`issuer`) sends the new setting to the community (`recipient`). The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### Answer only identified callers

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000003",
  "type": "https://trusttasks.org/spec/vtc/community/join-discovery/update/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-28T10:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "joinDiscovery": {
      "public": false
    }
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"`: what it stored. A refusal is a `trust-task-error`.

### Stored

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000004",
  "type": "https://trusttasks.org/spec/vtc/community/join-discovery/update/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-09-28T10:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000001ff",
  "payload": {
    "joinDiscovery": {
      "public": false
    }
  }
}
```

## Security & Privacy

### Data carried

Community configuration an administrator wrote. One boolean.

### Correlation

The community is declared with identifier scope `public`: it is the one DID every administrator, member and applicant addresses, and the records this task reads or writes are its own. A pairwise identifier for it would leave a caller unable to tell which community answered. The audit trail ties each change to the administrator who made it; that attribution is the point of requiring a proof.

### Retention

The stored value lives until the next update. The audit record of the change is durable and names the administrator and what changed, not the whole value.

### Consent/purpose

The purpose is to set how the community presents itself to, or what it asks of, prospective applicants. Using the value for anything else — for instance deciding an application on an attribute the community did not publish as requested — is outside it.
