---
slug: vta/notifications/list
version: "0.1"
title: "VTA Notifications — List"
summary: "A client lists the notifications its VTA holds for the principal in one trust context — filtered by kind or holder — which, for elections, is how a member sees what they have been told they may take part in."
status: draft
targetFrameworkVersion: "0.6.0"
category: notifications
keywords:
  - notifications
  - inbox
  - vta
  - election
  - eligibility
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
    and its own VTA. Each item names parties the VTA verified; a client that will not take the VTA's
    word reads the document itself with vta/notifications/show.
sideEffects:
  level: none
  rationale: "Read-only enumeration of held notifications."
subjectPath: /contextId
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Out: for each held notification, who originated and relayed it, where the thing is held and the
    audience it concerns — which says what the principal belongs to, but carries no thing's content,
    because no notification does. In: a context and filters.
retention:
  class: exchange
  rationale: >-
    The VTA remains the record; the page is the client's working copy for the view it renders.
errorCodes: []
related:
  - notifications/notify
  - notifications/relay
  - vta/notifications/show
  - vta/notifications/dismiss
  - vtc/elections/list
---

## Abstract

The **VTA Notifications — List** Trust Task returns the notifications a VTA holds for its
principal in one trust context: those it accepted, directly by
[`notifications/notify`](../../../../notifications/notify/0.1/spec.md) or by
[`notifications/relay`](../../../../notifications/relay/0.1/spec.md), for a DID of that context
(VTI-NTF-050). It filters by kind and by the DID holding the thing, and leaves out dismissed and
lapsed notifications unless asked for them.

For elections, this list with `kind` `election` is half of the member's view of the elections
they may take part in; the other half is what each community's
[`vtc/elections/list`](../../../../vtc/elections/list/0.5/spec.md) shows the member. An election put
to the members of a party that is itself a member of the community appears only in the first,
because the community's list never shows it (VTI-VOT-015, VTI-NTF-063).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

A filter suffices for this collection without a single-record twin for existence: the split pair is
this task and [`vta/notifications/show`](../../show/0.1/spec.md), which reads one record by
`recordId` and refuses an unknown one.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **consumer** (the VTA) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements).
2. Refuse with `permissionDenied` a caller without standing under [Authorization](#authorization),
   and answer nothing about any other context (VTI-NTF-051).
3. Return the held notifications of `contextId` matching every filter given, most recently
   received first: of `kind` only, where given; of `subject.heldBy` equal to `heldBy` only, where
   given; dismissed ones only where `includeDismissed` is `true`; and withdrawn ones and those whose
   `validUntil` has passed only where `includeLapsed` is `true`.
4. Page with `cursor` and `limit`, refusing with `malformedRequest` a `cursor` it did not issue for
   the same filters.
5. **MUST NOT** send anything to an originator, a relayer or a thing's holder because a client
   listed its notifications (VTI-NTF-053).

**Retry class** (VTI-OPS-060): the first — no durable effect.

## Authorization

**Standing in the trust context** named by `contextId`: an access control entry whose act scope
covers that context ([VTI-ACL](https://trustoverip.github.io/dtgwg-vti-spec/)), the same standing
that reads the context's credentials. The notifications are the principal's, held in the context of
the DID they were addressed to, so a party without that standing learns nothing of them.

## Definitions

- **`contextId`** — the trust context to list.
- **`kind`**, **`heldBy`** — optional filters.
- **`includeDismissed`**, **`includeLapsed`** — optional widenings; absent reads as `false`.
- **`cursor`**, **`limit`** — paging.
- **`items`** (response) — the held notifications
  ([`HeldNotification`](../../_shared/0.1/held-notification.schema.json)).
- **`nextCursor`** (response) — present when more remain.

## Request

The client sends its VTA the top-level schema in [`payload.schema.json`](payload.schema.json).

### The elections a member has been told of

```json
{
  "id": "urn:uuid:0c9a6e41-2b7d-4f13-8e5c-a1d2f3b4c5e6",
  "type": "https://trusttasks.org/spec/vta/notifications/list/0.1",
  "issuer": "did:key:z6MkWalletClient",
  "recipient": "did:web:vta.example",
  "issuedAt": "2027-03-02T08:00:00Z",
  "payload": {
    "contextId": "personal",
    "kind": "election"
  }
}
```

## Response

The VTA answers with the sub-schema reachable via `$anchor: "response"`.

### One election, relayed by the member's party

```json
{
  "id": "urn:uuid:6f1b2c3d-4e5f-4a6b-9c7d-8e9f0a1b2c3d",
  "type": "https://trusttasks.org/spec/vta/notifications/list/0.1#response",
  "issuer": "did:web:vta.example",
  "recipient": "did:key:z6MkWalletClient",
  "issuedAt": "2027-03-02T08:00:01Z",
  "threadId": "urn:uuid:0c9a6e41-2b7d-4f13-8e5c-a1d2f3b4c5e6",
  "payload": {
    "items": [
      {
        "recordId": "hn-4Tq8Zr2Lw9Ks6Pm1",
        "contextId": "personal",
        "addressee": "did:key:z6MkMember",
        "originator": "did:web:community.example",
        "relayedBy": ["did:web:partner.example"],
        "notification": {
          "notificationId": "ntf-7Qk2p9Xv3mR8",
          "revision": 1,
          "kind": "election",
          "subject": {
            "heldBy": "did:web:community.example",
            "id": "board-2027",
            "openWith": "https://trusttasks.org/spec/vtc/elections/show/0.6"
          },
          "audience": "did:web:partner.example",
          "status": "available",
          "relayable": true,
          "validFrom": "2027-03-08T00:00:00Z",
          "validUntil": "2027-03-15T00:00:00Z"
        },
        "state": "active",
        "receivedAt": "2027-03-01T11:20:01Z"
      }
    ]
  }
}
```

## Security & Privacy

### Data carried

Each item says who originated a notification, who relayed it, where its thing is held and whom it
concerns — together, a map of the communities, groups and networks the principal belongs to and
what they have been told of. No item carries a thing's content, because no notification does
(VTI-NTF-003).

### Correlation

The response joins every notification of a context into one view, which is its purpose; it goes
only to a party with standing in that context and never across contexts (VTI-NTF-051), so a
principal who keeps memberships in separate contexts keeps them apart here too.

### Retention

The VTA is the record. A client keeps the page for the view it renders and **SHOULD NOT** keep it
past that, since a dismissal or a withdrawal at the VTA does not reach a copy.

### Consent/purpose

The list shows the principal what they have been told they may take part in. A client
**MUST NOT** open a listed thing — for an election, read it from the community, which may present
the member's credential — except at the principal's choice (VTI-NTF-053, VTI-VOT-016).
