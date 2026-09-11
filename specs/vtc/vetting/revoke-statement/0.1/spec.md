---
slug: vtc/vetting/revoke-statement
version: "0.1"
title: VTC Vetting — Revoke Statement
summary: A vetter withdraws a Vetting Statement it issued by telling the community that relies on it. From then on the statement does not count, and the community records that it was withdrawn.
status: draft
targetFrameworkVersion: "0.5"
category: governance
keywords:
  - vtc
  - vetting
  - revocation
  - withdraw
  - statement
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: vetter
    requirement: REQUIRED
    member: issuer
    identifierScope: public
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  request: REQUIRED
  response: REQUIRED
  rationale: >-
    A notice permanently stops a statement counting, and the community acts on it
    only if it comes from the statement's own issuer — so the notice must be
    attributable to that DID on every transport, and retained as the record of who
    withdrew what. The response is the vetter's only evidence that the community
    recorded the withdrawal, and when.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A notice has a permanent effect on how the community counts evidence. Its issue time places the withdrawal against the decisions that relied on the statement, which is what a later review of those decisions needs.
sideEffects:
  level: destructive
  rationale: "Irreversible: once recorded, the withdrawn statement never counts again, for any application, and this task defines no way to withdraw a notice. What follows for a member already admitted on the statement is the community's policy, not this task's effect."
exposure:
  discloses: none
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries a statement's id, its digest and an optional closed reason code; the response is a timestamp. No claim about the applicant travels, and the applicant is not named."
retention:
  class: durable
  rationale: "The community keeps the notice for as long as the statement could still be submitted, so that a later submission does not count it, and — where the statement was relied on for an admission — for as long as it keeps that admission's record, which the withdrawal is now part of."
errorCodes:
  - code: vtc/vetting/revoke-statement:notMember
    meaning: The sender is not, and has never been, a member of this community, so cannot be the issuer of a statement that counts here.
    retryable: false
  - code: vtc/vetting/revoke-statement:issuerMismatch
    meaning: The community holds a statement with this id, and its issuer is not the sender. Only a statement's issuer can withdraw it.
    retryable: false
  - code: vtc/vetting/revoke-statement:digestMismatch
    meaning: The community holds a statement with this id whose digest differs from the one in the notice.
    retryable: false
related:
  - vetting/session
  - vtc/join-requests/submit
  - vtc/join-requests/manifest
---

## Abstract

A vetter can withdraw a Vetting Statement at any time. It may have made a mistake, learned something new, or found that the key it signed with is no longer only its own. The one party that relies on a statement is the community it was issued for, so the vetter tells that community directly. The community records the notice against the statement's `id` and digest. From then on the statement does not count: not in a pending application, and not in any later submission.

The notice is not a published status list. A list holding a handful of one vetter's statements would give no herd privacy, and a statement is scoped to one community anyway. If statements come to need checking by parties other than their community, a status mechanism can be added alongside this task; nothing here would change.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5 and may change in place while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **vetter** (`issuer`):

1. Sends the notice from the DID that issued the statement, to the community named in the statement's `endorsement.community`.
2. Sets `statementDigestMultibase` over the statement exactly as it issued it, `proof` included.
3. **SHOULD** tell the applicant that it withdrew the statement. Whether it also tells the applicant why is its own choice; this task does not carry the reason to the applicant.
4. **SHOULD** mark the statement withdrawn in its own records, so that it neither re-sends nor relies on it.

A conforming **community** (`recipient`):

