---
slug: vtc/vetting/vetters/grant
version: "0.1"
title: VTC Vetting — Grant Vetter Role
summary: A community administrator makes a member a vetter. The community issues the member a revocable CommunityRole endorsement credential for the vetter role, and a member who already holds a live grant gets it back unchanged.
status: draft
targetFrameworkVersion: "0.5"
category: governance
keywords:
  - vtc
  - vetting
  - vetter
  - role
  - endorsement
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: community administrator
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: A grant decides whose identity statements a community will count toward admitting someone. It must be attributable to the administrator who made it, on every transport, and survive as the record an audit — or a cascade review after a vetter is found to have vetted badly — reads back.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: A grant replayed after the member's vetter role was revoked would make them a vetter again. Placing the instruction in a window is what lets the community refuse the replay rather than re-grant.
sideEffects:
  level: mutating
  rationale: "Issues a role credential attributable to the community and consumes a slot on its revocation status list. Recoverable: the grant is revoked with vtc/endorsements/revoke."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries one member DID and a validity period. The response returns identifiers and validity dates for the grant — not the credential, which is delivered to the member alone over credential-exchange/issue."
retention:
  class: durable
  rationale: "The community keeps the endorsement record and its status-list slot for the life of the credential and beyond it: whether a vetter held the role when they issued a statement is checked when an application is decided, and again in any later review of the members that vetter vetted."
errorCodes:
  - code: vtc/vetting/vetters/grant:notMember
    meaning: "`memberDid` is not an active member of this community. Only members can be vetters."
    retryable: false
related:
  - vtc/endorsements/revoke
  - vtc/members/admin-remove
  - credential-exchange/issue
  - vetting/request
  - vtc/join-requests/manifest
---

## Abstract

A community that admits people on peer identity vetting counts statements only from the members it has made **vetters**. Eligibility is a credential: the community issues the member a **role credential**. That is an `EndorsementCredential` whose `credentialSubject.endorsement` is `{ "type": "CommunityRole", "role": "vetter", "communityDid": … }`, and it carries a revocation status entry. The vetter presents it to applicants in [`vetting/request`](../../../../../vetting/request/0.1/spec.md), so an applicant can check eligibility before arranging a session. The community checks it again when it decides, against its own records.

This task is how an administrator grants that role. The credential is revoked with [`vtc/endorsements/revoke`](../../../../endorsements/revoke/0.1/spec.md), and removing a member revokes it too. A community's manifest names the role it counts as `vetting.eligibleVetters.role`.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.5 and may change in place while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming **administrator** (`issuer`) names the member in `memberDid` and **MAY** set `validitySeconds`.

A conforming **community** (`recipient`):

1. Applies the [SPEC §7.2](/SPEC.md#72-consumer-requirements) pipeline, and refuses a sender without the community-administrator capability with the framework's `permissionDenied`.
2. Refuses a `memberDid` that is not an active member with `vtc/vetting/vetters/grant:notMember`.
3. Where the member already holds a **live grant** — a vetter role credential issued by this community that is neither expired nor revoked — **MUST** return that grant's `endorsementId`, `credentialId`, `validFrom` and `validUntil` unchanged, issue nothing, and ignore `validitySeconds`. Repeated execution is therefore safe and intended ([SPEC §7.2](/SPEC.md#72-consumer-requirements) item 11): a second grant is indistinguishable from the first.
4. Otherwise **MUST** issue a role credential and record it as an endorsement. The credential is an `EndorsementCredential` with:
   - `issuer` set to the community DID and `credentialSubject.id` set to `memberDid`;
   - `credentialSubject.endorsement` exactly `{ "type": "CommunityRole", "role": "vetter", "communityDid": <community DID> }`;
   - a `credentialStatus` entry for revocation on the community's published status list;
   - `validFrom` at issuance, and `validUntil` equal to `validFrom` plus `validitySeconds`, or 365 days where it is absent;
   - an `id`, returned as `credentialId`.

   `CommunityRole` is the community's reserved endorsement type. It is not registered through `vtc/endorsement-types/register`, which refuses it.
5. **MUST** deliver the credential to the member over [`credential-exchange/issue/0.1`](../../../../../credential-exchange/issue/0.1/spec.md). A failed delivery does not undo the grant. The community **MAY** deliver the same credential again when the grant is repeated.
6. **MUST** treat a grant as ended once its credential is revoked through [`vtc/endorsements/revoke/0.1`](../../../../endorsements/revoke/0.1/spec.md) with `endorsementId`, or expires. When a member is removed or leaves, the community **MUST** revoke every live vetter grant that member holds.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

The authority this task presupposes is the **community-administrator capability**. The issuer capability that suffices for [`vtc/endorsements/issue`](../../../../endorsements/issue/0.1/spec.md) does not: deciding whose statements count toward admission is a governance decision, not an issuance chore. No member can grant the role to themselves, and holding the role confers no authority to grant it to anyone else.

What the grant confers is narrow. Its holder's vetting statements are eligible to count, under the rest of the community's criteria. It does not authorize the holder to do anything at the community's service. Per [SPEC §7.2](/SPEC.md#72-consumer-requirements) item 10, verifying the request's `proof` establishes which administrator asked, never that they hold the capability; that is checked against the community's own access control.

## Definitions

**Vetter role credential** — the `EndorsementCredential` this task issues, whose endorsement is `{ type: "CommunityRole", role: "vetter", communityDid }`.

**Live grant** — a vetter role credential issued by this community to the member that has not expired and has not been revoked.

## Request

The administrator sends the request to the community. See the top-level schema in [`payload.schema.json`](payload.schema.json).

### Granting the vetter role for a year

```json
{
  "id": "urn:uuid:8d1f3a5c-7e9b-4c2d-a4f6-1b3d5e7f9a01",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/grant/0.1",
  "threadId": "urn:uuid:8d1f3a5c-7e9b-4c2d-a4f6-1b3d5e7f9a01",
  "issuer": "did:webvh:QmDanaScid1:kernel-vtc.example:dana",
  "recipient": "did:webvh:QmVtcScid:kernel-vtc.example",
  "issuedAt": "2026-09-13T10:00:00Z",
  "payload": {
    "memberDid": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
    "validitySeconds": 31536000
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmDanaScid1:kernel-vtc.example:dana#key-1",
    "created": "2026-09-13T10:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z2RA8945kouBqzqifZqkbB8ZSrj1sfVLZPvr6wz4RvHSaYqXySHQoep9vM1fRYit6tNfmaTDThA2ibMPhBMFh8w3N"
  }
}
```

## Response

The community, now responding, returns the grant, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). A repeated grant returns the same values. Refusals use `trust-task-error` with `permissionDenied` or this specification's code.

