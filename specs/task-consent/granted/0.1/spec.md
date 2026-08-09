---
slug: task-consent/granted
version: "0.1"
title: Task Consent — Granted
summary: A fire-and-forget notice from the executor to the requester that its pending task has reached the approval threshold and a single-use grant is waiting, so the requester re-submits immediately instead of polling.
status: draft
targetFrameworkVersion: "0.2"
category: consent
keywords:
  - consent
  - delegated-execution
  - approval
  - grant
  - notification
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Executor (Verifiable Trust Agent)
    requirement: REQUIRED
    member: issuer
  - role: Requester
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: OPTIONAL
  rationale: The notice is deliberately non-load-bearing — the executor's single-use grant lookup at re-submit is the real gate, so nothing in this document is the basis of any decision. Sender attribution comes from the authenticated transport (e.g. the DIDComm authcrypt envelope); a spoofed or replayed notice at worst provokes one futile re-submit, which the grant check refuses. Executors MAY sign it; consumers MUST NOT treat it as authorization either way.
sideEffects:
  level: none
  rationale: Receipt persists nothing at the requester; it only prompts an immediate re-submit the requester would otherwise perform on its next poll cycle.
exposure:
  discloses: none
  actsAsSubject: false
  rationale: An acknowledgement-style notice. It repeats only the salted payload digest and task type the requester already holds from its own submission.
errorCodes: []
related:
  - task-consent/request
  - task-consent/decision
  - push/wake
---

## Abstract

The **Task Consent — Granted** Trust Task closes the timing gap in the
delegated-execution consent flow. A
[`task-consent/request/0.1`](../../request/0.1/spec.md) puts a human in the
loop, which makes the approval window minutes wide; the requester holds a
rejected task and, without this notice, discovers the eventual grant only by
polling. When the final [`task-consent/decision/0.1`](../../decision/0.1/spec.md)
crosses the approval threshold and the executor mints its single-use grant, it
sends this notice to the requester so the re-submit happens the moment the
approval lands.

It is a **doorbell, not an authorization**. The design mirrors
[`push/wake`](../../../push/wake/0.2/spec.md): the useful content is the fact of
delivery, and every field is advisory. The grant the executor stored — looked up
and consumed at the re-submit — is the only thing that authorizes execution, and
it is re-checked against current policy, current enrolment, and the pinned state
at that moment. A lost notice costs the requester one poll cycle; a forged one
costs a futile re-submit that the grant check refuses. Neither is a security
event, and that is what keeps this task simple: nothing here needs to be
defended, because nothing here is relied upon.

## One direction, one outcome

Only a **grant** is announced. A denial deletes the pending request and sends
nothing: the requester's re-submit (or poll) discovers it. This is deliberate. A
notice that could carry `denied` would invite consumers to treat this advisory
channel as the authoritative outcome of the consent flow — and the moment a
consumer branches on an unauthenticated field, the field stops being advisory.
Keeping the vocabulary to the one value whose worst-case misuse is a refused
re-submit keeps the channel harmless by construction.

There is no `#response` form. The notice is fire-and-forget; the re-submit it
provokes is the observable effect, and a delivery acknowledgement would add a
round-trip to a path whose failure mode is already benign.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the executor) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/task-consent/granted/0.1`, with itself as `issuer` and the requester as `recipient`.
2. Send it only after the grant is durably stored — the notice tells the requester a re-submit will now succeed, and a notice that races its own grant makes the requester's immediate re-submit fail spuriously.
3. Populate `payloadDigest` with the same **salted wire digest** the matching `task-consent/request` carried. The internal digest the executor indexes by **MUST NOT** leave the executor.
4. Treat delivery as best-effort. A producer **MUST NOT** make grant validity, expiry, or any other consent-flow state depend on whether this notice was delivered.
5. Send **at most one** notice per grant. Redelivery by the transport is tolerable (see consumer rule 3); deliberate repetition is noise.

A conforming **consumer** (the requester) **MUST**:

1. Treat the notice as **advisory only**: correlate `payloadDigest` against its own pending task and re-submit that task through the normal submission path. It **MUST NOT** treat receipt as authorization, as proof the task executed, or as anything other than a hint to stop waiting.
2. Accept the notice only over a transport that authenticates the sender as the executor holding its pending task (e.g. the DIDComm authcrypt envelope), or carrying a verifiable `proof` from that executor. A notice from anyone else **MUST** be ignored — silently, since there is no response form.
3. Handle redelivery and spurious notices idempotently: a notice for an unknown or already-resolved `payloadDigest` is dropped, not an error.
4. Keep polling as the fallback. The notice is an optimization; a consumer whose correctness depends on receiving it is nonconforming.

## Payload

`payload.status` (REQUIRED) — always `granted`.
`payload.payloadDigest` (REQUIRED) — the salted wire digest of the approved task, for correlation.
`payload.taskType` (REQUIRED) — Type URI of the approved task, for correlation and display.
`payload.ext` — extension slot per [SPEC.md §4.5.1](../../../../SPEC.md#451-the-ext-extension-member).

## Examples

### The threshold is met and the requester is nudged to re-submit

```json
{
  "id": "urn:uuid:5e8b2a19-7c43-4f06-9d12-3b0a6e4c8f75",
  "type": "https://trusttasks.org/spec/task-consent/granted/0.1",
  "threadId": "zQmb1XVvHqbCe5nUPFxpJcRz3RtP4pQyKgTsWJgNBzVhE7d",
  "issuer": "did:key:z6MkExecutorVtaExample",
  "recipient": "did:key:z6MkRequesterBrowserExample",
  "issuedAt": "2026-07-13T09:43:20Z",
  "payload": {
    "status": "granted",
    "payloadDigest": "zQmb1XVvHqbCe5nUPFxpJcRz3RtP4pQyKgTsWJgNBzVhE7d",
    "taskType": "https://trusttasks.org/spec/webvh/dids/update/1.0"
  }
}
```

The requester now re-submits the original task unchanged. The executor consumes
the single-use grant, re-derives the digest from the payload it is about to
execute, re-asserts the state pin, and re-evaluates policy and approver
enrolment — none of which this notice participates in.

## Security & Privacy

**Nothing here is load-bearing.** The single-use grant at the executor is the
authorization; this document is a latency optimization. That inversion is the
security argument: because no consumer decision of consequence may branch on
this document, its threat model collapses to nuisance.

**Spoof and replay are harmless.** A forged or replayed notice provokes at most
one re-submit, which the grant lookup refuses (no grant, or already consumed).
Consumers still gate acceptance on transport-level sender authentication
(consumer rule 2) so the nuisance cannot be induced by arbitrary third parties.

**Disclosure is bounded to what the recipient already knows.** The notice
carries the salted wire digest and the task type — both already held by the
requester it is addressed to. The salted digest discloses nothing to an
observer who lacks the challenge (see the *Binding* section of
[`task-consent/request/0.1`](../../request/0.1/spec.md)); transport
confidentiality is expected regardless.

**A denial is never announced** — see *One direction, one outcome*. The absence
of a notice reveals nothing, since absence is also the normal state of a
pending approval.
