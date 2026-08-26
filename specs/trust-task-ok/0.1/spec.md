---
slug: trust-task-ok
version: "0.1"
title: Trust Task OK
summary: The framework-defined courtesy acknowledgement a consumer MAY return for a task that defines no success-response document of its own. Deliberately weak — a producer may never rely on receiving one, and its absence means nothing.
status: draft
targetFrameworkVersion: "0.4"
category: framework
keywords:
  - acknowledgement
  - response
  - success
  - framework
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: Acknowledging consumer
    requirement: REQUIRED
    member: issuer
  - role: Original producer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: RECOMMENDED
  rationale: >-
    The primary defence against a forged acknowledgement is not the proof but
    the non-reliance rule in Conformance: a producer that may not act on an
    acknowledgement cannot be induced to act by a false one. The proof is
    defence in depth, and it matters because these documents are routinely
    logged even though they may not be relied upon — an unattributable entry in
    an operator's record of what was acknowledged is worse than no entry.
    RECOMMENDED rather than REQUIRED because the document is consumed
    in-exchange and carries nothing a third party could rely on, so the SPEC
    §4.7.1 retained-and-relied-upon condition does not arise. A specification
    whose acknowledgement genuinely is evidence should not use this document at
    all — see the Abstract.
sideEffects:
  level: none
  rationale: "Reports that a task was received and performed. Changes no recipient state; the state change, if any, was made by the task being acknowledged."
exposure:
  discloses: metadata
  actsAsSubject: false
  rationale: "May carry opaque references the consumer chose to surface — a ticket number, a queue position, a processing handle. Descriptive data about the producer's own exchange, disclosed only to the party that initiated it."
errorCodes: []
related:
  - trust-task-error
  - trust-task-next-step
---

## Abstract

