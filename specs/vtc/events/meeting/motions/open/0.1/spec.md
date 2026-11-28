---
slug: vtc/events/meeting/motions/open
version: '0.1'
title: VTC Events — Meeting — Motions — Open
summary: 'The chair opens a motion while quorum holds: a community election whose roll is the members present at opening.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
parties:
- role: chair or host
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
  rationale: 'Creates a community election through the community''s registrar and records the motion naming it.'
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: 'A motion''s title and election template; the motion and its election''s identifier.'
retention:
  class: durable
  rationale: 'The motion is kept with the meeting; its election is the registrar''s.'
errorCodes:
- code: vtc/events/meeting/motions/open:notAMeeting
  meaning: 'The event is not of kind `meeting`; this task applies to meetings only (VTI-EVT-255).'
  retryable: false
- code: vtc/events/meeting/motions/open:eventCancelled
  meaning: 'The meeting is cancelled.'
  retryable: false
- code: vtc/events/meeting/motions/open:notDuringMeeting
  meaning: 'The meeting is not running now (VTI-EVT-262).'
  retryable: false
- code: vtc/events/meeting/motions/open:quorumNotMet
  meaning: 'Quorum is not met now, or the meeting has no quorum rule (VTI-EVT-261).'
  retryable: false
  detailsSchema: {"type": "object", "additionalProperties": false, "properties": {"present": {"type": "integer", "minimum": 0}, "required": {"type": "integer", "minimum": 0}}}
- code: vtc/events/meeting/motions/open:templateInvalid
  meaning: 'The election template is not one the community''s registrar accepts.'
  retryable: false
  detailsSchema: {"type": "object", "additionalProperties": false, "properties": {"reason": {"type": "string", "maxLength": 500}}}
- code: vtc/events/meeting/motions/open:electionRefused
  meaning: 'The community''s registrar or its ballot box refused the election; its own code is in `details`.'
  retryable: false
  detailsSchema: {"type": "object", "additionalProperties": false, "properties": {"code": {"type": "string", "maxLength": 128}, "message": {"type": "string", "maxLength": 500}}}
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/meeting/show
- vtc/elections/create
---

## Abstract

The **VTC Events — Meeting — Motions — Open** Trust Task puts a motion to the meeting (VTI-EVT-262): the community's registrar creates an election from the template, enfranchising the members present at opening, and the motion records the election's identifier. It is refused unless quorum is met at that moment; quorum lost later closes nothing already open.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), phase 6.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Answer `notFound` alike for an event that does not exist and one the caller may not see (VTI-EVT-023), and refuse an event of any kind but `meeting` with `notAMeeting` (VTI-EVT-255).
2. Refuse a caller who is neither the meeting's chair nor one of its hosts with `permissionDenied` (VTI-EVT-262).
3. Refuse a cancelled meeting with `eventCancelled`, one not running now with `notDuringMeeting`, and quorum not met — or no quorum rule — with `quorumNotMet`, carrying the counts (VTI-EVT-261).
4. Create the election through the community's registrar (VTI-VOT-*), with every check it makes, its roll the members present at opening as the roll call stands (roll rule `listed`), voting open now and closing at `votingClosesAt` or the meeting's end, and `binding: false` — a board's template has no `binding` member, and a binding decision is an election created through `vtc/elections/create` under its own consent (VTI-VOT-081). Refuse a template the registrar refuses with `templateInvalid`, a refusal by the registrar or its ballot box with `electionRefused`.
5. Record the motion naming the election before the election is created, and never copy the election's state onto the motion (VTI-EVT-262).
6. Audit the motion by reference (VTI-EVT-091).

Nothing this task records confers authority (VTI-EVT-002).

## Authorization

**Authority: the meeting's chair, or a host** (VTI-EVT-262), with the authority the registrar requires to create the election. Never from a portal-held key (VTI-EVT-095). The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

## Definitions

The shared definitions this task uses are in the schemas its [`payload.schema.json`](payload.schema.json) references.

## Request

The chair or host (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Example

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "type": "https://trusttasks.org/spec/vtc/events/meeting/motions/open/0.1#request",
  "issuer": "did:example:mary",
  "recipient": "did:example:community",
  "issuedAt": "2026-12-05T10:40:00Z",
  "payload": {
    "eventId": "evt_AGM2026HarbourRowing",
    "title": "Adopt the 2027 budget",
    "agendaItemId": "agi_Budget2027Item0001",
    "template": {
      "questions": [
        {
          "key": "adopt",
          "type": "choice",
          "text": "Adopt the 2027 budget?",
          "options": [
            {
              "key": "yes",
              "label": "Yes"
            },
            {
              "key": "no",
              "label": "No"
            }
          ],
          "minSelections": 1,
          "maxSelections": 1,
          "optional": false
        }
      ],
      "secrecy": "sealed",
      "revote": "replace",
      "board": {
        "visibility": "public",
        "auditors": []
      },
      "ballotBox": "did:example:ballot-box",
      "allowEarlyClose": true,
      "resultsBeforeClose": false,
      "textReleaseThreshold": 10
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

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/meeting/motions/open/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Example — answered

```json
{
  "id": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000602",
  "type": "https://trusttasks.org/spec/vtc/events/meeting/motions/open/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:mary",
  "issuedAt": "2026-12-05T10:40:01Z",
  "threadId": "urn:uuid:7d1e9a40-6c2b-4e5f-8a9b-000000000601",
  "payload": {
    "motion": {
      "motionId": "mot_AdoptBudget2027001",
      "title": "Adopt the 2027 budget",
      "agendaItemId": "agi_Budget2027Item0001",
      "electionId": "meeting.evt_AGM2026HarbourRowing.mot_AdoptBudget2027001",
      "openedAt": "2026-12-05T10:42:00Z",
      "presentAtOpening": 23
    }
  }
}
```

## Security & Privacy

### Data carried

A meeting, a motion's title and its election template; the motion.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. Members are named by their community-scoped `entryId`, never by DID.

### Retention

**Durable** for the motion; the election is kept by the registrar under its own rules.

### Consent/purpose

Deciding at a meeting. The election is the community's own, so the meeting adds a door to its voting, never a second one.
