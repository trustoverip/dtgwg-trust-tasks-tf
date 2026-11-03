---
slug: vtn/roles/list
version: "0.1"
title: "VTN Roles — List"
summary: "List the roles a network defines, and which of them a party must register a group for before it can become a member."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, roles, groups, broadcast]
parties:
  - role: reader
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: network
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  request: OPTIONAL
  response: RECOMMENDED
  rationale: >-
    A public network answers anyone, so a page that has not signed in can read its roles; a network whose directory is restricted refuses an unproved request with `permissionDenied`, so the proof is the reader's authentication where it matters. A listed party relies on the answer to decide what to register before acknowledging, so a signed response is RECOMMENDED.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A read that changes nothing, so a replay discloses no more than the original did; a restricted network still wants to place a proved request in its acceptance window.
sideEffects:
  level: none
  rationale: >-
    Reads the network's configuration; changes nothing.
exposure:
  discloses: metadata
  ingests: personal
  actsAsSubject: false
  rationale: >-
    Out: role names and descriptions from the network's configuration. In: nothing from an organisation; from a person reading a restricted network, that person's membership credential, which is personal data used only to decide whether the directory admits them.
retention:
  class: transient
  rationale: >-
    The network verifies a carried membership credential and discards it; it keeps nothing from a read beyond its operational logs and duplicate-detection window.
errorCodes: []
related:
  - vtn/groups/register
  - vtn/groups/list
  - vtn/membership/acknowledge
  - vtn/rooms/list
  - vtn/broadcasts/send
---

## Abstract

A Verifiable Trust Network reaches its members through **roles** — `admins`, `notifications`, `security-contacts` — each served by a group every member registers ([`vtn/groups/register`](../../../groups/register/0.1/spec.md)). Some roles are **required**: the network accepts a party's acknowledgement only once it has registered a group for each. **VTN Roles — List** tells a reader which roles the network defines, so that a listed party knows what to register before it acknowledges, and so that a member network knows which parent roles it can map its own groups to.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **reader** (`issuer`):

1. **MUST** sign the request when it reads a network whose directory is not public, and **MUST** then carry `membership` if it is a person admitted only by a membership credential.
2. A listed party **SHOULD** register a group for every role whose `required` is `true` before it acknowledges, since the network will otherwise refuse the acknowledgement.

A conforming **network** (`recipient`):

1. **MUST** answer a requester whose proven signer is a party it has an entry for in state `listed`, `acknowledged` or `suspended` — directly, or through a key that party delegated ([`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md)) — without applying its directory visibility: a listed party must be able to learn what it is asked to register.
2. **MUST** otherwise apply its directory visibility: under `public`, any requester, signed or not; under `members`, an administrator, an `acknowledged` party signing as itself, or a person whose `membership` verifies (a membership credential issued by an `acknowledged` member community, naming the requester's identity, signed for `assertionMethod`, valid, and with a status that resolves as valid); under `admins`, an administrator only. It **MUST** refuse anyone else, including any unproved request to a network that is not public, with `permissionDenied`.
3. **MUST** return every role its configuration defines, with `required` stating whether its acknowledgement rule requires a group for it, and **MUST** use the same `required` values when it applies that rule.
4. **MUST NOT** use a carried membership credential for anything but this admission decision, and **MUST NOT** retain it.

## Definitions

- **`membership`** — a person's membership credential (VMC) from a member community, holder-bound to the requester's identity ([`Membership`](../../../_shared/0.1/entry.schema.json)). Carried only where the network's directory admits members only and the requester is admitted by nothing else; ignored otherwise.
- **`roles`** — the roles the network defines ([`RoleDef`](../../../_shared/0.1/groups.schema.json)): each a `name`, an optional `description`, and `required`.

## Request

A reader (`issuer`) asks the network (`recipient`) for its roles. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A listed party asks before acknowledging

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000001001",
  "type": "https://trusttasks.org/spec/vtn/roles/list/0.1#request",
  "issuer": "did:example:community",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T09:50:00Z",
  "payload": {}
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`: its roles. A refusal is a `trust-task-error`.

### Two roles, one required

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000001002",
  "type": "https://trusttasks.org/spec/vtn/roles/list/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-03T09:50:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000001001",
  "payload": {
    "roles": [
      { "name": "admins", "description": "Each member's administrators, for network notices.", "required": true },
      { "name": "security-contacts", "required": false }
    ]
  }
}
```

## Security & Privacy

### Data carried

The response carries role names and descriptions from the network's configuration — no data about any party. The request carries nothing but, from a person reading a restricted network, their membership credential; a requester **MUST NOT** carry it where the network's visibility does not call for it.

### Correlation

The network learns that a reader asked for its roles, and from a person admitted by credential, who they are and their community. The network declares `identifierScope: public` because it is an organisation found by its public DID, and the roles it lists are only meaningful against the network a party is listed in, named the same way every time. The reader's scope is `any`: a public network serves anonymous readers.

### Retention

The network keeps nothing from a read beyond its operational logs and duplicate-detection window; a carried membership credential is verified and discarded (network item 4).

### Consent/purpose

The roles are the network's own configuration, published for parties to register against. A carried membership credential is presented for one purpose — admission to read — and is used for nothing else.
