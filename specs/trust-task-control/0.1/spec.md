---
slug: trust-task-control
version: "0.1"
title: Trust Task Control
summary: The framework-defined request by which a producer cancels, suspends, or resumes work a consumer has already accepted. Cancellation prevents future effects; it never undoes past ones, and the response reports what already occurred.
status: draft
targetFrameworkVersion: "0.4"
category: framework
keywords:
  - control
  - cancel
  - suspend
  - resume
  - corrigibility
  - framework
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Controlling producer
    requirement: REQUIRED
    member: issuer
  - role: Executing consumer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    A control document stops another party's work, which makes an unattributable
    one a denial-of-service instrument rather than merely a message to ignore: an
    error response that cannot be trusted is discarded, while a cancellation that
    cannot be trusted has already achieved its effect by the time anyone doubts
    it. The document is also the evidence that a withdrawal was requested, and is
    retained by both parties to explain why work stopped — the SPEC §4.7.1
    retained-and-relied-upon condition. REQUIRED on both variants: the response
    carries the record of what effects occurred before the operation took hold,
    which a producer relies on to decide whether to compensate, and which a third
    party may later need to evaluate.
sideEffects:
  level: mutating
  rationale: "Changes the execution state the consumer holds for another task — stopping it, pausing it, or restarting it. Recoverable in the sense that no external effect is produced by this document itself; the task it controls may be another matter, which is what the response reports."
exposure:
  discloses: metadata
  actsAsSubject: false
  rationale: "The response describes effects the controlled task produced — identifiers, references, and prose about what was created or changed. That is descriptive data about the exchange the producer initiated, not secret material, and it is disclosed only to the party that initiated the work."
errorCodes:
  - code: trust-task-control:notAuthorized
    meaning: The issuer is not the target document's initiator, and the consumer's policy does not recognize it as authorized to control the task (SPEC.md §12.1). Distinct from `permissionDenied`, which concerns authority to invoke this specification at all.
    retryable: false
  - code: trust-task-control:notControllable
    meaning: The consumer holds the target task but will not apply the requested operation to it — typically because the task has passed a point its specification declares unsafe to interrupt, or because the consumer does not implement `suspend`/`resume`.
    retryable: false
  - code: trust-task-control:alreadyCancelled
    meaning: The target task was already cancelled. Cancellation is terminal (SPEC.md §12.3), so it can be neither repeated nor undone; a producer that still wants the work issues a new document.
    retryable: false
related:
  - trust-task-error
  - trust-task-next-step
---

## Abstract

