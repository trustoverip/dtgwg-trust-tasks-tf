---
slug: vtc/vetting/hidden/publish
version: "0.1"
title: "VTC Vetting — Hidden — Publish"
summary: An administrator turns on hidden-vetter admission for one criterion, or rotates its live labels and events, and reads back what the community now stores and now publishes.
status: draft
targetFrameworkVersion: "0.6.0"
category: identity
keywords:
  - vetting
  - hidden-vetting
  - blind-signature
  - pcs
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
    This is the switch that turns a criterion's named vetting into one that also accepts a hidden proof, and it rotates the labels a whole community of vetters and applicants relies on. Only administrator standing may throw it, so the signer must be attributable on every transport.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor, and §7.2 item 11 can only absorb a duplicate inside a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Stores the criterion's hidden-vetting parameters and republishes them in the join manifest under vetting.ext. Deriving the keys again with the same labels changes nothing (the keys are a function of the community's signer secret); publishing different live periods, token labels or events is a real change that moves the criterion's requirementsDigest and can retire labels applicants and vetters were relying on.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    Configuration in — periods, token labels, event windows and rates, optionally who approved an event — the stored and published configuration out. The verification keys returned are public values by design (see Definitions); nothing here is the community's signing secret, which never leaves where it is derived.
retention:
  class: durable
  rationale: >-
    The stored configuration lives until the next publish, and it is what the community mints hidden attestations against for as long as its labels stay live. The change is audited against the administrator who made it.
errorCodes:
  - code: vtc/vetting/hidden/publish:noSuchCriterion
    meaning: "`criterionId` does not name an Accepts criterion this community has stored."
    retryable: false
  - code: vtc/vetting/hidden/publish:noVetting
    meaning: "The named criterion asks for no vetting requirements at all, so there is nothing for hidden-vetting parameters to qualify. Give the criterion vetting requirements first."
    retryable: false
  - code: vtc/vetting/hidden/publish:approverNotSigner
    meaning: "An event's `approvedBy` is new or changed relative to the stored configuration and does not name the publishing signer's principal. An approver names themselves; nobody approves on another's behalf."
    retryable: false
  - code: vtc/vetting/hidden/publish:approverInEvent
    meaning: "An event's `approvedBy` names a member who has themselves asked to vet at that event through `vtc/vetting/vetters/event-mode`. A vetter cannot approve the event it asked to join."
    retryable: false
  - code: vtc/vetting/hidden/publish:signerChanged
    meaning: "The criterion's stored helper and token keys are not the ones the community's current credential-signer secret derives — the signer was rotated, or the community was restored under another key. Re-publishing would re-key silently and every enrolled vetter's credential would stop verifying. Withdraw hidden vetting from the criterion and publish it again, under a new period, to start a new key generation."
    retryable: false
  - code: vtc/vetting/hidden/publish:otherCriterion
    meaning: "Hidden vetting is already on for a different criterion. A community runs it on at most one criterion, because enrolment, the drip and the challenge are served from a single configuration. Withdraw it from the other criterion first."
    retryable: false
related:
  - vtc/vetting/hidden/show
  - vtc/vetting/hidden/withdraw
  - vtc/vetting/vetters/pcs-root
  - vtc/vetting/vetters/pcs-tokens
  - vtc/vetting/pcs-challenge
  - vtc/vetting/vetters/event-mode
  - vtc/schemas/accepts/register
---

## Abstract

A community that hides its vetters still needs to publish four things before anyone can use hidden vetting for a criterion: the blind-signature suite, the two verification keys, and which class and token labels are currently live. **VTC Vetting — Hidden — Publish** is the one call that derives the keys, stores the parameters against the named criterion, and republishes the criterion's manifest entry so the new digest and the new parameters are what an applicant and a vetter read from that point on.

The keys are **derived, never chosen**: they come from the community's credential-signer secret through HKDF, so there is no second secret to provision, back up or leak — the signer secret *is* the vetter class. Sending this task again with the same labels derives the same keys and stores the same parameters; sending it with different labels **rotates** them, which is a real change that moves `requirementsDigest` and can retire a label an application in progress was counting on.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **administrator** (`issuer`) names the criterion in `criterionId` and, optionally, the live periods, live token labels, drip rate, tick length and events to publish under it. Every member but `criterionId` defaults to the current month's ordinary values when absent — the useful call is the short one.

A conforming **community** (`recipient`):

1. Applies the [SPEC §7.2](/SPEC.md#72-consumer-requirements) pipeline and **MUST** refuse a caller without administrator standing with `permissionDenied`.
2. **MUST** refuse with `vtc/vetting/hidden/publish:noSuchCriterion` when `criterionId` names no stored criterion.
3. **MUST** refuse with `vtc/vetting/hidden/publish:noVetting` when the named criterion asks for no vetting requirements.
4. **MUST** refuse with `vtc/vetting/hidden/publish:otherCriterion` when hidden vetting is already on for a criterion other than `criterionId`. A community runs hidden vetting on at most one criterion: enrolment, the drip and the challenge are each served from a single configuration, so a second would accept proofs nobody can mint for.
5. Derives the helper and token verification keys from its own credential-signer secret. Repeating this step with the same secret **MUST** yield the same keys every time. When the criterion already stores keys and they differ from what is derived now, the community **MUST** refuse with `vtc/vetting/hidden/publish:signerChanged` rather than replace them, per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 8: replacing them would leave every enrolled vetter holding a credential that no longer verifies. The recovery is deliberate — [`vtc/vetting/hidden/withdraw`](../../withdraw/0.1/spec.md), then this task, which, with no stored keys to compare, starts a new key generation. Vetters then enrol again; a member already enrolled under a class label that is still live cannot enrol under it a second time ([`vtc/vetting/vetters/pcs-root`](../../../vetters/pcs-root/0.1/spec.md)), so that publish **SHOULD** name a new period.
6. Defaults `livePeriods` to `["<this month>"]` and `liveTokenLabels` to `["token/<this month>"]` when absent, `dripPerTick` to `3`, and `tickLength` to `P3D`. A `tickLength` **MUST** be at least `PT1H`. A configuration stored before `tickLength` existed is read as `P3D`.
7. Replaces the whole `events` list with what was sent, absent or empty removing every event — this is a replace, not a merge.
   - An event's `approvedBy` that is **new or changed** relative to the stored configuration **MUST** equal the publishing signer's principal — the signer, or the identity its delegated signing key acts for — else the community refuses with `vtc/vetting/hidden/publish:approverNotSigner`. An approver names themselves.
   - An `approvedBy` **unchanged** from the stored value **MAY** be re-sent by any administrator, so that a re-publish preserves the approvals already given. Read the stored value with [`vtc/vetting/hidden/show`](../../show/0.1/spec.md); the manifest does not carry it.
   - The community **MUST** refuse with `vtc/vetting/hidden/publish:approverInEvent` when an event's `approvedBy` names a member who has asked to vet at that event through [`vtc/vetting/vetters/event-mode`](../../../vetters/event-mode/0.1/spec.md), which forbids the requesting member being its own approver.
8. Stores the full configuration (`stored`) and republishes the criterion's manifest entry, then answers with `criterionId`, `stored`, `published` and the criterion's `requirementsDigest` **after** the change.
9. **MUST NOT** include `approvedBy` or `graceDays` in any event inside `published` — those are the community's own record and a vetter is told the effective close date when its request is approved, not handed the raw grace period here.

## Authorization

The entitlement is **administrator standing at the community**: the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — read at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). The decision, and the vetter-eligibility policy the derived keys serve, are the community's.

## Definitions

**Suite** — the blind-signature scheme in force (`ps-ddh-bls12381` at the time of writing). Implementation-defined; the framework imposes no vocabulary on it.

**Helper key (`hvk`/`helperKey`)**, **token key (`tvk`/`tokenKey`)** — the two verification keys the suite requires, derived from the community's credential-signer secret and never the same key as each other ([`vtc/vetting/vetters/pcs-root`](../../../vetters/pcs-root/0.1/spec.md) §5.1). Both are **public values**: `published` carries them under the wire-facing names `helperKey` and `tokenKey`, so that a client implementing the manifest's `vetting.ext` entry never has to read the underlying suite's paper to find the right member.

**Live period**, **live token label** — the class labels (`vetter/<period>`) and token labels a submission may currently be built or spent against. Stored as the bare `<period>` (e.g. `2026-09`) and published as the label whole (`vetter/2026-09`), so a client reads exactly what it should present without reconstructing the string itself.

**Tick length** — how long one tick of the drip lasts, an ISO 8601 duration in days and/or hours (`P3D`, `PT12H`, `P1DT12H`). Every vetter may draw `dripPerTick` tokens in each tick of a label; [`vtc/vetting/vetters/pcs-tokens`](../../../vetters/pcs-tokens/0.1/spec.md) defines where a label's ticks start and refuses one that has not begun, which is what turns the drip rate into a velocity cap. Published whole, so a vetter's client knows when to draw.

**Event** — a bounded, named gathering this community publishes a rate menu for ([`vtc/vetting/vetters/event-mode`](../../../vetters/event-mode/0.1/spec.md) §5.1). Not live merely by being listed here: it also needs an approver named in `approvedBy` and to fall inside its `startDate`/`endDate` window (plus `graceDays`), which this task never checks — those conditions are evaluated where tokens are drawn and spent, not where events are configured.

## Request

The administrator sends the request to the community; the payload is the top-level object in [`payload.schema.json`](payload.schema.json).

### Turn on hidden vetting for this month, with defaults

```json
{
  "id": "urn:uuid:3c4d5e6f-7081-4a3c-8d4e-5f6a7b8c9d01",
  "type": "https://trusttasks.org/spec/vtc/vetting/hidden/publish/0.1",
  "issuer": "did:web:admin.kernel-vtc.example",
  "recipient": "did:webvh:QmVtcScid:kernel-vtc.example",
  "issuedAt": "2026-09-30T09:10:00Z",
  "payload": {
    "criterionId": "vetted-member"
  }
}
```

### Open a three-day summit event on top of the standing labels

```json
{
  "id": "urn:uuid:3c4d5e6f-7081-4a3c-8d4e-5f6a7b8c9d03",
  "type": "https://trusttasks.org/spec/vtc/vetting/hidden/publish/0.1",
  "issuer": "did:web:admin.kernel-vtc.example",
  "recipient": "did:webvh:QmVtcScid:kernel-vtc.example",
  "issuedAt": "2026-09-30T09:15:00Z",
  "payload": {
    "criterionId": "vetted-member",
    "events": [
      {
        "eventId": "summit-2026",
        "startDate": "2026-10-01",
        "endDate": "2026-10-03",
        "groupFloor": 5,
        "tiers": [
          { "name": "desk", "dripPerTick": 5 },
          { "name": "busy-desk", "dripPerTick": 10 }
        ],
        "approvedBy": "did:web:approver.kernel-vtc.example"
      }
    ]
  }
}
```

## Response

The community answers with `criterionId`, `stored`, `published` and `requirementsDigest`, in the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`.

### Published, defaults filled in

```json
{
  "id": "urn:uuid:3c4d5e6f-7081-4a3c-8d4e-5f6a7b8c9d02",
  "type": "https://trusttasks.org/spec/vtc/vetting/hidden/publish/0.1#response",
  "threadId": "urn:uuid:3c4d5e6f-7081-4a3c-8d4e-5f6a7b8c9d01",
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "recipient": "did:web:admin.kernel-vtc.example",
  "issuedAt": "2026-09-30T09:10:01Z",
  "payload": {
    "criterionId": "vetted-member",
    "stored": {
      "suite": "ps-ddh-bls12381",
      "hvk": "z6MkhVerificationKey1abcXYZ",
      "tvk": "z6MktVerificationKey1defXYZ",
      "livePeriods": ["2026-09"],
      "liveTokenLabels": ["token/2026-09"],
      "dripPerTick": 3,
      "tickLength": "P3D",
      "events": []
    },
    "published": {
      "suite": "ps-ddh-bls12381",
      "helperKey": "z6MkhVerificationKey1abcXYZ",
      "tokenKey": "z6MktVerificationKey1defXYZ",
      "vetterLabels": ["vetter/2026-09"],
      "tokenLabels": ["token/2026-09"],
      "dripPerTick": 3,
      "tickLength": "P3D",
      "events": []
    },
    "requirementsDigest": "zQmRequirementsDigestAfter1"
  }
}
```

### Refused: the criterion asks for no vetting

```json
{
  "id": "urn:uuid:3c4d5e6f-7081-4a3c-8d4e-5f6a7b8c9d05",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:3c4d5e6f-7081-4a3c-8d4e-5f6a7b8c9d04",
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "recipient": "did:web:admin.kernel-vtc.example",
  "issuedAt": "2026-09-30T09:16:00Z",
  "payload": {
    "code": "vtc/vetting/hidden/publish:noVetting",
    "message": "criterion `open-registration` asks for no vetting, so there is nothing for hidden-vetting parameters to qualify — give it vetting requirements first",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request carries a criterion identifier, optional class and token labels, a drip rate, and optional event windows and rate menus — one of which, `approvedBy`, may name a member DID. The response carries the same shape back, plus the two verification keys the community derived. Neither key is the community's signing secret; both are meant to be read by every applicant and vetter once published, and the response merely confirms what `published` now says. A producer **MUST NOT** put an applicant identifier — there is no applicant at configuration time — into `ext`.

### Correlation

The administrator declares `identifierScope: any`, matching [`vtc/vetting/auto-grant/update`](../../../auto-grant/update/0.1/spec.md): the community resolves the signer to its own access-control entry, whichever scope the signer's identity uses. `approvedBy`, when present, names a member the community already administers and already resolves for [`vtc/vetting/vetters/event-mode`](../../../vetters/event-mode/0.1/spec.md)'s own approval step; this task adds no new handle for that member beyond what event-mode already establishes.

The community declares `identifierScope: public` for the reason every community-facing task in this family does: it is the one DID every administrator, vetter and applicant addresses, and the parameters this task stores and republishes are its own.

### Retention

Durable. The stored configuration is what the community mints hidden attestations against for as long as the labels it names stay live, and what `published` continues to serve from the manifest until the next publish. A community **SHOULD** record an audit entry for each publish, on the same terms as its other administrative changes: who published, which criterion, and the deltas — labels and events added or removed, approvals given or changed. It **MUST NOT** record a reason or free text beside it.

### Consent/purpose

The purpose is to let a community turn on, or adjust, hidden-vetter admission for one criterion under its own vetter-eligibility policy. Rotating live periods or token labels is a real, visible change to what a vetter or applicant may currently present; an administrator **SHOULD** expect an application already in progress under a retired label to fail, and time a rotation accordingly. Whether a community requires further sign-off before publishing — beyond the administrator standing this task already checks — is the community's own policy, not this specification's. The off switch is [`vtc/vetting/hidden/withdraw`](../../withdraw/0.1/spec.md), which removes these parameters and leaves the criterion's named vetting as it was.

### Threats

*Signer drift.* If the community's credential-signer secret changes between two publishes (key rotation, restore from an older backup), the keys this task derives change with it. A consumer **MUST** detect a stored configuration whose keys no longer match what it would derive now, and refuse with `vtc/vetting/hidden/publish:signerChanged` rather than publish keys nobody can verify against — see [`vtc/vetting/vetters/pcs-root`](../../../vetters/pcs-root/0.1/spec.md) for the enrolment this failure would otherwise silently break. Re-keying stays possible, but only as the explicit withdraw-then-publish of Conformance item 5.

*Silent label retirement.* Because `events` is replaced wholesale and labels default to the current month, an administrator who calls this task without reviewing its current `published` output first can unintentionally drop a label or event still in use. A community **SHOULD** show the administrator the effective configuration — read with [`vtc/vetting/hidden/show`](../../show/0.1/spec.md), not from the manifest, which omits every event's `approvedBy` and so would un-approve live events if republished as read — before it republishes.
