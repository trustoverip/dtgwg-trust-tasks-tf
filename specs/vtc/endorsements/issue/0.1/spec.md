---
slug: vtc/endorsements/issue
version: "0.1"
title: VTC Endorsements — Issue
summary: A community issues a statement credential under a registered predicate — or, under the reserved typeUri IdentityVerificationCredential, its identity-verification credential — to a subject, with a published status-list slot for revocation.
status: draft
targetFrameworkVersion: "0.5.0"
category: credentials
keywords:
  - vtc
  - endorsements
  - credentials
  - issuance
  - verifiable-credential
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: community issuer
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: An issuance instruction mints a credential a third party will rely on; it is replayed by an auditor and corroborates the resulting credential's provenance, so transport-independent integrity is required.
issuedAtRequirement:
  requirement: REQUIRED
  rationale: An endorsement is a statement about a member that others rely on, and issuance returns material that leaves the community's control. A replayed issuance re-endorses a member whose standing may have changed since.
sideEffects:
  level: mutating
  rationale: "Mints a signed statement credential and consumes a status-list slot; revocable, but the slot is not reclaimed."
consequences:
  - "Issues a credential attributable to the community; valid until expiry or revocation."
  - "Permanently consumes one slot on the community's shared Revocation status list."
subjectPath: /subjectDid
exposure:
  discloses: secret
  actsAsSubject: false
  rationale: >-
    The response carries `credential` — the signed statement credential
    just minted, returned here and nowhere else, since reads carry only
    the `endorsement.issued` reference. The endorsed party retains it and
    presents it to relying parties, which is the same disclosure
    `vtc/invitations/issue` already declares `secret` for the Invitation
    Credential it returns.
errorCodes:
  - code: vtc/endorsements/issue:typeNotRegistered
    meaning: "`typeUri` is not a predicate registered in this community's endorsement-type registry."
    retryable: false
  - code: vtc/endorsements/issue:predicateNotIssuable
    meaning: "`typeUri` is registered, but its predicate's profile does not let the community issue it through this task — it requires `taskContext`, or names an issuer other than the community."
    retryable: false
  - code: vtc/endorsements/issue:claimSchemaViolation
    meaning: "`claim` failed validation against the registered predicate's declared claimSchema."
    retryable: false
  - code: vtc/endorsements/issue:claimTooLarge
    meaning: "`claim` exceeds the 8 KiB serialised cap."
    retryable: false
  - code: vtc/endorsements/issue:statusListExhausted
    meaning: The community's status list has no free slot; an operator must provision a new list.
    retryable: true
---

## Abstract

The **VTC Endorsements — Issue** Trust Task mints a **Verifiable Statement
Credential** (VSC) — the community attesting a claim under a *registered*
predicate about a subject DID. Under the predicate
`https://registry.trustoverip.org/dtg/vsc/endorses/1` — a favourable claim
whose content the community's own vocabulary defines — the statement is a
**Verifiable Endorsement Credential** (VEC); a community may also register
predicates of its own. It returns an
[`Endorsement`](../../../_shared/0.1/endorsement.schema.json), which embeds the
registry-wide
[`IssuedCredential`](../../../../credentials/_shared/0.1/credentials.schema.json)
receipt and adds the two VTC-specific parts: the `typeUri` and the allocated
`statusListIndex`.

### The credential issued

The family keeps its name, and its members theirs (`typeUri`, `claim`,
`endorsementId`), from when it issued `EndorsementCredential`s. That type no
longer exists in the DTG Credentials Core Specification: a statement's meaning is
its predicate, never a type string. The credential this task mints is:

- `@context` `["https://www.w3.org/ns/credentials/v2", "https://registry.trustoverip.org/dtg/context/v1"]`,
  and `type` `["VerifiableCredential", "DTGCredential", "StatementCredential"]`;
- `issuer` the community's DID, with `issuerScope` `public`;
- `credentialSubject.id` = `subjectDid`, `credentialSubject.predicate` =
  `typeUri`, and `credentialSubject.object.value` = `claim`;
