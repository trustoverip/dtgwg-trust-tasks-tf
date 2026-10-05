---
slug: vtc/vetting/hidden/withdraw
version: "0.1"
title: "VTC Vetting — Hidden — Withdraw"
summary: An administrator turns hidden-vetter admission off for one criterion, leaving its named vetting unchanged, and reads back the criterion's requirements digest after the change.
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
    This is the switch that stops a criterion accepting hidden proofs, and it can fail every application a vetter or applicant is building against the published parameters. Only administrator standing may throw it, so the signer must be attributable on every transport.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: >-
    Consequential, so SPEC §7.3 item 17 sets REQUIRED as the floor, and §7.2 item 11 can only absorb a duplicate inside a bounded window.
sideEffects:
  level: mutating
  rationale: >-
    Removes the criterion's stored hidden-vetting parameters and republishes its join-manifest entry without them, which moves the criterion's requirementsDigest. Reversible: a later vtc/vetting/hidden/publish derives the same keys from the same signer secret and restores the parameters. Withdrawing from a criterion that carries none changes nothing.
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: >-
    A criterion identifier in; whether anything was withdrawn and the criterion's digest out. No key, label or member identifier crosses the wire in either direction.
retention:
  class: durable
  rationale: >-
    The criterion stays without hidden-vetting parameters until the next publish. The change is audited against the administrator who made it. Vetter enrolment records and the spent-token ledger are not touched by this task.
errorCodes:
  - code: vtc/vetting/hidden/withdraw:noSuchCriterion
    meaning: "`criterionId` does not name an Accepts criterion this community has stored."
    retryable: false
related:
  - vtc/vetting/hidden/publish
  - vtc/vetting/vetters/pcs-root
  - vtc/vetting/vetters/pcs-tokens
  - vtc/vetting/pcs-challenge
  - vtc/schemas/accepts/register
---

## Abstract

[`vtc/vetting/hidden/publish`](../../publish/0.1/spec.md) turns a criterion's named vetting into one that also accepts a hidden proof. **VTC Vetting — Hidden — Withdraw** is its off switch: the one call that removes the criterion's stored hidden-vetting parameters and republishes its manifest entry without them, so from then on only named vetting statements count towards that criterion.

Nothing else about the criterion changes. Its vetting requirements — the statement type, how many statements, which methods count, who may vet — stay exactly as they were. Withdrawal is reversible: publishing again derives the same keys from the same signer secret, so the community can turn hidden vetting back on without anyone re-enrolling for a label that is still live.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **administrator** (`issuer`) names the criterion in `criterionId`. There is nothing else to send.

A conforming **community** (`recipient`):

