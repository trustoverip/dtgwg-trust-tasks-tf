---
slug: vtc/events/report
version: '0.1'
title: VTC Events — Report
summary: 'A member reports an event they may see to the community''s administrators — inappropriate, spam, unsafe, misleading or other — and the report goes on the administrators'' action list, with the reporter visible to administrators only.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
- moderation
- report
parties:
- role: member
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: A report puts an event before the administrators in a member's name; it must be attributable to the reporter, so one member cannot file reports as many, and so administrators can follow it up.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor. A replayed report would re-open a matter the administrators closed.
sideEffects:
  level: mutating
  rationale: Stores a report and puts it on the administrators' action list. Not withdrawable by the reporter; administrators close it.
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: The request carries a reason and a note about an event; the response is the report's identifier only.
retention:
  class: durable
  rationale: A report stands until an administrator closes it, and is kept in the audit after.
errorCodes:
- code: vtc/events/report:alreadyReported
  meaning: The caller has an open report of this event. Nothing was stored; the open report's identifier is in `details`.
  retryable: false
  detailsSchema:
    type: object
    additionalProperties: false
    required:
    - reportId
    properties:
      reportId:
        type: string
        pattern: ^rpt_[A-Za-z0-9_-]{16,64}$
- code: vtc/events/report:limitReached
  meaning: The caller has filed as many reports as the community allows in a day. Nothing was stored.
  retryable: true
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/show
- vtc/events/cancel
---

## Abstract

The **VTC Events — Report** Trust Task lets any member who may see an event bring it to the community's administrators (VTI-EVT-081): an event that is inappropriate, spam, unsafe or misleading, or something else the note explains. The report goes on the administrators' **action list**, where they decide what to do — speak to the hosts, change the event, cancel it — with the community's own tasks. A member has at most one open report of an event, and a community limits how many reports a member files in a day (VTI-EVT-082).

Who reported is **visible to administrators only**, never to the event's hosts or anyone else: a member who reports an event run by someone they know should not have to be identified to them.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), decision 25.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is not an active member with `permissionDenied`.
2. Answer `notFound` alike for an event that does not exist and one the caller may not see — a draft to anyone but its hosts, or an event whose audience does not admit the caller (VTI-EVT-020, VTI-EVT-023) — and while the community has events turned off (VTI-EVT-007). A member reports only what they may see.
3. Refuse a caller with an open report of the same event with `alreadyReported`, naming it (VTI-EVT-081), and a caller over the community's daily limit with `limitReached`. A report an administrator closed does not count as open.
4. Store the report — event, reason, note, reporter's entry, when — and put it on the administrators' action list.
5. Show who reported to administrators alone, and **MUST NOT** disclose it to the event's hosts, to other members, or in any answer of another task, unless the reporter is themselves an administrator acting as one (VTI-EVT-081).
6. Answer the report's identifier, and nothing about the event or what will be done.
7. Audit the report by reference — event, report, reporter (VTI-EVT-091) — with the audit readable as the administrators' action list is.

The operation budgets for senders and clients apply to this task as to every other (VTI-EVT-082).

## Authorization

**Seeing the event.** Any active member the event's audience admits may report it (VTI-EVT-020); no capability is involved, and a report confers nothing — it puts the event before those with authority to act, who decide under their own (VTI-EVT-002). The proof establishes who asked ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10), which is what makes one open report per member enforceable.

Reporting is an act of the member (VTI-EVT-095): a member portal that holds a key the member authorized to sign for them for `vtc/events` (`auth/oob` `signingScope`) **MAY** sign it.

## Definitions

- `EventId`, `ReportId` — see [`event.schema.json`](../../_shared/0.1/event.schema.json).
- **Action list** — the community's queue of things awaiting an administrator.

## Request

The member (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### Report a misleading event

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000501",
  "type": "https://trusttasks.org/spec/vtc/events/report/0.1#request",
  "issuer": "did:example:kofi",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-22T14:00:00Z",
  "payload": {
    "eventId": "evt_FreeLaserCutting001",
    "reason": "misleading",
    "note": "Listed as a free workshop; the description asks people to pay a third party to book."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:kofi#key-1",
    "created": "2026-11-22T14:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/report/0.1#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### Report a misleading event — answered

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000502",
  "type": "https://trusttasks.org/spec/vtc/events/report/0.1#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:kofi",
  "issuedAt": "2026-11-22T14:00:01Z",
  "threadId": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000501",
  "payload": {
    "reportId": "rpt_Q2w8Ez5Ry1Tu4Io7"
  }
}
```

## Security & Privacy

### Data carried

An event, a reason and a note in; a report identifier out.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. The reporter is identified to the community as its member, and to its administrators only.

### Retention

**Durable**: the report stands until an administrator closes it and is kept in the audit after.

### Consent/purpose

Letting members raise an event with those who can act on it, without facing its hosts. A report decides nothing: the administrators decide, with the community's own tasks, and a report is never read by an authorization decision (VTI-EVT-002).
