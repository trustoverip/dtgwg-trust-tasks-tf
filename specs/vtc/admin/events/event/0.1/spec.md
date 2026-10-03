---
slug: vtc/admin/events/event
version: "0.1"
title: "VTC Admin Events — Event"
summary: "A hint a Verifiable Trust Community sends down an administrator console's live channel — this topic changed, re-read it, and for badge topics the count is now n — that never carries a record, so it can neither leak data nor stand in for a read."
status: draft
targetFrameworkVersion: "0.6.0"
category: access-control
keywords: [vtc, admin, console, events, live, hint, server-sent-events, sse, push, badge, notifications]
parties:
  - role: Community maintainer (VTC)
    requirement: REQUIRED
    member: issuer
  - role: Administrator (console)
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    The stream is carried over transports such as the HTTPS binding that give no producer-to-consumer end-to-end
    integrity, so the framework default (SPEC §4.7.2) is that a proof SHOULD be included, and this specification may not
    declare less (SPEC §7.3 item 8). It does not require more because a hint carries no authority and no data: the only
    thing a console may do with one is re-fetch a topic through a read that is signed and authorized on its own. A
    forged, altered or replayed hint can therefore cause at most one extra read, or a badge that the read corrects
    moments later; a suppressed one is no worse than the polling fallback. A console MUST NOT persist a hint or act on
    one beyond re-fetching, which is why it accepts an unsigned hint on a stream it opened rather than refusing it.
issuedAtRequirement:
  requirement: RECOMMENDED
  rationale: >-
    Not consequential, so not required. `at` already places the change, and `resumeToken` orders the stream; issuedAt
    remains useful for diagnosing a stream that delivers late.
sideEffects:
  level: none
  rationale: >-
    The console re-fetches a topic and redraws. Receiving a hint changes nothing at the community and nothing in the
    console that the following read does not overwrite.
exposure:
  discloses: none
  actsAsSubject: false
  ingests: metadata
  rationale: >-
    The hint carries into the console that a topic changed, when, and for the badge topics the caller's count — activity
    metadata the caller is entitled to through the corresponding read. No record and no personal data.
retention:
  class: transient
  rationale: >-
    A hint is a trigger to re-read. Once the read is made it has no further use, except its resumeToken, which the
    console keeps only until the next one replaces it.
errorCodes: []
related:
  - vtc/admin/events/subscribe
  - vtc/admin/actions/list
  - vtc/join-requests/list
  - vtc/members/list
  - vtc/config/export
---

## Abstract

**VTC Admin Events — Event** is the push half of the live console channel opened by [`vtc/admin/events/subscribe`](../../subscribe/0.1/spec.md). Each event is a **hint**. It names one topic that changed, when the change happened, and, for the badge topics, the caller's new count. It never carries the record that changed. The console reacts by re-fetching that topic through its own signed read, so data reaches it only through reads that authorize themselves.

There is no response. A console that disagrees with a hint ignores it, and the next read settles the matter.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **producer** (the community) **MUST**:

