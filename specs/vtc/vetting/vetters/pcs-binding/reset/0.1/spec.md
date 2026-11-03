---
slug: vtc/vetting/vetters/pcs-binding/reset
version: "0.1"
title: "VTC — Vetting PCS Binding — Reset"
summary: A community administrator resets a vetter's hidden-vetting identifier binding after the key behind it is lost. The community accepts a new identifier from that member only once no label the old one enrolled under is still live, so the member is never counted twice.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vetting
  - hidden-vetting
  - pcs
  - recovery
  - admin
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: any
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    A reset lets a member hold a second hidden-vetting key over their life in the community. It
    must be attributable to the administrator who made it, on every transport, and survive as the
    record a later review of the community's hidden admissions reads back.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    A reset replayed after the member re-bound would reset them again. Placing the instruction in a
    window is what lets the community refuse the replay.
sideEffects:
  level: mutating
  rationale: >-
    Marks the member's enrolment record as reset. Recoverable until a new identifier is bound: the
    member enrolling again with the recorded identifier cancels the reset.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    In: one member DID and a reason. Out: the reset's time and the labels it waits on — never the
    member's identifier.
retention:
  class: durable
  rationale: >-
    The reset stays on the member's enrolment record until a new identifier is bound, and the audit
    record of it for as long as the community keeps its audit log.
errorCodes:
  - code: vtc/vetting/vetters/pcs-binding/reset:notBound
    meaning: "The community holds no hidden-vetting binding for `memberDid`: the member never enrolled, so there is nothing to reset."
    retryable: false
related:
  - vtc/vetting/vetters/pcs-root
  - vtc/vetting/vetters/pcs-binding/show
  - vtc/vetting/vetters/grant
  - vtc/vetting/hidden/publish
  - vta/pcs/vetter/recover
---

## Abstract

[`vtc/vetting/vetters/pcs-root`](../../../pcs-root/0.1/spec.md) binds each vetter to one PCS
identifier for good: every tag a vetter makes is a function of the secret behind that identifier,
and distinct tags are how a hidden-vetting proof counts distinct people. A member who comes back
with a second identifier is refused with `identifierRebound`, because a second key under a live
label would let them attest twice for the same applicant and be counted as two vetters.

That rule has no exit when the key is genuinely lost. Revoking and re-granting the vetter role
does not touch the binding, and a client that cannot recover the key
([`vta/pcs/vetter/recover`](../../../../../../vta/pcs/vetter/recover/0.1/spec.md)) is locked out of
hidden vetting for as long as the member stays in the community.

**VTC — Vetting PCS Binding — Reset** is the exit. An administrator resets the binding. The
community then accepts a new identifier from that member — but only once every label the old
identifier enrolled under has left the live set. Until then a credential under the old key could
still attest, and binding a second key would reopen exactly the double count the binding exists
to prevent. Waiting closes it without trusting anyone's word that the old key is gone.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. Applies the [SPEC §7.2](/SPEC.md#72-consumer-requirements) pipeline, and refuses a sender
   without the community-administrator capability with the framework's `permissionDenied`.
2. Refuses a `memberDid` it holds no enrolment record for, or a record with no identifier, with
   `notBound`. Current membership is not required: a removed member's binding may be reset ahead
   of their return, and resetting it grants nothing while they hold no live vetter grant.
3. Where the member's binding is already reset and not yet re-bound, **MUST** return the existing
   reset's `resetAt` unchanged and record nothing new. A repeated reset is indistinguishable from
   the first ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 11).
4. Otherwise **MUST** record the reset on the member's enrolment record — keeping the recorded
   identifier and its enrolled labels — and **SHOULD** audit it with `reason`, the administrator
   and the member, and **MUST NOT** put the identifier in the audit record.
5. Answers with `waitingOn` and `open` computed against the labels it accepts at the time.

### Enrolment after a reset

While a member's binding is reset, a community applying
[`pcs-root`](../../../pcs-root/0.1/spec.md) to a request from that member:

1. **MUST**, when the request's `id` is the recorded identifier, apply `pcs-root` exactly as if
   there were no reset, and on issuing **MUST** clear the reset. The key was found; nothing
   changes about who the member is. This is the outcome to prefer, and a client SHOULD attempt
   recovery before asking for a reset.
2. **MUST**, when the request's `id` differs, compute the recorded identifier's enrolled labels
   that are still live **at the time of the request** — not at the time of the reset — and refuse
   with `pcs-root:identifierRebound` if any is.
