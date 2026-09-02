---
slug: vta/management/reload-services
version: "1.0"
title: "VTA Management — Reload Services"
summary: "Re-read the agent's service configuration and restart its transports."
status: draft
targetFrameworkVersion: "0.5.0"
category: governance
keywords:
  - operations
  - configuration
  - restart
parties:
  - role: agent operator
    requirement: REQUIRED
    member: issuer
  - role: verifiable trust agent
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    A reload drops every open session the agent is serving. That is a denial of service to all of its counterparties for as long as it takes to come back, so an unattributable request is an anonymous party able to interrupt an agent at will, repeatedly, leaving nothing to attribute the interruption to.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    The payload carries nothing, so every reload request is byte-identical apart from its envelope. Without a bound on freshness, one captured document is a reusable interrupt — replayed on a loop it holds the agent permanently down, and `issuedAt` is the only member that distinguishes a captured request from a current one.
sideEffects:
  level: mutating
  rationale: >-
    Re-reads configuration and restarts the agent's transports. No stored state is created, altered or destroyed, and configuration already on disk is applied rather than changed. What does not survive is everything in flight: open sessions drop, and work in progress at the moment of the restart is lost rather than resumed.
exposure:
  discloses: none
  ingests: none
  actsAsSubject: false
retention:
  class: transient
  rationale: >-
    The request carries nothing to keep. The recipient acts on it and retains only the fact that it was asked — which belongs in an audit trail, not in the operational state this task reloads.
errorCodes:
  - code: vta/management/reload-services:reloadFailed
    meaning: >-
      The recipient read its configuration and could not bring its services up on it. The agent's disposition afterwards is deployment-specific and this code does not assert one — see Failure leaves an indeterminate agent.
    retryable: true
related: []
---

## Abstract

The **VTA Management — Reload Services** Trust Task asks an agent to re-read its service configuration and restart the transports it serves on.

It exists because configuration changes and process lifetime are separate concerns. An operator who has adjusted an agent's endpoints, mediators or listeners needs those changes to take effect, and the alternatives are worse: restarting the process out of band requires access to the host rather than to the agent, which is a different and usually larger authority, and leaves no record in the agent's own trail that anything happened.

The payload is empty by design. There is no member selecting which services to reload, no member supplying configuration, and no member forcing anything. A reload applies whatever the agent's configuration currently says, in full — see [Why the payload is empty](#why-the-payload-is-empty).

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

The authority is **operational custody of the agent** — entitlement to run it, not entitlement to use it. A recipient **MUST** refuse a producer that holds only the ability to transact through this agent, however broadly: a party entitled to every task the agent offers is entitled to none of its lifecycle, because being able to ask an agent for things is not the same as being able to take it away from everyone else.

That framing is the whole of the authorization question here, and it is easy to get wrong in one specific way. The task reads as harmless — it creates nothing, deletes nothing, and discloses nothing — so it invites a lower bar than the destructive tasks around it. The bar should be the same. Availability is the property this task affects, and it is the one property every other party depending on the agent shares.

Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the VID, `issuer`, `recipient`, transport identity or `proof` establishes who asked and that the document is unaltered, never that they are entitled.