**Trust Task OK** is the courtesy acknowledgement reserved at
[SPEC.md §8.6](/SPEC.md#86-reserved-response-type-slugs) since framework
`0.1`. A *consumer* **MAY** return one to confirm that it received and performed
a *Trust Task* which defines no success-response document of its own.

It is deliberately weak, and the weakness is the design rather than a gap in it:

> A *producer* **MUST NOT** rely on receiving an acknowledgement, and the
> absence of one carries **no information**.

A *consumer* may not implement this specification; a *consumer* that does may
still not send one; and the document may be lost in transit. A *producer* that
treats silence as failure will reissue work that succeeded — and for a
*consequential Trust Task* that is precisely the second effect
[SPEC.md §7.2](/SPEC.md#72-consumer-requirements) item 11 exists to
prevent.

**If an acknowledgement matters, do not use this document.** A task whose
acknowledgement is evidence — something a third party will later rely on, audit,
or dispute — declares its own `#response`, or a dedicated receipt task with its
own proof requirement and its own place in whatever chain it belongs to.
[`chat/message/0.1`](../../chat/message/0.1/spec.md) made exactly that choice,
and made it correctly: its acknowledgement is deferred to a task-specific
receipt so that it is "a signed, independently-verifiable link in the chain
rather than a transport-level ack".

## Status of this Document

This is a **draft** *Trust Task specification* per
[SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change
without notice. Feedback via the
[issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and
[[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** of this document (the *consumer* of the original task,
now acknowledging it) **MUST**:

1. Emit a *Trust Task document* whose `type` is
   `https://trusttasks.org/spec/trust-task-ok/0.1`, with itself as `issuer` and
   the original task's *producer* as `recipient`.
2. Set `threadId` to the originating document's `threadId` if it carried one, or
   to the originating document's `id` otherwise, per
   [SPEC.md §4.9](/SPEC.md#49-the-threadid-member). This document's own
   `id` **MUST NOT** reuse the originating document's `id`.
3. Send it **only** where the originating *Trust Task specification* defines no
   success-response document. Where a specification defines one, that document
   is the success reply and an acknowledgement **MUST NOT** be sent in its
   place: two success dispositions for one task leave a *producer* unable to
   tell which is authoritative.
4. **MUST NOT** convey through `payload.refs` anything the originating task's
   contract depends on (see [Payload](#payload)).

A conforming **consumer** of this document (the original *producer*) **MUST**:

1. Treat it as reporting that the originating task was **received and
   performed**, closing the exchange — unlike a
   [`trust-task-next-step`](../../trust-task-next-step/0.1/spec.md), which
   leaves it open.
2. **MUST NOT** rely on receiving one, and **MUST NOT** infer anything from its
   absence. In particular it **MUST NOT** treat absence as failure and reissue
   the task on that basis.
3. **MUST NOT** require any member of the payload in order to proceed. Every
   member is optional, and an empty payload is the ordinary case.

## Authorization

*Declared under [SPEC.md §7.3](/SPEC.md#73-specification-requirements)
item 15.*

This task is not *consequential* — it changes no recipient state and discloses
only what the acknowledging party chose to surface about the producer's own
exchange — so item 15 does not bind it. The declaration is made anyway because
its absence would be read as an oversight: **this document presupposes no
authorization evidence at all.**

There is nothing to authorize. An acknowledgement grants nothing, entitles its
recipient to nothing, and per Conformance may not be relied upon even when
genuine. A *consumer* receiving one from a party it did not transact with
discards it; that is a correlation matter rather than an authorization one, and
the `threadId` and `recipient` checks of
[SPEC.md §7.2](/SPEC.md#72-consumer-requirements) already cover it.

## Payload

Every member is **optional**. An empty payload — `{}` — is the ordinary case and
means exactly "received and performed".

`message` (optional) — human-readable confirmation for operator UI and logs.
Non-normative; a *producer* **MUST NOT** parse it for any value it needs.

`refs` (optional) — opaque `{ name, value }` references the acknowledging party
chose to surface: a ticket number, a queue position, a processing handle, a
retention deadline. **Convenience only.** The names are drawn from no registry
and are not interoperable; a party that does not recognize one ignores it.

The constraint that keeps this member honest: a value a *producer* needs in
order to proceed is part of the originating task's semantics and belongs in a
response that task defines. **A producer parsing `refs` to continue an exchange
is using the wrong document**, and the task it is performing should declare its
own `#response`. Without that line this member becomes the place specifications
put things they could not be bothered to model.

`ext` (optional) — the framework extension slot
([SPEC.md §4.5.1](/SPEC.md#451-the-ext-extension-member)).

## Examples

The ordinary case — a fire-and-forget task acknowledged with nothing to add:

```json
{
  "id": "urn:uuid:4c2f8b1a-0001-4a10-8a00-000000000001",
  "type": "https://trusttasks.org/spec/trust-task-ok/0.1",
  "threadId": "urn:uuid:6f1c8b2a-0001-4a10-8a00-000000000001",
  "issuer": "did:web:maintainer.example",
  "recipient": "did:web:agent.example",
  "issuedAt": "2026-08-16T11:02:00Z",
  "payload": {}
}
```

With a reference the operator may find useful, and a proof:

```json
{
  "id": "urn:uuid:4c2f8b1a-0002-4a10-8a00-000000000002",
  "type": "https://trusttasks.org/spec/trust-task-ok/0.1",
  "threadId": "urn:uuid:6f1c8b2a-0001-4a10-8a00-000000000001",
  "issuer": "did:web:maintainer.example",
  "recipient": "did:web:agent.example",
  "issuedAt": "2026-08-16T11:02:00Z",
  "payload": {
    "message": "Event accepted and queued for fan-out.",
    "refs": [
      { "name": "queuePosition", "value": "3" }
    ]
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-08-16T11:02:00Z",
    "verificationMethod": "did:web:maintainer.example#key-1",
    "proofPurpose": "assertionMethod",
    "proofValue": "z…(signature over the JCS-canonical document)"
  }
}
```

## Security & Privacy

**Silence carries no information, and that is normative.** The rule exists
because the alternative is worse in both directions. A *producer* that reads
absence as failure and reissues a *consequential Trust Task* causes the
duplicate effect [SPEC.md §7.2](/SPEC.md#72-consumer-requirements) item
11 exists to prevent; one that reads absence as success can believe work
happened that never did. Neither reading is available: an acknowledgement is
informative when it arrives and meaningless when it does not.

**A forged acknowledgement gains little, by construction.** Because a *producer*
may not act on an acknowledgement, an attacker who fabricates one cannot induce
an action — which is why `proof` is RECOMMENDED rather than REQUIRED. The
residual harm is to the record: these documents are routinely logged, and an
unattributable entry in an operator's account of what was acknowledged is worse
than no entry. A deployment that retains acknowledgements **SHOULD** require the
proof.

**`refs` is a disclosure surface.** It is free-form and populated by the
acknowledging party, which makes it the easiest place in this document to leak
something. A *consumer* **SHOULD** confine it to values the *producer* already
possesses or could trivially derive, and **MUST NOT** place secret material in
it: `exposure.discloses` is `metadata`, and a specification that needs to return
a secret is not describing an acknowledgement.
