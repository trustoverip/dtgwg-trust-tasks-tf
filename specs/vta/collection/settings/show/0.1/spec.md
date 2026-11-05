---
slug: vta/collection/settings/show
version: "0.1"
title: "VTA Collection — Settings — Show"
summary: "A client reads whether its VTA collects by default for the persona DIDs of one trust context that no client has claimed, who last changed that and when, and whether the VTA's operator has suspended it everywhere."
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
    A read that changes nothing, normally carried by an authenticated transport between a client
    and its own VTA.
sideEffects:
  level: none
  rationale: "Read-only: returns one context's collection setting."
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Out: whether the VTA collects by default in the context, the DID of the client that last
    changed it and when, and whether the operator has suspended it. In: a context.
retention:
  class: exchange
  rationale: >-
    The VTA remains the record; the answer is the client's working copy for the view it renders.
errorCodes: []
related:
  - vta/collection/settings/set
  - vta/collection/list
  - vta/collection/claim
---

## Abstract

The **VTA Collection — Settings — Show** Trust Task tells the principal whether its VTA collects,
by default, for the persona DIDs of one trust context that no client has claimed (VTI-NTF-080,
VTI-NTF-094). Collection by default is a setting of each context, enabled unless a context
administrator disables it with [`vta/collection/settings/set`](../../set/0.1/spec.md); the VTA's
operator may also suspend it for every context at once (VTI-NTF-098). This task answers both, with
who last changed the context's setting and when (VTI-NTF-096).

It is the explanation of a `none` / `disabled` entry in
[`vta/collection/list`](../../../list/0.1/spec.md): that reason is the same whether the context's
own setting disables collection or the operator has suspended it, and this task says which, and who
to ask.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The setting is read with its own task rather than carried in
[`vta/collection/list`](../../../list/0.1/spec.md)'s response because it belongs to the context,
not to any identifier in it, and because adding it to that released response would be a member
every existing client's schema refuses.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` a caller without standing under [Authorization](#authorization),
   and answer nothing about any other context (VTI-NTF-096).
3. Answer `collectByDefault` as the context's stored setting: `true` for a context whose setting
   has never been changed (VTI-NTF-094).
4. Answer `changedBy` and `changedAt` with the subject of the entry that last changed it and when,
   and omit both while it has never been changed (VTI-NTF-096).
5. Answer `suspendedByOperator` `true` exactly while its operator's suspension of collection by
   default is in force (VTI-NTF-098), and `false` where it offers its operator no suspension.

**Retry class** (VTI-OPS-060): the first — no durable effect.

## Authorization

**Standing in the trust context** named by `contextId`, as for
[`vta/collection/list`](../../../list/0.1/spec.md#authorization): an access control entry whose act
scope covers that context. Reading the setting needs no more than reading the listing it explains;
changing it needs administrative authority
([`vta/collection/settings/set`](../../set/0.1/spec.md#authorization)).

## Definitions

- **`contextId`** — the trust context to read.
- **`setting`** (response) — the context's
  [`CollectionSetting`](../../../_shared/0.1/collection.schema.json).
- **Collection by default** — the VTA collecting, from the mediator each identifier's DID document
  advertises, for every identifier of the context VTI-NTF-080 names that no client has claimed
  with [`vta/collection/claim`](../../../claim/0.1/spec.md). It happens exactly when
  `collectByDefault` is `true` and `suspendedByOperator` is `false`.

## Request

The client sends its VTA the top-level schema in [`payload.schema.json`](payload.schema.json).

### Read one context's setting

```json
{
  "id": "urn:uuid:0b1c2d3e-4f50-4617-8829-3a4b5c6d7e8f",
  "type": "https://trusttasks.org/spec/vta/collection/settings/show/0.1",
  "issuer": "did:key:z6MkWalletClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2027-03-01T09:00:00Z",
  "payload": {
    "contextId": "personal"
  }
}
```

## Response

The VTA answers with the sub-schema reachable via `$anchor: "response"`.

### Disabled by a context administrator

```json
{
  "id": "urn:uuid:1c2d3e4f-5061-4728-9930-4b5c6d7e8f90",
  "type": "https://trusttasks.org/spec/vta/collection/settings/show/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkWalletClient",
  "issuedAt": "2027-03-01T09:00:01Z",
  "threadId": "urn:uuid:0b1c2d3e-4f50-4617-8829-3a4b5c6d7e8f",
  "payload": {
    "setting": {
      "contextId": "personal",
      "collectByDefault": false,
      "suspendedByOperator": false,
      "changedBy": "did:key:z6MkDesktopClient",
      "changedAt": "2027-02-27T18:30:00Z"
    }
  }
}
```

### Never changed, and suspended by the operator

```json
{
  "id": "urn:uuid:2d3e4f50-6172-4839-8a41-5c6d7e8f9001",
  "type": "https://trusttasks.org/spec/vta/collection/settings/show/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkWalletClient",
  "issuedAt": "2027-03-01T09:00:01Z",
  "threadId": "urn:uuid:0b1c2d3e-4f50-4617-8829-3a4b5c6d7e8f",
  "payload": {
    "setting": {
      "contextId": "work",
      "collectByDefault": true,
      "suspendedByOperator": true
    }
  }
}
```

## Security & Privacy

### Data carried

A context in; whether the VTA collects by default there, the DID of the client that last changed
that and when, and whether the operator has suspended it, out. Nothing about any identifier or any
message.

### Correlation

`changedBy` names one of the principal's clients to another party with standing in the same
context, which the access control listing already shows it. It is answered to nobody else
(VTI-NTF-096).

### Retention

The VTA remains the record, and its audit trail records each change. A client **SHOULD NOT** keep
the answer beyond the view it renders.

### Consent/purpose

The answer lets a principal see why its personas in a context are collected by nobody, and whom —
a context administrator, or the VTA's operator — to ask to change it.
