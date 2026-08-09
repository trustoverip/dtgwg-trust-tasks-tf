---
slug: trust-task-next-step
version: "0.1"
title: Trust Task Next Step
summary: The framework-defined response a consumer returns when a task was understood but cannot complete in isolation, naming the Trust Task it expects in order to proceed.
status: draft
targetFrameworkVersion: "0.3"
category: framework
keywords:
  - continuation
  - response
  - next-step
  - framework
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Continuing consumer
    requirement: REQUIRED
    member: issuer
  - role: Original producer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    A next step is a redirection primitive — it steers the producer toward a task
    of the recipient's choosing — which makes an unattributable one a
    social-engineering surface in a way an error response is not: a refusal that
    cannot be trusted is merely ignored, while a redirection that cannot be
    trusted is followed. RECOMMENDED rather than REQUIRED because the document is
    consumed immediately, in-exchange, and is not retained or relied upon by third
    parties, so the SPEC §4.7.1 condition that makes a proof mandatory does not
    arise. Where the transport does not authenticate the sender end-to-end, a
    proof is what makes the redirection attributable, and the Conformance rules
    below forbid acting on one that is neither.
sideEffects:
  level: none
  rationale: "Reports that the originating task is blocked and names what would unblock it; changes no recipient state. The recipient may hold pending state on the producer's behalf, but this document does not create it."
exposure:
  discloses: metadata
  actsAsSubject: false
  rationale: "Names a Type URI the recipient is prepared to act upon, which is a capability hint about the recipient's configuration — the same disclosure a discovery response makes, and subject to the same restraint (SPEC §11.5)."
errorCodes: []
related:
  - trust-task-error
  - trust-task-discovery
---

## Abstract

The **Trust Task Next Step** is the framework-defined response a *consumer* returns when it understood a *Trust Task document* and is willing to act on it, but cannot complete it in isolation. It names the *Trust Task* the consumer expects in order to proceed.