**Trust Task Control** is how a *producer* stops work it has already asked for.
It is the registry publication of the mechanism defined at
[SPEC.md §12](../../../SPEC.md#12-task-control), and it carries three
operations: `cancel`, `suspend`, and `resume`.

It is a **request**, not a response. A *consumer* that stops work on its own
initiative does not send one of these — it returns a `trust-task-error` carrying
`cancelled`. That asymmetry is deliberate: it keeps a withdrawal ("you asked me
to stop") distinguishable from a refusal ("I stopped"), which imply opposite
things about whether the *producer* should try again, and which no party could
otherwise tell apart from a retained document.

**Cancellation prevents future effects. It does not undo past ones.** The
framework declines to require rollback
([SPEC.md §12.4](../../../SPEC.md#124-control-does-not-roll-back)), because many
effects are irreversible by construction and because the state needed to reverse
one is frequently the material the task existed to destroy. What this
specification provides instead is **information**: the response reports what
occurred before the operation took hold, so the *producer* can decide whether to
invoke a compensating task of its own.

## Status of this Document

This is a **draft** *Trust Task specification* per
[SPEC.md §5.3](../../../SPEC.md#53-maturity-levels); the schema **MAY** change
without notice. The design and its rationale are recorded in
[`docs/design-notes/task-control-and-corrigibility.md`](../../../docs/design-notes/task-control-and-corrigibility.md).
Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and
[[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the controlling party) **MUST**:

1. Emit a *Trust Task document* whose `type` is
   `https://trusttasks.org/spec/trust-task-control/0.1`, carrying a `proof`
   ([SPEC.md §12.1](../../../SPEC.md#121-authorization)) and an in-band
   `recipient` per the audience-binding rule of
   [SPEC.md §4.8.2](../../../SPEC.md#482-audience-binding).
2. Identify the target by its `id` in `payload.target.id`, and **SHOULD**
   populate `payload.target.typeUri`.
3. Set `threadId` to the target document's `threadId` where it carried one, or
   to the target's `id` otherwise, so the control operation correlates with the
   exchange it acts upon.
4. **MUST NOT** request that a suspension resume automatically after an interval
   of its own choosing; there is no member for it, and
   [SPEC.md §12.5](../../../SPEC.md#125-suspension-and-resumption) forbids it.

A conforming **consumer** (the executing party) **MUST**:

1. Apply the [SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements) pipeline.
2. Establish authorization per
   [SPEC.md §12.1](../../../SPEC.md#121-authorization): the target document's
   `issuer` is authorized by default, and any other party only under the
   *consumer*'s own policy. Reject an unauthorized control document with
   `trust-task-control:notAuthorized`.
3. Treat a valid, authorized operation as one of the conditions
   [SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements) item 12 re-evaluates
   before each irreversible or externally visible effect. **This is how the
   operation takes effect**; there is no separate mechanism, and a *consumer*
   that implements item 12 correctly has already implemented the race.
4. Reject an `operation` value it does not recognize rather than applying a
   default. Silently downgrading an unrecognized operation to a known one
   replaces the *producer*'s intent with the *consumer*'s guess.
5. Return the `#response` naming the `outcome`, and populate `effects` where the
   outcome is `appliedWithEffects` or `alreadyCompleted`. A *consumer*
   **MUST NOT** report `applied` where any irreversible or externally visible
   effect had already occurred.
6. Retain the [SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements) item 11
   record for a cancelled task for the remainder of its acceptance window, so a
   re-delivery of the original document is absorbed rather than executed.

A conforming *consumer* **SHOULD** record a control document naming a target it
has not yet received, and refuse the later-arriving document rather than
executing it. Out-of-order arrival is ordinary on asynchronous transports, and
the item 11 record already provides the storage and the expiry bound.

## Authorization

*Declared under [SPEC.md §7.3](../../../SPEC.md#73-specification-requirements)
item 15.*

The authorization evidence is **being the initiator of the target task** — the
*party* identified by the target document's `issuer`, or the identity
authenticated for it where no in-band `issuer` was carried. A *consumer*
**MUST NOT** require further evidence from that party.

That is a **floor, not a ceiling**. A *consumer* executing on behalf of a
mandate holder, a supervising principal, or an organization whose agent
initiated the work **MAY** recognize that party's authority to stop it, under
its own policy and applicable governance framework
([SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements) item 10). A
*consumer* that recognizes only the initiator is equally conforming. This
specification describes the evidence the task assumes; it does not oblige any
*consumer* to authorize any particular party, and it declares nothing about
consent or human approval.

The `proof` is **not** the authorization. It establishes that the control
document was composed by the party it names, which is what makes the comparison
against the target's `issuer` possible — per
[SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements) item 10, that
establishes *who asked*, never that they may. Membership of a *Trust Ceremony*
confers nothing here, as
[SPEC.md §4.11.4](../../../SPEC.md#4114-membership-is-a-claim-not-a-permission)
provides generally.

## Payload

`operation` (REQUIRED) — `cancel` | `suspend` | `resume`.

`target` (REQUIRED) — `{ id, typeUri? }`. `id` is the sole identifying member;
`threadId`, `parentThreadId` and ceremony membership **MUST NOT** identify the
target on their own
([SPEC.md §12.2](../../../SPEC.md#122-identifying-the-target)).

`reason` (optional) — human-readable explanation, for operator UI and audit. A
*consumer* **MUST NOT** condition its handling on this value.

`ext` (optional) — the framework extension slot
([SPEC.md §4.5.1](../../../SPEC.md#451-the-ext-extension-member)).

## Response

`operation` and `target` are echoed, so a retained response is self-describing.

`outcome` (REQUIRED) is the load-bearing member:

| Value | Meaning |
|---|---|
| `applied` | The operation took effect and **no** irreversible or externally visible effect had occurred. The only outcome meaning the task left no trace. |
| `appliedWithEffects` | The operation took effect, but effects had already occurred. `effects` is REQUIRED. |
| `alreadyCompleted` | The task finished before the control document was processed. Not a cancellation. |
| `unknownTask` | No record of the target `id` — never received, or its acceptance window has lapsed. |

`effects` describes what was created, changed, disclosed, or exercised before
the operation took hold: a `description`, an optional `ref` so a compensating
task can name the thing, and an optional `reversible` hint. Absent `reversible`
means unknown, which a *producer* **SHOULD** treat as no weaker than `false`.

## Examples

A producer cancels a long-running issuance:

```json
{
  "id": "urn:uuid:9f1c8b2a-c001-4a10-8a00-000000000001",
  "type": "https://trusttasks.org/spec/trust-task-control/0.1",
  "threadId": "urn:uuid:6f1c8b2a-0001-4a10-8a00-000000000001",
  "issuer": "did:web:agent.example",
  "recipient": "did:web:issuer.example",
  "issuedAt": "2026-08-16T09:00:00Z",
  "payload": {
    "operation": "cancel",
    "target": {
      "id": "urn:uuid:6f1c8b2a-0001-4a10-8a00-000000000001",
      "typeUri": "https://trusttasks.org/spec/credential-exchange/issue/0.1"
    },
    "reason": "Subject withdrew consent before issuance completed."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-16T09:00:00Z",
    "verificationMethod": "did:web:agent.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z…(signature over the JCS-canonical document)"
  }
}
```

The consumer stopped in time, and nothing had been issued:

```json
{
  "id": "urn:uuid:9f1c8b2a-c002-4a10-8a00-000000000002",
  "type": "https://trusttasks.org/spec/trust-task-control/0.1#response",
  "threadId": "urn:uuid:6f1c8b2a-0001-4a10-8a00-000000000001",
  "issuer": "did:web:issuer.example",
  "recipient": "did:web:agent.example",
  "issuedAt": "2026-08-16T09:00:02Z",
  "payload": {
    "operation": "cancel",
    "target": { "id": "urn:uuid:6f1c8b2a-0001-4a10-8a00-000000000001" },
    "outcome": "applied"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-16T09:00:02Z",
    "verificationMethod": "did:web:issuer.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z…(signature over the JCS-canonical document)"
  }
}
```

The same cancellation arriving a second too late — the credential exists, and
the producer now knows it must revoke rather than assume nothing happened:

```json
{
  "id": "urn:uuid:9f1c8b2a-c003-4a10-8a00-000000000003",
  "type": "https://trusttasks.org/spec/trust-task-control/0.1#response",
  "threadId": "urn:uuid:6f1c8b2a-0001-4a10-8a00-000000000001",
  "issuer": "did:web:issuer.example",
  "recipient": "did:web:agent.example",
  "issuedAt": "2026-08-16T09:00:02Z",
  "payload": {
    "operation": "cancel",
    "target": { "id": "urn:uuid:6f1c8b2a-0001-4a10-8a00-000000000001" },
    "outcome": "appliedWithEffects",
    "effects": [
      {
        "description": "Credential was issued and delivered to the holder before the cancellation was processed.",
        "ref": "urn:uuid:11111111-2222-3333-4444-555555555555",
        "reversible": true
      }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-16T09:00:02Z",
    "verificationMethod": "did:web:issuer.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z…(signature over the JCS-canonical document)"
  }
}
```

## Security & Privacy

**An unattributable control document is a denial-of-service instrument.** This
is why `proof` is REQUIRED on both variants rather than RECOMMENDED. A refusal
that cannot be trusted is discarded; a cancellation that cannot be trusted has
already achieved its effect by the time anyone doubts it. A *consumer*
**MUST NOT** act on a control document whose proof does not verify, and
**MUST NOT** fall back to transport-derived identity where the framework
requires the proof.

**Cancellation is terminal, and that is a hazard as well as a simplification.**
A *producer* that cancels and then discovers it still wants the work must issue
a new document with a fresh `id`. A *consumer* **MUST NOT** offer a way to
revive a cancelled task, because a document with two contradictory lifecycle
states cannot be reasoned about by any party that retained it.

**The response discloses what the task did.** `effects` is descriptive data
about the *producer*'s own exchange, disclosed only to the party that initiated
it, which is why `exposure.discloses` is `metadata` rather than `secret`. A
*consumer* **SHOULD** nonetheless keep `description` free of material the
*producer* did not already possess — the member exists to let a *producer*
compensate, not to convey the content of the effect.

**Silence means nothing.** A *producer* that receives no response cannot
conclude that the task was cancelled, nor that it was not. Task control is
best-effort ([SPEC.md §12.8](../../../SPEC.md#128-support-is-optional)), a
*consumer* may not implement it, and a notification may be lost. A *producer*
that reissues on the assumption that silence meant success can cause exactly the
second consequential effect
[SPEC.md §7.2](../../../SPEC.md#72-consumer-requirements) item 11 exists to
prevent.
