---
slug: vtc/events/registrations/list
version: '0.4'
title: VTC Events — Registrations — List
summary: 'List an event''s registrations as the caller may see them: all of them, answers included, for its hosts; on a visible roster, who else is going or maybe for a registrant who is; on a hidden roster, nothing for anyone but the hosts.'
status: draft
targetFrameworkVersion: 0.6.0
category: governance
keywords:
- vtc
- events
- calendar
- registration
- roster
parties:
- role: host or registrant
  requirement: REQUIRED
  member: issuer
  identifierScope: any
- role: community maintainer
  requirement: REQUIRED
  member: recipient
  identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: What is answered — every registrant and their answers, the others going, or nothing — depends entirely on who asks; an unattributable caller gets nothing.
sideEffects:
  level: none
  rationale: Reads an event's registrations; persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: Discloses registrants by community-scoped entry identifier — or, for a member registered under a persona, by the persona identifier and never the member — and, to hosts alone, their answers and the identifiers of registrants who are not members.
retention:
  class: transient
  rationale: A read.
errorCodes:
- code: vtc/events/registrations/list:rosterHidden
  meaning: The caller may see the event but is not a host, and its roster is `hidden` (VTI-EVT-053). Nobody but the hosts reads who is registered.
  retryable: false
- code: vtc/events/registrations/list:notRegistered
  meaning: The event's roster is `visible`, but the caller is not going or maybe, and is not a host. The roster is shown to those on it.
  retryable: false
- code: vtc/events/registrations/list:filterNotPermitted
  meaning: '`statuses` was given by a caller who gets the roster view, which is always going and maybe.'
  retryable: false
- code: vtc/events/registrations/list:occurrenceUnknown
  meaning: "`occurrence` is not an original start the series' rule gives, or the event is not a series."
  retryable: false
knownImplementations:
  - https://github.com/OpenVTC/verifiable-trust-infrastructure
related:
- vtc/events/registrations/decide
- vtc/events/registrations/remove
- vtc/events/registrations/register
- vtc/events/show
---

## Abstract

The **VTC Events — Registrations — List** Trust Task reads who is registered for an event, as the event's roster setting lets the caller see it (VTI-EVT-050, VTI-EVT-053):

- The event's **hosts** get the **host view**: every registration in every status, with answers, acknowledgements and waitlist positions — what they need to run the event and decide pending registrations (VTI-EVT-073).
- On a **`visible`** roster, a registrant who is going or maybe gets the **roster view**: the others going or maybe, by entry — or, for a member registered under a persona, by the persona identifier and never the member (VTI-EVT-230) — with their status and guests, and nothing more.
- On a **`hidden`** roster, nobody but the hosts reads anything here.

The roster setting is fixed when the event is published and recorded on every registration (VTI-EVT-051, VTI-EVT-052), so what this task answers never widens under a registrant.

A single registration has no read of its own: its registrant reads it with [`vtc/events/show`](../../../show/0.4/spec.md) (`myRegistration`) or [`vtc/events/mine/list`](../../../mine/list/0.3/spec.md), and a host acts on one by identifier with [`vtc/events/registrations/decide`](../../decide/0.3/spec.md) and [`remove`](../../remove/0.3/spec.md), whose answers carry it. Existence of another member's registration is never load-bearing for a caller, which is why a filter suffices here.

## Status of this Document

Version 0.4 names a member registered under a persona in the roster view by the persona identifier (VTI-EVT-230), where 0.3's roster item could name a registrant only by entry and so could count a persona registration but never show it. Everything else is 0.3's.

Version 0.2 counts partners' members in the roster view without naming them (VTI Community Events chapter, VTI-EVT-100 – 135). Its Event and Registration are `vtc/events/_shared/0.2`'s.

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

The design it implements is recorded in the VTC events design note (Verifiable Trust Infrastructure, `docs/05-design-notes/vtc-events.md`), decision 16.

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **consumer** (the community) **MUST**:

1. Refuse a caller who is not an active member with `permissionDenied`.
2. Answer `notFound` alike for an event that does not exist and one the caller may not see — a draft to anyone but its hosts, or an event whose audience does not admit the caller (VTI-EVT-023) — and while the community has events turned off (VTI-EVT-007).
3. Give a caller who hosts the event — `vtc.events.manage` covering it (VTI-EVT-070) — the host view: every registration matching `statuses`, or every status without it, whole, with `waitlistPosition` on those waitlisted (VTI-EVT-073).
4. Otherwise, on a `hidden` roster, refuse with `rosterHidden` (VTI-EVT-053).
5. Otherwise, on a `visible` roster, give a caller whose own registration is `going` or `maybe` the roster view: every registration that is `going` or `maybe`, the caller's included, as `EventRosterEntry` — who, status, guests — and **MUST NOT** answer any other member of a registration (VTI-EVT-053). Refuse `statuses` there with `filterNotPermitted`, and any other caller with `notRegistered`.
6. **MUST NOT** leave a registrant out of the roster view at the registrant's request: a `visible` roster lists everyone going or maybe (VTI-EVT-055). A member who does not want to be recognised registers under another persona.
7. Never answer answers or `codeOfConductAcknowledged` to anyone but the hosts and the registrant (VTI-EVT-034); answers deleted on the retention schedule are absent (VTI-EVT-090).
8. Page as [`vtc/badges/list`](../../../../badges/list/0.1/spec.md) does: an opaque cursor bound to the caller, the event and the filters, 50 items when `limit` is absent.
9. In the roster view, leave out registrants who are not members — partners' members and guests — and answer how many are going or maybe in `othersGoing`; in the host view, answer them with `registrantDid` (VTI-EVT-134, VTI-EVT-153).
10. In the roster view, name a member registered by the identifier they are a member by with `entryId`, and a member registered under a persona with `registrantDid`, the persona identifier — exactly one of the two. A persona registration **MUST NOT** be named by `entryId`, nor answered with anything from which the member could be told: the roster and the hosts show the persona and never the member (VTI-EVT-230). A persona registration is not counted in `othersGoing`.
11. Never show an anonymous registration (VTI-EVT-213) in the roster view, by `registrantDid` or otherwise: an `anonymous` roster is `hidden` and more (VTI-EVT-050, VTI-EVT-210), so a caller who is not a host is refused with `rosterHidden`.

### Series and anonymity (0.3)

0.3 adds an `occurrence` filter; registrations carry `occurrence` where the series is registered by occurrence.

### Personas on the roster (0.4)

0.4's roster item names a registrant by exactly one of `entryId` and `registrantDid`. `registrantDid` is only ever a persona identifier (VTI-EVT-230): partners' members and guests stay counted in `othersGoing`, never named (VTI-EVT-134, VTI-EVT-153).

## Authorization

**Hosting the event, or being on its visible roster.** The host view is authorized by `vtc.events.manage` covering the event — a host's grant qualified `event:<eventId>`, or the authority over its calendar (VTI-EVT-013, VTI-EVT-070) — read from the caller's access-control entry at execution time. The roster view is authorized by the caller's own `going` or `maybe` registration on a `visible` roster. The proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10).

Reading the roster as a registrant is an act of the member (VTI-EVT-095): a member portal that holds a key the member authorized to sign for them for `vtc/events` (`auth/oob` `signingScope`) **MAY** sign it, and it is then answered the roster view at most. The host view is a host's act and **MUST NOT** be answered to a document signed by a key the portal holds for a member.

## Definitions

- `EventId`, `Registration`, `RegistrationStatus`, `RosterSetting` — see [`event.schema.json`](../../../_shared/0.4/event.schema.json).
- `EntryId` — see the directory family's [`directory-card.schema.json`](../../../../directory/_shared/0.1/directory-card.schema.json).
- **Host view**, **roster view** — see the Abstract.

## Request

The host or registrant (`issuer`) sends the community maintainer (`recipient`) a document whose `payload` validates against the top-level schema in [`payload.schema.json`](payload.schema.json).

