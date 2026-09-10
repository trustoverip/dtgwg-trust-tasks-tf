---
slug: keys/set-exportability
version: "0.1"
title: "Keys Set Exportability"
summary: "Mark a key as releasable to callers, or refuse every future export of it."
status: draft
targetFrameworkVersion: "0.5.0"
category: key-management
keywords: [keys, exportability, custody, non-extractable, key-management]
parties:
  - role: "The party asking for the key's exportability to change"
    requirement: REQUIRED
    member: issuer
  - role: "The custodian holding the key"
    requirement: REQUIRED
    member: recipient
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    The request is the only evidence of who asked for a key's custody rule to change,
    and one direction of this task removes a protection. An unsigned ask must never be
    sufficient for that. The response is the custodian's own record of the state it now
    enforces, which is what a later audit reads to establish when a key stopped — or
    started — being releasable.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    What matters about this task is usually *when* it happened relative to something
    else — an export, an incident, a rotation. A document with no issuedAt cannot be
    placed in that order, nor in the bounded window SPEC §7.2 item 11 needs to absorb a
    duplicate.
sideEffects:
  level: mutating
  rationale: >-
    Changes one member of a stored key record and nothing else. No key is created,
    rotated, revoked or destroyed, and the material itself is untouched. Reversible in
    principle — but see Authorization: the two directions are not equally easy to
    travel, and that asymmetry is the point of the task.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: >-
    The response carries the key record — identifiers, type, status, the public half —
    and never the private material. Setting exportability neither reads nor releases the
    key.
retention:
  class: durable
  rationale: >-
    The custodian keeps the new state for the key's lifetime; that persistence is the
    whole function. The documents themselves are worth keeping for the same reason the
    response proof is required — they are the record of who changed a custody rule.
errorCodes:
  - code: keys:notFound
    meaning: No key record on this custodian carries the named `keyId`.
    retryable: false
  - code: keys/set-exportability:notPermittedForThisKey
    meaning: "The key's material can never be released regardless of this member, so asking for `exportable` true is asking for something the custodian cannot honour. Distinct from a permission refusal — no caller, at any authority, can change this answer."
    retryable: false
related:
  - keys/show
  - keys/list
  - keys/revoke
---

## Abstract

A custodian holds private keys and will, on request, release some of them to a caller
that is entitled to them. For some keys that is the point — a service fetches the keys of
the identity it operates. For others it is a risk worth foreclosing: the key signs
something whose authority should never be reproducible outside the custodian, and no
future caller, however well authorised, should be able to walk away with it.

This task records that decision on the key itself. `exportable: false` tells the
custodian to refuse every future export of the key and to serve only *uses* of it —
signing, key agreement — so the material never leaves. `exportable: true` restores the
ordinary state.

It is a Trust Task rather than a flag on some other request because it is a decision
about custody, made deliberately and worth having signed evidence of on both sides. A key
that stopped being exportable, or started, is exactly the kind of change an incident
review needs to be able to place in time and attribute to someone.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here, and the invariants in [the category conventions](../../_shared/0.1/CONVENTIONS.md).

## Authorization

The authority is **standing over the key's scope**: the custodian has recorded this
producer as entitled to administer the context the key belongs to, and a producer with no
authority over that scope is refused with the framework's `permissionDenied`.

**The two directions do not carry the same entitlement, and a consumer MUST NOT accept
the same evidence for both.** Imposing the restriction (`exportable: false`) forecloses
something; removing it (`exportable: true`) restores an ability that was deliberately
taken away, and it is the only step that can make a protected key leave. A consumer
**MUST** require strictly stronger authorization to set `exportable: true` on a key that
currently carries `false` than it requires to set `false` in the first place.

This is what makes the member worth having. A restriction that the party who imposed it
can lift unilaterally protects against accident but not against a compromised caller
holding that party's credentials — which is the case the restriction exists for. Because
the two directions are checked differently, an attacker who reaches the authority that
can impose the restriction does not thereby reach the authority that can remove it.

What "strictly stronger" is made of is the **consumer's** policy, and this specification
deliberately does not name it: an approval from a second party, an operator with
authority the ordinary administrator lacks, or a re-authentication are all reasonable
readings, and per [SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) a
specification **MUST NOT** declare that consent, human approval or a step-up is required.
What it does require is the *relation* between the two: whatever a consumer accepts for
`false`, it must demand more for `true`.