- `validFrom` at issuance and, where `validitySeconds` is given, `validUntil`
  that many seconds later;
- a `credentialStatus` entry for revocation at the allocated `statusListIndex`;
- an `id`, reported in `endorsement.issued`.

```json
{
  "@context": ["https://www.w3.org/ns/credentials/v2", "https://registry.trustoverip.org/dtg/context/v1"],
  "id": "urn:uuid:0b6f2d4e-8a1c-4e3f-9b5d-7c2a1e0f3d01",
  "type": ["VerifiableCredential", "DTGCredential", "StatementCredential"],
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "issuerScope": "public",
  "validFrom": "2026-09-20T09:00:00Z",
  "validUntil": "2027-09-20T09:00:00Z",
  "credentialSubject": {
    "id": "did:webvh:QmCarolScid1:kernel-vtc.example:carol",
    "predicate": "https://registry.trustoverip.org/dtg/vsc/endorses/1",
    "object": {
      "value": { "skill": "kernel.memoryManagement", "level": "maintainer" }
    }
  },
  "credentialStatus": {
    "id": "https://kernel-vtc.example/status/revocation/1#4214",
    "type": "BitstringStatusListEntry",
    "statusPurpose": "revocation",
    "statusListIndex": "4214",
    "statusListCredential": "https://kernel-vtc.example/status/revocation/1"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid:kernel-vtc.example#key-1",
    "created": "2026-09-20T09:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z63jiSzsVJshBfyZwcr6nUopHo5M1QnBnWJHtwTpdNEFeD7KoX5rezJcGeoY8AVuTSo5Q3uH2KqMoEZk68qqGu3AR"
  }
}
```

The claim members and the status list URL are illustrative. A role is never
issued here: a role is a decision by the community, carried in a Verifiable
Authority Credential ([`vtc/vetting/vetters/grant`](../../../vetting/vetters/grant/0.1/spec.md)).

### The identity-verification credential

One `typeUri` is not a predicate. The reserved value
`IdentityVerificationCredential` asks this task for the community's
**identity-verification credential** (IDVC): the record that an administrator
verified, typically in person, that the member controls the DID in front of
them. A community that vets its own members this way acts as its own
identity-verification provider, lists its own DID in `acceptedIdvps`, and
accepts the credential as personhood evidence in
[`vtc/members/personhood/assert`](../../../members/personhood/assert/0.1/spec.md).

The DTG Credentials Core Specification defines an IDVC as any W3C VC that
satisfies a community's identity-proofing requirements, and says IDVCs are
**not** `DTGCredential` subtypes. So this is the one credential this task mints
outside the DTG catalog, and deliberately so:

- `@context` `["https://www.w3.org/ns/credentials/v2"]` alone, and `type`
  `["VerifiableCredential", "IdentityVerificationCredential"]` — no
  `DTGCredential`, no DTG context, no `issuerScope`, no `predicate`;
- `issuer` the community's DID;
- `credentialSubject.id` = `subjectDid`, with the members of `claim` beside it
  (how the identity was verified, and by whom). `claim` **MUST NOT** carry
  `id`: the subject is `subjectDid`, never a value the caller supplies;
- `validFrom` at issuance and `validUntil` after it;
- a `credentialStatus` entry for revocation on the community's shared status
  list, so it is withdrawn with [`vtc/endorsements/revoke`](../../revoke/0.1/spec.md)
  like any other row;
- an `id`, reported in `endorsement.issued`.

