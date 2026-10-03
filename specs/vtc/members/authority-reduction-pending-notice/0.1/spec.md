---
slug: vtc/members/authority-reduction-pending-notice
version: "0.1"
title: VTC Members — Authority-Reduction-Pending Notice
summary: A community tells a subject that a reduction of their administrative authority has been requested and is cooling off — by whom, why, and when it lands unless its requester cancels it — so the subject learns of it before it takes effect.
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - vtc
  - acl
  - notice
  - revocation
  - demotion
  - cooling-off
parties:
  - role: community maintainer
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: subject whose authority the pending reduction would reduce
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The subject may need to show a third party — another administrator, an auditor, a governance body — that a reduction of their authority was requested, by whom and when, and that they were told before it landed. Without a proof the notice evidences nothing beyond the transport that carried it.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: The notice's value is that it arrived before `landsAt`. A document that cannot be placed in time cannot show that, and cannot be compared against the audit trail of the action it reports.
sideEffects:
  level: none
  rationale: Reports a reduction already parked; the notice changes nothing at the recipient and nothing at the community. The reduction lands, or is cancelled, whether or not the notice is delivered.
exposure:
  discloses: metadata
  actsAsSubject: false
  ingests: metadata
  rationale: Discloses the subject's own pending change of role, the requesting administrator's DID, the requester's stated reason and the action's identifier to the subject only.
retention:
  class: durable
  rationale: The subject's evidence that a reduction nobody else reviewed was requested and announced to them before it took effect, kept for as long as they might contest it.
errorCodes: []
related:
  - vtc/members/authority-reduced-notice
  - vtc/members/removal-notice
  - vtc/admin/actions/show
  - vtc/admin/actions/cancel
  - acl/revoke
  - acl/change-role
---

## Abstract

The **VTC Members — Authority-Reduction-Pending Notice** Trust Task is how a
community tells a subject that an administrator has asked to reduce their
administrative authority — revoke their access control entry, demote their role,
or narrow what it allows — and that the reduction is **cooling off**: it will
take effect at `landsAt` unless the administrator who requested it cancels it
first.

The specification of the Verifiable Trust Infrastructure requires a community to
obtain the consent of a party other than both the requester and the subject
before reducing another subject's unrestricted authority, and, where no such
party exists, to notify the subject (VTI-APV-019). A community with no such
party parks the reduction as a `coolingOff` action
([`vtc/admin/actions/_shared/0.2`](../../../admin/actions/_shared/0.2/action.schema.json))
rather than applying it at once. This task tells the subject when it is parked;
[`vtc/members/authority-reduced-notice/0.1`](../../authority-reduced-notice/0.1/spec.md)
tells them, with `agreement: unopposed`, when it lands.

## The subject is told, not asked

The subject **cannot block** the reduction. The notice carries no veto, no
objection handle and no challenge, and the cooling-off action names nobody but
its requester in `cancellableBy`. That is deliberate. The reductions that most
need to land are the ones made because the subject's key or judgement can no
longer be trusted — a reported key compromise, a rogue administrator — and in
exactly those cases a veto held by the subject is held by the party the
reduction is meant to stop. A subject able to refuse its own removal would make a
compromised subject irremovable.

What the subject gets instead is time and knowledge: they learn of the
reduction, who asked for it and why, before it takes effect. A subject who
believes it is a mistake can raise it with the requester — the only party who
can cancel — or with the community's governance, through whatever channel that
community keeps; and a reduction requested by a compromised *requester* is
visible to its target for the whole cooling-off period instead of discovered
after the fact.

## Status of this Document