1. Applies the [SPEC §7.2](/SPEC.md#72-consumer-requirements) pipeline and **MUST** refuse a caller without administrator standing with `permissionDenied`.
2. **MUST** refuse with `vtc/vetting/hidden/withdraw:noSuchCriterion` when `criterionId` names no stored criterion.
3. When the criterion carries hidden-vetting parameters, **MUST** remove them, republish the criterion's manifest entry without its hidden-vetting `vetting.ext` entry, and answer `withdrawn: true` with the criterion's `requirementsDigest` **after** the change.
4. When the criterion carries no hidden-vetting parameters, **MUST** answer with success, `withdrawn: false` and the criterion's current `requirementsDigest`, and **MUST NOT** refuse. Withdrawal is idempotent: an administrator who sends it twice, or sends it to a criterion that never had hidden vetting, has got what they asked for.
5. **MUST NOT** change the criterion's vetting requirements, admission mode, credential query or anything else this task does not name.
6. **MUST NOT** accept a hidden proof towards the criterion once the response has been sent. A submission whose proof was built against the withdrawn parameters is decided on its named statements alone.

## Authorization

The entitlement is **administrator standing at the community**: the signer's access-control entry — or, for a delegated signing key ([`auth/signing-key/enroll`](../../../../../auth/signing-key/enroll/0.1/spec.md)), the entry of the identity it acts for — read at execution time. Verifying the proof establishes who asked, never that they may ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10). This is the same standing [`vtc/vetting/hidden/publish`](../../publish/0.1/spec.md) requires: the administrator who may turn hidden vetting on may turn it off.

## Definitions

**Hidden-vetting parameters** — what [`vtc/vetting/hidden/publish`](../../publish/0.1/spec.md) stores against a criterion and publishes in its join-manifest entry: the suite, the helper and token verification keys, the live class and token labels, the drip rate and any events.

**Withdrawn** — the criterion carried hidden-vetting parameters when this task ran, and no longer does.

## Request

The administrator sends the request to the community; the payload is the top-level object in [`payload.schema.json`](payload.schema.json).

### Turn hidden vetting off for one criterion

```json
{
  "id": "urn:uuid:4d5e6f70-8192-4a3c-8d4e-5f6a7b8c9d11",
  "type": "https://trusttasks.org/spec/vtc/vetting/hidden/withdraw/0.1",
  "issuer": "did:web:admin.kernel-vtc.example",
  "recipient": "did:webvh:QmVtcScid:kernel-vtc.example",
  "issuedAt": "2026-10-05T09:10:00Z",
  "payload": {
    "criterionId": "vetted-member"
  }
}
```

## Response

The community answers with `criterionId`, `withdrawn` and `requirementsDigest`, in the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`.

### Withdrawn

```json
{
  "id": "urn:uuid:4d5e6f70-8192-4a3c-8d4e-5f6a7b8c9d12",
  "type": "https://trusttasks.org/spec/vtc/vetting/hidden/withdraw/0.1#response",
  "threadId": "urn:uuid:4d5e6f70-8192-4a3c-8d4e-5f6a7b8c9d11",
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "recipient": "did:web:admin.kernel-vtc.example",
  "issuedAt": "2026-10-05T09:10:01Z",
  "payload": {
    "criterionId": "vetted-member",
    "withdrawn": true,
    "requirementsDigest": "zQmRequirementsDigestAfterWithdraw"
  }
}
```

### Nothing to withdraw

The criterion never accepted a hidden proof, or it was already withdrawn. Success, not a refusal.

```json
{
  "id": "urn:uuid:4d5e6f70-8192-4a3c-8d4e-5f6a7b8c9d14",
  "type": "https://trusttasks.org/spec/vtc/vetting/hidden/withdraw/0.1#response",
  "threadId": "urn:uuid:4d5e6f70-8192-4a3c-8d4e-5f6a7b8c9d13",
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "recipient": "did:web:admin.kernel-vtc.example",
  "issuedAt": "2026-10-05T09:12:01Z",
  "payload": {
    "criterionId": "open-registration",
    "withdrawn": false,
    "requirementsDigest": "zQmRequirementsDigestUnchanged"
  }
}
```

### Refused: no such criterion

```json
{
  "id": "urn:uuid:4d5e6f70-8192-4a3c-8d4e-5f6a7b8c9d16",
  "type": "https://trusttasks.org/spec/trust-task-error/0.5",
  "threadId": "urn:uuid:4d5e6f70-8192-4a3c-8d4e-5f6a7b8c9d15",
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "recipient": "did:web:admin.kernel-vtc.example",
  "issuedAt": "2026-10-05T09:14:00Z",
  "payload": {
    "code": "vtc/vetting/hidden/withdraw:noSuchCriterion",
    "message": "no Accepts criterion `vetted-membr` is stored at this community",
    "retryable": false
  }
}
```

## Security & Privacy

### Data carried

The request carries a criterion identifier. The response carries the same identifier, a boolean and the criterion's digest. No verification key, label, event or member identifier crosses the wire in either direction. A producer **MUST NOT** put an applicant or vetter identifier into `ext`.

### Correlation

The administrator declares `identifierScope: any`, matching [`vtc/vetting/hidden/publish`](../../publish/0.1/spec.md): the community resolves the signer to its own access-control entry, whichever scope the signer's identity uses.

The community declares `identifierScope: public` for the reason every community-facing task in this family does: it is the one DID every administrator, vetter and applicant addresses, and the criterion this task changes is its own.

### Retention

Durable. The criterion stays without hidden-vetting parameters until the next publish. A community **SHOULD** record an audit entry for each withdrawal, on the same terms as its other administrative changes: who withdrew, from which criterion, and which labels and events were in force. It **MUST NOT** record a reason or free text beside it.

This task does **not** delete vetter enrolment records ([`vtc/vetting/vetters/pcs-root`](../../../vetters/pcs-root/0.1/spec.md)) or the spent-token ledger. What a community keeps of those, and for how long, is its own retention policy, and withdrawing hidden vetting neither requires nor implies erasing them. Withdrawal unmasks nothing: no record the community holds after it identifies a hidden vetter that it could not identify before.

### Consent/purpose

The purpose is to let a community stop accepting hidden proofs for one criterion — because it no longer wants hidden vetting, because a label or key is suspected compromised, or before changing the criterion in a way hidden vetting should not survive. An application already in progress whose proof was built against the withdrawn parameters will no longer count those hidden attestations, and fails unless its named statements alone meet the criterion. An administrator **SHOULD** time a withdrawal accordingly, and a community **SHOULD** show the administrator, before it withdraws, that applications may be relying on the parameters.

### Threats

*Silent re-enablement.* Publishing again derives the same keys from the same signer secret, so vetters already enrolled under a label that is still live hold credentials that become usable again the moment hidden vetting is republished. That is the intended reversibility. An administrator withdrawing because a vetter credential is suspected compromised **SHOULD** republish with labels the compromised credential was never issued under, rather than with the defaults, before turning hidden vetting back on.

*Withdrawal as suppression.* An administrator could withdraw hidden vetting to force applicants to name their vetters. The community's audit record of the withdrawal, and the requirementsDigest change every applicant sees, are what make that visible; whether a second administrator must agree to it is the community's own policy, not this specification's.