This specification is the registry publication of the type reserved at [SPEC.md §8.6](../../../SPEC.md#86-reserved-response-type-slugs), whose payload was left out of scope at framework 0.3. It is itself a *Trust Task document*, validated and signed by the same pipeline as any other, so an implementation needs no third response path beside success and failure.

A `trust-task-next-step` document is a *response*. It has no `#response` variant of its own: a producer answers it by issuing a document of the **expected** type, not by responding to this one.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](../../../SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## It is neither success nor failure

The framework already distinguishes two replies cleanly ([SPEC.md §8](../../../SPEC.md#8-error-responses)): a success response carries the originating *Type URI* with the `#response` fragment, and a failure carries `trust-task-error`. A next step is a **third** disposition and must not be conflated with either.

| Reply | Means | The originating task is |
|---|---|---|
| `<type>#response` | The task completed | closed, successfully |
| `trust-task-error` | The task will not be performed | closed, unsuccessfully |
| `trust-task-next-step` | The task was understood and is blocked | **open** |

A consumer that means "no" returns an *error response*; one that means "done" returns the originating specification's `#response`. Returning a next step leaves the exchange live, and the producer is entitled to act on that.

This is why a blocked task is not reported as `taskFailed` with a helpful `message`. An error terminates the document under [SPEC.md §8.4](../../../SPEC.md#84-retry-semantics); a next step does not, and only one of the two tells a producer that continuing is expected of it.

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the consumer of the original task, now reporting that it is blocked) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/trust-task-next-step/0.1`, with itself as `issuer` and the original task's producer as `recipient`.
2. Set `threadId` to the originating document's `threadId` if one was carried, or to the originating document's `id` otherwise, per [SPEC.md §4.9](../../../SPEC.md#49-the-threadid-member). This document's own `id` **MUST NOT** reuse the originating document's.
3. Populate `payload.expects` with at least one entry, each naming a *Type URI* it will accept next.
4. **SHOULD** populate `payload.inResponseTo`, for the reason [SPEC.md §8.2](../../../SPEC.md#82-error-payload) gives for the identical member on an error response.
5. **MUST NOT** emit a next step for a task it has already answered with a `#response` or a `trust-task-error` on the same thread.
6. **SHOULD** carry `parentThreadId` and, where the exchange is a ceremony step, the `ceremony` member, unchanged from the document it answers — a next step belongs to the same exchange it interrupts.

A conforming **consumer** (the original producer, receiving the suggestion) **MUST**:

1. Apply the [SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements) pipeline as for any other document.
2. **Treat `expects` as a suggestion, not an instruction.** The decision to perform any named task is the consumer's alone, taken under its own policy. See [Security & Privacy](#security--privacy).
3. **MUST NOT** act on a next step whose origin it can authenticate neither in-band (via `proof`) nor from the transport. An unauthenticated redirection is indistinguishable from an injected one.
4. Where it chooses to continue, satisfy **one** entry of `expects` — they are alternatives, never a conjunction — and carry the same `threadId` onto the resulting document.
5. Honour `continuation`: under `resubmit` (the default), re-issue the originating request after the expected task completes, as a **new document with a fresh `id`** and the same `threadId`. Per [SPEC.md §8.4](../../../SPEC.md#84-retry-semantics) that is not a *retry*, which is a bit-for-bit re-send; the distinction matters because the originating document may have expired in the interim.
6. **SHOULD** bound the number of consecutive next steps it will follow on one thread, and abandon the exchange beyond that bound.

A consumer that declines to continue simply stops. There is no "declined" reply: the recipient learns nothing it is entitled to, and the originating task expires on its own terms.

## Alternatives, not prerequisites

`expects` is a list of **alternatives**. Satisfying any single entry unblocks the exchange; satisfying all of them is never required.

A recipient that genuinely needs two things done first names the one it wants next and issues a further next step when that lands. Expressing a conjunction here would make this response a small flow definition — ordering, optionality, completion — which is exactly the material that belongs to a [Trust Ceremony](../../../docs/adr/0001-naming-the-multi-task-flow-layer.md) rather than to a single reply. Keeping `expects` disjunctive is what stops this specification growing into one by accident.

## Relationship to ceremonies

A next step is *coordination* with no definition and no evidence — the second of the three concerns a ceremony separates. The two compose without either depending on the other:

- Used **alone**, it drives an ad-hoc exchange in which the recipient decides at each turn what comes next. No definition is published and nothing attests the sequence beyond the documents themselves.
- Used **within a ceremony**, it is how a recipient signals which step it expects where the definition leaves that open. The `ceremony` member is carried unchanged (Conformance, producer rule 6), so the next step belongs to the enactment it interrupts and is chained like any other document in it.

Neither requires the other, and this specification is usable today at framework 0.3, where the `ceremony` member does not yet exist.

## Security & Privacy

**A next step is a redirection primitive, and redirection is a social-engineering surface.** A refusal that cannot be trusted is ignored; a redirection that cannot be trusted is *followed*. That asymmetry is why consumer rule 3 forbids acting on a next step whose origin cannot be authenticated either in-band or from the transport, and why `proof` is RECOMMENDED here on a stronger rationale than for an error response.

**It confers no authorization whatsoever.** That a recipient suggests a task does not make performing it approved, safe, or policy-compliant. Every gate the consumer would otherwise apply still applies in full — the side-effect and exposure classifications of [SPEC.md §7.3](../../../SPEC.md#73-specification-requirements) items 13 and 14, any consent requirement, any approval policy. A consumer that performs a `destructive` task because a counterparty asked it to has been talked into it, not authorized. This mirrors the advisory status of a discovery response ([SPEC.md §11.4](../../../SPEC.md#114-status-of-the-response)): a suggestion narrows what a party chooses to send, and binds nothing.

**Downgrade.** A recipient can suggest a weaker path than the one a producer intended — a lesser authentication, a broader disclosure, a task with a softer proof requirement. The producer applies its own policy to the suggestion and **SHOULD** reject any continuation weaker than the one it was already attempting.

**Loops.** Two parties can redirect each other indefinitely, and a chain of next steps consumes work at every hop. Consumer rule 6 requires a bound. This is local hardening in the manner of [SPEC.md §10.2](../../../SPEC.md#102-parser-hardening) rather than a wire-level construct: a counter on the wire would be trivially reset by either party.

**Disclosure.** `expects` names types the recipient is prepared to act upon, which fingerprints its configuration exactly as a discovery response does. A recipient that considers its supported task set sensitive **SHOULD** authenticate the producer before returning a next step, and **MAY** return an error instead. `message` reaches a party that may not be entitled to learn why a task is blocked, and **SHOULD** disclose nothing the producer could not already infer.

**`hint` is untrusted input.** It is composed by the recipient and consumed by the producer when building the next document. A producer **MUST NOT** treat a hint as authoritative for any value it can determine itself, and **MUST** validate it against the schema of the specification named in `typeUri` before use.