1. Emit documents whose `type` is `https://trusttasks.org/spec/vtc/admin/events/event/0.1`, with itself as `issuer` and the subscribing administrator as `recipient`. It **SHOULD** include a `proof`. It **SHOULD** set `parentThreadId` to the `threadId` of the subscribe that opened the stream, so that a log can find the subscription a hint belongs to. That value is for navigation only ([SPEC §4.9.2](/SPEC.md#492-the-parentthreadid-member)).
2. Send them only on the stream the subscribe opened, and only for the effective topics its response named, under the per-hint authorization of [`vtc/admin/events/subscribe`](../../subscribe/0.1/spec.md#authorization). It **MUST NOT** store a hint for later delivery: a hint the console was not connected for is lost, and resumption or the console's re-read covers it.
3. **Never carry record data.** A hint **MUST NOT** carry a record, any member of one, an identifier of one — an `actionId`, a join-request id, a member's DID — or any other personal data, in any member including `ext`. It carries `topic`, `at`, `resumeToken` and, on a count topic, `count`, and nothing else.
4. Include `count` on a count topic (`actions`, `acknowledgements`, `joinRequests`), computed for the recipient exactly as that topic's read would compute it at `at`. It **MUST NOT** include `count` on any other topic.
5. Make each `resumeToken` a position after the previous one on the same stream, bound to the recipient. Over HTTPS it **SHOULD** also carry the token as the SSE `id` of the event ([binding 0.3 §2.1](../../../../../../bindings/https/0.3/spec.md)).

The topic counts are:

| Topic | `count` is |
|---|---|
| `actions` | the recipient's `waitingForMe` count, as [`vtc/admin/actions/list`](../../../actions/list/0.2/spec.md) returns it in `counts` |
| `acknowledgements` | the open acknowledge-category actions on which the recipient is an expected acknowledger who has not yet acknowledged |
| `joinRequests` | the community's join requests awaiting a decision that [`vtc/join-requests/list`](../../../../join-requests/list/0.1/spec.md) would show the recipient |

A conforming **consumer** (the console) **MUST**:

1. Discard a hint that does not arrive on a stream it opened, or whose `topic` is not among that stream's effective topics.
2. React to a hint only by re-fetching that topic's read, and **MUST NOT** treat a hint as a record, as authority, or as evidence that any particular record exists or changed. It **MAY** show `count` on a badge until the read returns, and **MUST** replace it with the read's own answer when it does.
3. Ignore `count` on a topic that is not a count topic.
4. Keep the latest `resumeToken` and present it as `since` when it re-subscribes.
5. Not respond. There is no `#response` form for this task, and the stream has no return path.
6. Treat hints as transient, and **MUST NOT** persist them beyond the session.

## Authorization

The community sends a hint only because the recipient holds an open stream, opened by a signed [`vtc/admin/events/subscribe`](../../subscribe/0.1/spec.md), and only for a topic whose read the recipient is entitled to at the moment the hint is sent. That subscription, and that per-hint check, are the whole entitlement. They are stated in the subscribe specification's [Authorization](../../subscribe/0.1/spec.md#authorization) section. No hint is sent for a topic the recipient cannot read.

The hint itself asserts nothing the console must act on and authorizes nothing: the read it prompts is decided afresh, on that read's own proof and the community's records at that moment. That is why the `proof` is RECOMMENDED rather than REQUIRED: the transport gives no end-to-end integrity, so the community SHOULD sign, but a console loses nothing by accepting an unsigned hint on a stream it opened to the community it addressed. The [HTTPS binding 0.3](../../../../../../bindings/https/0.3/spec.md) says what that stream does and does not guarantee. The consequence of a forged hint is bounded by consumer rule 2 to one unnecessary read.

## Definitions

- **`topic`** (REQUIRED) — the shared [`Topic`](../../_shared/0.1/topic.schema.json) that changed.
- **`at`** (REQUIRED) — when the community observed the change. Where several changes were coalesced into this hint, the latest.
- **`count`** (OPTIONAL) — count topics only: the recipient's badge count as of `at`.
- **`resumeToken`** (REQUIRED) — the stream position after this hint, opaque to the console.

## Request

The community sends the hint to the console over the stream the subscribe opened. Over HTTPS each hint is one SSE event whose single `data:` line is the whole document. The payload is the top-level schema in [`payload.schema.json`](payload.schema.json). An event has no response.

### Two actions now wait for Bob

```json
{
  "id": "urn:uuid:8b5f6424-a563-456b-ab29-161af01004e2",
  "type": "https://trusttasks.org/spec/vtc/admin/events/event/0.1",
  "parentThreadId": "urn:uuid:195bd22b-70cc-4f96-88d4-73040fff978b",
  "issuer": "did:web:community.example",
  "recipient": "did:web:bob.example",
  "issuedAt": "2026-10-03T09:15:01Z",
  "payload": {
    "topic": "actions",
    "at": "2026-10-03T09:12:40Z",
    "count": 2,
    "resumeToken": "evt.000142"
  }
}
```

### The member list changed

No count: the console re-reads `vtc/members/list` to learn what changed.

```json
{
  "id": "urn:uuid:609b97b0-b2ce-4961-ade9-b10b7d70d530",
  "type": "https://trusttasks.org/spec/vtc/admin/events/event/0.1",
  "parentThreadId": "urn:uuid:195bd22b-70cc-4f96-88d4-73040fff978b",
  "issuer": "did:web:community.example",
  "recipient": "did:web:bob.example",
  "issuedAt": "2026-10-03T09:31:07Z",
  "payload": {
    "topic": "members",
    "at": "2026-10-03T09:31:06Z",
    "resumeToken": "evt.000143"
  }
}
```

### A signed hint

The recommended form. A console treats it exactly as it would an unsigned hint: the proof makes the hint attributable, and grants it nothing.

```json
{
  "id": "urn:uuid:206b61fb-c830-413c-8d09-cdcc540e3b25",
  "type": "https://trusttasks.org/spec/vtc/admin/events/event/0.1",
  "parentThreadId": "urn:uuid:195bd22b-70cc-4f96-88d4-73040fff978b",
  "issuer": "did:web:community.example",
  "recipient": "did:web:bob.example",
  "issuedAt": "2026-10-03T09:40:12Z",
  "payload": {
    "topic": "joinRequests",
    "at": "2026-10-03T09:40:11Z",
    "count": 1,
    "resumeToken": "evt.000144"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-03T09:40:12Z",
    "verificationMethod": "did:web:community.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "zf54eS56FtdNCW1cjvHgHKCy3V9xaLSuiuGFTtgKbbFVJwtRGvS2kPwsPMjRSBuUg5TJPtmUGXRTs3GYDYTGv4CJ"
  }
}
```

## Security & Privacy

### Data carried

A topic name, a timestamp, an opaque token and, for three topics, one integer. A producer **MUST NOT** carry record data in a hint: no record, no member of a record, no record identifier, no DID and no other personal data, in any member including `ext`. The rule is what lets a console accept an unsigned hint, and the stream need no authorization model of its own. Everything with a subject reaches the console through a read that authorizes it.

### Correlation

Hints disclose activity **volume and timing** to an administrator: how many actions and acknowledgements await them, how many join requests await a decision, and when the community's administration, membership and configuration change. That is justified because the recipient is entitled to the reads that reveal the same counts, and the records behind them besides. A hint reveals that information sooner, not additionally. The `actions` count does reflect colleagues' activity: it rises when another administrator requests something and falls when another decides it. A community that wants that rhythm less legible can coalesce hints over longer intervals. Someone observing the connection without its content sees hint timing and size, which tracks administrative activity, and fixed-interval heartbeats reveal only that a console is open. `resumeToken` is per recipient and **SHOULD** be unlinkable across recipients.

### Retention

None. A hint is a trigger to re-read, discarded once the read is made, apart from its `resumeToken`, which the next hint replaces. The community keeps no per-hint record, beyond any short history it holds to honour `since`.

### Consent/purpose

The purpose is to keep an open administrator console current. Using hints to measure how quickly administrators respond, or to watch colleagues' activity, is outside it. Whether a community offers the channel at all is its policy, not this specification's.
