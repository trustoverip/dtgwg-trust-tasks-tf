---
slug: vta/collection/settings/set
version: "0.1"
title: "VTA Collection — Settings — Set"
summary: "A context administrator enables or disables, for one trust context, whether the VTA collects the messages queued at the mediator for the persona DIDs there that no client has claimed — stored, audited, and applied at once."
status: draft
targetFrameworkVersion: "0.6.0"
category: notifications
keywords:
  - collection
  - mediator
  - persona
  - vta
  - settings
parties:
  - role: Client acting for the principal
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTA
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    Normally carried by an authenticated transport between a client and its own VTA. The change
    confers no authority and moves no key: it decides whether the VTA, which already holds the
    identifiers' keys, collects for them, and the caller's administrative authority over the context
    is what the VTA checks.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task changes a stored setting and opens or closes the VTA's connections, so the VTA must be
    able to place the request in a bounded window to absorb a repeat (SPEC §7.2 item 11).
sideEffects:
  level: mutating
  rationale: >-
    Stores the context's setting and applies it: disabling stops the VTA collecting for the
    context's unclaimed identifiers and closes its connections for them; enabling starts it. Either
    is undone by setting the other value, and messages not collected meanwhile wait at the
    mediator.
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a context and a boolean. Out: the context's setting, the DID of the client that last changed
    it and when, and whether the operator has suspended collection by default.
retention:
  class: durable
  rationale: >-
    The VTA keeps the setting across restarts, including a trusted execution environment's, and its
    audit trail records each change with who made it (VTI-NTF-094, VTI-NTF-096).
errorCodes: []
related:
  - vta/collection/settings/show
  - vta/collection/list
  - vta/collection/claim
  - vta/collection/release
---

## Abstract

The **VTA Collection — Settings — Set** Trust Task turns collection by default on or off for one
trust context. A VTA collects, from the mediator each persona DID's document advertises, for every
persona of the principal's that no client has claimed with
[`vta/collection/claim`](../../../claim/0.1/spec.md) (VTI-NTF-080), so that a principal whose only
client is a browser still receives what is addressed to its personas. That is enabled for every
context unless it is disabled there (VTI-NTF-094).

A principal disables it for a context while a client of its collects for that context's personas
without claiming them — a client released before collection claims — because the VTA collecting
beside such a client is the second connection to each persona, and each would evict the other
(VTI-NTF-085, VTI-NTF-097). The setting is stored by the VTA and applied when the change is
answered: no restart, and nothing lost when the VTA restarts.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **producer** (the client) **SHOULD** disable collection by default for a context
while any client of the principal collects for an identifier there without claiming it
(VTI-NTF-097), and enable it again once every such client claims first.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), and refuse with
   `permissionDenied` a caller without the authority stated under [Authorization](#authorization).
2. Store the setting for `contextId` alone — not its ancestors or descendants — where it survives
   the VTA's restart, including the restart of a trusted execution environment that holds it
   (VTI-NTF-094).
3. Where `collectByDefault` is `false` and the context's setting was enabled: before it answers,
   stop collecting under VTI-NTF-080 for the context's identifiers and close every connection it
   held to do so (VTI-NTF-095). It **MUST NOT** answer while such a connection remains open.
4. Where `collectByDefault` is `true` and the context's setting was disabled: begin to collect for
   each identifier VTI-NTF-080 then names, including one it had stopped collecting for after a
   conflict (VTI-NTF-085). It need not wait for those connections to open before it answers.
5. Where the context already held the value asked for: change nothing, and answer
   `changed: false` (VTI-NTF-095).
6. Record each change in its audit trail with the caller, the context and the value set
   (VTI-NTF-096), and answer with the setting as it now holds it, `changedBy` and `changedAt`
   included.
7. Store the change while its operator's suspension (VTI-NTF-098) is in force, and answer it with
   `suspendedByOperator: true`, but collect for nothing under it until the suspension is lifted.

Claims are granted, renewed, released and listed whatever the setting (VTI-NTF-097): a claim made
while collection by default is disabled changes only the record, and is in place when it is enabled.

**Retry class** (VTI-OPS-060): the second — a durable effect whose repeat is harmless. A repeat of
the same document is answered with the answer first given
([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 11).

**A repeated request** sets a state: a new document setting the value already held succeeds with
`changed: false`.

## Authorization

**Administrative authority over the trust context** named by `contextId` (VTI-ACL-022,
VTI-NTF-095): an access control entry whose role is administrative and whose act scope covers that
context — a context administrator of it or of an ancestor of it, or a super-administrator. Standing
in the context alone, which is enough to claim an identifier and to read the setting, is not:
a claim decides for one identifier and the client that makes it, whereas this setting decides for
every identifier and every client in the context at once, and disabling it leaves the personas of
every browser-only client there collected by nobody.

## Definitions

- **`contextId`** — the trust context whose setting to change.
- **`collectByDefault`** — the value to set.
- **`setting`** (response) — the context's
  [`CollectionSetting`](../../../_shared/0.1/collection.schema.json) once the change has taken
  effect.
- **`changed`** (response) — whether this request changed the setting.

## Request

The client sends its VTA the top-level schema in [`payload.schema.json`](payload.schema.json).

### Disable collection by default while an older client is in use

```json
{
  "id": "urn:uuid:3e4f5061-7283-494a-8b52-6d7e8f900112",
  "type": "https://trusttasks.org/spec/vta/collection/settings/set/0.1",
  "issuer": "did:key:z6MkDesktopClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2027-02-27T18:30:00Z",
  "payload": {
    "contextId": "personal",
    "collectByDefault": false
  }
}
```

## Response

The VTA answers with the sub-schema reachable via `$anchor: "response"`. A change it refuses is
answered with `trust-task-error`.

### Disabled

```json
{
  "id": "urn:uuid:4f506172-8394-4a5b-9c63-7e8f90011223",
  "type": "https://trusttasks.org/spec/vta/collection/settings/set/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkDesktopClient",
  "issuedAt": "2027-02-27T18:30:00Z",
  "threadId": "urn:uuid:3e4f5061-7283-494a-8b52-6d7e8f900112",
  "payload": {
    "setting": {
      "contextId": "personal",
      "collectByDefault": false,
      "suspendedByOperator": false,
      "changedBy": "did:key:z6MkDesktopClient",
      "changedAt": "2027-02-27T18:30:00Z"
    },
    "changed": true
  }
}
```

## Security & Privacy

### Data carried

A context and a boolean in; the context's setting, the DID of the client that last changed it and
when, and whether the operator has suspended collection by default, out. Nothing about any
identifier or any message.

### Correlation

Nothing about the change reaches a mediator or any sender beyond what the VTA's connections
already show: a mediator sees the VTA's connections for the context's personas close or open,
which is what it sees when a client claims or releases them. What a sender receives in reply to a
notification is the same whoever collects it (VTI-NTF-086).

### Retention

The VTA keeps the setting until it is changed again, and its audit trail records each change with
the caller, the context and the value.

### Consent/purpose

The setting exists so that a principal can keep its VTA from becoming the second connection to its
personas while a client that does not claim them is in use. It confers nothing: it gives no party
the identifiers' keys or any authority at the VTA, and it changes nothing about the acceptance
policy, which still decides every notification the VTA collects.