This specification does not, and **MUST NOT**, declare that a human approval or a step-up is required ([SPEC §7.3 item 13](/SPEC.md#73-specification-requirements)). A consumer imposing one is making a policy judgement this document has no standing to make for it.

## Why the payload is empty

A reload takes no parameters, and the absence is deliberate rather than unfinished.

Every member this task might plausibly carry is a way of asking the agent to run on something other than its configuration: a list of services to reload selectively, an inline override, a flag to skip validation. Each turns "apply what is written down" into "apply what this document says", which is a different and much wider authority — one that would let a producer put an agent into a state its configuration does not describe, with no record of it anywhere but this request.

Keeping the payload empty means the agent's configuration remains the single account of how the agent runs, and this task remains a way to make that account current. An operator who wants different behaviour changes the configuration, which is a separate, recorded act, and then reloads.

The one consequence worth naming: a reload is **all or nothing**. There is no way to restart one transport and leave another alone, and a recipient **MUST NOT** add one through `ext`.

## Failure leaves an indeterminate agent

A reload has an outcome this specification cannot describe, and pretending otherwise would be the more dangerous choice.

If the recipient reads a configuration it cannot bring services up on, what happens next is a property of the implementation: it may hold the previous configuration and stay up, come up partially, or fail to come up at all. `reloadFailed` reports that the reload did not succeed and **MUST NOT** be read as an assertion that the agent is still serving on its previous configuration.

There is a second case with no response at all. A successful reload drops the very transport the response would travel on, so a producer may see the connection close instead of a `#response` document. That is not a failure and **MUST NOT** be retried as one — the reload has very likely succeeded. A producer **SHOULD** treat a dropped connection as an unknown outcome, wait, and re-establish; recipients **SHOULD** send the response before restarting where the transport allows it, so that the ambiguity is rare rather than routine.

In both cases the reliable check is the same, and it is external to this task: reconnect and observe whether the agent answers.

## Definitions

The request payload defines no members. The response defines one:

**`status`** — the recipient's account of what it is doing, as free text. Advisory: a producer **MUST NOT** parse it or branch on its value. It exists so that a response is legible to an operator reading a transcript, not so that a client can drive logic from it — the reliable signal is whether the agent answers afterwards.

## Request

The producer is the agent's operator; the recipient is the agent. The request payload is the top-level schema in [`payload.schema.json`](payload.schema.json), which permits only `ext`.

### The whole request

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-00000000000b",
  "type": "https://trusttasks.org/spec/vta/management/reload-services/1.0#request",
  "issuer": "did:example:operator",
  "recipient": "did:example:agent",
  "issuedAt": "2026-01-01T02:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fd",
  "payload": {}
}
```

## Response

The producer of the response is the recipient of the request. Its payload is the sub-schema reachable via `$anchor: "response"`. Failures use `trust-task-error` with the code declared in the front matter, not a `#response` document.

A recipient **SHOULD** send this before restarting, where the transport allows it. See [Failure leaves an indeterminate agent](#failure-leaves-an-indeterminate-agent).

### Acknowledged, restarting

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-00000000000c",
  "type": "https://trusttasks.org/spec/vta/management/reload-services/1.0#response",
  "issuer": "did:example:agent",
  "recipient": "did:example:operator",
  "issuedAt": "2026-01-01T02:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000fd",
  "payload": {
    "status": "restarting"
  }
}
```

## Security & Privacy

### Data carried

Nothing in, and a status string out. This is the smallest payload in the family and it should stay that way.

A recipient **MUST NOT** return its configuration, its endpoints, its mediators, or a list of the services it restarted. A reload is an instruction, not a read, and answering it with a description of the agent's deployment would make this task a way to enumerate an agent's infrastructure — available, by construction, to a producer who has been given no read entitlement at all.

`status` is free text shown to a human. A recipient **MUST NOT** place a hostname, credential, file path or error detail in it that it would not publish; the value is routinely copied into tickets and transcripts.

The absence of data does not make the task low-risk. What this request costs is measured in availability rather than disclosure, and that cost falls on every counterparty of the agent rather than on the producer.

### Correlation

The recipient learns that this operator reloaded it and when. Over time that is a record of when the agent's configuration changed, which is a reasonable thing for the agent to know about its own operator and is intrinsic — an agent cannot restart without knowing it did.

The more interesting correlation is visible to **everyone else**. A reload drops every open session simultaneously, so all of the agent's counterparties observe a synchronous disconnect at the same instant. That is a broadcast signal: it tells unrelated parties that something happened to this agent, lets them infer that its operator was active, and lets any two of them establish they depend on the same agent by comparing when their connections dropped.

Nothing in this protocol prevents that — it is what restarting shared infrastructure looks like from outside. Operators **SHOULD** be aware that a reload is publicly observable to the agent's counterparties, and **SHOULD NOT** treat frequent reloads as private operational activity.

`threadId` joins request to response, where a response arrives at all.

### Retention

Nothing of the request is retained; it carries nothing to keep.

The recipient **SHOULD** record that a reload was requested, by whom and when, and **SHOULD** record the outcome separately from the request — because the two can differ, and a trail showing only "a reload was asked for" cannot answer whether the agent came back. This is the one durable trace of an operation whose effects are otherwise entirely in the past tense.

A recipient **MUST NOT** write that record into state that the reload itself re-reads or replaces.

### Consent/purpose

The purpose is applying configuration the operator has already changed. Two uses are outside it.

A producer **MUST NOT** use a reload to interrupt an agent — to clear another party's session, to cut off an in-progress exchange, or to make the agent unavailable during some window. The task's effects are indistinguishable from a denial of service and its authorization rests entirely on being asked in good faith, which is why the recipient's record of who asked is the control that matters.

A producer **SHOULD NOT** reload speculatively, as a way of resolving unexplained behaviour, without expecting the cost: every counterparty pays for it, and none of them asked.

Per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements), this specification does not declare a consent, approval or step-up requirement; the paragraphs above state purpose limitations, which is a different thing.
