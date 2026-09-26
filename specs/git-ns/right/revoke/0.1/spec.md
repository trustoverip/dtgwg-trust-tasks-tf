---
slug: git-ns/right/revoke
version: "0.1"
title: "Git Namespaces — Revoke Right"
summary: "The granter, an authority over the resource, or the subject itself revokes one recorded git right. The VTC refuses to leave a repository without an owner or a namespace without an admin, and withdraws the right from its Trust Registry and the forge."
status: draft
targetFrameworkVersion: "0.6.0"
category: governance
keywords:
  - git
  - forge
  - revocation
  - commit-signing
parties:
  - role: member
    requirement: REQUIRED
    member: issuer
    identifierScope: pairwise
  - role: VTC
    requirement: REQUIRED
    member: recipient
    identifierScope: public
proofRequirement:
  requirement: REQUIRED
  rationale: "A revocation changes whose commits a repository's CI check accepts. It must be attributable to the actor, and it is part of the audit history of the resource."
issuedAtRequirement:
  requirement: REQUIRED
  rationale: "A revocation replayed after the right was granted again would silently remove it. Placing the request in time is what lets the VTC refuse the replay."
sideEffects:
  level: mutating
  rationale: "Removes a right from the VTC's records, its Trust Registry and the forge's roles. Reversible by granting again with git-ns/right/grant."
exposure:
  discloses: metadata
  ingests: metadata
  actsAsSubject: false
  rationale: "The request carries a DID, a right, a resource and optional free text. The response returns the revoked record."
retention:
  class: durable
  rationale: "The revocation closes a grant in the resource's audit history and is kept with it."
errorCodes:
  - code: git-ns:escalation
    meaning: "No right the actor holds on this resource carries the authority to grant or revoke this right."
    retryable: false
  - code: git-ns:lastOwner
    meaning: "The change would leave the repository with no owner. A repository always has at least one."
    retryable: false
  - code: git-ns:lastAdmin
    meaning: "The change would leave the namespace with no `git.ns.admin`. A bound namespace always has at least one."
    retryable: false
  - code: git-ns:policyDenied
    meaning: "The community's git-namespace policy refused the request after the fixed rules passed."
    retryable: false
  - code: git-ns/right/revoke:notGranted
    meaning: "No live record matches this subject, right and resource. Implied rights are not records and cannot be revoked."
    retryable: false
related:
  - git-ns/right/grant
  - git-ns/view
  - git-ns/repo/transfer
  - git-trust/revoke
---

## Abstract

Revokes one recorded git right and withdraws it from the VTC's Trust Registry and, through the bridge, from the forge. The counterpart of [`git-ns/right/grant`](../../../../git-ns/right/grant/0.1/spec.md), whose [rights model](../../../../git-ns/right/grant/0.1/spec.md#the-rights-model) this specification relies on.

## Status of this Document

