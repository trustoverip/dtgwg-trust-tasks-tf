---
slug: vtc/endorsements/issue
version: "0.1"
title: VTC Endorsements — Issue
summary: A community issues a Verifiable Statement Credential under a registered predicate to a subject — an endorsement under endorses/1, or a vetted/1 statement recording its own identity check — with a published status-list slot for revocation.
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
retention:
  class: durable
  rationale: >-
    The community keeps the row it records — the issuance receipt and the
    status-list slot — for the life of the credential, because
    vtc/endorsements/revoke names it. For a vetted/1 statement it also keeps
    this request and its response: the statement carries this request's `id`
    as `taskContext` and its task digest as `taskDigestMultibase`, so a
    verifier examining the check cannot recompute the digest without the
    document it was taken over.
outcomeEvidence:
  response: https://trusttasks.org/spec/vtc/endorsements/issue/0.1#response
  binding:
    id: /payload/credential/taskContext
    taskDigest: /payload/credential/taskDigestMultibase
  rationale: >-
    A vetted/1 statement the community issues for its own identity check cites
    this request as its taskContext. The exchange closes with this task's own
    response, whose credential names this document by id and binds it by task
    digest under the community's proof. Statements under other predicates
    carry no taskContext and cite nothing.
errorCodes:
  - code: vtc/endorsements/issue:typeNotRegistered
    meaning: "`typeUri` is not a predicate registered in this community's endorsement-type registry."
    retryable: false
  - code: vtc/endorsements/issue:predicateNotIssuable
    meaning: "`typeUri` is registered, but its predicate's profile does not let the community issue it through this task — it requires a `taskContext` naming an exchange other than this request (`witnessed/1`, `presented/1`), or names an issuer the community can never be."
    retryable: false
  - code: vtc/endorsements/issue:claimSchemaViolation
    meaning: "`claim` failed validation against the registered predicate's declared claimSchema — or, under `vetted/1`, against the registry's vetting schema, or its `community` is not this community's DID, or it carries a vetter-only member (`identityCommitment`, `cardDigestMultibase`, `declaredRelationship`)."
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
**Verifiable Endorsement Credential** (VEC). Under
`https://registry.trustoverip.org/dtg/vsc/vetted/1` it records an identity
check the community's own operators carried out. A community may also register
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
- under `vetted/1` only, `taskContext` and `taskDigestMultibase` citing this
  request, as [below](#a-communitys-own-identity-check-vetted1);
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

### A community's own identity check: `vetted/1`

When a community's own operators check a member's identity — at a desk, in
person, or on a call — the community records that check as a statement under
`https://registry.trustoverip.org/dtg/vsc/vetted/1`, issued under its own DID.
The predicate's profile permits this: its issuer is either a member the
community has made eligible to vet, or the community named in
`object.value.community` itself. The community mints no separate credential
type for the purpose, so every identity check in its graph has one shape,
whoever made it; a vetter's statement at the end of a
[`vetting/session`](../../../../vetting/session/0.1/spec.md) differs in its
issuer, its `issuerScope`, the exchange its `taskContext` names, and the
vetter-only members only it carries.

When `typeUri` is `https://registry.trustoverip.org/dtg/vsc/vetted/1`, the
credential is the statement described above, with these particulars:

- `issuer` is the community's DID, and `issuerScope` is `public`;
- `credentialSubject` is `{ "id": subjectDid, "predicate": "https://registry.trustoverip.org/dtg/vsc/vetted/1", "object": { "value": claim } }`;
- `claim` **MUST** validate against the registry's vetting schema,
  `https://registry.trustoverip.org/dtg/vsc/vetted/1/vetting.schema.json`,
  whether or not the registered row declares a `claimSchema`, and
  `claim.community` **MUST** be this community's DID — a `vetted/1` statement
  counts for the one community it names, and a community records only its own
  checks. `claim` **MUST NOT** carry any of the three vetter-only members —
  `identityCommitment`, `cardDigestMultibase` or `declaredRelationship`. The
  salt behind `identityCommitment` travels to vetters inside the Vetting Card
  and never to the community, which could otherwise test candidate values
  against every vetter's commitment; and `declaredRelationship` describes a
  vetter's tie to the subject, which means nothing when the issuer is the party
  weighing the statements. The community's statement and its vetters' are
  linked by their common subject, `credentialSubject.id`;
- `taskContext` is the `id` of **this request document**, and
  `taskDigestMultibase` is its *task digest* under
  [SPEC §4.9.3](/SPEC.md#493-binding-a-citation-to-the-document-it-names):
  `multibase(multihash(H(JCS(document ∖ proof))))`, computed over the
  `vtc/endorsements/issue` request document as the consumer received it, with its
  top-level `proof` removed and nothing else removed or added, SHA-256
  RECOMMENDED. The profile requires both. They name the exchange in which the
  check was recorded, and for the community's own check that exchange is this
  request — the innermost exchange that attests it
  ([SPEC §4.9.1](/SPEC.md#491-naming-an-exchange-from-outside-the-framework)),
  exactly as a vetter's statement cites its `vetting/session` document;
- a `credentialStatus` entry, so it is withdrawn with
  [`vtc/endorsements/revoke`](../../revoke/0.1/spec.md) like any other row,
  whose `typeUri` is the `vetted/1` IRI.

Statements under every other predicate this task issues carry no
`taskContext` or `taskDigestMultibase`.

**This statement is evidence, not a status.** It establishes that the
community states it carried out the described check, and nothing more: not that
the claimed identity is true, and not that the subject is a person or a member.
Personhood, like admission, is the community's decision, recorded on the
member's membership credential (VMC) by
[`vtc/members/personhood/assert`](../../../members/personhood/assert/0.1/spec.md);
a `vetted/1` statement, the community's own or a vetter's, is one input to it.

#### Example: a community desk check

Dana, an administrator of the kernel community, checks Alice's national
identity card at the community's desk and confirms in person that Alice
controls her DID. As a community-issued statement must, the claim carries none
of the vetter-only members (`identityCommitment`, `cardDigestMultibase`,
`declaredRelationship`). Dana records the check:

```json
{
  "id": "urn:uuid:4c8e2a6f-1d3b-4f7a-9e5c-0b2d4f6a8c01",
  "type": "https://trusttasks.org/spec/vtc/endorsements/issue/0.1",
  "issuer": "did:webvh:QmDanaScid1:kernel-vtc.example:dana",
  "recipient": "did:webvh:QmVtcScid:kernel-vtc.example",
  "issuedAt": "2026-09-21T14:00:00Z",
  "payload": {
    "subjectDid": "did:webvh:QmAliceScid1:alice.example",
    "typeUri": "https://registry.trustoverip.org/dtg/vsc/vetted/1",
    "claim": {
      "community": "did:webvh:QmVtcScid:kernel-vtc.example",
      "method": "inPerson",
      "documentClasses": ["nationalId"],
      "claimsVerified": ["name.legal"],
      "livenessConfirmed": true
    },
    "validitySeconds": 31536000
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmDanaScid1:kernel-vtc.example:dana#key-1",
    "created": "2026-09-21T14:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "z63jiSzsVJshBfyZwcr6nUopHo5M1QnBnWJHtwTpdNEFeD7KoX5rezJcGeoY8AVuTSo5Q3uH2KqMoEZk68qqGu3AR"
  }
}
```

The community issues the statement below and returns it as the response's
`credential`. Its `taskDigestMultibase` is the real task digest (SHA-256) of the
request as printed; its `proofValue` is illustrative.

```json
{
  "@context": ["https://www.w3.org/ns/credentials/v2", "https://registry.trustoverip.org/dtg/context/v1"],
  "id": "urn:uuid:3d9a7c5e-1f2b-4a6d-8e0c-9b4f2a1d7e01",
  "type": ["VerifiableCredential", "DTGCredential", "StatementCredential"],
  "issuer": "did:webvh:QmVtcScid:kernel-vtc.example",
  "issuerScope": "public",
  "validFrom": "2026-09-21T14:00:00Z",
  "validUntil": "2027-09-21T14:00:00Z",
  "taskContext": "urn:uuid:4c8e2a6f-1d3b-4f7a-9e5c-0b2d4f6a8c01",
  "taskDigestMultibase": "zQmUXBBKmqKHhXshFGeMh43p8HWPbmmPLSgvSAQkkRqLsHM",
  "credentialSubject": {
    "id": "did:webvh:QmAliceScid1:alice.example",
    "predicate": "https://registry.trustoverip.org/dtg/vsc/vetted/1",
    "object": {
      "value": {
        "community": "did:webvh:QmVtcScid:kernel-vtc.example",
        "method": "inPerson",
        "documentClasses": ["nationalId"],
        "claimsVerified": ["name.legal"],
        "livenessConfirmed": true
      }
    }
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

#### Outcome evidence

Because a `vetted/1` statement cites this request, this task's exchanges can be
cited as evidence under
[SPEC §7.3](/SPEC.md#73-specification-requirements) item 20, and the front
matter's `outcomeEvidence` makes the declaration: the outcome evidence is this
task's own `#response`, whose `credential` names the request by `taskContext`
and binds it by `taskDigestMultibase` under the community's proof. The
community retains the request and the response for the life of the statement,
and produces both where the statement is disputed: a verifier cannot recompute
the digest without the document it was taken over
([SPEC §4.9.4](/SPEC.md#494-evidence-that-a-cited-exchange-completed)).

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
   `vtc/endorsement-types/register`; the VTA task treats `credentialType` as a
   free string.

## Conformance

Producer: supply `subjectDid`, a registered predicate IRI as `typeUri`, and a non-empty `claim`. Under `vetted/1`, `claim` is a vetting object for this community, as [above](#a-communitys-own-identity-check-vetted1).

Consumer:

1. Verify the community-admin **or** issuer capability against the live ACL —
   do not infer it from a cached token role.
2. Reject a `typeUri` that is not a registered predicate — `role:vetter`
   included, which only
   [`vtc/vetting/vetters/grant`](../../../vetting/vetters/grant/0.1/spec.md)
   issues — with `typeNotRegistered`. Reject a registered predicate the
   community cannot truthfully issue through this task with
   `predicateNotIssuable`: one whose profile names an issuer the community can
   never be, or requires a `taskContext` naming an exchange other than this
   request. `witnessed/1` and `presented/1` are refused on the second ground —
   each cites the exchange witnessed or presented, which this request is not.
   `vetted/1` is issuable: its profile admits the community as issuer, and for
   the community's own check the exchange its `taskContext` names is this
   request.
3. Reject a `claim` over 8 KiB with `claimTooLarge`. Under `vetted/1`, validate
   `claim` against the registry's vetting schema
   (`https://registry.trustoverip.org/dtg/vsc/vetted/1/vetting.schema.json`) and
   reject a failure, a `claim.community` other than this community's DID, or a
   `claim` carrying `identityCommitment`, `cardDigestMultibase` or
   `declaredRelationship`, with `claimSchemaViolation`. Otherwise, when the
   registered predicate declares a `claimSchema`, validate `claim` against it
   and reject a failure with `claimSchemaViolation`.
4. Allocate the next free slot on the shared Revocation status list
   (`statusListExhausted` if none remains), sign the statement credential as
   described above — under `vetted/1`, with `taskContext` this request's `id`
   and `taskDigestMultibase` its task digest — persist the row, and return the
   full `Endorsement` and the `credential`.

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
