---
slug: vtc/events/attachments/add
version: '0.1'
title: VTC Events — Attachments — Add
summary: 'A host attaches to an event a community room bound to its roster, an opt-in room, a board whose audience is the event, a community election, or a material released to each registrant who is going.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
parties:
- role: event host
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: 'What the caller may do or see depends on who asks; the request must be attributable after the transport has closed.'
issuedAtRequirement:
  requirement: REQUIRED
  rationale: 'Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed request would repeat a change.'
sideEffects:
  level: mutating
  rationale: 'Stores the attachment; for a bound room, the community starts offering admission to the registrants the binding admits. Recoverable with vtc/events/attachments/remove.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'Names a room, board or election by identifier, or a material by name and address; answers the attachment.'
retention:
  class: durable
  rationale: 'An attachment stands until removed or its event is deleted.'
errorCodes:
- code: vtc/events/attachments/add:eventsOff
  meaning: 'The community has events turned off (VTI-EVT-007). Nothing was stored.'
  retryable: false
- code: vtc/events/attachments/add:eventCancelled
  meaning: 'The event is cancelled; it takes no new attachment.'
  retryable: false
- code: vtc/events/attachments/add:rosterNotVisible
  meaning: 'A `room` was asked for on an event whose roster is `hidden` or `anonymous`: a shared room would show its registrants to each other (VTI-EVT-243). Release materials instead, or offer an `optInRoom` on a hidden roster.'
  retryable: false
- code: vtc/events/attachments/add:optInNotAllowed
  meaning: 'An `optInRoom` was asked for on an event whose roster is not `hidden` (VTI-EVT-244).'
  retryable: false
- code: vtc/events/attachments/add:maybeNotAdmitted
  meaning: '`admits: goingAndMaybe` was asked for on an event whose roster is not `visible` (VTI-EVT-241).'
  retryable: false
- code: vtc/events/attachments/add:roomNotManaged
  meaning: 'The room is not a community room the signer may manage (`vtc.rooms.manage` over it, VTI-EVT-240).'
  retryable: false
- code: vtc/events/attachments/add:boardAudienceMismatch
  meaning: 'The board''s audience is not this event (VTI-EVT-250).'
  retryable: false
- code: vtc/events/attachments/add:electionNotFound
  meaning: 'No community election has that identifier.'
  retryable: false
- code: vtc/events/attachments/add:alreadyAttached
  meaning: 'The room, board or election is already attached to the event.'
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/attachments/remove
- vtc/events/attachments/list
- vtc/groups/bindings/add
- vtc/boards/create
---

## Abstract

The **VTC Events — Attachments — Add** Trust Task attaches something to an event (VTI-EVT-240 – 252): a community room bound to the event's roster, so the registrants who are going are offered admission as a group's members are; an opt-in room on a `hidden` roster, entered only by a registrant's own act after a disclosure; a board whose audience is the event; a community election; or a material — a file or page — released to each registrant who is going, as access details are.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), phase 6.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for an event that does not exist and one the caller neither hosts nor may see, and `permissionDenied` to a caller who may see it but does not host it (VTI-EVT-070). Answer `notFound` while events are turned off (VTI-EVT-007).
2. Accept exactly the member `kind` names (`roomId` for `room` and `optInRoom`, `boardId`, `electionId`, `material`), and `disclosure` exactly for `optInRoom`; refuse any other combination with `malformedRequest`.
3. Refuse a `room` on a `hidden` or `anonymous` roster with `rosterNotVisible` (VTI-EVT-243); an `optInRoom` on any roster but `hidden` with `optInNotAllowed`, and on an `anonymous` event always (VTI-EVT-244); `admits: goingAndMaybe` on any roster but `visible` with `maybeNotAdmitted` (VTI-EVT-241).
4. For a room, require that the signer may manage the community room (`vtc.rooms.manage` over it) as well as host the event (VTI-EVT-240), else `roomNotManaged`; then bind it to the event's roster as a group binding binds a room (VTI-DIR-090 – 093): offer admission to each registrant the binding admits, admit only on their accepting with key material of their own, record the binding as a source of each admission.
5. For a board, refuse one whose audience is not this event — an audience of kind `event` (`vtc/boards/_shared/0.2`), which only `vtc/boards/create` and `update` 0.2 can set — with `boardAudienceMismatch` (VTI-EVT-250); for an election, one that does not exist with `electionNotFound`.
6. Refuse an attachment already present with `alreadyAttached`.
7. Audit the attachment by reference (VTI-EVT-091).

Nothing this task records confers authority (VTI-EVT-002).

## Authorization

**Authority: hosting the event** (VTI-EVT-070), and for a room also `vtc.rooms.manage` over it. A host's act: it **MUST NOT** be accepted from a key the portal holds for a member (VTI-EVT-095). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

The shared definitions this task uses are in the schemas its [`payload.schema.json`](payload.schema.json) references.

## Request

The event host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "type": "https://trusttasks.org/spec/vtc/events/attachments/add/0.1#request",
  "issuer": "did:example:mary",
  "recipient": "did:example:community",
  "issuedAt": "2026-12-05T10:40:00Z",
  "payload": {
    "eventId": "evt_IntroSoldering2611",
    "kind": "material",
    "material": {
      "name": "Kit list",
      "url": "https://riverside-makers.example/files/kit-list.pdf"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:mary#key-1",
    "created": "2026-12-05T10:40:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/attachments/add/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Example — answered

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000602",
  "type": "https://trusttasks.org/spec/vtc/events/attachments/add/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:mary",
  "issuedAt": "2026-12-05T10:40:01Z",
  "threadId": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "payload": {
    "attachment": {
      "attachmentId": "att_KitListSoldering01",
      "eventId": "evt_IntroSoldering2611",
      "kind": "material",
      "material": {
        "name": "Kit list",
        "url": "https://riverside-makers.example/files/kit-list.pdf"
      },
      "attachedAt": "2026-11-18T10:00:01Z"
    }
  }
}
```

## Security & Privacy

### Data carried

An event, the kind of attachment and what it names; the attachment.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID.

### Retention

**Durable**, until the attachment is removed or its event deleted. A bound room's admissions follow the roster.

### Consent/purpose

Giving an event its working space. A shared room shows its members to each other, which is why it is refused on a roster that promised otherwise.