### The grant

```json
{
  "id": "urn:uuid:8d1f3a5c-7e9b-4c2d-a4f6-1b3d5e7f9a02",
  "type": "https://trusttasks.org/spec/vtc/vetting/vetters/grant/0.1#response",
  "threadId": "urn:uuid:8d1f3a5c-7e9b-4c2d-a4f6-1b3d5e7f9a01",
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "recipient": "did:webvh:QmDanaScid1:kernel-vtc.example:dana",
  "issuedAt": "2026-09-13T10:00:01Z",
  "payload": {
    "endorsementId": "end-7Kq2mX9p",
    "credentialId": "urn:uuid:5c7e9a1b-3d5f-4b7c-9e1a-2c4e6a8b0d01",
    "validFrom": "2026-09-13T10:00:01Z",
    "validUntil": "2027-09-13T10:00:01Z"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid:kernel-vtc.example#key-1",
    "created": "2026-09-13T10:00:01Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z5k2pxtz3XrdADnsNJ1XQiZ4oj7XHVqTamscwe3Wir6JjjNKp6mJZZTmynBW32NBmWCxjs5g8Xmjck9ZLfPKtU5G4"
  }
}
```

### The role credential delivered to the member

Carol receives this over `credential-exchange/issue/0.1`. It is the credential she later presents in `vetting/request`'s `eligibilityVp`. The status list URL is illustrative.

```json
{
  "@context": ["https://www.w3.org/ns/credentials/v2", "https://firstperson.network/credentials/dtg/v1"],
  "id": "urn:uuid:5c7e9a1b-3d5f-4b7c-9e1a-2c4e6a8b0d01",
  "type": ["VerifiableCredential", "DTGCredential", "EndorsementCredential"],
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "validFrom": "2026-09-13T10:00:01Z",
  "validUntil": "2027-09-13T10:00:01Z",
  "credentialSubject": {
    "id": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
    "endorsement": {
      "type": "CommunityRole",
      "role": "vetter",
      "communityDid": "did:webvh:QmVtcScid:kernel-vtc.example"
    }
  },
  "credentialStatus": {
    "id": "https://kernel-vtc.example/status/revocation/1#4213",
    "type": "BitstringStatusListEntry",
    "statusPurpose": "revocation",
    "statusListIndex": "4213",
    "statusListCredential": "https://kernel-vtc.example/status/revocation/1"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid:kernel-vtc.example#key-1",
    "created": "2026-09-13T10:00:01Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z63jiSzsVJshBfyZwcr6nUopHo5M1QnBnWJHtwTpdNEFeD7KoX5rezJcGeoY8AVuTSo5Q3uH2KqMoEZk68qqGu3AR"
  }
}
```

## Security & Privacy

### Data carried

The request names one member and a validity period. The response returns identifiers and dates, never the credential. The credential goes to the member alone, over `credential-exchange/issue/0.1`, so a grant does not copy role material to whichever administrator or tool issued it. The credential itself says only that the member holds the `vetter` role in this community. It carries no claim about the member beyond that, and no reason for the grant.

### Correlation

The community declares `identifierScope: public`. The role credential is worth something only because an applicant who has never met the community can verify that its issuer is the community named in the manifest and in the credential's `communityDid`. A pairwise community identifier would make the credential unverifiable to exactly the applicants it is shown to.

The administrator declares `identifierScope: pairwise`, because only this community needs to recognise it, against its own access control.

The credential links the vetter's member DID to the vetter role in this community, for every applicant the vetter presents it to. That is its purpose: vetters are reached one applicant at a time, and each applicant needs to check eligibility before investing in a session. The status list is shared across the community's endorsements, so checking revocation does not reveal which vetter is being checked.

### Retention

Durable. The community keeps the endorsement record, the status-list slot, and the audit record of who granted the role and when. Whether a statement counts depends on its issuer having held the role when it was issued. A later review of the members a vetter vetted depends on that history too. Deleting it would leave the community unable to explain decisions it already made. The member keeps the credential for as long as it is valid; once it is expired or revoked it has no remaining use.

### Consent/purpose

The purpose is to make a member eligible to vet for this community, and the credential is presented only for that. A community **MUST NOT** use grants for anything else, such as a public list of vetters. Which members are vetters is disclosed by each vetter, to the applicants they choose, not by the community. Whether a member must agree before being made a vetter, and whether an administrator needs a step-up to grant the role, are the community's policy and the administrator's agent's. Per [SPEC §7.3](/SPEC.md#73-specification-requirements) item 13, this specification does not decide them.
