---
slug: vtc/relationships/publish
version: "0.1"
title: VTC Relationships — Publish
summary: A member publishes a Verifiable Relationship Credential asserting a relationship to another community member.
status: draft
targetFrameworkVersion: "0.2"
category: governance
keywords:
  - vtc
  - relationships
  - vrc
  - publish
authors:
  - Glenn Gore (https://github.com/stormer78)
parties:
  - role: member
    requirement: REQUIRED
    member: issuer
  - role: community maintainer
    requirement: REQUIRED
    member: recipient
proofRequirement:
  requirement: REQUIRED
  rationale: The publisher is the document proof signer, which MUST equal the VRC's issuer; publishing a relationship attribution must be attributable.
sideEffects:
  level: mutating
  rationale: "Stores a relationship credential; reversible via relationships/revoke."
exposure:
  discloses: none
  actsAsSubject: false
errorCodes:
  - code: vtc/relationships/publish:vrcInvalid
    meaning: The VRC failed verification, or its issuer did not match the proof signer.
    retryable: false
  - code: vtc/relationships/publish:subjectNotMember
    meaning: The credentialSubject.id is not a member of this community.
    retryable: false
---

## Abstract

The **VTC Relationships — Publish** Trust Task records a member-issued Verifiable Relationship Credential (`vrc`) asserting a relationship to another member. The community verifies and stores it, returning its `id`, the `issuerDid`/`subjectDid`, and a `vrcSha256` for out-of-band integrity. Revoke via [`vtc/relationships/revoke`](../../revoke/0.1/).


An edge may be published under a **relationship DID** — an identifier scoped to
one counterparty, which names no member. The community still needs to know that
a member published it, and this document's own `proof` says only that a member
sent it, not that they control the credential's issuing key. `pop` closes that
gap: a proof of possession by the VRC's `issuer`, bound to this document's `id`
and to the credential's digest. Omit it when the VRC's `issuer` is this
document's issuer, where the document's own proof already establishes control.

Without it, any member who was ever handed a VRC could publish another party's
edge. Issuing a credential and publishing it to a community's trust graph are
different disclosures, and the second is the issuer's to make.
## Conformance

Producer: supply the signed `vrc`; its issuer MUST equal the proof signer. Carry a proof.

Consumer: verify the proof and the VRC; if the VRC fails or its issuer mismatches the signer, return `vrcInvalid`. Confirm the `credentialSubject.id` is a member (`subjectNotMember` otherwise). Store the VRC and return `{ id, issuerDid, subjectDid, vrcSha256 }`.


Producer: supply `pop` whenever the VRC's `issuer` is not this document's
issuer. Sign it with a verification method that `issuer` controls, and set
`documentId` to this document's `id` and `vrcDigestMultibase` to the digest of
the `vrc` being published.

Consumer: where the VRC's `issuer` is not the document issuer, require `pop`,
verify its proof against a verification method the VRC's `issuer` controls, and
confirm `documentId` matches this document and `vrcDigestMultibase` matches the
submitted credential. Reject the publication otherwise — an unbound or
absent authorization is not a weaker claim, it is no claim.

A consumer **MUST NOT** retain the authorization after verifying it. It exists
to answer one question at one moment, and storing it accumulates a durable link
between the publishing member and a relationship DID that names nobody — which
is the correlation publishing under a relationship DID exists to avoid.
## Security & Privacy

**Self-attested, attributable.** A relationship is a claim by its issuer, bound to the proof signer, so it cannot be forged in another member's name. The subject is a member but does not countersign here — the credential is the issuer's assertion.