This specification is a **draft** ([SPEC §5.3](/SPEC.md#53-maturity-levels)). It targets framework version 0.6.0 and may change without a version bump while it remains a draft ([SPEC §5.2](/SPEC.md#52-compatibility-rules)).

## Conformance

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY** and **OPTIONAL** in this document are to be interpreted as described in [BCP 14](https://www.rfc-editor.org/info/bcp14) when, and only when, they appear in all capitals.

A conforming producer and consumer satisfy [SPEC §7.1 and §7.2](/SPEC.md#7-minimum-requirements) in addition to the requirements stated here.

## Authorization

*Declared under [SPEC §7.3](/SPEC.md#73-specification-requirements) item 15.*

A VTC **MUST** accept a revocation only from one of:

- **the granter** — the record's `grantedBy`, while still a member of the community and while still holding a right whose grant authority covers the right on this resource;
- **an authority over the resource** — a holder, explicit or implied, of a right whose grant authority covers the revoked right on this resource: an owner for `own`, `maintain` and `commit.sign` on their repository, a namespace admin for anything in the namespace;
- **the subject itself**, resigning its own right.

Anyone else is refused with `permissionDenied`, or with `git-ns:escalation` when they hold rights on the resource but none that covers this right. An owner therefore can revoke a co-owner, and cannot revoke a `git.commit.sign` granted on the whole namespace. The `proof` establishes who is acting, never that they may.

## Definitions

**`subject`**, **`right`**, **`resource`** — identify the record exactly. Revocation matches the record; it does not revoke a narrower right the subject holds inside `resource`, and it cannot revoke an implied right, which is not a record.

**`reason`** — the revoker's free text, kept in the VTC's audit record.

## Request

The actor sends the revocation to the VTC. See the top-level schema in [`payload.schema.json`](payload.schema.json). A conforming VTC:

1. Refuses a triple that matches no live record with `git-ns/right/revoke:notGranted`. A client that only needs the right to be gone **MAY** treat that code as success.
2. Checks the authorization above.
3. Refuses to remove the last explicit owner of a repository with `git-ns:lastOwner`, and the last explicit `git.ns.admin` of a bound namespace with `git-ns:lastAdmin`. This applies to resignations too.
4. Evaluates the community's policy, which may refuse with `git-ns:policyDenied` and may not override the rules above.
5. Removes the record, withdraws the right — and, where no other record still implies it, the explicit `git.commit.sign` published for it — from the registry projection, and queues the forge projection. The response **MUST NOT** be sent before the removal is durable.

### Bob withdraws Dan's commit right after the release

```json
{
  "id": "urn:uuid:e6110a8b-0429-4714-a8fd-a32011e31201",
  "type": "https://trusttasks.org/spec/git-ns/right/revoke/0.1",
  "threadId": "urn:uuid:e6110a8b-0429-4714-a8fd-a32011e31201",
  "issuer": "did:webvh:QmBobScid2:acme-vtc.example:bob",
  "recipient": "did:webvh:QmVtcScid7:acme-vtc.example",
  "issuedAt": "2026-10-30T09:00:00Z",
  "payload": {
    "subject": "did:webvh:QmDanScid4:dan.example",
    "right": "git.commit.sign",
    "resource": "github.com/acme/gadgets",
    "reason": "1.0 shipped"
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmBobScid2:acme-vtc.example:bob#key-1",
    "created": "2026-10-30T09:00:00Z",
    "proofPurpose": "authentication",
    "proofValue": "zxaAku4fwygNT9a9njHkiy5c6g6nGp6BF6JKqZ9nQH7zS9NXVbX1VSRaK3Ykm9PCXPBCfauX3J5CGHc9zC9eoBV"
  }
}
```

## Response

The VTC, now responding, returns the record as it stood when revoked, per the sub-schema reachable via `$anchor: "response"` in [`payload.schema.json`](payload.schema.json). Refusals use `trust-task-error`.

### The revoked record

```json
{
  "id": "urn:uuid:e6110a8b-0429-4714-a8fd-a32011e31202",
  "type": "https://trusttasks.org/spec/git-ns/right/revoke/0.1#response",
  "threadId": "urn:uuid:e6110a8b-0429-4714-a8fd-a32011e31201",
  "issuer": "did:webvh:QmVtcScid7:acme-vtc.example",
  "recipient": "did:webvh:QmBobScid2:acme-vtc.example:bob",
  "issuedAt": "2026-10-30T09:00:01Z",
  "payload": {
    "revoked": {
      "subject": "did:webvh:QmDanScid4:dan.example",
      "right": "git.commit.sign",
      "resource": "github.com/acme/gadgets",
      "grantedBy": "did:webvh:QmBobScid2:acme-vtc.example:bob",
      "grantedAt": "2026-09-23T11:00:01Z",
      "expiresAt": "2026-12-22T00:00:00Z",
      "reason": "External contributor for the 1.0 push"
    }
  },
  "proof": {
    "type": "DataIntegrityProof",
    "cryptosuite": "eddsa-jcs-2022",
    "verificationMethod": "did:webvh:QmVtcScid7:acme-vtc.example#key-1",
    "created": "2026-10-30T09:00:01Z",
    "proofPurpose": "authentication",
    "proofValue": "zbxqLJo8L25NAFufkrjX1Y38vbi5cfoFDyL7iHzsh1uPVDBLKm5FrwZjGGi6ei2UZqq4b2Z1tTNs2EqLMgmDemv"
  }
}
```

## Security & Privacy

### Data carried

The same triple as the grant, plus optional free text, which a revoker **MUST NOT** use for anything they would not show the resource's owners and namespace admins. The response returns the revoked record, `reason` included, to an actor who is by construction the granter, an authority over the resource, or the subject.

### Correlation

The VTC declares `identifierScope: public`: it is the registry authority whose published rights this withdraws, and verifiers must recognise the same identifier before and after — a pairwise one would leave them unable to tell whose right had gone. The actor declares `pairwise`. A revocation is visible to anyone who queried the registry before and after it.

### Retention

Durable. The revocation, who made it and when, stay in the VTC's audit history with the grant they end; a commit made while the right was live remains explicable after it is gone.

### Consent/purpose

The purpose is to end a right. The VTC **MUST NOT** use a revocation or its reason for anything else. Whether any revocation warrants a step-up is the VTC's policy ([SPEC §7.3](/SPEC.md#73-specification-requirements) item 13).