3. Otherwise **MUST** apply every other `pcs-root` check, and on issuing **MUST** record the new
   identifier in place of the old, start its enrolled labels afresh with the label just issued,
   clear the reset, and **SHOULD** audit the re-binding.

Rule 2 is evaluated at each request so that it holds however the live set later moves: a
community that turns hidden vetting off and on again with an old label still listed keeps
refusing a new identifier until that label is gone.

### Labels never come back

A community that has re-bound a member **MUST NOT** afterwards accept attestations under any label
the old identifier was enrolled under, and so **MUST** refuse to make such a label live again
through [`vtc/vetting/hidden/publish`](../../../../hidden/publish/0.1/spec.md). A label that has
left the live set is retired for good, for every vetter — publish refuses it with
`hidden/publish:labelRetired`; vetter labels name months, and a past month never needs to
be live again.

### Shortening the wait

The wait is at most as long as the community keeps old labels live — typically the current and
the previous month. An administrator who needs the member back sooner MAY rotate early: publish a
new current label and drop the labels the old identifier holds from the live set
([`vtc/vetting/hidden/publish`](../../../../hidden/publish/0.1/spec.md)). That affects every vetter
— each re-enrols under the new label, and attestations held under a dropped label must be
refreshed — which is why this task does not do it implicitly.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authority is the **community-administrator capability**, the same that
[`vtc/vetting/vetters/grant`](../../../grant/0.1/spec.md) requires: who may hold a second key over
their life in the community is a governance decision. No member can reset their own binding. Per
[SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10, the request's `proof` establishes which
administrator asked, never that they hold the capability.

The reset grants nothing by itself. It does not issue a credential, restore a revoked vetter
grant or let the member enrol while the old labels are live; [`pcs-root`](../../../pcs-root/0.1/spec.md)
still requires a live vetter grant at enrolment.

## Definitions

**Recorded identifier**, **live labels**, **`waitingOn`** — as in
[`vtc/vetting/vetters/pcs-binding/show`](../../show/0.1/spec.md).

**Re-binding** — the enrolment under a new identifier that completes a reset.

## Request

The administrator sends the request to the community. See the top-level schema in
[`payload.schema.json`](payload.schema.json).

### Resetting a vetter whose key was lost

```json
{
  "id": "urn:uuid:5e2a9f10-6b3c-4d7e-8f01-000000000001",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/pcs-binding/reset/0.1#request",
  "issuer": "did:example:admin",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-02T14:20:00Z",
  "threadId": "urn:uuid:5e2a9f10-6b3c-4d7e-8f01-0000000000ff",
  "payload": {
    "memberDid": "did:example:vetter",
    "reason": "Vetter rebuilt their VTA; the old hidden-vetting key cannot be recovered."
  }
}
```

## Response

The community answers with the payload under `$anchor: "response"` in
[`payload.schema.json`](payload.schema.json). Every refusal is a `trust-task-error`.

### Reset, waiting on the previous month's label

```json
{
  "id": "urn:uuid:5e2a9f10-6b3c-4d7e-8f01-000000000002",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/pcs-binding/reset/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:admin",
  "issuedAt": "2026-11-02T14:20:01Z",
  "threadId": "urn:uuid:5e2a9f10-6b3c-4d7e-8f01-0000000000ff",
  "payload": {
    "memberDid": "did:example:vetter",
    "resetAt": "2026-11-02T14:20:00Z",
    "waitingOn": ["vetter/2026-10"],
    "open": false
  }
}
```

## Security & Privacy

### Data carried

A member DID and a reason in; a time and labels out. The member's PCS identifier never appears in
either direction or in the audit record: the enrolment row is already one half of a future
deanonymisation, and the audit log holds no second copy.

### Correlation

The community already knows which members enrolled. A reset adds that this member lost a key,
which the administrator learns from the member anyway.

The community is declared `identifierScope: public` because the reset acts on its own enrolment
record, which is keyed on its published DID and on the member DID it recognises; a pairwise
identifier for the community would leave the administrator unable to say which community's record
they are changing.

### Retention

The reset lives on the enrolment record until re-binding or cancellation. A community SHOULD keep
the old identifier's retired labels on the record after re-binding for as long as it needs to
enforce [Labels never come back](#labels-never-come-back), and MAY keep only the labels.

### Consent/purpose

A reset is made at the member's request, to restore their ability to vet. A community MUST NOT
reset a binding to remove a vetter — revoking the vetter grant does that, and a reset does not.
