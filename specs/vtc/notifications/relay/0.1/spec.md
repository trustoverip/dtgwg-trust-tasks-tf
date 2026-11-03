---
slug: vtc/notifications/relay
version: "0.1"
title: "VTC Notifications — Relay"
summary: "An administrator, shown what it discloses, decides that the community relays a notification it received to its members — all, one group's or one role's — and the community relays the originator's document unchanged to each."
status: draft
targetFrameworkVersion: "0.6.0"
category: notifications
keywords:
  - notifications
  - relay
  - vtc
  - administrator
  - election
parties:
  - role: administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: community (the addressee and relayer)
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    A relay decision discloses to members something about the community — for an election, that it
    is a member of the community holding it — and cannot be taken back once the relays are sent. It
    must be attributable to the administrator who made it, and to their acknowledgement of what it
    discloses, after the transport has closed.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A decision replayed against a
    later revision under the same record would relay it to members nobody decided to tell.
sideEffects:
  level: destructive
  rationale: >-
    Queues one notifications/relay to every member the selection reaches. Irreversible: what each
    member is told — that the thing exists and, for the kind, what a relay discloses about the
    community — cannot be recalled. A later withdrawal tells them the thing is withdrawn; it does not
    untell them that the community passed it on.
consequences:
  - "Each member the selection reaches learns that the community received the notification and passed it on — for an election, that the community is a member of the community holding it."
  - "Every later revision of the notification, a withdrawal included, is relayed to the same members without a further decision."
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: a record identifier, a selection of members by scope, and the acknowledgement. Out: how many
    parties the decision added. The members reached are never returned.
retention:
  class: durable
  rationale: >-
    The decision is a governance record — who decided, when, which selection, how many, and the
    acknowledgement — and the community keeps whom it relayed to until the notification's period ends,
    for relaying later revisions and nothing else (VTI-NTF-044).
errorCodes:
  - code: vtc/notifications/relay:notificationUnknown
    meaning: "`recordId` names no notification the community holds — never held, or discarded after it lapsed."
    retryable: false
  - code: vtc/notifications/relay:notRelayable
    meaning: >-
      The community may not relay the notification: its `relayable` is false, its kind may not be
      relayed, or the chain it arrived by already carries four relays, so that the community's own
      would make more than four (VTI-NTF-041).
    retryable: false
  - code: vtc/notifications/relay:lapsed
    meaning: "The notification is withdrawn, or its `validUntil` has passed: there is nothing to tell members of."
    retryable: false
  - code: vtc/notifications/relay:disclosureNotAcknowledged
    meaning: >-
      The notification's kind says a relay discloses something about the community, and the request
      does not carry `disclosureAcknowledged` true (VTI-NTF-046). `details` carries the statement to
      show the administrator.
    retryable: false
    detailsSchema:
      type: object
      additionalProperties: false
      required:
        - relayDisclosure
      properties:
        relayDisclosure:
          type: string
          minLength: 1
          maxLength: 1024
  - code: vtc/notifications/relay:groupUnknown
    meaning: "`recipients.groupId` names no community group, or one the caller may not see."
    retryable: false
related:
  - vtc/notifications/list
  - vtc/notifications/show
  - notifications/relay
  - notifications/notify
  - vtc/groups/show
---

## Abstract

The **VTC Notifications — Relay** Trust Task is a community administrator's decision to pass on a
notification the community received. The administrator names the received notification and a
selection of the community's members — **all of them**, the members of **one group**, or the
holders of **one community role** — and acknowledges what a relay of it discloses about the
community (VTI-NTF-046). The community resolves the selection from its own records, and relays the
document it received **exactly as received** to each member it reaches, one
[`notifications/relay`](../../../../notifications/relay/0.1/spec.md) each, signed as itself. It
never relays to itself, to the notification's originator or to any party that relayed it on the way
here (VTI-NTF-047), and it relays every later revision, a withdrawal included, to the same members
without a further decision (VTI-NTF-044).

