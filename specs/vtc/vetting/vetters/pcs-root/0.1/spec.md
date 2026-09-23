---
slug: vtc/vetting/vetters/pcs-root
version: "0.1"
title: "VTC — Vetting PCS Root Credential"
summary: A vetter asks its community to blind-sign the class credential for a vetting period, so it can later attest to applicants without the community learning which vetter attested.
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords:
  - vetting
  - hidden-vetting
  - blind-signature
  - enrolment
  - anonymous-credential
parties:
  - role: vetter
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
    The community signs on the strength of who is asking: a member holding a live vetter grant, at
    most once per class label. A proofless request would let anyone claim another member's one
    enrolment, and the community cannot tell afterwards — the credential it issued is one it
    deliberately cannot recognise.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: Enrolment is once per member per label. Placing the request in time is what lets a community tell a first attempt from a replay of one it has already served.
sideEffects:
  level: mutating
  rationale: The community records that this member has enrolled under this label, and binds them to the identifier they presented.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
retention:
  class: durable
  rationale: >-
    The enrolment record is what enforces one credential per member per label, and one identifier
    per member across labels — the rule that makes distinct tags mean distinct people. It outlives
    the exchange by design. It is also one half of a future deanonymisation, so a community keeps
    the smallest record that enforces the rule and nothing more.
errorCodes:
  - code: vtc/vetting/vetters/pcs-root:notAVetter
    meaning: "The requester holds no live vetter grant in this community."
    retryable: false
  - code: vtc/vetting/vetters/pcs-root:wrongLabel
    meaning: "`label` is not the label this community is currently issuing under."
    retryable: false
  - code: vtc/vetting/vetters/pcs-root:alreadyEnrolled
    meaning: "This member already holds a credential under `label`. One per member per label is what makes distinct tags distinct people."
    retryable: false
  - code: vtc/vetting/vetters/pcs-root:identifierRebound
    meaning: "`id` is not the identifier this member was bound to at their first enrolment. A member holding two identifiers could be counted twice in one proof."
    retryable: false
  - code: vtc/vetting/vetters/pcs-root:badRequest
    meaning: "The blinded request does not verify under the community's published parameters."
    retryable: false
related:
  - vtc/vetting/vetters/grant
  - vtc/vetting/vetters/pcs-tokens
  - vetting/attestation
---

## Abstract

A community that hides its vetters still has to know which of its members may vet. It resolves
that at enrolment: the member proves who they are, the community checks its own records, and what
it signs is a commitment it cannot open. The credential the vetter unblinds from the answer is one
the community has never seen, which is why an attestation made under it can be counted without
being attributed.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

The entitlement is a **live vetter grant in this community** — the same grant
[`vtc/vetting/vetters/grant`](../../grant/0.1/spec.md) records and the named vetting path checks
before it counts a statement. A community MUST check it against its own membership and grant
records at the moment of issuance, not against anything in the request: per
[SPEC §7.2 item 10](/SPEC.md#72-consumer-requirements) the proof establishes who is asking and
nothing about what they may have.

Two further checks are authorization, not validation, and a consumer MUST make both: this member
has not already enrolled under this label, and the identifier presented is the one they were bound
to. Both exist because the counting rule downstream assumes one credential per member per label.

## Definitions

**`label`** — the class label asked for, `vetter/<YYYY-MM>`. The period lives in the label rather
than in the community's key, so that one proof can carry attestations made either side of a
rotation. A community issues under its current label only.

**`id`** — the vetter's PCS identifier, derived from a secret only the vetter holds. The community
records it at the first enrolment and requires the same one at every later label.

**`request`** — the blinded root request. Its members are fixed by the suite the community
publishes in its criterion, not by this schema; a consumer verifies it under the published
parameters and learns nothing else from it.

**`preCredential`** — what the community signs. Only the holder of the request's blinding state
can unblind it, and what they unblind is not derivable from this document.

## Request

The vetter issues the request to its community; the payload is the top-level object in
[`payload.schema.json`](payload.schema.json).

### A vetter enrols for the current period

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000001",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/pcs-root/0.1#request",
  "issuer": "did:example:vetter",
  "recipient": "did:example:community",
  "issuedAt": "2026-09-01T09:00:00Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "label": "vetter/2026-09",
    "id": "z5jokfsiZx1sk1mJyQnBnR519B9mcQhw9a8LrF9VQdAjExwoLzcdVdZ2aXaH14a3mdF",
    "request": {
      "encoding": "z6P4jNSt4qHKYhSBGSWgr6VcQGMsjmfYGUBdYWNtS7xbpGsTuiY1PDfoce4yTApuqUm",
      "proof": "zSzFTCny3qaXpSHTsTE877ffQfSuF9T5zo53iuZkwx9NbLYSq3GeLZpRda286SBhqwJg"
    }
  }
}
```

## Response

The community answers with the blind pre-credential, in the sub-schema reachable via
`$anchor: "response"`. Every refusal above is a `trust-task-error`, never a `#response` document.

A community SHOULD record the enrolment only after the signature exists: losing the record costs
a repeated enrolment the vetter asks for, while recording one that was never signed costs the
member their single enrolment for the period.

### The blind pre-credential

```json
{
  "id": "urn:uuid:00000000-0000-4000-8000-000000000002",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/pcs-root/0.1#response",
  "issuer": "did:example:community",
  "recipient": "did:example:vetter",
  "issuedAt": "2026-09-01T09:00:01Z",
  "threadId": "urn:uuid:00000000-0000-4000-8000-0000000000ff",
  "payload": {
    "label": "vetter/2026-09",
    "preCredential": "z3Xef1JY2x1s2vY6yKWfLu8Ap2vKuoZGwaQLZ2tjq8QkEZbGeZyaNgtznZ8B7N1yf14"
  }
}
```

## Security & Privacy

### Data carried

The request carries the member's PCS identifier and a blinded commitment; the response carries a
signature over a value the community cannot read. Nothing here names an applicant, because at
enrolment there is no applicant — a vetter enrols before it knows who it will meet. A producer
MUST NOT put an applicant identifier in `ext`.

### Correlation

The community learns exactly which members enrolled and when, which it already decides. What it
cannot do is join that list to any later attestation: the credential is blind, and the tags made
under it are unlinkable to the identifier recorded here. The enrolment record and the tags are,
however, the two halves of a future deanonymisation should the suite's assumptions fail, so a
community SHOULD treat this record's lifetime as a privacy decision rather than bookkeeping.

Both parties are declared `identifierScope: public` because enrolment is exactly the point where
a vetter is named. The community must resolve the requester to a membership row and a live grant,
and must recognise the same member again at the next label to enforce one credential each; a
pairwise identifier would make both impossible. That naming is deliberate and bounded: it happens
here, once per label, and never again on the path — the attestation this credential is later used
for carries no identifier at all.


### Retention

The record is kept for as long as the labels it covers are live, because it is what enforces
one credential per member per label. A community SHOULD keep the identifier binding for as long
as it issues overlapping labels, and MAY discard the record once no label it names is accepted.

### Consent/purpose

The identifier is collected to enforce the enrolment rules of this community and for nothing else.
Using it to attribute an attestation is outside that purpose, and the design is deliberately such
that it cannot be done from what is exchanged here.
