---
slug: vta/notifications/dismiss
version: "0.1"
title: "VTA Notifications — Dismiss"
summary: "A client dismisses one notification its VTA holds for the principal, taking it out of the default list without telling anyone — the originator, a relayer or the thing's holder."
status: draft
targetFrameworkVersion: "0.6.0"
category: notifications
keywords:
  - notifications
  - inbox
  - vta
  - dismiss
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
    A reversible change to the principal's own view, normally carried by an authenticated transport
    between a client and its own VTA. Nothing outside the VTA relies on it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The task changes a record, so the VTA must be able to place the request in a bounded window to
    absorb a repeat (SPEC §7.2 item 11).
sideEffects:
  level: mutating
  rationale: >-
    Sets the held notification's state to dismissed. The notification stays held, readable and
    listable when asked for, until it lapses.
subjectPath: /contextId
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a context and a record identifier. Out: whether the request changed anything.
retention:
  class: durable
  rationale: >-
    The dismissal is part of the held record until the notification is discarded.
errorCodes:
  - code: vta/notifications/dismiss:notificationUnknown
    meaning: "`recordId` names no notification the VTA holds in `contextId`."
    retryable: false
related:
  - vta/notifications/list
  - vta/notifications/show
  - notifications/notify
---

## Abstract

The **VTA Notifications — Dismiss** Trust Task sets one notification a VTA holds for its principal
to `dismissed`, so that [`vta/notifications/list`](../../list/0.1/spec.md) leaves it out unless
asked for dismissed ones. Dismissal is the principal's alone and changes nothing outside the VTA:
the originator, any relayer and the thing's holder are told nothing, and the notification stays
readable until it lapses (VTI-NTF-052). A later revision of a dismissed notification replaces the
notification's members and leaves it dismissed.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` a caller without standing under [Authorization](#authorization).
3. Refuse with `vta/notifications/dismiss:notificationUnknown` a `recordId` it holds no notification
   under in `contextId`, including one it holds in another context (VTI-NTF-051).
4. Set the record's `state` to `dismissed` and its `dismissedAt` to now, and answer `changed`
   `true`; or, where it is already dismissed, change nothing and answer `changed` `false`.
5. **MUST NOT** send anything to the originator, a relayer or the thing's holder because the
   notification was dismissed (VTI-NTF-053), and **MUST NOT** dismiss a notification on any
   party's behalf but the principal's or a party the principal authorized (VTI-NTF-052).

**A repeated request** sets a state: dismissing a dismissed notification succeeds, with `changed`
`false`.

**Retry class** (VTI-OPS-060): the second — a durable effect whose repeat is harmless.

## Authorization

**Standing in the trust context** named by `contextId`, as for
[`vta/notifications/list`](../../list/0.1/spec.md#authorization): the principal, or a party the
principal authorized in that context. Dismissal is a change to the principal's own view and to
nothing else.

## Definitions

- **`contextId`** — the trust context the notification is held in.
- **`recordId`** — the VTA's identifier for it.
- **`changed`** (response) — whether this request dismissed it.

## Request

The client sends its VTA the top-level schema in [`payload.schema.json`](payload.schema.json).

### Dismiss an election the member will not take part in

```json
{
  "id": "urn:uuid:3f4a5b60-7c8d-4e9f-8a0b-2c3d4e5f6a71",
  "type": "https://trusttasks.org/spec/vta/notifications/dismiss/0.1",
  "issuer": "did:key:z6MkWalletClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2027-03-02T08:05:00Z",
  "payload": {
    "contextId": "personal",
    "recordId": "hn-4Tq8Zr2Lw9Ks6Pm1"
  }
}
```

## Response

The VTA answers with the sub-schema reachable via `$anchor: "response"`.

### Dismissed

```json
{
  "id": "urn:uuid:4a5b6c70-8d9e-4f0a-9b1c-3d4e5f6a7b82",
  "type": "https://trusttasks.org/spec/vta/notifications/dismiss/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkWalletClient",
  "issuedAt": "2027-03-02T08:05:01Z",
  "threadId": "urn:uuid:3f4a5b60-7c8d-4e9f-8a0b-2c3d4e5f6a71",
  "payload": {
    "recordId": "hn-4Tq8Zr2Lw9Ks6Pm1",
    "changed": true
  }
}
```

## Security & Privacy

### Data carried

A context and a record identifier in; whether anything changed out.

### Correlation

Nothing leaves the VTA. In particular, that a member dismissed an election is never visible to the
community or the party that relayed it, so dismissal cannot be read as a vote, a refusal or a
signal of interest.

### Retention

The dismissal lives in the held record and goes when the VTA discards the notification after it
lapses (VTI-NTF-052).

### Consent/purpose

Dismissal serves the principal's own view and nothing else, and is not a decision about the thing:
a dismissed election can still be opened, and the member can still take part in it.