1. Applies the [SPEC §7.2](/SPEC.md#72-consumer-requirements) pipeline, and refuses a notice from a DID that is not and has never been a member with `notMember`.
2. Where it already holds a statement with `statementId` — in a pending or decided application — refuses the notice with `issuerMismatch` if that statement's `issuer` is not the sender, or with `digestMismatch` if its digest differs.
3. Otherwise **MUST** record the notice, keyed by `statementId` and `statementDigestMultibase`, together with the sender, and return `recordedAt`. A notice for a statement the community has not yet seen is recorded all the same: it takes effect if and when that statement is submitted.
4. **MUST** treat a statement as not counting whenever it holds a notice whose `statementId`, digest and sender all match the statement's `id`, digest and `issuer`. A notice matching on some of these and not all has no effect on the statement.
5. **MUST** re-evaluate a pending application that relies on the withdrawn statement.
6. **MUST NOT** remove or suspend a member solely because a statement relied on at their admission was withdrawn. What follows for an admitted member — a review, a request for a replacement statement — is the community's published policy.
7. On a repeated notice for a statement it has already recorded, **MUST** return the original `recordedAt` and change nothing.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authority to withdraw a statement is **having issued it**. A notice has effect only on a statement whose `issuer` is the notice's sender and whose digest is the notice's digest (Conformance item 4). A party that is not the issuer can send a notice, and the community may record it, but it can never match the statement it names. That is why the community can accept notices for statements it has not seen yet without opening a way to withdraw other people's evidence.

The sender must also be, or have been, a member. Statements count only when issued by eligible members, and a vetter that has since left or been removed keeps the right to withdraw what it issued, which matters most in the key-compromise case.

A vetter needs no justification, and the community does not weigh one. The `reason` is information for the community's own review, not a condition of the withdrawal. Per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10, verifying the notice's `proof` establishes who sent it; the match against the statement's `issuer` is what establishes that they may withdraw it.

## Definitions

**Vetting Statement** — the `EndorsementCredential` a vetter issues after a [`vetting/session`](../../../../vetting/session/0.1/spec.md), carrying an identity-vetting endorsement. It has an `id`.

**Notice** — a request under this task. It names a statement; it never carries one.

**Statement digest** — the SHA-256 multihash, multibase-encoded, of the RFC 8785 canonicalization of the statement exactly as issued, `proof` included. It identifies one issued credential, and it differs from the statement's `id`, which anyone could reuse.

## Request

The vetter sends the notice to the community. See the top-level schema in [`payload.schema.json`](payload.schema.json).

### Withdrawing a statement after learning something new

The digest is the real value for the statement shown in [`vetting/session`](../../../../vetting/session/0.1/spec.md).

```json
{
  "id": "urn:uuid:2a4c6e8f-1b3d-4f5a-9c7e-0d2f4a6b8c01",
  "type": "https://trusttasks.org/spec/vtc/vetting/revoke-statement/0.1",
  "threadId": "urn:uuid:2a4c6e8f-1b3d-4f5a-9c7e-0d2f4a6b8c01",
  "issuer": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
  "recipient": "did:webvh:QmVtcScid:kernel-vtc.example",
  "issuedAt": "2026-10-02T11:30:00Z",
  "payload": {
    "statementId": "urn:uuid:7e5d3c1b-9f8a-4b6c-a2d1-e0f9a8b7c601",
    "statementDigestMultibase": "zQmeKtmNKfNz6njsiUEfa4WKzHMN3JRvu67dr6LyoQXAAJW",
    "reason": "new-information"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmCarolScid1:kernel-vtc.example:carol#key-1",
    "created": "2026-10-02T11:30:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z63jiSzsVJshBfyZwcr6nUopHo5M1QnBnWJHtwTpdNEFeD7KoX5rezJcGeoY8AVuTSo5Q3uH2KqMoEZk68qqGu3AR"
  }
}
```

## Response

The community, now responding, confirms it recorded the notice, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error` with this specification's codes.

### Recorded

```json
{
  "id": "urn:uuid:2a4c6e8f-1b3d-4f5a-9c7e-0d2f4a6b8c02",
  "type": "https://trusttasks.org/spec/vtc/vetting/revoke-statement/0.1#response",
  "threadId": "urn:uuid:2a4c6e8f-1b3d-4f5a-9c7e-0d2f4a6b8c01",
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "recipient": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
  "issuedAt": "2026-10-02T11:30:01Z",
  "payload": {
    "recordedAt": "2026-10-02T11:30:01Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid:kernel-vtc.example#key-1",
    "created": "2026-10-02T11:30:01Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z5k2pxtz3XrdADnsNJ1XQiZ4oj7XHVqTamscwe3Wir6JjjNKp6mJZZTmynBW32NBmWCxjs5g8Xmjck9ZLfPKtU5G4"
  }
}
```

## Security & Privacy

### Data carried

A statement id, a digest and, optionally, one of four reason codes. The notice does not name the applicant, and it does not carry the statement, so a community learns nothing about an applicant it has not already been sent. If the statement was never submitted, the community holds only an id and a digest it cannot link to anyone. There is no free-text member: a withdrawal explained in prose would be an unverifiable statement about a person, sent to the body deciding on them. A vetter with a concern about fraud raises it through the community's own process, not in a notice.

### Correlation

Both parties declare `identifierScope: public`. The vetter's DID is its member DID in the community: a notice has effect only when its sender matches the statement's issuer, and that issuer is a member DID by construction. The community's DID is the one named in the statement. A pairwise identifier on either side would leave a notice unable to meet the statement it names.

A notice received before submission tells the community that this vetter withdrew *something*. When the statement later arrives, the community learns which applicant it concerned. That linkage is the purpose of the task. The community already learns from the statement itself that this vetter vetted this applicant.

### Retention

The community keeps each notice for as long as the statement it names could still be submitted and count — its own validity period, or the community's `maxStatementAge` where that is shorter. Where the statement was relied on for an admission, the notice becomes part of that admission's record and is kept with it. Keeping notices any longer than that would build a history of withdrawals with no remaining purpose.

The vetter keeps the response as its evidence of when the community recorded the withdrawal.

### Consent/purpose

The purpose is to stop a statement from counting, and the community's use of a notice is limited to that and to reviewing decisions the statement was part of. Using notices to rate vetters, or to profile the applicants they concern beyond that review, is a purpose the vetter did not address. A key-compromise notice is also a signal about the vetter's own key, and the community **SHOULD** treat it as one — for instance by reviewing other statements that key signed — within its published policy. Whether a vetter's agent requires a step-up before it sends a notice is that agent's policy. Per [SPEC §7.3](/SPEC.md#73-specification-requirements) item 13, this specification does not decide it.