```json
{
  "@context": ["https://www.w3.org/ns/credentials/v2"],
  "id": "urn:uuid:3d9a7c5e-1f2b-4a6d-8e0c-9b4f2a1d7e01",
  "type": ["VerifiableCredential", "IdentityVerificationCredential"],
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "validFrom": "2026-09-21T14:00:00Z",
  "validUntil": "2027-09-21T14:00:00Z",
  "credentialSubject": {
    "id": "did:webvh:QmAliceScid1:alice.example",
    "method": "inPerson",
    "verifiedBy": "did:webvh:QmDanaScid1:kernel-vtc.example:dana"
  },
  "credentialStatus": {
    "id": "https://kernel-vtc.example/status/revocation/1#4215",
    "type": "BitstringStatusListEntry",
    "statusPurpose": "revocation",
    "statusListIndex": "4215",
    "statusListCredential": "https://kernel-vtc.example/status/revocation/1"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid:kernel-vtc.example#key-1",
    "created": "2026-09-21T14:00:00Z",
    "proofPurpose": "assertionMethod",
    "proofValue": "z63jiSzsVJshBfyZwcr6nUopHo5M1QnBnWJHtwTpdNEFeD7KoX5rezJcGeoY8AVuTSo5Q3uH2KqMoEZk68qqGu3AR"
  }
}
```

The `claim` members shown are illustrative. `IdentityVerificationCredential` is
reserved: it is never registered through
[`vtc/endorsement-types/register`](../../../endorsement-types/register/0.1/spec.md),
which refuses it, and a consumer recognises it before it looks for a
registered predicate, so a caller cannot reach it through the statement path or
shadow it with a registration. The row the community records for it carries
`typeUri` `IdentityVerificationCredential`.

### Why this is not `vta/credentials/issue`

The minting *mechanism* is shared — and this task reuses it literally, via the
`credentials/_shared` component. The *trust operation* differs on three axes,
which is why it is a separate Trust Task rather than a variant of one:

1. **Third-party revocation-verifiability.** A statement is checked by a *foreign*
   community (see `vtc/auth/recognise`), so it MUST carry a published
   status-list slot a stranger can read. A `vta/credentials/*` share credential
   is verified once by its recipient and revoked by removing an ACL entry —
   there is no published bit. One URI cannot promise both.
2. **Approval plane.** Endorsement issuance may run on the community's
   *self-management* plane, gated by policy with no human in the loop.
   `vta/credentials/issue` is on the *management* plane, gated by operator
   step-up.
3. **Governance gating.** `typeUri` MUST already be registered via
   `vtc/endorsement-types/register`, or be the one reserved value; the VTA task treats `credentialType` as a
   free string.

## Conformance

Producer: supply `subjectDid`, a registered predicate IRI or the reserved `IdentityVerificationCredential` as `typeUri`, and a non-empty `claim`.

Consumer:

1. Verify the community-admin **or** issuer capability against the live ACL —
   do not infer it from a cached token role.
2. Where `typeUri` is the reserved `IdentityVerificationCredential`, mint the
   identity-verification credential described above instead of a statement,
   and reject a `claim` that carries `id` with `claimSchemaViolation`; the
   rest of this item does not apply. Otherwise reject a `typeUri` that is not
   a registered predicate — `role:vetter` included, which only
   [`vtc/vetting/vetters/grant`](../../../vetting/vetters/grant/0.1/spec.md)
   issues — with `typeNotRegistered`. Reject a
   registered predicate whose profile requires `taskContext`, or names an
   issuer other than the community, with `predicateNotIssuable` — this task
   carries no task citation, and the community may make only the statements
   it can truthfully make itself (`vetted/1`, whose issuer is an eligible
   vetter, is one it cannot).
3. Reject a `claim` over 8 KiB with `claimTooLarge`; when the registered predicate
   declares a `claimSchema`, validate `claim` against it and reject a failure
   with `claimSchemaViolation`.
4. Allocate the next free slot on the shared Revocation status list
   (`statusListExhausted` if none remains), sign the credential as
   described above, persist the row, and return the full `Endorsement` and the
   `credential`.

The status-list slot MUST be allocated durably before the credential is
returned — a credential handed out with no reachable revocation slot cannot be
revoked.

## Security & Privacy

**Issuer-class mutation.** The community's signature is applied to a claim a
third party will rely on, so the framework proof is REQUIRED and the live-ACL
check is mandatory (a JWT role alone is insufficient — a token minted before an
issuer grant was withdrawn must not still issue).

Slot allocation is **not reclaimed** on revocation, by design: reusing a slot
would silently un-revoke a credential for any verifier holding a cached list.