### A host reads who is waiting for a decision

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000441",
  "type": "https://trusttasks.org/spec/vtc/events/registrations/list/0.4#request",
  "issuer": "did:example:priya",
  "recipient": "did:example:riverside-makers",
  "issuedAt": "2026-11-21T08:00:00Z",
  "payload": {
    "eventId": "evt_IntroSoldering2611",
    "statuses": ["pending"]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:example:priya#key-1",
    "created": "2026-11-21T08:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z3ExampleProofValueNotReal"
  }
}
```

## Response

The community maintainer answers with a document of type `https://trusttasks.org/spec/vtc/events/registrations/list/0.4#response`, whose `payload` validates against the `$anchor: "response"` sub-schema in [`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` ([SPEC §8](/SPEC.md#8-error-responses)), never a `#response` document.

### A host reads who is waiting for a decision — answered

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000442",
  "type": "https://trusttasks.org/spec/vtc/events/registrations/list/0.4#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:priya",
  "issuedAt": "2026-11-21T08:00:01Z",
  "threadId": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000441",
  "payload": {
    "eventId": "evt_IntroSoldering2611",
    "view": "host",
    "registrations": [
      {
        "registrationId": "reg_Kf3nP8qLs2Vx7Bd1",
        "eventId": "evt_IntroSoldering2611",
        "entryId": "dir_AdaLovelace000000",
        "status": "pending",
        "guests": 1,
        "answers": { "dietary": "Vegetarian", "step-free": true },
        "codeOfConductAcknowledged": "3a7bd3e2360a3d29eea436fcfb7e44c735d117c42d1c1835420b6b9942dd4f1b",
        "roster": "hidden",
        "eventSequence": 1,
        "createdAt": "2026-11-20T19:02:11Z",
        "updatedAt": "2026-11-20T19:02:11Z"
      }
    ]
  }
}
```

### A registrant on a visible roster — the roster view

```json
{
  "id": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000444",
  "type": "https://trusttasks.org/spec/vtc/events/registrations/list/0.4#response",
  "issuer": "did:example:riverside-makers",
  "recipient": "did:example:kofi",
  "issuedAt": "2026-12-02T18:00:01Z",
  "threadId": "urn:uuid:5c2e8a71-3d4b-4f6a-9b1c-000000000443",
  "payload": {
    "eventId": "evt_BuildNight0312Thurs",
    "view": "roster",
    "roster": [
      { "entryId": "dir_AdaLovelace000000", "status": "going", "guests": 0 },
      { "entryId": "dir_KofiOsei000000000", "status": "going", "guests": 1 },
      { "entryId": "dir_MeiRamos000000000", "status": "maybe", "guests": 0 },
      { "registrantDid": "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH", "status": "going", "guests": 0 }
    ]
  }
}
```

## Security & Privacy

### Data carried

An event and a status filter in; registrations out, as the caller may see them. Registrants are named by community-scoped `entryId`; the one exception is a member registered under a persona, named by the persona identifier they chose to be seen as (VTI-EVT-230), never by the identifier they are a member by; what a reader may see *of* an entry — a name, a picture — is the directory's projection ([`vtc/directory/entries/show`](../../../../directory/entries/show/0.1/spec.md)), not this task's.

### Correlation

The community maintainer's identifier scope is public: it is addressed by the public DID its members already know it by. The roster view tells registrants on a `visible` roster who else is coming, which is what that setting promised them before they registered (VTI-EVT-051, VTI-EVT-052); a `hidden` roster tells them nothing, and no count either except as [`vtc/events/show`](../../../show/0.4/spec.md) states it under the event's `showCount` and threshold (VTI-EVT-054).

### Retention

Nothing beyond the audit record of the request.

### Consent/purpose

Running an event (the host view) and seeing who else is coming to one whose roster was promised visible (the roster view). Answers are about members as people and are read by the hosts alone. **An event confers nothing** (VTI-EVT-002).
