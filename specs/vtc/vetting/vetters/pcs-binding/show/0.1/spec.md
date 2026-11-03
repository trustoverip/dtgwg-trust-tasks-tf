---
slug: vtc/vetting/vetters/pcs-binding/show
version: "0.1"
title: "VTC — Vetting PCS Binding — Show"
summary: A member asks its community which hidden-vetting identifier it is bound to, so its client can recover the matching key — or learn that an administrator's reset is pending and when a new key will be accepted.
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords:
  - vetting
  - hidden-vetting
  - pcs
  - recovery
  - enrolment
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: member
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: community
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: >-
    The answer is one member's enrolment record — one half of a future deanonymisation should the
    suite's assumptions fail. It is given to that member only, and the community must be able to
    attribute the read to them on every transport, relayed ones included.
sideEffects:
  level: none
  rationale: Reads the requester's own enrolment record. Persists nothing.
exposure:
  discloses: metadata
  ingests: none
  actsAsSubject: false
  rationale: >-
    Discloses to the member their own PCS identifier, the labels they enrolled under and the state
    of any reset. Nothing about any other member.
retention:
  class: transient
  rationale: A read; the community keeps nothing.
errorCodes:
  - code: vtc/vetting/vetters/pcs-binding/show:notAMember
    meaning: "The requester is not a member of this community."
    retryable: false
related:
  - vtc/vetting/vetters/pcs-root
  - vtc/vetting/vetters/pcs-binding/reset
  - vta/pcs/vetter/recover
---

## Abstract

A community that runs hidden vetting binds each vetter to one PCS identifier at their first
enrolment ([`vtc/vetting/vetters/pcs-root`](../../../pcs-root/0.1/spec.md)), and refuses any other
identifier from that member with `pcs-root:identifierRebound`. A vetter whose client or VTA has
lost track of the key behind that identifier — a re-created context, a rebuilt engine record —
is then locked out, and the refusal alone does not say which key the community wants.

**VTC — Vetting PCS Binding — Show** answers that question for the member, and only for them. With
the identifier in hand, a VTA-held key can be searched for and restored
([`vta/pcs/vetter/recover`](../../../../../../vta/pcs/vetter/recover/0.1/spec.md)). If it cannot be
restored, the member asks an administrator to reset the binding
([`vtc/vetting/vetters/pcs-binding/reset`](../../reset/0.1/spec.md)), and this read says when a new
identifier will be accepted.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

A conforming **community** (`recipient`):

1. Applies the [SPEC §7.2](/SPEC.md#72-consumer-requirements) pipeline, and refuses a requester
   that is not a member with `notAMember`.
2. **MUST** answer about the requester only. The request names no member, and a community
   **MUST NOT** accept a member identifier in `ext` or anywhere else.
3. **MUST** include `id` when `bound` is true and **MUST NOT** include it otherwise.
4. Answers `bound: false` and an empty `enrolledLabels` for a member it holds no enrolment record
   for — including a member who is not a vetter. Whether a member holds a vetter grant is not
   what this task reports.
5. When a reset is pending ([`pcs-binding/reset`](../../reset/0.1/spec.md)), **MUST** include
   `reset`, with `waitingOn` computed against the labels it accepts at the time of the answer.

## Authorization

The entitlement is **membership of this community**, checked against the community's own
records. A vetter grant is not required: a member whose grant was revoked and re-issued is exactly
the member who needs this read. Per [SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements), the
proof establishes who is asking; it is the community's membership record that establishes they
may.

## Definitions

**Recorded identifier** (`id`) — the PCS identifier the community bound to this member at their
first enrolment, as [`pcs-root`](../../../pcs-root/0.1/spec.md) defines it.

**Live labels** — the vetter class labels (`vetter/<YYYY-MM>`) the community currently accepts
attestations under: its published `liveVetterLabels`. Token labels and event labels are not
class labels and never appear in `waitingOn`. With hidden vetting withdrawn, no label is live and
a pending reset reads `open: true`; that can change back if hidden vetting is published again with
a waited-on label still listed, which is why a community re-checks at every enrolment request
([reset](../../reset/0.1/spec.md#enrolment-after-a-reset)).

**`waitingOn`** — the recorded identifier's enrolled labels that are still live. A credential
under any of them can still attest, so a second identifier bound now could count the same member
twice in one proof.

### Reading the answer after a refusal

`pcs-root` refuses a new identifier with the same `identifierRebound` whether the member is bound
for good or a reset is waiting on a live label. This read is how a client tells the two apart:
`reset` absent means recover the key or ask for a reset; `reset` with `open: false` means wait for
the labels in `waitingOn` to retire; `open: true` means enrol now. A client waiting on a reset
SHOULD read again when the community publishes a new set of live labels (a change of its join
manifest's `liveVetterLabels`), and SHOULD NOT read more often than every 15 minutes otherwise:
labels retire on the community's schedule, at most monthly.

## Request

The member sends the request to the community. The payload is the top-level schema in
[`payload.schema.json`](payload.schema.json), and is normally empty.

### A vetter asks which identifier it is bound to

```json
{
  "id": "urn:uuid:3b0e7c52-1d4a-4f60-9a51-000000000001",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/pcs-binding/show/0.1#request",
  "issuer": "did:example:vetter",
  "recipient": "did:example:community",
  "issuedAt": "2026-11-03T09:00:00Z",
  "threadId": "urn:uuid:3b0e7c52-1d4a-4f60-9a51-0000000000ff",
  "payload": {}
}
```

## Response

The community answers with the payload under `$anchor: "response"` in
[`payload.schema.json`](payload.schema.json). Every refusal is a `trust-task-error`.

### Bound, with a reset waiting on a live label

```json
{
  "id": "urn:uuid:3b0e7c52-1d4a-4f60-9a51-000000000002",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/pcs-binding/show/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:vetter",
  "issuedAt": "2026-11-03T09:00:01Z",
  "threadId": "urn:uuid:3b0e7c52-1d4a-4f60-9a51-0000000000ff",
  "payload": {
    "bound": true,
    "id": "z5jokfsiZx1sk1mJyQnBnR519B9mcQhw9a8LrF9VQdAjExwoLzcdVdZ2aXaH14a3mdF",
    "enrolledLabels": ["vetter/2026-09", "vetter/2026-10"],
    "reset": {
      "resetAt": "2026-11-02T14:20:00Z",
      "waitingOn": ["vetter/2026-10"],
      "open": false
    }
  }
}
```

## Security & Privacy

### Data carried

The member's own PCS identifier and enrolled labels. The identifier is public by construction
(`id = g^usk`), but the community's record joining it to a member DID is the sensitive half; this
task hands that record only to the member it describes.

### Correlation

None beyond what the member already knows about themselves. A client SHOULD NOT forward the
answer anywhere but to its own PCS engine.

### Authenticity of the answer

The answer travels on the same authenticated channel as the request. A client SHOULD treat it as
it treats the community's other enrolment answers. A forged answer could at worst point the
member's own engine at an identifier it cannot derive, which ends in
`vta/pcs/vetter/recover:notFound` and changes nothing.

### Retention

None at the community. A client MAY keep the identifier to compare against its engine, and SHOULD
discard it once the engine matches.