None of this is identity checking. Per
[SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), verifying the `issuer`, the
transport identity or the `proof` establishes *who asked* and *that the request is
unaltered*; it never establishes that this asker may change this key's custody rule.

## Definitions

**`keyId`** — the key whose exportability changes. Chosen by the producer, from the keys
it can already see via `keys/list` or `keys/show`. Custodian-scoped; there is no globally
meaningful form.

**`exportable`** — the state the key should be in when the request succeeds. Chosen by
the producer. `false` means the custodian **MUST** refuse every subsequent request to
release the key's private half, and **MUST** continue to serve requests that merely *use*
the key. `true` means ordinary release rules apply again — it is not itself a grant, and
a caller still has to be entitled to the key to receive it.

Stated absolutely rather than as a toggle. A producer that retried a request whose reply
was lost must land in the state it asked for, not the opposite one; see Response.

**The response's `exportable`** — the state the custodian now enforces. A producer
**SHOULD** read it rather than assume its request was honoured verbatim: a custodian that
cannot make a key exportable answers `notPermittedForThisKey` rather than silently
succeeding, but a custodian is also free to have been already in the requested state.

## Request

The producer is the party asking; the recipient is the custodian holding the key. The
payload names one key and the state it should carry — see the top-level schema in
[`payload.schema.json`](payload.schema.json).

### Foreclosing export of a key that should never leave

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/keys/set-exportability/0.1#request",
  "issuer": "did:example:administrator",
  "recipient": "did:example:custodian",
  "issuedAt": "2026-01-01T00:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "keyId": "did:example:issuer#key-0",
    "exportable": false
  }
}
```

## Response

The producer of the response is the recipient of the request. The payload wraps the key record
as it now stands in a `key` member, reachable via `$anchor: "response"` in
[`payload.schema.json`](payload.schema.json). Failures use `trust-task-error` rather than
a `#response` document.

The task is **idempotent**: a request whose `exportable` already matches the record
succeeds and returns that record. A consumer **MUST NOT** treat "already in this state"
as a conflict. This follows from `exportable` being an absolute state rather than a
delta, and it is what makes the task safe to retry when a reply is lost — the failure
this avoids is a producer that retries a lost `false` and thereby sets `true`.

### The record, now non-exportable

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/keys/set-exportability/0.1#response",
  "issuer": "did:example:custodian",
  "recipient": "did:example:administrator",
  "issuedAt": "2026-01-01T00:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "key": {
      "keyId": "did:example:issuer#key-0",
      "keyType": "ed25519",
      "status": "active",
      "publicKey": "z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH",
      "exportable": false,
      "createdAt": "2026-01-01T00:00:00Z",
      "updatedAt": "2026-01-01T00:00:01Z"
    }
  }
}
```

## Security & Privacy

### Data carried

Both documents carry key **metadata** — an identifier, a type, a status, the public half
— and never private key material. That is true of the response in particular: this task
changes whether the private half may be released and does not itself release it.

A producer **MUST NOT** put key material in `keyId` or in `ext`. There is no free-form
member for it to occupy, and `ext` is not one.

### Correlation

The custodian learns which keys an administrator considers sensitive, and when that
assessment changed. To an observer of the channel, a sequence of these requests reveals
the shape of a custody policy — which keys are being locked down, and in what order —
without revealing the keys themselves. That is inherent: the task's subject is the
identifier, and the identifier is what has to be on the wire.

A `keyId` that embeds a DID correlates the key to that identity for anyone who sees the
request, exactly as it does in every other `keys/*` task.

### Retention

The custodian keeps the resulting state for the key's lifetime — that persistence is the
function, not a side effect. The documents themselves **SHOULD** be retained as the
record of who changed a custody rule and when, which is the reason
`proofRequirement.response` is REQUIRED. A consumer that discards them keeps the
protection but loses the ability to say who applied it.

### Consent/purpose

The purpose is administering the custodian's own key set. Neither document carries
personal data beyond the identifiers a `keys/*` task inherently carries, and neither
needs to be shared outside the two parties to serve its purpose. Descriptive only: per
[SPEC §7.3 item 13](/SPEC.md#73-specification-requirements) a specification **MUST NOT**
declare that consent, approval or a step-up is required — including for the asymmetry
described under Authorization, which constrains the *relation* between two entitlements
and not the mechanism a consumer uses to establish either.