This is a **draft** *Trust Task specification* per [SPEC.md §5.3](/SPEC.md#53-maturity-levels); the schema **MAY** change without notice. Feedback via the [issue tracker](https://github.com/trustoverip/dtgwg-trust-tasks-tf/issues).

## Conformance

[[RFC2119]](https://www.rfc-editor.org/rfc/rfc2119) and [[RFC8174]](https://www.rfc-editor.org/rfc/rfc8174) key-word conventions apply.

A conforming **producer** (the community) **MUST**:

1. Emit a *Trust Task document* whose `type` is `https://trusttasks.org/spec/vtc/members/authority-reduction-pending-notice/0.1`, with itself as `issuer` and the subject as `recipient`, carrying a `proof` and an `issuedAt`.
2. Send it when the reduction is parked as a `coolingOff` action, and not for a reduction that takes effect at once or that awaits another administrator's consent — those are reported, when they land, by [`authority-reduced-notice`](../../authority-reduced-notice/0.1/spec.md) alone.
3. Set `landsAt` and `actionId` to the parked action's `landsAt` and `actionId`, and `requestedAt` to the action's `createdAt`.
4. Omit `resultingRole` exactly when `code` is `revoked`, and include it otherwise.
5. Show the parked action to the subject, with `callerRole: subject`, on [`vtc/admin/actions/show/0.2`](../../../admin/actions/show/0.2/spec.md) for as long as the subject keeps the standing to call it.
6. When the reduction lands, send [`authority-reduced-notice`](../../authority-reduced-notice/0.1/spec.md) with `agreement: unopposed`; when its requester cancels it, send no authority-reduced notice.

A conforming **producer** **MUST NOT** make the reduction's landing conditional on delivery of this notice. Delivery is best-effort over an asynchronous transport, and a reduction that waited on it would hand the subject a veto by unreachability.

A conforming **consumer** (the subject) **MUST** verify the `proof` and that the `issuer` is the community that holds the entry before relying on the notice, and **MUST** treat the payload `did` as authoritative over transport addressing.

## Authorization

The notice is informational and confers nothing: it neither grants nor removes
authority, gives the subject no authority over the pending reduction, and
receiving one is not evidence of standing at the community. The community's
authority to send it is its authority over its own access control; the subject's
entitlement to it is being the party the pending reduction is about. The only
authority over the reduction itself is the requester's, exercised through
[`vtc/admin/actions/cancel/0.2`](../../../admin/actions/cancel/0.2/spec.md).

## Definitions

- **Cooling-off action** — a parked administrative action of category `coolingOff`: an operation with no eligible third party to consent, which lands at `landsAt` unless its requester cancels it first.
- **`decidedBy`** — the administrator who requested the reduction. Named as in `authority-reduced-notice`, so that the pair report the same party the same way.
- **`requestedAt`** / **`landsAt`** — when the reduction was parked, and when it takes effect unless cancelled.

## Request

The notice is one-way: there is no response document, and the payload schema
carries no `$defs.Response`.

### A revocation cooling off in a two-administrator community

```json
{
  "id": "urn:uuid:6b3f1d8a-2c5e-4a97-b0d4-8e1f3a5c7d92",
  "type": "https://trusttasks.org/spec/vtc/members/authority-reduction-pending-notice/0.1",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkFormerCoAdmin",
  "issuedAt": "2026-10-02T09:00:04Z",
  "payload": {
    "did": "did:key:z6MkFormerCoAdmin",
    "code": "revoked",
    "previousRole": "community-admin",
    "decidedBy": "did:key:z6MkFoundingAdmin",
    "requestedAt": "2026-10-02T09:00:00Z",
    "landsAt": "2026-10-03T09:00:00Z",
    "actionId": "act_Cf9xT2kWq7Ln4Rv8",
    "reason": "Signing key reported compromised."
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-02T09:00:04Z",
    "verificationMethod": "did:web:community.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "zEiPBT7CkXy8xobWYWxPWrwqb2t1ih2SXMRV2ZckBRJxvAV3gqrETyYSU5xboBAueZ2UXrted4ZGUD9BrH5KzGoh"
  }
}
```

### A demotion with no stated reason

```json
{
  "id": "urn:uuid:9e2a4c6b-8d1f-4e3a-a5b7-0c9d2e4f6a81",
  "type": "https://trusttasks.org/spec/vtc/members/authority-reduction-pending-notice/0.1",
  "issuer": "did:web:community.example",
  "recipient": "did:key:z6MkRepoManager",
  "issuedAt": "2026-10-02T10:00:03Z",
  "payload": {
    "did": "did:key:z6MkRepoManager",
    "code": "demoted",
    "previousRole": "community-admin",
    "resultingRole": "member",
    "decidedBy": "did:key:z6MkCommunityAdmin",
    "requestedAt": "2026-10-02T10:00:00Z",
    "landsAt": "2026-10-03T10:00:00Z",
    "actionId": "act_Hn5vK8pQz2Wm7Ty4"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "created": "2026-10-02T10:00:03Z",
    "verificationMethod": "did:web:community.example#key-1",
    "proofPurpose": "authentication",
    "proofValue": "zKWVEdnti3xhkBNHXpFDv9c1Sbdb3SaVCLRRMJTsEZwrbtERZrLX8SAT39WGVF1EpMD9bLJ7FExH5gEsFu16Y2sZ"
  }
}
```

## Security & Privacy

### Data carried

The subject's own DID, the role they hold and would hold, the requesting
administrator's DID, an operator-authored reason, the two instants that bound
the cooling-off period, and the action's identifier. `reason` is free text that
reaches the subject verbatim: a consumer **MUST NOT** render it as markup, and a
producer **MUST NOT** put anything in it, or in `ext`, about any other subject.
The notice carries nothing the subject could use to act on the reduction — no
challenge, no decision handle — because the subject holds no authority over it.

### Correlation

The notice names the requesting administrator to the subject, which is the
point: a pending reduction whose requester is unnamed cannot be raised with the
one party who can cancel it. `actionId` joins the notice to the cooling-off
action and, later, to the `authority-reduced-notice` that reports its landing;
that join is intended. The notice discloses nothing about other subjects or the
community's other decisions. A subject receiving several notices can correlate
them by `issuer`, as they could by their own membership.

The community's identifier is **public** on purpose (`identifierScope: public`):
the notice is evidence the subject may show others, and evidence is only
checkable against an issuer a third party can recognise and resolve. A pairwise
community identifier would make the notice verifiable to nobody but its
recipient, which defeats the reason it carries a proof.

### Retention

The subject is expected to keep the notice for as long as they might contest
the reduction — with the `authority-reduced-notice` that follows it if it lands,
the pair show that the subject was told before it took effect. The community
need not retain the sent notice beyond its own audit record of the action, which
already records the request and its landing time.

### Consent/purpose

The notice exists to tell the subject, ahead of time, about a decision
concerning them and to give them verifiable evidence of it. It is not a request
for the subject's consent and must not be read as one: the reduction lands or is
cancelled independently of anything the subject does. It obliges the subject to
nothing. Whether a reduction is parked for cooling off, and for how long, is
decided by the community's own rules; this task only reports that it was.