This is the administrator surface VTI-NTF-071 requires: a community relays a notification only on
such a decision. An election put by another community to this community's members is the case it
was made for: the holding community's list never shows the election to them, and only this
community holds their roster (VTI-NTF-061).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **producer** (the administrator's client) **MUST** show the administrator the
notification's `relayDisclosure`, where present, before it sends a decision carrying
`disclosureAcknowledged` true, and **MUST NOT** set it true on the administrator's behalf
(VTI-NTF-046).

A conforming **consumer** (the community) **MUST**:

1. Validate the document per [SPEC §7.2](/SPEC.md#72-consumer-requirements), and refuse with
   `permissionDenied` a caller without the authority in [Authorization](#authorization) for the
   selection named, read from the signer's live access-control entry at execution time.
2. Refuse with `notificationUnknown` a `recordId` it holds no notification under; with
   `notRelayable` a notification it may not relay ([`notifications/relay`](../../../../notifications/relay/0.1/spec.md)
   producer item 2); with `lapsed` one that is withdrawn or past its `validUntil`; and with
   `groupUnknown` a `groupId` that names no group the caller may see.
3. Refuse with `disclosureNotAcknowledged`, carrying the statement in `details`, a decision for a
   notification whose kind says a relay discloses something about the relayer, unless
   `disclosureAcknowledged` is `true` (VTI-NTF-046).
4. Resolve `recipients` from its own records when the decision is made (VTI-NTF-072): every member
   whose membership is active, for `allMembers`; the active members of the group, for `group`; the
   active members holding the role, for `role`. It **MUST** leave out itself, the notification's
   originator, every relayer the received chain names, and every party it already relays the
   notification to (VTI-NTF-047), and **MUST NOT** add a member who joins the selection later
   without a new decision.
5. Record the decision and the parties it added before it sends anything, then relay the document
   it received, unchanged, to each, one `notifications/relay/0.1` per party, signed as itself, through
   the transport each party's DID document advertises (VTI-NTF-020). A party whose relay cannot be
   delivered stays recorded and is retried by the delivery layer; a crash between recording and
   sending sends the same revision again, which a recipient holds as the one notification it is
   (VTI-NTF-022).
6. Relay every later revision of the notification it accepts, a withdrawal included, to every party
   any decision added, until the notification's period ends — or, without one, for the period the
   community states (VTI-NTF-044) — and use that record for nothing else.
7. Audit the decision (VTI-NTF-073): the administrator, the notification's originator and
   `notificationId`, the revision, the selection, `recipientCount`, and the acknowledgement. The audit
   records **MUST NOT** name the parties reached.
8. **MUST NOT** tell the originator, or any earlier relayer, that it relayed the notification, to
   whom or to how many (VTI-NTF-043).

**Retry class** (VTI-OPS-060): the second — a durable effect whose repeat is harmless. A repeat of
the same signed decision is answered with the answer first given
([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 11); a new decision with the same selection
reaches no party consumer item 4 did not already leave out, and answers `recipientCount` 0.

**A repeated request** sets a state: relaying to a selection already reached succeeds with
`recipientCount` 0 and sends nothing.

## Authorization

The authority is the **`vtc.notifications.relay` capability at the community** (VTI-NTF-071), read
from the signer's access-control entry at execution time, for the selection named. The capability is
**qualified by group**: unqualified, it permits every selection; qualified `group:<groupId>`, it
permits only `scope` `group` naming that group, so that a community can let a group's manager pass a
notification to their own group and no further. The proof establishes who asked, never that they
may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The capability is not
authority-conferring: a relay grants its recipients nothing (VTI-NTF-004). Whether a decision — in
particular one that reaches every member — takes a step-up or a second administrator's consent is
the community's policy ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13); this task is
`destructive` because what members are told cannot be recalled.

## Definitions

- **`recordId`** — the received notification to relay.
- **`recipients`** — the selection ([`Recipients`](../../_shared/0.1/received-notification.schema.json)):
  `scope` `allMembers`, `group` with `groupId`, or `role` with `role`.
- **`disclosureAcknowledged`** — the administrator's statement that they were shown what the relay
  discloses.
- **`revision`** (response) — the revision relayed.
- **`recipientCount`** (response) — the parties this decision added.

## Request

The administrator sends the community the top-level schema in [`payload.schema.json`](payload.schema.json).

### Relay an election put to this community's members to all of them

```json
{
  "id": "urn:uuid:3a4b5c6d-7e8f-4a9b-8c0d-1e2f3a4b5c6d",
  "type": "https://trusttasks.org/spec/vtc/notifications/relay/0.1",
  "issuer": "did:web:carol.example",
  "recipient": "did:web:partner.example",
  "issuedAt": "2027-03-01T10:05:00Z",
  "payload": {
    "recordId": "rn-4Tq8Zr2Lw9Ks6Pm1",
    "recipients": {
      "scope": "allMembers"
    },
    "disclosureAcknowledged": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-03-01T10:05:00Z",
    "verificationMethod": "did:web:carol.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z5Ef..."
  }
}
```

### Relay it to one group only

```json
{
  "id": "urn:uuid:4b5c6d7e-8f9a-4b0c-9d1e-2f3a4b5c6d7e",
  "type": "https://trusttasks.org/spec/vtc/notifications/relay/0.1",
  "issuer": "did:web:dave.example",
  "recipient": "did:web:partner.example",
  "issuedAt": "2027-03-01T10:06:00Z",
  "payload": {
    "recordId": "rn-4Tq8Zr2Lw9Ks6Pm1",
    "recipients": {
      "scope": "group",
      "groupId": "grp_Q2xpbWJpbmdDbHViMjAyNw"
    },
    "disclosureAcknowledged": true
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2027-03-01T10:06:00Z",
    "verificationMethod": "did:web:dave.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "z6Gh..."
  }
}
```

## Response

The community answers with the sub-schema reachable via `$anchor: "response"` once it has recorded
the decision and queued the relays.

### Relayed to 42 members

```json
{
  "id": "urn:uuid:5c6d7e8f-9a0b-4c1d-8e2f-3a4b5c6d7e8f",
  "type": "https://trusttasks.org/spec/vtc/notifications/relay/0.1#response",
  "issuer": "did:web:partner.example",
  "recipient": "did:web:carol.example",
  "issuedAt": "2027-03-01T10:05:02Z",
  "threadId": "urn:uuid:3a4b5c6d-7e8f-4a9b-8c0d-1e2f3a4b5c6d",
  "payload": {
    "recordId": "rn-4Tq8Zr2Lw9Ks6Pm1",
    "revision": 1,
    "recipientCount": 42
  }
}
```

## Security & Privacy

### Data carried

A record identifier, a selection by scope, and an acknowledgement in; a count out. The relays the
community sends carry the originator's document whole and nothing of the community's own
(VTI-NTF-041). The decision never names members, so neither the request nor the response is a
roster.

### Correlation

The community's `identifierScope` is `public`: it is named by its public DID because it is the
party the notification was addressed to, and the relays it sends are signed as that DID, which is
what lets each member verify that the relayer was the notification's addressee (VTI-NTF-042). A
pairwise identifier would break that check. The administrator's identifier is pairwise.

Each member reached learns that the community received the notification and passed it on — for an
election, that the community is a member of the community holding it, which the community's own
directory listing there may otherwise keep from them (VTI-NTF-046, VTI-DIR-040). That is the
disclosure the administrator acknowledged. Nothing goes back to the originator or any earlier
relayer (VTI-NTF-043), and the members reached are never returned to the administrator, recorded in
the audit trail or used for anything but relaying later revisions to them (VTI-NTF-044).

### Retention

The decision is kept as a governance record. The record of whom the community relayed to is kept
until the notification's period ends, or for the period the community states for a notification
without one, and then discarded (VTI-NTF-044).

### Consent/purpose

The relay serves the notification's own purpose — telling the parties the community stands for that
the thing exists — and no other. The community **MUST NOT** use the relay to learn anything of its
members, whether they opened the thing or took part in it, beyond each relay's receipt.
