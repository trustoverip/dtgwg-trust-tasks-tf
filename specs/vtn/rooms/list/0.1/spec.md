---
slug: vtn/rooms/list
version: "0.1"
title: "VTN Rooms — List"
summary: "List the rooms a network opens, and the role whose groups' members may enter each."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords: [vtn, rooms, roles, room-pass]
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
    A public network answers anyone, so a page that has not signed in can show its rooms; a network whose directory is restricted refuses an unproved request with `permissionDenied`, so the proof is the reader's authentication where it matters. A person relies on the answer to decide which pass to ask for, so a signed response is RECOMMENDED.
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
    Out: room names, roles and addresses from the network's configuration. In: from a person reading a restricted network, that person's membership credential, which is personal data used only to decide whether the directory admits them.
retention:
  class: transient
  rationale: >-
    The network verifies a carried membership credential and discards it; it keeps nothing from a read beyond its operational logs and duplicate-detection window.
errorCodes: []
related:
  - vtn/rooms/pass
  - vtn/roles/list
  - vtn/directory/search
---

## Abstract

A Verifiable Trust Network can open **rooms** for a role — a "Pandemic Response" room for everyone in the groups its members registered for `admins`, say. It does not host them; it issues the passes a room host honours ([`vtn/rooms/pass`](../../pass/0.1/spec.md)). **VTN Rooms — List** tells a reader which rooms the network opens, for which role, where each is hosted, and how long a pass lasts. A room's existence is as visible as the network's directory.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **reader** (`issuer`):

1. **MUST** sign the request when it reads a network whose directory is not public, and **MUST** then carry `membership` if it is a person admitted only by a membership credential.
2. **MUST NOT** treat a listed room as evidence that it may enter: eligibility is decided only when a pass is asked for.

A conforming **network** (`recipient`):

1. **MUST** apply its directory visibility: under `public`, any requester, signed or not; under `members`, an administrator, an `acknowledged` party signing as itself, or a person whose `membership` verifies (a membership credential issued by an `acknowledged` member community, naming the requester's identity, signed for `assertionMethod`, valid, and with a status that resolves as valid); under `admins`, an administrator only. It **MUST** refuse anyone else, including any unproved request to a network that is not public, with `permissionDenied`.
2. **MUST** return every room its configuration opens, each with the role whose groups' members may enter it and the lifetime of its passes.
3. **MUST NOT** use a carried membership credential for anything but this admission decision, and **MUST NOT** retain it.

## Definitions

- **Requester's identity** — the request's `issuer`, or, where the issuer is a key delegated through [`auth/signing-key/enroll`](../../../../auth/signing-key/enroll/0.2/spec.md) or approved through `auth/oob/*`, the identity it was delegated by.
- **`membership`** — a person's membership credential (VMC) from a member community, holder-bound to the requester's identity ([`Membership`](../../../_shared/0.1/entry.schema.json)). Carried only where the network's directory admits members only and the requester is admitted by nothing else; ignored otherwise.
- **`rooms`** — the rooms the network opens ([`RoomDef`](../../../_shared/0.1/rooms.schema.json)): each an `id`, a `name`, the `role` whose groups' members may enter, an optional `description` and `url`, and `passMinutes`, how long a pass lasts.

## Request

A reader (`issuer`) asks the network (`recipient`) for its rooms. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json).

### A person on a public network

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000001101",
  "type": "https://trusttasks.org/spec/vtn/rooms/list/0.1#request",
  "issuer": "did:example:alice",
  "recipient": "did:example:whn",
  "issuedAt": "2026-11-03T13:00:00Z",
  "payload": {}
}
```

## Response

The network answers with the sub-schema reachable via `$anchor: "response"`: its rooms. A refusal is a `trust-task-error`.

### One room

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000001102",
  "type": "https://trusttasks.org/spec/vtn/rooms/list/0.1#response",
  "issuer": "did:example:whn",
  "recipient": "did:example:alice",
  "issuedAt": "2026-11-03T13:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-000000001101",
  "payload": {
    "rooms": [
      {
        "id": "pandemic-response",
        "name": "Pandemic Response",
        "role": "admins",
        "description": "Coordination for every member's administrators.",
        "url": "https://rooms.whn.example/pandemic-response",
        "passMinutes": 60
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

The response carries the network's room configuration — no data about any party. The request carries nothing but, from a person reading a restricted network, their membership credential; a requester **MUST NOT** carry it where the network's visibility does not call for it.

### Correlation

The network learns that a reader looked at its rooms, and from a person admitted by credential, who they are and their community. The network declares `identifierScope: public` because it is an organisation found by its public DID, and a room is scoped to that DID (`<networkDid>#room-<id>`), so a reader must be able to name the network the same way every time. The reader's scope is `any`: a public network serves anonymous readers.

### Retention

The network keeps nothing from a read beyond its operational logs and duplicate-detection window; a carried membership credential is verified and discarded (network item 3).

### Consent/purpose

The rooms are the network's own configuration, published so that eligible people can ask for passes. A carried membership credential is presented for one purpose — admission to read — and is used for nothing else.
